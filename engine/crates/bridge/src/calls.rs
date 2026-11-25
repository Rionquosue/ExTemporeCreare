//! What the UI can ask of the engine, as plain Rust: run a command, feed pointer input to the
//! active tool, select a tool, render part of the active document, and set the presets folder.
//! Every call answers with JSON text: `{"ok": <result>}` or `{"error": {"kind", "message"}}`, and a
//! panic inside the engine is caught and answered as an error, never unwound into the JVM.

use serde_json::{Value, json};
use vectorcraft_engine::EngineError;
use vectorcraft_geom::Rect;
use vectorcraft_render::{RenderOptions, Renderer};
use vectorcraft_tools::PointerEvent;

use crate::convert::{ui_requests, view_info};
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
/// tool with the view state `view` (JSON) → the tool's requests to the UI.
pub fn pointer(handle: i64, event: &str, view: &str) -> String {
    guarded(handle, |b| {
        let ev: PointerEvent = serde_json::from_str(event).map_err(|e| format!("event: {e}"))?;
        let rs = b.session.pointer(&ev, view_info(view)).map_err(engine_error)?;
        Ok(ui_requests(&rs))
    })
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

    #[test]
    fn a_session_runs_commands_takes_pointer_input_and_renders() {
        let h = sessions::create();
        ok(&execute(h, "file.new", r#"{"width": 100, "height": 50}"#));
        ok(&execute(h, "shape.rectangle", r#"{"x": 10, "y": 10, "width": 20, "height": 20}"#));
        let bad: Value = serde_json::from_str(&execute(h, "no.such.command", "")).unwrap();
        assert_eq!(bad["error"]["kind"], "engine");
        ok(&select_tool(h, "rectangle", "{}"));
        for kind in ["down", "drag", "up"] {
            let ev = json!({"kind": kind, "pos": {"x": 50.0, "y": 30.0}}).to_string();
            if let Some(e) = serde_json::from_str::<Value>(&pointer(h, &ev, r#"{"zoom": 1}"#)).unwrap().get("error") {
                panic!("{kind}: {e}");
            }
        }
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
// - `pointer`: feed a pointer event to the active tool.
// - `select_tool`: make a tool active.
// - `render`: premultiplied RGBA pixels of a region of the active document (`RENDERER`: one renderer per thread).
// - `set_presets_root`: set the built-in presets folder.
