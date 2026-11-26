//! Pencil grain (`stylize.pencilGrain`): the paper's tooth showing through a pencil stroke. The
//! grain lies in document space, fixed to the canvas rather than to the object, so strokes drawn
//! over the same paper catch the same tooth; it is evaluated for each pixel at the resolution the
//! object is drawn at, so it stays sharp at any zoom and never shimmers between redraws (hash noise
//! of document-space lattice cells). Grain finer than a pixel fades to its average instead of
//! aliasing.

use vectorcraft_geom::Point;

use super::PixelSpace;

/// The grain's layers: each one's feature size as a share of the grain scale, and its weight.
const OCTAVES: [(f64, f64); 3] = [(1.0, 0.55), (0.47, 0.30), (0.23, 0.15)];

/// The grain value (0..1) where the tooth stops letting graphite through and where it takes it all.
const TOOTH: (f64, f64) = (0.38, 0.62);

/// A repeatable value in 0..1 for the lattice cell (`x`, `y`).
fn hash(x: i64, y: i64) -> f64 {
    let mut h = (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 30;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94D0_49BB_1331_11EB);
    h ^= h >> 31;
    (h >> 11) as f64 / (1u64 << 53) as f64
}

/// Smooth value noise at (`x`, `y`) in lattice units, 0..1.
fn value_noise(x: f64, y: f64) -> f64 {
    let (fx, fy) = (x.floor(), y.floor());
    let (ix, iy) = (fx as i64, fy as i64);
    let fade = |t: f64| t * t * (3.0 - 2.0 * t);
    let (u, v) = (fade(x - fx), fade(y - fy));
    let top = hash(ix, iy) + (hash(ix + 1, iy) - hash(ix, iy)) * u;
    let bottom = hash(ix, iy + 1) + (hash(ix + 1, iy + 1) - hash(ix, iy + 1)) * u;
    top + (bottom - top) * v
}

/// The paper's grain at document point `p`, 0..1, for grain of `scale` document units seen through
/// pixels `px` document units wide: each layer finer than about two pixels fades to its average.
pub(super) fn grain_at(p: Point, scale: f64, px: f64) -> f64 {
    let (mut sum, mut total) = (0.0, 0.0);
    for (k, (share, weight)) in OCTAVES.into_iter().enumerate() {
        let feature = scale * share;
        let seen = ((feature / px.max(1e-12)) - 1.0).clamp(0.0, 1.0);
        // Each layer is offset so the layers' lattices don't line up.
        let n = if seen > 0.0 { value_noise(p.x / feature + 17.3 * k as f64, p.y / feature - 31.7 * k as f64) } else { 0.5 };
        sum += weight * (seen * n + (1.0 - seen) * 0.5);
        total += weight;
    }
    sum / total
}

/// Let the paper's tooth show through the pixels: grain `scale` document units across, `strength`
/// 0..100 (how much of the stroke the tooth's hollows take away). Premultiplied RGBA, so each
/// pixel's channels are scaled together.
pub(super) fn pencil_grain(px: &mut [[u8; 4]], w: usize, h: usize, space: &PixelSpace, scale: f64, strength: f64) {
    let s = (strength / 100.0).clamp(0.0, 1.0);
    if s <= 0.0 || w == 0 {
        return;
    }
    let smooth = |t: f64| {
        let t = ((t - TOOTH.0) / (TOOTH.1 - TOOTH.0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };
    for y in 0..h {
        for x in 0..w {
            let pixel = &mut px[y * w + x];
            if pixel[3] == 0 {
                continue;
            }
            let p = space.to_doc * Point::new(x as f64 + 0.5, y as f64 + 0.5);
            let keep = 1.0 - s * (1.0 - smooth(grain_at(p, scale, space.px)));
            for c in pixel.iter_mut() {
                *c = (f64::from(*c) * keep).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grain_is_fixed_to_the_canvas_and_fades_below_a_pixel() {
        let p = Point::new(1234.5, -987.25);
        assert_eq!(grain_at(p, 0.6, 0.01), grain_at(p, 0.6, 0.01), "the same on every redraw");
        assert!((0.0..=1.0).contains(&grain_at(p, 0.6, 0.01)));
        assert_eq!(grain_at(p, 0.6, 100.0), 0.5, "far finer than a pixel: the average");
        let spread: Vec<f64> = (0..200).map(|i| grain_at(Point::new(i as f64 * 0.37, 5.0), 0.6, 0.01)).collect();
        let (lo, hi) = spread.iter().fold((1.0f64, 0.0f64), |(l, h), v| (l.min(*v), h.max(*v)));
        assert!(hi - lo > 0.3, "the grain varies: {lo}..{hi}");
    }
}

// GLOSSARY
// - `OCTAVES`: the grain layers' feature sizes (share of the scale) and weights.
// - `TOOTH`: the grain values between which the tooth goes from blocking to taking graphite.
// - `hash`: a repeatable 0..1 value per lattice cell.
// - `value_noise`: smooth value noise in lattice units.
// - `grain_at`: the paper's grain at a document point, fading layers finer than a pixel.
// - `pencil_grain`: let the paper's tooth show through premultiplied RGBA pixels.
