//! Native and `.ai` save options: each artboard to a separate file (a master file too), Include
//! Linked Files, Embed ICC Profiles, Create PDF-Compatible File and the `.ai` Use Compression.

use std::sync::Arc;

use serde_json::{Value, json};
use vectorcraft_color::cms;

use crate::tests_links::{Folder, png};
use crate::*;

/// Three artboards (named A, B and C) with a rectangle on each.
fn three_boards() -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
    s.execute("artboard.setProps", &json!({"index": 0, "name": "A"})).unwrap();
    for (i, name) in ["B", "C"].iter().enumerate() {
        let x = 200.0 * (i + 1) as f64;
        s.execute("artboard.new", &json!({"x": x, "y": 0, "width": 100, "height": 100, "name": name})).unwrap();
    }
    for x in [10, 210, 410] {
        s.execute("shape.rectangle", &json!({"x": x, "y": 10, "width": 30, "height": 30})).unwrap();
    }
    s
}

fn objects(d: &vectorcraft_doc::Document) -> usize {
    d.layers.iter().map(|l| l.count() - 1).sum()
}

fn bytes_of(v: &Value) -> Vec<u8> {
    vectorcraft_format::base64_decode(v.as_str().expect("base64")).unwrap()
}

#[test]
fn three_artboards_give_four_files() {
    let dir = Folder::new("separate");
    let mut s = three_boards();
    let path = dir.file("Poster.vectorcraft");
    let r = s.execute("document.save", &json!({"path": path, "separateArtboards": true})).unwrap();
    let files: Vec<&str> = r["files"].as_array().unwrap().iter().map(|f| f.as_str().unwrap()).collect();
    assert_eq!(files, [path.clone(), dir.file("Poster-A.vectorcraft"), dir.file("Poster-B.vectorcraft"), dir.file("Poster-C.vectorcraft")]);
    // The master file has everything; each artboard's file that artboard and its art.
    let master = vectorcraft_format::load(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!((master.artboards.len(), objects(&master)), (3, 3));
    for (f, name, x) in [(files[1], "A", 0.0), (files[2], "B", 200.0), (files[3], "C", 400.0)] {
        let d = vectorcraft_format::load(&std::fs::read(f).unwrap()).unwrap();
        assert_eq!((d.artboards.len(), d.artboards[0].name.as_str(), d.artboards[0].rect.x0), (1, name, x));
        assert_eq!(objects(&d), 1, "{name}");
    }
    // The document takes the master file; Save remembers the choice.
    let st = s.doc().unwrap();
    assert_eq!((st.path.as_deref(), st.is_dirty()), (Some(path.as_str()), false));
    // A range: the master and two artboards.
    let r = s.execute("document.save", &json!({"path": path, "separateArtboards": true, "range": "2-3"})).unwrap();
    assert_eq!(r["files"].as_array().unwrap().len(), 3);
    assert!(s.execute("document.save", &json!({"path": path, "separateArtboards": true, "range": "4"})).is_err(), "no fourth artboard");
    // No path: the bytes of every file.
    let r = s.execute("file.saveCopy", &json!({"separateArtboards": true})).unwrap();
    let names: Vec<&str> = r["files"].as_array().unwrap().iter().map(|f| f["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["Poster copy.vectorcraft", "Poster copy-A.vectorcraft", "Poster copy-B.vectorcraft", "Poster copy-C.vectorcraft"]);
    assert!(vectorcraft_format::sniff(&bytes_of(&r["files"][3]["dataBase64"])));
}


#[test]
fn a_linked_pngs_bytes_are_embedded() {
    let dir = Folder::new("include-linked");
    let pic = dir.file("photo.png");
    let bytes = png(400, 300, [200, 30, 30]);
    std::fs::write(&pic, &bytes).unwrap();
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 500, "height": 500})).unwrap();
    s.execute("file.place", &json!({"path": pic, "link": true})).unwrap();
    let images = |v: &Value| -> Value {
        let text = bytes_of(&v["dataBase64"]);
        serde_json::from_slice::<Value>(&text).unwrap()["images"].clone()
    };
    // By default only the preview.
    let plain = images(&s.execute("file.saveCopy", &json!({"compress": false})).unwrap());
    let (_, image) = plain.as_object().unwrap().iter().next().unwrap();
    assert_eq!(image["proxy"], true);
    // Include Linked Files: the file's own bytes.
    let r = s.execute("file.saveCopy", &json!({"compress": false, "includeLinked": true})).unwrap();
    let full = images(&r);
    let (_, image) = full.as_object().unwrap().iter().next().unwrap();
    assert!(image.get("proxy").is_none());
    assert_eq!(vectorcraft_format::base64_decode(image["data"].as_str().unwrap()).unwrap(), bytes);
    // With the file gone the document still shows the full pixels.
    std::fs::remove_file(&pic).unwrap();
    let r = s.execute("document.open", &json!({"name": "copy.vectorcraft", "dataBase64": r["dataBase64"]})).unwrap();
    assert_eq!(r["missingLinks"].as_array().unwrap().len(), 1);
    let d = &s.doc().unwrap().doc;
    let blob = d.images.values().next().unwrap();
    assert_eq!(blob.bytes.as_slice(), bytes.as_slice());
    let mut linked = 0;
    d.visit_images(|_, im| linked += usize::from(im.link.is_some()));
    assert_eq!(linked, 1, "still linked");
}

#[test]
fn an_embedded_profile_is_restored() {
    if !super::in_own_process("tests::saveoptions::an_embedded_profile_is_restored") {
        return;
    }
    let builtin = cms::profiles().into_iter().find(|p| p.kind == cms::ProfileKind::Rgb && p.builtin && p.name != cms::SRGB).unwrap();
    let icc = cms::icc_bytes(&builtin.name).unwrap();
    let name = format!("Studio RGB {}", std::process::id());
    cms::register_icc(&icc, Some(name.clone())).unwrap();
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
    Arc::make_mut(&mut s.doc_mut().unwrap().doc).color_profiles.rgb = Some(name.clone());
    let file = bytes_of(&s.execute("file.saveCopy", &json!({"compress": false})).unwrap()["dataBase64"]);
    let v: Value = serde_json::from_slice(&file).unwrap();
    assert_eq!(vectorcraft_format::base64_decode(v["profiles"][&name]["data"].as_str().unwrap()).unwrap(), icc.to_vec());
    // Not embedded when asked not to; built-in profiles never are (they are everywhere).
    let none: Value =
        serde_json::from_slice(&bytes_of(&s.execute("file.saveCopy", &json!({"compress": false, "embedProfiles": false})).unwrap()["dataBase64"]))
            .unwrap();
    assert!(none.get("profiles").is_none());
    // Elsewhere (a profile name this machine lacks): opening installs it and the document keeps it.
    let elsewhere = format!("Studio RGB elsewhere {}", std::process::id());
    let moved = String::from_utf8(file).unwrap().replace(&name, &elsewhere);
    assert!(cms::profile(&elsewhere).is_none());
    let r =
        s.execute("document.open", &json!({"name": "moved.vectorcraft", "dataBase64": vectorcraft_format::base64_encode(moved.as_bytes())})).unwrap();
    assert!(r["warnings"].as_array().unwrap().is_empty(), "{r}");
    assert_eq!(cms::profile(&elsewhere).map(|p| p.kind), Some(cms::ProfileKind::Rgb));
    assert_eq!(s.doc().unwrap().doc.color_profiles.rgb.as_deref(), Some(elsewhere.as_str()));
    // Damaged profile data opens the document with a warning.
    let broken = format!("Studio RGB broken {}", std::process::id());
    let mut v: Value = serde_json::from_str(&moved.replace(&elsewhere, &broken)).unwrap();
    v["profiles"][&broken]["data"] = json!("AAAA");
    let r = s
        .execute(
            "document.open",
            &json!({"name": "b.vectorcraft", "dataBase64": vectorcraft_format::base64_encode(&serde_json::to_vec(&v).unwrap())}),
        )
        .unwrap();
    assert!(r["warnings"][0].as_str().unwrap().contains(&broken), "{r}");
}

#[test]
fn pdf_compatible_files_carry_a_pdf() {
    let mut s = three_boards();
    let file = bytes_of(&s.execute("file.saveCopy", &json!({"pdfCompatible": true})).unwrap()["dataBase64"]);
    let pdf = vectorcraft_format::pdf_content(&file).unwrap();
    assert!(pdf.starts_with(b"%PDF"));
    let info = s.execute("document.pdfInfo", &json!({"dataBase64": vectorcraft_format::base64_encode(&pdf)})).unwrap();
    assert_eq!(info["pages"], 3);
    assert!(
        vectorcraft_format::pdf_content(&bytes_of(&s.execute("file.saveCopy", &json!({"pdfCompatible": false})).unwrap()["dataBase64"])).is_none()
    );
}

/// A native save is portable to the other apps by default. The option is still an explicit
/// opt-out, and a remembered opt-out survives later saves of the document.
#[test]
fn native_saves_carry_all_artboards_as_pdf_by_default() {
    let mut s = three_boards();
    for compress in [false, true] {
        let result = s.execute("file.saveCopy", &json!({"compress": compress})).unwrap();
        let file = bytes_of(&result["dataBase64"]);
        let pdf = vectorcraft_format::pdf_content(&file).expect("the default native save carries a PDF");
        let info = s.execute("document.pdfInfo", &json!({"dataBase64": vectorcraft_format::base64_encode(&pdf)})).unwrap();
        assert_eq!(info["pages"], 3);
        assert_eq!(vectorcraft_format::load(&file).unwrap().artboards.len(), 3);
    }
    let dir = Folder::new("pdf-opt-out");
    let path = dir.file("compact.vectorcraft");
    s.execute("file.saveAs", &json!({"path": path, "pdfCompatible": false})).unwrap();
    s.execute("shape.rectangle", &json!({"x": 60, "y": 10, "width": 10, "height": 10})).unwrap();
    s.execute("document.save", &json!({})).unwrap();
    assert!(vectorcraft_format::pdf_content(&std::fs::read(&path).unwrap()).is_none());
    let options = s.execute("file.formatOptions", &json!({"format":"vectorcraft"})).unwrap();
    assert_eq!(options["options"]["pdfCompatible"]["value"], false);
    s.execute("document.save", &json!({"compress": true})).unwrap();
    assert!(vectorcraft_format::pdf_content(&std::fs::read(&path).unwrap()).is_none());
    s.execute("document.open", &json!({"path": path})).unwrap();
    assert_eq!(s.doc().unwrap().save_options["pdfCompatible"], false);
    let options = s.execute("file.formatOptions", &json!({"format": "vectorcraft"})).unwrap();
    assert_eq!(options["options"]["pdfCompatible"]["value"], false);
    s.execute("document.save", &json!({})).unwrap();
    assert!(vectorcraft_format::pdf_content(&std::fs::read(&path).unwrap()).is_none());
    s.execute("document.save", &json!({"pdfCompatible": true})).unwrap();
    assert!(vectorcraft_format::pdf_content(&std::fs::read(&path).unwrap()).is_some());
}

#[test]
fn native_saves_without_artboards_keep_the_editable_document() {
    let mut s = three_boards();
    std::sync::Arc::make_mut(&mut s.doc_mut().unwrap().doc).artboards.clear();
    let result = s.execute("file.saveCopy", &json!({})).unwrap();
    let file = bytes_of(&result["dataBase64"]);
    let restored = vectorcraft_format::load(&file).unwrap();
    assert!(restored.artboards.is_empty());
    assert_eq!(restored.node_count(), s.doc().unwrap().doc.node_count());
    assert!(vectorcraft_format::pdf_content(&file).is_none());
    s.execute("document.open", &json!({"name": "no-artboards.vectorcraft", "dataBase64": vectorcraft_format::base64_encode(&file)})).unwrap();
    assert_eq!(s.doc().unwrap().save_options["pdfCompatible"], true);
    s.execute("artboard.new", &json!({"x": 0, "y": 0, "width": 100, "height": 80})).unwrap();
    let result = s.execute("file.saveCopy", &json!({})).unwrap();
    assert!(vectorcraft_format::pdf_content(&bytes_of(&result["dataBase64"])).is_some());
}

#[test]
fn native_template_saves_keep_the_document_pdf_preference() {
    let mut s = three_boards();
    let dir = Folder::new("template-pdf-preference");
    s.execute("file.saveAs", &json!({"path": dir.file("compact.vectorcraft"), "pdfCompatible": false, "version": 3})).unwrap();
    let options = s.execute("file.formatOptions", &json!({"format": "template"})).unwrap();
    assert_eq!(options["options"]["pdfCompatible"]["value"], false, "the save dialog must offer the same retained choice");
    let r = s.execute("file.saveAsTemplate", &json!({"format": "template"})).unwrap();
    let bytes = bytes_of(&r["dataBase64"]);
    assert!(vectorcraft_format::pdf_content(&bytes).is_none());
    assert_eq!(
        vectorcraft_format::load_file(&bytes).unwrap().info.version,
        vectorcraft_format::VERSION,
        "unrelated format options are not carried over"
    );
    let r = s.execute("file.saveAsTemplate", &json!({"format": "template", "pdfCompatible": true})).unwrap();
    assert!(vectorcraft_format::pdf_content(&bytes_of(&r["dataBase64"])).is_some());
}

#[test]
fn a_default_native_save_keeps_its_pdf_after_another_document_opts_out() {
    let dir = Folder::new("default-native-pdf-choice");
    for (index, options) in [json!({}), json!({"options":{"pdfCompatible":null}})].into_iter().enumerate() {
        let mut s = three_boards();
        let path = dir.file(&format!("first-{index}.vectorcraft"));
        let mut params = options.as_object().unwrap().clone();
        params.insert("path".into(), json!(path));
        s.execute("file.saveAs", &Value::Object(params)).unwrap();
        assert!(vectorcraft_format::pdf_content(&std::fs::read(&path).unwrap()).is_some());
        s.execute("file.new", &json!({"width": 10, "height": 10})).unwrap();
        s.execute("file.saveAs", &json!({"path":dir.file(&format!("other-{index}.vectorcraft")),"pdfCompatible":false})).unwrap();
        s.execute("document.activate", &json!({"index":0})).unwrap();
        s.execute("document.save", &json!({})).unwrap();
        assert!(vectorcraft_format::pdf_content(&std::fs::read(&path).unwrap()).is_some(), "the first document keeps the PDF it was saved with");
        assert_eq!(s.doc().unwrap().save_options.get("pdfCompatible"), Some(&json!(true)));
    }
}

#[test]
fn a_null_nested_pdf_option_keeps_the_saved_opt_out() {
    let dir = Folder::new("null-nested-pdf-option");
    let mut s = three_boards();
    let path = dir.file("compact.vectorcraft");
    s.execute("file.saveAs", &json!({"path": path, "pdfCompatible": false})).unwrap();
    s.execute("document.save", &json!({"options": {"pdfCompatible": null}})).unwrap();
    assert!(vectorcraft_format::pdf_content(&std::fs::read(&path).unwrap()).is_none(), "null leaves the saved choice");
    assert_eq!(s.doc().unwrap().save_options.get("pdfCompatible"), Some(&json!(false)));
}

#[test]
fn reopening_native_files_keeps_each_files_pdf_compatibility() {
    let dir = Folder::new("pdf-reopened-options");
    let mut s = three_boards();
    let pdf = dir.file("portable.vectorcraft");
    let compact = dir.file("compact.vectorcraft");
    s.execute("file.saveAs", &json!({"path": pdf, "pdfCompatible": true})).unwrap();
    s.execute("file.saveAs", &json!({"path": compact, "pdfCompatible": false})).unwrap();
    s.execute("file.new", &json!({"width": 20, "height": 20})).unwrap();
    let copy = s.execute("file.saveCopy", &json!({})).unwrap();
    assert!(vectorcraft_format::pdf_content(&bytes_of(&copy["dataBase64"])).is_none(), "new files use the last native preference");
    for (path, expected) in [(&pdf, true), (&compact, false)] {
        s.execute("document.open", &json!({"path": path})).unwrap();
        let options = s.execute("file.formatOptions", &json!({"format": "vectorcraft"})).unwrap();
        assert_eq!(options["options"]["pdfCompatible"]["value"], expected);
        s.execute("document.save", &json!({})).unwrap();
        assert_eq!(vectorcraft_format::pdf_content(&std::fs::read(path).unwrap()).is_some(), expected);
    }
}

#[test]
fn reverting_native_files_updates_their_pdf_compatibility() {
    let dir = Folder::new("pdf-revert-options");
    for saved in [false, true] {
        let mut s = three_boards();
        let path = dir.file(if saved { "with-pdf.vectorcraft" } else { "compact.vectorcraft" });
        s.execute("file.saveAs", &json!({"path": path, "pdfCompatible": saved})).unwrap();
        let replacement = s.execute("file.saveCopy", &json!({"pdfCompatible": !saved})).unwrap();
        std::fs::write(&path, bytes_of(&replacement["dataBase64"])).unwrap();
        s.execute("shape.rectangle", &json!({"x": 70, "y": 20, "width": 8, "height": 8})).unwrap();
        s.execute("file.revert", &json!({})).unwrap();
        let options = s.execute("file.formatOptions", &json!({"format": "vectorcraft"})).unwrap();
        assert_eq!(options["options"]["pdfCompatible"]["value"], !saved);
        s.execute("document.save", &json!({})).unwrap();
        assert_eq!(vectorcraft_format::pdf_content(&std::fs::read(&path).unwrap()).is_some(), !saved);
    }
}

/// The message of a refused command's error.
fn refusal(s: &mut Session, cmd: &str, p: &Value) -> String {
    match s.execute(cmd, p) {
        Err(EngineError::BadParams { cmd: c, msg }) if c == cmd => msg,
        other => panic!("{cmd} {p}: {:?}", other.map(|r| r["bytes"].clone())),
    }
}

#[test]
fn format_options_list_the_save_options() {
    let mut s = three_boards();
    let v = s.execute("file.formatOptions", &json!({"format": "vectorcraft"})).unwrap();
    assert_eq!((v["options"]["pdfCompatible"]["value"].as_bool(), v["options"]["range"]["value"].is_null()), (Some(true), true));
    // Saved with them, a document remembers them for Save.
    let dir = Folder::new("remember");
    let path = dir.file("r.vectorcraft");
    s.execute("document.save", &json!({"path": path, "includeLinked": true})).unwrap();
    let v = s.execute("file.formatOptions", &json!({})).unwrap();
    assert_eq!(v["options"]["includeLinked"]["value"], true);
}

/// The anchors of the first object in the native file at `path`.
fn saved_anchors(path: &str) -> Vec<vectorcraft_geom::Point> {
    let d = vectorcraft_format::load(&std::fs::read(path).unwrap()).unwrap();
    let NodeKind::Path { path, .. } = &d.layers[0].children().unwrap()[0].kind else { panic!("not a path") };
    path.subpaths.iter().flat_map(|sp| sp.anchors.iter().map(|a| a.p)).collect()
}

#[test]
fn precision_rounds_the_paths_of_that_save_only() {
    let dir = Folder::new("precision");
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 1000, "height": 1000})).unwrap();
    // 1,000 anchors at coordinates with many decimal places (a fixed pseudo-random sequence).
    let mut seed = 0x2545_f491_4f6c_dd1du64;
    let mut next = || {
        seed = seed.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1_442_695_040_888_963_407);
        (seed >> 11) as f64 / (1u64 << 53) as f64 * 1000.0
    };
    let anchors: Vec<Value> = (0..1000).map(|_| json!({"x": next(), "y": next()})).collect();
    s.execute("path.create", &json!({"anchors": anchors})).unwrap();
    let (exact, rounded) = (dir.file("exact.vectorcraft"), dir.file("rounded.vectorcraft"));
    s.execute("document.save", &json!({"path": exact, "compress": false})).unwrap();
    s.execute("document.save", &json!({"path": rounded, "compress": false, "precision": 3})).unwrap();
    let (e, r) = (std::fs::metadata(&exact).unwrap().len(), std::fs::metadata(&rounded).unwrap().len());
    assert!(r < e, "{r} bytes rounded, {e} exact");
    let (want, got) = (saved_anchors(&exact), saved_anchors(&rounded));
    assert_eq!(got.len(), 1000);
    for (w, g) in want.iter().zip(&got) {
        assert!((w.x - g.x).abs() <= 0.0005 && (w.y - g.y).abs() <= 0.0005, "{w:?} saved as {g:?}");
        assert_eq!(((g.x * 1000.0).round() / 1000.0, (g.y * 1000.0).round() / 1000.0), (g.x, g.y));
    }
    // The document keeps its exact anchors, and Save doesn't remember the precision.
    let v = s.execute("file.formatOptions", &json!({})).unwrap();
    assert!(v["options"]["precision"]["value"].is_null(), "{}", v["options"]["precision"]);
    s.execute("document.save", &json!({})).unwrap();
    assert_eq!(saved_anchors(&rounded), want);
    // A precision that isn't a whole number from 0 to 9 is refused, by the command that was given it.
    for (wrong, got) in [(json!(10), "10"), (json!(2.5), "2.5"), (json!(-1), "-1"), (json!("4"), "\"4\""), (json!(true), "true")] {
        for cmd in ["file.saveCopy", "document.save"] {
            let e = s.execute(cmd, &json!({"precision": wrong})).unwrap_err().to_string();
            assert!(e.contains(&format!("`{cmd}`")) && !e.contains("document.export"), "{cmd} {wrong}: {e}");
            assert!(e.contains(&format!("`precision` must be a whole number between 0 and 9 (got {got})")), "{cmd} {wrong}: {e}");
        }
    }
    // A type error in another native option names that command too.
    let e = s.execute("document.save", &json!({"version": "4"})).unwrap_err().to_string();
    assert!(e.contains("`document.save`") && !e.contains("document.export"), "{e}");
}

#[test]
fn precision_is_not_remembered_for_new_documents() {
    let dir = Folder::new("precision-preferences");
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
    s.execute("document.save", &json!({"path": dir.file("rounded.vectorcraft"), "precision": 3})).unwrap();
    assert!(!s.prefs.native_save_options.contains_key("precision"));

    s.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
    let options = s.execute("file.formatOptions", &json!({"format": "vectorcraft"})).unwrap();
    assert!(options["options"]["precision"]["value"].is_null(), "{}", options["options"]["precision"]);
}


/// The anchors of every path in `n` and the art it holds.
fn all_anchors(n: &vectorcraft_doc::Node, out: &mut Vec<vectorcraft_geom::Point>) {
    if let NodeKind::Path { path, .. } = &n.kind {
        out.extend(path.subpaths.iter().flat_map(|sp| sp.anchors.iter().map(|a| a.p)));
    }
    for c in n.children().into_iter().flatten() {
        all_anchors(c, out);
    }
}

#[test]
fn the_preview_and_pdf_of_a_save_at_a_precision_show_the_rounded_art() {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
    let anchors = json!([{"x": 10.4, "y": 20.3}, {"x": 80.6, "y": 30.7}, {"x": 45.35, "y": 85.65}]);
    s.execute("path.create", &json!({"anchors": anchors, "closed": true})).unwrap();
    let file = bytes_of(&s.execute("file.saveCopy", &json!({"precision": 0, "pdfCompatible": true, "preview": true})).unwrap()["dataBase64"]);
    // The embedded PDF, opened, has the path on whole points.
    let pdf = vectorcraft_format::pdf_content(&file).unwrap();
    let mut t = Session::new();
    t.execute("document.open", &json!({"name": "x.pdf", "dataBase64": vectorcraft_format::base64_encode(&pdf)})).unwrap();
    let mut got = vec![];
    for l in &t.doc().unwrap().doc.layers {
        all_anchors(l, &mut got);
    }
    for (x, y) in [(10.0, 20.0), (81.0, 31.0), (45.0, 86.0)] {
        assert!(got.iter().any(|p| (p.x - x).abs() < 1e-3 && (p.y - y).abs() < 1e-3), "({x}, {y}) not in {got:?}");
    }
    assert!(got.iter().all(|p| (p.x - p.x.round()).abs() < 1e-3 && (p.y - p.y.round()).abs() < 1e-3), "{got:?}");
    // The preview is the one the saved document draws, not the exact art's.
    let preview = vectorcraft_format::preview(&file);
    assert!(preview.is_some());
    assert_eq!(preview, crate::cmd::fileio::preview_png(&vectorcraft_format::load(&file).unwrap()).unwrap());
    assert_ne!(preview, crate::cmd::fileio::preview_png(&s.doc().unwrap().doc).unwrap());
}
