//! Swatch libraries: read-only sets of swatches the library panel opens and adds from. The
//! built-in libraries are data files the engine reads at run time; this module keeps the library
//! type and the OKLab/OKLCH conversions the gradients use.


use serde::{Deserialize, Serialize};

use crate::{Color, Swatch, SwatchGroup};

/// A library: its name, its ungrouped swatches and its colour groups.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SwatchLibrary {
    pub name: String,
    #[serde(default)]
    pub swatches: Vec<Swatch>,
    #[serde(default)]
    pub groups: Vec<SwatchGroup>,
}

impl SwatchLibrary {
    /// Every swatch: the ungrouped ones first, then each group's in order.
    pub fn iter(&self) -> impl Iterator<Item = &Swatch> {
        self.swatches.iter().chain(self.groups.iter().flat_map(|g| g.swatches.iter()))
    }
    /// The number of swatches.
    pub fn len(&self) -> usize {
        self.swatches.len() + self.groups.iter().map(|g| g.swatches.len()).sum::<usize>()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn swatch(&self, name: &str) -> Option<&Swatch> {
        self.iter().find(|s| s.name == name)
    }
    pub fn group(&self, name: &str) -> Option<&SwatchGroup> {
        self.groups.iter().find(|g| g.name == name)
    }
}

// ---------- colour construction ----------

/// sRGB (0..=1, gamma encoded) of OKLab `(l, a, b)`, unclamped. A linear channel below
/// −0.0031308 is encoded with the curve's sign turned, as CSS Color 4's extended sRGB does
/// (<https://www.w3.org/TR/css-color-4/#color-conversion-code>).
pub(crate) fn oklab_to_srgb(l: f64, a: f64, b: f64) -> [f64; 3] {
    oklab_to_linear_srgb(l, a, b).map(|v| if v.abs() <= 0.003_130_8 { 12.92 * v } else { v.signum() * (1.055 * v.abs().powf(1.0 / 2.4) - 0.055) })
}

/// Linear sRGB of OKLab `(l, a, b)`, unclamped.
pub(crate) fn oklab_to_linear_srgb(l: f64, a: f64, b: f64) -> [f64; 3] {
    let l_ = l + 0.396_337_777_4 * a + 0.215_803_757_3 * b;
    let m_ = l - 0.105_561_345_8 * a - 0.063_854_172_8 * b;
    let s_ = l - 0.089_484_177_5 * a - 1.291_485_548_0 * b;
    let (l3, m3, s3) = (l_ * l_ * l_, m_ * m_ * m_, s_ * s_ * s_);
    [
        4.076_741_662_1 * l3 - 3.307_711_591_3 * m3 + 0.230_969_929_2 * s3,
        -1.268_438_004_6 * l3 + 2.609_757_401_1 * m3 - 0.341_319_396_5 * s3,
        -0.004_196_086_3 * l3 - 0.703_418_614_7 * m3 + 1.707_614_701_0 * s3,
    ]
}

/// OKLab `[l, a, b]` of a colour's display RGB.
pub fn to_oklab(c: &Color) -> [f64; 3] {
    srgb_to_oklab(c.to_rgb_uncalibrated())
}

/// OKLab `[l, a, b]` of sRGB `rgb` (0..=1, gamma encoded). A channel below −0.04045 is decoded
/// with the curve's sign turned, as CSS Color 4's extended sRGB does
/// (<https://www.w3.org/TR/css-color-4/#color-conversion-code>).
pub(crate) fn srgb_to_oklab(rgb: [f32; 3]) -> [f64; 3] {
    let [r, g, b] = rgb.map(|v| {
        let v = f64::from(v);
        if v.abs() <= 0.040_45 { v / 12.92 } else { v.signum() * ((v.abs() + 0.055) / 1.055).powf(2.4) }
    });
    let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
    let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
    let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();
    [
        0.210_454_255_3 * l + 0.793_617_785_0 * m - 0.004_072_046_8 * s,
        1.977_998_495_1 * l - 2.428_592_205_0 * m + 0.450_593_709_9 * s,
        0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766_0 * s,
    ]
}











// ---------- the libraries ----------




























