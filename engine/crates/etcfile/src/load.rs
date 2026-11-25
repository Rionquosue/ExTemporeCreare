//! Reading and validating `.etc` files (`filestandard.md` §3, §11). The file is untrusted: the
//! archive and its entry sizes are checked before anything is unpacked, the identity fields
//! before the document, and the document is read within the engine's object budgets and then
//! brought within the ranges the editor allows.

use std::collections::BTreeMap;
use std::sync::Arc;

use vectorcraft_doc::{Document, ImageBlob};

use crate::error::EtcError;
use crate::manifest::{self, Identity, Project};
use crate::save::sha256_hex;
use crate::zipread::{ZipArchive, ZipError};
use crate::{FORMAT, MERGED_IMAGE, MIME, PROJECT_ID_KEY, PROJECT_JSON, THUMBNAIL};

/// Most bytes `project.json` may unpack to.
pub const MAX_JSON: u64 = 1 << 30;
/// Most bytes one asset (an image, a profile, a preview) may unpack to.
pub const MAX_ASSET: u64 = 1 << 30;
/// Most bytes all entries together may unpack to.
pub const MAX_UNPACKED: u64 = 4 << 30;
/// Most bytes the `mimetype` entry may hold.
const MAX_MIMETYPE: u64 = 256;

/// What a file says about itself besides its document.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileInfo {
    /// The file's `formatVersion`.
    pub format_version: String,
    /// The file's `projectId`.
    pub project_id: String,
}

/// An `.etc` file read.
#[derive(Debug)]
pub struct EtcFile {
    pub doc: Document,
    /// What was done to bring an untrusted document within the editor's limits, and diagnostics
    /// such as a profile whose bytes don't match their recorded hash.
    pub notes: Vec<String>,
    pub info: FileInfo,
    /// Embedded ICC profiles by the document's profile name.
    pub profiles: BTreeMap<String, Vec<u8>>,
}

/// Whether `bytes` start like an `.etc` file: a ZIP whose first entry is `mimetype`, stored,
/// holding exactly [`MIME`].
pub fn sniff(bytes: &[u8]) -> bool {
    let field = |at: usize| bytes.get(at..at + 2).map(|s| u16::from_le_bytes([s[0], s[1]]) as usize);
    let (Some(method), Some(name_len), Some(extra_len)) = (field(8), field(26), field(28)) else { return false };
    let start = 30 + name_len + extra_len;
    let size = bytes.get(18..22).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]) as usize);
    bytes.starts_with(b"PK\x03\x04")
        && method == 0
        && size == Some(MIME.len())
        && bytes.get(30..30 + name_len) == Some(b"mimetype")
        && bytes.get(start..start + MIME.len()) == Some(MIME.as_bytes())
}

/// Check an opened archive's identity: `mimetype` first, stored, exactly [`MIME`].
fn check_mimetype(zip: &ZipArchive) -> Result<(), EtcError> {
    let first = zip.entries().first().ok_or_else(|| EtcError::NotEtc("the archive is empty".into()))?;
    if first.name != "mimetype" || first.method != 0 {
        return Err(EtcError::NotEtc("its first entry isn't a stored `mimetype`".into()));
    }
    let mime = zip.read_entry(first, MAX_MIMETYPE)?;
    if mime != MIME.as_bytes() {
        return Err(EtcError::NotEtc(format!("its mimetype is `{}`", String::from_utf8_lossy(&mime))));
    }
    Ok(())
}

/// The unpacked entry `name` (`None`: the file has none, or isn't an `.etc` file).
fn entry(bytes: &[u8], name: &str, max: u64) -> Option<Vec<u8>> {
    let zip = ZipArchive::new(bytes).ok()?;
    check_mimetype(&zip).ok()?;
    zip.read(name, max).ok()?
}

/// The `project.json` of an `.etc` file.
pub fn project_json(bytes: &[u8]) -> Option<Vec<u8>> {
    entry(bytes, PROJECT_JSON, MAX_JSON)
}

/// `bytes` with its `project.json` replaced by `project`, every other entry kept as it was (for
/// tools and tests that edit a manifest; the result is not validated).
pub fn with_project_json(bytes: &[u8], project: &[u8]) -> Result<Vec<u8>, EtcError> {
    use crate::zipwrite::{Method, ZipWriter};
    let zip = ZipArchive::new(bytes)?;
    check_mimetype(&zip)?;
    let mut w = ZipWriter::new(Vec::with_capacity(bytes.len()));
    for e in zip.entries() {
        let data = if e.name == PROJECT_JSON { project.to_vec() } else { zip.read_entry(e, MAX_ASSET.max(MAX_JSON))? };
        w.add(&e.name, &data, if e.method == 0 { Method::Stored } else { Method::Deflate }).map_err(EtcError::encode)?;
    }
    w.finish().map_err(EtcError::encode)
}

/// An `.etc` file holding only `mimetype` and `project` as its `project.json` (for tools and tests
/// that build a manifest by hand; the result is not validated).
pub fn from_project_json(project: &[u8]) -> Vec<u8> {
    use crate::zipwrite::{Method, ZipWriter};
    let mut w = ZipWriter::new(Vec::with_capacity(project.len() / 2 + 256));
    let written = w.add("mimetype", MIME.as_bytes(), Method::Stored).and_then(|()| w.add(PROJECT_JSON, project, Method::Deflate));
    written.and_then(|()| w.finish()).expect("writing to memory doesn't fail")
}

/// The thumbnail PNG of an `.etc` file.
pub fn thumbnail(bytes: &[u8]) -> Option<Vec<u8>> {
    entry(bytes, THUMBNAIL, MAX_ASSET)
}

/// The flattened `mergedimage.png` of an `.etc` file.
pub fn merged_image(bytes: &[u8]) -> Option<Vec<u8>> {
    entry(bytes, MERGED_IMAGE, MAX_ASSET)
}

/// Read only the document of an `.etc` file.
pub fn load(bytes: &[u8]) -> Result<Document, EtcError> {
    load_file(bytes).map(|f| f.doc)
}

/// Read an `.etc` file.
pub fn load_file(bytes: &[u8]) -> Result<EtcFile, EtcError> {
    load_file_capped(bytes, MAX_UNPACKED)
}

/// Validate an `.etc` file as [`load_file`] does, without keeping the document.
pub fn validate(bytes: &[u8]) -> Result<(), EtcError> {
    load_file(bytes).map(drop)
}

/// [`load_file`], refusing a file whose entries unpack to more than `max` bytes together.
pub fn load_file_capped(bytes: &[u8], max: u64) -> Result<EtcFile, EtcError> {
    let zip = ZipArchive::new(bytes).map_err(|e| match e {
        ZipError::NotZip => EtcError::NotEtc("it isn't a ZIP archive".into()),
        e => e.into(),
    })?;
    check_mimetype(&zip)?;
    let total: u64 = zip.entries().iter().map(|e| e.size).fold(0, u64::saturating_add);
    if total > max.min(MAX_UNPACKED) {
        return Err(EtcError::TooBig);
    }
    let json = zip.read(PROJECT_JSON, MAX_JSON.min(max))?.ok_or_else(|| EtcError::NotEtc("it has no project.json".into()))?;
    let id: Identity = serde_json::from_slice(&json).map_err(|e| EtcError::NotEtc(format!("project.json: {e}")))?;
    if id.format != FORMAT {
        return Err(EtcError::NotEtc(format!("its format is `{}`", id.format)));
    }
    if manifest::version_parts(&id.format_version).is_none() {
        return Err(EtcError::NotEtc(format!("its formatVersion `{}` isn't a version", id.format_version)));
    }
    if manifest::is_newer(&id.format_version) {
        return Err(EtcError::TooNew(id.format_version));
    }
    let project = within_budgets(|| serde_json::from_slice::<Project>(&json)).map_err(|e| {
        let why = if e.to_string().contains("recursion limit") { format!("its objects are nested too deep ({e})") } else { e.to_string() };
        EtcError::NotEtc(format!("project.json: {why}"))
    })?;
    drop(json);
    let mut notes = Vec::new();
    let mut doc = project.document;
    for (key, asset) in project.assets.images {
        let bytes = zip.read(&asset.path, MAX_ASSET)?.ok_or_else(|| EtcError::BadAsset(asset.path.clone()))?;
        let bytes = Arc::new(bytes);
        let proxy = asset.proxy.then(|| bytes.clone());
        doc.images.insert(key, ImageBlob { mime: asset.mime, bytes, proxy });
    }
    let mut profiles = BTreeMap::new();
    for (id, p) in project.color_profiles {
        let Some(path) = p.path.filter(|_| p.kind == "embedded") else { continue };
        let icc = zip.read(&path, MAX_ASSET)?.ok_or_else(|| EtcError::BadAsset(path.clone()))?;
        if p.sha256.as_deref().is_some_and(|h| !h.eq_ignore_ascii_case(&sha256_hex(&icc))) {
            notes.push(format!("the colour profile `{id}` ({}) doesn't match its recorded SHA-256 and wasn't used", p.name));
            continue;
        }
        profiles.insert(p.name, icc);
    }
    tidy(&mut doc, &mut notes)?;
    // An id that came in through `extra` is not part of the document.
    doc.extra.remove(PROJECT_ID_KEY);
    // The board links are the document's (`extra`), whichever place the file keeps them.
    if let Some(boards) = project.reference_boards.filter(|v| v.is_array()) {
        doc.extra.insert(crate::REFERENCE_BOARDS_KEY.into(), boards);
    }
    // So are the export slices and grid.
    if let Some(export) = project.export.filter(|v| v.is_object()) {
        doc.extra.insert(crate::EXPORT_KEY.into(), export);
    }
    let project_id = if project.project_id.is_empty() { crate::save::new_project_id() } else { project.project_id };
    Ok(EtcFile { doc, notes, info: FileInfo { format_version: id.format_version, project_id }, profiles })
}

/// Run `read` within the engine's reading budgets: anchors, objects, and values buffered before
/// an object's type.
fn within_budgets<T>(read: impl FnOnce() -> T) -> T {
    use vectorcraft_doc::geom::path::{MAX_READ_ANCHORS, with_anchor_budget};
    use vectorcraft_doc::node::{MAX_BUFFERED_VALUES, MAX_READ_NODES, with_buffered_value_budget, with_node_budget};
    with_anchor_budget(MAX_READ_ANCHORS, || with_node_budget(MAX_READ_NODES, || with_buffered_value_budget(MAX_BUFFERED_VALUES, read)))
}

/// Bring an untrusted document within the editor's limits; what was changed goes into `notes`.
fn tidy(doc: &mut Document, notes: &mut Vec<String>) -> Result<(), EtcError> {
    // A mask-editing layer that no longer exists.
    if doc.mask_edit.is_some_and(|me| !doc.layers.iter().any(|l| l.id == me.layer)) {
        doc.mask_edit = None;
    }
    doc.drop_edit_modes();
    doc.fix_next_id();
    doc.setup.keep_in_range();
    doc.tidy_saved_selections();
    doc.tidy_views();
    doc.tidy_graph_designs();
    doc.raster_effects.add_around = doc.raster_effects.add_around.clamp(0.0, vectorcraft_doc::rastersettings::MAX_ADD_AROUND);
    doc.limit_depth_noting(notes);
    // Ruler guides only directly in layers (after the depth limit; before the canvas check).
    doc.tidy_ruler_guides();
    let had_artboards = !doc.artboards.is_empty();
    let beyond = doc.drop_beyond_canvas();
    if had_artboards && doc.artboards.is_empty() {
        return Err(EtcError::NotEtc(format!("its artboards all lie beyond the canvas (more than {} pt from the origin)", vectorcraft_doc::MAX_COORD)));
    }
    notes.extend(beyond.note());
    // Saved ruler guides become nodes on their layers (after `fix_next_id`).
    doc.guides_into_nodes();
    doc.keep_type_values_in_range();
    doc.fix_ruby();
    doc.raster_effects_ppi = Some(doc.raster_effects_ppi).filter(|v| v.is_finite()).unwrap_or(72.0).clamp(1.0, 2400.0);
    doc.keep_scripts_in_range();
    doc.version = doc.version.max(vectorcraft_doc::FORMAT_VERSION);
    Ok(())
}

// GLOSSARY
// - `MAX_JSON`, `MAX_ASSET`, `MAX_UNPACKED`, `MAX_MIMETYPE`: unpacked size limits.
// - `FileInfo`: `formatVersion` and `projectId` of a file read.
// - `EtcFile`: a file read (document, notes, info, embedded profiles).
// - `sniff`: whether bytes start like an `.etc` file.
// - `check_mimetype`: check the stored first `mimetype` entry.
// - `entry`: one unpacked entry of an `.etc` file.
// - `project_json`, `thumbnail`, `merged_image`: single entries of an `.etc` file.
// - `with_project_json`: an `.etc` file with its `project.json` replaced.
// - `from_project_json`: an `.etc` file of only `mimetype` and a given `project.json`.
// - `load`, `load_file`, `load_file_capped`: read an `.etc` file.
// - `validate`: check an `.etc` file without keeping the document.
// - `within_budgets`: read within the engine's anchor, object and buffered-value budgets.
// - `tidy`: bring an untrusted document within the editor's limits.
