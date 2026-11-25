//! Pattern libraries (Window → Swatch Libraries → Patterns, the Swatches panel's library button):
//! read-only sets of pattern swatches the swatch library panel shows. Each built-in library is a
//! document holding its patterns, a data file in the presets folder ([`crate::presets`]).
//! `swatch.library.add` copies a pattern into the document with its swatch.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

use crate::presets::Kind;

use vectorcraft_color::{Paint, Swatch, SwatchLibrary};
use vectorcraft_doc::pattern::pattern_paint;
use vectorcraft_doc::{Appearance, Document, Node, NodeId, PatternDef};
use vectorcraft_geom::{Rect, shapes};

use super::Result;
use super::clipboard::Clipboard;
use super::clipboard::resources::{Imported, SwatchChoices};
use super::swatchlib::LibraryInfo;

/// The library category of the built-in pattern libraries.
pub const CATEGORY: &str = "patterns";

/// The built-in pattern libraries as the menus list them, from the presets folder
/// ([`crate::presets`]).
pub fn infos() -> impl Iterator<Item = LibraryInfo> {
    let list = crate::presets::entries(Kind::Patterns);
    (0..list.len()).map(move |i| LibraryInfo { id: list[i].id.clone(), name: list[i].name.clone(), category: CATEGORY })
}

/// Built-in pattern library `id`: the document holding its patterns, read once.
pub fn library_doc(id: &str) -> Option<Arc<Document>> {
    crate::presets::library(Kind::Patterns, id, |bytes, e| {
        let mut d = etcfile::load(bytes).ok()?;
        d.title = e.name.clone();
        Some(d)
    })
}

/// Built-in pattern library `id` as the library panel shows it: a swatch for each pattern. The
/// same library is returned each time (a panel finds a swatch's library by its address).
pub fn library(id: &str) -> Option<(LibraryInfo, Arc<SwatchLibrary>)> {
    static MADE: LazyLock<Mutex<HashMap<String, Arc<SwatchLibrary>>>> = LazyLock::new(Default::default);
    let info = infos().find(|i| i.id == id)?;
    let mut made = MADE.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(lib) = made.get(id) {
        return Some((info, lib.clone()));
    }
    let doc = library_doc(id)?;
    let swatches = doc.patterns.iter().map(|p| Swatch { name: p.name.clone(), paint: pattern_paint(&p.name), global: false, spot: false }).collect();
    let lib = Arc::new(SwatchLibrary { name: info.name.clone(), swatches, groups: vec![] });
    made.insert(id.to_string(), lib.clone());
    Some((info, lib))
}

/// The built-in library document that holds the pattern of swatch `w`, when `w` is one of a
/// built-in pattern library's own swatches (a library panel draws them from it). A swatch of
/// another library is not, even under the same name.
pub fn swatch_source(w: &Swatch) -> Option<Arc<Document>> {
    let Paint::Pattern { pattern, .. } = &w.paint else { return None };
    let own = |lib: &SwatchLibrary| lib.swatches.iter().any(|s| std::ptr::eq(s, w));
    let b = infos().find(|b| library(&b.id).is_some_and(|(_, lib)| own(&lib)))?;
    library_doc(&b.id).filter(|d| d.pattern(pattern).is_some())
}

/// Add pattern `name` of library `lib` to `d` with the resources its art uses, as pasting art
/// painted with it does: the same pattern there already is used, and a name another pattern or
/// swatch has gets a number → the pattern's name in `d`, and the swatches the import added for
/// the pattern's art: (the swatch's name in `lib`, its name in `d`), pattern swatches left out.
/// A global or spot color whose name `d` gives another color comes in under a free name, as Add
/// to Swatches adds such a color, so that the pattern keeps the library's colors.
pub(crate) fn import(d: &mut Document, lib: &Document, name: &str) -> Result<(String, Vec<(String, String)>)> {
    let had: std::collections::HashSet<String> = d.swatches_iter().map(|w| w.name.clone()).collect();
    let imported = gather(lib, name).import_into(d, &SwatchChoices::add_all(), false)?;
    let from = |to: &str| {
        let renamed = imported.renamed.iter().find(|r| r["kind"] == "swatch" && r["to"] == to).and_then(|r| r["from"].as_str());
        renamed.unwrap_or(to).to_string()
    };
    let added = d
        .swatches_iter()
        .filter(|w| !had.contains(&w.name) && !matches!(w.paint, Paint::Pattern { .. }))
        .map(|w| (from(&w.name), w.name.clone()))
        .collect();
    Ok((imported_name(&imported, name), added))
}

/// Pattern `name` of `lib` with the resources its art uses, as an object painted with it is copied.
fn gather(lib: &Document, name: &str) -> Clipboard {
    let user = Node::path(NodeId(0), shapes::rectangle(Rect::new(0.0, 0.0, 1.0, 1.0)), Appearance::basic(pattern_paint(name), Paint::None, 0.0));
    Clipboard::gather(lib, vec![user])
}

/// The name pattern `name` has in the document `imported` went into.
fn imported_name(imported: &Imported, name: &str) -> String {
    let renamed = imported.renamed.iter().find(|r| r["kind"] == "pattern" && r["from"] == name).and_then(|r| r["to"].as_str());
    renamed.unwrap_or(name).to_string()
}

/// The pattern of `d` that [`import`] of pattern `name` of library `lib` would use without adding
/// anything to `d` → its name in `d`. A pattern whose art uses no other resource is compared as
/// [`existing_pattern`] does. One whose art uses other resources, such as a pattern it is painted
/// with, is imported into a copy of `d`, so that those resources are compared too.
pub(crate) fn present_pattern(d: &Document, lib: &Document, name: &str) -> Option<String> {
    let gathered = gather(lib, name);
    let only_pattern = gathered.patterns.len() == 1
        && gathered.symbols.is_empty()
        && gathered.swatches.is_empty()
        && gathered.graphic_styles.is_empty()
        && gathered.char_styles.is_empty()
        && gathered.para_styles.is_empty()
        && gathered.brushes.is_empty()
        && gathered.images.is_empty();
    if only_pattern {
        return existing_pattern(d, lib, name);
    }
    // Importing reuses only a pattern under that name or a numbered copy of it with the same tile
    // options and as many art objects; without one, the document is not copied.
    let def = lib.pattern(name)?;
    let bare = |p: &PatternDef| PatternDef { name: String::new(), art: vec![], ..p.clone() };
    let copy = |n: &str| {
        n == name || n.strip_prefix(name).and_then(|r| r.strip_prefix(' ')).is_some_and(|r| !r.is_empty() && r.bytes().all(|b| b.is_ascii_digit()))
    };
    if !d.patterns.iter().any(|p| copy(&p.name) && p.art.len() == def.art.len() && bare(p) == bare(def)) {
        return None;
    }
    let mut probe = d.clone();
    let imported = gathered.import_into(&mut probe, &SwatchChoices::add_all(), false).ok()?;
    (imported.added == 0).then(|| imported_name(&imported, name))
}

/// The pattern of `d` that is pattern `name` of library `lib`: the same tile options and art under
/// that name or a numbered copy of it ("Polka Dots 2"), the name itself first.
pub(crate) fn existing_pattern(d: &Document, lib: &Document, name: &str) -> Option<String> {
    let def = lib.pattern(name)?;
    let bare = |p: &PatternDef| PatternDef { name: String::new(), art: vec![], ..p.clone() };
    let copy = |n: &str| {
        n == name || n.strip_prefix(name).and_then(|r| r.strip_prefix(' ')).is_some_and(|r| !r.is_empty() && r.bytes().all(|b| b.is_ascii_digit()))
    };
    let mut found: Vec<&PatternDef> = d.patterns.iter().filter(|p| copy(&p.name) && bare(p) == bare(def) && p.art == def.art).collect();
    found.sort_by_key(|p| p.name != name);
    found.first().map(|p| p.name.clone())
}

/// The name of the first swatch of `d` that paints with pattern `pattern` as placed in its own
/// space, as a pattern's own swatch does.
pub(crate) fn pattern_swatch(d: &Document, pattern: &str) -> Option<String> {
    let paint = pattern_paint(pattern);
    d.swatches_iter().find(|w| w.paint == paint).map(|w| w.name.clone())
}


















