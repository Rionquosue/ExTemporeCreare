//! Raster painting: the app's Brush tool paints into **splotches** (paint blocks), embedded images
//! on the canvas whose pixels it paints with round dabs, as a raster painting app's brush does.
//! Nothing it paints is vector art.
//!
//! - A splotch is an image object whose image key starts with [`BLOCK_KEY_PREFIX`]. It keeps the
//!   resolution it was started at: the zoom of the view, in pixels per point.
//! - When the pen goes down ([`begin`]) the stroke's splotch is ready at once: the view's frame
//!   plus a buffer ([`BUFFER_SHARE`] of it on each side). It is an existing splotch that covers the
//!   view at a resolution within [`ZOOM_TOLERANCE`] of the zoom, else a new one. A locked splotch is
//!   always the one painted; when the view goes past it, it grows to take in the view's frame and
//!   buffer (up to [`MAX_SIDE`] pixels a side and [`MAX_PIXELS`] in all).
//! - The brush's size is in screen pixels, or with a locked splotch in the splotch's own pixels, so
//!   it no longer changes with the zoom.
//! - The stroke is painted dab by dab as the pen moves ([`Stroke::add`]) into its own layer, which
//!   the app draws over the canvas while the stroke lasts ([`Stroke::layer_pixels`]). Within a
//!   stroke the dabs build up by their flow and never past its opacity. On release
//!   ([`Stroke::finish`]) the layer is laid over the splotch's pixels as one undo step.

use std::sync::Arc;

use serde_json::{Value, json};
use vectorcraft_color::Paint;
use vectorcraft_doc::{ImageBlob, ImageObject, Node, NodeId, NodeKind};
use vectorcraft_geom::{Affine, Point, Rect};

use super::{CommandSpec, Result, Session, bad, cmd, has_doc};

/// The image key prefix that marks a splotch.
pub const BLOCK_KEY_PREFIX: &str = "paint-block-";

/// The most pixels a splotch may have on a side.
pub const MAX_SIDE: u32 = 16_384;

/// The most pixels a splotch may have in all.
pub const MAX_PIXELS: u64 = 64 * 1024 * 1024;

/// The resolutions a new splotch may have, in pixels per point.
pub const RESOLUTIONS: (f64, f64) = (0.25, 64.0);

/// The buffer round the view's frame a splotch takes in: this share of the view's width (and
/// height) on each side. It also lets the view jitter that far while a splotch is reused.
pub const BUFFER_SHARE: f64 = 0.1;

/// How far the zoom may be from a splotch's resolution (as a share of it) for the splotch to be
/// painted again without the lock.
pub const ZOOM_TOLERANCE: f64 = 0.05;

pub fn specs() -> Vec<CommandSpec> {
    vec![cmd!(
        "paint.stroke",
        "Brush Stroke",
        [],
        None,
        "{points: [[x, y] | [x, y, pressure 0..1]…] (document points), view: {x, y, width, height} (the view's frame, document points), zoom: screen pixels per point (0.25–64 for a new splotch's resolution), locked?: id (the locked splotch: always painted, grown to take in the view), size: screen px (20; 0.5–5000, the dab's diameter; with locked: the splotch's pixels), hardness?: % (100), spacing?: % (25; 1–1000 of the dab's diameter), flow?: % (100; 1–100), opacity?: % (100; 1–100), pressureSize?: bool (true), color?: \"#rrggbb\" (default: the stroke colour, else the fill colour, else black)} paint one whole stroke as the Brush tool does (begin, add, finish), one undo step → {id, created: bool, width, height}",
        has_doc,
        stroke_cmd
    )]
}

/// A stroke's brush.
#[derive(Clone, Debug)]
struct Brush {
    size: f64,
    hardness: f64,
    spacing: f64,
    flow: f64,
    opacity: f64,
    pressure_size: bool,
    color: [f64; 3],
}

/// A number `key` of `p` within `range` (`default` when absent or null).
fn number(p: &Value, key: &str, default: f64, (lo, hi): (f64, f64), c: &str) -> Result<f64> {
    match p.get(key).filter(|v| !v.is_null()) {
        None => Ok(default),
        Some(v) => v.as_f64().filter(|n| (lo..=hi).contains(n)).ok_or_else(|| bad(c, format!("`{key}` must be a number from {lo} to {hi}"))),
    }
}

/// Points `[[x, y, pressure?]…]` with their pressure (1 when not given).
pub fn points_of(list: &Value, c: &str) -> Result<Vec<(Point, f64)>> {
    let list = list.as_array().ok_or_else(|| bad(c, "`points` must be a list of [x, y] or [x, y, pressure]"))?;
    list.iter()
        .map(|e| {
            let a = e.as_array().ok_or_else(|| bad(c, "each point is [x, y] or [x, y, pressure]"))?;
            let n = |i: usize| a.get(i).and_then(Value::as_f64).filter(|v| v.is_finite());
            let (x, y) = n(0).zip(n(1)).ok_or_else(|| bad(c, "a point's x and y must be numbers"))?;
            Ok((Point::new(x, y), n(2).unwrap_or(1.0).clamp(0.0, 1.0)))
        })
        .collect()
}

/// The colour `p` gives (`#rrggbb`), else the current stroke colour, else the fill colour, else black.
fn color(s: &Session, p: &Value, c: &str) -> Result<[f64; 3]> {
    if let Some(v) = p.get("color").filter(|v| !v.is_null()) {
        let hex = v.as_str().and_then(|t| t.strip_prefix('#')).filter(|h| h.len() == 6 && h.chars().all(|ch| ch.is_ascii_hexdigit()));
        let hex = hex.ok_or_else(|| bad(c, "`color` must be \"#rrggbb\""))?;
        let byte = |i: usize| f64::from(u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0)) / 255.0;
        return Ok([byte(0), byte(2), byte(4)]);
    }
    let solid = |paint: &Paint| match paint {
        Paint::Solid { color, .. } => Some(color.to_rgb()),
        _ => None,
    };
    let rgb = solid(&s.paint.stroke).or_else(|| solid(&s.paint.fill)).unwrap_or([0.0, 0.0, 0.0]);
    Ok(rgb.map(f64::from))
}

/// A rectangle `{x, y, width, height}` (width and height above 0).
fn rect_of(v: Option<&Value>, key: &str, c: &str) -> Result<Rect> {
    let v = v.ok_or_else(|| bad(c, format!("`{key}` is needed: {{x, y, width, height}}")))?;
    let n = |k: &str| v.get(k).and_then(Value::as_f64).filter(|n| n.is_finite());
    match (n("x"), n("y"), n("width").filter(|w| *w > 0.0), n("height").filter(|h| *h > 0.0)) {
        (Some(x), Some(y), Some(w), Some(h)) => Ok(Rect::new(x, y, x + w, y + h)),
        _ => Err(bad(c, format!("`{key}` must be {{x, y, width, height}}, with a size above 0"))),
    }
}

/// Whether `n` is a splotch: (its image key, pixel size, pixels → document map).
pub fn block_of(n: &Node) -> Option<(&str, u32, u32, Affine)> {
    match &n.kind {
        NodeKind::Image(im) if im.key.starts_with(BLOCK_KEY_PREFIX) && im.link.is_none() => Some((&im.key, im.width, im.height, im.xf)),
        _ => None,
    }
}

/// A splotch's pixels per point.
fn resolution_of(xf: Affine) -> f64 {
    let c = xf.as_coeffs();
    1.0 / c[0].hypot(c[1]).max(1e-12)
}

/// The document area a splotch of `w` × `h` pixels placed by `xf` covers.
fn area_of(w: u32, h: u32, xf: Affine) -> Rect {
    xf.transform_rect_bbox(Rect::new(0.0, 0.0, f64::from(w), f64::from(h)))
}

/// The topmost editable splotch that covers `view` at a resolution within [`ZOOM_TOLERANCE`] of `zoom`.
fn reusable(doc: &vectorcraft_doc::Document, view: Rect, zoom: f64) -> Option<NodeId> {
    let mut found = None;
    let eps = 1e-6;
    doc.walk(|n| {
        if let Some((_, w, h, xf)) = block_of(n)
            && doc.is_editable(n.id)
            && (resolution_of(xf) / zoom - 1.0).abs() <= ZOOM_TOLERANCE
        {
            let a = area_of(w, h, xf);
            if a.x0 <= view.x0 + eps && a.y0 <= view.y0 + eps && a.x1 + eps >= view.x1 && a.y1 + eps >= view.y1 {
                found = Some(n.id);
            }
        }
    });
    found
}

/// How much of a dab of `radius` pixels covers a pixel `d` pixels from its centre, at `hardness` 0..1.
fn dab_cover(d: f64, radius: f64, hardness: f64) -> f64 {
    if d >= radius + 0.5 {
        return 0.0;
    }
    let solid = radius * hardness;
    if hardness >= 0.999 || d <= solid {
        // The solid part, its edge antialiased over a pixel.
        return (radius - d + 0.5).clamp(0.0, 1.0);
    }
    let t = ((d - solid) / (radius - solid).max(1e-9)).clamp(0.0, 1.0);
    1.0 - t * t * (3.0 - 2.0 * t)
}

/// A splotch's decoded pixels kept between strokes, so painting the same splotch again doesn't
/// decode it again: its image key, the image bytes they were decoded from, and the pixels.
pub struct Decoded {
    pub key: String,
    pub bytes: Arc<Vec<u8>>,
    pub pixels: image::RgbaImage,
}

/// A stroke being painted ([`begin`]), from the pen going down to [`Stroke::finish`].
pub struct Stroke {
    /// The splotch painted, or `None` for a new one.
    target: Option<NodeId>,
    /// Its image key (`None`: new).
    key: Option<String>,
    /// Its pixels → document map, size, and pixels before the stroke (grown as needed).
    xf: Affine,
    w: u32,
    h: u32,
    base: image::RgbaImage,
    /// How much of each pixel the stroke covers, 0..1.
    cover: Vec<f32>,
    brush: Brush,
    /// The splotch's pixels per screen pixel of brush size.
    size_scale: f64,
    /// The last point painted (splotch pixels, pressure) and how far past its last dab it is.
    last: Option<(Point, f64)>,
    carry: f64,
    /// The pixels the stroke has painted so far: x0, y0, x1, y1.
    painted: Option<[u32; 4]>,
}

/// Get a stroke ready as the pen goes down: its splotch (reused, locked or new, see the module
/// notes) with its pixels, `cache` saving their decoding when it holds them. `p` as `paint.stroke`
/// takes it, without `points`.
pub fn begin(s: &Session, p: &Value, cache: Option<&Decoded>) -> Result<Stroke> {
    const C: &str = "paint.stroke";
    let brush = Brush {
        size: number(p, "size", 20.0, (0.5, 5000.0), C)?,
        hardness: number(p, "hardness", 100.0, (0.0, 100.0), C)? / 100.0,
        spacing: number(p, "spacing", 25.0, (1.0, 1000.0), C)? / 100.0,
        flow: number(p, "flow", 100.0, (1.0, 100.0), C)? / 100.0,
        opacity: number(p, "opacity", 100.0, (1.0, 100.0), C)? / 100.0,
        pressure_size: p.get("pressureSize").and_then(Value::as_bool).unwrap_or(true),
        color: color(s, p, C)?,
    };
    let view = rect_of(p.get("view"), "view", C)?;
    let zoom = number(p, "zoom", 1.0, (1e-6, 1e6), C)?;
    let frame = view.inflate(view.width() * BUFFER_SHARE, view.height() * BUFFER_SHARE);
    let doc = &s.doc()?.doc;
    let locked = match p.get("locked").filter(|v| !v.is_null()) {
        Some(v) => {
            let id = NodeId(v.as_u64().ok_or_else(|| bad(C, "`locked` must be an object id"))?);
            doc.node(id).and_then(block_of).ok_or_else(|| bad(C, format!("{} is not a paint splotch", id.0)))?;
            Some(id)
        }
        None => None,
    };
    let target = locked.or_else(|| reusable(doc, view, zoom));
    let (key, old, xf) = match target.and_then(|id| doc.node(id)).and_then(block_of) {
        Some((key, w, h, xf)) => {
            let blob = doc.images.get(key).ok_or_else(|| bad(C, "the paint splotch's pixels are missing"))?;
            let img = match cache.filter(|c| c.key == key && Arc::ptr_eq(&c.bytes, &blob.bytes)) {
                Some(c) => c.pixels.clone(),
                None => image::load_from_memory(&blob.bytes).map_err(|e| bad(C, format!("the paint splotch can't be read: {e}")))?.to_rgba8(),
            };
            if img.width() != w || img.height() != h {
                return Err(bad(C, "the paint splotch's pixels don't match its size"));
            }
            (Some(key.to_string()), img, xf)
        }
        None => {
            let res = zoom.clamp(RESOLUTIONS.0, RESOLUTIONS.1);
            let origin = Point::new((frame.x0 * res).floor() / res, (frame.y0 * res).floor() / res);
            (None, image::RgbaImage::new(0, 0), Affine::translate(origin.to_vec2()) * Affine::scale(1.0 / res))
        }
    };
    // A new splotch takes in the frame; a locked one grows to take it in when the view goes past it.
    let needed = if target.is_none() || locked.is_some() { Some(xf.inverse().transform_rect_bbox(frame)) } else { None };
    let (x0, y0, x1, y1) = match needed {
        Some(b) => (b.x0.floor().min(0.0), b.y0.floor().min(0.0), b.x1.ceil().max(f64::from(old.width())), b.y1.ceil().max(f64::from(old.height()))),
        None => (0.0, 0.0, f64::from(old.width()), f64::from(old.height())),
    };
    let (w, h) = ((x1 - x0) as u64, (y1 - y0) as u64);
    if w > u64::from(MAX_SIDE) || h > u64::from(MAX_SIDE) || w * h > MAX_PIXELS {
        return Err(bad(C, "the paint splotch would grow too large: unlock it, or paint at a lower zoom"));
    }
    let (w, h) = (w.max(1) as u32, h.max(1) as u32);
    let base = if (x0, y0, u64::from(w), u64::from(h)) == (0.0, 0.0, u64::from(old.width()), u64::from(old.height())) {
        old
    } else {
        let mut grown = image::RgbaImage::new(w, h);
        image::imageops::replace(&mut grown, &old, -(x0 as i64), -(y0 as i64));
        grown
    };
    let xf = xf * Affine::translate((x0, y0));
    let size_scale = if locked.is_some() { 1.0 } else { resolution_of(xf) / zoom };
    Ok(Stroke { target, key, xf, w, h, base, cover: vec![0.0; (w as usize) * (h as usize)], brush, size_scale, last: None, carry: 0.0, painted: None })
}

impl Stroke {
    /// The splotch's document area, its size in pixels, and its id (`None`: new until the stroke
    /// finishes).
    pub fn frame(&self) -> (Rect, u32, u32, Option<NodeId>) {
        (area_of(self.w, self.h, self.xf), self.w, self.h, self.target)
    }

    /// A dab's radius in the splotch's pixels at `pressure`.
    fn radius(&self, pressure: f64) -> f64 {
        let b = &self.brush;
        (b.size * self.size_scale / 2.0 * if b.pressure_size { pressure } else { 1.0 }).max(0.5)
    }

    /// Paint one dab centred at `c` (splotch pixels).
    fn dab(&mut self, c: Point, r: f64) {
        let (w, h) = (self.w, self.h);
        let (bx0, by0) = ((c.x - r - 1.0).floor().max(0.0) as u32, (c.y - r - 1.0).floor().max(0.0) as u32);
        let (bx1, by1) = (((c.x + r + 1.0).ceil().max(0.0) as u32).min(w), ((c.y + r + 1.0).ceil().max(0.0) as u32).min(h));
        if bx0 >= bx1 || by0 >= by1 {
            return;
        }
        for y in by0..by1 {
            for x in bx0..bx1 {
                let d = (f64::from(x) + 0.5 - c.x).hypot(f64::from(y) + 0.5 - c.y);
                let v = dab_cover(d, r, self.brush.hardness) * self.brush.flow;
                if v > 0.0 {
                    let m = &mut self.cover[(y as usize) * (w as usize) + x as usize];
                    *m += (v as f32) * (1.0 - *m);
                }
            }
        }
        self.painted = Some(match self.painted {
            Some([a, b, c2, d]) => [a.min(bx0), b.min(by0), c2.max(bx1), d.max(by1)],
            None => [bx0, by0, bx1, by1],
        });
    }

    /// Paint the stroke on to `pts` (document points with pressure): dabs spaced along it from
    /// where it got to. → the splotch pixels this call painted (x, y, width, height), if any.
    pub fn add(&mut self, pts: &[(Point, f64)]) -> Option<[u32; 4]> {
        let before = self.painted;
        self.painted = None;
        let to_px = self.xf.inverse();
        for (q, pressure) in pts {
            let at = to_px * *q;
            let Some((from, pf)) = self.last else {
                let r = self.radius(*pressure);
                self.dab(at, r);
                self.last = Some((at, *pressure));
                continue;
            };
            let len = from.distance(at);
            let mut along = -self.carry;
            loop {
                let t = if len > 0.0 { (along.max(0.0) / len).min(1.0) } else { 1.0 };
                let step = (self.brush.spacing * 2.0 * self.radius(pf + (pressure - pf) * t)).max(0.5);
                if along + step > len {
                    self.carry = len - along;
                    break;
                }
                along += step;
                let u = along / len;
                let r = self.radius(pf + (pressure - pf) * u);
                self.dab(from.lerp(at, u), r);
            }
            self.last = Some((at, *pressure));
        }
        let now = self.painted;
        self.painted = match (before, now) {
            (Some([a, b, c, d]), Some([e, f, g, h])) => Some([a.min(e), b.min(f), c.max(g), d.max(h)]),
            (x, None) => x,
            (None, y) => y,
        };
        now.map(|[x0, y0, x1, y1]| [x0, y0, x1 - x0, y1 - y0])
    }

    /// The stroke's own layer over the splotch pixels `x`, `y`, `width` × `height` (clipped to the
    /// splotch): premultiplied RGBA, rows top to bottom, as an Android `ARGB_8888` bitmap holds
    /// them. The app draws it over the canvas while the stroke lasts.
    pub fn layer_pixels(&self, x: u32, y: u32, width: u32, height: u32) -> Vec<u8> {
        let mut out = vec![0u8; (width as usize) * (height as usize) * 4];
        let c = self.brush.color;
        for row in 0..height {
            let sy = y + row;
            if sy >= self.h {
                break;
            }
            for col in 0..width {
                let sx = x + col;
                if sx >= self.w {
                    break;
                }
                let a = f64::from(self.cover[(sy as usize) * (self.w as usize) + sx as usize]) * self.brush.opacity;
                if a <= 0.0 {
                    continue;
                }
                let o = ((row as usize) * (width as usize) + col as usize) * 4;
                let q = |v: f64| (v * 255.0).round().clamp(0.0, 255.0) as u8;
                out[o..o + 4].copy_from_slice(&[q(c[0] * a), q(c[1] * a), q(c[2] * a), q(a)]);
            }
        }
        out
    }

    /// Lay the stroke over the splotch's pixels and keep them in the document, a new splotch on the
    /// layer new art goes into, as one undo step named Brush. → `({id, created, width, height}, the
    /// pixels kept)`.
    pub fn finish(self, s: &mut Session) -> Result<(Value, Decoded)> {
        const C: &str = "paint.stroke";
        let Stroke { target, key, xf, w, h, mut base, cover, brush, .. } = self;
        for (i, m) in cover.iter().enumerate() {
            let a = f64::from(*m) * brush.opacity;
            if a <= 0.0 {
                continue;
            }
            let px = base.get_pixel_mut((i % w as usize) as u32, (i / w as usize) as u32);
            let da = f64::from(px[3]) / 255.0;
            let oa = a + da * (1.0 - a);
            for k in 0..3 {
                let dc = f64::from(px[k]) / 255.0;
                px[k] = ((brush.color[k] * a + dc * da * (1.0 - a)) / oa.max(1e-9) * 255.0).round().clamp(0.0, 255.0) as u8;
            }
            px[3] = (oa * 255.0).round().clamp(0.0, 255.0) as u8;
        }
        let mut png = Vec::new();
        base.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).map_err(|e| bad(C, format!("the paint splotch can't be written: {e}")))?;
        let st = s.doc()?;
        // A splotch whose pixels another object shares (a copy) takes pixels of its own.
        let shared = key.as_ref().is_some_and(|k| {
            let mut users = 0;
            st.doc.walk(|n| {
                if matches!(&n.kind, NodeKind::Image(im) if &im.key == k) {
                    users += 1;
                }
            });
            users > 1
        });
        let parent = st.target_parent()?;
        let bytes = Arc::new(png);
        let kept = bytes.clone();
        let (id, key) = s.edit("Brush", |d, sel| {
            let key = match key.filter(|_| !shared) {
                Some(k) => k,
                None => {
                    let mut k = format!("{BLOCK_KEY_PREFIX}{}", d.alloc_id().0);
                    while d.images.contains_key(&k) {
                        k.push('+');
                    }
                    k
                }
            };
            d.images.insert(key.clone(), ImageBlob { mime: "image/png".into(), bytes, proxy: None });
            let image = ImageObject { key: key.clone(), width: w, height: h, xf, link: None, placement: Default::default(), pixels: None };
            match target {
                Some(id) => {
                    let n = d.node_mut(id).ok_or(crate::EngineError::NoNode(id))?;
                    n.kind = NodeKind::Image(Box::new(image));
                    Ok((id, key))
                }
                None => {
                    let id = d.alloc_id();
                    let mut n = Node::new(id, NodeKind::Image(Box::new(image)));
                    n.name = Some("Paint".into());
                    d.insert(parent, usize::MAX, n)?;
                    sel.clear();
                    Ok((id, key))
                }
            }
        })?;
        let answer = json!({ "id": id.0, "created": target.is_none(), "width": w, "height": h });
        Ok((answer, Decoded { key, bytes: kept, pixels: base }))
    }
}

fn stroke_cmd(s: &mut Session, p: &Value) -> Result<Value> {
    let pts = points_of(p.get("points").unwrap_or(&Value::Null), "paint.stroke")?;
    if pts.is_empty() {
        return Err(bad("paint.stroke", "`points` must not be empty"));
    }
    let mut stroke = begin(s, p, None)?;
    stroke.add(&pts);
    Ok(stroke.finish(s)?.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel(s: &Session, id: u64, at: Point) -> [u8; 4] {
        let n = s.doc().unwrap().doc.node(NodeId(id)).unwrap();
        let (key, _, _, xf) = block_of(n).unwrap();
        let img = image::load_from_memory(&s.doc().unwrap().doc.images[key].bytes).unwrap().to_rgba8();
        let q = xf.inverse() * at;
        img.get_pixel(q.x as u32, q.y as u32).0
    }

    fn view(x: f64, y: f64) -> Value {
        json!({"x": x, "y": y, "width": 200, "height": 100})
    }

    #[test]
    fn a_splotch_covers_the_view_and_is_reused_or_locked() {
        let mut s = Session::new();
        s.execute("file.new", &json!({"width": 400, "height": 400})).unwrap();
        // Pen down: a splotch of the view and its buffer, at the zoom.
        let mut st = begin(&s, &json!({"view": view(0.0, 0.0), "zoom": 2, "size": 20, "color": "#ff0000"}), None).unwrap();
        let (area, w, h, target) = st.frame();
        assert_eq!((w, h, target), (480, 240, None), "200 × 100 pt and 10 % each side, at 2 px a point");
        assert!((area.x0 + 20.0).abs() < 1e-9 && (area.y0 + 10.0).abs() < 1e-9, "{area:?}");
        let painted = st.add(&[(Point::new(50.0, 50.0), 1.0), (Point::new(90.0, 50.0), 1.0)]).unwrap();
        assert!(painted[2] > 80, "dabs painted as the pen moved: {painted:?}");
        let layer = st.layer_pixels(0, 0, w, h);
        assert!(layer.chunks(4).any(|p| p == [255, 0, 0, 255]), "the live layer shows the stroke");
        let (first, decoded) = st.finish(&mut s).unwrap();
        let id = first["id"].as_u64().unwrap();
        assert_eq!(pixel(&s, id, Point::new(70.0, 50.0)), [255, 0, 0, 255]);
        // Panning a little at almost the same zoom paints the same splotch (decoding skipped).
        let mut again = begin(&s, &json!({"view": view(5.0, -3.0), "zoom": 2.05, "size": 20, "color": "#0000ff"}), Some(&decoded)).unwrap();
        assert_eq!(again.frame().3, Some(NodeId(id)));
        again.add(&[(Point::new(70.0, 80.0), 1.0)]);
        again.finish(&mut s).unwrap();
        assert_eq!(pixel(&s, id, Point::new(70.0, 50.0)), [255, 0, 0, 255], "earlier paint kept");
        // Zoomed in: a new splotch; locked: the old one, grown to the new view, the size its own pixels.
        let zoomed = begin(&s, &json!({"view": view(0.0, 0.0), "zoom": 4, "size": 20}), None).unwrap();
        assert_eq!(zoomed.frame().3, None);
        let locked = begin(&s, &json!({"view": view(300.0, 0.0), "zoom": 4, "size": 20, "locked": id}), None).unwrap();
        let (grown, ..) = locked.frame();
        assert_eq!(locked.frame().3, Some(NodeId(id)));
        assert!(grown.x1 >= 520.0 && grown.x0 <= -20.0 + 1e-9, "grown to take in the view: {grown:?}");
        assert!((locked.radius(1.0) - 10.0).abs() < 1e-9, "locked: 20 of the splotch's pixels across");
        // One undo step a stroke; the one-shot command paints the same way.
        s.execute("edit.undo", &json!({})).unwrap();
        assert_eq!(pixel(&s, id, Point::new(70.0, 80.0))[3], 0, "the second stroke undone");
        let one = s.execute("paint.stroke", &json!({"points": [[60, 20], [80, 20]], "view": view(0.0, 0.0), "zoom": 2, "hardness": 0, "opacity": 50})).unwrap();
        assert_eq!(one["id"].as_u64(), Some(id));
        assert!(s.execute("paint.stroke", &json!({"points": [], "view": view(0.0, 0.0)})).is_err());
    }

    #[test]
    fn a_dab_fades_by_its_hardness() {
        assert_eq!(dab_cover(0.0, 10.0, 0.0), 1.0);
        assert!(dab_cover(5.0, 10.0, 0.0) < 1.0 && dab_cover(5.0, 10.0, 0.0) > 0.0);
        assert_eq!(dab_cover(5.0, 10.0, 1.0), 1.0);
        assert_eq!(dab_cover(11.0, 10.0, 1.0), 0.0);
    }
}

// GLOSSARY
// - `BLOCK_KEY_PREFIX`: the image key prefix of a splotch, "paint-block-".
// - `MAX_SIDE`, `MAX_PIXELS`: the largest a splotch may grow (16 384 px a side, 64 Mi pixels).
// - `RESOLUTIONS`: a new splotch's resolution range, 0.25–64 px per point.
// - `BUFFER_SHARE`: the buffer round the view a splotch takes in, 10 % of the view on each side.
// - `ZOOM_TOLERANCE`: how far the zoom may be from a splotch's resolution for it to be reused, 5 %.
// - `specs`: the `paint.stroke` command.
// - `Brush`: a stroke's size, hardness, spacing, flow, opacity, pressure sizing and colour.
// - `number`: a checked number parameter.
// - `points_of`: points with pressure from JSON.
// - `color`: the stroke's colour (param, stroke, fill or black).
// - `rect_of`: a checked `{x, y, width, height}`.
// - `block_of`: a node's splotch fields, if it is one.
// - `resolution_of`: a splotch's pixels per point.
// - `area_of`: the document area a splotch covers.
// - `reusable`: the topmost splotch covering the view at about the zoom.
// - `dab_cover`: how much of a pixel a soft or hard dab covers.
// - `Decoded`: a splotch's decoded pixels kept between strokes.
// - `Stroke`: a stroke being painted (splotch, pixels, coverage, brush, position along it).
// - `begin`: get a stroke and its splotch ready as the pen goes down.
// - `Stroke::frame`: the splotch's area, size and id.
// - `Stroke::radius`: a dab's radius in splotch pixels.
// - `Stroke::dab`: paint one dab.
// - `Stroke::add`: paint the stroke on to more points → the pixels painted.
// - `Stroke::layer_pixels`: the stroke's live layer as premultiplied RGBA.
// - `Stroke::finish`: lay the stroke over the splotch and keep it, one undo step.
// - `stroke_cmd`: `paint.stroke`, a whole stroke at once.
