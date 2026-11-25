//! Ex-Tempore Creare native project files (`.etc`), per `docs/Standardsdocs/filestandard.md`.
//!
//! An `.etc` file is a ZIP archive: `mimetype` first and stored, `project.json` (authoritative:
//! the identity fields and the engine's document model as JSON under `document`), `stack.xml`
//! (an ORA-style projection), `mergedimage.png`, `Thumbnails/thumbnail.png`, the embedded images
//! under `data/` and the embedded ICC profiles under `profiles/`.
//!
//! This crate replaces VectorCraft's `.etc` serializer. The Android side reaches content
//! URIs itself; this crate builds, validates and reads the bytes and saves paths atomically.
#![forbid(unsafe_code)]

pub mod atomic;
pub mod base64;
mod brushart;
mod error;
mod load;
pub mod manifest;
mod save;
mod stack;
pub mod zipread;
pub mod zipwrite;

pub use atomic::{read_only_location, sync_file, write_atomic, write_atomic_with, write_new_with};
pub use brushart::{placed_as_groups, with_brush_art};
pub use error::EtcError;
pub use load::{EtcFile, FileInfo, MAX_ASSET, MAX_JSON, MAX_UNPACKED, load, load_file, load_file_capped, merged_image, project_json, sniff, thumbnail, validate};
pub use save::{PRETTY_MAX_BYTES, PRETTY_MAX_OBJECTS, SaveOptions, new_project_id, save, save_file, save_to, save_with, sha256_hex, with_project_id};
pub use stack::stack_xml;

/// The native file extension.
pub const EXTENSION: &str = "etc";
/// The `mimetype` entry and the provisional MIME type (§3).
pub const MIME: &str = "application/x-ex-tempore-creare";
/// `project.json`'s `format`.
pub const FORMAT: &str = "ex-tempore-creare";
/// The `formatVersion` this crate writes, and the newest it reads.
pub const FORMAT_VERSION: &str = "0.4.0";
/// Archive entry names.
pub const PROJECT_JSON: &str = "project.json";
pub const STACK_XML: &str = "stack.xml";
pub const MERGED_IMAGE: &str = "mergedimage.png";
pub const THUMBNAIL: &str = "Thumbnails/thumbnail.png";
/// The thumbnail's longest side, in pixels.
pub const THUMBNAIL_MAX: u32 = 256;
/// The most decimal places [`SaveOptions::precision`] rounds path geometry to.
pub const MAX_PRECISION: u8 = 9;
/// The document `extra` key through which a caller gives a save its project id; the save writes it
/// as `projectId` and not in the document, and reading never puts it back in.
pub const PROJECT_ID_KEY: &str = "etcProjectId";

/// Whether `ext` (without the dot, any case) is the native extension.
pub fn is_native_ext(ext: &str) -> bool {
    ext.eq_ignore_ascii_case(EXTENSION)
}

/// Whether a file name or path ends in the native extension.
pub fn is_native_name(name: &str) -> bool {
    std::path::Path::new(name).extension().and_then(|e| e.to_str()).is_some_and(is_native_ext)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use vectorcraft_doc::{Document, ImageBlob};

    use super::*;
    use crate::zipread::ZipArchive;

    fn png() -> Vec<u8> {
        b"\x89PNG\r\n\x1a\nnot really a png".to_vec()
    }

    #[test]
    fn round_trips_a_document_with_identity_and_layout() {
        let mut doc = Document::new(400.0, 300.0);
        doc.title = "Round trip".into();
        let id = new_project_id();
        let mut saved = doc.clone();
        with_project_id(&mut saved, &id);
        let o = SaveOptions { pretty: true, merged_image: Some(png()), thumbnail: Some(png()), ..SaveOptions::default() };
        let bytes = save_with(&saved, &o).unwrap();
        assert!(sniff(&bytes));
        let zip = ZipArchive::new(&bytes).unwrap();
        let names: Vec<&str> = zip.entries().iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, ["mimetype", PROJECT_JSON, STACK_XML, MERGED_IMAGE, THUMBNAIL]);
        let json: serde_json::Value = serde_json::from_slice(&project_json(&bytes).unwrap()).unwrap();
        assert_eq!((json["format"].as_str(), json["formatVersion"].as_str()), (Some(FORMAT), Some(FORMAT_VERSION)));
        assert_eq!(json["projectId"].as_str(), Some(id.as_str()));
        assert_eq!(json["canvas"]["mode"], "finite");
        assert_eq!((json["canvas"]["width"].as_u64(), json["canvas"]["height"].as_u64()), (Some(400), Some(300)));
        assert!(json["document"].get(PROJECT_ID_KEY).is_none(), "the id is written once, at the top");
        assert_eq!(thumbnail(&bytes), Some(png()));
        let f = load_file(&bytes).unwrap();
        assert_eq!(f.doc.title, "Round trip");
        assert_eq!(f.info.project_id, id);
        assert_eq!(f.doc, doc, "the document reads back as it was");
        // Saving what was read writes the same document again.
        assert_eq!(project_json(&save(&f.doc, true)), project_json(&save(&doc, true)));
    }

    #[test]
    fn embeds_only_used_images_and_checks_profiles() {
        let mut doc = Document::new(100.0, 100.0);
        doc.images.insert("unused".into(), ImageBlob { mime: "image/png".into(), bytes: Arc::new(png()), proxy: None });
        let icc = b"fake icc".to_vec();
        let o = SaveOptions { profiles: [("My Profile".to_owned(), icc.clone())].into(), ..SaveOptions::default() };
        let bytes = save_with(&doc, &o).unwrap();
        let zip = ZipArchive::new(&bytes).unwrap();
        assert!(zip.entries().iter().all(|e| !e.name.starts_with("data/")), "an image nothing uses isn't written");
        let f = load_file(&bytes).unwrap();
        assert_eq!(f.profiles.get("My Profile"), Some(&icc));
        assert!(f.doc.images.is_empty());
    }

    #[test]
    fn refuses_other_and_newer_files() {
        assert!(matches!(load(b"{}"), Err(EtcError::NotEtc(_))));
        let mut w = zipwrite::ZipWriter::new(Vec::new());
        w.add("mimetype", MIME.as_bytes(), zipwrite::Method::Stored).unwrap();
        w.add(PROJECT_JSON, br#"{"format":"ex-tempore-creare","formatVersion":"9.0.0"}"#, zipwrite::Method::Deflate).unwrap();
        assert!(matches!(load(&w.finish().unwrap()), Err(EtcError::TooNew(v)) if v == "9.0.0"));
        let mut w = zipwrite::ZipWriter::new(Vec::new());
        w.add("project.json", b"{}", zipwrite::Method::Stored).unwrap();
        let wrong_first = w.finish().unwrap();
        assert!(!sniff(&wrong_first));
        assert!(matches!(load(&wrong_first), Err(EtcError::NotEtc(_))));
        assert!(is_native_name("a/b.ETC") && !is_native_name("a.etcr"));
    }
}

// GLOSSARY
// - `atomic`, `base64`, `manifest`, `zipread`, `zipwrite`: public modules (atomic saving, base64, project.json, ZIP reading and writing).
// - `brushart`, `error`, `load`, `save`, `stack`: private modules re-exported below.
// - `EXTENSION`: `etc`.
// - `MIME`: `application/x-ex-tempore-creare`, the `mimetype` entry.
// - `FORMAT`: `ex-tempore-creare`, `project.json`'s `format`.
// - `FORMAT_VERSION`: `0.4.0`, the version written and the newest read.
// - `PROJECT_JSON`, `STACK_XML`, `MERGED_IMAGE`, `THUMBNAIL`: archive entry names.
// - `THUMBNAIL_MAX`: the thumbnail's longest side in pixels.
// - `MAX_PRECISION`: the most decimal places geometry is rounded to.
// - `PROJECT_ID_KEY`: the `Document::extra` key through which a save is given its project id.
// - `is_native_ext`, `is_native_name`: whether an extension or file name is `.etc`.
