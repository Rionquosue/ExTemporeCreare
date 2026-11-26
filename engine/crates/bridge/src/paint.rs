//! The Brush tool's live strokes: raster painting into a splotch while the pen moves
//! (`vectorcraft_engine::cmd::rasterpaint`). The stroke lives in the session's [`Bridge`] from the
//! pen going down ([`begin`]) to its release ([`end`]); meanwhile [`add`] paints the points the pen
//! reached and [`pixels`] hands the app the stroke's layer to draw over the canvas. The splotch's
//! decoded pixels are kept after a stroke, so the next stroke on it starts at once.

use serde_json::{Value, json};
use vectorcraft_engine::cmd::rasterpaint::{self, Stroke};

use crate::calls::guarded;
use crate::sessions::{self, Bridge};

/// The splotch area and size of a stroke as JSON.
fn frame_json(stroke: &Stroke) -> Value {
    let (area, w, h, target) = stroke.frame();
    json!({ "x": area.x0, "y": area.y0, "width": area.width(), "height": area.height(), "pixelsWide": w, "pixelsHigh": h, "splotch": target.map(|id| id.0) })
}

/// The pen went down: get the stroke and its splotch ready (`params` as `paint.stroke` takes them,
/// without `points`; the points come through [`add`]) → `{"x", "y", "width", "height" (the
/// splotch's document area), "pixelsWide", "pixelsHigh", "splotch": id | null (new)}`. A stroke
/// left from before is dropped.
pub fn begin(handle: i64, params: &str) -> String {
    guarded(handle, |b: &mut Bridge| {
        let p: Value = serde_json::from_str(params).map_err(|e| format!("params: {e}"))?;
        b.stroke = None;
        let stroke = rasterpaint::begin(&b.session, &p, b.splotch_cache.as_ref()).map_err(|e| e.to_string())?;
        let frame = frame_json(&stroke);
        b.stroke = Some(stroke);
        Ok(frame)
    })
}

/// Paint the stroke on to `points` (`[[x, y, pressure?]…]`, document points) → `{"painted": [x,
/// y, width, height] | null}`, the splotch pixels this call painted.
pub fn add(handle: i64, points: &str) -> String {
    guarded(handle, |b: &mut Bridge| {
        let v: Value = serde_json::from_str(points).map_err(|e| format!("points: {e}"))?;
        let pts = rasterpaint::points_of(&v, "paint").map_err(|e| e.to_string())?;
        let stroke = b.stroke.as_mut().ok_or("no stroke is being painted")?;
        Ok(json!({ "painted": stroke.add(&pts) }))
    })
}

/// The stroke's layer over splotch pixels `x`, `y`, `width` × `height`: premultiplied RGBA, rows
/// top to bottom (an Android `ARGB_8888` bitmap's layout). `None`: no session or no stroke.
pub fn pixels(handle: i64, x: u32, y: u32, width: u32, height: u32) -> Option<Vec<u8>> {
    let run = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        sessions::with(handle, |b| b.stroke.as_ref().map(|s| s.layer_pixels(x, y, width, height))).flatten()
    }));
    run.ok().flatten()
}

/// The pen was released: lay the stroke over its splotch and keep it in the document, one undo
/// step → `{"id", "created", "width", "height"}`.
pub fn end(handle: i64) -> String {
    guarded(handle, |b: &mut Bridge| {
        let stroke = b.stroke.take().ok_or("no stroke is being painted")?;
        let (answer, decoded) = stroke.finish(&mut b.session).map_err(|e| e.to_string())?;
        b.splotch_cache = Some(decoded);
        Ok(answer)
    })
}

/// Drop the stroke being painted, changing nothing → `true`.
pub fn cancel(handle: i64) -> String {
    guarded(handle, |b: &mut Bridge| {
        b.stroke = None;
        Ok(Value::Bool(true))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(text: &str) -> Value {
        let v: Value = serde_json::from_str(text).unwrap();
        v.get("ok").cloned().unwrap_or_else(|| panic!("{text}"))
    }

    #[test]
    fn a_live_stroke_paints_shows_and_keeps() {
        let h = sessions::create();
        ok(&crate::calls::execute(h, "file.new", r#"{"width": 400, "height": 400}"#));
        let params = json!({"view": {"x": 0, "y": 0, "width": 200, "height": 100}, "zoom": 2, "size": 20, "color": "#00ff00"});
        let frame = ok(&begin(h, &params.to_string()));
        assert_eq!((frame["pixelsWide"].as_u64(), frame["splotch"].is_null()), (Some(480), true));
        let painted = ok(&add(h, "[[50, 50, 1], [80, 50, 1]]"))["painted"].clone();
        let [x, y, w, hh] = [0, 1, 2, 3].map(|i| painted[i].as_u64().unwrap() as u32);
        let layer = pixels(h, x, y, w, hh).unwrap();
        assert!(layer.chunks(4).any(|p| p == [0, 255, 0, 255]), "the stroke's layer shows live");
        let kept = ok(&end(h));
        assert_eq!(kept["created"], true);
        assert!(pixels(h, 0, 0, 1, 1).is_none(), "no stroke after the release");
        // The next stroke over the same view paints the same splotch.
        let again = ok(&begin(h, &params.to_string()));
        assert_eq!(again["splotch"], kept["id"]);
        ok(&cancel(h));
        sessions::destroy(h);
    }
}

// GLOSSARY
// - `frame_json`: a stroke's splotch area and size as JSON.
// - `begin`: the pen went down: get the stroke and its splotch ready.
// - `add`: paint the stroke on to more points → the pixels painted.
// - `pixels`: the stroke's live layer over a region, premultiplied RGBA.
// - `end`: the pen was released: keep the stroke, one undo step.
// - `cancel`: drop the stroke being painted.
