//! Layer masks (`uistandards.md` §7.1, as in Photoshop): a greyscale mask on a layer or folder,
//! shown as its sublayer, where white shows the layer, black hides it and greys partly show it.
//! It is the node's opacity mask (`OpacityMask`): its art's luminance over a white background
//! (Reveal All, `clip` off) or a black one (Hide All, `clip` on), optionally inverted. Painting
//! into it goes through `transparency.editOpacityMask` / `stopEditingOpacityMask`.
//!
//! The quick operations: add (Reveal All, Hide All, or from the selection), replace with the
//! selection (tapping the mask while something is selected), invert, enable or disable, delete.

use std::sync::Arc;

use serde_json::{Value, json};
use vectorcraft_doc::node::whiten;
use vectorcraft_doc::{Document, Node, NodeId, OpacityMask, Selection};

use super::maskedit::mask_group;
use super::{CommandSpec, Result, Session, bad, cmd, has_doc};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "layer.mask.add",
            "Add Layer Mask",
            [],
            None,
            "{id (layer or folder), mode?: reveal|hide|selection (default reveal)} give the layer a mask: Reveal All (white: all shows), Hide All (black: nothing shows) or the selection (white where the selected objects are, black elsewhere; refused with nothing selected); refused when it has one; one undo step → {id, mode}",
            has_doc,
            add
        ),
        cmd!(
            "layer.mask.fromSelection",
            "Mask from Selection",
            [],
            None,
            "{id} make the layer's mask the selection: white where the selected objects are (their shapes, all paints white), black elsewhere, not inverted, enabled; adds a mask when it has none; refused with nothing selected; one undo step → {id}",
            has_doc,
            from_selection
        ),
        cmd!("layer.mask.invert", "Invert Layer Mask", [], None, "{id} swap the mask's white and black, one undo step → {id, inverted}", has_doc, invert),
        cmd!(
            "layer.mask.setEnabled",
            "Enable Layer Mask",
            [],
            None,
            "{id, enabled: bool} apply the mask or keep it without applying it, one undo step (none when unchanged) → {id, enabled}",
            has_doc,
            set_enabled
        ),
        cmd!("layer.mask.delete", "Delete Layer Mask", [], None, "{id} remove the layer's mask, one undo step → {id}", has_doc, delete),
        cmd!(
            query "layer.mask.info",
            "Layer Mask Info",
            [],
            None,
            "{id} → {id, mask: null | {enabled, inverted, background: white|black, objects (the mask art's objects)}}",
            has_doc,
            info
        ),
    ]
}

/// The layer or folder `id` of `p`, checked.
fn layer_id(s: &Session, p: &Value, c: &str) -> Result<NodeId> {
    let id = p.get("id").and_then(Value::as_u64).map(NodeId).ok_or_else(|| bad(c, "`id` is required"))?;
    let n = s.doc()?.doc.node(id).ok_or_else(|| bad(c, format!("no object {}", id.0)))?;
    if !n.is_container() {
        return Err(bad(c, format!("{} is not a layer or folder", id.0)));
    }
    Ok(id)
}

/// The selected objects' shapes, all paints white, with fresh ids, grouped as mask art (`None`:
/// nothing usable selected). The layer being masked is left out.
fn selection_art(d: &mut Document, sel: &Selection, masked: NodeId) -> Option<Node> {
    let picked: Vec<NodeId> = sel.objects.iter().copied().filter(|id| *id != masked).collect();
    let mut children = vec![];
    for id in picked {
        let Some(n) = d.node(id).cloned() else { continue };
        let mut copy = d.reid(&n);
        copy.extras_mut().mask = None;
        copy.visible = true;
        whiten(&mut copy);
        children.push(Arc::new(copy));
    }
    (!children.is_empty()).then(|| mask_group(children))
}

fn set_mask(s: &mut Session, label: &str, id: NodeId, f: impl FnOnce(&mut Document, &Selection, Option<OpacityMask>) -> Result<Option<OpacityMask>>) -> Result<()> {
    s.edit(label, move |d, sel| {
        let old = d.node(id).and_then(|n| n.extras().mask.as_deref().cloned());
        let new = f(d, sel, old)?;
        let n = d.node_mut(id).ok_or(crate::EngineError::NoNode(id))?;
        n.extras_mut().mask = new.map(Box::new);
        Ok(())
    })
}

fn add(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "layer.mask.add";
    let id = layer_id(s, p, C)?;
    let mode = p.get("mode").and_then(Value::as_str).unwrap_or("reveal").to_string();
    if !["reveal", "hide", "selection"].contains(&mode.as_str()) {
        return Err(bad(C, "`mode` is reveal, hide or selection"));
    }
    if s.doc()?.doc.node(id).is_some_and(|n| n.extras().mask.is_some()) {
        return Err(bad(C, "the layer already has a mask"));
    }
    if mode == "selection" && s.doc()?.selection.objects.iter().all(|o| *o == id) {
        return Err(bad(C, "nothing is selected"));
    }
    let m = mode.clone();
    set_mask(s, "Add Layer Mask", id, move |d, sel, _| {
        Ok(Some(match m.as_str() {
            "selection" => OpacityMask::new(selection_art(d, sel, id).ok_or_else(|| crate::EngineError::Other("nothing is selected".into()))?, true),
            "hide" => OpacityMask::new(mask_group(vec![]), true),
            _ => OpacityMask::new(mask_group(vec![]), false),
        }))
    })?;
    Ok(json!({ "id": id.0, "mode": mode }))
}

fn from_selection(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "layer.mask.fromSelection";
    let id = layer_id(s, p, C)?;
    if s.doc()?.selection.objects.iter().all(|o| *o == id) {
        return Err(bad(C, "nothing is selected"));
    }
    set_mask(s, "Mask from Selection", id, move |d, sel, old| {
        let art = selection_art(d, sel, id).ok_or_else(|| crate::EngineError::Other("nothing is selected".into()))?;
        let mut m = OpacityMask::new(art, true);
        m.linked = old.as_ref().is_none_or(|o| o.linked);
        Ok(Some(m))
    })?;
    Ok(json!({ "id": id.0 }))
}

/// The mask of layer `id`, refused when it has none.
fn mask_of(s: &Session, id: NodeId, c: &str) -> Result<OpacityMask> {
    s.doc()?.doc.node(id).and_then(|n| n.extras().mask.as_deref().cloned()).ok_or_else(|| bad(c, "the layer has no mask"))
}

fn invert(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "layer.mask.invert";
    let id = layer_id(s, p, C)?;
    let mut m = mask_of(s, id, C)?;
    m.invert = !m.invert;
    let inverted = m.invert;
    set_mask(s, "Invert Layer Mask", id, move |_, _, _| Ok(Some(m)))?;
    Ok(json!({ "id": id.0, "inverted": inverted }))
}

fn set_enabled(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "layer.mask.setEnabled";
    let id = layer_id(s, p, C)?;
    let enabled = p.get("enabled").and_then(Value::as_bool).ok_or_else(|| bad(C, "`enabled` must be true or false"))?;
    let mut m = mask_of(s, id, C)?;
    if m.disabled != enabled {
        return Ok(json!({ "id": id.0, "enabled": enabled }));
    }
    m.disabled = !enabled;
    set_mask(s, if enabled { "Enable Layer Mask" } else { "Disable Layer Mask" }, id, move |_, _, _| Ok(Some(m)))?;
    Ok(json!({ "id": id.0, "enabled": enabled }))
}

fn delete(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "layer.mask.delete";
    let id = layer_id(s, p, C)?;
    mask_of(s, id, C)?;
    if s.doc()?.doc.mask_edit.is_some_and(|m| m.object == id) {
        s.execute("transparency.stopEditingOpacityMask", &json!({}))?;
    }
    set_mask(s, "Delete Layer Mask", id, |_, _, _| Ok(None))?;
    Ok(json!({ "id": id.0 }))
}

fn info(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "layer.mask.info";
    let id = layer_id(s, p, C)?;
    let mask = s.doc()?.doc.node(id).and_then(|n| n.extras().mask.as_deref().cloned());
    Ok(json!({
        "id": id.0,
        "mask": mask.map(|m| json!({
            "enabled": !m.disabled,
            "inverted": m.invert,
            "background": if m.clip { "black" } else { "white" },
            "objects": m.art.children().map_or(0, |c| c.len()),
        })),
    }))
}

// GLOSSARY
// - specs: layer.mask.add, layer.mask.fromSelection, layer.mask.invert, layer.mask.setEnabled, layer.mask.delete, layer.mask.info.
// - layer_id(s, p, c): the layer or folder a command names.
// - selection_art(d, sel, masked): the selected objects' shapes, white, as mask art.
// - set_mask(s, label, id, f): change a layer's mask as one undo step.
// - mask_of(s, id, c): a layer's mask, refused without one.
// - add, from_selection, invert, set_enabled, delete, info: the commands.
