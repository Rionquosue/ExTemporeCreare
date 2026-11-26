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
    if n.extras().texture.is_some() {
        return "texture";
    }
    if n.extras().filter.is_some() {
        return "filter";
    }
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
        "fill": (f64::from(n.extras().fill_opacity.unwrap_or(1.0)) * 100.0).round(),
        "mask": n.extras().mask.as_deref().map_or(Value::Null, |m| json!({ "enabled": !m.disabled, "inverted": m.invert, "background": if m.clip { "black" } else { "white" } })),
        "current": current == Some(n.id.0),
    });
    if let Some(t) = n.extras().texture.as_deref() {
        v["scale"] = json!(t.scale);
    }
    if let Some(fl) = n.extras().filter.as_deref() {
        v["effect"] = json!(fl.effect);
    }
    if kind == "folder"
        && let Some(children) = n.children()
    {
        v["children"] = Value::Array(children.iter().rev().map(|c| row(c, current)).collect());
    }
    v
}

/// The active document's rows, topmost first → `{"rows": [{"id", "name", "kind":
/// "folder"|"raster"|"vector"|"guide", "visible", "locked", "opacity" (0–100), "blend" (engine
/// blend mode name), "fill" (0–100, Photoshop's Fill), "mask" (null or {"enabled", "inverted", "background": white|black}), "current" (the layer new art goes into), "children"? (a folder's rows)}], "selected" (how many objects are
/// selected), "editingMask" (the id of the layer whose mask is being painted, or null)}`;
/// `{"rows": []}` with no document open.
pub fn layers(handle: i64) -> String {
    guarded(handle, |b: &mut Bridge| {
        let Ok(st) = b.session.doc() else { return Ok(json!({ "rows": [] })) };
        let current = st.current_layer().map(|id| id.0);
        Ok(json!({
            "rows": st.doc.layers.iter().rev().map(|l| row(l, current)).collect::<Vec<_>>(),
            "selected": st.selection.objects.len(),
            "editingMask": st.doc.mask_edit.map(|m| m.object.0),
        }))
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
        assert_eq!(rows[1]["children"][0]["fill"], 100.0);
        sessions::destroy(h);
    }

    /// Fill (as in Photoshop) fades a layer's content and leaves its styles: at Fill 0 the shape
    /// is gone and its drop shadow stays, knocked out under the shape.
    #[test]
    fn fill_fades_content_but_not_styles() {
        let h = sessions::create();
        let ok = |r: String| -> Value { serde_json::from_str::<Value>(&r).unwrap().get("ok").cloned().unwrap_or_else(|| panic!("{r}")) };
        ok(calls::execute(h, "file.new", r#"{"width": 200, "height": 200, "backgroundContents": "transparent"}"#));
        ok(calls::execute(h, "shape.rectangle", r#"{"x": 50, "y": 50, "width": 80, "height": 80}"#));
        ok(calls::execute(h, "paint.setFill", r##"{"color": "#ff0000"}"##));
        ok(calls::execute(h, "paint.setStroke", r#"{"none": true}"#));
        let v: Value = serde_json::from_str(&layers(h)).unwrap();
        let layer = v["ok"]["rows"][0]["id"].as_u64().unwrap();
        ok(calls::execute(h, "effect.apply", &json!({"ids": [layer], "effect": "stylize.dropShadow"}).to_string()));
        let at = |px: &[u8], x: usize, y: usize| px[(y * 200 + x) * 4..(y * 200 + x) * 4 + 4].to_vec();
        let full = calls::render(h, 0.0, 0.0, 200, 200, 1.0).unwrap();
        assert!(at(&full, 90, 90)[0] > 200 && at(&full, 90, 90)[3] > 200, "content shows at Fill 100: {:?}", at(&full, 90, 90));
        ok(calls::execute(h, "layer.setFill", &json!({"ids": [layer], "fill": 0}).to_string()));
        let v: Value = serde_json::from_str(&layers(h)).unwrap();
        assert_eq!(v["ok"]["rows"][0]["fill"], 0.0);
        // Over the white artboard: the content gone (white, the shadow knocked out under it) and
        // the shadow (grey) beyond it.
        let faded = calls::render(h, 0.0, 0.0, 200, 200, 1.0).unwrap();
        assert!(at(&faded, 90, 90)[..3].iter().all(|c| *c > 245), "content gone at Fill 0, shadow knocked out: {:?}", at(&faded, 90, 90));
        let grey = |p: Vec<u8>| p[0] < 235 && p[0].abs_diff(p[1]) < 8;
        let shadow = (100..200).flat_map(|y| (100..200).map(move |x| (x, y))).filter(|(x, y)| *x > 134 || *y > 134).any(|(x, y)| grey(at(&faded, x, y)));
        assert!(shadow, "the drop shadow stays");
        // An object row: Fill and a mask work on it too.
        ok(calls::execute(h, "shape.rectangle", r#"{"x": 10, "y": 150, "width": 30, "height": 30}"#));
        ok(calls::execute(h, "paint.setFill", r##"{"color": "#0000ff"}"##));
        let v: Value = serde_json::from_str(&layers(h)).unwrap();
        let object = v["ok"]["rows"][0]["children"][0]["id"].as_u64().unwrap();
        ok(calls::execute(h, "layer.setFill", &json!({"ids": [object], "fill": 0}).to_string()));
        let px = calls::render(h, 0.0, 0.0, 200, 200, 1.0).unwrap();
        assert!(at(&px, 25, 165)[..3].iter().all(|c| *c > 245), "an object's Fill fades it: {:?}", at(&px, 25, 165));
        ok(calls::execute(h, "layer.setFill", &json!({"ids": [object], "fill": 100}).to_string()));
        ok(calls::execute(h, "layer.mask.add", &json!({"id": object, "mode": "hide"}).to_string()));
        let px = calls::render(h, 0.0, 0.0, 200, 200, 1.0).unwrap();
        assert!(at(&px, 25, 165)[..3].iter().all(|c| *c > 245), "an object's mask hides it: {:?}", at(&px, 25, 165));
        sessions::destroy(h);
    }

    /// A layer mask shows the layer where it is white and hides it where black: Hide All, Reveal
    /// All, the selection, inverted, disabled.
    #[test]
    fn layer_masks_show_white_and_hide_black() {
        let h = sessions::create();
        let ok = |r: String| -> Value { serde_json::from_str::<Value>(&r).unwrap().get("ok").cloned().unwrap_or_else(|| panic!("{r}")) };
        ok(calls::execute(h, "file.new", r#"{"width": 200, "height": 200, "backgroundContents": "transparent"}"#));
        ok(calls::execute(h, "shape.rectangle", r#"{"x": 20, "y": 20, "width": 160, "height": 160}"#));
        ok(calls::execute(h, "paint.setFill", r##"{"color": "#ff0000"}"##));
        let v: Value = serde_json::from_str(&layers(h)).unwrap();
        let layer = v["ok"]["rows"][0]["id"].as_u64().unwrap();
        // How red the pixel is over the white artboard: 255 where the red layer shows, 0 where hidden.
        let alpha = |x: usize, y: usize| 255 - calls::render(h, 0.0, 0.0, 200, 200, 1.0).unwrap()[(y * 200 + x) * 4 + 1];
        let id = json!({"id": layer}).to_string();
        ok(calls::execute(h, "layer.mask.add", &json!({"id": layer, "mode": "hide"}).to_string()));
        assert!(alpha(100, 100) < 10, "Hide All hides");
        let v: Value = serde_json::from_str(&layers(h)).unwrap();
        assert_eq!(v["ok"]["rows"][0]["mask"]["enabled"], true);
        ok(calls::execute(h, "layer.mask.invert", &id));
        assert!(alpha(100, 100) > 245, "inverted, it shows");
        ok(calls::execute(h, "layer.mask.delete", &id));
        ok(calls::execute(h, "layer.mask.add", &json!({"id": layer, "mode": "reveal"}).to_string()));
        assert!(alpha(100, 100) > 245, "Reveal All shows");
        // A small shape selected: only it shows.
        ok(calls::execute(h, "shape.rectangle", r#"{"x": 40, "y": 40, "width": 30, "height": 30}"#));
        ok(calls::execute(h, "layer.mask.fromSelection", &id));
        ok(calls::execute(h, "edit.clear", "{}"));
        ok(calls::execute(h, "select.none", "{}"));
        assert!(alpha(55, 55) > 245 && alpha(150, 150) < 10, "the selection shows, the rest is hidden: {} {}", alpha(55, 55), alpha(150, 150));
        ok(calls::execute(h, "layer.mask.setEnabled", &json!({"id": layer, "enabled": false}).to_string()));
        assert!(alpha(150, 150) > 245, "disabled, all shows");
        let info = ok(calls::execute(h, "layer.mask.info", &id));
        assert_eq!((info["mask"]["enabled"].as_bool(), info["mask"]["background"].as_str()), (Some(false), Some("black")));
        // Painting black into a Reveal All mask hides the layer there.
        ok(calls::execute(h, "layer.mask.delete", &id));
        ok(calls::execute(h, "layer.mask.add", &json!({"id": layer, "mode": "reveal"}).to_string()));
        ok(calls::execute(h, "transparency.editOpacityMask", &id));
        let v: Value = serde_json::from_str(&layers(h)).unwrap();
        assert_eq!(v["ok"]["editingMask"], layer);
        ok(calls::execute(h, "shape.rectangle", r#"{"x": 120, "y": 120, "width": 40, "height": 40}"#));
        ok(calls::execute(h, "paint.setFill", r##"{"color": "#000000"}"##));
        ok(calls::execute(h, "transparency.stopEditingOpacityMask", "{}"));
        assert!(alpha(140, 140) < 10 && alpha(60, 60) > 245, "painted black hides, the rest shows: {} {}", alpha(140, 140), alpha(60, 60));
        sessions::destroy(h);
    }

    /// The layer styles paint from the layer's silhouette over it, and a Fill of 0 leaves them:
    /// Color Overlay, Stroke, Gradient Overlay, Bevel & Emboss.
    #[test]
    fn layer_styles_paint_over_the_layer_at_any_fill() {
        let h = sessions::create();
        let ok = |r: String| -> Value { serde_json::from_str::<Value>(&r).unwrap().get("ok").cloned().unwrap_or_else(|| panic!("{r}")) };
        ok(calls::execute(h, "file.new", r#"{"width": 200, "height": 200}"#));
        ok(calls::execute(h, "shape.rectangle", r#"{"x": 50, "y": 50, "width": 100, "height": 100}"#));
        ok(calls::execute(h, "paint.setFill", r##"{"color": "#ff0000"}"##));
        ok(calls::execute(h, "paint.setStroke", r#"{"none": true}"#));
        let v: Value = serde_json::from_str(&layers(h)).unwrap();
        let layer = v["ok"]["rows"][0]["id"].as_u64().unwrap();
        let at = |x: usize, y: usize| calls::render(h, 0.0, 0.0, 200, 200, 1.0).unwrap()[(y * 200 + x) * 4..(y * 200 + x) * 4 + 3].to_vec();
        let style = |effect: &str, params: Value| {
            ok(calls::execute(h, "effect.apply", &json!({"ids": [layer], "effect": effect, "params": params}).to_string()));
        };
        style("style.colorOverlay", json!({"color": "#00ff00"}));
        ok(calls::execute(h, "layer.setFill", &json!({"ids": [layer], "fill": 0}).to_string()));
        assert_eq!(at(100, 100), vec![0, 255, 0], "the overlay shows at Fill 0");
        style("style.stroke", json!({"size": 6, "color": "#0000ff"}));
        assert_eq!(at(47, 100), vec![0, 0, 255], "the stroke outside the edge");
        assert_eq!(at(40, 100), vec![255, 255, 255], "not beyond its size");
        ok(calls::execute(h, "edit.undo", "{}"));
        ok(calls::execute(h, "edit.undo", "{}"));
        style("style.gradientOverlay", json!({"angle": 0}));
        let (left, right) = (at(52, 100), at(148, 100));
        assert!(left[0] < 40 && right[0] > 215, "black to white: {left:?} {right:?}");
        ok(calls::execute(h, "edit.undo", "{}"));
        ok(calls::execute(h, "layer.setFill", &json!({"ids": [layer], "fill": 100}).to_string()));
        style("style.bevelEmboss", json!({"size": 8}));
        let (lit, shaded, middle) = (at(53, 53), at(147, 147), at(100, 100));
        assert!(lit[0] > 60 && shaded[1] < 200 && middle == vec![0, 255, 0], "over the green overlay, lit top left, shaded bottom right: {lit:?} {shaded:?} {middle:?}");
        sessions::destroy(h);
    }

    /// A texture layer tiles its image as a live pattern over the canvas at its scale, is a
    /// "texture" row, and sets no export bounds.
    #[test]
    fn a_texture_layer_tiles_its_image() {
        let h = sessions::create();
        let ok = |r: String| -> Value { serde_json::from_str::<Value>(&r).unwrap().get("ok").cloned().unwrap_or_else(|| panic!("{r}")) };
        ok(calls::execute(h, "file.new", r#"{"width": 200, "height": 200}"#));
        let dir = std::env::temp_dir().join(format!("etc-bridge-texture-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let png = dir.join("checks.png");
        image::RgbImage::from_fn(2, 1, |x, _| if x == 0 { image::Rgb([255, 0, 0]) } else { image::Rgb([0, 0, 255]) }).save(&png).unwrap();
        let made = ok(calls::execute(h, "texture.new", &json!({"path": png.to_string_lossy(), "name": "Checks", "scale": 10}).to_string()));
        let id = made["id"].as_u64().unwrap();
        let at = |x: usize, y: usize| calls::render(h, 0.0, 0.0, 200, 200, 1.0).unwrap()[(y * 200 + x) * 4..(y * 200 + x) * 4 + 3].to_vec();
        // Image smoothing blends a little at the pixels' edges.
        let red = |p: Vec<u8>| p[0] > 200 && p[2] < 60;
        let blue = |p: Vec<u8>| p[2] > 200 && p[0] < 60;
        assert!(red(at(5, 5)) && blue(at(15, 5)) && red(at(25, 5)), "tiles at 10 pt a pixel: {:?} {:?} {:?}", at(5, 5), at(15, 5), at(25, 5));
        let v: Value = serde_json::from_str(&layers(h)).unwrap();
        let row = &v["ok"]["rows"][0]["children"][0];
        assert_eq!((row["kind"].as_str(), row["scale"].as_f64(), row["id"].as_u64()), (Some("texture"), Some(10.0), Some(id)));
        assert!(ok(calls::execute(h, "export.slice.list", "{}"))["contentBounds"].is_null(), "a texture sets no export bounds");
        ok(calls::execute(h, "texture.set", &json!({"id": id, "scale": 20}).to_string()));
        assert!(red(at(8, 5)) && blue(at(30, 5)), "scaled up: {:?} {:?}", at(8, 5), at(30, 5));
        let _ = std::fs::remove_dir_all(&dir);
        sessions::destroy(h);
    }

    /// A filter layer filters everything drawn below it on the page, in other folders too, and
    /// nothing above it; hidden, it filters nothing.
    #[test]
    fn a_filter_layer_filters_everything_below_it() {
        let h = sessions::create();
        let ok = |r: String| -> Value { serde_json::from_str::<Value>(&r).unwrap().get("ok").cloned().unwrap_or_else(|| panic!("{r}")) };
        ok(calls::execute(h, "file.new", r#"{"width": 200, "height": 200}"#));
        // Below, in the first layer: a black square; then a second layer with the filter and, above
        // it, a black square of its own.
        ok(calls::execute(h, "shape.rectangle", r#"{"x": 20, "y": 20, "width": 60, "height": 60}"#));
        ok(calls::execute(h, "paint.setFill", r##"{"color": "#000000"}"##));
        ok(calls::execute(h, "layer.new", r#"{"name": "Top", "top": true}"#));
        let filters = ok(calls::execute(h, "filter.list", "{}"));
        assert!(filters["filters"].as_array().unwrap().iter().any(|f| f["effect"] == "pixelate.crystallize"));
        let made = ok(calls::execute(h, "filter.new", r#"{"effect": "sketch.photocopy"}"#));
        ok(calls::execute(h, "shape.rectangle", r#"{"x": 120, "y": 120, "width": 60, "height": 60}"#));
        ok(calls::execute(h, "paint.setFill", r##"{"color": "#000000"}"##));
        let v: Value = serde_json::from_str(&layers(h)).unwrap();
        let top = &v["ok"]["rows"][0]["children"];
        assert!(top.as_array().unwrap().iter().any(|r| r["kind"] == "filter" && r["effect"] == "sketch.photocopy"));
        let px = |p: &[u8], x: usize, y: usize| p[(y * 200 + x) * 4..(y * 200 + x) * 4 + 3].to_vec();
        let filtered = calls::render(h, 0.0, 0.0, 200, 200, 1.0).unwrap();
        ok(calls::execute(h, "layer.setProps", &json!({"id": made["id"], "visible": false}).to_string()));
        let plain = calls::render(h, 0.0, 0.0, 200, 200, 1.0).unwrap();
        let below_changed = (20..80).flat_map(|y| (20..80).map(move |x| (x, y))).any(|(x, y)| px(&filtered, x, y) != px(&plain, x, y));
        let above_same = (125..175).flat_map(|y| (125..175).map(move |x| (x, y))).all(|(x, y)| px(&filtered, x, y) == px(&plain, x, y));
        assert!(below_changed, "the square below in another layer is filtered");
        assert!(above_same, "the square above is not");
        sessions::destroy(h);
    }
}


// GLOSSARY
// - `kind_of`: a row's type: folder, raster, guide, texture, filter or vector.
// - `name_of`: a row's name, else its kind's.
// - `row`: one row with a folder's rows, topmost first.
// - `layers`: the active document's rows for the Layers panel.
