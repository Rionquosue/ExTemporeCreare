//! Ruler guides in `.vectorcraft` files of every version: the saved `guides` list becomes ruler
//! guide nodes on their layers when a file opens, and is written back as the same list.

use serde::Deserialize;
use serde_json::{Value, json};
use vectorcraft_doc::{Document, LayerColor, Node, NodeId};
use vectorcraft_format::{MIN_VERSION, SaveOptions, VERSION, load, save_with};
use vectorcraft_testkit::fixtures;

fn json_of(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).unwrap()
}

fn save_v(d: &Document, version: u32) -> Vec<u8> {
    save_with(d, &SaveOptions { version, ..SaveOptions::default() }).unwrap()
}

/// The document of a saved file without its guide list and its next id (giving guides ids
/// raises the next id).
fn without_guides(v: &Value) -> Value {
    let mut d = v["document"].clone();
    let o = d.as_object_mut().unwrap();
    o.remove("guides");
    o.remove("next_id");
    d
}

/// A document with art on two layers, a sublayer in the bottom one, and two artboards:
/// (document, bottom layer, sublayer, top layer, second artboard's id).
fn doc() -> (Document, NodeId, NodeId, NodeId, u32) {
    let mut d = (*fixtures::rich_session().doc().unwrap().doc).clone();
    let bottom = d.layers[0].id;
    let sub = d.alloc_id();
    d.insert(Some(bottom), usize::MAX, Node::layer(sub, "Sub", LayerColor::Preset(1))).unwrap();
    let top = d.add_layer(Some("Top"));
    let mut ab = d.artboards[0].clone();
    ab.id = d.next_artboard_id();
    d.artboards.push(ab);
    let board = d.artboards[1].id;
    (d, bottom, sub, top, board)
}


/// The guide entry of a build from before ruler guides were nodes, field for field.
#[derive(Debug, PartialEq, Deserialize)]
struct OldGuide {
    vertical: bool,
    pos: f64,
    #[serde(default)]
    artboard: Option<u32>,
    #[serde(default)]
    layer: Option<NodeId>,
}

