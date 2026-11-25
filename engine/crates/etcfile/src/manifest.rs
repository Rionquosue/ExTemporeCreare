//! `project.json`, the authoritative manifest of an `.etc` file (`filestandard.md` §4, §6.6, §8).
//!
//! It holds the identity fields, a descriptive `canvas` block, the embedded colour profiles, the
//! asset table (image key → archive path) and, under `document`, the engine's document model as
//! JSON. The document is the source of truth for every editing feature; the other blocks describe
//! it for other readers.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use vectorcraft_doc::Document;
use vectorcraft_doc::ColorMode;

use crate::{FORMAT, FORMAT_VERSION};

/// One embedded asset: where its bytes are and what they are.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AssetRef {
    pub path: String,
    pub mime: String,
    /// Only the low-resolution preview of a linked image is stored; the engine reads the linked
    /// file again on open.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub proxy: bool,
}

/// One colour profile reference (`filestandard.md` §6.2).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProfileRef {
    /// `embedded` (bytes in `path`) or `builtin`.
    #[serde(rename = "type")]
    pub kind: String,
    /// The profile's name as the document names it.
    #[serde(default)]
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

/// The finite region `mergedimage.png` shows, in document points.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Region {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// The `assets` block: embedded images by the document's image key.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Assets {
    #[serde(default)]
    pub images: BTreeMap<String, AssetRef>,
}

/// Everything in `project.json` except the document, as written.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Head<'a> {
    pub format: &'a str,
    pub format_version: &'a str,
    pub project_id: &'a str,
    pub title: &'a str,
    pub generator: String,
    pub canvas: serde_json::Value,
    pub color_profiles: BTreeMap<String, ProfileRef>,
    pub display_order: &'a str,
    pub assets: &'a Assets,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<serde_json::Value>,
    /// Links to separate `.etcr` boards (§7.5): `[{"boardId", "path", "missingPolicy"}]`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_boards: Option<serde_json::Value>,
    /// Export slices and dot grid (§7A): `{"grid", "slices"}`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export: Option<serde_json::Value>,
}

/// The identity fields, read first so a wrong or newer file is named before its document is read.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    #[serde(default)]
    pub format: String,
    #[serde(default)]
    pub format_version: String,
}

/// `project.json` as read.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    #[serde(default)]
    pub project_id: String,
    #[serde(default)]
    pub color_profiles: BTreeMap<String, ProfileRef>,
    #[serde(default)]
    pub assets: Assets,
    /// Links to separate `.etcr` boards (§7.5).
    #[serde(default)]
    pub reference_boards: Option<serde_json::Value>,
    /// Export slices and dot grid (§7A).
    #[serde(default)]
    pub export: Option<serde_json::Value>,
    pub document: Document,
}

/// The `[major, minor, patch]` of a `formatVersion` (`None`: not three whole numbers).
pub fn version_parts(v: &str) -> Option<[u64; 3]> {
    let mut parts = v.split('.').map(|p| p.parse::<u64>().ok());
    let out = [parts.next()??, parts.next()??, parts.next()??];
    parts.next().is_none().then_some(out)
}

/// Whether a file's `formatVersion` is newer than [`FORMAT_VERSION`].
pub fn is_newer(v: &str) -> bool {
    is_newer_than(v, FORMAT_VERSION)
}

/// Whether version `v` is newer than `ours` (an unreadable one counts as newer).
pub fn is_newer_than(v: &str, ours: &str) -> bool {
    match (version_parts(v), version_parts(ours)) {
        (Some(file), Some(ours)) => file > ours,
        _ => true,
    }
}

/// The profile reference id a document profile name gets: `builtin:<name>` unless embedded.
pub fn profile_ref(name: &str, embedded: &BTreeMap<String, ProfileRef>) -> String {
    embedded.iter().find(|(_, p)| p.name == name).map(|(id, _)| id.clone()).unwrap_or_else(|| format!("builtin:{name}"))
}

/// The descriptive `canvas` block of `doc` (`filestandard.md` §4.1, §6.6, §7A). With artboards
/// the canvas is `finite`, sized to their bounds in pixels at 72 per inch (one per point);
/// without, it is `infinite`.
pub fn canvas(doc: &Document, profiles: &BTreeMap<String, ProfileRef>) -> serde_json::Value {
    let (model, working) = match doc.color_mode {
        ColorMode::Rgb => ("RGB", doc.color_profiles.rgb.as_deref().unwrap_or("sRGB")),
        ColorMode::Cmyk => ("CMYK", doc.color_profiles.cmyk.as_deref().unwrap_or("CMYK")),
    };
    let mut c = serde_json::json!({
        "mode": "infinite",
        "resolutionPpi": if doc.raster_effects_ppi.is_finite() { doc.raster_effects_ppi } else { 72.0 },
        "bitDepthPerChannel": 8,
        "alpha": "straight",
        "colorManagement": {
            "colorModel": model,
            "workingProfileRef": profile_ref(working, profiles),
            "compositingSpace": "working-profile",
            "compositingTransfer": "profile-encoded",
            "renderingIntent": "relative-colorimetric",
            "blackPointCompensation": true,
            "previewProfileRef": "builtin:sRGB"
        },
        "background": "transparent"
    });
    if let Some(r) = artboard_bounds(doc) {
        c["mode"] = "finite".into();
        c["width"] = (r.width().round().max(1.0) as u64).into();
        c["height"] = (r.height().round().max(1.0) as u64).into();
    }
    c
}

/// The bounds of every artboard together (`None`: no artboards).
pub fn artboard_bounds(doc: &Document) -> Option<vectorcraft_doc::geom::Rect> {
    doc.artboards.iter().map(|a| a.rect).reduce(|a, b| a.union(b))
}

/// The `project.json` bytes: `head`'s fields, then `document_json` under `document`.
pub fn project_json(head: &Head, document_json: &[u8], pretty: bool) -> Result<Vec<u8>, serde_json::Error> {
    let mut out = if pretty { serde_json::to_vec_pretty(head)? } else { serde_json::to_vec(head)? };
    // Reopen the head object and add the document as its last member.
    while out.last().is_some_and(|b| b.is_ascii_whitespace()) {
        out.pop();
    }
    debug_assert_eq!(out.last(), Some(&b'}'));
    out.pop();
    out.extend_from_slice(if pretty { b",\n  \"document\": " } else { b",\"document\":" });
    out.extend_from_slice(document_json);
    out.extend_from_slice(if pretty { b"\n}\n" } else { b"}" });
    Ok(out)
}

/// Whether `id` is the format name this crate writes.
pub fn is_etc_format(id: &str) -> bool {
    id == FORMAT
}

// GLOSSARY
// - `AssetRef`: an embedded asset's archive path, MIME type and `proxy` flag.
// - `ProfileRef`: a colour profile reference (`type`, `name`, `path`, `sha256`).
// - `Region`: the finite region `mergedimage.png` shows, in points.
// - `Assets`: the `assets` block, images by document image key.
// - `Head`: every `project.json` member but `document`, as written.
// - `Identity`: `format` and `formatVersion`, read first.
// - `Project`: `project.json` as read (project id, profiles, assets, document).
// - `version_parts`: a `formatVersion` as three numbers.
// - `is_newer`: whether a `formatVersion` is newer than `FORMAT_VERSION`.
// - `is_newer_than`: whether one version is newer than another.
// - `profile_ref`: the reference id of a document profile name.
// - `canvas`: the descriptive `canvas` block of a document.
// - `artboard_bounds`: the union of the document's artboards.
// - `project_json`: the `project.json` bytes from a `Head` and the document JSON.
// - `is_etc_format`: whether a `format` value is `ex-tempore-creare`.
