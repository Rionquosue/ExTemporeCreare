//! What the Layers panel shows: the active document's stack as a tree of rows, topmost first
//! (`uistandards.md` §7.1). Edits go through engine commands (`layer.setProps`,
//! `transparency.set`, `layer.new`, `layer.delete`, `layer.setCurrent`).

use serde_json::{Value, json};
use vectorcraft_doc::{Node, NodeKind};

use crate::calls::guarded;
use crate::sessions::Bridge;

/// A row's type as the panel names it: `folder` (a layer or group, which holds rows), `raster`
/// (an image, the Brush's splotches included), `guide` (a ruler guide) or `vector` (any other art).
fn kind_of(n: &Node) -> &'static str {
    match &n.kind {
        NodeKind::Layer { .. } | NodeKind::Group { .. } => "folder",
        NodeKind::Image(_) => "raster",
        NodeKind::RulerGuide { .. } => "guide",
        _ => "vector",
    }
}

/// The name a row shows: its own, else its kind's.
fn name_of(n: &Node) -> String {
    if let Some(name) = n.name.as_deref().filter(|s| !s.is_empty()) {
        return name.to_string();
    }
    match &n.kind {
        NodeKind::Layer { .. } => "Layer",
        NodeKind::Group { .. } => "Group",
        NodeKind::Image(_) => "Image",
        NodeKind::Text(_) => "Text",
        NodeKind::Path { .. } | NodeKind::Compound { .. } => "Path",
        NodeKind::RulerGuide { .. } => "Guide",
        _ => "Object",
    }
    .to_string()
}

/// One row and, for a folder, its rows topmost first.
fn row(n: &Node, current: Option<u64>) -> Value {
    let kind = kind_of(n);
    let mut v = json!({
        "id": n.id.0,
        "name": name_of(n),
        "kind": kind,
        "visible": n.visible,
        "locked": n.locked,
        "opacity": (f64::from(n.opacity) * 100.0).round(),
        "blend": serde_json::to_value(n.blend).unwrap_or(Value::Null),
        "current": current == Some(n.id.0),
    });
    if kind == "folder"
        && let Some(children) = n.children()
    {
        v["children"] = Value::Array(children.iter().rev().map(|c| row(c, current)).collect());
    }
    v
}

/// The active document's rows, topmost first → `{"rows": [{"id", "name", "kind":
/// "folder"|"raster"|"vector"|"guide", "visible", "locked", "opacity" (0–100), "blend" (engine
/// blend mode name), "current" (the layer new art goes into), "children"? (a folder's rows)}]}`;
/// `{"rows": []}` with no document open.
pub fn layers(handle: i64) -> String {
    guarded(handle, |b: &mut Bridge| {
        let Ok(st) = b.session.doc() else { return Ok(json!({ "rows": [] })) };
        let current = st.current_layer().map(|id| id.0);
        Ok(json!({ "rows": st.doc.layers.iter().rev().map(|l| row(l, current)).collect::<Vec<_>>() }))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{calls, sessions};

    #[test]
    fn rows_are_topmost_first_with_their_kinds() {
        let h = sessions::create();
        calls::execute(h, "file.new", r#"{"width": 100, "height": 100}"#);
        calls::execute(h, "shape.rectangle", r#"{"x": 0, "y": 0, "width": 10, "height": 10}"#);
        calls::execute(h, "layer.new", r#"{"name": "Top", "top": true}"#);
        let v: Value = serde_json::from_str(&layers(h)).unwrap();
        let rows = &v["ok"]["rows"];
        assert_eq!(rows[0]["name"], "Top");
        assert_eq!(rows[1]["kind"], "folder");
        assert_eq!(rows[1]["children"][0]["kind"], "vector");
        assert_eq!(rows[1]["children"][0]["opacity"], 100.0);
        assert_eq!(rows[1]["children"][0]["blend"], "Normal");
        sessions::destroy(h);
    }
}

// GLOSSARY
// - `kind_of`: a row's type: folder, raster, guide or vector.
// - `name_of`: a row's name, else its kind's.
// - `row`: one row with a folder's rows, topmost first.
// - `layers`: the active document's rows for the Layers panel.
