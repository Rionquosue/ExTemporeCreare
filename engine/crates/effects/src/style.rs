//! Photoshop-style layer styles (`uistandards.md` §7.2): Stroke, Color Overlay, Gradient Overlay
//! and Bevel & Emboss. Each is computed from the object's silhouette (its alpha at full Fill,
//! rendered at the view's resolution) and paints over the object, so a layer's Fill never fades
//! it. Sizes are in points; [`LayerStyle::paint`] takes the view's pixels per point.

use serde_json::{Value, json};
use vectorcraft_doc::color::{BlendMode, Color};

use crate::raster::{blend_mode, color_param};
use crate::util::{flag, num, text};

/// The effect ids of the layer styles.
pub const STYLE_EFFECTS: [&str; 4] = ["style.stroke", "style.colorOverlay", "style.gradientOverlay", "style.bevelEmboss"];

/// Where a Stroke style lies against the silhouette's edge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StrokePosition {
    Outside,
    Inside,
    Center,
}

/// Which Bevel & Emboss.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BevelStyle {
    /// Raised inside the edge.
    Inner,
    /// Raised outside the edge.
    Outer,
    /// Both: inside raised, outside lowered.
    Emboss,
    /// Inside raised, outside sunk back: the edge stamped in.
    Pillow,
}

/// One layer style, with its own blend mode and opacity (0..1).
#[derive(Clone, Debug, PartialEq)]
pub enum LayerStyle {
    Stroke { size: f64, position: StrokePosition, color: Color, mode: BlendMode, opacity: f32 },
    ColorOverlay { color: Color, mode: BlendMode, opacity: f32 },
    /// Stops at 0..1 across the silhouette's bounds along `angle` (degrees, counter-clockwise from
    /// the right, as Photoshop's).
    GradientOverlay { stops: Vec<(f64, Color)>, angle: f64, reverse: bool, mode: BlendMode, opacity: f32 },
    BevelEmboss {
        style: BevelStyle,
        /// How steep the bevel looks (0.01..10, Photoshop's Depth / 100).
        depth: f64,
        size: f64,
        soften: f64,
        /// Light from below (Direction Down).
        down: bool,
        angle: f64,
        altitude: f64,
        highlight: (Color, BlendMode, f32),
        shadow: (Color, BlendMode, f32),
    },
}

/// A colour parameter named `key`.
fn color_key(p: &Value, key: &str, default: Color) -> Color {
    color_param(&json!({ "color": p.get(key).cloned().unwrap_or(Value::Null) }), default)
}

fn opacity_key(p: &Value, key: &str, default: f64) -> f32 {
    (num(p, key, default) / 100.0).clamp(0.0, 1.0) as f32
}

impl LayerStyle {
    /// The style effect `id` with parameters `p` (defaults merged in), or `None` for another id.
    pub fn parse(id: &str, p: &Value) -> Option<Self> {
        Some(match id {
            "style.stroke" => LayerStyle::Stroke {
                size: num(p, "size", 3.0).clamp(0.0, 250.0),
                position: match text(p, "position", "outside").to_ascii_lowercase().as_str() {
                    "inside" => StrokePosition::Inside,
                    "center" | "centre" => StrokePosition::Center,
                    _ => StrokePosition::Outside,
                },
                color: color_param(p, Color::BLACK),
                mode: blend_mode(text(p, "mode", "normal")),
                opacity: opacity_key(p, "opacity", 100.0),
            },
            "style.colorOverlay" => LayerStyle::ColorOverlay {
                color: color_param(p, Color::rgb(1.0, 0.0, 0.0)),
                mode: blend_mode(text(p, "mode", "normal")),
                opacity: opacity_key(p, "opacity", 100.0),
            },
            "style.gradientOverlay" => {
                let mut stops: Vec<(f64, Color)> = p
                    .get("stops")
                    .and_then(Value::as_array)
                    .map(|a| {
                        a.iter()
                            .map(|s| ((num(s, "location", 0.0) / 100.0).clamp(0.0, 1.0), color_param(s, Color::BLACK)))
                            .collect()
                    })
                    .unwrap_or_default();
                if stops.len() < 2 {
                    stops = vec![(0.0, Color::BLACK), (1.0, Color::WHITE)];
                }
                stops.sort_by(|a, b| a.0.total_cmp(&b.0));
                LayerStyle::GradientOverlay {
                    stops,
                    angle: num(p, "angle", 90.0),
                    reverse: flag(p, "reverse", false),
                    mode: blend_mode(text(p, "mode", "normal")),
                    opacity: opacity_key(p, "opacity", 100.0),
                }
            }
            "style.bevelEmboss" => LayerStyle::BevelEmboss {
                style: match text(p, "style", "innerBevel").to_ascii_lowercase().as_str() {
                    "outerbevel" => BevelStyle::Outer,
                    "emboss" => BevelStyle::Emboss,
                    "pillowemboss" => BevelStyle::Pillow,
                    _ => BevelStyle::Inner,
                },
                depth: (num(p, "depth", 100.0) / 100.0).clamp(0.01, 10.0),
                size: num(p, "size", 5.0).clamp(0.0, 250.0),
                soften: num(p, "soften", 0.0).clamp(0.0, 16.0),
                down: text(p, "direction", "up").eq_ignore_ascii_case("down"),
                angle: num(p, "angle", 120.0),
                altitude: num(p, "altitude", 30.0).clamp(0.0, 90.0),
                highlight: (
                    color_key(p, "highlightColor", Color::WHITE),
                    blend_mode(text(p, "highlightMode", "screen")),
                    opacity_key(p, "highlightOpacity", 75.0),
                ),
                shadow: (
                    color_key(p, "shadowColor", Color::BLACK),
                    blend_mode(text(p, "shadowMode", "multiply")),
                    opacity_key(p, "shadowOpacity", 75.0),
                ),
            },
            _ => return None,
        })
    }

    /// How far it paints beyond the object's edge, in points.
    pub fn outset(&self) -> f64 {
        match self {
            LayerStyle::Stroke { size, position: StrokePosition::Outside, .. } => *size,
            LayerStyle::Stroke { size, position: StrokePosition::Center, .. } => size / 2.0,
            LayerStyle::BevelEmboss { style: BevelStyle::Outer | BevelStyle::Emboss | BevelStyle::Pillow, size, soften, .. } => size + soften,
            _ => 0.0,
        }
    }

    /// The style's paint over a `w`×`h` silhouette `alpha` (row-major, 0..255) at `px` pixels a
    /// point: layers of premultiplied RGBA pixels, each with the blend mode it composites with.
    /// `rgb` turns a colour into 8-bit sRGB.
    pub fn paint(&self, alpha: &[u8], w: usize, h: usize, px: f64, rgb: &dyn Fn(&Color) -> [u8; 3]) -> Vec<(BlendMode, Vec<u8>)> {
        let n = w * h;
        if alpha.len() < n || n == 0 {
            return vec![];
        }
        let a = |i: usize| alpha[i] as f32 / 255.0;
        match self {
            LayerStyle::Stroke { size, position, color, mode, opacity } => {
                let s = (size * px) as f32;
                let (outer, inner) = match position {
                    StrokePosition::Outside => (s, 0.0),
                    StrokePosition::Inside => (0.0, s),
                    StrokePosition::Center => (s / 2.0, s / 2.0),
                };
                let out_d = if outer > 0.0 { distance(alpha, w, h, true) } else { vec![] };
                let in_d = if inner > 0.0 { distance(alpha, w, h, false) } else { vec![] };
                let c = rgb(color);
                let px: Vec<u8> = (0..n)
                    .flat_map(|i| {
                        let mut cov = 0.0f32;
                        if outer > 0.0 {
                            cov = cov.max((outer + 0.5 - out_d[i]).clamp(0.0, 1.0) * (1.0 - a(i)));
                        }
                        if inner > 0.0 {
                            cov = cov.max((inner + 0.5 - in_d[i]).clamp(0.0, 1.0) * a(i));
                        }
                        premul(c, cov * opacity)
                    })
                    .collect();
                vec![(*mode, px)]
            }
            LayerStyle::ColorOverlay { color, mode, opacity } => {
                let c = rgb(color);
                vec![(*mode, (0..n).flat_map(|i| premul(c, a(i) * opacity)).collect())]
            }
            LayerStyle::GradientOverlay { stops, angle, reverse, mode, opacity } => {
                // Across the silhouette's bounds along the angle (y grows downwards on screen).
                let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
                for y in 0..h {
                    for x in 0..w {
                        if alpha[y * w + x] > 0 {
                            (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
                        }
                    }
                }
                if x1 < x0 {
                    return vec![];
                }
                let r = angle.to_radians();
                let (dx, dy) = (r.cos(), -r.sin());
                let (cx, cy) = ((x0 + x1) as f64 / 2.0, (y0 + y1) as f64 / 2.0);
                let half = ((x1 - x0) as f64 * dx.abs() + (y1 - y0) as f64 * dy.abs()) / 2.0 + 0.5;
                let colors: Vec<(f64, [u8; 3])> = stops.iter().map(|(t, c)| (*t, rgb(c))).collect();
                let px: Vec<u8> = (0..n)
                    .flat_map(|i| {
                        let (x, y) = ((i % w) as f64, (i / w) as f64);
                        let mut t = (((x - cx) * dx + (y - cy) * dy) / half * 0.5 + 0.5).clamp(0.0, 1.0);
                        if *reverse {
                            t = 1.0 - t;
                        }
                        premul(gradient_at(&colors, t), a(i) * opacity)
                    })
                    .collect();
                vec![(*mode, px)]
            }
            LayerStyle::BevelEmboss { style, depth, size, soften, down, angle, altitude, highlight, shadow } => {
                let s = (size * px).max(1.0) as f32;
                let inside = distance(alpha, w, h, false);
                let outside = distance(alpha, w, h, true);
                // A height field: rising over `s` pixels from the edge.
                let mut height: Vec<f32> = (0..n)
                    .map(|i| {
                        let up = (inside[i] / s).min(1.0) * a(i);
                        let ring = (1.0 - (outside[i] / s).min(1.0)) * (1.0 - a(i));
                        match style {
                            BevelStyle::Inner => up,
                            BevelStyle::Outer => a(i) + ring,
                            BevelStyle::Emboss => 0.5 + up * 0.5 - ring * 0.5 * (1.0 - a(i)),
                            BevelStyle::Pillow => up - ring,
                        }
                    })
                    .collect();
                let blur = (soften * px).round() as usize + 1;
                box_blur(&mut height, w, h, blur);
                let (az, el) = (angle.to_radians(), altitude.to_radians());
                let sign = if *down { -1.0 } else { 1.0 };
                let light = [az.cos() * el.cos(), -az.sin() * el.cos(), el.sin()];
                let k = (*depth as f32) * s;
                let mut hi = vec![0u8; n * 4];
                let mut lo = vec![0u8; n * 4];
                let (hc, lc) = (rgb(&highlight.0), rgb(&shadow.0));
                // Where the style shows: inside for Inner, the outer ring for Outer, both else.
                let reach = |i: usize| match style {
                    BevelStyle::Inner => a(i),
                    BevelStyle::Outer => (1.0 - a(i)) * f32::from(outside[i] <= s + 1.0),
                    _ => a(i).max(f32::from(outside[i] <= s + 1.0)),
                };
                for y in 0..h {
                    for x in 0..w {
                        let i = y * w + x;
                        let at = |xx: usize, yy: usize| height[yy * w + xx];
                        let gx = (at((x + 1).min(w - 1), y) - at(x.saturating_sub(1), y)) * 0.5;
                        let gy = (at(x, (y + 1).min(h - 1)) - at(x, y.saturating_sub(1))) * 0.5;
                        let (nx, ny, nz) = (-gx * k * sign, -gy * k * sign, 1.0f32);
                        let len = (nx * nx + ny * ny + nz * nz).sqrt();
                        let lit = (nx * light[0] as f32 + ny * light[1] as f32 + nz * light[2] as f32) / len - light[2] as f32;
                        let cover = reach(i);
                        let (up, dn) = ((lit * 2.0).clamp(0.0, 1.0), (-lit * 2.0).clamp(0.0, 1.0));
                        hi[i * 4..i * 4 + 4].copy_from_slice(&premul(hc, up * cover * highlight.2));
                        lo[i * 4..i * 4 + 4].copy_from_slice(&premul(lc, dn * cover * shadow.2));
                    }
                }
                vec![(shadow.1, lo), (highlight.1, hi)]
            }
        }
    }
}

/// `c` at coverage `cov` (0..1), premultiplied RGBA.
fn premul(c: [u8; 3], cov: f32) -> [u8; 4] {
    let a = cov.clamp(0.0, 1.0);
    let m = |v: u8| (v as f32 * a + 0.5) as u8;
    [m(c[0]), m(c[1]), m(c[2]), (a * 255.0 + 0.5) as u8]
}

/// The colour of `stops` (sorted, at 0..1) at `t`.
fn gradient_at(stops: &[(f64, [u8; 3])], t: f64) -> [u8; 3] {
    let Some(first) = stops.first() else { return [0, 0, 0] };
    if t <= first.0 {
        return first.1;
    }
    for pair in stops.windows(2) {
        let ((t0, c0), (t1, c1)) = (pair[0], pair[1]);
        if t <= t1 {
            let u = if t1 > t0 { (t - t0) / (t1 - t0) } else { 1.0 };
            let mix = |a: u8, b: u8| (a as f64 + (b as f64 - a as f64) * u).round() as u8;
            return [mix(c0[0], c1[0]), mix(c0[1], c1[1]), mix(c0[2], c1[2])];
        }
    }
    stops[stops.len() - 1].1
}

/// Each pixel's distance (pixels) to the nearest pixel inside the silhouette (`to_inside`, alpha at
/// least half), or to the nearest outside it: exact Euclidean (Felzenszwalb & Huttenlocher).
pub fn distance(alpha: &[u8], w: usize, h: usize, to_inside: bool) -> Vec<f32> {
    const FAR: f32 = 1e12;
    let mut grid: Vec<f32> = alpha[..w * h].iter().map(|a| if (*a >= 128) == to_inside { 0.0 } else { FAR }).collect();
    let mut f = vec![0f32; w.max(h)];
    let mut d = vec![0f32; w.max(h)];
    let mut v = vec![0usize; w.max(h)];
    let mut z = vec![0f32; w.max(h) + 1];
    for x in 0..w {
        for y in 0..h {
            f[y] = grid[y * w + x];
        }
        dt1d(&f[..h], &mut d[..h], &mut v, &mut z);
        for y in 0..h {
            grid[y * w + x] = d[y];
        }
    }
    for y in 0..h {
        f[..w].copy_from_slice(&grid[y * w..(y + 1) * w]);
        dt1d(&f[..w], &mut d[..w], &mut v, &mut z);
        grid[y * w..(y + 1) * w].copy_from_slice(&d[..w]);
    }
    grid.iter().map(|g| g.sqrt()).collect()
}

/// The 1D squared distance transform of `f` into `d`.
fn dt1d(f: &[f32], d: &mut [f32], v: &mut [usize], z: &mut [f32]) {
    let n = f.len();
    if n == 0 {
        return;
    }
    let mut k = 0usize;
    v[0] = 0;
    z[0] = f32::NEG_INFINITY;
    z[1] = f32::INFINITY;
    for q in 1..n {
        loop {
            let p = v[k];
            let s = ((f[q] + (q * q) as f32) - (f[p] + (p * p) as f32)) / (2.0 * q as f32 - 2.0 * p as f32);
            if s <= z[k] && k > 0 {
                k -= 1;
                continue;
            }
            if s <= z[k] {
                // k == 0: replace the first parabola.
                v[0] = q;
                z[0] = f32::NEG_INFINITY;
                z[1] = f32::INFINITY;
                break;
            }
            k += 1;
            v[k] = q;
            z[k] = s;
            z[k + 1] = f32::INFINITY;
            break;
        }
    }
    k = 0;
    for q in 0..n {
        while z[k + 1] < q as f32 {
            k += 1;
        }
        let p = v[k];
        d[q] = (q as f32 - p as f32).powi(2) + f[p];
    }
}

/// Blur `data` (`w`×`h`) with a box `r` pixels wide, rows then columns.
fn box_blur(data: &mut [f32], w: usize, h: usize, r: usize) {
    if r <= 1 {
        return;
    }
    let half = r / 2;
    let mut line = vec![0f32; w.max(h)];
    for y in 0..h {
        line[..w].copy_from_slice(&data[y * w..(y + 1) * w]);
        for x in 0..w {
            let (a, b) = (x.saturating_sub(half), (x + half).min(w - 1));
            data[y * w + x] = line[a..=b].iter().sum::<f32>() / (b - a + 1) as f32;
        }
    }
    for x in 0..w {
        for y in 0..h {
            line[y] = data[y * w + x];
        }
        for y in 0..h {
            let (a, b) = (y.saturating_sub(half), (y + half).min(h - 1));
            data[y * w + x] = line[a..=b].iter().sum::<f32>() / (b - a + 1) as f32;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(w: usize, h: usize, x0: usize, x1: usize) -> Vec<u8> {
        (0..w * h).map(|i| if (x0..x1).contains(&(i % w)) && (x0..x1).contains(&(i / w)) { 255 } else { 0 }).collect()
    }

    #[test]
    fn distances_are_euclidean() {
        let a = square(20, 20, 5, 15);
        let d = distance(&a, 20, 20, true);
        assert_eq!(d[10 * 20 + 10], 0.0);
        assert!((d[10 * 20 + 2] - 3.0).abs() < 1e-4);
        assert!((d[2 * 20 + 2] - (18.0f32).sqrt()).abs() < 1e-4);
    }

    #[test]
    fn styles_paint_where_they_should() {
        let a = square(40, 40, 10, 30);
        let rgb = |c: &Color| {
            let [r, g, b] = c.to_rgb();
            [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]
        };
        let stroke = LayerStyle::parse("style.stroke", &json!({"size": 3})).unwrap();
        let s = &stroke.paint(&a, 40, 40, 1.0, &rgb)[0].1;
        assert_eq!(s[(20 * 40 + 8) * 4 + 3], 255, "outside the edge");
        assert_eq!(s[(20 * 40 + 20) * 4 + 3], 0, "not inside");
        assert_eq!(s[(20 * 40 + 3) * 4 + 3], 0, "not beyond its size");
        let overlay = LayerStyle::parse("style.colorOverlay", &json!({"color": "#00ff00"})).unwrap();
        let o = &overlay.paint(&a, 40, 40, 1.0, &rgb)[0].1;
        assert_eq!((o[(20 * 40 + 20) * 4 + 1], o[(5 * 40 + 5) * 4 + 3]), (255, 0));
        let gradient = LayerStyle::parse("style.gradientOverlay", &json!({"angle": 0})).unwrap();
        let g = &gradient.paint(&a, 40, 40, 1.0, &rgb)[0].1;
        assert!(g[(20 * 40 + 11) * 4] < 40 && g[(20 * 40 + 28) * 4] > 215, "black on the left to white on the right");
        let bevel = LayerStyle::parse("style.bevelEmboss", &json!({"size": 4})).unwrap();
        let b = bevel.paint(&a, 40, 40, 1.0, &rgb);
        let (shadow, high) = (&b[0].1, &b[1].1);
        assert!(high[(11 * 40 + 11) * 4 + 3] > 0, "lit from the top left");
        assert!(shadow[(28 * 40 + 28) * 4 + 3] > 0, "shaded at the bottom right");
        assert_eq!(high[(20 * 40 + 20) * 4 + 3], 0, "flat in the middle");
    }
}

// GLOSSARY
// - STYLE_EFFECTS: the layer style effect ids.
// - StrokePosition, BevelStyle: Stroke's position, Bevel & Emboss's style.
// - LayerStyle: one layer style; parse(id, p), outset(), paint(alpha, w, h, px, rgb).
// - color_key(p, key, default), opacity_key(p, key, default): parameter helpers.
// - premul(c, cov), gradient_at(stops, t): pixel helpers.
// - distance(alpha, w, h, to_inside), dt1d(...): Euclidean distance transform.
// - box_blur(data, w, h, r): a box blur.
