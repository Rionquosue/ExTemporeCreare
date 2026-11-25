//! `stack.xml`: the ORA-style projection of the layer stack (`filestandard.md` §2, §4). It is
//! derived from the document at every save and never read back: `project.json` is authoritative.
//!
//! Each top-level layer becomes a `<stack>` with its name, visibility, opacity and composite
//! operation, topmost first. Layer pixels aren't rendered per layer; when the file carries
//! `mergedimage.png`, a last `<layer>` points at it so ORA readers show the picture.

use vectorcraft_doc::Document;

/// Escape `s` for an XML attribute value.
fn attr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c if (c as u32) < 0x20 && !matches!(c, '\t' | '\n' | '\r') => {}
            c => out.push(c),
        }
    }
    out
}

/// The ORA `composite-op` of a blend mode's serialized name (`"multiply"`, `"colorDodge"`…).
fn composite_op(blend: &str) -> String {
    if blend.is_empty() || blend == "normal" {
        return "svg:src-over".into();
    }
    let mut kebab = String::with_capacity(blend.len() + 4);
    for c in blend.chars() {
        if c.is_ascii_uppercase() {
            kebab.push('-');
            kebab.push(c.to_ascii_lowercase());
        } else {
            kebab.push(c);
        }
    }
    format!("svg:{kebab}")
}

/// The `stack.xml` of `doc`, `size` pixels wide and high, at `ppi`; `merged` says whether the
/// file carries `mergedimage.png`.
pub fn stack_xml(doc: &Document, size: (u32, u32), ppi: f64, merged: bool) -> String {
    let mut x = String::new();
    x.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    let ppi = if ppi.is_finite() && ppi > 0.0 { ppi.round() as u32 } else { 72 };
    x.push_str(&format!("<image version=\"0.0.6\" w=\"{}\" h=\"{}\" xres=\"{ppi}\" yres=\"{ppi}\">\n<stack>\n", size.0.max(1), size.1.max(1)));
    for layer in doc.layers.iter().rev() {
        let name = layer.name.as_deref().unwrap_or("Layer");
        let blend = serde_json::to_value(layer.blend).ok().and_then(|v| v.as_str().map(str::to_owned)).unwrap_or_default();
        x.push_str(&format!(
            "  <stack name=\"{}\" visibility=\"{}\" opacity=\"{:.3}\" composite-op=\"{}\"/>\n",
            attr(name),
            if layer.visible { "visible" } else { "hidden" },
            layer.opacity.clamp(0.0, 1.0),
            attr(&composite_op(&blend)),
        ));
    }
    if merged {
        x.push_str("  <layer name=\"Flattened\" src=\"mergedimage.png\" x=\"0\" y=\"0\" visibility=\"visible\" opacity=\"1.000\"/>\n");
    }
    x.push_str("</stack>\n</image>\n");
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_and_names_composite_ops() {
        assert_eq!(attr("a<b>&\"c'"), "a&lt;b&gt;&amp;&quot;c&apos;");
        assert_eq!(composite_op("normal"), "svg:src-over");
        assert_eq!(composite_op("colorDodge"), "svg:color-dodge");
    }

    #[test]
    fn lists_layers_top_first() {
        let doc = Document::new(612.0, 792.0);
        let xml = stack_xml(&doc, (612, 792), 72.0, true);
        assert!(xml.starts_with("<?xml") && xml.contains("<image version=\"0.0.6\" w=\"612\" h=\"792\""));
        assert!(xml.contains("src=\"mergedimage.png\""));
        assert_eq!(xml.matches("<stack name=").count(), doc.layers.len());
    }
}

// GLOSSARY
// - `attr`: XML attribute escaping.
// - `composite_op`: a blend mode's serialized name as an ORA `composite-op` (`svg:...`).
// - `stack_xml`: the `stack.xml` projection of a document's top-level layers, topmost first.
