//! The colours the tools paint with: the session's paint defaults, read and set without touching
//! the selection. The app's primary colour is the stroke default and its alternate the fill
//! default (the dial's central colour control, `uistandards.md` §6.3).

use serde_json::{Value, json};
use vectorcraft_color::{Color, Paint};

use crate::calls::guarded;
use crate::sessions::Bridge;

/// `paint` as `"#rrggbb"`, or `null` for no paint (a gradient or pattern also reads as `null`).
fn hex_of(paint: &Paint) -> Value {
    match paint {
        Paint::Solid { color, .. } => {
            let [r, g, b, _] = color.to_rgba8(1.0);
            json!(format!("#{r:02x}{g:02x}{b:02x}"))
        }
        _ => Value::Null,
    }
}

/// `"#rrggbb"` as a solid paint; `null` as no paint.
fn paint_of(v: &Value) -> Result<Paint, String> {
    if v.is_null() {
        return Ok(Paint::None);
    }
    let hex = v.as_str().and_then(|t| t.strip_prefix('#')).filter(|h| h.len() == 6 && h.chars().all(|c| c.is_ascii_hexdigit()));
    let hex = hex.ok_or("a colour is \"#rrggbb\" or null")?;
    let byte = |i: usize| f32::from(u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0)) / 255.0;
    Ok(Paint::solid(Color::rgb(byte(0), byte(2), byte(4))))
}

/// The paint defaults → `{"stroke": "#rrggbb"|null, "fill": "#rrggbb"|null}`.
pub fn paint_defaults(handle: i64) -> String {
    guarded(handle, |b: &mut Bridge| Ok(json!({ "stroke": hex_of(&b.session.paint.stroke), "fill": hex_of(&b.session.paint.fill) })))
}

/// Set the paint defaults `values` gives (`{"stroke"?, "fill"?}`, each `"#rrggbb"` or null), the
/// selection keeping its paint → the defaults.
pub fn set_paint_defaults(handle: i64, values: &str) -> String {
    guarded(handle, |b: &mut Bridge| {
        let v: Value = serde_json::from_str(values).map_err(|e| format!("values: {e}"))?;
        if let Some(s) = v.get("stroke") {
            b.session.paint.stroke = paint_of(s)?;
        }
        if let Some(f) = v.get("fill") {
            b.session.paint.fill = paint_of(f)?;
        }
        Ok(json!({ "stroke": hex_of(&b.session.paint.stroke), "fill": hex_of(&b.session.paint.fill) }))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sessions;

    #[test]
    fn defaults_are_read_and_set() {
        let h = sessions::create();
        let set: Value = serde_json::from_str(&set_paint_defaults(h, r##"{"stroke": "#ff8000", "fill": null}"##)).unwrap();
        assert_eq!(set["ok"], json!({"stroke": "#ff8000", "fill": null}));
        assert!(serde_json::from_str::<Value>(&set_paint_defaults(h, r#"{"stroke": "red"}"#)).unwrap().get("error").is_some());
        sessions::destroy(h);
    }
}

// GLOSSARY
// - `hex_of`: a paint as "#rrggbb" or null.
// - `paint_of`: "#rrggbb" or null as a paint.
// - `paint_defaults`: the stroke and fill defaults.
// - `set_paint_defaults`: set them without touching the selection.
