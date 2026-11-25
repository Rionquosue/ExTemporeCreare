//! The native `.etc` format's save options (`docs/Standardsdocs/filestandard.md`): path precision,
//! each artboard to a separate file, Include Linked Files and Embed ICC Profiles. Every native save
//! carries `mergedimage.png` and `Thumbnails/thumbnail.png`, which the format requires.

use std::borrow::Cow;
use std::collections::BTreeMap;

use etcfile::SaveOptions;
use serde::Deserialize;
use serde_json::Value;
use vectorcraft_color::cms;
use vectorcraft_doc::Document;

use super::super::{EngineError, Result, bad};
use super::{Format, FormatOption};

/// The options `document.save` and the native encoder read.
pub const OPTIONS: &[FormatOption] = &[
    FormatOption {
        name: "precision",
        ty: "integer",
        default: "null",
        description: "round path anchors and handles to this many decimal places, 0 to 9 (a smaller file; colors, transforms and type are kept exactly); null writes them exactly. Only the save that gives it rounds",
    },
    SEPARATE_ARTBOARDS,
    SEPARATE_RANGE,
    INCLUDE_LINKED,
    EMBED_PROFILES,
];

const SEPARATE_ARTBOARDS: FormatOption = FormatOption {
    name: "separateArtboards",
    ty: "boolean",
    default: "false",
    description: "also save each artboard (of range) to a file of its own beside this one, <name>-<artboard>.<ext>, holding that artboard and the art on it",
};
const SEPARATE_RANGE: FormatOption = FormatOption {
    name: "range",
    ty: "string",
    default: "null",
    description: "with separateArtboards: the artboards saved separately, 1-based such as \"1-3, 5\", or \"all\" (default)",
};
const INCLUDE_LINKED: FormatOption = FormatOption {
    name: "includeLinked",
    ty: "boolean",
    default: "false",
    description: "keep the linked files' own pixels in the file (they stay linked), not just a preview: the document shows them in full where the files can't be found",
};
const EMBED_PROFILES: FormatOption = FormatOption {
    name: "embedProfiles",
    ty: "boolean",
    default: "true",
    description: "carry the ICC profiles the document is tagged with that were loaded from files (Edit → Assign Profile), so the document keeps them where they aren't installed",
};

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct NativeOptions {
    precision: Option<Value>,
    include_linked: bool,
    embed_profiles: Option<bool>,
}

/// The options in `p` for saving `doc`, checked, with the profiles to embed (the previews are
/// left to [`encode`]).
pub(super) fn save_options(cmd: &str, f: &Format, doc: &Document, p: &Value) -> Result<SaveOptions> {
    let o: NativeOptions = super::encode::options(f, p)?;
    // Indented or compact is decided by [`encode_to`] on the document it writes.
    let mut so = SaveOptions {
        include_linked: o.include_linked,
        precision: o.precision.map(|v| precision(cmd, &v)).transpose()?,
        ..SaveOptions::default()
    };
    if o.embed_profiles.unwrap_or(true) {
        so.profiles = embedded_profiles(doc);
    }
    Ok(so)
}

/// The `precision` option `v`: a whole number of decimal places, 0 to
/// [`etcfile::MAX_PRECISION`]. Any other value, a string included, gets the range message.
fn precision(cmd: &str, v: &Value) -> Result<u8> {
    let (key, min, max) = ("precision", 0u8, etcfile::MAX_PRECISION);
    if let Some(x) = v.as_f64()
        && x.fract() == 0.0
        && (f64::from(min)..=f64::from(max)).contains(&x)
    {
        return Ok(x as u8);
    }
    let x = v.as_f64().map_or_else(|| v.to_string(), |x| x.to_string());
    Err(bad(cmd, format!("`{key}` must be a whole number between {min} and {max} (got {x})")))
}

/// The `.etc` file of `doc` with the options in `p`.
pub(super) fn encode(cmd: &str, f: &Format, doc: &Document, p: &Value) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    encode_to(cmd, f, doc, p, &mut out)?;
    Ok(out)
}

/// [`encode`], written to `w` ([`etcfile::save_to`]: give it a buffered writer).
pub(super) fn encode_to(cmd: &str, f: &Format, doc: &Document, p: &Value, w: impl std::io::Write) -> Result<()> {
    let mut so = save_options(cmd, f, doc, p)?;
    // Include Linked Files keeps placed documents' files.
    let full = if so.include_linked { crate::cmd::place::document::full_documents(doc).0 } else { Cow::Borrowed(doc) };
    // With a precision, the previews show the rounded art the file holds.
    let drawn = match so.precision {
        Some(decimals) => Cow::Owned(rounded(&full, decimals)),
        None => Cow::Borrowed(full.as_ref()),
    };
    so.merged_image = merged_png(&drawn)?;
    so.thumbnail = preview_png(&drawn)?;
    drop(drawn);
    let native = match full {
        Cow::Owned(full) if so.include_linked => {
            // Include available bytes without replacing missing, relinkable placed objects.
            let mut original = doc.clone();
            original.images = full.images;
            Cow::Owned(original)
        }
        other => other,
    };
    // Without an Export for Screens folder outside the read roots.
    let native = super::access::with_readable_settings_folder(&native);
    so.pretty = so.pretty_fits(&native);
    etcfile::save_to(&native, &so, w).map_err(|e| bad(cmd, e.to_string()))
}

/// `doc` with its path geometry rounded to `decimals` places, as [`etcfile::save_to`] writes it.
fn rounded(doc: &Document, decimals: u8) -> Document {
    let mut d = doc.clone();
    d.round_geometry(decimals.min(etcfile::MAX_PRECISION));
    d
}

/// The ICC files of the profiles `doc` is tagged with that were loaded from files (the built-in
/// ones are there wherever VectorCraft runs).
fn embedded_profiles(doc: &Document) -> BTreeMap<String, Vec<u8>> {
    [&doc.color_profiles.rgb, &doc.color_profiles.cmyk]
        .into_iter()
        .flatten()
        .filter_map(|name| cms::profile(name).filter(|p| !p.builtin))
        .filter_map(|p| Some((p.name.clone(), cms::icc_bytes(&p.name).ok()?.to_vec())))
        .collect()
}

/// Most profiles read from one file.
const MAX_PROFILES: usize = 8;

/// Make the profiles a file carries available (those not installed here), named as in the file →
/// a warning per profile that can't be used.
pub(super) fn install_profiles(profiles: BTreeMap<String, Vec<u8>>) -> Vec<String> {
    // A document is tagged with an RGB and a CMYK profile at most: more is junk.
    profiles
        .into_iter()
        .take(MAX_PROFILES)
        .filter(|(name, _)| cms::profile(name).is_none())
        .filter_map(|(name, icc)| {
            cms::register_icc(&icc, Some(name.clone())).err().map(|e| format!("the colour profile “{name}” the file carries can't be used: {e}"))
        })
        .collect()
}



/// The longest side of `mergedimage.png`, in pixels: larger documents are scaled down to it.
const MERGED_MAX: f64 = 8192.0;

/// The region the previews show: every artboard together, else the art.
fn preview_region(doc: &Document) -> Option<vectorcraft_doc::geom::Rect> {
    etcfile::manifest::artboard_bounds(doc).or_else(|| vectorcraft_render::encode::art_bounds(doc))
}

/// `doc`'s preview region as a PNG at `scale` (`None`: nothing to show, or too large to render).
fn region_png(doc: &Document, scale: impl FnOnce(vectorcraft_doc::geom::Rect) -> f64) -> Result<Option<Vec<u8>>> {
    let Some(r) = preview_region(doc) else { return Ok(None) };
    let scale = scale(r);
    if vectorcraft_render::raster_size(r, scale).is_err() {
        return Ok(None);
    }
    vectorcraft_render::Renderer::new().render_region(doc, r, scale, false).to_png().map(Some).map_err(EngineError::Other)
}

/// `mergedimage.png`: the preview region at one pixel per point, at most [`MERGED_MAX`] pixels on
/// its longer side.
pub(crate) fn merged_png(doc: &Document) -> Result<Option<Vec<u8>>> {
    region_png(doc, |r| (MERGED_MAX / r.width().max(r.height())).min(1.0))
}

/// The thumbnail: the preview region fitted into [`etcfile::THUMBNAIL_MAX`] pixels.
pub(crate) fn preview_png(doc: &Document) -> Result<Option<Vec<u8>>> {
    region_png(doc, |r| f64::from(etcfile::THUMBNAIL_MAX) / r.width().max(r.height()))
}
