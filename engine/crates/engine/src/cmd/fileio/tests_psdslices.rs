//! A PSD or PSB file's slices (image resource 1050) opened and placed as user slices
//! (`document.open`'s and `file.place`'s `importSlices`): version 6 records and version 7 and 8
//! descriptors, built from the Photoshop File Formats Specification's "Slices resource format";
//! their options, slices outside the canvas, damaged or unknown slice data, placing scaled and
//! moved in one undo step, linked files, and `document.psdInfo`'s count.

use serde_json::{Value, json};
use vectorcraft_doc::SliceOptions;
use vectorcraft_geom::Rect;

use crate::Session;

/// A Unicode string: a count of UTF-16 units, then the units.
fn ustr(s: &str) -> Vec<u8> {
    let units: Vec<u16> = s.encode_utf16().collect();
    let mut b = (units.len() as u32).to_be_bytes().to_vec();
    b.extend(units.iter().flat_map(|u| u.to_be_bytes()));
    b
}

/// One version 6 slice record.
#[derive(Clone)]
struct S {
    /// 0: generated automatically, 1: follows a layer, 2: made by the user.
    origin: u32,
    name: &'static str,
    /// 0: No Image, 1: Image.
    kind: u32,
    /// Left, top, right, bottom (px).
    rect: [i32; 4],
    url: &'static str,
    target: &'static str,
    message: &'static str,
    alt: &'static str,
    html: bool,
    cell: &'static str,
    /// Alpha, red, green, blue.
    argb: [u8; 4],
}

fn user(name: &'static str, rect: [i32; 4]) -> S {
    S { origin: 2, name, kind: 1, rect, url: "", target: "", message: "", alt: "", html: false, cell: "", argb: [0; 4] }
}

/// Resource 1050 in version 6, each record followed by `trailing` (a Photoshop 7 descriptor, or
/// nothing).
fn v6(slices: &[S], trailing: &[u8]) -> Vec<u8> {
    let mut b = 6u32.to_be_bytes().to_vec();
    b.extend([0u8; 16]);
    b.extend(ustr("group"));
    b.extend((slices.len() as u32).to_be_bytes());
    for (i, s) in slices.iter().enumerate() {
        b.extend((i as u32 + 1).to_be_bytes());
        b.extend(0u32.to_be_bytes());
        b.extend(s.origin.to_be_bytes());
        if s.origin == 1 {
            b.extend(7u32.to_be_bytes());
        }
        b.extend(ustr(s.name));
        b.extend(s.kind.to_be_bytes());
        for v in s.rect {
            b.extend(v.to_be_bytes());
        }
        for t in [s.url, s.target, s.message, s.alt] {
            b.extend(ustr(t));
        }
        b.push(u8::from(s.html));
        b.extend(ustr(s.cell));
        b.extend([0u8; 8]);
        b.extend(s.argb);
        b.extend(trailing);
    }
    b
}

/// A descriptor value.
enum D {
    Long(i32),
    Text(&'static str),
    Enum(&'static str, &'static str),
    Obj(Vec<(&'static str, D)>),
    List(Vec<D>),
}

/// A key or class ID: a four-character one with length 0, else its length and characters.
fn key(k: &str) -> Vec<u8> {
    let mut b = if k.len() == 4 { 0u32 } else { k.len() as u32 }.to_be_bytes().to_vec();
    b.extend(k.as_bytes());
    b
}

/// A descriptor: an empty class name, the class ID, the count, then each item.
fn descriptor(items: &[(&str, D)]) -> Vec<u8> {
    let mut b = ustr("");
    b.extend(key("null"));
    b.extend((items.len() as u32).to_be_bytes());
    for (k, v) in items {
        b.extend(key(k));
        b.extend(value(v));
    }
    b
}

/// A value's type, then the value.
fn value(v: &D) -> Vec<u8> {
    let mut b = vec![];
    match v {
        D::Long(n) => {
            b.extend(b"long");
            b.extend(n.to_be_bytes());
        }
        D::Text(t) => {
            b.extend(b"TEXT");
            b.extend(ustr(t));
        }
        D::Enum(t, e) => {
            b.extend(b"enum");
            b.extend(key(t));
            b.extend(key(e));
        }
        D::Obj(items) => {
            b.extend(b"Objc");
            b.extend(descriptor(items));
        }
        D::List(items) => {
            b.extend(b"VlLs");
            b.extend((items.len() as u32).to_be_bytes());
            for i in items {
                b.extend(value(i));
            }
        }
    }
    b
}

/// A bounds object: top, left, bottom and right.
fn bounds(l: i32, t: i32, r: i32, b: i32) -> D {
    D::Obj(vec![("Top ", D::Long(t)), ("Left", D::Long(l)), ("Btom", D::Long(b)), ("Rght", D::Long(r))])
}

/// Resource 1050 in `version` 7 or 8 holding `slices`.
fn v7(version: u32, slices: Vec<D>) -> Vec<u8> {
    let mut b = version.to_be_bytes().to_vec();
    b.extend(16u32.to_be_bytes());
    b.extend(descriptor(&[("bounds", bounds(0, 0, 20, 20)), ("baseName", D::Text("poster")), ("slices", D::List(slices))]));
    b
}

/// A 20 × 20 px PSD file written by `document.export`, with its layers (two) or flat.
fn psd(layers: bool) -> Vec<u8> {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 20, "height": 20})).unwrap();
    s.execute("paint.setFill", &json!({ "color": "#ff0000" })).unwrap();
    s.execute("shape.rectangle", &json!({"x": 2, "y": 2, "width": 10, "height": 10})).unwrap();
    s.execute("layer.new", &json!({ "name": "Top" })).unwrap();
    s.execute("shape.rectangle", &json!({"x": 8, "y": 8, "width": 10, "height": 10})).unwrap();
    let r = s.execute("document.export", &json!({"format": "psd", "layers": layers})).unwrap();
    etcfile::base64::decode(r["dataBase64"].as_str().unwrap()).unwrap()
}

/// `psd` with image resource `id` holding `data` added to its image resources.
fn with_resource(psd: &[u8], id: u16, data: &[u8]) -> Vec<u8> {
    let u32_at = |at: usize| u32::from_be_bytes(psd[at..at + 4].try_into().unwrap()) as usize;
    let colors = 26 + 4 + u32_at(26);
    let len = u32_at(colors);
    let mut block = b"8BIM".to_vec();
    block.extend(id.to_be_bytes());
    block.extend([0, 0]);
    block.extend((data.len() as u32).to_be_bytes());
    block.extend(data);
    block.resize(block.len().next_multiple_of(2), 0);
    let mut out = psd[..colors].to_vec();
    out.extend(((len + block.len()) as u32).to_be_bytes());
    out.extend(&psd[colors + 4..colors + 4 + len]);
    out.extend(block);
    out.extend(&psd[colors + 4 + len..]);
    out
}

fn b64(f: &[u8]) -> String {
    etcfile::base64::encode(f)
}

fn open(f: &[u8], extra: Value) -> (Session, Value) {
    let mut s = Session::new();
    let mut p = json!({"name": "poster.psd", "dataBase64": b64(f)});
    for (k, v) in extra.as_object().unwrap() {
        p[k] = v.clone();
    }
    let r = s.execute("document.open", &p).unwrap();
    (s, r)
}

/// The document's user slices: rectangle and options.
fn slices(s: &Session) -> Vec<(Rect, SliceOptions)> {
    s.doc().unwrap().doc.slices.iter().map(|x| (x.rect, x.options.clone())).collect()
}

fn warnings(r: &Value) -> Vec<String> {
    r["warnings"].as_array().map(|a| a.iter().filter_map(|w| w.as_str().map(str::to_string)).collect()).unwrap_or_default()
}






#[test]
fn import_slices_false_leaves_them_out_and_must_be_a_switch() {
    let f = with_resource(&psd(true), 1050, &v6(&[user("a", [0, 0, 5, 5])], &[]));
    let (s, _) = open(&f, json!({"importSlices": false}));
    assert!(slices(&s).is_empty());
    let mut s = Session::new();
    let e = s.execute("document.open", &json!({"name": "poster.psd", "dataBase64": b64(&f), "importSlices": "yes"})).unwrap_err();
    assert!(e.to_string().contains("importSlices must be true or false"), "{e}");
}


#[test]
fn a_slices_resource_cut_short_warns_and_the_file_still_opens() {
    let good = v6(&[user("a", [0, 0, 5, 5])], &[]);
    // Image resource `id`, the last of the image resources, declares 10 bytes more than they hold.
    let cut = |layers: bool, id: u16, data: &[u8]| {
        let mut f = with_resource(&psd(layers), id, data);
        let sig = [&b"8BIM"[..], &id.to_be_bytes()].concat();
        let at = f.windows(6).rposition(|w| w == sig.as_slice()).unwrap() + 8;
        f[at..at + 4].copy_from_slice(&(data.len() as u32 + 10).to_be_bytes());
        f
    };
    for layers in [true, false] {
        let (s, r) = open(&cut(layers, 1050, &good), json!({}));
        let w = warnings(&r);
        assert!(w.iter().any(|w| w.starts_with("the slices can't be read (") && w.contains("cut short")), "{w:?}");
        assert!(slices(&s).is_empty());
        assert!(s.doc().unwrap().doc.art_bounds().is_some());
        // Another resource cut short says nothing about slices.
        let (s, r) = open(&cut(layers, 1051, &[0, 1]), json!({}));
        assert!(warnings(&r).is_empty() && slices(&s).is_empty(), "{r}");
    }
}




#[test]
fn psd_info_counts_the_slices_that_come_in() {
    let mut s = Session::new();
    let info = |s: &mut Session, f: &[u8]| s.execute("document.psdInfo", &json!({"dataBase64": b64(f)})).unwrap();
    let data = v6(&[user("a", [0, 0, 5, 5]), S { origin: 0, ..user("auto", [5, 5, 20, 20]) }, user("out", [30, 30, 40, 40])], &[]);
    assert_eq!(info(&mut s, &with_resource(&psd(false), 1050, &data))["slices"], json!(1));
    assert_eq!(info(&mut s, &with_resource(&psd(true), 1050, &data[..data.len() - 4]))["slices"], json!(0));
    assert_eq!(info(&mut s, &psd(true))["slices"], json!(0));
}

#[test]
fn damaged_slices_never_panic() {
    let data = v7(8, vec![D::Obj(vec![("origin", D::Enum("ESliceOrigin", "userGenerated")), ("bounds", bounds(0, 0, 5, 5))])]);
    let base = with_resource(&psd(false), 1050, &data);
    let start = base.windows(data.len()).position(|w| w == data).unwrap();
    for at in (start - 12..start + data.len()).step_by(3) {
        for v in [0u8, 0x7f, 0xff] {
            let mut f = base.clone();
            f[at] = v;
            let mut s = Session::new();
            // The engine's guard reports a panic as an internal error.
            for r in [
                s.execute("document.open", &json!({"name": "poster.psd", "dataBase64": b64(&f)})),
                s.execute("document.psdInfo", &json!({"dataBase64": b64(&f)})),
            ] {
                assert!(!matches!(r, Err(crate::EngineError::Internal { .. })), "byte {at} set to {v}: {r:?}");
            }
        }
    }
}




