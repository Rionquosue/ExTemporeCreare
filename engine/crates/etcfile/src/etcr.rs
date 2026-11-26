//! Reference boards: `.etcr` files (`filestandard.md` §7). A board is an unbounded canvas of
//! reference images linked to files outside it (never embedded in the board), each with its own
//! position, size, rotation, flips, non-destructive crop and visibility, and the last camera
//! (centre, zoom and rotation). An image can instead be stored inside an artwork `.etc` file
//! ([`EMBEDDED_KIND`]): its source then names that file and the asset key it is kept under.
//!
//! An `.etcr` file is a ZIP archive: `mimetype` first and stored (holding [`ETCR_MIME`]),
//! `board.json` (authoritative) and an optional `Thumbnails/thumbnail.png`. Fields this version
//! doesn't know are kept (`extra`) so saving never drops them.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::error::EtcError;
use crate::load::MAX_ASSET;
use crate::manifest::{is_newer_than, version_parts};
use crate::zipread::{ZipArchive, ZipError};
use crate::zipwrite::{Method, ZipWriter};
use crate::{MAX_JSON, THUMBNAIL};

/// The reference board file extension.
pub const ETCR_EXTENSION: &str = "etcr";
/// The `mimetype` entry and provisional MIME type of `.etcr` files (§7.1).
pub const ETCR_MIME: &str = "application/x-ex-tempore-creare-reference";
/// `board.json`'s `format`.
pub const ETCR_FORMAT: &str = "ex-tempore-creare-reference";
/// The `formatVersion` this crate writes, and the newest it reads.
pub const ETCR_FORMAT_VERSION: &str = "0.1.0";
/// The board's manifest entry.
pub const BOARD_JSON: &str = "board.json";
/// The source kind of an image linked to its own file.
pub const LINKED_KIND: &str = "linked-file";
/// The source kind of an image stored inside an artwork `.etc` file (§7.2): the source's `uri`
/// and/or `relativePath` locate that `.etc`, its `assetKey` the image in it.
pub const EMBEDDED_KIND: &str = "etc-embedded";

/// The last camera over the board: its centre in board units, its zoom and its rotation in
/// degrees, clockwise (§7.3).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Viewport {
    pub center: [f64; 2],
    pub zoom: f64,
    #[serde(default, rename = "rotationDegrees")]
    pub rotation_degrees: f64,
}

impl Default for Viewport {
    fn default() -> Self {
        Self { center: [0.0, 0.0], zoom: 1.0, rotation_degrees: 0.0 }
    }
}

/// Where a reference image's file is (§7.2): a path relative to the board's folder and/or a URI,
/// with what identifies the file for relinking.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    /// [`LINKED_KIND`] or [`EMBEDDED_KIND`].
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relative_path: Option<String>,
    /// A device-specific absolute URI (`file://…`, `content://…`), the fallback to `relativePath`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(default)]
    pub original_filename: String,
    /// The image's size in pixels.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// Lowercase hex SHA-256 of the file's bytes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    /// For [`EMBEDDED_KIND`]: the key the image is kept under in the `.etc` file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_key: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// An image's place on the board (§7.3): position and size in board units, rotation about the
/// pivot (fractions of the size), and whether it is mirrored left-right and/or top-bottom.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transform {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(default)]
    pub rotation_degrees: f64,
    #[serde(default = "centre")]
    pub pivot: [f64; 2],
    #[serde(default, skip_serializing_if = "is_false")]
    pub flip_x: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub flip_y: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

fn centre() -> [f64; 2] {
    [0.5, 0.5]
}

/// A crop in source-image fractions, `[0, 1]` (§7.3).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Crop {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

/// One reference image on the board.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BoardImage {
    pub id: String,
    pub source: Source,
    pub transform: Transform,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crop: Option<Crop>,
    #[serde(default = "shown")]
    pub visible: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

fn shown() -> bool {
    true
}

/// A reference board: `board.json` (§7.4). `images` are in back-to-front order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Board {
    pub format: String,
    pub format_version: String,
    pub board_id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub viewport: Viewport,
    #[serde(default)]
    pub images: Vec<BoardImage>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Board {
    /// A new empty board titled `title`, with a new board id.
    pub fn new(title: &str) -> Self {
        Self {
            format: ETCR_FORMAT.into(),
            format_version: ETCR_FORMAT_VERSION.into(),
            board_id: crate::new_project_id(),
            title: title.into(),
            viewport: Viewport::default(),
            images: vec![],
            extra: Map::new(),
        }
    }

    /// Image `id`.
    pub fn image(&self, id: &str) -> Option<&BoardImage> {
        self.images.iter().find(|i| i.id == id)
    }

    /// Image `id`, to change.
    pub fn image_mut(&mut self, id: &str) -> Option<&mut BoardImage> {
        self.images.iter_mut().find(|i| i.id == id)
    }

    /// Why the board breaks a rule of §7 (`None`: it doesn't): identity, finite numbers, a
    /// positive zoom and size, crops within the source, unique image ids.
    pub fn problem(&self) -> Option<String> {
        if self.format != ETCR_FORMAT {
            return Some(format!("its format is `{}`", self.format));
        }
        if self.board_id.trim().is_empty() {
            return Some("it has no boardId".into());
        }
        let finite = |v: &[f64]| v.iter().all(|x| x.is_finite());
        if !finite(&self.viewport.center) || !self.viewport.rotation_degrees.is_finite() || !self.viewport.zoom.is_finite() || self.viewport.zoom <= 0.0 {
            return Some("its viewport isn't a finite centre and rotation and a positive zoom".into());
        }
        let mut ids = std::collections::HashSet::new();
        for i in &self.images {
            if i.id.is_empty() || !ids.insert(i.id.as_str()) {
                return Some(format!("image id `{}` is empty or used twice", i.id));
            }
            let t = &i.transform;
            if !finite(&[t.x, t.y, t.width, t.height, t.rotation_degrees, t.pivot[0], t.pivot[1]]) || t.width <= 0.0 || t.height <= 0.0 {
                return Some(format!("image `{}` has no finite position and positive size", i.id));
            }
            if let Some(c) = i.crop {
                let inside = finite(&[c.x, c.y, c.width, c.height])
                    && c.width > 0.0
                    && c.height > 0.0
                    && c.x >= 0.0
                    && c.y >= 0.0
                    && c.x + c.width <= 1.0 + 1e-9
                    && c.y + c.height <= 1.0 + 1e-9;
                if !inside {
                    return Some(format!("image `{}` has a crop outside its source", i.id));
                }
            }
            if i.source.relative_path.is_none() && i.source.uri.is_none() {
                return Some(format!("image `{}` has neither a relative path nor a URI", i.id));
            }
            if i.source.kind == EMBEDDED_KIND && i.source.asset_key.as_deref().is_none_or(str::is_empty) {
                return Some(format!("image `{}` is stored in an .etc file but has no assetKey", i.id));
            }
        }
        None
    }
}

/// Whether `bytes` start like an `.etcr` file (its first entry a stored `mimetype` of [`ETCR_MIME`]).
pub fn sniff_board(bytes: &[u8]) -> bool {
    let field = |at: usize| bytes.get(at..at + 2).map(|s| u16::from_le_bytes([s[0], s[1]]) as usize);
    let (Some(method), Some(name_len), Some(extra_len)) = (field(8), field(26), field(28)) else { return false };
    let start = 30 + name_len + extra_len;
    let size = bytes.get(18..22).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]) as usize);
    bytes.starts_with(b"PK\x03\x04")
        && method == 0
        && size == Some(ETCR_MIME.len())
        && bytes.get(30..30 + name_len) == Some(b"mimetype")
        && bytes.get(start..start + ETCR_MIME.len()) == Some(ETCR_MIME.as_bytes())
}

/// Read and check an `.etcr` file.
pub fn read_board(bytes: &[u8]) -> Result<Board, EtcError> {
    let zip = ZipArchive::new(bytes).map_err(|e| match e {
        ZipError::NotZip => EtcError::NotEtc("it isn't a ZIP archive".into()),
        e => e.into(),
    })?;
    let first = zip.entries().first().ok_or_else(|| EtcError::NotEtc("the archive is empty".into()))?;
    if first.name != "mimetype" || first.method != 0 || zip.read_entry(first, 256)? != ETCR_MIME.as_bytes() {
        return Err(EtcError::NotEtc("it isn't a reference board (no stored `mimetype` of the board format)".into()));
    }
    let json = zip.read(BOARD_JSON, MAX_JSON)?.ok_or_else(|| EtcError::NotEtc("it has no board.json".into()))?;
    let board: Board = serde_json::from_slice(&json).map_err(|e| EtcError::NotEtc(format!("board.json: {e}")))?;
    if version_parts(&board.format_version).is_none() {
        return Err(EtcError::NotEtc(format!("its formatVersion `{}` isn't a version", board.format_version)));
    }
    if is_newer_than(&board.format_version, ETCR_FORMAT_VERSION) {
        return Err(EtcError::TooNew(board.format_version));
    }
    if let Some(p) = board.problem() {
        return Err(EtcError::NotEtc(p));
    }
    Ok(board)
}

/// The thumbnail PNG of an `.etcr` file.
pub fn board_thumbnail(bytes: &[u8]) -> Option<Vec<u8>> {
    ZipArchive::new(bytes).ok()?.read(THUMBNAIL, MAX_ASSET).ok()?
}

/// Write `board` as an `.etcr` file, with `thumbnail` (an sRGB PNG) when given. A board that
/// breaks a rule of §7 is refused.
pub fn write_board(board: &Board, thumbnail: Option<&[u8]>) -> Result<Vec<u8>, EtcError> {
    if let Some(p) = board.problem() {
        return Err(EtcError::Encode(p));
    }
    let mut b = board.clone();
    b.format_version = ETCR_FORMAT_VERSION.into();
    let json = serde_json::to_vec_pretty(&b).map_err(EtcError::encode)?;
    let mut zip = ZipWriter::new(Vec::new());
    zip.add("mimetype", ETCR_MIME.as_bytes(), Method::Stored).map_err(EtcError::encode)?;
    zip.add(BOARD_JSON, &json, Method::Deflate).map_err(EtcError::encode)?;
    if let Some(png) = thumbnail {
        zip.add(THUMBNAIL, png, Method::Stored).map_err(EtcError::encode)?;
    }
    zip.finish().map_err(EtcError::encode)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board() -> Board {
        let mut b = Board::new("Character References");
        b.viewport = Viewport { center: [250.0, -125.0], zoom: 0.85, rotation_degrees: 15.0 };
        b.images.push(BoardImage {
            id: "reference-001".into(),
            source: Source {
                kind: "linked-file".into(),
                relative_path: Some("assets/pose-reference.png".into()),
                original_filename: "pose-reference.png".into(),
                ..Source::default()
            },
            transform: Transform { x: 150.0, y: -80.0, width: 600.0, height: 420.0, rotation_degrees: -12.0, pivot: [0.5, 0.5], flip_x: true, flip_y: false },
            crop: Some(Crop { x: 0.1, y: 0.0, width: 0.8, height: 1.0 }),
            visible: true,
            extra: Map::new(),
        });
        b
    }

    #[test]
    fn round_trips_a_board_and_keeps_unknown_fields() {
        let mut b = board();
        b.extra.insert("fromTheFuture".into(), Value::from(1));
        let bytes = write_board(&b, None).unwrap();
        assert!(sniff_board(&bytes) && !crate::sniff(&bytes));
        assert_eq!(read_board(&bytes).unwrap(), b);
        assert!(read_board(&crate::save(&vectorcraft_doc::Document::new(10.0, 10.0), false)).is_err(), "an .etc is no board");
    }

    #[test]
    fn refuses_what_breaks_the_rules() {
        let mut b = board();
        b.images[0].crop = Some(Crop { x: 0.5, y: 0.0, width: 0.8, height: 1.0 });
        assert!(write_board(&b, None).is_err());
        let mut b = board();
        b.viewport.zoom = 0.0;
        assert!(b.problem().is_some());
        let mut b = board();
        b.images.push(b.images[0].clone());
        assert!(b.problem().unwrap().contains("used twice"));
        let mut b = board();
        b.images[0].source.kind = EMBEDDED_KIND.into();
        assert!(b.problem().unwrap().contains("assetKey"));
        b.images[0].source.asset_key = Some("reference-asset-1".into());
        assert!(b.problem().is_none());
    }

    #[test]
    fn reads_a_board_without_the_newer_fields() {
        let json = r#"{"format":"ex-tempore-creare-reference","formatVersion":"0.1.0","boardId":"b","title":"t",
            "viewport":{"center":[1,2],"zoom":2},"images":[{"id":"a","source":{"kind":"linked-file","uri":"content://x"},
            "transform":{"x":0,"y":0,"width":10,"height":10}}]}"#;
        let b: Board = serde_json::from_str(json).unwrap();
        assert_eq!(b.viewport.rotation_degrees, 0.0);
        assert!(!b.images[0].transform.flip_x && !b.images[0].transform.flip_y);
        let out = serde_json::to_string(&b).unwrap();
        assert!(!out.contains("flipX") && out.contains("rotationDegrees"));
    }
}

// GLOSSARY
// - `ETCR_EXTENSION`, `ETCR_MIME`, `ETCR_FORMAT`, `ETCR_FORMAT_VERSION`, `BOARD_JSON`: `.etcr` identity and entry names.
// - `LINKED_KIND`, `EMBEDDED_KIND`: source kinds (own file; stored inside an `.etc`).
// - `Viewport`: the board's last camera (centre, zoom, rotation).
// - `Source`: where a reference image's file is (relative path, URI, asset key inside an `.etc`) and what identifies it (name, size, SHA-256).
// - `Transform`: an image's position, size, rotation, pivot and flips on the board. `is_false`: serde skip helper.
// - `Crop`: a non-destructive crop in source-image fractions.
// - `BoardImage`: one reference image (id, source, transform, crop, visibility).
// - `Board`: `board.json`; `new`, `image`, `image_mut`, `problem` (the §7 rule it breaks).
// - `centre`, `shown`: serde defaults (pivot at the centre, visible).
// - `sniff_board`: whether bytes start like an `.etcr` file.
// - `read_board`: read and check an `.etcr` file.
// - `board_thumbnail`: the thumbnail of an `.etcr` file.
// - `write_board`: write a board as an `.etcr` file.
