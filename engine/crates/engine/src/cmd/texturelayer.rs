//! Texture layers (`uistandards.md` §7.4): a texture from the Texture library inserted as an
//! editable layer, not baked into the artwork. The layer is a locked rectangle over the canvas
//! (the artboards, or [`INFINITE_EXTENT`] around the origin on an infinite canvas) filled with the
//! texture's image as a live pattern; it takes part in ordering, folders, visibility, opacity,
//! blend, saves and exports like any layer, and its scale changes the tiling. Export bounds leave
//! it out (`render::encode::art_bounds`). The image is stored once per content (`texture-` key).

use std::sync::Arc;

use serde_json::{Value, json};
use vectorcraft_doc::color::Paint;
use vectorcraft_doc::{Appearance, AppearanceItem, FillLayer, ImageBlob, ImageObject, Node, NodeId, NodeKind, PatternDef, TextureSpec};
use vectorcraft_geom::{Affine, Rect};

use super::referenceassets::sniff_mime;
use super::{CommandSpec, Result, Session, bad, cmd, has_doc};

/// How far a texture reaches from the origin on an infinite canvas, in points (each way).
pub const INFINITE_EXTENT: f64 = 100_000.0;

/// The prefix of a texture image's key.
pub const TEXTURE_KEY_PREFIX: &str = "texture-";

/// The largest texture image, in bytes.
pub const MAX_TEXTURE_BYTES: u64 = 64 * 1024 * 1024;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "texture.new",
            "New Texture Layer",
            [],
            None,
            "{path (a PNG, JPEG, WebP, GIF or BMP image), name (the texture's name), scale?: above 0 (1: one image pixel a point)} insert a texture layer at the top of the current layer: a locked rectangle over the artboards (an infinite canvas: 100 000 pt each way) filled with the image as a live pattern; one undo step → {id}",
            has_doc,
            new_texture
        ),
        cmd!(
            "texture.set",
            "Texture Options",
            [],
            None,
            "{id, scale: above 0} change a texture layer's scale, one undo step (none when unchanged) → {id, scale}",
            has_doc,
            set_texture
        ),
    ]
}

/// Image width and height from PNG, JPEG, GIF, BMP or WebP bytes.
fn image_size(bytes: &[u8]) -> Option<(u32, u32)> {
    image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format().ok()?.into_dimensions().ok()
}

fn new_texture(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "texture.new";
    let path = p.get("path").and_then(Value::as_str).filter(|p| !p.is_empty()).ok_or_else(|| bad(C, "`path` is required"))?;
    let name = p.get("name").and_then(Value::as_str).map(str::trim).filter(|n| !n.is_empty()).ok_or_else(|| bad(C, "`name` is required"))?.to_string();
    let scale = match p.get("scale") {
        None | Some(Value::Null) => 1.0,
        Some(v) => v.as_f64().filter(|k| k.is_finite() && *k > 0.0).ok_or_else(|| bad(C, "`scale` must be above 0"))?,
    };
    if std::fs::metadata(path).map_err(|e| bad(C, format!("{path}: {e}")))?.len() > MAX_TEXTURE_BYTES {
        return Err(bad(C, "the texture image is too large"));
    }
    let bytes = std::fs::read(path).map_err(|e| bad(C, format!("{path}: {e}")))?;
    let mime = sniff_mime(&bytes).ok_or_else(|| bad(C, "the file isn't a PNG, JPEG, WebP, GIF or BMP image"))?.to_string();
    let (w, h) = image_size(&bytes).ok_or_else(|| bad(C, "the image can't be read"))?;
    let key = format!("{TEXTURE_KEY_PREFIX}{}", &etcfile::sha256_hex(&bytes)[..16]);
    let st = s.doc()?;
    let parent = st.target_parent()?;
    let area = st.doc.artboards.iter().map(|a| a.rect).reduce(|a, b| a.union(b)).unwrap_or(Rect::new(-INFINITE_EXTENT, -INFINITE_EXTENT, INFINITE_EXTENT, INFINITE_EXTENT));
    let id = s.edit("New Texture Layer", move |d, sel| {
        d.images.entry(key.clone()).or_insert_with(|| ImageBlob { mime, bytes: Arc::new(bytes), proxy: None });
        // The pattern's tile: the image at one pixel a point.
        let mut pattern_name = format!("Texture {name}");
        let mut n = 1;
        while d.patterns.iter().any(|p| p.name == pattern_name) {
            n += 1;
            pattern_name = format!("Texture {name} {n}");
        }
        let tile_id = d.alloc_id();
        let image = ImageObject { key: key.clone(), width: w, height: h, xf: Affine::IDENTITY, link: None, placement: Default::default(), pixels: None };
        let mut def = PatternDef::new(&pattern_name, vec![Arc::new(Node::new(tile_id, NodeKind::Image(Box::new(image))))]);
        def.tile = Rect::new(0.0, 0.0, f64::from(w), f64::from(h));
        d.patterns.push(def);
        let mut appearance = Appearance::default();
        appearance.items.push(AppearanceItem::Fill(FillLayer::new(Paint::Pattern { pattern: pattern_name.clone(), xf: Affine::scale(scale) })));
        let id = d.alloc_id();
        let mut rect = Node::path(id, vectorcraft_geom::shapes::rectangle(area), appearance);
        rect.name = Some(name.clone());
        rect.locked = true;
        rect.extras_mut().texture = Some(Box::new(TextureSpec { asset: key, pattern: pattern_name, name, scale }));
        d.insert(parent, usize::MAX, rect)?;
        sel.clear();
        Ok(id)
    })?;
    Ok(json!({ "id": id.0 }))
}

fn set_texture(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "texture.set";
    let id = p.get("id").and_then(Value::as_u64).map(NodeId).ok_or_else(|| bad(C, "`id` is required"))?;
    let scale = p.get("scale").and_then(Value::as_f64).filter(|k| k.is_finite() && *k > 0.0).ok_or_else(|| bad(C, "`scale` must be above 0"))?;
    let spec = s.doc()?.doc.node(id).and_then(|n| n.extras().texture.as_deref().cloned()).ok_or_else(|| bad(C, format!("{} is not a texture layer", id.0)))?;
    if spec.scale == scale {
        return Ok(json!({ "id": id.0, "scale": scale }));
    }
    s.edit("Texture Options", move |d, _| {
        let n = d.node_mut(id).ok_or(crate::EngineError::NoNode(id))?;
        for item in &mut n.appearance.items {
            if let AppearanceItem::Fill(f) = item
                && let Paint::Pattern { pattern, xf } = &mut f.paint
                && *pattern == spec.pattern
            {
                *xf = Affine::scale(scale);
            }
        }
        n.extras_mut().texture = Some(Box::new(TextureSpec { scale, ..spec }));
        Ok(())
    })?;
    Ok(json!({ "id": id.0, "scale": scale }))
}

// GLOSSARY
// - INFINITE_EXTENT: how far a texture reaches on an infinite canvas. TEXTURE_KEY_PREFIX, MAX_TEXTURE_BYTES: its image's key prefix and largest size.
// - specs: texture.new, texture.set.
// - image_size(bytes): an image's width and height.
// - new_texture, set_texture: the commands.
