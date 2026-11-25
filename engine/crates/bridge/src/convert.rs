//! JSON conversions between the UI and the engine's tool types: the view state the UI sends with
//! pointer events, and the requests tools send back.

use serde::Deserialize;
use serde_json::{Value, json};
use vectorcraft_engine::{UiRequest, ViewInfo};
use vectorcraft_geom::Point;
use vectorcraft_tools::Overlay;

/// The view state the UI sends (camelCase); a missing field keeps the engine's default.
#[derive(Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct ViewIn {
    zoom: Option<f64>,
    outline: Option<bool>,
    smart_guides: Option<bool>,
    guides: Option<bool>,
    snap_to_grid: Option<bool>,
    pixel_preview: Option<bool>,
    show_bbox: Option<bool>,
    snap_to_pixel: Option<bool>,
    snap_to_point: Option<bool>,
    snap_to_glyph: Option<bool>,
    artboard: Option<usize>,
}

/// The engine's view state from the UI's JSON (empty or unreadable JSON: the defaults).
pub fn view_info(json: &str) -> ViewInfo {
    let v: ViewIn = serde_json::from_str(json).unwrap_or_default();
    let d = ViewInfo::default();
    ViewInfo {
        zoom: v.zoom.filter(|z| z.is_finite() && *z > 0.0).unwrap_or(d.zoom),
        outline: v.outline.unwrap_or(d.outline),
        smart_guides: v.smart_guides.unwrap_or(d.smart_guides),
        guides: v.guides.unwrap_or(d.guides),
        snap_to_grid: v.snap_to_grid.unwrap_or(d.snap_to_grid),
        pixel_preview: v.pixel_preview.unwrap_or(d.pixel_preview),
        show_bbox: v.show_bbox.unwrap_or(d.show_bbox),
        snap_to_pixel: v.snap_to_pixel.unwrap_or(d.snap_to_pixel),
        snap_to_point: v.snap_to_point.unwrap_or(d.snap_to_point),
        snap_to_glyph: v.snap_to_glyph.unwrap_or(d.snap_to_glyph),
        artboard: v.artboard.unwrap_or(d.artboard),
        ..d
    }
}

/// A tool's request to the UI as JSON: `{"type": …, …}`.
pub fn ui_request(r: &UiRequest) -> Value {
    match r {
        UiRequest::Dialog(id, params) => json!({"type": "dialog", "id": id, "params": params}),
        UiRequest::Panel(id) => json!({"type": "panel", "id": id}),
        UiRequest::SwitchTool(id) => json!({"type": "switchTool", "id": id}),
        UiRequest::Status(text) => json!({"type": "status", "text": text}),
        UiRequest::Place(params) => json!({"type": "place", "params": params}),
        UiRequest::HeldPress { doc, pos, tool } => json!({"type": "heldPress", "doc": doc, "x": pos.x, "y": pos.y, "tool": tool}),
    }
}

/// A list of tool requests as a JSON array.
pub fn ui_requests(rs: &[UiRequest]) -> Value {
    Value::Array(rs.iter().map(ui_request).collect())
}

/// A point as `[x, y]`.
fn pt(p: Point) -> Value {
    json!([p.x, p.y])
}

/// One overlay of the active tool as JSON, in document coordinates: `{"type", …}`. Paths are SVG
/// path data; colours are `[r, g, b]` or `[r, g, b, a]`. Hover text draws nothing and is left out.
pub fn overlay_json(o: &Overlay) -> Option<Value> {
    Some(match o {
        Overlay::Marquee(r) => json!({"type": "marquee", "x": r.x0, "y": r.y0, "width": r.width(), "height": r.height()}),
        Overlay::SelectionMarquee { rect: r, enclosed, .. } => {
            json!({"type": "marquee", "x": r.x0, "y": r.y0, "width": r.width(), "height": r.height(), "enclosed": enclosed})
        }
        Overlay::Path { path, color, width, dashed } => json!({"type": "path", "d": path.to_svg(), "color": color, "width": width, "dashed": dashed}),
        Overlay::SharedPath { path, color, width, dash } => {
            json!({"type": "path", "d": path.to_svg(), "color": color, "width": width, "dashed": dash.is_some()})
        }
        Overlay::Line { a, b, color, dashed } => json!({"type": "line", "a": pt(*a), "b": pt(*b), "color": color, "dashed": dashed}),
        Overlay::Anchor { p, color, filled, size } => json!({"type": "anchor", "p": pt(*p), "color": color, "filled": filled, "size": size}),
        Overlay::CurvePoint { p, color, filled, size } => json!({"type": "curvePoint", "p": pt(*p), "color": color, "filled": filled, "size": size}),
        Overlay::Handle { p, color } => json!({"type": "handle", "p": pt(*p), "color": color}),
        Overlay::Label { p, text, color } => json!({"type": "label", "p": pt(*p), "text": text, "color": color}),
        Overlay::Measure { p, text } => json!({"type": "measure", "p": pt(*p), "text": text}),
        Overlay::Distance { a, b, text, color } => json!({"type": "distance", "a": pt(*a), "b": pt(*b), "text": text, "color": color}),
        Overlay::Highlight { quad, color } => json!({"type": "highlight", "quad": quad.iter().map(|p| pt(*p)).collect::<Vec<_>>(), "color": color}),
        Overlay::Swatch { p, color, selected } => json!({"type": "swatch", "p": pt(*p), "color": color, "selected": selected}),
        Overlay::GridLine { a, b, color } => json!({"type": "gridLine", "a": pt(*a), "b": pt(*b), "color": color}),
        Overlay::HoverText { .. } => return None,
    })
}

/// The active tool's overlays as a JSON array.
pub fn overlays_json(os: &[Overlay]) -> Value {
    Value::Array(os.iter().filter_map(overlay_json).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_fields_and_defaults() {
        let v = view_info(r#"{"zoom": 2.5, "outline": true, "artboard": 1}"#);
        assert_eq!((v.zoom, v.outline, v.artboard), (2.5, true, 1));
        assert_eq!(v.smart_guides, ViewInfo::default().smart_guides);
        assert_eq!(view_info("not json").zoom, ViewInfo::default().zoom);
        assert_eq!(view_info(r#"{"zoom": -1}"#).zoom, ViewInfo::default().zoom);
    }

    #[test]
    fn requests_are_tagged() {
        let v = ui_requests(&[UiRequest::SwitchTool("pen".into()), UiRequest::Status("hi".into())]);
        assert_eq!(v, json!([{"type": "switchTool", "id": "pen"}, {"type": "status", "text": "hi"}]));
    }
}

// GLOSSARY
// - `ViewIn`: the view state JSON the UI sends.
// - `view_info`: the engine's `ViewInfo` from that JSON, defaults for missing fields.
// - `ui_request`: one tool request to the UI as tagged JSON.
// - `ui_requests`: a list of tool requests as a JSON array.
// - `pt`: a point as `[x, y]`.
// - `overlay_json`: one tool overlay as tagged JSON (hover text left out).
// - `overlays_json`: the tool's overlays as a JSON array.
