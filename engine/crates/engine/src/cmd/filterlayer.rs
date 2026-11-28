//! Filter layers (`uistandards.md` §7.3): an empty folder whose pixel filter (one of the Photoshop-
//! style filters, `effects::PIXEL_EFFECTS`) applies to everything visually below it on the page,
//! across folders, not only to its own folder. Its visibility switches it, its opacity mixes the
//! filtered picture with the unfiltered one. The renderer draws the page in passes at the filter
//! layers ([`vectorcraft_render`]'s `draw_frame`).


use serde_json::{Value, json};
use vectorcraft_doc::{FilterSpec, Node, NodeId};

use super::{CommandSpec, Result, Session, bad, cmd, has_doc};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            query "filter.list",
            "Filter Layer Filters",
            [],
            None,
            "{} → {filters: [{effect, label, params (documentation), defaults}…]} the filters a filter layer can apply",
            |_| Ok(()),
            list
        ),
        cmd!(
            "filter.new",
            "New Filter Layer",
            [],
            None,
            "{effect (one of filter.list), params? (merged over the filter's defaults), name? (default: the filter's label)} insert a filter layer directly above the current layer, at its level: an empty layer that filters everything below it on the page; one undo step → {id}",
            has_doc,
            new_filter
        ),
        cmd!(
            "filter.set",
            "Filter Options",
            [],
            None,
            "{id, params} change a filter layer's parameters (merged over its own), one undo step → {id}",
            has_doc,
            set_filter
        ),
    ]
}

fn label_of(effect: &str) -> Option<String> {
    vectorcraft_render::effects::effect_info(effect).map(|i| i.label.trim_end_matches('…').to_string())
}

fn list(_: &mut Session, _: &Value) -> Result<Value> {
    let filters: Vec<Value> = vectorcraft_render::effects::PIXEL_EFFECTS
        .iter()
        .filter_map(|id| vectorcraft_render::effects::effect_info(id))
        .map(|i| json!({ "effect": i.id, "label": i.label.trim_end_matches('…'), "params": i.params, "defaults": i.defaults }))
        .collect();
    Ok(json!({ "filters": filters }))
}

/// `over` merged onto `base` (objects key by key).
fn merged(base: &Value, over: Option<&Value>) -> Value {
    let mut out = base.clone();
    if let (Some(o), Some(Value::Object(add))) = (out.as_object_mut(), over) {
        for (k, v) in add {
            o.insert(k.clone(), v.clone());
        }
    }
    out
}

fn new_filter(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "filter.new";
    let effect = p.get("effect").and_then(Value::as_str).ok_or_else(|| bad(C, "`effect` is required"))?.to_string();
    if !vectorcraft_render::effects::PIXEL_EFFECTS.contains(&effect.as_str()) {
        return Err(bad(C, format!("`{effect}` is not a filter (see filter.list)")));
    }
    let defaults = vectorcraft_render::effects::effect_info(&effect).map(|i| i.defaults).unwrap_or_else(|| json!({}));
    let params = merged(&defaults, p.get("params"));
    let name = p.get("name").and_then(Value::as_str).map(str::to_string).or_else(|| label_of(&effect)).unwrap_or_else(|| effect.clone());
    let (parent, index) = super::texturelayer::beside_current_layer(s.doc()?);
    let id = s.edit("New Filter Layer", move |d, sel| {
        let id = d.alloc_id();
        let mut n = Node::layer(id, &name, vectorcraft_doc::LayerColor::Preset(0));
        n.extras_mut().filter = Some(Box::new(FilterSpec { effect, params }));
        d.insert(parent, index, n)?;
        sel.clear();
        Ok(id)
    })?;
    Ok(json!({ "id": id.0 }))
}

fn set_filter(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "filter.set";
    let id = p.get("id").and_then(Value::as_u64).map(NodeId).ok_or_else(|| bad(C, "`id` is required"))?;
    let spec = s.doc()?.doc.node(id).and_then(|n| n.extras().filter.as_deref().cloned()).ok_or_else(|| bad(C, format!("{} is not a filter layer", id.0)))?;
    let params = merged(&spec.params, p.get("params"));
    s.edit("Filter Options", move |d, _| {
        let n = d.node_mut(id).ok_or(crate::EngineError::NoNode(id))?;
        n.extras_mut().filter = Some(Box::new(FilterSpec { params, ..spec }));
        Ok(())
    })?;
    Ok(json!({ "id": id.0 }))
}

// GLOSSARY
// - specs: filter.list, filter.new, filter.set.
// - label_of(effect): a filter's label. merged(base, over): parameters merged.
// - list, new_filter, set_filter: the commands.
