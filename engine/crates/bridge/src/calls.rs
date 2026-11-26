//! What the UI can ask of the engine, as plain Rust: run a command, feed pointer input to the
//! active tool, select a tool, render part of the active document, and set the presets folder.
//! Every call answers with JSON text: `{"ok": <result>}` or `{"error": {"kind", "message"}}`, and a
//! panic inside the engine is caught and answered as an error, never unwound into the JVM.

use std::collections::HashSet;

use serde_json::{Value, json};
use vectorcraft_engine::EngineError;
use vectorcraft_geom::Rect;
use vectorcraft_render::{RenderOptions, Renderer};
use vectorcraft_tools::{PointerEvent, PointerKind};

use crate::convert::{overlays_json, ui_requests, view_info};
use crate::sessions;

/// The JSON answer for a result.
fn answer(r: Result<Value, String>) -> String {
    match r {
        Ok(v) => json!({ "ok": v }).to_string(),
        Err(message) => json!({ "error": { "kind": "engine", "message": message } }).to_string(),
    }
}

/// The JSON answer for an unknown session handle.
fn no_session() -> String {
    json!({ "error": { "kind": "noSession", "message": "no such engine session" } }).to_string()
}

/// Run `f` on session `handle`, catching panics; answers as JSON.
pub(crate) fn guarded(handle: i64, f: impl FnOnce(&mut sessions::Bridge) -> Result<Value, String>) -> String {
    let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| sessions::with(handle, f)));
    match run {
        Ok(Some(r)) => answer(r),
        Ok(None) => no_session(),
        Err(_) => json!({ "error": { "kind": "panic", "message": "the engine stopped this call" } }).to_string(),
    }
}

fn engine_error(e: EngineError) -> String {
    e.to_string()
}

/// Run engine command `command` with `params` (JSON text; empty means `{}`).
pub fn execute(handle: i64, command: &str, params: &str) -> String {
    guarded(handle, |b| {
        let p: Value = if params.trim().is_empty() { json!({}) } else { serde_json::from_str(params).map_err(|e| format!("params: {e}"))? };
        b.session.execute(command, &p).map_err(engine_error)
    })
}

/// Feed a pointer event (JSON of the engine's `PointerEvent`, document coordinates) to the active
/// tool with the view state `view` (JSON) → `{"requests": [the tool's requests to the UI],
/// "created": [ids], "changed": bool}`. On a release the tool's edit is applied before answering (a
/// tool may compute it on another thread), `created` lists the outermost objects added to the
/// document since the press, which the app's tools finish (stroke weight, feather), and `changed`
/// tells whether the gesture changed the document at all; empty and false on other events.
pub fn pointer(handle: i64, event: &str, view: &str) -> String {
    guarded(handle, |b| {
        let ev: PointerEvent = serde_json::from_str(event).map_err(|e| format!("event: {e}"))?;
        if ev.kind == PointerKind::Down {
            b.gesture_ids = node_ids(b);
            b.gesture_doc = b.session.doc().ok().map(|st| st.doc.clone());
        }
        let mut rs = b.session.pointer(&ev, view_info(view)).map_err(engine_error)?;
        let mut created = Vec::new();
        let mut changed = false;
        if ev.kind == PointerKind::Up {
            rs.extend(b.session.finish_tool_edit().map_err(engine_error)?);
            if let Some(ids) = b.gesture_ids.take() {
                created = created_ids(b, &ids);
            }
            // An edit replaces the document the press kept a reference to.
            if let (Some(before), Ok(st)) = (b.gesture_doc.take(), b.session.doc()) {
                changed = !std::sync::Arc::ptr_eq(&before, &st.doc);
            }
        }
        Ok(json!({ "requests": ui_requests(&rs), "created": created, "changed": changed }))
    })
}

/// The ids of every object of the active document (`None`: no document is open).
fn node_ids(b: &sessions::Bridge) -> Option<HashSet<u64>> {
    let st = b.session.doc().ok()?;
    let mut ids = HashSet::new();
    st.doc.walk(|n| {
        ids.insert(n.id.0);
    });
    Some(ids)
}

/// The outermost objects of the active document that are not in `before`, in paint order: an
/// added group is listed, not the objects inside it.
fn created_ids(b: &sessions::Bridge, before: &HashSet<u64>) -> Vec<u64> {
    let Ok(st) = b.session.doc() else { return vec![] };
    let mut added = Vec::new();
    st.doc.walk(|n| {
        if !before.contains(&n.id.0) {
            added.push(n);
        }
    });
    let mut inner = HashSet::new();
    for n in &added {
        for c in n.children().into_iter().flatten() {
            c.walk(&mut |d| {
                inner.insert(d.id.0);
            });
        }
    }
    added.into_iter().map(|n| n.id.0).filter(|id| !inner.contains(id)).collect()
}

/// The active tool's options → `{"tool": id, "options": {…}}`.
pub fn tool_options(handle: i64) -> String {
    guarded(handle, |b| Ok(json!({ "tool": b.session.tool_id(), "options": b.session.tool_options() })))
}

/// Set the active tool's option `key` to `value` (JSON text); the tool keeps it when another tool
/// is chosen → the tool's options.
pub fn set_tool_option(handle: i64, key: &str, value: &str) -> String {
    guarded(handle, |b| {
        let v: Value = serde_json::from_str(value).map_err(|e| format!("value: {e}"))?;
        b.session.set_tool_option(key, &v);
        Ok(json!({ "tool": b.session.tool_id(), "options": b.session.tool_options() }))
    })
}

/// What the app's tools read before they act → `{"tool", "document": bool, "canvas":
/// "finite"|"infinite", "artboards": [{"index", "x", "y", "width", "height"}], "selection": [ids],
/// "images": [the selected image objects' ids], "editing": bool}`. A canvas with artboards is
/// finite (`filestandard.md` §7A); the artboards are in document points.
pub fn state(handle: i64) -> String {
    guarded(handle, |b| {
        let tool = b.session.tool_id();
        let editing = b.session.tool_editing();
        let Ok(st) = b.session.doc() else {
            return Ok(json!({ "tool": tool, "document": false, "editing": editing }));
        };
        let artboards: Vec<Value> = st
            .doc
            .artboards
            .iter()
            .enumerate()
            .map(|(i, a)| json!({ "index": i, "x": a.rect.x0, "y": a.rect.y0, "width": a.rect.width(), "height": a.rect.height() }))
            .collect();
        let selection: Vec<u64> = st.selection.objects.iter().map(|id| id.0).collect();
        let images: Vec<u64> = st
            .selection
            .objects
            .iter()
            .filter(|id| st.doc.node(**id).is_some_and(|n| matches!(n.kind, vectorcraft_doc::NodeKind::Image(_))))
            .map(|id| id.0)
            .collect();
        Ok(json!({
            "tool": tool,
            "document": true,
            "canvas": if artboards.is_empty() { "infinite" } else { "finite" },
            "artboards": artboards,
            "selection": selection,
            "images": images,
            "editing": editing,
        }))
    })
}

/// What the active tool draws over the canvas, in document coordinates → `[{"type", …}]`
/// (`convert::overlay_json`).
pub fn overlays(handle: i64, view: &str) -> String {
    guarded(handle, |b| Ok(overlays_json(&b.session.overlays(view_info(view)))))
}

/// Make tool `id` the active tool.
pub fn select_tool(handle: i64, id: &str, view: &str) -> String {
    guarded(handle, |b| {
        b.session.select_tool(id, view_info(view)).map_err(engine_error)?;
        Ok(json!({ "tool": b.session.tool_id() }))
    })
}

/// The pixels of the active document's region at (`x`, `y`) document points, `width` × `height`
/// pixels at `scale` pixels per point: premultiplied RGBA, 4 bytes a pixel, rows top to bottom (as
/// an Android `ARGB_8888` bitmap holds them). `None`: no such session, no open document, or a
/// size of zero.
pub fn render(handle: i64, x: f64, y: f64, width: u32, height: u32, scale: f64) -> Option<Vec<u8>> {
    thread_local! {
        /// The renderer of the calling thread (a renderer keeps caches that stay on one thread).
        static RENDERER: std::cell::RefCell<Renderer> = std::cell::RefCell::new(Renderer::new());
    }
    if width == 0 || height == 0 || !(scale.is_finite() && scale > 0.0) || !x.is_finite() || !y.is_finite() {
        return None;
    }
    let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        sessions::with(handle, |b| {
            let doc = b.session.doc().ok()?.doc.clone();
            let region = Rect::new(x, y, x + f64::from(width) / scale, y + f64::from(height) / scale);
            let opts = RenderOptions { artboards: true, ..RenderOptions::default() };
            let out = RENDERER.with_borrow_mut(|r| r.render_region_with(&doc, region, scale, &opts));
            (out.width == width && out.height == height).then_some(out.pixels)
        })
        .flatten()
    }));
    run.ok().flatten()
}

/// Set the folder the built-in preset libraries are read from.
pub fn set_presets_root(path: &str) {
    vectorcraft_engine::presets::set_root(path);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(text: &str) -> Value {
        let v: Value = serde_json::from_str(text).unwrap();
        assert!(v.get("ok").is_some(), "{text}");
        v["ok"].clone()
    }

    /// A tool's gesture, the stroke weight and the feather the app's Brush gives what it drew, and
    /// the undo group around them: one undo step.
    #[test]
    fn a_brush_stroke_and_its_finish_undo_together() {
        let h = sessions::create();
        ok(&execute(h, "file.new", r#"{"width": 200, "height": 200}"#));
        ok(&select_tool(h, "paintbrush", "{}"));
        ok(&execute(h, "edit.undoGroup.begin", r#"{"label": "Brush"}"#));
        let mut created = Vec::new();
        for (kind, x) in [("down", 20.0), ("drag", 40.0), ("drag", 60.0), ("drag", 80.0), ("up", 80.0)] {
            let ev = json!({"kind": kind, "pos": {"x": x, "y": 20.0 + x / 2.0}}).to_string();
            created = ok(&pointer(h, &ev, "{}"))["created"].as_array().unwrap().clone();
        }
        assert_eq!(created.len(), 1, "the stroke");
        ok(&execute(h, "stroke.set", &json!({ "ids": created, "weight": 12 }).to_string()));
        ok(&execute(h, "effect.apply", &json!({ "ids": created, "effect": "stylize.feather", "params": {"radius": 3} }).to_string()));
        ok(&execute(h, "edit.undoGroup.end", "{}"));
        let count = || sessions::with(h, |b| node_ids(b).map(|i| i.len())).flatten();
        assert_eq!(count(), Some(2));
        ok(&execute(h, "edit.undo", "{}"));
        assert_eq!(count(), Some(1), "one undo removes the stroke with its finish");
        sessions::destroy(h);
    }

    /// The Marker's chisel tip on the Paintbrush: a Calligraphic rectangle brush turned by the pen's
    /// bearing. A level stroke with the pen leaning up the page draws the tip across it (tall); leaning
    /// right, along it (thin).
    #[test]
    fn a_chisel_tip_turns_with_the_pen_tilt_on_the_paintbrush() {
        let h = sessions::create();
        ok(&execute(h, "file.new", r#"{"width": 400, "height": 400}"#));
        let params = json!({"angle": 0, "roundness": 20, "size": 1, "shape": "rectangle", "variation": [180, 0, 0], "modes": ["bearing", "fixed", "fixed"]});
        ok(&execute(h, "brush.new", &json!({"type": "calligraphic", "name": "Marker Tip", "params": params}).to_string()));
        ok(&execute(h, "brush.setCurrent", r#"{"name": "Marker Tip"}"#));
        ok(&select_tool(h, "paintbrush", "{}"));
        let stroke = |y: f64, tilt: [f64; 2]| {
            let mut created = Vec::new();
            for (kind, x) in [("down", 50.0), ("drag", 100.0), ("drag", 150.0), ("drag", 200.0), ("up", 200.0)] {
                let ev = json!({"kind": kind, "pos": {"x": x, "y": y}, "pen": {"tilt": tilt}}).to_string();
                created = ok(&pointer(h, &ev, "{}"))["created"].as_array().unwrap().clone();
            }
            ok(&execute(h, "stroke.set", &json!({"ids": created, "weight": 20}).to_string()));
            // How many rows of the column at x = 125 the (black) stroke darkens on the white artboard.
            let px = render(h, 125.0, y - 30.0, 1, 60, 1.0).expect("pixels");
            px.chunks(4).filter(|p| p[3] > 127 && p[0] < 128).count() as f64
        };
        let across = stroke(100.0, [0.0, -40.0]);
        let along = stroke(300.0, [40.0, 0.0]);
        assert!(across > 15.0 && along < 8.0, "across {across}, along {along}");
        sessions::destroy(h);
    }

    /// Pencil grain on a black square leaves the paper showing through in places, at the view's
    /// resolution: some pixels stay dark, some lighten, zoomed out and zoomed in.
    #[test]
    fn pencil_grain_lets_the_paper_show_at_any_zoom() {
        let h = sessions::create();
        ok(&execute(h, "file.new", r#"{"width": 200, "height": 200}"#));
        let id = ok(&execute(h, "shape.rectangle", r#"{"x": 20, "y": 20, "width": 100, "height": 100}"#))["id"].clone();
        ok(&execute(h, "paint.setFill", &json!({"ids": [id], "color": "#000000"}).to_string()));
        ok(&execute(h, "paint.setStroke", &json!({"ids": [id], "none": true}).to_string()));
        ok(&execute(h, "effect.apply", &json!({"ids": [id], "effect": "stylize.pencilGrain", "params": {"scale": 2, "strength": 100}}).to_string()));
        for scale in [1.0, 8.0] {
            let px = render(h, 40.0, 40.0, 64, 64, scale).expect("pixels");
            let dark = px.chunks(4).filter(|p| p[0] < 64).count();
            let light = px.chunks(4).filter(|p| p[0] > 192).count();
            assert!(dark > 200 && light > 200, "scale {scale}: dark {dark}, light {light}");
        }
        sessions::destroy(h);
    }

    /// The Pencil's cone tip on the Paintbrush: a pen leaning across a level stroke draws it wider
    /// than one held upright.
    #[test]
    fn a_cone_tip_widens_as_the_pen_leans() {
        let h = sessions::create();
        ok(&execute(h, "file.new", r#"{"width": 400, "height": 400}"#));
        let params = json!({"size": 1, "cone": 20});
        ok(&execute(h, "brush.new", &json!({"type": "calligraphic", "name": "Pencil Tip", "params": params}).to_string()));
        ok(&execute(h, "brush.setCurrent", r#"{"name": "Pencil Tip"}"#));
        ok(&select_tool(h, "paintbrush", "{}"));
        let stroke = |y: f64, tilt: [f64; 2]| {
            let mut created = Vec::new();
            for (kind, x) in [("down", 50.0), ("drag", 100.0), ("drag", 150.0), ("drag", 200.0), ("up", 200.0)] {
                let ev = json!({"kind": kind, "pos": {"x": x, "y": y}, "pen": {"tilt": tilt}}).to_string();
                created = ok(&pointer(h, &ev, "{}"))["created"].as_array().unwrap().clone();
            }
            ok(&execute(h, "stroke.set", &json!({"ids": created, "weight": 8}).to_string()));
            let px = render(h, 125.0, y - 30.0, 1, 60, 1.0).expect("pixels");
            px.chunks(4).filter(|p| p[3] > 127 && p[0] < 128).count()
        };
        let upright = stroke(100.0, [0.0, 0.0]);
        let leaning = stroke(300.0, [0.0, -60.0]);
        assert!((7..=9).contains(&upright) && leaning > upright + 4, "upright {upright}, leaning {leaning}");
        sessions::destroy(h);
    }

    #[test]
    fn a_session_runs_commands_takes_pointer_input_and_renders() {
        let h = sessions::create();
        ok(&execute(h, "file.new", r#"{"width": 100, "height": 50}"#));
        ok(&execute(h, "shape.rectangle", r#"{"x": 10, "y": 10, "width": 20, "height": 20}"#));
        let bad: Value = serde_json::from_str(&execute(h, "no.such.command", "")).unwrap();
        assert_eq!(bad["error"]["kind"], "engine");
        ok(&select_tool(h, "rectangle", "{}"));
        let mut created = Vec::new();
        for (kind, x, y) in [("down", 60.0, 25.0), ("drag", 75.0, 35.0), ("drag", 90.0, 45.0), ("up", 90.0, 45.0)] {
            let ev = json!({"kind": kind, "pos": {"x": x, "y": y}}).to_string();
            let out = ok(&pointer(h, &ev, r#"{"zoom": 1}"#));
            created.extend(out["created"].as_array().unwrap().iter().cloned());
        }
        assert_eq!(created.len(), 1, "the drawn rectangle is reported once");
        ok(&select_tool(h, "fillArea", "{}"));
        let tap = |x: f64, y: f64| {
            let mut changed = Vec::new();
            for kind in ["down", "up"] {
                let ev = json!({"kind": kind, "pos": {"x": x, "y": y}}).to_string();
                changed.push(ok(&pointer(h, &ev, "{}"))["changed"].clone());
            }
            changed
        };
        assert_eq!(tap(5.0, 45.0), [json!(false), json!(false)], "a fill tap that encloses nothing changes nothing");
        assert_eq!(tap(20.0, 20.0), [json!(false), json!(true)], "a fill tap inside the square fills it");
        let st = ok(&state(h));
        assert_eq!(st["canvas"], "finite");
        assert_eq!(st["artboards"][0]["width"], 100.0);
        assert_eq!(ok(&tool_options(h))["tool"], "fillArea");
        assert!(ok(&overlays(h, "{}")).is_array());
        let px = render(h, 0.0, 0.0, 100, 50, 1.0).expect("pixels");
        assert_eq!(px.len(), 100 * 50 * 4);
        assert!(px.chunks(4).any(|p| p[3] != 0), "something is drawn");
        sessions::destroy(h);
        assert_eq!(serde_json::from_str::<Value>(&execute(h, "file.new", "{}")).unwrap()["error"]["kind"], "noSession");
        assert!(render(h, 0.0, 0.0, 10, 10, 1.0).is_none());
    }
}

// GLOSSARY
// - `answer`: a result as `{"ok": …}` or `{"error": …}` JSON text.
// - `no_session`: the error answer for an unknown handle.
// - `guarded`: run a call on a session with panics caught.
// - `engine_error`: an engine error's message.
// - `execute`: run an engine command with JSON params.
// - `pointer`: feed a pointer event to the active tool; on release, reports the objects the gesture added and whether it changed the document.
// - `node_ids`: the ids of every object of the active document.
// - `created_ids`: the outermost objects added since a set of ids was taken.
// - `tool_options`: the active tool's options.
// - `set_tool_option`: set one option of the active tool.
// - `state`: canvas mode, artboards, selection and selected images, for the app's tools.
// - `overlays`: what the active tool draws over the canvas.
// - `select_tool`: make a tool active.
// - `render`: premultiplied RGBA pixels of a region of the active document (`RENDERER`: one renderer per thread).
// - `set_presets_root`: set the built-in presets folder.
