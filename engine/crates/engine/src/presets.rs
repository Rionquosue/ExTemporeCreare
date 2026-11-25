//! Built-in preset libraries, read at run time from data files (`toolstandard.md` §2.11): brush,
//! symbol, swatch, gradient and pattern libraries and graphic style libraries.
//!
//! Each kind has a folder under the presets root (`tools/Libraries/presets/<kind>/`) with an
//! `index.json` listing its libraries in menu order: `{"libraries": [{"id", "name", "file"}]}`.
//! The host sets the root once at start ([`set_root`]); on Android that is the folder the app
//! copies its bundled presets into. Without it, the repository's own folder is used (development
//! and tests). Files are read when a library is first asked for and kept for the session.

use std::any::Any;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use serde::Deserialize;

/// A kind of preset library: its folder under the presets root.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Brushes,
    Symbols,
    Swatches,
    Gradients,
    Patterns,
    GraphicStyles,
}

impl Kind {
    /// The kind's folder name.
    pub fn folder(self) -> &'static str {
        match self {
            Kind::Brushes => "brushes",
            Kind::Symbols => "symbols",
            Kind::Swatches => "swatches",
            Kind::Gradients => "gradients",
            Kind::Patterns => "patterns",
            Kind::GraphicStyles => "graphic-styles",
        }
    }
}

/// One library an `index.json` lists.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Entry {
    pub id: String,
    pub name: String,
    pub file: String,
}

#[derive(Deserialize)]
struct Index {
    libraries: Vec<Entry>,
}

/// The presets root the host set ([`set_root`]).
static ROOT: RwLock<Option<PathBuf>> = RwLock::new(None);

/// The libraries read so far, by kind and id.
type Cache = Mutex<HashMap<(Kind, String), Arc<dyn Any + Send + Sync>>>;

fn cache() -> &'static Cache {
    static CACHE: OnceLock<Cache> = OnceLock::new();
    CACHE.get_or_init(Default::default)
}

/// Each kind's index, read once per root.
fn indexes() -> &'static Mutex<HashMap<Kind, Arc<Vec<Entry>>>> {
    static INDEXES: OnceLock<Mutex<HashMap<Kind, Arc<Vec<Entry>>>>> = OnceLock::new();
    INDEXES.get_or_init(Default::default)
}

/// Set the presets root (the folder holding `brushes/`, `symbols/`…). Libraries and indexes read
/// from an earlier root are forgotten.
pub fn set_root(dir: impl Into<PathBuf>) {
    *ROOT.write().unwrap_or_else(|e| e.into_inner()) = Some(dir.into());
    cache().lock().unwrap_or_else(|e| e.into_inner()).clear();
    indexes().lock().unwrap_or_else(|e| e.into_inner()).clear();
}

/// The presets root: the one set, else the repository's `tools/Libraries/presets`.
pub fn root() -> PathBuf {
    ROOT.read()
        .unwrap_or_else(|e| e.into_inner())
        .clone()
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../tools/Libraries/presets"))
}

/// Kind `kind`'s libraries in menu order (empty when its index is missing or unreadable).
pub fn entries(kind: Kind) -> Arc<Vec<Entry>> {
    let mut indexes = indexes().lock().unwrap_or_else(|e| e.into_inner());
    indexes
        .entry(kind)
        .or_insert_with(|| {
            let path = root().join(kind.folder()).join("index.json");
            let read = std::fs::read(&path).ok().and_then(|b| serde_json::from_slice::<Index>(&b).ok());
            Arc::new(read.map(|i| i.libraries).unwrap_or_default())
        })
        .clone()
}

/// Library `id` of `kind` in its index.
pub fn entry(kind: Kind, id: &str) -> Option<Entry> {
    entries(kind).iter().find(|e| e.id == id).cloned()
}

/// The bytes of library `id`'s file.
pub fn bytes(kind: Kind, id: &str) -> Option<Vec<u8>> {
    let e = entry(kind, id)?;
    // The file name comes from the index: only a plain name inside the kind's folder is read.
    if e.file.contains(['/', '\\']) || e.file.starts_with('.') {
        return None;
    }
    std::fs::read(root().join(kind.folder()).join(&e.file)).ok()
}

/// Library `id` of `kind`, read with `read` from its file the first time and kept.
pub fn library<L: Send + Sync + 'static>(kind: Kind, id: &str, read: impl FnOnce(&[u8], &Entry) -> Option<L>) -> Option<Arc<L>> {
    let key = (kind, id.to_string());
    if let Some(hit) = cache().lock().unwrap_or_else(|e| e.into_inner()).get(&key) {
        return hit.clone().downcast::<L>().ok();
    }
    let e = entry(kind, id)?;
    let lib = Arc::new(read(&bytes(kind, id)?, &e)?);
    cache().lock().unwrap_or_else(|e| e.into_inner()).insert(key, lib.clone());
    Some(lib)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_lists_its_libraries_and_each_file_is_there() {
        for kind in [Kind::Brushes, Kind::Symbols, Kind::Swatches, Kind::Gradients, Kind::Patterns, Kind::GraphicStyles] {
            let list = entries(kind);
            assert!(!list.is_empty(), "{kind:?}");
            for e in list.iter() {
                assert!(bytes(kind, &e.id).is_some(), "{kind:?}/{}", e.file);
            }
        }
        assert!(entry(Kind::Brushes, "no-such-library").is_none());
    }
}

// GLOSSARY
// - `Kind`: a kind of preset library; `folder` gives its folder under the presets root.
// - `Entry`: one library of an `index.json` (`id`, `name`, `file`).
// - `Index`: an `index.json` as read.
// - `ROOT`: the presets root the host set.
// - `Cache`, `cache`: libraries read so far, by kind and id.
// - `indexes`: each kind's index, read once per root.
// - `set_root`: set the presets root and forget what was read from the earlier one.
// - `root`: the presets root in use.
// - `entries`: a kind's libraries in menu order.
// - `entry`: one library of a kind by id.
// - `bytes`: the bytes of a library's file.
// - `library`: a library read from its file once and kept.
