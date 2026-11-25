//! Symbol libraries (Window → Symbol Libraries, the Symbols panel's library button): read-only sets
//! of symbols the library panel shows. A library is a document holding the symbols with the
//! resources they use (nested symbols, patterns, swatches, brushes, graphic styles, images) and
//! their Symbol Options; saved, it is a native `.etc` file, which File › Open also opens.
//! The built-in libraries are data files in the presets folder ([`crate::presets`]); User Defined ones are the `.etc`
//! files in the user library folder ([`Libraries`]), and Other Library… loads the symbols of any
//! document VectorCraft opens. `symbol.addFromLibrary` copies symbols into the document;
//! `symbol.saveLibrary` writes the document's.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serde_json::{Value, json};
use vectorcraft_doc::{Document, Node, NodeId, NodeKind, Symbol};
use vectorcraft_geom::{Affine, Point};

use super::brushsym::{CURRENT_SYMBOL, Placement, natural_size, place_xf, set_natural_size};
use super::clipboard::Clipboard;
use super::clipboard::resources::SwatchChoices;
use super::swatch::str_list;
use super::swatchlib::{FileData, LibraryFile, LibraryInfo, concealed_library_id, concealed_library_ids, shown_id, shown_library_id, stem};
use super::{CommandSpec, Result, Session, always, bad, bool_or, cmd, has_doc, number_or, str_param, u64_req};
use crate::presets::{Entry, Kind};

/// The extension of saved symbol libraries: native documents.
pub const SYMBOLS_EXT: &str = etcfile::EXTENSION;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            query "symbol.libraries",
            "Symbol Libraries",
            [],
            None,
            "{} → {libraries: [{id, name, category: \"builtIn\"|\"user\" (User Defined: the .etc files in the user library folder, rescanned now)|\"loaded\" (symbol.loadLibrary), count, path?}], userFolder} every symbol library the library panel opens, in menu order",
            always,
            list
        ),
        cmd!(
            query "symbol.library",
            "Symbol Library",
            [],
            None,
            "{library: id or name} → {id, name, category, symbols: [{name, size: [w, h], registration: 0..8, exportType}] (as symbol.list)}",
            always,
            get
        ),
        cmd!(
            "symbol.addFromLibrary",
            "Add to Symbols",
            ["Window", "Symbol Libraries"],
            None,
            "{library: id or name, name? | names?: [symbol names in the library] (default: all of it), place?: false (also place an instance of the first one, its registration point at x, y: default the first artboard's centre, in the stacking order symbol.place gives it, Draw Behind included), x?, y?, swatchConflict? (as edit.paste)} copy library symbols into the document's Symbols as one undo step (with the place), with their Symbol Options and the resources they use (the symbols they nest, patterns, swatches, brushes, graphic styles, images). A symbol the document already has (same name and art) isn't added again; a name another symbol has gets a number (\"Star 2\"). The first one becomes the current symbol → {library, names: [each symbol's name in the document], added: [names added], existing: [names already there], placed?: id}",
            has_doc,
            add
        ),
        cmd!(
            "symbol.placeFromDocument",
            "Place Symbol Instance",
            [],
            None,
            "{sourceDocument: uid, name: a symbol of that document, x?, y?} place an instance of a symbol of another open document so its registration point is at (x, y) (default: the first artboard's center), in the stacking order symbol.place gives it (Draw Behind included), as a symbol dragged from that document's Symbols panel onto the canvas is placed. The symbol, the symbols it nests and the resources they use are added as a drop of art from that document adds them, in the same undo step as the instance: a symbol the active document already has with the same name, art and Symbol Options is used, and one with the same name but other art or Symbol Options gets a number (\"Leaf 2\"); each symbol added takes its Symbol Options along. When sourceDocument is the active document, its own symbol is placed, as symbol.place places it. An error, with no edit, when that document is no longer open or has no symbol `name` → {id, name: the symbol's name in the active document}",
            has_doc,
            place_from_document
        ),
        cmd!(
            "symbol.saveLibrary",
            "Save Symbol Library…",
            ["Window", "Symbols"],
            None,
            "{path?, names?: [symbol names] (default: all), name?: library name (default: the document's), user?: false (save into the user library folder, listed under User Defined)} save the document's symbols, with their Symbol Options and the resources they use, as a symbol library: a .etc document holding only them → {path, count, library?: id when saved to the user folder}; without path or user → {dataBase64: the file's bytes, count}",
            has_doc,
            save
        ),
        cmd!(
            "symbol.loadLibrary",
            "Other Library…",
            ["Window", "Symbol Libraries"],
            None,
            "{path? | data?: file text | dataBase64?, name?: file name (default: the path's)} load the symbols of any document VectorCraft opens (see document.formats), a saved symbol library included, for the library panel (Window → Symbol Libraries lists it until the app quits; a file of the user library folder, or a file with the same bytes as one there, opens as that User Defined library) → {library: id, name, count}",
            always,
            load
        ),
    ]
}

/// A symbol library: its name and a document holding its symbols (in panel order) with the
/// resources they use and their Symbol Options; the document has no art of its own.
#[derive(Clone, Debug)]
pub struct SymbolLibrary {
    pub name: String,
    pub doc: Document,
}

impl Default for SymbolLibrary {
    fn default() -> Self {
        Self { name: String::new(), doc: Document::new(100.0, 100.0) }
    }
}

impl SymbolLibrary {
    pub fn len(&self) -> usize {
        self.doc.symbols.len()
    }
    pub fn is_empty(&self) -> bool {
        self.doc.symbols.is_empty()
    }
    pub fn symbol(&self, name: &str) -> Option<&Symbol> {
        self.doc.symbols.iter().find(|s| s.name == name)
    }

    /// Document `src`'s symbols `names` (empty: all of them, in panel order) as a library named
    /// `name`: the symbols, the symbols they nest and every other resource they use, with their
    /// Symbol Options.
    pub fn from_document(src: &Document, names: &[String], name: String) -> std::result::Result<Self, String> {
        // Each name's first place in the panel, looked up rather than scanned for: scanning the
        // symbols for each one made a library of N symbols cost N² comparisons.
        let mut order: HashMap<&str, usize> = HashMap::new();
        for (i, s) in src.symbols.iter().enumerate() {
            order.entry(s.name.as_str()).or_insert(i);
        }
        if let Some(n) = names.iter().find(|n| !order.contains_key(n.as_str())) {
            return Err(format!("no symbol `{n}`"));
        }
        let wanted: HashSet<&str> = names.iter().map(String::as_str).collect();
        let picked = src.symbols.iter().filter(|s| names.is_empty() || wanted.contains(s.name.as_str()));
        let clip = Clipboard::gather(src, picked.map(|s| instance(&s.name)).collect());
        let mut doc = clip.to_document();
        for l in &mut doc.layers {
            if let Some(ch) = Arc::make_mut(l).children_mut() {
                ch.clear();
            }
        }
        // The panel order of the document they came from.
        doc.symbols.sort_by_key(|s| order.get(s.name.as_str()).copied());
        for s in &doc.symbols.clone() {
            copy_options(src, &s.name, &mut doc, &s.name);
        }
        doc.title = name.clone();
        Ok(Self { name, doc })
    }
}

impl LibraryFile for SymbolLibrary {
    const EXTS: &[&str] = &[SYMBOLS_EXT];
    fn read(bytes: &[u8], stem: &str) -> std::result::Result<Self, String> {
        let doc = etcfile::load(bytes).map_err(|e| e.to_string())?;
        SymbolLibrary::from_document(&doc, &[], stem.to_string())
    }
    fn name(&self) -> &str {
        &self.name
    }
}

/// User Defined and loaded symbol libraries.
pub type Libraries = super::swatchlib::Libraries<SymbolLibrary>;

/// Built-in library `id`, read from the presets folder ([`crate::presets`]) once.
fn builtin_library(id: &str) -> Option<Arc<SymbolLibrary>> {
    crate::presets::library(Kind::Symbols, id, |bytes, e| SymbolLibrary::read(bytes, &e.name).ok())
}









/// An instance of symbol `name` (what the resource gathering follows).
fn instance(name: &str) -> Node {
    Node::new(NodeId(0), NodeKind::SymbolInstance { symbol: name.into(), xf: Affine::IDENTITY })
}

/// Symbol `from`'s Symbol Options in `src` (natural size, registration, export type, 9-slice
/// guides, Align to Pixel Grid) given to symbol `to` of `d`.
fn copy_options(src: &Document, from: &str, d: &mut Document, to: &str) {
    set_natural_size(d, to, Some(natural_size(src, from)));
    d.set_symbol_registration(to, src.symbol_registration(from));
    d.set_symbol_export_type(to, src.symbol_export_type(from));
    d.set_symbol_nine_slice(to, src.symbol_nine_slice(from));
    d.set_symbol_pixel_aligned(to, src.symbol_pixel_aligned(from));
}

fn builtin_info(e: &Entry) -> LibraryInfo {
    LibraryInfo { id: e.id.clone(), name: e.name.clone(), category: "builtIn" }
}

/// Every symbol library, in menu order: built-in, User Defined, loaded.
pub fn libraries(s: &Session) -> Vec<LibraryInfo> {
    crate::presets::entries(Kind::Symbols).iter().map(builtin_info).chain(s.symbol_libraries.infos().cloned()).collect()
}

/// Library `key` (an id, or a name in any case) with its info.
pub fn library(s: &Session, key: &str) -> Option<(LibraryInfo, Arc<SymbolLibrary>)> {
    if let Some(found) = s.symbol_libraries.get(key) {
        return Some(found);
    }
    if let Some(found) = concealed_library_id(s, key).and_then(|id| s.symbol_libraries.get(&id)) {
        return Some(found);
    }
    if let Some(e) = crate::presets::entry(Kind::Symbols, key) {
        return Some((builtin_info(&e), builtin_library(key)?));
    }
    let id = libraries(s).into_iter().find(|l| l.name.eq_ignore_ascii_case(key))?.id;
    library(s, &id)
}

fn library_param(s: &Session, p: &Value, cmd: &str) -> Result<(LibraryInfo, Arc<SymbolLibrary>)> {
    let key = str_param(p, "library").ok_or_else(|| bad(cmd, "missing `library` (see symbol.libraries)"))?;
    library(s, key).ok_or_else(|| bad(cmd, format!("no symbol library `{key}` (see symbol.libraries)")))
}

fn list(s: &mut Session, _: &Value) -> Result<Value> {
    s.symbol_libraries.rescan();
    let concealed = concealed_library_ids(s, []);
    let libs: Vec<Value> = libraries(s)
        .into_iter()
        .map(|l| {
            let count = library(s, &l.id).map_or(0, |(_, lib)| lib.len());
            let (id, path) = (shown_id(&concealed, &l.id), s.symbol_libraries.shown_path(&l.id));
            json!({"id": id, "name": l.name, "category": l.category, "count": count, "path": path})
        })
        .collect();
    Ok(json!({ "libraries": libs, "userFolder": s.symbol_libraries.shown_user_dir() }))
}

fn get(s: &mut Session, p: &Value) -> Result<Value> {
    let (info, lib) = library_param(s, p, "symbol.library")?;
    let d = &lib.doc;
    let symbols: Vec<Value> = d
        .symbols
        .iter()
        .map(|sym| {
            let (w, h) = natural_size(d, &sym.name);
            json!({"name": sym.name, "size": [w, h], "registration": d.symbol_registration(&sym.name).0, "exportType": d.symbol_export_type(&sym.name).id()})
        })
        .collect();
    Ok(json!({"id": shown_library_id(s, &info.id), "name": info.name, "category": info.category, "symbols": symbols}))
}

fn add(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "symbol.addFromLibrary";
    let (info, lib) = library_param(s, p, C)?;
    let mut names = str_list(p, "names");
    names.extend(str_param(p, "name").map(str::to_string));
    if let Some(n) = names.iter().find(|n| lib.symbol(n).is_none()) {
        return Err(bad(C, format!("no symbol `{n}` in `{}`", lib.name)));
    }
    let mut picked: Vec<String> = if names.is_empty() { lib.doc.symbols.iter().map(|s| s.name.clone()).collect() } else { names };
    // Each once, in the order asked for.
    let mut seen = HashSet::new();
    picked.retain(|n| seen.insert(n.clone()));
    let place = bool_or(p, "place", false);
    if place && picked.is_empty() {
        return Err(bad(C, format!("`{}` has no symbols to place", lib.name)));
    }
    let choices = SwatchChoices::parse(C, p)?;
    let clip = Clipboard::gather(&lib.doc, picked.iter().map(|n| instance(n)).collect());
    let st = s.doc()?;
    let before: HashSet<String> = st.doc.symbols.iter().map(|s| s.name.clone()).collect();
    // A library symbol's name in the document: its own, or the one the import gave it.
    let target = |renamed: &[Value], n: &str| -> String {
        let to = renamed.iter().find(|r| r["kind"] == "symbol" && r["from"] == n).and_then(|r| r["to"].as_str());
        to.unwrap_or(n).to_string()
    };
    // Where the symbols land, worked out on a copy: nothing to add and nothing to place is no edit.
    let mut probe = (*st.doc).clone();
    let imported = clip.import_into(&mut probe, &choices, false)?;
    let doc_names: Vec<String> = picked.iter().map(|n| target(&imported.renamed, n)).collect();
    let centre = st.doc.artboards.first().map(|a| a.rect.center()).unwrap_or(Point::ZERO);
    let at = if place { Some(Point::new(number_or(p, "x", centre.x, C)?, number_or(p, "y", centre.y, C)?)) } else { None };
    let spot = if place { Some(Placement::of(s)?) } else { None };
    let mut out = json!({ "library": shown_library_id(s, &info.id) });
    if imported.added > 0 || place {
        let placed = s.edit("Add to Symbols", |d, sel| {
            let imported = clip.import_into(d, &choices, false)?;
            // Every symbol brought in (nested ones too) takes its Symbol Options along.
            for sym in &clip.symbols {
                let to = target(&imported.renamed, &sym.name);
                if !before.contains(&to) {
                    copy_options(&lib.doc, &sym.name, d, &to);
                }
            }
            let Some(first) = doc_names.first() else { return Ok(None) };
            d.unknown.insert(CURRENT_SYMBOL.into(), json!(first));
            let (Some(at), Some(spot)) = (at, &spot) else { return Ok(None) };
            let xf = place_xf(at, natural_size(d, first), d.symbol_registration(first));
            let id = d.alloc_id();
            spot.insert(d, Node::new(id, NodeKind::SymbolInstance { symbol: first.clone(), xf }))?;
            sel.set([id]);
            Ok(Some(id))
        })?;
        if let Some(id) = placed {
            out["placed"] = json!(id.0);
        }
    } else if let Some(first) = doc_names.first() {
        s.execute("symbol.setCurrent", &json!({ "name": first }))?;
    }
    let (added, existing): (Vec<&String>, Vec<&String>) = doc_names.iter().partition(|n| !before.contains(*n));
    out["names"] = json!(doc_names);
    out["added"] = json!(added);
    out["existing"] = json!(existing);
    Ok(out)
}

/// `symbol.placeFromDocument`: an instance of symbol `name` of the open document `sourceDocument`
/// placed in the active document, with the symbol and the resources it uses added as a drop of
/// art from that document adds them, in one undo step. The source document is not changed.
fn place_from_document(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "symbol.placeFromDocument";
    let uid = u64_req(p, "sourceDocument", C)?;
    let name = str_param(p, "name").ok_or_else(|| bad(C, "missing `name`"))?;
    let st = s.doc()?;
    let centre = st.doc.artboards.first().map(|a| a.rect.center()).unwrap_or(Point::ZERO);
    let at = Point::new(number_or(p, "x", centre.x, C)?, number_or(p, "y", centre.y, C)?);
    let spot = Placement::of(s)?;
    let same = uid == st.uid;
    let src = s.documents().iter().find(|d| d.uid == uid).map(|d| Arc::clone(&d.doc));
    let src = src.ok_or_else(|| bad(C, "the document the symbol came from is no longer open"))?;
    if !src.symbols.iter().any(|sym| sym.name == name) {
        return Err(bad(C, format!("no symbol named `{name}` in the document it came from")));
    }
    // A symbol of the active document itself is placed as it is.
    let clip = (!same).then(|| Clipboard::gather(&src, vec![instance(name)]));
    // The source's colors are kept even where the active document has swatches of the same names.
    let choices = SwatchChoices::parse(C, &json!({"swatchConflict": "add"}))?;
    let (id, placed) = s.edit("Place Symbol Instance", |d, sel| {
        let placed = match &clip {
            // Each symbol added takes its Symbol Options along.
            Some(clip) => {
                let imported = clip.import_into(d, &choices, false)?;
                let renamed = imported.renamed.iter().find(|r| r["kind"] == "symbol" && r["from"] == name).and_then(|r| r["to"].as_str());
                renamed.unwrap_or(name).to_string()
            }
            None => name.to_string(),
        };
        let xf = place_xf(at, natural_size(d, &placed), d.symbol_registration(&placed));
        let id = d.alloc_id();
        spot.insert(d, Node::new(id, NodeKind::SymbolInstance { symbol: placed.clone(), xf }))?;
        sel.set([id]);
        Ok((id, placed))
    })?;
    Ok(json!({ "id": id.0, "name": placed }))
}

fn save(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "symbol.saveLibrary";
    let st = s.doc()?;
    let name = str_param(p, "name").map(str::trim).filter(|n| !n.is_empty()).map_or_else(|| stem(&st.title()).to_string(), str::to_string);
    let lib = SymbolLibrary::from_document(&st.doc, &str_list(p, "names"), name).map_err(|e| bad(C, e))?;
    if lib.is_empty() {
        return Err(bad(C, "the document has no symbols to save"));
    }
    let mut out = json!({ "count": lib.len() });
    let bytes = etcfile::save_file(&lib.doc);
    s.symbol_libraries.write(p, &lib.name, SYMBOLS_EXT, FileData::Binary(bytes), &mut out, C)?;
    Ok(out)
}

fn load(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "symbol.loadLibrary";
    let (info, lib) = s.symbol_libraries.load(p, C, |bytes, file| {
        let loaded = super::fileio::load(file, bytes).map_err(|e| bad(C, e.to_string()))?;
        if loaded.preview_only {
            return Err(bad(C, "only this Affinity file's embedded preview could be read; use File › Open to see it with its warning"));
        }
        SymbolLibrary::from_document(&loaded.doc, &[], stem(file).to_string()).map_err(|e| bad(C, e))
    })?;
    Ok(json!({"library": shown_library_id(s, &info.id), "name": info.name, "count": lib.len()}))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A library made from thousands of symbols holds them all, once each, in the panel order of
    /// the document they came from, a name given twice kept at its first place.
    #[test]
    fn many_symbols_keep_their_panel_order() {
        let mut src = Document::new(100.0, 100.0);
        let n = 4000;
        // Names out of alphabetical order, so the order kept is the panel's.
        src.symbols = (0..n)
            .map(|i| Symbol { name: format!("s{}", (i * 7919) % n), art: Arc::new(Node::group(NodeId(i as u64 + 1), vec![])), dynamic: false })
            .collect();
        let lib = SymbolLibrary::from_document(&src, &[], "Many".into()).unwrap();
        let names = |d: &Document| d.symbols.iter().map(|s| s.name.clone()).collect::<Vec<_>>();
        assert_eq!(names(&lib.doc), names(&src));
        let picked = ["s3".to_string(), "s1".to_string()];
        let lib = SymbolLibrary::from_document(&src, &picked, "Two".into()).unwrap();
        let order = |name: &str| src.symbols.iter().position(|s| s.name == name);
        let mut want = vec!["s3", "s1"];
        want.sort_by_key(|n| order(n));
        assert_eq!(names(&lib.doc), want);
        assert!(SymbolLibrary::from_document(&src, &["nope".to_string()], "None".into()).is_err());
    }
}
