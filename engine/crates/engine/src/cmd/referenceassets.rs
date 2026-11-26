//! Reference images stored inside the artwork document (`filestandard.md` §7.2, "Embed" on the
//! reference board): the image's bytes are kept as a document image under a `reference-asset-` key
//! and recorded in `Document::extra["referenceAssets"]` (`{key: {"sha256", "originalFilename",
//! "mime"}}`), so `.etc` saves write the bytes and a board can name them by key. No artwork object
//! uses them: they never render or export as artwork.

use std::sync::Arc;

use serde_json::{Map, Value, json};
use vectorcraft_doc::ImageBlob;

use super::{CommandSpec, Result, Session, bad, cmd, has_doc};

/// The `Document::extra` key of the stored reference images.
pub const REFERENCE_ASSETS_KEY: &str = "referenceAssets";

/// The prefix of a stored reference image's key.
pub const REFERENCE_ASSET_PREFIX: &str = "reference-asset-";

/// The largest reference image stored, in bytes.
pub const MAX_REFERENCE_BYTES: u64 = 256 * 1024 * 1024;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "reference.embed",
            "Embed Reference",
            [],
            None,
            "{path (an image file to read), mime? (default: from its first bytes), originalFilename?} store the image's bytes in the document under a reference-asset key (the same bytes keep one key), one undo step (none when already stored) → {assetKey, sha256}",
            has_doc,
            embed
        ),
        cmd!(
            query "reference.asset",
            "Stored Reference",
            [],
            None,
            "{assetKey, path} write a stored reference image's bytes to path → {assetKey, mime, path, sha256, originalFilename}",
            has_doc,
            asset
        ),
        cmd!(query "reference.list", "Stored References", [], None, "{} → {assets: [{assetKey, sha256, originalFilename, mime}…]}", has_doc, list),
    ]
}

/// The MIME type of image bytes, from their signature (`None`: not an image the board reads).
pub fn sniff_mime(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("image/png")
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some("image/jpeg")
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("image/gif")
    } else if bytes.starts_with(b"BM") {
        Some("image/bmp")
    } else {
        None
    }
}

fn records(doc: &vectorcraft_doc::Document) -> Map<String, Value> {
    doc.extra.get(REFERENCE_ASSETS_KEY).and_then(Value::as_object).cloned().unwrap_or_default()
}

fn embed(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "reference.embed";
    let path = p.get("path").and_then(Value::as_str).filter(|p| !p.is_empty()).ok_or_else(|| bad(C, "`path` is required"))?;
    let size = std::fs::metadata(path).map_err(|e| bad(C, format!("{path}: {e}")))?.len();
    if size > MAX_REFERENCE_BYTES {
        return Err(bad(C, "the image is too large to store"));
    }
    let bytes = std::fs::read(path).map_err(|e| bad(C, format!("{path}: {e}")))?;
    let mime = match p.get("mime").and_then(Value::as_str).filter(|m| !m.is_empty()) {
        Some(m) => m.to_string(),
        None => sniff_mime(&bytes).ok_or_else(|| bad(C, "the file isn't a PNG, JPEG, WebP, GIF or BMP image"))?.to_string(),
    };
    let sha = etcfile::sha256_hex(&bytes);
    let key = format!("{REFERENCE_ASSET_PREFIX}{}", &sha[..16]);
    let name = p.get("originalFilename").and_then(Value::as_str).unwrap_or_default().to_string();
    let mut rec = records(&s.doc()?.doc);
    if rec.contains_key(&key) && s.doc()?.doc.images.contains_key(&key) {
        return Ok(json!({ "assetKey": key, "sha256": sha }));
    }
    rec.insert(key.clone(), json!({ "sha256": sha, "originalFilename": name, "mime": mime }));
    let blob = ImageBlob { mime, bytes: Arc::new(bytes), proxy: None };
    let k = key.clone();
    s.edit("Embed Reference", move |d, _| {
        d.images.insert(k, blob);
        d.extra.insert(REFERENCE_ASSETS_KEY.into(), Value::Object(rec));
        Ok(())
    })?;
    Ok(json!({ "assetKey": key, "sha256": sha }))
}

fn asset(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "reference.asset";
    let key = p.get("assetKey").and_then(Value::as_str).ok_or_else(|| bad(C, "`assetKey` is required"))?;
    let path = p.get("path").and_then(Value::as_str).filter(|p| !p.is_empty()).ok_or_else(|| bad(C, "`path` is required"))?;
    let doc = &s.doc()?.doc;
    let blob = doc.images.get(key).filter(|_| key.starts_with(REFERENCE_ASSET_PREFIX)).ok_or_else(|| bad(C, format!("no stored reference `{key}`")))?;
    std::fs::write(path, blob.bytes.as_slice()).map_err(|e| bad(C, format!("{path}: {e}")))?;
    let rec = records(doc).get(key).cloned().unwrap_or_else(|| json!({}));
    Ok(json!({
        "assetKey": key,
        "mime": blob.mime,
        "path": path,
        "sha256": rec.get("sha256").cloned().unwrap_or(Value::Null),
        "originalFilename": rec.get("originalFilename").cloned().unwrap_or(Value::Null),
    }))
}

fn list(s: &mut Session, _: &Value) -> Result<Value> {
    let doc = &s.doc()?.doc;
    let assets: Vec<Value> = records(doc)
        .into_iter()
        .filter(|(k, _)| doc.images.contains_key(k))
        .map(|(k, mut v)| {
            v["assetKey"] = json!(k);
            v
        })
        .collect();
    Ok(json!({ "assets": assets }))
}

// GLOSSARY
// - REFERENCE_ASSETS_KEY: the document `extra` key recording stored reference images.
// - REFERENCE_ASSET_PREFIX: the image key prefix of a stored reference image. MAX_REFERENCE_BYTES: the largest stored.
// - specs: reference.embed, reference.asset, reference.list.
// - sniff_mime(bytes): an image's MIME type from its signature.
// - records(doc): the stored references' records.
// - embed, asset, list: the commands.
