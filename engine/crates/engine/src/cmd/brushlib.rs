//! Brush libraries (Window → Brush Libraries, the Brushes panel's library button): read-only sets
//! of brushes the library panel shows. A library is a document holding the brushes with the
//! resources their art uses (patterns, symbols, swatches, graphic styles, images); saved, it is a
//! native `.etc` file, which File › Open also opens. The built-in libraries are data files in the
//! presets folder ([`crate::presets`]); User Defined ones are the `.etc` files in the user library folder
//! ([`Libraries`]), and Other Library… loads the brushes of any document VectorCraft opens.
//! `brush.addFromLibrary` copies brushes into the document; `brush.saveLibrary` writes the
//! document's.

use std::collections::HashSet;
use std::sync::Arc;

use serde_json::{Value, json};
use vectorcraft_brush::{self as brush, Brush};
use vectorcraft_color::{Color, Paint};
use vectorcraft_doc::{Appearance, Document, Node, NodeId};
use vectorcraft_geom::{PathData, Point, SubPath};

use super::brushsym::{apply_brush, brush_targets, roots, stroke_paint};
use super::clipboard::Clipboard;
use super::clipboard::resources::SwatchChoices;
use super::swatch::str_list;
use super::swatchlib::{FileData, LibraryFile, LibraryInfo, concealed_library_id, concealed_library_ids, shown_id, shown_library_id, stem};
use super::{CommandSpec, Result, Session, always, bad, bool_or, cmd, has_doc, str_param};
use crate::presets::{Entry, Kind};

/// The extension of saved brush libraries: native documents.
pub const BRUSHES_EXT: &str = etcfile::EXTENSION;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            query "brush.libraries",
            "Brush Libraries",
            [],
            None,
            "{} → {libraries: [{id, name, category: \"builtIn\"|\"user\" (User Defined: the .etc files in the user library folder, rescanned now)|\"loaded\" (brush.loadLibrary), count, path?}], userFolder} every brush library the library panel opens, in menu order",
            always,
            list
        ),
        cmd!(
            query "brush.library",
            "Brush Library",
            [],
            None,
            "{library: id or name} → {id, name, category, brushes: [{name, type}] (as brush.list)}",
            always,
            get
        ),
        cmd!(
            "brush.addFromLibrary",
            "Add to Brushes",
            ["Window", "Brush Libraries"],
            None,
            "{library: id or name, name? | names?: [brush names in the library] (default: all of it), apply?: false (also apply the first one to paths, as brush.apply does), ids?|id? (the paths apply takes; default the selected ones), swatchConflict? (as edit.paste)} copy library brushes into the document's Brushes as one undo step (with the apply; named Apply Brush when no brush is added and it only applies one), with the resources their art uses (patterns, symbols, swatches, graphic styles, images). A brush the document already has (same name and definition) isn't added again; a name another brush has gets a number (\"Ribbon 2\"). The first one becomes the current brush → {library, names: [each brush's name in the document], added: [names added], existing: [names already there], applied: [ids of the paths it was applied to]}",
            has_doc,
            add
        ),
        cmd!(
            "brush.saveLibrary",
            "Save Brush Library…",
            ["Window", "Brushes"],
            None,
            "{path?, names?: [brush names] (default: all), name?: library name (default: the document's), user?: false (save into the user library folder, listed under User Defined)} save the document's brushes, with the resources their art uses, as a brush library: a .etc document holding only them → {path, count, library?: id when saved to the user folder}; without path or user → {dataBase64: the file's bytes, count}",
            has_doc,
            save
        ),
        cmd!(
            "brush.loadLibrary",
            "Other Library…",
            ["Window", "Brush Libraries"],
            None,
            "{path? | data?: file text | dataBase64?, name?: file name (default: the path's)} load the brushes of any document VectorCraft opens (see document.formats; one without brushes of its own has the default ones), a saved brush library included, for the library panel (Window → Brush Libraries lists it until the app quits; a file of the user library folder, or a file with the same bytes as one there, opens as that User Defined library) → {library: id, name, count}",
            always,
            load
        ),
    ]
}

/// A brush library: its name, its brushes (in panel order) and a document holding them with the
/// resources their art uses; the document has no art of its own.
#[derive(Clone, Debug)]
pub struct BrushLibrary {
    pub name: String,
    pub brushes: Vec<Brush>,
    pub doc: Document,
}

impl Default for BrushLibrary {
    fn default() -> Self {
        Self { name: String::new(), brushes: vec![], doc: Document::new(100.0, 100.0) }
    }
}

impl BrushLibrary {
    pub fn len(&self) -> usize {
        self.brushes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.brushes.is_empty()
    }
    pub fn brush(&self, name: &str) -> Option<&Brush> {
        self.brushes.iter().find(|b| b.name == name)
    }

    /// Document `src`'s brushes `names` (empty: all of them, in panel order) as a library named
    /// `name`: the brushes and every resource their art uses.
    pub fn from_document(src: &Document, names: &[String], name: String) -> std::result::Result<Self, String> {
        let all = brush::library(src);
        // Names looked up in sets: scanning the lists for each brush made a library of N brushes
        // cost N² comparisons.
        let have: HashSet<&str> = all.iter().map(|b| b.name.as_str()).collect();
        if let Some(n) = names.iter().find(|n| !have.contains(n.as_str())) {
            return Err(format!("no brush `{n}`"));
        }
        let wanted: HashSet<&str> = names.iter().map(String::as_str).collect();
        let picked = all.iter().filter(|b| names.is_empty() || wanted.contains(b.name.as_str()));
        let clip = Clipboard::gather(src, picked.map(|b| stroke(&b.name)).collect());
        let mut doc = clip.to_document();
        for l in &mut doc.layers {
            if let Some(ch) = Arc::make_mut(l).children_mut() {
                ch.clear();
            }
        }
        // The brushes picked and those their art paints with, in the panel order of the document
        // they came from.
        let kept: HashSet<&str> = clip.brushes.iter().map(|c| c.name.as_str()).collect();
        let brushes: Vec<Brush> = all.iter().filter(|b| kept.contains(b.name.as_str())).cloned().collect();
        brush::store(&mut doc, &brushes);
        doc.title = name.clone();
        Ok(Self { name, brushes, doc })
    }
}

impl LibraryFile for BrushLibrary {
    const EXTS: &'static [&'static str] = &[BRUSHES_EXT];
    fn read(bytes: &[u8], stem: &str) -> std::result::Result<Self, String> {
        let doc = etcfile::load(bytes).map_err(|e| e.to_string())?;
        BrushLibrary::from_document(&doc, &[], stem.to_string())
    }
    fn name(&self) -> &str {
        &self.name
    }
}

/// User Defined and loaded brush libraries.
pub type Libraries = super::swatchlib::Libraries<BrushLibrary>;

/// A path stroked with brush `name` (what the resource gathering follows).
fn stroke(name: &str) -> Node {
    let mut ap = Appearance::basic(Paint::None, Paint::solid(Color::BLACK), 1.0);
    if let Some(st) = ap.stroke_mut() {
        st.brush = Some(name.into());
    }
    let line = SubPath::polyline(&[Point::new(0.0, 0.0), Point::new(100.0, 0.0)], false);
    Node::path(NodeId(0), PathData::single(line), ap)
}

// ---------- built-in libraries ----------

/// Built-in library `id`, read from the presets folder ([`crate::presets`]) once.
fn builtin_library(id: &str) -> Option<Arc<BrushLibrary>> {
    crate::presets::library(Kind::Brushes, id, |bytes, e| BrushLibrary::read(bytes, &e.name).ok())
}














































// ---------- commands ----------

fn builtin_info(e: &Entry) -> LibraryInfo {
    LibraryInfo { id: e.id.clone(), name: e.name.clone(), category: "builtIn" }
}

/// Every brush library, in menu order: built-in, User Defined, loaded.
pub fn libraries(s: &Session) -> Vec<LibraryInfo> {
    crate::presets::entries(Kind::Brushes).iter().map(builtin_info).chain(s.brush_libraries.infos().cloned()).collect()
}

/// Library `key` (an id, or a name in any case) with its info.
pub fn library(s: &Session, key: &str) -> Option<(LibraryInfo, Arc<BrushLibrary>)> {
    if let Some(found) = s.brush_libraries.get(key) {
        return Some(found);
    }
    if let Some(found) = concealed_library_id(s, key).and_then(|id| s.brush_libraries.get(&id)) {
        return Some(found);
    }
    if let Some(e) = crate::presets::entry(Kind::Brushes, key) {
        return Some((builtin_info(&e), builtin_library(&e.id)?));
    }
    if let Some(id) = libraries(s).into_iter().find(|l| l.name.eq_ignore_ascii_case(key)).map(|l| l.id) {
        return library(s, &id);
    }
    // Keep the short built-in alias, but let a user or loaded library own its display name.
    let e = key.eq_ignore_ascii_case("Artistic").then(|| crate::presets::entry(Kind::Brushes, "artistic")).flatten()?;
    Some((builtin_info(&e), builtin_library(&e.id)?))
}

fn library_param(s: &Session, p: &Value, cmd: &str) -> Result<(LibraryInfo, Arc<BrushLibrary>)> {
    let key = str_param(p, "library").ok_or_else(|| bad(cmd, "missing `library` (see brush.libraries)"))?;
    library(s, key).ok_or_else(|| bad(cmd, format!("no brush library `{key}` (see brush.libraries)")))
}

fn list(s: &mut Session, _: &Value) -> Result<Value> {
    s.brush_libraries.rescan();
    let concealed = concealed_library_ids(s, []);
    let libs: Vec<Value> = libraries(s)
        .into_iter()
        .map(|l| {
            let count = library(s, &l.id).map_or(0, |(_, lib)| lib.len());
            let (id, path) = (shown_id(&concealed, &l.id), s.brush_libraries.shown_path(&l.id));
            json!({"id": id, "name": l.name, "category": l.category, "count": count, "path": path})
        })
        .collect();
    Ok(json!({ "libraries": libs, "userFolder": s.brush_libraries.shown_user_dir() }))
}

fn get(s: &mut Session, p: &Value) -> Result<Value> {
    let (info, lib) = library_param(s, p, "brush.library")?;
    let brushes: Vec<Value> = lib.brushes.iter().map(|b| json!({"name": b.name, "type": b.kind.type_id()})).collect();
    Ok(json!({"id": shown_library_id(s, &info.id), "name": info.name, "category": info.category, "brushes": brushes}))
}

fn add(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "brush.addFromLibrary";
    let (info, lib) = library_param(s, p, C)?;
    let mut names = str_list(p, "names");
    names.extend(str_param(p, "name").map(str::to_string));
    if let Some(n) = names.iter().find(|n| lib.brush(n).is_none()) {
        return Err(bad(C, format!("no brush `{n}` in `{}`", lib.name)));
    }
    let mut picked: Vec<String> = if names.is_empty() { lib.brushes.iter().map(|b| b.name.clone()).collect() } else { names };
    // Each once, in the order asked for.
    let mut seen = HashSet::new();
    picked.retain(|n| seen.insert(n.clone()));
    let apply = bool_or(p, "apply", false);
    if apply && picked.is_empty() {
        return Err(bad(C, format!("`{}` has no brushes to apply", lib.name)));
    }
    let choices = SwatchChoices::parse(C, p)?;
    let clip = Clipboard::gather(&lib.doc, picked.iter().map(|n| stroke(n)).collect());
    // The typing `s.edit` would settle first is settled before the probe below, so the document
    // the edit installs is the one it starts from.
    if !s.previewing && super::typing_in_progress(s) {
        super::settle_interaction(s)?;
    }
    let st = s.doc()?;
    let before: HashSet<String> = brush::library(&st.doc).into_iter().map(|b| b.name).collect();
    // A library brush's name in the document: its own, or the one the import gave it.
    let target = |renamed: &[Value], n: &str| -> String {
        let to = renamed.iter().find(|r| r["kind"] == "brush" && r["from"] == n).and_then(|r| r["to"].as_str());
        to.unwrap_or(n).to_string()
    };
    // Where the brushes land, worked out on a copy that the edit then installs: nothing to add and
    // nothing to apply is no edit, and a click that only applies brushes the document has is
    // named for that.
    let mut probe = (*st.doc).clone();
    let imported = clip.import_into(&mut probe, &choices, false)?;
    let doc_names: Vec<String> = picked.iter().map(|n| target(&imported.renamed, n)).collect();
    let paths = if apply { brush_targets(&st.doc, &roots(s, p)?) } else { vec![] };
    let paint = stroke_paint(s);
    if imported.added > 0 || !paths.is_empty() {
        let label = if imported.added > 0 { "Add to Brushes" } else { "Apply Brush" };
        s.edit(label, |d, _| {
            *d = probe;
            if let Some(first) = doc_names.first() {
                d.unknown.insert(brush::CURRENT_KEY.into(), json!(first));
                apply_brush(d, &paths, first, &paint);
            }
            Ok(())
        })?;
    } else if let Some(first) = doc_names.first() {
        s.execute("brush.setCurrent", &json!({ "name": first }))?;
    }
    let (added, existing): (Vec<&String>, Vec<&String>) = doc_names.iter().partition(|n| !before.contains(*n));
    let applied: Vec<u64> = paths.iter().map(|id| id.0).collect();
    Ok(json!({"library": shown_library_id(s, &info.id), "names": doc_names, "added": added, "existing": existing, "applied": applied}))
}

fn save(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "brush.saveLibrary";
    let st = s.doc()?;
    let name = str_param(p, "name").map(str::trim).filter(|n| !n.is_empty()).map_or_else(|| stem(&st.title()).to_string(), str::to_string);
    let lib = BrushLibrary::from_document(&st.doc, &str_list(p, "names"), name).map_err(|e| bad(C, e))?;
    if lib.is_empty() {
        return Err(bad(C, "the document has no brushes to save"));
    }
    let mut out = json!({ "count": lib.len() });
    let bytes = etcfile::save_file(&lib.doc);
    s.brush_libraries.write(p, &lib.name, BRUSHES_EXT, FileData::Binary(bytes), &mut out, C)?;
    Ok(out)
}

fn load(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "brush.loadLibrary";
    let (info, lib) = s.brush_libraries.load(p, C, |bytes, file| {
        let loaded = super::fileio::load(file, bytes).map_err(|e| bad(C, e.to_string()))?;
        if loaded.preview_only {
            return Err(bad(C, "only this Affinity file's embedded preview could be read; use File › Open to see it with its warning"));
        }
        BrushLibrary::from_document(&loaded.doc, &[], stem(file).to_string()).map_err(|e| bad(C, e))
    })?;
    Ok(json!({"library": shown_library_id(s, &info.id), "name": info.name, "count": lib.len()}))
}
