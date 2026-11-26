//! Raster painting: the app's Brush tool paints into **paint blocks**, embedded images on the
//! canvas whose pixels it paints with round dabs, as a raster painting app's brush does.
//!
//! - A block is an image object whose image key starts with [`BLOCK_KEY_PREFIX`]. It keeps the
//!   resolution it was started at (`resolution` pixels per point: the zoom when it was made) and
//!   grows to take in paint that runs past its edges (up to [`MAX_SIDE`] pixels a side and
//!   [`MAX_PIXELS`] in all).
//! - A stroke paints into the topmost block under its first point, or a new block on the layer new
//!   art goes into. Its dabs are spaced along the stroke ([`Brush::spacing`]), each soft as the
//!   hardness says; within a stroke they build up by their flow and never past the stroke's
//!   opacity, then the stroke is laid over the block's pixels. One undo step.

use serde_json::{Value, json};
use vectorcraft_color::Paint;
use vectorcraft_doc::{ImageBlob, ImageObject, Node, NodeId, NodeKind};
use vectorcraft_geom::{Affine, Point, Rect};

use super::{CommandSpec, Result, Session, bad, cmd, has_doc};

/// The image key prefix that marks a paint block.
pub const BLOCK_KEY_PREFIX: &str = "paint-block-";

/// The most pixels a block may have on a side.
pub const MAX_SIDE: u32 = 16_384;

/// The most pixels a block may have in all.
pub const MAX_PIXELS: u64 = 64 * 1024 * 1024;

/// The resolutions a new block may have, in pixels per point.
pub const RESOLUTIONS: (f64, f64) = (0.25, 64.0);

pub fn specs() -> Vec<CommandSpec> {
    vec![cmd!(
        "paint.stroke",
        "Brush Stroke",
        [],
        None,
        "{points: [[x, y] | [x, y, pressure 0..1]…] (document points), size: pt (10; 0.1–5000, the dab's diameter), hardness?: % (100; 0–100, how much of the dab is solid before it fades), spacing?: % (25; 1–1000 of the dab's diameter between dabs), flow?: % (100; 1–100, how much each dab adds), opacity?: % (100; 1–100, the most the stroke covers), pressureSize?: bool (true: the pen's pressure scales each dab), color?: \"#rrggbb\" (default: the stroke colour, else the fill colour, else black), resolution?: pixels per point (1; 0.25–64, for a new block: the zoom it is started at), block?: id (the block to paint into; default the topmost block under the first point, else a new one)} paint one stroke into a paint block, growing it to take in the paint, one undo step → {id, created: bool, width, height}",
        has_doc,
        stroke
    )]
}

/// A stroke's brush.
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

/// The stroke's points with their pressure (1 when not given).
fn points(p: &Value, c: &str) -> Result<Vec<(Point, f64)>> {
    let list = p.get("points").and_then(Value::as_array).filter(|l| !l.is_empty()).ok_or_else(|| bad(c, "`points` must be a list of [x, y] or [x, y, pressure]"))?;
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

/// Whether `n` is a paint block: (its image key, pixel size, pixels → document map).
fn block_of(n: &Node) -> Option<(&str, u32, u32, Affine)> {
    match &n.kind {
        NodeKind::Image(im) if im.key.starts_with(BLOCK_KEY_PREFIX) && im.link.is_none() => Some((&im.key, im.width, im.height, im.xf)),
        _ => None,
    }
}

/// The topmost editable paint block whose area holds `at`.
fn block_under(doc: &vectorcraft_doc::Document, at: Point) -> Option<NodeId> {
    let mut found = None;
    doc.walk(|n| {
        if let Some((_, w, h, xf)) = block_of(n)
            && doc.is_editable(n.id)
        {
            let inside = xf.inverse() * at;
            if inside.x >= 0.0 && inside.y >= 0.0 && inside.x <= f64::from(w) && inside.y <= f64::from(h) {
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

/// The dabs (centre in block pixels, radius in pixels) spaced along `pts` (block pixels, with pressure).
fn dabs(pts: &[(Point, f64)], brush: &Brush, scale: f64) -> Vec<(Point, f64)> {
    let radius = |pressure: f64| (brush.size * scale / 2.0 * if brush.pressure_size { pressure } else { 1.0 }).max(0.5);
    let mut out = vec![(pts[0].0, radius(pts[0].1))];
    let mut carry = 0.0;
    for w in pts.windows(2) {
        let ((a, pa), (b, pb)) = (w[0], w[1]);
        let len = a.distance(b);
        let mut at = carry;
        loop {
            let t = if len > 0.0 { at / len } else { 1.0 };
            let r = radius(pa + (pb - pa) * t.min(1.0));
            let step = (brush.spacing * 2.0 * r).max(0.5);
            if at + step > len {
                carry = at - len;
                break;
            }
            at += step;
            let u = at / len;
            out.push((a.lerp(b, u), radius(pa + (pb - pa) * u)));
        }
        carry = carry.min(0.0);
    }
    out
}

fn stroke(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "paint.stroke";
    let pts = points(p, C)?;
    let brush = Brush {
        size: number(p, "size", 10.0, (0.1, 5000.0), C)?,
        hardness: number(p, "hardness", 100.0, (0.0, 100.0), C)? / 100.0,
        spacing: number(p, "spacing", 25.0, (1.0, 1000.0), C)? / 100.0,
        flow: number(p, "flow", 100.0, (1.0, 100.0), C)? / 100.0,
        opacity: number(p, "opacity", 100.0, (1.0, 100.0), C)? / 100.0,
        pressure_size: p.get("pressureSize").and_then(Value::as_bool).unwrap_or(true),
        color: color(s, p, C)?,
    };
    let resolution = number(p, "resolution", 1.0, RESOLUTIONS, C)?;
    let st = s.doc()?;
    let doc = &st.doc;
    let target = match p.get("block").filter(|v| !v.is_null()) {
        Some(v) => {
            let id = NodeId(v.as_u64().ok_or_else(|| bad(C, "`block` must be an object id"))?);
            doc.node(id).and_then(block_of).ok_or_else(|| bad(C, format!("{} is not a paint block", id.0)))?;
            Some(id)
        }
        None => block_under(doc, pts[0].0),
    };
    // The block's pixels → document map and pixels, or a new empty block's at this resolution.
    let reach = brush.size / 2.0 + 1.0 / resolution;
    let stroke_box = pts.iter().fold(Rect::from_points(pts[0].0, pts[0].0), |r, (q, _)| r.union_pt(*q)).inflate(reach, reach);
    let (key, old, xf) = match target.and_then(|id| doc.node(id)).and_then(block_of) {
        Some((key, w, h, xf)) => {
            let blob = doc.images.get(key).ok_or_else(|| bad(C, "the paint block's pixels are missing"))?;
            let img = image::load_from_memory(&blob.bytes).map_err(|e| bad(C, format!("the paint block can't be read: {e}")))?.to_rgba8();
            if img.width() != w || img.height() != h {
                return Err(bad(C, "the paint block's pixels don't match its size"));
            }
            (Some(key.to_string()), img, xf)
        }
        None => {
            let origin = Point::new((stroke_box.x0 * resolution).floor() / resolution, (stroke_box.y0 * resolution).floor() / resolution);
            (None, image::RgbaImage::new(0, 0), Affine::translate(origin.to_vec2()) * Affine::scale(1.0 / resolution))
        }
    };
    let scale = 1.0 / xf.as_coeffs()[0].hypot(xf.as_coeffs()[1]).max(1e-12);
    // The block grown to take in the stroke, in its pixels.
    let inv = xf.inverse();
    let px_box = inv.transform_rect_bbox(stroke_box);
    let (x0, y0) = (px_box.x0.floor().min(0.0), px_box.y0.floor().min(0.0));
    let (x1, y1) = (px_box.x1.ceil().max(f64::from(old.width())), px_box.y1.ceil().max(f64::from(old.height())));
    let (w, h) = ((x1 - x0) as u64, (y1 - y0) as u64);
    if w > u64::from(MAX_SIDE) || h > u64::from(MAX_SIDE) || w * h > MAX_PIXELS {
        return Err(bad(C, "the paint block would grow too large: start a new block, or paint at a lower zoom"));
    }
    let (w, h) = (w as u32, h as u32);
    let (dx, dy) = ((-x0) as u32, (-y0) as u32);
    let mut img = image::RgbaImage::new(w, h);
    image::imageops::replace(&mut img, &old, i64::from(dx), i64::from(dy));
    let xf = xf * Affine::translate((x0, y0));
    // The stroke's coverage, built up dab by dab.
    let to_px = xf.inverse();
    let px_pts: Vec<(Point, f64)> = pts.iter().map(|(q, pr)| (to_px * *q, *pr)).collect();
    let mut cover = vec![0f32; (w as usize) * (h as usize)];
    for (c, r) in dabs(&px_pts, &brush, scale) {
        let (bx0, by0) = ((c.x - r - 1.0).floor().max(0.0) as u32, (c.y - r - 1.0).floor().max(0.0) as u32);
        let (bx1, by1) = (((c.x + r + 1.0).ceil() as u32).min(w), ((c.y + r + 1.0).ceil() as u32).min(h));
        for y in by0..by1 {
            for x in bx0..bx1 {
                let d = (f64::from(x) + 0.5 - c.x).hypot(f64::from(y) + 0.5 - c.y);
                let v = dab_cover(d, r, brush.hardness) * brush.flow;
                if v > 0.0 {
                    let m = &mut cover[(y as usize) * (w as usize) + x as usize];
                    *m += (v as f32) * (1.0 - *m);
                }
            }
        }
    }
    // The stroke laid over the block's pixels (straight alpha).
    for (i, m) in cover.iter().enumerate() {
        let a = f64::from(*m) * brush.opacity;
        if a <= 0.0 {
            continue;
        }
        let px = img.get_pixel_mut((i % w as usize) as u32, (i / w as usize) as u32);
        let da = f64::from(px[3]) / 255.0;
        let oa = a + da * (1.0 - a);
        for k in 0..3 {
            let dc = f64::from(px[k]) / 255.0;
            px[k] = ((brush.color[k] * a + dc * da * (1.0 - a)) / oa.max(1e-9) * 255.0).round().clamp(0.0, 255.0) as u8;
        }
        px[3] = (oa * 255.0).round().clamp(0.0, 255.0) as u8;
    }
    let mut png = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png).map_err(|e| bad(C, format!("the paint block can't be written: {e}")))?;
    // A block whose pixels another object shares (a copy) takes pixels of its own.
    let shared = key.as_ref().is_some_and(|k| {
        let mut users = 0;
        doc.walk(|n| {
            if matches!(&n.kind, NodeKind::Image(im) if &im.key == k) {
                users += 1;
            }
        });
        users > 1
    });
    let parent = st.target_parent()?;
    let created = target.is_none();
    let id = s.edit("Brush", |d, sel| {
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
        d.images.insert(key.clone(), ImageBlob::new("image/png", png));
        let image = ImageObject { key, width: w, height: h, xf, link: None, placement: Default::default(), pixels: None };
        match target {
            Some(id) => {
                let n = d.node_mut(id).ok_or(crate::EngineError::NoNode(id))?;
                n.kind = NodeKind::Image(Box::new(image));
                Ok(id)
            }
            None => {
                let id = d.alloc_id();
                let mut n = Node::new(id, NodeKind::Image(Box::new(image)));
                n.name = Some("Paint".into());
                d.insert(parent, usize::MAX, n)?;
                sel.clear();
                Ok(id)
            }
        }
    })?;
    Ok(json!({ "id": id.0, "created": created, "width": w, "height": h }))
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

    #[test]
    fn strokes_paint_one_block_that_grows_at_its_resolution() {
        let mut s = Session::new();
        s.execute("file.new", &json!({"width": 400, "height": 400})).unwrap();
        let first = s.execute("paint.stroke", &json!({"points": [[100, 100], [140, 100]], "size": 10, "color": "#ff0000", "resolution": 2})).unwrap();
        assert_eq!(first["created"], true);
        let id = first["id"].as_u64().unwrap();
        assert_eq!(pixel(&s, id, Point::new(120.0, 100.0)), [255, 0, 0, 255], "painted solid");
        let width = first["width"].as_u64().unwrap();
        assert!((100..=104).contains(&width), "40 pt + the 10 pt dab and a pixel each side, at 2 px a point: {width}");
        // A stroke from inside the block running past its edge grows it, keeping the first paint.
        let second = s.execute("paint.stroke", &json!({"points": [[130, 100], [130, 200]], "size": 10, "color": "#0000ff", "resolution": 8})).unwrap();
        assert_eq!((second["created"].as_bool(), second["id"].as_u64()), (Some(false), Some(id)));
        assert!(second["height"].as_u64().unwrap() > 200, "grown at the block's own 2 px a point");
        assert_eq!(pixel(&s, id, Point::new(110.0, 100.0)), [255, 0, 0, 255], "the first stroke kept");
        assert_eq!(pixel(&s, id, Point::new(130.0, 180.0)), [0, 0, 255, 255]);
        // Soft, half-opacity paint mixes over what is there.
        s.execute("paint.stroke", &json!({"points": [[120, 100], [120, 101]], "size": 10, "hardness": 0, "opacity": 50, "color": "#0000ff"})).unwrap();
        let mixed = pixel(&s, id, Point::new(120.0, 100.0));
        assert!(mixed[0] > 60 && mixed[2] > 60 && mixed[3] == 255, "{mixed:?}");
        // Each stroke is one undo step.
        s.execute("edit.undo", &json!({})).unwrap();
        s.execute("edit.undo", &json!({})).unwrap();
        assert_eq!(pixel(&s, id, Point::new(110.0, 100.0)), [255, 0, 0, 255]);
        assert!(s.execute("paint.stroke", &json!({"points": []})).is_err());
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
// - `BLOCK_KEY_PREFIX`: the image key prefix of a paint block, "paint-block-".
// - `MAX_SIDE`, `MAX_PIXELS`: the largest a block may grow (16 384 px a side, 64 Mi pixels).
// - `RESOLUTIONS`: a new block's resolution range, 0.25–64 px per point.
// - `specs`: the `paint.stroke` command.
// - `Brush`: a stroke's size, hardness, spacing, flow, opacity, pressure sizing and colour.
// - `number`: a checked number parameter.
// - `points`: the stroke's points with pressure.
// - `color`: the stroke's colour (param, stroke, fill or black).
// - `block_of`: a node's paint block fields, if it is one.
// - `block_under`: the topmost editable block under a point.
// - `dab_cover`: how much of a pixel a soft or hard dab covers.
// - `dabs`: the dabs spaced along a stroke.
// - `stroke`: `paint.stroke`: paint one stroke into a block, growing it, one undo step.
