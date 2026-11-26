//! New documents and the artwork camera saved with them (`uistandards.md` §5.3, §12).
//!
//! - [`new_document`]: the New Document dialog's settings → `file.new`. An **infinite** canvas is a
//!   document without artboards (`filestandard.md` §7A: no artboard, no bounds); the artboard
//!   `file.new` makes is removed before the document's history starts, so it can't be undone back.
//! - [`view`] / [`set_view`]: the camera the document keeps (`DocState::view`), written into the
//!   `.etc` on save and read back on open; the UI sets it before saving.

use serde_json::{Value, json};
use vectorcraft_doc::SavedView;
use vectorcraft_geom::Point;

use crate::calls::guarded;
use crate::sessions::Bridge;

/// Make a new document from `settings` (`{"name", "infinite": bool, "width", "height" (pt;
/// finite), "units"?, "colorMode"?: "rgb"|"cmyk", "ppi"?, "background"?: "transparent"|"white"|
/// "#rrggbb"}`) → `{"infinite", "artboards"}`.
pub fn new_document(handle: i64, settings: &str) -> String {
    guarded(handle, |b: &mut Bridge| {
        let p: Value = serde_json::from_str(settings).map_err(|e| format!("settings: {e}"))?;
        let infinite = p.get("infinite").and_then(Value::as_bool).unwrap_or(false);
        let mut params = json!({});
        for (from, to) in [("name", "name"), ("width", "width"), ("height", "height"), ("units", "units"), ("colorMode", "colorMode"), ("ppi", "rasterEffectsPpi")] {
            if let Some(v) = p.get(from).filter(|v| !v.is_null()) {
                params[to] = v.clone();
            }
        }
        let background = p.get("background").and_then(Value::as_str).unwrap_or("transparent");
        if background == "white" {
            params["backgroundContents"] = json!("white");
        }
        b.session.execute("file.new", &params).map_err(|e| e.to_string())?;
        if infinite {
            b.session.edit("New Document", |d, _| {
                d.artboards.clear();
                Ok(())
            }).map_err(|e| e.to_string())?;
        } else if background.starts_with('#') {
            b.session.execute("artboard.setProps", &json!({ "index": 0, "color": background })).map_err(|e| e.to_string())?;
        }
        // The new document's history starts here.
        b.session.execute("edit.clearHistory", &json!({})).map_err(|e| e.to_string())?;
        let artboards = b.session.doc().map(|st| st.doc.artboards.len()).unwrap_or(0);
        Ok(json!({ "infinite": artboards == 0, "artboards": artboards }))
    })
}

/// The camera the active document keeps → `{"x", "y" (the view's centre), "zoom"}`, or null when
/// it keeps none.
pub fn view(handle: i64) -> String {
    guarded(handle, |b: &mut Bridge| {
        let st = b.session.doc().map_err(|e| e.to_string())?;
        Ok(st.view.as_ref().map_or(Value::Null, |v| json!({ "x": v.center.x, "y": v.center.y, "zoom": v.zoom })))
    })
}

/// Keep `camera` (`{"x", "y", "zoom"}`) as the active document's camera, saved with it (not an
/// undo step) → true.
pub fn set_view(handle: i64, camera: &str) -> String {
    guarded(handle, |b: &mut Bridge| {
        let v: Value = serde_json::from_str(camera).map_err(|e| format!("camera: {e}"))?;
        let n = |k: &str| v.get(k).and_then(Value::as_f64).filter(|n| n.is_finite());
        let (x, y, zoom) = match (n("x"), n("y"), n("zoom").filter(|z| *z > 0.0)) {
            (Some(x), Some(y), Some(z)) => (x, y, z),
            _ => return Err("camera is {x, y, zoom}, zoom above 0".into()),
        };
        let st = b.session.doc_mut().map_err(|e| e.to_string())?;
        st.view = Some(SavedView { name: "Artwork".into(), center: Point::new(x, y), zoom, rotation: 0.0 });
        Ok(Value::Bool(true))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{calls, files, paint, sessions};

    fn ok(text: &str) -> Value {
        let v: Value = serde_json::from_str(text).unwrap();
        v.get("ok").cloned().unwrap_or_else(|| panic!("{text}"))
    }

    #[test]
    fn an_infinite_canvas_draws_paints_exports_and_round_trips() {
        let h = sessions::create();
        let made = ok(&new_document(h, r#"{"name": "Sketch", "infinite": true}"#));
        assert_eq!(made, json!({"infinite": true, "artboards": 0}));
        assert_eq!(ok(&calls::state(h))["canvas"], "infinite");
        assert!(serde_json::from_str::<Value>(&calls::execute(h, "edit.undo", "{}")).unwrap().get("ok").is_none(), "nothing to undo back to an artboard");
        ok(&calls::execute(h, "shape.rectangle", r#"{"x": -500, "y": -300, "width": 100, "height": 80}"#));
        let px = calls::render(h, -520.0, -320.0, 140, 120, 1.0).expect("renders without artboards");
        assert!(px.chunks(4).any(|p| p[3] != 0));
        ok(&paint::begin(h, r#"{"view": {"x": -600, "y": -400, "width": 400, "height": 300}, "zoom": 1, "size": 10}"#));
        ok(&paint::add(h, "[[-450, -250, 1], [-400, -250, 1]]"));
        ok(&paint::end(h));
        let slices = ok(&calls::execute(h, "export.slice.list", "{}"));
        assert!(slices["contentBounds"].is_object(), "content bounds without artboards");
        ok(&calls::select_tool(h, "fillArea", "{}"));
        for kind in ["down", "up"] {
            ok(&calls::pointer(h, &json!({"kind": kind, "pos": {"x": 900, "y": 900}}).to_string(), "{}"));
        }
        ok(&set_view(h, r#"{"x": -450, "y": -260, "zoom": 2.5}"#));
        // Save through temporary files and open again: still infinite, camera kept.
        let dir = std::env::temp_dir().join(format!("etc-bridge-infinite-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("sketch.etc");
        let saved = ok(&files::save(h, "saveAs", &json!({"path": target.to_string_lossy()}).to_string(), &dir.to_string_lossy()));
        let temp = saved["files"][0]["temp"].as_str().unwrap().to_string();
        std::fs::rename(&temp, &target).unwrap();
        ok(&files::save_finish(h, true, ""));
        let h2 = sessions::create();
        ok(&files::open(h2, &target.to_string_lossy(), "sketch.etc", &target.to_string_lossy()));
        assert_eq!(ok(&calls::state(h2))["canvas"], "infinite");
        assert_eq!(ok(&view(h2)), json!({"x": -450.0, "y": -260.0, "zoom": 2.5}));
        sessions::destroy(h);
        sessions::destroy(h2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_finite_document_takes_its_size_and_background() {
        let h = sessions::create();
        ok(&new_document(h, r##"{"name": "Card", "width": 500, "height": 300, "background": "#204060"}"##));
        let st = ok(&calls::state(h));
        assert_eq!((st["canvas"].as_str(), st["artboards"][0]["width"].as_f64()), (Some("finite"), Some(500.0)));
        sessions::destroy(h);
    }
}

// GLOSSARY
// - `new_document`: the New Document dialog's settings → a finite or infinite document.
// - `view`: the camera the active document keeps.
// - `set_view`: keep a camera with the document, saved with it.
