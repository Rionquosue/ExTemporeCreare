//! Export slices (`filestandard.md` §7A, `uistandards.md` §9): named, non-destructive output
//! regions and the export dot grid, kept as document data in `Document::extra["export"]`
//! (`{"grid": {"visible", "snapEnabled", "spacingPx"}, "slices": [slice…]}`), which `.etc` saves
//! mirror to `project.json`'s `export` object. A slice is `{"id", "name", "rectPx": {"x", "y",
//! "width", "height"}, "aspectRatio": {"x", "y"}, "format", "scale", "exportProfileRef",
//! "visible"}`, in document pixels (one per point). Every change is one undo step.

use serde_json::{Map, Value, json};

use super::{CommandSpec, Result, Session, bad, cmd, has_doc};

/// The `Document::extra` key of the export data.
pub const EXPORT_KEY: &str = "export";

/// The output formats a slice can have.
pub const SLICE_FORMATS: [&str; 3] = ["png", "jpg", "webp"];

/// The export grid's spacing when none is set, in document pixels.
pub const DEFAULT_GRID_SPACING: f64 = 32.0;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            query "export.slice.list",
            "Export Slices",
            [],
            None,
            "{} → {slices: [{id, name, rectPx: {x, y, width, height}, aspectRatio: {x, y}, format, scale, exportProfileRef, visible}…] (in order), grid: {visible, snapEnabled, spacingPx}, contentBounds: {x, y, width, height} | null (the visible art an export draws, effects included; null: nothing to export)}",
            has_doc,
            list
        ),
        cmd!(
            "export.slice.add",
            "New Export Slice",
            [],
            None,
            "{x, y, width, height (document pixels; width and height above 0), name? (default: Slice <n>), aspectRatio?: {x, y} (positive; default: width:height in lowest whole terms), format?: png|jpg|webp (png), scale?: above 0 (1), exportProfileRef? (builtin:sRGB)} add a slice after the others, one undo step → {id}",
            has_doc,
            add
        ),
        cmd!(
            "export.slice.update",
            "Edit Export Slice",
            [],
            None,
            "{id, x?, y?, width?, height?, name?, aspectRatio?: {x, y}, format?, scale?, exportProfileRef?, visible?: bool} change a slice, one undo step (none when nothing changes) → {id}",
            has_doc,
            update
        ),
        cmd!("export.slice.delete", "Delete Export Slice", [], None, "{id} remove a slice, one undo step → {id}", has_doc, delete),
        cmd!(
            "export.slice.duplicate",
            "Duplicate Export Slice",
            [],
            None,
            "{id} a copy of a slice right after it, named <name> copy, one undo step → {id}",
            has_doc,
            duplicate
        ),
        cmd!(
            "export.grid.set",
            "Export Grid",
            [],
            None,
            "{visible?: bool, snapEnabled?: bool, spacingPx?: above 0} the export dot grid, one undo step (none when nothing changes) → {grid}",
            has_doc,
            grid_set
        ),
    ]
}

/// The export data of `doc`, with an empty slice list and the default grid when it has none.
fn export_data(doc: &vectorcraft_doc::Document) -> Map<String, Value> {
    let mut e = doc.extra.get(EXPORT_KEY).and_then(Value::as_object).cloned().unwrap_or_default();
    if !e.get("slices").is_some_and(Value::is_array) {
        e.insert("slices".into(), json!([]));
    }
    let grid = e.entry("grid").or_insert_with(|| json!({}));
    if !grid.is_object() {
        *grid = json!({});
    }
    let g = grid.as_object_mut().expect("an object");
    g.entry("visible").or_insert(json!(false));
    g.entry("snapEnabled").or_insert(json!(false));
    g.entry("spacingPx").or_insert(json!(DEFAULT_GRID_SPACING));
    e
}

/// The slices of export data `e`.
fn slices(e: &mut Map<String, Value>) -> &mut Vec<Value> {
    e.get_mut("slices").and_then(Value::as_array_mut).expect("export_data makes the list")
}

/// Commit export data `e` as one undo step named `label`.
fn commit(s: &mut Session, label: &str, e: Map<String, Value>) -> Result<()> {
    s.edit(label, |doc, _| {
        doc.extra.insert(EXPORT_KEY.into(), Value::Object(e));
        Ok(())
    })
}

/// A finite number `key` of `p` (`None`: absent), refused when it is not one or, with `positive`, not above 0.
fn number(p: &Value, key: &str, positive: bool, c: &str) -> Result<Option<f64>> {
    match p.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => match v.as_f64().filter(|n| n.is_finite() && (!positive || *n > 0.0)) {
            Some(n) => Ok(Some(n)),
            None => Err(bad(c, format!("`{key}` must be a {}number", if positive { "positive " } else { "" }))),
        },
    }
}

/// `width:height` in lowest whole terms (rounded to whole pixels; at least 1:1).
fn ratio_of(width: f64, height: f64) -> Value {
    let (mut a, mut b) = ((width.round() as u64).max(1), (height.round() as u64).max(1));
    let (w, h) = (a, b);
    while b != 0 {
        (a, b) = (b, a % b);
    }
    json!({ "x": w / a, "y": h / a })
}

/// The `aspectRatio` of `p`, checked (`None`: absent).
fn aspect(p: &Value, c: &str) -> Result<Option<Value>> {
    let Some(r) = p.get("aspectRatio").filter(|v| !v.is_null()) else { return Ok(None) };
    let x = number(r, "x", true, c)?;
    let y = number(r, "y", true, c)?;
    match (x, y) {
        (Some(x), Some(y)) => Ok(Some(json!({ "x": x, "y": y }))),
        _ => Err(bad(c, "`aspectRatio` must be {x, y}, both positive")),
    }
}

/// Copy the output fields `p` gives (name, aspect ratio, format, scale, profile, visibility) into `slice`.
fn set_outputs(slice: &mut Map<String, Value>, p: &Value, c: &str) -> Result<()> {
    if let Some(n) = p.get("name").filter(|v| !v.is_null()) {
        let n = n.as_str().map(str::trim).filter(|n| !n.is_empty()).ok_or_else(|| bad(c, "`name` must be a non-empty string"))?;
        slice.insert("name".into(), json!(n));
    }
    if let Some(r) = aspect(p, c)? {
        slice.insert("aspectRatio".into(), r);
    }
    if let Some(f) = p.get("format").filter(|v| !v.is_null()) {
        let f = f.as_str().filter(|f| SLICE_FORMATS.contains(f)).ok_or_else(|| bad(c, format!("`format` must be one of {}", SLICE_FORMATS.join(", "))))?;
        slice.insert("format".into(), json!(f));
    }
    if let Some(k) = number(p, "scale", true, c)? {
        slice.insert("scale".into(), json!(k));
    }
    if let Some(r) = p.get("exportProfileRef").filter(|v| !v.is_null()) {
        let r = r.as_str().filter(|r| !r.is_empty()).ok_or_else(|| bad(c, "`exportProfileRef` must be a string"))?;
        slice.insert("exportProfileRef".into(), json!(r));
    }
    if let Some(v) = p.get("visible").filter(|v| !v.is_null()) {
        slice.insert("visible".into(), json!(v.as_bool().ok_or_else(|| bad(c, "`visible` must be true or false"))?));
    }
    Ok(())
}

/// The index of slice `id` (the `id` of `p`) in `list`.
fn find(list: &[Value], p: &Value, c: &str) -> Result<usize> {
    let id = p.get("id").and_then(Value::as_str).ok_or_else(|| bad(c, "`id` must be a slice id (see export.slice.list)"))?;
    list.iter().position(|s| s.get("id").and_then(Value::as_str) == Some(id)).ok_or_else(|| bad(c, format!("no slice `{id}`")))
}

/// A new slice id: `slice-<n>`, one past the highest number in use.
fn new_id(list: &[Value]) -> String {
    let top = list
        .iter()
        .filter_map(|s| s.get("id")?.as_str()?.strip_prefix("slice-")?.parse::<u64>().ok())
        .max()
        .unwrap_or(0);
    format!("slice-{:03}", top + 1)
}

fn list(s: &mut Session, _p: &Value) -> Result<Value> {
    let doc = &s.doc()?.doc;
    let mut e = export_data(doc);
    let bounds = vectorcraft_render::encode::art_bounds(doc).map(|r| json!({ "x": r.x0, "y": r.y0, "width": r.width(), "height": r.height() }));
    e.insert("contentBounds".into(), bounds.unwrap_or(Value::Null));
    Ok(Value::Object(e))
}

fn add(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "export.slice.add";
    let need = |k: &str, positive: bool| number(p, k, positive, C)?.ok_or_else(|| bad(C, format!("`{k}` is needed")));
    let (x, y, w, h) = (need("x", false)?, need("y", false)?, need("width", true)?, need("height", true)?);
    let mut e = export_data(&s.doc()?.doc);
    let list = slices(&mut e);
    let id = new_id(list);
    let mut slice = Map::new();
    slice.insert("id".into(), json!(id));
    slice.insert("name".into(), json!(format!("Slice {}", list.len() + 1)));
    slice.insert("rectPx".into(), json!({ "x": x, "y": y, "width": w, "height": h }));
    slice.insert("aspectRatio".into(), ratio_of(w, h));
    slice.insert("format".into(), json!("png"));
    slice.insert("scale".into(), json!(1.0));
    slice.insert("exportProfileRef".into(), json!("builtin:sRGB"));
    slice.insert("visible".into(), json!(true));
    set_outputs(&mut slice, p, C)?;
    list.push(Value::Object(slice));
    commit(s, "New Export Slice", e)?;
    Ok(json!({ "id": id }))
}

fn update(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "export.slice.update";
    let mut e = export_data(&s.doc()?.doc);
    let list = slices(&mut e);
    let i = find(list, p, C)?;
    let before = list[i].clone();
    let slice = list[i].as_object_mut().ok_or_else(|| bad(C, "the slice is not an object"))?;
    let mut rect = slice.get("rectPx").cloned().unwrap_or_else(|| json!({}));
    for (key, positive) in [("x", false), ("y", false), ("width", true), ("height", true)] {
        if let Some(n) = number(p, key, positive, C)? {
            rect[key] = json!(n);
        }
    }
    slice.insert("rectPx".into(), rect);
    set_outputs(slice, p, C)?;
    let id = slice.get("id").cloned().unwrap_or(Value::Null);
    if list[i] != before {
        commit(s, "Edit Export Slice", e)?;
    }
    Ok(json!({ "id": id }))
}

fn delete(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "export.slice.delete";
    let mut e = export_data(&s.doc()?.doc);
    let list = slices(&mut e);
    let i = find(list, p, C)?;
    let gone = list.remove(i);
    commit(s, "Delete Export Slice", e)?;
    Ok(json!({ "id": gone.get("id").cloned().unwrap_or(Value::Null) }))
}

fn duplicate(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "export.slice.duplicate";
    let mut e = export_data(&s.doc()?.doc);
    let list = slices(&mut e);
    let i = find(list, p, C)?;
    let id = new_id(list);
    let mut copy = list[i].clone();
    let name = copy.get("name").and_then(Value::as_str).unwrap_or("Slice").to_string();
    copy["id"] = json!(id);
    copy["name"] = json!(format!("{name} copy"));
    list.insert(i + 1, copy);
    commit(s, "Duplicate Export Slice", e)?;
    Ok(json!({ "id": id }))
}

fn grid_set(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "export.grid.set";
    let mut e = export_data(&s.doc()?.doc);
    let before = e.get("grid").cloned();
    let grid = e.get_mut("grid").and_then(Value::as_object_mut).expect("export_data makes the grid");
    for key in ["visible", "snapEnabled"] {
        if let Some(v) = p.get(key).filter(|v| !v.is_null()) {
            grid.insert(key.into(), json!(v.as_bool().ok_or_else(|| bad(C, format!("`{key}` must be true or false")))?));
        }
    }
    if let Some(n) = number(p, "spacingPx", true, C)? {
        grid.insert("spacingPx".into(), json!(n));
    }
    let after = Value::Object(grid.clone());
    if Some(&after) != before.as_ref() {
        commit(s, "Export Grid", e)?;
    }
    Ok(json!({ "grid": after }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slices_are_added_edited_copied_and_removed_one_undo_step_each() {
        let mut s = Session::new();
        s.execute("file.new", &json!({"width": 200, "height": 100})).unwrap();
        let id = s.execute("export.slice.add", &json!({"x": -10, "y": 5, "width": 160, "height": 90})).unwrap()["id"].clone();
        assert_eq!(id, "slice-001");
        let l = s.execute("export.slice.list", &json!({})).unwrap();
        assert_eq!(l["slices"][0]["aspectRatio"], json!({"x": 16, "y": 9}));
        assert_eq!(l["grid"]["spacingPx"], DEFAULT_GRID_SPACING);
        assert!(l["contentBounds"].is_null(), "an empty document has nothing to export");
        s.execute("export.slice.update", &json!({"id": id, "x": 0, "name": "Detail", "format": "jpg"})).unwrap();
        let copy = s.execute("export.slice.duplicate", &json!({"id": id})).unwrap()["id"].clone();
        assert_eq!(copy, "slice-002");
        let l = s.execute("export.slice.list", &json!({})).unwrap();
        assert_eq!(l["slices"][1]["name"], "Detail copy");
        assert_eq!(l["slices"][0]["rectPx"]["x"], 0.0);
        assert!(s.execute("export.slice.update", &json!({"id": id, "width": 0})).is_err());
        s.execute("export.slice.delete", &json!({"id": copy})).unwrap();
        s.execute("edit.undo", &json!({})).unwrap();
        assert_eq!(s.execute("export.slice.list", &json!({})).unwrap()["slices"].as_array().unwrap().len(), 2);
        s.execute("export.grid.set", &json!({"snapEnabled": true, "spacingPx": 10})).unwrap();
        assert_eq!(s.execute("export.slice.list", &json!({})).unwrap()["grid"]["snapEnabled"], true);
    }
}

// GLOSSARY
// - `EXPORT_KEY`: the `Document::extra` key of the export data, "export".
// - `SLICE_FORMATS`: png, jpg, webp.
// - `DEFAULT_GRID_SPACING`: the grid spacing when none is set, 32 px.
// - `specs`: the export slice and grid commands.
// - `export_data`: the document's export data, with defaults filled in.
// - `slices`: the slice list of export data.
// - `commit`: store export data as one undo step.
// - `number`: a checked number parameter.
// - `ratio_of`: width:height in lowest whole terms.
// - `aspect`: a checked `aspectRatio` parameter.
// - `set_outputs`: copy name, ratio, format, scale, profile and visibility into a slice.
// - `find`: a slice's index by id.
// - `new_id`: the next `slice-<n>` id.
// - `list`: `export.slice.list` (slices, grid, content bounds).
// - `add`: `export.slice.add`.
// - `update`: `export.slice.update`.
// - `delete`: `export.slice.delete`.
// - `duplicate`: `export.slice.duplicate`.
// - `grid_set`: `export.grid.set`.
