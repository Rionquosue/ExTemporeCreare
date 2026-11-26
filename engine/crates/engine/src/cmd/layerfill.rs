//! Layer Fill (`uistandards.md` §7.1, as in Photoshop): a row's Fill (a folder, or an object the
//! Layers panel shows as a layer) fades its own content while its layer styles (shadows, glows,
//! the `style.*` effects) keep their opacity, unlike Opacity, which fades both. Kept as
//! `NodeExtras::fill_opacity`.

use serde_json::{Value, json};
use vectorcraft_doc::NodeId;

use super::{CommandSpec, Result, Session, bad, cmd, has_doc};

pub fn specs() -> Vec<CommandSpec> {
    vec![cmd!(
        "layer.setFill",
        "Layer Fill",
        [],
        None,
        "{ids: [object, layer or folder id…], fill: 0–100 (percent)} set their Fill: their content's opacity apart from their styles, one undo step (none when nothing changes) → {ids, fill}",
        has_doc,
        set_fill
    )]
}

fn set_fill(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "layer.setFill";
    let fill = p.get("fill").and_then(Value::as_f64).filter(|f| f.is_finite() && (0.0..=100.0).contains(f)).ok_or_else(|| bad(C, "`fill` must be a number from 0 to 100"))?;
    let ids: Vec<NodeId> = p
        .get("ids")
        .and_then(Value::as_array)
        .filter(|a| !a.is_empty())
        .ok_or_else(|| bad(C, "`ids` must list layers"))?
        .iter()
        .map(|v| v.as_u64().map(NodeId).ok_or_else(|| bad(C, "`ids` must be numbers")))
        .collect::<Result<_>>()?;
    let value = (fill < 100.0).then_some((fill / 100.0) as f32);
    let doc = &s.doc()?.doc;
    for id in &ids {
        doc.node(*id).ok_or_else(|| bad(C, format!("no object {}", id.0)))?;
    }
    if ids.iter().all(|id| doc.node(*id).is_some_and(|n| n.extras().fill_opacity == value)) {
        return Ok(json!({ "ids": ids.iter().map(|i| i.0).collect::<Vec<_>>(), "fill": fill }));
    }
    let list = ids.clone();
    s.edit("Layer Fill", move |d, _| {
        for id in &list {
            if let Some(n) = d.node_mut(*id) {
                n.extras_mut().fill_opacity = value;
            }
        }
        Ok(())
    })?;
    Ok(json!({ "ids": ids.iter().map(|i| i.0).collect::<Vec<_>>(), "fill": fill }))
}

// GLOSSARY
// - specs: layer.setFill.
// - set_fill: set layers' Fill (NodeExtras::fill_opacity; None at 100 %).
