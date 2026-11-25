//! Writing `.etc` files (`filestandard.md` §3): `mimetype` first and stored, `project.json`,
//! `stack.xml`, the previews, the embedded images and the embedded ICC profiles.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::io::Write;

use sha2::{Digest, Sha256};
use vectorcraft_doc::{Document, ImageBlob};

use crate::brushart::with_brush_art;
use crate::error::EtcError;
use crate::manifest::{self, AssetRef, Assets, Head, ProfileRef};
use crate::stack::stack_xml;
use crate::zipwrite::{Method, ZipWriter};
use crate::{FORMAT, FORMAT_VERSION, MAX_PRECISION, MERGED_IMAGE, MIME, PROJECT_ID_KEY, PROJECT_JSON, STACK_XML, THUMBNAIL};

/// How [`save_with`] writes a document.
#[derive(Clone, Debug, Default)]
pub struct SaveOptions {
    /// Indented `project.json` (readable, diffable); compact otherwise.
    pub pretty: bool,
    /// Include Linked Files: linked images keep the file's own bytes, not just their preview.
    pub include_linked: bool,
    /// Embedded ICC profiles: ICC bytes by the document's profile name.
    pub profiles: BTreeMap<String, Vec<u8>>,
    /// `mergedimage.png`: the flattened render, sRGB PNG.
    pub merged_image: Option<Vec<u8>>,
    /// `Thumbnails/thumbnail.png`: an sRGB PNG of at most [`crate::THUMBNAIL_MAX`] pixels a side.
    pub thumbnail: Option<Vec<u8>>,
    /// The region `merged_image` shows, in points (infinite canvases record it, §7A).
    pub preview_region: Option<manifest::Region>,
    /// Decimal places to round path anchors and handles to, at most [`MAX_PRECISION`]; `None`
    /// writes them exactly.
    pub precision: Option<u8>,
}

/// Documents with at most this many objects are saved with an indented `project.json`.
pub const PRETTY_MAX_OBJECTS: usize = 2000;

/// Documents whose indented document JSON passes this many bytes are saved compact.
pub const PRETTY_MAX_BYTES: usize = 1 << 20;

impl SaveOptions {
    /// The defaults for saving `doc` to disk: indented when small (see [`Self::pretty_fits`]).
    pub fn for_doc(doc: &Document) -> Self {
        let o = Self::default();
        Self { pretty: o.pretty_fits(doc), ..o }
    }

    /// Whether `doc` is small enough to save indented: at most [`PRETTY_MAX_OBJECTS`] objects and
    /// at most [`PRETTY_MAX_BYTES`] of indented document JSON (serializing stops at the limit).
    pub fn pretty_fits(&self, doc: &Document) -> bool {
        struct Capped(usize);
        impl Write for Capped {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.0 = self.0.checked_sub(buf.len()).ok_or_else(|| std::io::Error::other("over the limit"))?;
                Ok(buf.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let objects: usize = doc.layers.iter().map(|l| l.count()).sum();
        objects <= PRETTY_MAX_OBJECTS && serde_json::to_writer_pretty(Capped(PRETTY_MAX_BYTES), &doc.without_edit_modes()).is_ok()
    }
}

/// The document as written: without editing-mode working copies, guides as the saved list, path
/// geometry rounded, and its images and project id taken out.
struct Body {
    d: Document,
    blobs: BTreeMap<String, ImageBlob>,
    /// Images only linked images show.
    linked: BTreeSet<String>,
    project_id: String,
}

fn body(doc: &Document, o: &SaveOptions) -> Body {
    let mut d = doc.without_edit_modes().into_owned();
    d.guides_to_saved();
    let linked = with_brush_art(&mut d, |d| {
        d.drop_placed_resources();
        d.linked_only_images()
    });
    if let Some(decimals) = o.precision {
        d.round_geometry(decimals.min(MAX_PRECISION));
    }
    let blobs = std::mem::take(&mut d.images);
    let project_id = d.extra.remove(PROJECT_ID_KEY).and_then(|v| v.as_str().map(str::to_owned)).unwrap_or_else(new_project_id);
    Body { d, blobs, linked, project_id }
}

/// A new project id (a random UUID).
pub fn new_project_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// The project id `doc` keeps, giving it a new one first when it has none. Call it on the live
/// document before saving, so every save of it writes the same id.
pub fn ensure_project_id(doc: &mut Document) -> String {
    if let Some(id) = doc.extra.get(PROJECT_ID_KEY).and_then(|v| v.as_str()) {
        return id.to_owned();
    }
    let id = new_project_id();
    doc.extra.insert(PROJECT_ID_KEY.into(), id.clone().into());
    id
}

/// Every JSON string in `json` that is in `wanted` (the image keys the document uses).
fn used_strings(json: &[u8], wanted: &BTreeMap<String, ImageBlob>) -> HashSet<String> {
    let mut used = HashSet::new();
    let mut i = 0;
    while i < json.len() {
        if json[i] != b'"' {
            i += 1;
            continue;
        }
        let start = i + 1;
        let mut j = start;
        let mut escaped = false;
        while j < json.len() && json[j] != b'"' {
            escaped |= json[j] == b'\\';
            j += if json[j] == b'\\' { 2 } else { 1 };
        }
        let raw = &json[start..j.min(json.len())];
        let s: Option<String> = if escaped {
            serde_json::from_slice::<String>(&json[i..(j + 1).min(json.len())]).ok()
        } else {
            std::str::from_utf8(raw).ok().map(str::to_owned)
        };
        if let Some(s) = s.filter(|s| wanted.contains_key(s)) {
            used.insert(s);
        }
        i = j + 1;
    }
    used
}

/// The file extension for a MIME type.
fn extension(mime: &str) -> &'static str {
    match mime {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/tiff" => "tif",
        "image/bmp" => "bmp",
        "image/svg+xml" => "svg",
        "application/pdf" => "pdf",
        _ => "bin",
    }
}

/// Whether bytes of this MIME type are compressed already (stored, not deflated again).
fn is_packed(mime: &str) -> bool {
    matches!(mime, "image/png" | "image/jpeg" | "image/gif" | "image/webp")
}

/// The MIME type of a linked image's preview bytes (JPEG or PNG).
fn preview_mime(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(&[0xff, 0xd8]) { "image/jpeg" } else { vectorcraft_doc::links::PROXY_MIME }
}

/// Lowercase hex of `bytes`' SHA-256.
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{b:02x}")).collect()
}

/// Serialize `doc` as an `.etc` file (indented `project.json` when `pretty`), without previews.
pub fn save(doc: &Document, pretty: bool) -> Vec<u8> {
    save_with(doc, &SaveOptions { pretty, ..SaveOptions::default() }).unwrap_or_default()
}

/// Serialize `doc` for saving to disk: indented when small, compact when large.
pub fn save_file(doc: &Document) -> Vec<u8> {
    save_with(doc, &SaveOptions::for_doc(doc)).unwrap_or_default()
}

/// Serialize `doc` as `o` says.
pub fn save_with(doc: &Document, o: &SaveOptions) -> Result<Vec<u8>, EtcError> {
    let mut out = Vec::new();
    save_to(doc, o, &mut out)?;
    Ok(out)
}

/// [`save_with`], written to `w`. On an error `w` holds part of a file: write to a temporary file
/// ([`crate::write_atomic_with`]).
pub fn save_to(doc: &Document, o: &SaveOptions, w: impl Write) -> Result<(), EtcError> {
    let Body { d, blobs, linked, project_id } = body(doc, o);
    let document_json = if o.pretty { serde_json::to_vec_pretty(&d) } else { serde_json::to_vec(&d) }.map_err(EtcError::encode)?;
    let used = if blobs.is_empty() { HashSet::new() } else { used_strings(&document_json, &blobs) };

    // Images the document uses, numbered in key order.
    let mut assets = Assets::default();
    let mut files: Vec<(String, &[u8], Method)> = Vec::new();
    for (n, (key, blob)) in blobs.iter().filter(|(k, _)| used.contains(k.as_str())).enumerate() {
        let preview_only = linked.contains(key) && (!o.include_linked || blob.is_proxy());
        let (bytes, mime, proxy): (&[u8], &str, bool) = match blob.proxy.as_ref().filter(|_| preview_only) {
            Some(p) => (p.as_slice(), preview_mime(p), true),
            None => (blob.bytes.as_slice(), blob.mime.as_str(), false),
        };
        let folder = if mime.starts_with("image/") && mime != "image/svg+xml" { "data/raster" } else { "data/vector" };
        let path = format!("{folder}/image-{:04}.{}", n + 1, extension(mime));
        files.push((path.clone(), bytes, if is_packed(mime) { Method::Stored } else { Method::Deflate }));
        assets.images.insert(key.clone(), AssetRef { path, mime: mime.to_owned(), proxy });
    }

    // Embedded profiles, with their SHA-256 (§6.2).
    let mut color_profiles = BTreeMap::new();
    color_profiles.insert("builtin:sRGB".to_owned(), ProfileRef { kind: "builtin".into(), name: "sRGB".into(), path: None, sha256: None });
    for (n, (name, icc)) in o.profiles.iter().enumerate() {
        let path = format!("profiles/profile-{:03}.icc", n + 1);
        files.push((path.clone(), icc.as_slice(), Method::Deflate));
        let id = format!("profile-{:03}", n + 1);
        color_profiles.insert(id, ProfileRef { kind: "embedded".into(), name: name.clone(), path: Some(path), sha256: Some(sha256_hex(icc)) });
    }

    let canvas = manifest::canvas(&d, &color_profiles);
    let preview = o.merged_image.as_ref().map(|_| {
        let region = o.preview_region.or_else(|| {
            manifest::artboard_bounds(&d).map(|r| manifest::Region { x: r.x0, y: r.y0, width: r.width(), height: r.height() })
        });
        serde_json::json!({ "mergedImage": MERGED_IMAGE, "thumbnail": o.thumbnail.as_ref().map(|_| THUMBNAIL), "region": region })
    });
    let head = Head {
        format: FORMAT,
        format_version: FORMAT_VERSION,
        project_id: &project_id,
        title: &d.title,
        generator: format!("Ex-Tempore Creare engine {}", env!("CARGO_PKG_VERSION")),
        canvas,
        color_profiles,
        display_order: "top-to-bottom",
        assets: &assets,
        preview,
    };
    let project = manifest::project_json(&head, &document_json, o.pretty).map_err(EtcError::encode)?;
    drop(document_json);
    let size = manifest::artboard_bounds(&d).map_or((1, 1), |r| (r.width().round().max(1.0) as u32, r.height().round().max(1.0) as u32));
    let stack = stack_xml(&d, size, d.raster_effects_ppi, o.merged_image.is_some());

    let mut zip = ZipWriter::new(w);
    zip.add("mimetype", MIME.as_bytes(), Method::Stored).map_err(EtcError::encode)?;
    zip.add(PROJECT_JSON, &project, Method::Deflate).map_err(EtcError::encode)?;
    zip.add(STACK_XML, stack.as_bytes(), Method::Deflate).map_err(EtcError::encode)?;
    if let Some(png) = &o.merged_image {
        zip.add(MERGED_IMAGE, png, Method::Stored).map_err(EtcError::encode)?;
    }
    if let Some(png) = &o.thumbnail {
        zip.add(THUMBNAIL, png, Method::Stored).map_err(EtcError::encode)?;
    }
    for (path, bytes, method) in files {
        zip.add(&path, bytes, method).map_err(EtcError::encode)?;
    }
    zip.finish().map_err(EtcError::encode)?;
    Ok(())
}

// GLOSSARY
// - `SaveOptions`: how a document is written (`pretty`, `include_linked`, `profiles`, `merged_image`, `thumbnail`, `preview_region`, `precision`).
// - `SaveOptions::for_doc`, `SaveOptions::pretty_fits`: indented `project.json` for small documents.
// - `PRETTY_MAX_OBJECTS`, `PRETTY_MAX_BYTES`: the limits for an indented `project.json`.
// - `Body`: the document as written, with its images, linked-only image keys and project id.
// - `body`: prepare a `Body` from a document and options.
// - `new_project_id`: a random UUID for `projectId`.
// - `ensure_project_id`: the document's project id, adding one when missing.
// - `used_strings`: the image keys the document JSON references.
// - `extension`: the file extension for a MIME type.
// - `is_packed`: whether a MIME type is compressed already.
// - `preview_mime`: the MIME type of a linked image's preview bytes.
// - `sha256_hex`: lowercase hex SHA-256.
// - `save`, `save_file`, `save_with`, `save_to`: write a document as `.etc` bytes.
