//! PDF and .ai files that reopen as they were: Preserve Editing embeds the native document, which
//! `document.open` restores unless the pages changed elsewhere; `.ai` saves always carry it and
//! `.ait` templates open from it untitled.

use serde_json::{Value, json};

use super::tests_svgedit::{comparable, rich};
use super::*;

fn b64(v: &Value) -> Vec<u8> {
    etcfile::base64::decode(v["dataBase64"].as_str().unwrap_or_else(|| panic!("no dataBase64 in {v}"))).unwrap()
}

fn open(s: &mut Session, name: &str, bytes: &[u8]) -> Value {
    s.execute("document.open", &json!({"name": name, "dataBase64": etcfile::base64::encode(bytes)})).unwrap()
}


/// `pdf` with `from` replaced by `to` (the same length).
fn patched(pdf: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    assert_eq!(from.len(), to.len());
    let at = pdf.windows(from.len()).position(|w| w == from).unwrap_or_else(|| panic!("no {:?}", String::from_utf8_lossy(from)));
    let mut out = pdf.to_vec();
    out[at..at + to.len()].copy_from_slice(to);
    out
}

#[test]
fn a_pdf_saved_with_preserve_editing_reopens_as_the_same_document() {
    let mut s = rich();
    let before = comparable(&s.doc().unwrap().doc);
    // On in the default preset.
    let v = s.execute("document.exportPdf", &json!({})).unwrap();
    assert_eq!(v["warnings"].as_array().map(|w| w.iter().any(|w| w.as_str().unwrap().contains("Preserve"))), Some(false), "{v}");
    let pdf = b64(&v);
    let r = open(&mut s, "rich.pdf", &pdf);
    assert_eq!((&r["warnings"], &r["restored"]), (&json!([]), &json!(true)), "{r}");
    assert_eq!(comparable(&s.doc().unwrap().doc), before, "the document came back exactly");
    assert_eq!(s.doc().unwrap().path, None, "a PDF is not saved over");
    // Picking pages imports them as artwork.
    let r = s.execute("document.open", &json!({"name": "rich.pdf", "dataBase64": etcfile::base64::encode(&pdf), "pages": "1"})).unwrap();
    assert_eq!(r["restored"], false, "{r}");
    // Off: the pages import as artwork.
    let plain = b64(&s.execute("document.exportPdf", &json!({"preserveEditing": false})).unwrap());
    assert!(vectorcraft_pdf::editing(&plain).is_none());
    assert_eq!(open(&mut s, "plain.pdf", &plain)["restored"], false);
    // document.export and serialize take the same option.
    let v = s.execute("document.serialize", &json!({"format": "pdf"})).unwrap();
    assert!(vectorcraft_pdf::editing(&b64(&v)).is_some_and(|e| e.intact));
}

#[test]
fn pages_changed_elsewhere_open_as_artwork_with_a_warning() {
    let mut s = rich();
    let pdf = b64(&s.execute("document.exportPdf", &json!({"compression": {"compressText": false}})).unwrap());
    // Another app changes the artboard's height (300 × 200 pt).
    let edited = patched(&pdf, b"/MediaBox[0 0 300 200]", b"/MediaBox[0 0 300 201]");
    let r = open(&mut s, "edited.pdf", &edited);
    assert_eq!((&r["warnings"][0], &r["restored"]), (&json!(load::EDITING_STALE), &json!(false)), "{r}");
    assert!(s.doc().unwrap().doc.symbols.is_empty(), "plain import: no symbols");
    // Editing data that isn't a native document can't be read.
    let damaged = patched(&pdf, b"{\"format\":\"vectorcraft\"", b"{\"format\":\"vectorcrafX\"");
    let r = open(&mut s, "damaged.pdf", &damaged);
    assert_eq!(r["warnings"][0], load::EDITING_DAMAGED, "{r}");
}

/// `pdf` with its editing file's description rewritten by another app in UTF-16BE with a byte
/// order mark: "hash " and the hash it held, padded with spaces to its length.
fn utf16_described(pdf: &[u8]) -> Vec<u8> {
    let from = b"(VectorCraft editing data; page hash ";
    let at = pdf.windows(from.len()).position(|w| w == from).unwrap_or_else(|| panic!("no description"));
    let (hash, len) = (&pdf[at + from.len()..at + from.len() + 16], from.len() + 17);
    let mut desc = b"(\xFE\xFF".to_vec();
    desc.extend(b"hash ".iter().chain(hash).flat_map(|&c| [0, c]));
    desc.push(b')');
    desc.resize(len, b' ');
    patched(pdf, &pdf[at..at + len], &desc)
}

#[test]
fn pages_changed_elsewhere_are_noticed_with_a_utf16_description() {
    let mut s = rich();
    let pdf = b64(&s.execute("document.exportPdf", &json!({"compression": {"compressText": false}})).unwrap());
    for (name, file) in [("edited.pdf", pdf)] {
        let r = open(&mut s, name, &utf16_described(&file));
        assert_eq!((&r["warnings"], &r["restored"]), (&json!([]), &json!(true)), "{name}: the same pages: {r}");
        let edited = patched(&file, b"/MediaBox[0 0 300 200]", b"/MediaBox[0 0 300 201]");
        let r = open(&mut s, name, &utf16_described(&edited));
        assert_eq!((&r["warnings"][0], &r["restored"]), (&json!(load::EDITING_STALE), &json!(false)), "{name}: {r}");
    }
}


#[test]
fn the_legacy_payload_name_is_accepted() {
    let mut s = rich();
    let before = comparable(&s.doc().unwrap().doc);
    let pdf = b64(&s.execute("document.exportPdf", &json!({"compression": {"compressText": false}})).unwrap());
    let (name, legacy) = (format!("({})", vectorcraft_pdf::EDITING_FILE), format!("({})", vectorcraft_pdf::LEGACY_EDITING_FILE));
    let legacy = format!("{legacy}{}", " ".repeat(name.len() - legacy.len()));
    let mut old = pdf.clone();
    while let Some(at) = old.windows(name.len()).position(|w| w == name.as_bytes()) {
        old[at..at + name.len()].copy_from_slice(legacy.as_bytes());
    }
    assert!(!String::from_utf8_lossy(&old).contains(vectorcraft_pdf::EDITING_FILE));
    assert_eq!(open(&mut s, "old.pdf", &old)["restored"], true);
    assert_eq!(comparable(&s.doc().unwrap().doc), before);
}

#[test]
fn standards_and_screens_leave_the_editing_data_out() {
    let mut s = rich();
    // Choosing PDF/A turns Preserve Editing off unless it is asked for, which PDF/A refuses.
    let pdf_a = b64(&s.execute("document.exportPdf", &json!({"standard": "pdfA2b"})).unwrap());
    assert!(vectorcraft_pdf::editing(&pdf_a).is_none());
    let e = s.execute("document.exportPdf", &json!({"standard": "pdfA2b", "preserveEditing": true})).unwrap_err();
    assert!(matches!(e, crate::EngineError::BadParams { .. }), "{e}");
    // Export for Screens writes a PDF per artboard for viewing.
    let r = s.execute("document.exportForScreens", &json!({"formats": [{"format": "pdf"}]})).unwrap();
    let file = etcfile::base64::decode(r["files"][0]["dataBase64"].as_str().unwrap()).unwrap();
    assert!(vectorcraft_pdf::editing(&file).is_none());
}

#[test]
fn save_dialogs_offer_every_format_save_writes() {
    let labels = |name: &str| save_filters(name).into_iter().map(|(l, _)| l).collect::<Vec<_>>();
    assert_eq!(labels("a.etc").len(), SAVE_FORMATS.len());
    assert_eq!(labels("a.svg")[0], "SVG");
    assert!(labels("a.png").is_empty() && labels("noext").is_empty(), "exports pick their own name");
    assert!(save_format(Some("png"), None).is_err());
}

#[test]
fn a_pdf_of_some_artboards_carries_no_editing_data() {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 100, "height": 80, "artboards": 3})).unwrap();
    for (range, kept) in [("2", false), ("1,3", false), ("3,2,1", false), ("1-3", true), ("all", true)] {
        let v = s.execute("document.exportPdf", &json!({ "range": range })).unwrap();
        assert_eq!(vectorcraft_pdf::editing(&b64(&v)).is_some(), kept, "{range}");
        let warned = v["warnings"].as_array().unwrap().iter().any(|w| w == super::pdf::EDITING_NEEDS_EVERY_ARTBOARD);
        assert_eq!(warned, !kept, "{range}: {v}");
    }
}

/// Color Halftone's Max Radius is in pixels of the raster effects resolution since document
/// version 3 (#3260). A PDF saved with Preserve Editing carries the native document, so one from
/// before that opens with the radius converted, drawing the same dots; one saved since opens as
/// it was.
#[test]
fn a_preserve_editing_pdf_opens_with_its_halftone_radius_in_pixels() {
    let halftone = |old: bool| {
        let mut s = Session::new();
        s.execute("file.new", &json!({"width": 300, "height": 200})).unwrap();
        s.execute("document.rasterEffectsSettings", &json!({"resolution": 300})).unwrap();
        let r = super::tests_svgedit::rect(&mut s, 10.0, 10.0);
        s.execute("effect.apply", &json!({"effect": "pixelate.colorHalftone", "params": {"maxRadius": 8}, "ids": [r]})).unwrap();
        if old {
            // As VectorCraft 0.18 wrote it: the document model's version 1, the radius in points.
            std::sync::Arc::make_mut(&mut s.doc_mut().unwrap().doc).version = 1;
        }
        let pdf = b64(&s.execute("document.exportPdf", &json!({})).unwrap());
        let opened = open(&mut s, "halftone.pdf", &pdf);
        assert_eq!(opened["restored"], json!(true), "{opened}");
        let d = &s.doc().unwrap().doc;
        assert_eq!(d.version, vectorcraft_doc::FORMAT_VERSION);
        let n = d.node(vectorcraft_doc::NodeId(r)).unwrap();
        n.appearance.effects.iter().find(|e| e.id == "pixelate.colorHalftone").and_then(|e| e.params["maxRadius"].as_f64()).unwrap()
    };
    // 8 pt at 300 ppi are 33⅓ px.
    assert!((halftone(true) - 8.0 * 300.0 / 72.0).abs() < 1e-9, "{}", halftone(true));
    assert_eq!(halftone(false), 8.0);
}
















