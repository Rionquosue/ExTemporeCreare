//! Ex-Tempore Creare native project files (`.etc`), per `docs/Standardsdocs/filestandard.md`.
//!
//! An `.etc` file is a ZIP archive: `mimetype` first and stored, `project.json` (authoritative:
//! the identity fields and the engine's document model as JSON), `stack.xml` (an ORA-style
//! projection), `mergedimage.png`, `Thumbnails/thumbnail.png`, the embedded images under `data/`
//! and the embedded ICC profiles under `profiles/`.
//!
//! This crate replaces VectorCraft's `.vectorcraft` serializer. Work in progress: the archive,
//! manifest, save and load modules follow.
#![forbid(unsafe_code)]

pub mod atomic;
pub mod base64;

pub use atomic::{read_only_location, sync_file, write_atomic, write_atomic_with, write_new_with};

// GLOSSARY
// - `atomic`: module; temporary-file-then-rename saving.
// - `base64`: module; base64 encode and decode for `dataBase64` replies.
// - `read_only_location`, `sync_file`, `write_atomic`, `write_atomic_with`, `write_new_with`: re-exported from `atomic`.
