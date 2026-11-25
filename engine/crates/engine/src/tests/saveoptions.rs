//! Native `.etc` save options: each artboard to a separate file (a master file too), Include
//! Linked Files, Embed ICC Profiles and path precision.

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
    etcfile::base64::decode(v.as_str().expect("base64")).unwrap()
}

#[test]
fn three_artboards_give_four_files() {
    let dir = Folder::new("separate");
    let mut s = three_boards();
    let path = dir.file("Poster.etc");
    let r = s.execute("document.save", &json!({"path": path, "separateArtboards": true})).unwrap();
    let files: Vec<&str> = r["files"].as_array().unwrap().iter().map(|f| f.as_str().unwrap()).collect();
    assert_eq!(files, [path.clone(), dir.file("Poster-A.etc"), dir.file("Poster-B.etc"), dir.file("Poster-C.etc")]);
    // The master file has everything; each artboard's file that artboard and its art.
    let master = etcfile::load(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!((master.artboards.len(), objects(&master)), (3, 3));
    for (f, name, x) in [(files[1], "A", 0.0), (files[2], "B", 200.0), (files[3], "C", 400.0)] {
        let d = etcfile::load(&std::fs::read(f).unwrap()).unwrap();
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
    assert_eq!(names, ["Poster copy.etc", "Poster copy-A.etc", "Poster copy-B.etc", "Poster copy-C.etc"]);
    assert!(etcfile::sniff(&bytes_of(&r["files"][3]["dataBase64"])));
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
        serde_json::from_slice::<Value>(&etcfile::project_json(&text).unwrap()).unwrap()["images"].clone()
    };
    // By default only the preview.
    let plain = images(&s.execute("file.saveCopy", &json!({})).unwrap());
    let (_, image) = plain.as_object().unwrap().iter().next().unwrap();
    assert_eq!(image["proxy"], true);
    // Include Linked Files: the file's own bytes.
    let r = s.execute("file.saveCopy", &json!({"includeLinked": true})).unwrap();
    let full = images(&r);
    let (_, image) = full.as_object().unwrap().iter().next().unwrap();
    assert!(image.get("proxy").is_none());
    assert_eq!(etcfile::base64::decode(image["data"].as_str().unwrap()).unwrap(), bytes);
    // With the file gone the document still shows the full pixels.
    std::fs::remove_file(&pic).unwrap();
    let r = s.execute("document.open", &json!({"name": "copy.etc", "dataBase64": r["dataBase64"]})).unwrap();
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
    let file = bytes_of(&s.execute("file.saveCopy", &json!({})).unwrap()["dataBase64"]);
    let v: Value = serde_json::from_slice(&file).unwrap();
    assert_eq!(etcfile::base64::decode(v["profiles"][&name]["data"].as_str().unwrap()).unwrap(), icc.to_vec());
    // Not embedded when asked not to; built-in profiles never are (they are everywhere).
    let none: Value =
        serde_json::from_slice(&bytes_of(&s.execute("file.saveCopy", &json!({"embedProfiles": false})).unwrap()["dataBase64"]))
            .unwrap();
    assert!(none.get("profiles").is_none());
    // Elsewhere (a profile name this machine lacks): opening installs it and the document keeps it.
    let elsewhere = format!("Studio RGB elsewhere {}", std::process::id());
    let moved = String::from_utf8(file).unwrap().replace(&name, &elsewhere);
    assert!(cms::profile(&elsewhere).is_none());
    let r =
        s.execute("document.open", &json!({"name": "moved.etc", "dataBase64": etcfile::base64::encode(moved.as_bytes())})).unwrap();
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
            &json!({"name": "b.etc", "dataBase64": etcfile::base64::encode(&serde_json::to_vec(&v).unwrap())}),
        )
        .unwrap();
    assert!(r["warnings"][0].as_str().unwrap().contains(&broken), "{r}");
}



#[test]
fn native_saves_without_artboards_keep_the_editable_document() {
    let mut s = three_boards();
    std::sync::Arc::make_mut(&mut s.doc_mut().unwrap().doc).artboards.clear();
    let result = s.execute("file.saveCopy", &json!({})).unwrap();
    let file = bytes_of(&result["dataBase64"]);
    let restored = etcfile::load(&file).unwrap();
    assert!(restored.artboards.is_empty());
    assert_eq!(restored.node_count(), s.doc().unwrap().doc.node_count());
    let project: Value = serde_json::from_slice(&etcfile::project_json(&file).unwrap()).unwrap();
    assert_eq!(project["canvas"]["mode"], "infinite", "without artboards the canvas is infinite");
}

#[test]
fn save_option_errors_name_the_save_command() {
    let mut s = three_boards();
    for (cmd, p) in [
        ("file.saveCopy", json!({"format": "pdf", "range": "5"})),
        ("file.saveCopy", json!({"format": "svg", "artboard": 5})),
        ("file.saveAs", json!({"format": "svg", "range": "4"})),
        ("document.save", json!({"format": "pdf", "artboards": [3]})),
        ("file.saveCopy", json!({"format": "ai", "compress": "yes"})),
        ("file.saveCopy", json!({"format": "ai", "artboard": "B"})),
        ("file.saveCopy", json!({"format": "ai", "aiVersion": 7, "artboard": "B"})),
        ("file.saveCopy", json!({"format": "ait", "compress": "yes"})),
    ] {
        let e = s.execute(cmd, &p).unwrap_err().to_string();
        assert!(e.contains(&format!("`{cmd}`")) && !e.contains("document.export"), "{p}: {e}");
    }
    // A native file written to a path.
    let dir = Folder::new("save-errors");
    for (cmd, name) in [("document.save", "a.etc"), ("file.saveAs", "b.etc"), ("file.saveCopy", "c.etc")] {
        let p = json!({"path": dir.file(name), "compress": "yes"});
        let e = s.execute(cmd, &p).unwrap_err().to_string();
        assert!(e.contains(&format!("`{cmd}`")) && !e.contains("document.export"), "{p}: {e}");
    }
}







#[test]
fn a_remembered_artboard_the_document_no_longer_has_is_dropped() {
    let dir = Folder::new("remembered-artboards");
    for (name, p) in [
        ("legacy.ai", json!({"aiVersion": "7", "range": "3"})),
        ("separate.ai", json!({"separateArtboards": true, "range": "3"})),
        ("separate.etc", json!({"separateArtboards": true, "range": "1, 3"})),
        ("doc.pdf", json!({"range": "3"})),
        ("doc.svg", json!({"artboard": 2})),
    ] {
        let mut s = three_boards();
        let mut first = p.clone();
        first["path"] = json!(dir.file(name));
        s.execute("file.saveAs", &first).unwrap_or_else(|e| panic!("{name}: {e}"));
        s.execute("artboard.delete", &json!({"index": 2})).unwrap();
        let v = s.execute("file.formatOptions", &json!({})).unwrap();
        for k in ["artboard", "range"].iter().filter(|k| p.get(**k).is_some()) {
            assert_ne!(v["options"][*k]["value"], p[*k], "{name}: Save no longer uses {k}");
        }
        s.execute("document.save", &json!({})).unwrap_or_else(|e| panic!("{name}: {e}"));
        // Given to the save, it is still refused.
        let e = s.execute("document.save", &p).unwrap_err().to_string();
        assert!(e.contains("from 1 to 2") || e.contains("no artboard 2"), "{name}: {e}");
    }
    // A remembered artboard that is still there is kept.
    let mut s = three_boards();
    s.execute("file.saveAs", &json!({"path": dir.file("kept.ai"), "aiVersion": "7", "range": "2"})).unwrap();
    s.execute("artboard.delete", &json!({"index": 2})).unwrap();
    let v = s.execute("file.formatOptions", &json!({})).unwrap();
    assert_eq!(v["options"]["range"]["value"], json!("2"));
    let r = s.execute("document.save", &json!({})).unwrap();
    assert!(r["path"].as_str().is_some_and(|p| p.ends_with("kept.ai")), "{r}");
}

#[test]
fn a_remembered_artboard_pick_follows_the_artboards_it_named() {
    let dir = Folder::new("remembered-pick");
    // Save each artboard of A, B and C to a file with `range`, change the artboards, then save
    // without options: the range Save now uses and the artboards it writes, by file name.
    let save = |range: &str, change: &[(&str, Value)]| -> (Value, Vec<String>) {
        let mut s = three_boards();
        let path = dir.file("x.ai");
        s.execute("file.saveAs", &json!({"path": path, "separateArtboards": true, "range": range})).unwrap();
        for (cmd, p) in change {
            s.execute(cmd, p).unwrap();
        }
        let range = s.execute("file.formatOptions", &json!({})).unwrap()["options"]["range"]["value"].clone();
        let r = s.execute("document.save", &json!({})).unwrap();
        let files = r["files"].as_array().unwrap().iter().skip(1).map(|f| f.as_str().unwrap().rsplit(['/', '\\']).next().unwrap().to_string());
        (range, files.collect())
    };
    let delete = |i: usize| ("artboard.delete", json!({"index": i}));
    assert_eq!(save("1, 3", &[delete(2)]), (json!("1"), vec!["x-A.ai".to_string()]));
    assert_eq!(save("1, 3", &[delete(0)]), (json!("2"), vec!["x-C.ai".to_string()]));
    assert_eq!(save("1-3", &[delete(1)]), (json!("1-2"), vec!["x-A.ai".to_string(), "x-C.ai".to_string()]));
    let (range, mut files) = save("1, 3", &[("artboard.reorder", json!({"index": 2, "to": 0}))]);
    files.sort();
    assert_eq!((range, files), (json!("2, 1"), vec!["x-A.ai".to_string(), "x-C.ai".to_string()]));
    // None of them left: every artboard, as when none is named.
    assert_eq!(save("1, 3", &[delete(2), delete(0)]), (Value::Null, vec!["x-B.ai".to_string()]));
    assert_eq!(save("all", &[delete(0)]), (json!("all"), vec!["x-B.ai".to_string(), "x-C.ai".to_string()]));
    // An artboard made after one was deleted is not taken for it.
    let new = ("artboard.new", json!({"x": 600, "y": 0, "width": 100, "height": 100, "name": "D"}));
    let all = ["x-A.ai", "x-B.ai", "x-D.ai"].map(String::from).to_vec();
    assert_eq!(save("3", &[delete(2), new.clone()]), (Value::Null, all));
    assert_eq!(save("1, 3", &[delete(2), new.clone()]), (json!("1"), vec!["x-A.ai".to_string()]));
    // A range open at its end keeps to the artboards it named.
    assert_eq!(save("2-", std::slice::from_ref(&new)), (json!("2-3"), vec!["x-B.ai".to_string(), "x-C.ai".to_string()]));
    // Nor is one made after undoing the artboard (and the three rectangles made after it).
    let undo = ("edit.undo", json!({}));
    let undone = [undo.clone(), undo.clone(), undo.clone(), undo, new];
    assert_eq!(save("1, 3", &undone), (json!("1"), vec!["x-A.ai".to_string()]));

    // The other saves that remember a pick: a PDF's range, an SVG's artboard, a legacy .ai's
    // range and a native file's separate files.
    for (name, p, gone, after) in [
        ("doc.pdf", json!({"range": "1, 3"}), 0, json!({"range": "2"})),
        // `range` wins over `artboard`, which a save then does not use.
        ("unused.pdf", json!({"artboard": 0, "range": "3"}), 0, json!({"range": "2"})),
        ("doc.svg", json!({"artboard": 2}), 0, json!({"artboard": 1})),
        ("legacy.ai", json!({"aiVersion": "7", "range": "1, 3"}), 2, json!({"range": "1"})),
        ("doc.etc", json!({"separateArtboards": true, "range": "1, 3"}), 0, json!({"range": "2"})),
    ] {
        let mut s = three_boards();
        let mut first = p.clone();
        first["path"] = json!(dir.file(name));
        s.execute("file.saveAs", &first).unwrap();
        s.execute("artboard.delete", &json!({"index": gone})).unwrap();
        let v = s.execute("file.formatOptions", &json!({})).unwrap();
        for (k, value) in after.as_object().unwrap() {
            assert_eq!(&v["options"][k]["value"], value, "{name}: {k}");
        }
        s.execute("document.save", &json!({})).unwrap_or_else(|e| panic!("{name}: {e}"));
        if name.ends_with(".pdf") {
            let bytes = std::fs::read(dir.file(name)).unwrap();
            let info = s.execute("document.pdfInfo", &json!({"dataBase64": etcfile::base64::encode(&bytes)})).unwrap();
            assert_eq!(info["pages"], 1, "only the artboard that was 3");
        }
    }

    // File > Revert: a PDF holds only the pages it was saved with, so its pick is left out; a
    // native file keeps every artboard, and the pick stays.
    let mut s = three_boards();
    s.execute("file.saveAs", &json!({"path": dir.file("revert.pdf"), "range": "1, 3"})).unwrap();
    s.execute("shape.rectangle", &json!({"x": 10, "y": 50, "width": 30, "height": 30})).unwrap();
    s.execute("file.revert", &json!({})).unwrap();
    assert_eq!(s.doc().unwrap().doc.artboards.len(), 2);
    assert_eq!(s.execute("file.formatOptions", &json!({})).unwrap()["options"]["range"]["value"], Value::Null);
    // A PDF without the editable document reads back with new ids in page order: after a reorder,
    // the pick is left out, not taken for another reorder.
    let mut s = three_boards();
    s.execute("artboard.reorder", &json!({"index": 2, "to": 0})).unwrap();
    s.execute("file.saveAs", &json!({"path": dir.file("reordered.pdf"), "range": "1-3", "preserveEditing": false})).unwrap();
    s.execute("shape.rectangle", &json!({"x": 10, "y": 50, "width": 30, "height": 30})).unwrap();
    s.execute("file.revert", &json!({})).unwrap();
    assert_eq!(s.doc().unwrap().doc.artboards.iter().map(|a| a.id).collect::<Vec<_>>(), [1, 2, 3]);
    assert_eq!(s.execute("file.formatOptions", &json!({})).unwrap()["options"]["range"]["value"], Value::Null);
    let mut s = three_boards();
    s.execute("file.saveAs", &json!({"path": dir.file("revert.etc"), "separateArtboards": true, "range": "1, 3"})).unwrap();
    s.execute("artboard.delete", &json!({"index": 0})).unwrap();
    s.execute("file.revert", &json!({})).unwrap();
    assert_eq!(s.execute("file.formatOptions", &json!({})).unwrap()["options"]["range"]["value"], json!("1, 3"));
}

#[test]
fn format_options_list_the_save_options() {
    let mut s = three_boards();
    let v = s.execute("file.formatOptions", &json!({"format": "etc"})).unwrap();
    assert_eq!((v["options"]["pdfCompatible"]["value"].as_bool(), v["options"]["range"]["value"].is_null()), (Some(true), true));
    // Saved with them, a document remembers them for Save.
    let dir = Folder::new("remember");
    let path = dir.file("r.etc");
    s.execute("document.save", &json!({"path": path, "includeLinked": true})).unwrap();
    let v = s.execute("file.formatOptions", &json!({})).unwrap();
    assert_eq!(v["options"]["includeLinked"]["value"], true);
}

/// The anchors of the first object in the native file at `path`.
fn saved_anchors(path: &str) -> Vec<vectorcraft_geom::Point> {
    let d = etcfile::load(&std::fs::read(path).unwrap()).unwrap();
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
    let (exact, rounded) = (dir.file("exact.etc"), dir.file("rounded.etc"));
    s.execute("document.save", &json!({"path": exact})).unwrap();
    s.execute("document.save", &json!({"path": rounded, "precision": 3})).unwrap();
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
    s.execute("document.save", &json!({"path": dir.file("rounded.etc"), "precision": 3})).unwrap();
    assert!(!s.prefs.native_save_options.contains_key("precision"));

    s.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
    let options = s.execute("file.formatOptions", &json!({"format": "etc"})).unwrap();
    assert!(options["options"]["precision"]["value"].is_null(), "{}", options["options"]["precision"]);
}



#[test]
fn the_previews_of_a_save_at_a_precision_show_the_rounded_art() {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 100, "height": 100})).unwrap();
    let anchors = json!([{"x": 10.4, "y": 20.3}, {"x": 80.6, "y": 30.7}, {"x": 45.35, "y": 85.65}]);
    s.execute("path.create", &json!({"anchors": anchors, "closed": true})).unwrap();
    let file = bytes_of(&s.execute("file.saveCopy", &json!({"precision": 0})).unwrap()["dataBase64"]);
    // The previews are the ones the saved document draws, not the exact art's.
    let preview = etcfile::thumbnail(&file);
    assert!(preview.is_some());
    assert_eq!(preview, crate::cmd::fileio::preview_png(&etcfile::load(&file).unwrap()).unwrap());
    assert_ne!(preview, crate::cmd::fileio::preview_png(&s.doc().unwrap().doc).unwrap());
}
