//! `.vectorcraft` v3: compression, pruning unused images, keys from newer versions, colour
//! profiles as a document field, saving for older versions and the embedded preview.

use std::io::Write as _;
use std::sync::Arc;

use serde_json::{Value, json};
use vectorcraft_doc::{ColorProfiles, Composer, Document, ImageBlob, ImageObject, Node, NodeKind, OpacityMask, Symbol};
use vectorcraft_format::{FormatError, SaveOptions, VERSION, is_compressed, load, preview, save, save_with, sniff};
use vectorcraft_geom::Affine;
use vectorcraft_testkit::fixtures;
use vectorcraft_testkit::invariants::{check_native_roundtrip, doc_json, json_approx_eq};

fn rich_doc() -> Document {
    (*fixtures::rich_session().doc().unwrap().doc).clone()
}

fn json_of(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).unwrap()
}

fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut gz = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    gz.write_all(bytes).unwrap();
    gz.finish().unwrap()
}

fn blob(bytes: &[u8]) -> ImageBlob {
    ImageBlob::new("image/png", bytes.to_vec())
}

fn image_node(d: &mut Document, key: &str) -> Node {
    let id = d.alloc_id();
    Node::new(
        id,
        NodeKind::Image(Box::new(ImageObject {
            key: key.into(),
            width: 2,
            height: 2,
            xf: Affine::IDENTITY,
            link: None,
            placement: Default::default(),
            pixels: None,
        })),
    )
}

fn opts(version: u32, compress: bool) -> SaveOptions {
    SaveOptions { version, compress, ..SaveOptions::default() }
}



fn flate2_unpack(bytes: &[u8]) -> Vec<u8> {
    use std::io::Read as _;
    let mut out = vec![];
    flate2::read::GzDecoder::new(bytes).read_to_end(&mut out).unwrap();
    out
}

#[test]
fn unused_images_are_pruned() {
    let mut d = Document::new(100.0, 100.0);
    let layer = d.layers[0].id;
    for (k, b) in [("used", &[1u8, 2][..]), ("unused", &[3, 4]), ("in symbol", &[5]), ("in brush", &[6]), ("in mask", &[7])] {
        d.images.insert(k.into(), blob(b));
    }
    let mut n = image_node(&mut d, "used");
    n.extras_mut().mask = Some(Box::new(OpacityMask::new(image_node(&mut d, "in mask"), false)));
    d.insert(Some(layer), 0, n).unwrap();
    let art = image_node(&mut d, "in symbol");
    d.symbols.push(Symbol { dynamic: false, name: "S".into(), art: Arc::new(art) });
    // Image objects inside foreign data (a brush library kept as JSON) keep theirs too.
    d.unknown.insert("brushes".into(), json!([{"art": {"type": "image", "key": "in brush"}}]));
    let back = load(&save(&d, true)).unwrap();
    let mut keys: Vec<&str> = back.images.keys().map(String::as_str).collect();
    keys.sort_unstable();
    assert_eq!(keys, ["in brush", "in mask", "in symbol", "used"]);
    assert_eq!(back.images["used"].bytes.as_slice(), &[1, 2]);
    let v = json_of(&save(&d, false));
    assert!(v["images"].get("unused").is_none() && v["document"]["images"].as_object().is_some_and(|m| m.is_empty()));
}


/// An opacity-mask editing state in a file drops the editing layer it names only when that is a
/// top-level layer, as files saved mid-edit have it: a sublayer it names loads and stays.
#[test]
fn saved_mask_editing_drops_only_a_top_level_layer() {
    let mut d = Document::new(612.0, 792.0);
    let host = d.layers[0].id;
    let sub = d.add_layer(Some("Sub"));
    d.move_node(sub, Some(host), 0).unwrap();
    let top = d.add_layer(Some("Mask"));
    for (layer, kept) in [(sub, true), (top, false)] {
        let mut v = json_of(&save(&d, false));
        v["document"]["mask_edit"] = json!({"object": host, "layer": layer});
        let back = load(&serde_json::to_vec(&v).unwrap()).unwrap();
        assert_eq!(back.node(layer).is_some(), kept, "layer {layer:?}");
        assert!(back.mask_edit.is_none() && back.node(host).is_some());
    }
}


/// Every composer reads back as it was saved (the two Standard ones as their own values), and the
/// ME&SA names are the ones files have always used: `everyLine` stays the unwritten default and
/// `singleLine` keeps its spelling, so a document that never picks a Standard composer is unchanged.
#[test]
fn all_four_composers_round_trip() {
    use vectorcraft_doc::{TextKind, TextObject};
    let one = |c: Composer| {
        let mut d = Document::new(200.0, 200.0);
        let mut t = TextObject::point(vectorcraft_geom::Point::ZERO, "ક\u{ACD}ક", Default::default());
        t.para.composer = c;
        t.kind = TextKind::Point;
        let id = d.alloc_id();
        d.insert(None, usize::MAX, Node::new(id, NodeKind::Text(Box::new(t)))).unwrap();
        d
    };
    for (c, written) in [
        (Composer::SingleLine, Some("singleLine")),
        (Composer::EveryLine, None),
        (Composer::StandardSingleLine, Some("standardSingleLine")),
        (Composer::StandardEveryLine, Some("standardEveryLine")),
    ] {
        let bytes = save(&one(c), false);
        let text = String::from_utf8_lossy(&bytes);
        match written {
            Some(w) => assert!(text.contains(&format!("\"composer\":\"{w}\"")), "{c:?}"),
            None => assert!(!text.contains("\"composer\""), "{c:?}"),
        }
        let back = load(&bytes).unwrap();
        let mut got = None;
        back.walk(|n| {
            if let NodeKind::Text(t) = &n.kind {
                got = Some(t.para.composer);
            }
        });
        assert_eq!(got, Some(c));
    }
}

/// A document as the first version wrote it (format name `drawcraft`, anchors as maps).
const V1_FILE: &str = r#"{"format":"drawcraft","version":1,"generator":"DrawCraft 0.1.0","document":{"version":1,"title":"Old","artboards":[{"id":1,"name":"Artboard 1","rect":{"x0":0.0,"y0":0.0,"x1":100.0,"y1":80.0}}],
"layers":[{"id":1,"name":"Layer 1","visible":true,"locked":false,"opacity":1.0,"kind":{"type":"layer","children":[
{"id":2,"visible":true,"locked":false,"opacity":1.0,"kind":{"type":"path","path":{"subpaths":[{"anchors":[
{"p":{"x":10.0,"y":10.0},"in":{"x":10.0,"y":10.0},"out":{"x":30.0,"y":0.0},"kind":"Smooth"},
{"p":{"x":50.0,"y":40.0},"in":{"x":50.0,"y":40.0},"out":{"x":50.0,"y":40.0},"kind":"Corner"}],"closed":false}]},"rule":"NonZero","clipping":false,"guide":false}}],"color":{"Preset":0}}}],
"next_id":3,"unknown":{"colorProfiles":{"rgb":"sRGB IEC61966-2.1","cmyk":null}}},"images":{}}"#;

#[test]
fn version_1_files_load() {
    let d = load(V1_FILE.as_bytes()).unwrap();
    assert_eq!(d.title, "Old");
    let NodeKind::Path { path, .. } = &d.layers[0].children().unwrap()[0].kind else { panic!("not a path") };
    let a = &path.subpaths[0].anchors;
    assert_eq!((a[0].h_out.x, a[0].h_out.y, a[1].p.x), (30.0, 0.0, 50.0));
    // Profiles migrate out of `unknown`.
    assert_eq!(d.color_profiles, ColorProfiles { rgb: Some("sRGB IEC61966-2.1".into()), cmyk: None });
    assert!(d.unknown.is_empty());
}


#[test]
fn preview_is_embedded_and_read_back() {
    let d = Document::new(10.0, 10.0);
    assert_eq!(preview(&save(&d, false)), None);
    let png = vec![0x89, b'P', b'N', b'G', 1, 2, 3];
    for compress in [false, true] {
        let bytes = save_with(&d, &SaveOptions { compress, preview: Some(png.clone()), ..SaveOptions::default() }).unwrap();
        assert_eq!(preview(&bytes), Some(png.clone()));
        assert!(load(&bytes).is_ok());
    }
    assert_eq!(preview(b"not a file"), None);
}


/// #451: a guide's artboard is written only for artboard guides, so canvas guides save as they
/// did, and a file without it (an older version's) opens with canvas guides.
#[test]
fn artboard_guides_round_trip_and_canvas_guides_save_as_before() {
    use vectorcraft_doc::Guide;
    let mut d = Document::new(100.0, 100.0);
    let id = d.artboards[0].id;
    d.add_ruler_guide(&Guide::new(true, 10.0));
    d.add_ruler_guide(&Guide { artboard: Some(id), ..Guide::new(false, 20.0) });
    let mut v = json_of(&save(&d, false));
    let lines: Vec<_> = v["document"]["guides"]
        .as_array()
        .unwrap()
        .iter()
        .map(|g| json!({"vertical": g["vertical"], "pos": g["pos"], "artboard": g.get("artboard")}))
        .collect();
    assert_eq!(lines, [json!({"vertical": true, "pos": 10.0, "artboard": null}), json!({"vertical": false, "pos": 20.0, "artboard": id})]);
    assert_eq!(load(&save(&d, false)).unwrap().ruler_guides(), d.ruler_guides());
    v["document"]["guides"] = json!([{"vertical": false, "pos": 20.0}]);
    let old = load(&serde_json::to_vec(&v).unwrap()).unwrap();
    let old_lines: Vec<_> = old.ruler_guides().into_iter().map(|g| (g.vertical, g.pos, g.artboard)).collect();
    assert_eq!(old_lines, [(false, 20.0, None)]);
}

