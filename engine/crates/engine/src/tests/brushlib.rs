//! Brush libraries: the built-in libraries, saving a document's brushes as a library (User
//! Defined, or a file), reopening it and adding its brushes to another document (the patterns and
//! symbols their art uses along, one undo step, deduped, renamed on a clash, applied), other
//! documents' brushes, and files that can't be read.

use serde_json::{Value, json};
use vectorcraft_brush::BrushKind;
use vectorcraft_doc::{NodeId, NodeKind};

use crate::*;

fn session() -> Session {
    let mut s = Session::new();
    s.execute("file.new", &json!({"width": 200, "height": 200})).unwrap();
    s
}

fn run(s: &mut Session, id: &str, p: Value) -> Value {
    s.execute(id, &p).unwrap_or_else(|e| panic!("{id} {p}: {e}"))
}

fn doc(s: &Session) -> &vectorcraft_doc::Document {
    &s.doc().unwrap().doc
}

fn undo_len(s: &Session) -> usize {
    s.doc().unwrap().history.undo.len()
}

fn brushes(s: &Session) -> Vec<String> {
    vectorcraft_brush::library(doc(s)).into_iter().map(|b| b.name).collect()
}

fn brush(s: &Session, name: &str) -> vectorcraft_brush::Brush {
    vectorcraft_brush::find(doc(s), name).unwrap_or_else(|| panic!("no brush {name}"))
}

fn strs(v: &Value) -> Vec<&str> {
    v.as_array().unwrap().iter().map(|n| n.as_str().unwrap()).collect()
}

/// The patterns a brush's art fills with.
fn art_patterns(b: &vectorcraft_brush::Brush) -> Vec<String> {
    let mut out = vec![];
    for a in b.art() {
        a.walk(&mut |n| {
            for it in &n.appearance.items {
                if let vectorcraft_color::Paint::Pattern { pattern, .. } = it.paint() {
                    out.push(pattern.clone());
                }
            }
        });
    }
    out
}

/// A folder of its own under the temp folder, emptied.
fn folder(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("vc-brushlib-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// Pattern `name` (a 4 pt dot of `color` in a 10 pt tile) with its swatch, as one step.
fn add_pattern(s: &mut Session, name: &str, color: &str) {
    let color = vectorcraft_color::Color::from_hex(color).unwrap();
    s.edit("Pattern", |d, _| {
        let dot = vectorcraft_doc::Node::path(
            d.alloc_id(),
            vectorcraft_geom::shapes::ellipse(vectorcraft_geom::Rect::new(0.0, 0.0, 4.0, 4.0)),
            vectorcraft_doc::Appearance::basic(vectorcraft_color::Paint::solid(color), vectorcraft_color::Paint::None, 0.0),
        );
        let def = vectorcraft_doc::PatternDef {
            tile: vectorcraft_geom::Rect::new(0.0, 0.0, 10.0, 10.0),
            ..vectorcraft_doc::PatternDef::new(name, vec![std::sync::Arc::new(dot)])
        };
        d.swatches.push(vectorcraft_color::Swatch {
            name: name.into(),
            paint: vectorcraft_color::Paint::Pattern { pattern: name.into(), xf: Default::default() },
            global: false,
            spot: false,
        });
        d.patterns.push(def);
        Ok(())
    })
    .unwrap();
}

/// A document with brushes of its own: "My Nib" (calligraphic) and "Dotted Band" (an art brush
/// whose art is filled with pattern "Dots").
fn kit() -> Session {
    let mut s = session();
    run(&mut s, "brush.new", json!({"type": "calligraphic", "name": "My Nib", "params": {"angle": 20, "roundness": 40, "size": 7}}));
    add_pattern(&mut s, "Dots", "#ff0000");
    let band = run(&mut s, "shape.rectangle", json!({"x": 0, "y": 0, "width": 100, "height": 8}))["id"].as_u64().unwrap();
    run(&mut s, "paint.setFill", json!({"swatch": "Dots"}));
    run(&mut s, "brush.new", json!({"type": "art", "name": "Dotted Band", "ids": [band]}));
    s
}

#[test]
fn the_builtin_libraries_are_original_brushes_generated_in_code() {
    let mut s = Session::new();
    let list = run(&mut s, "brush.libraries", json!({}));
    let libs = list["libraries"].as_array().unwrap();
    let ids: Vec<&str> = libs.iter().map(|l| l["id"].as_str().unwrap()).collect();
    assert_eq!(
        ids,
        [
            "art-ribbons",
            "ink-and-wash",
            "calligraphic-nibs",
            "pressure-sensitive",
            "bristle-brushes",
            "paint",
            "pattern-borders",
            "decorative",
            "ornamental",
            "scatter-shapes",
            "stipple",
            "game-trails",
            "energy-beams",
            "dry-media",
            "tech-circuit",
            "engraving",
            "arrows",
            "image-brush",
            "artistic",
            "vector-packs"
        ]
    );
    assert!(libs.iter().all(|l| l["category"] == "builtIn"));
    // None of them is a default brush.
    let defaults: Vec<&str> = vectorcraft_brush::defaults().iter().map(|b| b.name.as_str()).collect();
    for id in ids {
        let (_, lib) = cmd::brushlib::library(&s, id).unwrap();
        assert!(lib.brushes.iter().all(|b| !defaults.contains(&b.name.as_str())), "{id}");
    }
    assert!(s.execute("brush.library", &json!({"library": "nope"})).is_err());
}

#[test]
fn the_builtin_libraries_list_their_brush_names_and_counts() {
    let mut s = Session::new();
    let list = run(&mut s, "brush.libraries", json!({}));
    let libs = list["libraries"].as_array().unwrap();
    // Looked up by id, so a library added to the list does not shift these checks.
    let entry = |id: &str| libs.iter().find(|l| l["id"] == id).unwrap();
    let lib = run(&mut s, "brush.library", json!({"library": "art ribbons"}));
    assert_eq!(
        lib["brushes"].as_array().unwrap().iter().map(|b| b["name"].as_str().unwrap()).collect::<Vec<_>>(),
        ["Notched Ribbon", "Double Line", "Wave", "Teardrop", "Striped Ribbon"]
    );
    assert_eq!(entry("art-ribbons")["count"].as_u64(), Some(5));
    assert_eq!((entry("decorative")["name"].as_str(), entry("decorative")["count"].as_u64()), (Some("Decorative"), Some(4)));
    let trails = run(&mut s, "brush.library", json!({"library": "game-trails"}));
    assert_eq!(
        trails["brushes"].as_array().unwrap().iter().map(|b| b["name"].as_str().unwrap()).collect::<Vec<_>>(),
        ["Speed Line", "Dash Trail", "Burst Ray", "Zigzag"]
    );
    assert_eq!(entry("game-trails")["count"].as_u64(), Some(4));
    let beams = run(&mut s, "brush.library", json!({"library": "energy-beams"}));
    assert_eq!(
        beams["brushes"].as_array().unwrap().iter().map(|b| b["name"].as_str().unwrap()).collect::<Vec<_>>(),
        ["Laser Bolt", "Plasma Arc", "Charge Wave", "Spark Trail"]
    );
    assert_eq!(entry("energy-beams")["count"].as_u64(), Some(4));
    let dry = run(&mut s, "brush.library", json!({"library": "Dry Media"}));
    assert_eq!(
        dry["brushes"].as_array().unwrap().iter().map(|b| (b["name"].as_str().unwrap(), b["type"].as_str().unwrap())).collect::<Vec<_>>(),
        [("Chalk Grain", "art"), ("Graphite Pencil", "art"), ("Wax Crayon", "art"), ("Charcoal Dust", "art")]
    );
    assert_eq!((entry("dry-media")["name"].as_str(), entry("dry-media")["count"].as_u64()), (Some("Dry Media"), Some(4)));
    let arrows = run(&mut s, "brush.library", json!({"library": "Arrows"}));
    assert_eq!(
        arrows["brushes"].as_array().unwrap().iter().map(|b| (b["name"].as_str().unwrap(), b["type"].as_str().unwrap())).collect::<Vec<_>>(),
        [("Solid Arrow", "art"), ("Open Arrow", "art"), ("Double Arrow", "art"), ("Chevron Arrow", "art"), ("Dashed Arrow", "art")]
    );
    assert_eq!((entry("arrows")["name"].as_str(), entry("arrows")["count"].as_u64()), (Some("Arrows"), Some(5)));
    assert_eq!((entry("stipple")["name"].as_str(), entry("stipple")["count"].as_u64()), (Some("Stipple"), Some(4)));
    run(&mut s, "file.new", json!({"width": 200, "height": 200}));
    let id = run(&mut s, "shape.rectangle", json!({"x": 10, "y": 20, "width": 40, "height": 8}))["id"].as_u64().unwrap();
    let added = run(&mut s, "brush.addFromLibrary", json!({"library": "energy-beams", "name": "Laser Bolt", "apply": true, "ids": [id]}));
    assert_eq!(strs(&added["added"]), ["Laser Bolt"]);
}

#[test]
fn every_builtin_library_brush_has_its_kind_and_draws() {
    let s = Session::new();
    // One kind of brush per library (Ink and Wash: art brushes and a calligraphic one), every
    // brush drawing something.
    for (id, kind) in [
        ("art-ribbons", "art"),
        ("ink-and-wash", "art"),
        ("calligraphic-nibs", "calligraphic"),
        ("pressure-sensitive", "art"),
        ("bristle-brushes", "bristle"),
        ("paint", "art"),
        ("pattern-borders", "pattern"),
        ("decorative", "pattern"),
        ("ornamental", "art"),
        ("scatter-shapes", "scatter"),
        ("stipple", "scatter"),
        ("game-trails", "art"),
        ("energy-beams", "art"),
        ("dry-media", "art"),
        ("arrows", "art"),
        ("image-brush", "scatter"),
        ("artistic", "art"),
        ("vector-packs", "scatter"),
    ] {
        let (_, lib) = cmd::brushlib::library(&s, id).unwrap();
        assert!(lib.len() >= 4, "{id}");
        for b in &lib.brushes {
            let kind = if ["Brush Pen", "Round Flow", "Chisel Flow"].contains(&b.name.as_str()) { "calligraphic" } else { kind };
            assert_eq!(b.kind.type_id(), kind, "{}", b.name);
            let mut line = vectorcraft_geom::BezPath::new();
            line.move_to((0.0, 0.0));
            line.curve_to((60.0, -40.0), (140.0, 40.0), (200.0, 0.0));
            let stroke = vectorcraft_doc::StrokeLayer::new(vectorcraft_color::Paint::solid(vectorcraft_color::Color::BLACK), 1.0);
            assert!(!vectorcraft_brush::stroke_pieces(&lib.doc, b, &line, &stroke).is_empty(), "{} draws nothing", b.name);
        }
    }
}

#[test]
fn builtin_library_brushes_bring_the_resources_their_art_uses() {
    let s = Session::new();
    // The resources their art uses come with them: Striped Ribbon's pattern, Sparkles' symbol.
    let (_, ribbons) = cmd::brushlib::library(&s, "art-ribbons").unwrap();
    assert_eq!(art_patterns(ribbons.brush("Striped Ribbon").unwrap()), ["Pinstripes"]);
    assert!(ribbons.doc.pattern("Pinstripes").is_some());
    let (_, shapes) = cmd::brushlib::library(&s, "scatter-shapes").unwrap();
    let BrushKind::Scatter(sparkles) = &shapes.brush("Sparkles").unwrap().kind else { panic!("not a scatter brush") };
    assert!(matches!(&sparkles.art.kind, NodeKind::SymbolInstance { symbol, .. } if symbol == "Sparkle"));
    assert!(shapes.doc.symbols.iter().any(|x| x.name == "Sparkle"));
}

#[test]
fn artistic_library_strokes_are_editable_and_reopen_from_a_saved_library() {
    let mut s = session();
    let (_, lib) = cmd::brushlib::library(&s, "artistic").unwrap();
    let (_, alias) = cmd::brushlib::library(&s, "Artistic").unwrap();
    assert_eq!(alias.brushes, lib.brushes);
    assert_eq!(run(&mut s, "brush.library", json!({"library": "Artistic"}))["id"], "artistic");
    let names = ["Orbit Lines", "Petal Garland", "Lattice Braid", "Facet Sweep"];
    assert_eq!(lib.brushes.iter().map(|b| b.name.as_str()).collect::<Vec<_>>(), names);
    let line = run(&mut s, "shape.line", json!({"x1": 10, "y1": 50, "x2": 190, "y2": 50}))["id"].as_u64().unwrap();
    let red = vectorcraft_doc::StrokeLayer::new(vectorcraft_color::Paint::solid(vectorcraft_color::Color::rgb(1.0, 0.0, 0.0)), 1.0);
    for b in &lib.brushes {
        let BrushKind::Art(a) = &b.kind else { panic!("{} is not an art brush", b.name) };
        assert!(art_paths(&a.art).len() >= 2, "{} has no editable vector art", b.name);
        for path in [line_to(180.0), line_to(1.0)] {
            assert!(!vectorcraft_brush::stroke_pieces(&lib.doc, b, &path, &black_stroke()).is_empty(), "{} draws nothing", b.name);
        }
        // The art is black, so with Tints every path takes the stroke color itself.
        let mut fills = vec![];
        for piece in vectorcraft_brush::stroke_pieces(&lib.doc, b, &line_to(180.0), &red) {
            piece.walk(&mut |n| {
                if n.path_data().is_some() {
                    fills.push(n.appearance.fill_paint().color().map(|c| c.to_rgb()));
                }
            });
        }
        let is_red = |c: &Option<[f32; 3]>| c.is_some_and(|[r, g, b]| (r - 1.0).abs() < 1e-5 && g.abs() < 1e-5 && b.abs() < 1e-5);
        assert!(!fills.is_empty() && fills.iter().all(is_red), "{} does not take the stroke color: {fills:?}", b.name);
        let before = undo_len(&s);
        let added = run(&mut s, "brush.addFromLibrary", json!({"library": "artistic", "name": b.name, "apply": true, "ids": [line]}));
        assert_eq!(strs(&added["added"]), [b.name.as_str()]);
        assert_eq!(added["applied"], json!([line]));
        assert_eq!(undo_len(&s), before + 1);
    }
    let saved = run(&mut s, "brush.saveLibrary", json!({"names": names}));
    let bytes = etcfile::base64::decode(saved["dataBase64"].as_str().unwrap()).unwrap();
    let reopened = etcfile::load(&bytes).unwrap();
    assert_eq!(vectorcraft_brush::library(&reopened), lib.brushes);
}

#[test]
fn vector_packs_marks_are_editable_and_survive_a_saved_library() {
    let mut s = session();
    let (_, lib) = cmd::brushlib::library(&s, "vector-packs").unwrap();
    let names = ["Compass Rose", "Botanical Sprig", "Facet Diamond", "Tide Crest"];
    assert_eq!(lib.brushes.iter().map(|b| b.name.as_str()).collect::<Vec<_>>(), names);
    let line = run(&mut s, "shape.line", json!({"x1": 10, "y1": 50, "x2": 190, "y2": 50}))["id"].as_u64().unwrap();
    for b in &lib.brushes {
        let BrushKind::Scatter(sc) = &b.kind else { panic!("{} is not a scatter brush", b.name) };
        assert!(art_paths(&sc.art).len() >= 2, "{} has no editable vector mark", b.name);
        let pieces = vectorcraft_brush::stroke_pieces(&lib.doc, b, &line_to(180.0), &black_stroke());
        assert!(pieces.len() >= 3, "{} does not repeat along a path", b.name);
        // The black marks take a non-black stroke color as their fill.
        let teal = vectorcraft_color::Color::rgb(0.1, 0.6, 0.5);
        let stroke = vectorcraft_doc::StrokeLayer::new(vectorcraft_color::Paint::solid(teal), 1.0);
        let mut fills = vec![];
        for piece in vectorcraft_brush::stroke_pieces(&lib.doc, b, &line_to(180.0), &stroke) {
            piece.walk(&mut |c| {
                if c.path_data().is_some() {
                    fills.push(c.appearance.fill().map(|f| f.paint.clone()));
                }
            });
        }
        assert!(!fills.is_empty(), "{} draws no paths", b.name);
        for fill in fills {
            let Some(vectorcraft_color::Paint::Solid { color, .. }) = fill else { panic!("{} draws a path without a solid fill: {fill:?}", b.name) };
            let (got, want) = (color.to_rgb(), teal.to_rgb());
            assert!(got.iter().zip(want).all(|(g, w)| (g - w).abs() < 1e-3), "{} fills {got:?}, not the stroke color {want:?}", b.name);
        }
        let before = undo_len(&s);
        let added = run(&mut s, "brush.addFromLibrary", json!({"library": "vector-packs", "name": b.name, "apply": true, "ids": [line]}));
        assert_eq!(strs(&added["added"]), [b.name.as_str()]);
        assert_eq!(added["applied"], json!([line]));
        assert_eq!(undo_len(&s), before + 1);
        assert_eq!(doc(&s).node(NodeId(line)).unwrap().appearance.stroke().unwrap().brush.as_deref(), Some(b.name.as_str()));
    }
    let saved = run(&mut s, "brush.saveLibrary", json!({"names": names}));
    let bytes = etcfile::base64::decode(saved["dataBase64"].as_str().unwrap()).unwrap();
    let reopened = etcfile::load(&bytes).unwrap();
    assert_eq!(vectorcraft_brush::library(&reopened), lib.brushes);
}

#[test]
fn image_brush_stamps_draw_and_carry_their_pixels_when_added_and_saved() {
    let mut s = session();
    let (_, lib) = cmd::brushlib::library(&s, "image-brush").unwrap();
    let names = ["Soft Bloom", "Halftone Field", "Solar Grain", "Color Flecks"];
    assert_eq!(lib.brushes.iter().map(|b| b.name.as_str()).collect::<Vec<_>>(), names);
    assert_eq!(lib.doc.images.len(), 4);
    let path = run(&mut s, "shape.line", json!({"x1": 10, "y1": 40, "x2": 190, "y2": 40}))["id"].as_u64().unwrap();
    for b in &lib.brushes {
        let BrushKind::Scatter(sc) = &b.kind else { panic!("{} is not a scatter brush", b.name) };
        let NodeKind::Image(image) = &sc.art.kind else { panic!("{} has no image stamp", b.name) };
        assert!(lib.doc.images.get(&image.key).is_some_and(|blob| blob.bytes.starts_with(b"\x89PNG\r\n\x1a\n")));
        let pieces = vectorcraft_brush::stroke_pieces(&lib.doc, b, &line_to(180.0), &black_stroke());
        assert!(pieces.len() >= 3 && pieces.iter().all(|p| matches!(p.kind, NodeKind::Image(_))), "{}", b.name);
        // A short accent shows at least one stamp at a 1 pt stroke.
        let short = vectorcraft_brush::stroke_pieces(&lib.doc, b, &line_to(10.0), &black_stroke());
        assert!(!short.is_empty() && short.iter().all(|p| matches!(p.kind, NodeKind::Image(_))), "{} on a 10 pt line", b.name);
        let before = undo_len(&s);
        let added = run(&mut s, "brush.addFromLibrary", json!({"library": "image-brush", "name": b.name, "apply": true, "ids": [path]}));
        assert_eq!(strs(&added["added"]), [b.name.as_str()]);
        assert_eq!(added["applied"], json!([path]));
        assert_eq!(undo_len(&s), before + 1);
        assert!(doc(&s).images.contains_key(&image.key), "{} pixels were not copied", b.name);
    }
    let saved = run(&mut s, "brush.saveLibrary", json!({"names": names}));
    let bytes = etcfile::base64::decode(saved["dataBase64"].as_str().unwrap()).unwrap();
    let reopened = etcfile::load(&bytes).unwrap();
    let mut saved_brushes = vectorcraft_brush::library(&reopened);
    for brush in &mut saved_brushes {
        if let BrushKind::Scatter(sc) = &mut brush.kind {
            sc.art.id = NodeId(0); // Loaded image art is given a fresh document node ID.
        }
    }
    assert_eq!(saved_brushes, lib.brushes);
    assert_eq!(reopened.images, lib.doc.images);
}

#[test]
fn a_loaded_library_named_artistic_keeps_its_name_ahead_of_the_builtin_alias() {
    let mut file = vectorcraft_doc::Document::new(100.0, 100.0);
    vectorcraft_brush::store(&mut file, &[vectorcraft_brush::defaults()[0].clone()]);
    let b64 = etcfile::base64::encode(&etcfile::save_file(&file));
    let mut s = session();
    let loaded = run(&mut s, "brush.loadLibrary", json!({"dataBase64": b64, "name": "Artistic.etc"}));
    assert_eq!(loaded["name"], "Artistic");
    let id = loaded["library"].as_str().unwrap();
    assert_eq!(run(&mut s, "brush.library", json!({"library": "Artistic"}))["id"], id);
    assert_eq!(run(&mut s, "brush.library", json!({"library": "ARTISTIC"}))["id"], id);
    assert_eq!(run(&mut s, "brush.library", json!({"library": "artistic"}))["id"], "artistic", "exact built-in id still works");
    // A name a built-in library has selects the built-in, as for every built-in; the loaded one keeps its id.
    let full = run(&mut s, "brush.loadLibrary", json!({"dataBase64": b64, "name": "Artistic Brushes.etc"}));
    assert_eq!(full["name"], "Artistic Brushes");
    let full_id = full["library"].as_str().unwrap();
    assert_eq!(run(&mut s, "brush.library", json!({"library": "Artistic Brushes"}))["id"], "artistic");
    assert_eq!(run(&mut s, "brush.library", json!({"library": full_id}))["id"], full_id);
}

/// Every path of brush art `n`: (anchors, opacity).
fn art_paths(n: &vectorcraft_doc::Node) -> Vec<(usize, f32)> {
    let mut out = vec![];
    n.walk(&mut |c| {
        if let Some(p) = c.path_data() {
            out.push((p.anchor_count(), c.opacity));
        }
    });
    out
}

/// The Ink and Wash library, made once.
fn inks(s: &Session) -> std::sync::Arc<cmd::brushlib::BrushLibrary> {
    cmd::brushlib::library(s, "ink-and-wash").unwrap().1
}

/// A straight line from the origin to (`x`, 0).
fn line_to(x: f64) -> vectorcraft_geom::BezPath {
    let mut line = vectorcraft_geom::BezPath::new();
    line.move_to((0.0, 0.0));
    line.line_to((x, 0.0));
    line
}

fn black_stroke() -> vectorcraft_doc::StrokeLayer {
    vectorcraft_doc::StrokeLayer::new(vectorcraft_color::Paint::solid(vectorcraft_color::Color::BLACK), 1.0)
}

#[test]
fn engraving_library_draws_fine_parallel_strokes_and_survives_save() {
    let mut s = session();
    let listed = run(&mut s, "brush.libraries", json!({}));
    let entry = listed["libraries"].as_array().unwrap().iter().find(|l| l["id"] == "engraving").unwrap();
    assert_eq!((entry["name"].as_str(), entry["category"].as_str(), entry["count"].as_u64()), (Some("Engraving"), Some("builtIn"), Some(4)));
    let (_, lib) = cmd::brushlib::library(&s, "Engraving").unwrap();
    for b in &lib.brushes {
        for length in [2.0, 200.0] {
            let pieces = vectorcraft_brush::stroke_pieces(&lib.doc, b, &line_to(length), &black_stroke());
            let pts = anchors(&pieces.iter().collect::<Vec<_>>());
            assert!(!pts.is_empty() && pts.iter().all(|p| p.x.is_finite() && p.y.is_finite()), "{} at {length} pt", b.name);
        }
    }
    let path = run(&mut s, "shape.line", json!({"x1": 10, "y1": 20, "x2": 210, "y2": 20}))["id"].as_u64().unwrap();
    let added = run(&mut s, "brush.addFromLibrary", json!({"library": "engraving", "name": "Fine Hatching", "apply": true, "ids": [path]}));
    assert_eq!(strs(&added["added"]), ["Fine Hatching"]);
    assert_eq!(added["applied"], json!([path]));
    let saved = run(&mut s, "brush.saveLibrary", json!({"names": ["Fine Hatching"]}));
    let bytes = etcfile::base64::decode(saved["dataBase64"].as_str().unwrap()).unwrap();
    let reopened = etcfile::load(&bytes).unwrap();
    assert_eq!(vectorcraft_brush::find(&reopened, "Fine Hatching").unwrap(), lib.brushes[0]);
}

#[test]
fn stipple_library_draws_dot_textures_and_survives_save() {
    let mut s = session();
    let (_, lib) = cmd::brushlib::library(&s, "stipple").unwrap();
    let names: Vec<&str> = lib.brushes.iter().map(|b| b.name.as_str()).collect();
    assert_eq!(names, ["Fine Stipple", "Dense Stipple", "Soft Stipple", "Bold Stipple"]);
    let path = run(&mut s, "shape.line", json!({"x1": 10, "y1": 20, "x2": 210, "y2": 20}))["id"].as_u64().unwrap();
    for b in &lib.brushes {
        let BrushKind::Scatter(scatter) = &b.kind else { panic!("{} is not a scatter brush", b.name) };
        assert_eq!(scatter.colorization, vectorcraft_brush::Colorization::Tints);
        assert!(scatter.spacing.0 > 0.0 && scatter.spacing.1 >= scatter.spacing.0);
        // A red stroke: the black dots take it as their color, and Soft Stipple's dots stay translucent.
        let red = vectorcraft_color::Color::rgb(0.8, 0.1, 0.2);
        let stroke = vectorcraft_doc::StrokeLayer::new(vectorcraft_color::Paint::solid(red), 1.0);
        let pieces = vectorcraft_brush::stroke_pieces(&lib.doc, b, &line_to(200.0), &stroke);
        let points = anchors(&pieces.iter().collect::<Vec<_>>());
        assert!(!points.is_empty() && points.iter().all(|p| p.x.is_finite() && p.y.is_finite()), "{}", b.name);
        let mut dots = vec![];
        for piece in &pieces {
            piece.walk(&mut |c| {
                if c.path_data().is_some() {
                    let fill = c.appearance.fill().unwrap_or_else(|| panic!("a {} dot has no fill", b.name));
                    dots.push((fill.paint.color(), fill.opacity));
                }
            });
        }
        assert!(!dots.is_empty(), "{}", b.name);
        for (color, opacity) in dots {
            let rgb = color.unwrap_or_else(|| panic!("a {} dot is not a solid color", b.name)).to_rgb();
            assert!(rgb.iter().zip(red.to_rgb()).all(|(a, e)| (a - e).abs() < 1e-4), "{} dot is {rgb:?}, not the stroke color", b.name);
            if b.name == "Soft Stipple" {
                assert!(opacity > 0.0 && opacity < 1.0, "Soft Stipple dot opacity {opacity}");
            } else {
                assert_eq!(opacity, 1.0, "{}", b.name);
            }
        }
        let before = undo_len(&s);
        let result = run(&mut s, "brush.addFromLibrary", json!({"library": "stipple", "name": b.name, "apply": true, "ids": [path]}));
        assert_eq!(strs(&result["added"]), [b.name.as_str()]);
        assert_eq!(undo_len(&s), before + 1);
        assert_eq!(doc(&s).node(NodeId(path)).unwrap().appearance.stroke().unwrap().brush.as_deref(), Some(b.name.as_str()));
    }
    let saved = run(&mut s, "brush.saveLibrary", json!({"names": names}));
    let bytes = etcfile::base64::decode(saved["dataBase64"].as_str().unwrap()).unwrap();
    let reopened = etcfile::load(&bytes).unwrap();
    assert_eq!(vectorcraft_brush::library(&reopened), lib.brushes);
}

#[test]
fn arrows_library_lists_renders_and_applies_original_brushes() {
    let mut s = session();
    let listed = run(&mut s, "brush.libraries", json!({}));
    let entry = listed["libraries"].as_array().unwrap().iter().find(|l| l["id"] == "arrows").unwrap();
    assert_eq!((entry["name"].as_str(), entry["category"].as_str(), entry["count"].as_u64()), (Some("Arrows"), Some("builtIn"), Some(5)));
    let (_, lib) = cmd::brushlib::library(&s, "Arrows").unwrap();
    assert_eq!(
        lib.brushes.iter().map(|b| (b.name.as_str(), b.kind.type_id())).collect::<Vec<_>>(),
        [("Solid Arrow", "art"), ("Open Arrow", "art"), ("Double Arrow", "art"), ("Chevron Arrow", "art"), ("Dashed Arrow", "art")]
    );
    for b in &lib.brushes {
        for length in [2.0, 200.0] {
            let pieces = vectorcraft_brush::stroke_pieces(&lib.doc, b, &line_to(length), &black_stroke());
            let pts = anchors(&pieces.iter().collect::<Vec<_>>());
            assert!(!pts.is_empty() && pts.iter().all(|p| p.x.is_finite() && p.y.is_finite()), "{} at {length} pt", b.name);
        }
    }
    let path = run(&mut s, "shape.line", json!({"x1": 10, "y1": 20, "x2": 210, "y2": 20}))["id"].as_u64().unwrap();
    let before = undo_len(&s);
    let added = run(&mut s, "brush.addFromLibrary", json!({"library": "arrows", "name": "Double Arrow", "apply": true, "ids": [path]}));
    assert_eq!(strs(&added["added"]), ["Double Arrow"]);
    assert_eq!(added["applied"], json!([path]));
    assert_eq!(undo_len(&s), before + 1);
    assert_eq!(doc(&s).node(NodeId(path)).unwrap().appearance.stroke().unwrap().brush.as_deref(), Some("Double Arrow"));
    run(&mut s, "edit.undo", json!({}));
    assert!(vectorcraft_brush::find(doc(&s), "Double Arrow").is_none());
    run(&mut s, "edit.redo", json!({}));
    assert_eq!(brush(&s, "Double Arrow"), lib.brushes[2]);
}

#[test]
fn the_chevron_arrows_second_chevron_keeps_its_length_on_a_long_path() {
    let s = session();
    let (_, lib) = cmd::brushlib::library(&s, "Arrows").unwrap();
    let b = lib.brush("Chevron Arrow").unwrap();
    let widths = |length: f64| {
        let pieces = vectorcraft_brush::stroke_pieces(&lib.doc, b, &line_to(length), &black_stroke());
        let children = pieces[0].children().unwrap();
        assert_eq!(children.len(), 3, "shaft, head and second chevron");
        children[1..].iter().map(|n| n.geometric_bounds().unwrap().width()).collect::<Vec<_>>()
    };
    let short = widths(100.0);
    for (long, short) in widths(400.0).into_iter().zip(short) {
        assert!((long - short).abs() < 1e-6, "the heads keep their length: {long} on 400 pt, {short} on 100 pt");
    }
}

#[test]
fn dashed_arrow_keeps_separate_dashes_and_survives_library_save() {
    let mut s = session();
    let (_, lib) = cmd::brushlib::library(&s, "Arrows").unwrap();
    let b = lib.brush("Dashed Arrow").unwrap();
    let mut curve = line_to(40.0);
    curve.curve_to((60.0, -40.0), (140.0, 40.0), (200.0, 0.0));
    for path in [line_to(2.0), line_to(200.0), curve] {
        let pieces = vectorcraft_brush::stroke_pieces(&lib.doc, b, &path, &black_stroke());
        let pts = anchors(&pieces.iter().collect::<Vec<_>>());
        assert!(!pts.is_empty() && pts.iter().all(|p| p.x.is_finite() && p.y.is_finite()));
    }
    let pieces = vectorcraft_brush::stroke_pieces(&lib.doc, b, &line_to(200.0), &black_stroke());
    let children = pieces[0].children().unwrap();
    assert_eq!(children.len(), 6, "five separate dashes and one head");
    let bounds: Vec<_> = children.iter().map(|n| n.geometric_bounds().unwrap()).collect();
    assert!(bounds.windows(2).all(|p| p[0].x1 < p[1].x0), "the dashes and head remain separated");
    run(&mut s, "brush.addFromLibrary", json!({"library": "arrows", "name": "Dashed Arrow"}));
    let saved = run(&mut s, "brush.saveLibrary", json!({"names": ["Dashed Arrow"]}));
    let bytes = etcfile::base64::decode(saved["dataBase64"].as_str().unwrap()).unwrap();
    let reopened = etcfile::load(&bytes).unwrap();
    assert_eq!(vectorcraft_brush::find(&reopened, "Dashed Arrow").unwrap(), *b);
}

/// Original circuit art and repeat tiles survive adding, applying and saving a library.
#[test]
fn the_tech_circuit_library_constructs_adds_and_applies_each_brush() {
    let mut s = session();
    let listed = run(&mut s, "brush.libraries", json!({}));
    let entry = listed["libraries"].as_array().unwrap().iter().find(|l| l["id"] == "tech-circuit").unwrap();
    assert_eq!((entry["name"].as_str(), entry["category"].as_str(), entry["count"].as_u64()), (Some("Tech Circuit"), Some("builtIn"), Some(3)));
    let (_, lib) = cmd::brushlib::library(&s, "Tech Circuit").unwrap();
    let expected = [("Circuit Trace", "art"), ("Bus Lines", "pattern"), ("Terminal Nodes", "pattern")];
    assert_eq!(lib.brushes.iter().map(|b| (b.name.as_str(), b.kind.type_id())).collect::<Vec<_>>(), expected);
    let path = run(&mut s, "shape.line", json!({"x1": 10, "y1": 20, "x2": 210, "y2": 20}))["id"].as_u64().unwrap();
    let curve = {
        let mut p = line_to(40.0);
        p.curve_to((60.0, -40.0), (140.0, 40.0), (200.0, 0.0));
        p
    };
    let corner = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(0.0, 0.0, 80.0, 60.0)).to_bezpath();
    for b in &lib.brushes {
        let mut sanitized = b.clone();
        sanitized.sanitize();
        assert_eq!(&sanitized, b, "{} changes on load", b.name);
        for tile in b.art() {
            let pts = anchors(&[tile]);
            assert!(!pts.is_empty() && pts.len() <= 100, "{}", b.name);
            assert!(pts.iter().all(|p| p.x.is_finite() && p.y.is_finite()));
            tile.walk(&mut |n| {
                if let Some(p) = n.path_data() {
                    assert!(p.subpaths.iter().all(|s| s.closed));
                }
            });
        }
        if let BrushKind::Pattern(p) = &b.kind {
            assert_eq!(p.colorization, vectorcraft_brush::Colorization::Tints);
            assert_eq!(p.spacing, 0.0, "conductors join adjacent tiles");
            if b.name == "Bus Lines" {
                let lanes = p.side.children().unwrap();
                assert_eq!(lanes.len(), 3);
                let boxes: Vec<_> = lanes.iter().map(|n| n.path_data().unwrap().bounds().unwrap()).collect();
                for r in &boxes {
                    assert_eq!((r.x0, r.x1), (0.0, 24.0));
                }
                assert!(boxes.windows(2).all(|r| r[0].y1 < r[1].y0), "lanes stay separate");
            }
            if b.name == "Terminal Nodes" {
                let mut rings = 0;
                p.side.walk(&mut |n| {
                    if matches!(n.kind, NodeKind::Compound { rule: vectorcraft_geom::FillRule::EvenOdd, .. }) {
                        rings += 1;
                    }
                });
                assert_eq!(rings, 1, "a hollow circular test point per tile");
            }
        }
        for p in [line_to(200.0), line_to(2.0), curve.clone(), corner.clone()] {
            let pieces = vectorcraft_brush::stroke_pieces(&lib.doc, b, &p, &black_stroke());
            let pts = anchors(&pieces.iter().collect::<Vec<_>>());
            assert!(!pts.is_empty() && pts.iter().all(|p| p.x.is_finite() && p.y.is_finite()), "{}", b.name);
        }
        let before = undo_len(&s);
        let r = run(&mut s, "brush.addFromLibrary", json!({"library": "tech-circuit", "name": b.name, "apply": true, "ids": [path]}));
        assert_eq!(strs(&r["added"]), [b.name.as_str()]);
        assert_eq!(r["applied"], json!([path]));
        assert_eq!(undo_len(&s), before + 1);
        assert_eq!(brush(&s, &b.name), *b);
        assert_eq!(doc(&s).node(NodeId(path)).unwrap().appearance.stroke().unwrap().brush.as_deref(), Some(b.name.as_str()));
        run(&mut s, "edit.undo", json!({}));
        assert!(vectorcraft_brush::find(doc(&s), &b.name).is_none());
        run(&mut s, "edit.redo", json!({}));
        let before = undo_len(&s);
        let again = run(&mut s, "brush.addFromLibrary", json!({"library": "tech-circuit", "name": b.name}));
        assert!(strs(&again["added"]).is_empty());
        assert_eq!(strs(&again["existing"]), [b.name.as_str()]);
        assert_eq!(undo_len(&s), before);
    }
    let names: Vec<&str> = expected.iter().map(|(name, _)| *name).collect();
    let saved = run(&mut s, "brush.saveLibrary", json!({"names": names}));
    let bytes = etcfile::base64::decode(saved["dataBase64"].as_str().unwrap()).unwrap();
    let reopened = etcfile::load(&bytes).unwrap();
    assert_eq!(vectorcraft_brush::library(&reopened), lib.brushes);
}

#[test]
fn decorative_brushes_repeat_distinct_art_and_survive_a_saved_library() {
    let mut s = session();
    let (_, lib) = cmd::brushlib::library(&s, "Decorative").unwrap();
    let names = ["Leaf Vine", "Blossom Chain", "Pennant Garland", "Lantern String"];
    assert_eq!(lib.brushes.iter().map(|b| b.name.as_str()).collect::<Vec<_>>(), names);
    let path = run(&mut s, "shape.line", json!({"x1": 10, "y1": 40, "x2": 190, "y2": 40}))["id"].as_u64().unwrap();
    let mut signatures = std::collections::HashSet::new();
    for b in &lib.brushes {
        let BrushKind::Pattern(pattern) = &b.kind else { panic!("{} is not a pattern brush", b.name) };
        assert_eq!(pattern.colorization, vectorcraft_brush::Colorization::Tints);
        let points = |len: f64| anchors(&vectorcraft_brush::stroke_pieces(&lib.doc, b, &line_to(len), &black_stroke()).iter().collect::<Vec<_>>());
        let (one, four) = (points(48.0), points(192.0));
        assert!(!one.is_empty() && four.iter().all(|p| p.x.is_finite() && p.y.is_finite()), "{}", b.name);
        // A path four tiles long places the motif four times, each copy one tile further along.
        assert_eq!(four.len(), 4 * one.len(), "{} repeats its motif", b.name);
        for (i, p) in four.iter().enumerate() {
            let q = one[i % one.len()];
            let shift = 48.0 * (i / one.len()) as f64;
            assert!((p.x - q.x - shift).abs() < 1e-3 && (p.y - q.y).abs() < 1e-3, "{} copy {} at {p:?}, not {q:?}", b.name, i / one.len());
        }
        signatures.insert(one.iter().map(|p| ((p.x * 100.0).round() as i64, (p.y * 100.0).round() as i64)).collect::<Vec<_>>());
        let before = undo_len(&s);
        let applied = run(&mut s, "brush.addFromLibrary", json!({"library": "decorative", "name": b.name, "apply": true, "ids": [path]}));
        assert_eq!((strs(&applied["added"]), &applied["applied"]), (vec![b.name.as_str()], &json!([path])));
        assert_eq!(undo_len(&s), before + 1);
        assert_eq!(doc(&s).node(NodeId(path)).unwrap().appearance.stroke().unwrap().brush.as_deref(), Some(b.name.as_str()));
    }
    assert_eq!(signatures.len(), names.len(), "each design produces a different repeating shape");
    let saved = run(&mut s, "brush.saveLibrary", json!({"names": names}));
    let bytes = etcfile::base64::decode(saved["dataBase64"].as_str().unwrap()).unwrap();
    let reopened = etcfile::load(&bytes).unwrap();
    assert_eq!(vectorcraft_brush::library(&reopened), lib.brushes);
}

#[test]
fn ornamental_brushes_draw_flourishes_and_survive_a_saved_library() {
    let mut s = session();
    let (_, lib) = cmd::brushlib::library(&s, "Ornamental").unwrap();
    let names = ["Scrollwork", "Filigree Arrow", "Petal Sweep", "Twin Curl"];
    assert_eq!(lib.brushes.iter().map(|b| b.name.as_str()).collect::<Vec<_>>(), names);
    let path = run(&mut s, "shape.line", json!({"x1": 10, "y1": 40, "x2": 190, "y2": 40}))["id"].as_u64().unwrap();
    let mut curve = line_to(20.0);
    curve.curve_to((60.0, -25.0), (140.0, 25.0), (180.0, 0.0));
    for b in &lib.brushes {
        let BrushKind::Art(art) = &b.kind else { panic!("{} is not an art brush", b.name) };
        assert_eq!(art.colorization, vectorcraft_brush::Colorization::Tints);
        assert_eq!(art.width, 300.0);
        for line in [line_to(2.0), line_to(180.0), curve.clone()] {
            let pieces = vectorcraft_brush::stroke_pieces(&lib.doc, b, &line, &black_stroke());
            let points = anchors(&pieces.iter().collect::<Vec<_>>());
            assert!(!points.is_empty() && points.iter().all(|p| p.x.is_finite() && p.y.is_finite()), "{}", b.name);
        }
        let before = undo_len(&s);
        let added = run(&mut s, "brush.addFromLibrary", json!({"library": "ornamental", "name": b.name, "apply": true, "ids": [path]}));
        assert_eq!(strs(&added["added"]), [b.name.as_str()]);
        assert_eq!(added["applied"], json!([path]));
        assert_eq!(undo_len(&s), before + 1);
        assert_eq!(doc(&s).node(NodeId(path)).unwrap().appearance.stroke().unwrap().brush.as_deref(), Some(b.name.as_str()));
    }
    let saved = run(&mut s, "brush.saveLibrary", json!({"names": names}));
    let bytes = etcfile::base64::decode(saved["dataBase64"].as_str().unwrap()).unwrap();
    let reopened = etcfile::load(&bytes).unwrap();
    assert_eq!(vectorcraft_brush::library(&reopened), lib.brushes);
}

/// Filigree Arrow's shaft runs into its head with no gap, so a straight stroke draws one arrow.
#[test]
fn the_filigree_arrow_shaft_meets_its_head() {
    let s = session();
    let (_, lib) = cmd::brushlib::library(&s, "Ornamental").unwrap();
    let BrushKind::Art(art) = &lib.brush("Filigree Arrow").unwrap().kind else { panic!("Filigree Arrow is not an art brush") };
    let mut parts = vec![];
    art.art.walk(&mut |c| {
        if c.path_data().is_some() {
            parts.push(c.contains_fn());
        }
    });
    for y in [-0.6, 0.0, 0.6] {
        for i in 0..=930 {
            let p = vectorcraft_geom::Point::new(4.05 + 0.1 * f64::from(i), y);
            assert!(parts.iter().any(|inside| inside(p)), "gap at {p:?}");
        }
    }
}

#[test]
fn pressure_sensitive_brushes_widen_with_pen_pressure_and_survive_a_saved_library() {
    let mut s = session();
    let (_, lib) = cmd::brushlib::library(&s, "Pressure Sensitive").unwrap();
    let names = ["Flex Ribbon", "Pulse Stroke", "Round Flow", "Chisel Flow"];
    assert_eq!(lib.brushes.iter().map(|b| b.name.as_str()).collect::<Vec<_>>(), names);
    let path = run(&mut s, "shape.line", json!({"x1": 10, "y1": 40, "x2": 190, "y2": 40}))["id"].as_u64().unwrap();
    let mut curve = line_to(20.0);
    curve.curve_to((60.0, -25.0), (140.0, 25.0), (180.0, 0.0));
    for b in &lib.brushes {
        let width_at = |pressure| {
            let stroke = vectorcraft_doc::StrokeLayer {
                pressure: Some(vectorcraft_doc::PressureProfile { points: vec![(0.0, pressure), (1.0, pressure)] }),
                ..black_stroke()
            };
            let pieces = vectorcraft_brush::stroke_pieces(&lib.doc, b, &line_to(180.0), &stroke);
            let points = anchors(&pieces.iter().collect::<Vec<_>>());
            assert!(!points.is_empty() && points.iter().all(|p| p.x.is_finite() && p.y.is_finite()), "{} at pressure {pressure}", b.name);
            let min = points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
            let max = points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
            max - min
        };
        let light = width_at(0.0);
        let heavy = width_at(1.0);
        for width in [light, heavy] {
            assert!(width.is_finite() && width > 0.0, "{}: light={light}, heavy={heavy}", b.name);
        }
        assert!(heavy > light * 1.3, "{}: light={light}, heavy={heavy}", b.name);
        for line in [line_to(2.0), curve.clone()] {
            let pieces = vectorcraft_brush::stroke_pieces(&lib.doc, b, &line, &black_stroke());
            let points = anchors(&pieces.iter().collect::<Vec<_>>());
            assert!(!points.is_empty() && points.iter().all(|p| p.x.is_finite() && p.y.is_finite()), "{}", b.name);
        }
        let before = undo_len(&s);
        let added = run(&mut s, "brush.addFromLibrary", json!({"library": "pressure-sensitive", "name": b.name, "apply": true, "ids": [path]}));
        assert_eq!(strs(&added["added"]), [b.name.as_str()]);
        assert_eq!(added["applied"], json!([path]));
        assert_eq!(undo_len(&s), before + 1);
        assert_eq!(doc(&s).node(NodeId(path)).unwrap().appearance.stroke().unwrap().brush.as_deref(), Some(b.name.as_str()));
    }
    let saved = run(&mut s, "brush.saveLibrary", json!({"names": names}));
    let bytes = etcfile::base64::decode(saved["dataBase64"].as_str().unwrap()).unwrap();
    let reopened = etcfile::load(&bytes).unwrap();
    assert_eq!(vectorcraft_brush::library(&reopened), lib.brushes);
}

#[test]
fn paint_brushes_draw_distinct_marks_and_survive_a_saved_library() {
    let mut s = session();
    let (_, lib) = cmd::brushlib::library(&s, "Paint").unwrap();
    let names = ["Flat Sweep", "Dry Drag", "Loaded Stroke", "Fan Sweep"];
    assert_eq!(lib.brushes.iter().map(|b| b.name.as_str()).collect::<Vec<_>>(), names);
    let path = run(&mut s, "shape.line", json!({"x1": 10, "y1": 40, "x2": 190, "y2": 40}))["id"].as_u64().unwrap();
    let mut curve = line_to(20.0);
    curve.curve_to((60.0, -25.0), (140.0, 25.0), (180.0, 0.0));
    for b in &lib.brushes {
        let BrushKind::Art(art) = &b.kind else { panic!("{} is not an art brush", b.name) };
        assert_eq!(art.colorization, vectorcraft_brush::Colorization::Tints);
        assert_eq!(art.width, 150.0);
        for line in [line_to(2.0), line_to(180.0), curve.clone()] {
            let pieces = vectorcraft_brush::stroke_pieces(&lib.doc, b, &line, &black_stroke());
            let points = anchors(&pieces.iter().collect::<Vec<_>>());
            assert!(!points.is_empty() && points.iter().all(|p| p.x.is_finite() && p.y.is_finite()), "{}", b.name);
        }
        let before = undo_len(&s);
        let added = run(&mut s, "brush.addFromLibrary", json!({"library": "paint", "name": b.name, "apply": true, "ids": [path]}));
        assert_eq!(strs(&added["added"]), [b.name.as_str()]);
        assert_eq!(added["applied"], json!([path]));
        assert_eq!(undo_len(&s), before + 1);
        assert_eq!(doc(&s).node(NodeId(path)).unwrap().appearance.stroke().unwrap().brush.as_deref(), Some(b.name.as_str()));
    }
    let saved = run(&mut s, "brush.saveLibrary", json!({"names": names}));
    let bytes = etcfile::base64::decode(saved["dataBase64"].as_str().unwrap()).unwrap();
    let reopened = etcfile::load(&bytes).unwrap();
    assert_eq!(vectorcraft_brush::library(&reopened), lib.brushes);
}


/// Every Ink and Wash brush draws along straight, curved, short and tight paths within finite
/// bounds, and Tapered Ink is broad near its start and fine at its end.
#[test]
fn ink_and_wash_brushes_draw_along_straight_curved_short_and_tight_paths() {
    let inks = inks(&Session::new());
    let mut curve = vectorcraft_geom::BezPath::new();
    curve.move_to((0.0, 0.0));
    curve.curve_to((60.0, -40.0), (140.0, 40.0), (200.0, 0.0));
    let tight = vectorcraft_geom::shapes::ellipse(vectorcraft_geom::Rect::new(0.0, 0.0, 6.0, 6.0)).to_bezpath();
    let paths = [line_to(200.0), line_to(3.0), curve, tight];
    for b in &inks.brushes {
        for line in &paths {
            let pieces = vectorcraft_brush::stroke_pieces(&inks.doc, b, line, &black_stroke());
            assert!(!pieces.is_empty(), "{} draws nothing", b.name);
            let bounds: Vec<_> = pieces.iter().map(|p| p.geometric_bounds()).collect();
            assert!(bounds.iter().all(|r| r.is_some_and(|r| [r.x0, r.y0, r.x1, r.y1].iter().all(|v| v.is_finite()))), "{}", b.name);
        }
    }
    let pieces = vectorcraft_brush::stroke_pieces(&inks.doc, inks.brush("Tapered Ink").unwrap(), &line_to(100.0), &black_stroke());
    let widest = |from: f64, to: f64| {
        pieces
            .iter()
            .filter_map(|n| n.path_data())
            .flat_map(|p| p.anchors().map(|(_, _, a)| a.p).collect::<Vec<_>>())
            .filter(|p| p.x >= from && p.x <= to)
            .map(|p| p.y.abs())
            .fold(0.0, f64::max)
    };
    assert!(widest(15.0, 30.0) > 4.0 * widest(95.0, 100.0), "broad start, fine tip");
}

/// Ink and Wash brushes are added and applied in one step, undone and redone, not added twice,
/// and kept with the brushed stroke by a saved and reopened document.
#[test]
fn ink_and_wash_brushes_add_apply_undo_and_survive_a_saved_document() {
    let mut s = session();
    let inks = inks(&s);
    let path = run(&mut s, "shape.line", json!({"x1": 10, "y1": 10, "x2": 150, "y2": 60}))["id"].as_u64().unwrap();
    let (defaults, before) = (brushes(&s), undo_len(&s));
    let names = ["Tapered Ink", "Brush Pen", "Soft Wash", "Pooling Wash"];
    let r = run(&mut s, "brush.addFromLibrary", json!({"library": "ink-and-wash", "name": "Soft Wash", "apply": true, "ids": [path]}));
    assert_eq!((strs(&r["added"]), r["applied"].clone(), undo_len(&s)), (vec!["Soft Wash"], json!([path]), before + 1));
    let stroke_brush = |s: &Session| doc(s).node(NodeId(path)).unwrap().appearance.stroke().and_then(|st| st.brush.clone());
    assert_eq!(stroke_brush(&s).as_deref(), Some("Soft Wash"));
    run(&mut s, "edit.undo", json!({}));
    assert_eq!((brushes(&s), stroke_brush(&s)), (defaults.clone(), None));
    run(&mut s, "edit.redo", json!({}));
    assert_eq!(stroke_brush(&s).as_deref(), Some("Soft Wash"));
    let r = run(&mut s, "brush.addFromLibrary", json!({"library": "ink-and-wash"}));
    assert_eq!(
        (strs(&r["names"]), strs(&r["existing"]), strs(&r["added"])),
        (names.to_vec(), vec!["Soft Wash"], vec!["Tapered Ink", "Brush Pen", "Pooling Wash"])
    );
    let before = undo_len(&s);
    let r = run(&mut s, "brush.addFromLibrary", json!({"library": "ink-and-wash"}));
    assert_eq!((strs(&r["existing"]), undo_len(&s)), (names.to_vec(), before));
    assert_eq!(brushes(&s).len(), defaults.len() + 4);
    for n in names {
        assert_eq!(&brush(&s, n), inks.brush(n).unwrap(), "{n}");
    }

    // Saved and opened again, the document keeps the definitions and the brushed stroke.
    let b64 = run(&mut s, "document.serialize", json!({}))["dataBase64"].as_str().unwrap().to_string();
    let reopened = etcfile::load(&etcfile::base64::decode(&b64).unwrap()).unwrap();
    for n in names {
        assert_eq!(vectorcraft_brush::find(&reopened, n).as_ref(), inks.brush(n), "{n}");
    }
    let n = reopened.node(NodeId(path)).unwrap();
    assert_eq!(n.appearance.stroke().and_then(|st| st.brush.clone()).as_deref(), Some("Soft Wash"));
    assert_eq!(
        vectorcraft_brush::node_pieces(&reopened, n).map(|p| p.len()),
        vectorcraft_brush::node_pieces(doc(&s), doc(&s).node(NodeId(path)).unwrap()).map(|p| p.len())
    );
}


/// Every anchor of the paths in `nodes` and their descendants.
fn anchors(nodes: &[&vectorcraft_doc::Node]) -> Vec<vectorcraft_geom::Point> {
    let mut out = vec![];
    for n in nodes {
        n.walk(&mut |c| out.extend(c.path_data().into_iter().flat_map(|p| p.subpaths.iter().flat_map(|sp| sp.anchors.iter().map(|a| a.p)))));
    }
    out
}



#[test]
fn a_saved_library_reopens_and_its_brushes_join_another_document_in_one_step() {
    let dir = folder("user");
    let mut s = kit();
    s.brush_libraries.set_user_dir(Some(dir.to_string_lossy().to_string()));
    let r = run(&mut s, "brush.saveLibrary", json!({"user": true, "name": "Inks", "names": ["My Nib", "Dotted Band"]}));
    assert_eq!((r["library"].as_str(), r["count"].as_u64()), (Some("user/Inks.etc"), Some(2)));
    let path = r["path"].as_str().unwrap().to_string();
    let list = run(&mut s, "brush.libraries", json!({}));
    let user: Vec<&Value> = list["libraries"].as_array().unwrap().iter().filter(|l| l["category"] == "user").collect();
    assert_eq!(user.len(), 1);
    assert_eq!((user[0]["name"].as_str(), user[0]["count"].as_u64()), (Some("Inks"), Some(2)));
    // The file is a document: the two brushes and the pattern their art uses, no art.
    let saved = etcfile::load(&std::fs::read(&path).unwrap()).unwrap();
    let names: Vec<String> = vectorcraft_brush::library(&saved).into_iter().map(|b| b.name).collect();
    assert_eq!(names, ["My Nib", "Dotted Band"]);
    assert_eq!(saved.patterns.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(), ["Dots"]);
    assert!(saved.layers.iter().all(|l| l.children().is_none_or(|c| c.is_empty())));
    // Loading the file gives that User Defined library.
    let r = run(&mut s, "brush.loadLibrary", json!({ "path": path }));
    assert_eq!((r["library"].as_str(), r["name"].as_str()), (Some("user/Inks.etc"), Some("Inks")));

    // Into a new document: Dotted Band and its pattern, as one step.
    run(&mut s, "file.new", json!({"width": 100, "height": 100}));
    let defaults = brushes(&s);
    let before = undo_len(&s);
    let r = run(&mut s, "brush.addFromLibrary", json!({"library": "Inks", "name": "Dotted Band"}));
    assert_eq!((strs(&r["names"]), strs(&r["added"]), strs(&r["existing"])), (vec!["Dotted Band"], vec!["Dotted Band"], vec![]));
    assert_eq!(undo_len(&s), before + 1);
    assert_eq!(s.doc().unwrap().history.undo.last().unwrap().label, "Add to Brushes");
    assert_eq!(brushes(&s), [defaults.clone(), vec!["Dotted Band".to_string()]].concat());
    let (_, lib) = cmd::brushlib::library(&s, "Inks").unwrap();
    assert_eq!(&brush(&s, "Dotted Band"), lib.brush("Dotted Band").unwrap());
    assert!(doc(&s).pattern("Dots").is_some() && doc(&s).swatch("Dots").is_some(), "the pattern its art fills with");
    assert_eq!(run(&mut s, "brush.list", json!({}))["current"], "Dotted Band");
    // Undo takes it all back.
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(brushes(&s), defaults);
    assert!(doc(&s).pattern("Dots").is_none());
    run(&mut s, "edit.redo", json!({}));
    // Again: already there, so no step and no copy.
    let before = undo_len(&s);
    let r = run(&mut s, "brush.addFromLibrary", json!({"library": "user/Inks.etc", "names": ["Dotted Band", "My Nib", "Dotted Band"]}));
    assert_eq!((strs(&r["names"]), strs(&r["existing"]), strs(&r["added"])), (vec!["Dotted Band", "My Nib"], vec!["Dotted Band"], vec!["My Nib"]));
    assert_eq!(undo_len(&s), before + 1, "My Nib was new");
    let before = undo_len(&s);
    let r = run(&mut s, "brush.addFromLibrary", json!({"library": "Inks"}));
    assert_eq!((strs(&r["existing"]).len(), undo_len(&s)), (2, before));
    assert_eq!(brushes(&s).len(), defaults.len() + 2);
    assert_eq!(doc(&s).patterns.len(), 1);
    assert_eq!(run(&mut s, "brush.list", json!({}))["current"], "My Nib", "the library's first brush");
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn library_brushes_bring_their_patterns_and_symbols_rename_clashes_and_apply() {
    let mut s = session();
    // The document has a pattern "Pinstripes" and a brush "Wave" of its own.
    add_pattern(&mut s, "Pinstripes", "#00ff00");
    let mine = doc(&s).pattern("Pinstripes").unwrap().clone();
    run(&mut s, "brush.new", json!({"type": "calligraphic", "name": "Wave"}));
    let my_wave = brush(&s, "Wave");
    let path = run(&mut s, "shape.line", json!({"x1": 10, "y1": 10, "x2": 150, "y2": 60}))["id"].as_u64().unwrap();
    run(&mut s, "select.set", json!({"ids": [path]}));
    let before = undo_len(&s);
    let r = run(&mut s, "brush.addFromLibrary", json!({"library": "art-ribbons", "names": ["Striped Ribbon", "Wave"], "apply": true}));
    assert_eq!(undo_len(&s), before + 1, "adding and applying are one step");
    assert_eq!(strs(&r["names"]), ["Striped Ribbon", "Wave 2"]);
    assert_eq!(r["applied"], json!([path]));
    assert_eq!(brush(&s, "Wave"), my_wave, "the document's own Wave stays");
    // The library's Pinstripes comes in under a free name, and the brush's art follows it.
    assert_eq!(doc(&s).pattern("Pinstripes").unwrap(), &mine);
    assert!(doc(&s).pattern("Pinstripes 2").is_some() && doc(&s).swatch("Pinstripes 2").is_some());
    assert_eq!(art_patterns(&brush(&s, "Striped Ribbon")), ["Pinstripes 2"]);
    // The first one is applied to the selected path and is the current brush.
    let n = doc(&s).node(NodeId(path)).unwrap();
    assert_eq!(n.appearance.stroke().and_then(|st| st.brush.clone()).as_deref(), Some("Striped Ribbon"));
    assert_eq!(run(&mut s, "brush.list", json!({}))["current"], "Striped Ribbon");
    // Adding it again finds it, its pattern renamed, and adds nothing.
    let before = undo_len(&s);
    let r = run(&mut s, "brush.addFromLibrary", json!({"library": "art-ribbons", "names": ["Striped Ribbon", "Wave"]}));
    assert_eq!((strs(&r["names"]), strs(&r["existing"])), (vec!["Striped Ribbon", "Wave 2"], vec!["Striped Ribbon", "Wave 2"]));
    assert_eq!(undo_len(&s), before);
    assert!(doc(&s).pattern("Pinstripes 3").is_none() && vectorcraft_brush::find(doc(&s), "Striped Ribbon 2").is_none());
    // Undo of the first add takes the brushes, the pattern and the brush stroke back.
    run(&mut s, "edit.undo", json!({}));
    assert!(vectorcraft_brush::find(doc(&s), "Striped Ribbon").is_none() && doc(&s).pattern("Pinstripes 2").is_none());
    assert!(doc(&s).node(NodeId(path)).unwrap().appearance.stroke().is_none_or(|st| st.brush.is_none()));
    // A scatter brush brings the symbol it scatters.
    let r = run(&mut s, "brush.addFromLibrary", json!({"library": "Scatter Shapes", "name": "Sparkles"}));
    assert_eq!(strs(&r["added"]), ["Sparkles"]);
    assert_eq!(doc(&s).symbols.iter().map(|x| x.name.as_str()).collect::<Vec<_>>(), ["Sparkle"]);
    // Applying a brush the document has still applies (one step) to the paths given, without
    // adding it again.
    run(&mut s, "select.none", json!({}));
    let before = undo_len(&s);
    let r = run(&mut s, "brush.addFromLibrary", json!({"library": "scatter-shapes", "name": "Sparkles", "apply": true, "ids": [path]}));
    assert_eq!((undo_len(&s), strs(&r["existing"]), r["applied"].clone()), (before + 1, vec!["Sparkles"], json!([path])));
    assert_eq!(doc(&s).node(NodeId(path)).unwrap().appearance.stroke().and_then(|st| st.brush.clone()).as_deref(), Some("Sparkles"));
    assert_eq!(doc(&s).symbols.len(), 1);
    assert!(s.execute("brush.addFromLibrary", &json!({"library": "art-ribbons", "name": "Nope"})).is_err());
    assert!(s.execute("brush.addFromLibrary", &json!({"name": "Wave"})).is_err());
}

/// The undo step is Add to Brushes when a brush comes in (applied or not), and Apply Brush when
/// the brushes are all there and the click only applies one; undo and redo of either restore the
/// document as it was and as the step left it.
#[test]
fn adding_library_brushes_is_add_to_brushes_and_only_applying_one_is_apply_brush() {
    let mut s = session();
    let path = run(&mut s, "shape.line", json!({"x1": 10, "y1": 10, "x2": 150, "y2": 60}))["id"].as_u64().unwrap();
    let label = |s: &Session| s.doc().unwrap().history.undo.last().unwrap().label.clone();
    let stroke_brush = |s: &Session| doc(s).node(NodeId(path)).unwrap().appearance.stroke().and_then(|st| st.brush.clone());
    run(&mut s, "brush.addFromLibrary", json!({"library": "art-ribbons", "name": "Wave", "apply": true, "ids": [path]}));
    assert_eq!((label(&s), stroke_brush(&s).as_deref()), ("Add to Brushes".to_string(), Some("Wave")));
    run(&mut s, "brush.addFromLibrary", json!({"library": "art-ribbons", "name": "Teardrop"}));
    assert_eq!(label(&s), "Add to Brushes");
    let (before, brushes_before) = (undo_len(&s), brushes(&s));
    let r = run(&mut s, "brush.addFromLibrary", json!({"library": "art-ribbons", "name": "Teardrop", "apply": true, "ids": [path]}));
    assert_eq!((strs(&r["added"]), strs(&r["existing"])), (vec![], vec!["Teardrop"]));
    assert_eq!((undo_len(&s), label(&s), brushes(&s)), (before + 1, "Apply Brush".to_string(), brushes_before.clone()));
    assert_eq!(stroke_brush(&s).as_deref(), Some("Teardrop"));
    run(&mut s, "edit.undo", json!({}));
    assert_eq!((stroke_brush(&s).as_deref(), brushes(&s)), (Some("Wave"), brushes_before.clone()));
    run(&mut s, "edit.redo", json!({}));
    assert_eq!((stroke_brush(&s).as_deref(), label(&s)), (Some("Teardrop"), "Apply Brush".to_string()));
}

#[test]
fn pasted_brushed_art_brings_the_patterns_its_brush_art_uses() {
    let mut s = kit();
    let line = run(&mut s, "shape.line", json!({"x1": 10, "y1": 100, "x2": 150, "y2": 120}))["id"].as_u64().unwrap();
    run(&mut s, "brush.apply", json!({"name": "Dotted Band", "ids": [line]}));
    run(&mut s, "select.set", json!({"ids": [line]}));
    run(&mut s, "edit.copy", json!({}));
    run(&mut s, "file.new", json!({"width": 200, "height": 200}));
    run(&mut s, "edit.paste", json!({}));
    assert!(vectorcraft_brush::find(doc(&s), "Dotted Band").is_some());
    assert!(doc(&s).pattern("Dots").is_some(), "the brush's art is filled with it");
}

#[test]
fn other_documents_brushes_load_and_unreadable_files_are_errors() {
    let mut s = kit();
    // Another document's brushes load as a library, without its art.
    let b64 = run(&mut s, "document.serialize", json!({}))["dataBase64"].as_str().unwrap().to_string();
    let count = brushes(&s).len();
    let r = run(&mut s, "brush.loadLibrary", json!({"dataBase64": b64, "name": "Poster.etc"}));
    assert_eq!((r["name"].as_str(), r["count"].as_u64()), (Some("Poster"), Some(count as u64)));
    let (_, lib) = cmd::brushlib::library(&s, r["library"].as_str().unwrap()).unwrap();
    assert!(lib.doc.layers.iter().all(|l| l.children().is_none_or(|c| c.is_empty())));
    assert!(lib.brush("Dotted Band").is_some() && lib.doc.pattern("Dots").is_some());
    // An SVG has the default brushes, as it does opened.
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><rect width="10" height="10"/></svg>"##;
    let r = run(&mut s, "brush.loadLibrary", json!({"data": svg, "name": "Box.svg"}));
    assert_eq!(r["count"].as_u64(), Some(vectorcraft_brush::defaults().len() as u64), "{r}");
    // Files that aren't documents, missing files and damaged native files are errors.
    assert!(s.execute("brush.loadLibrary", &json!({"data": "hello", "name": "x.txt"})).is_err());
    assert!(s.execute("brush.loadLibrary", &json!({"path": "/no/such/Library.etc"})).is_err());
    let dir = folder("bad");
    std::fs::create_dir_all(&dir).unwrap();
    let bad = dir.join("Broken.etc");
    std::fs::write(&bad, b"{\"format\": \"vectorcraft\", \"doc\": 7").unwrap();
    assert!(s.execute("brush.loadLibrary", &json!({"path": bad.to_string_lossy()})).is_err());
    // A damaged file in the user folder isn't listed.
    s.brush_libraries.set_user_dir(Some(dir.to_string_lossy().to_string()));
    let list = run(&mut s, "brush.libraries", json!({}));
    assert!(list["libraries"].as_array().unwrap().iter().all(|l| l["category"] != "user"));
    s.brush_libraries.set_user_dir(None);
    let _ = std::fs::remove_dir_all(dir);
    // No such brush, or a headless session's user folder, is an error.
    assert!(s.execute("brush.saveLibrary", &json!({"names": ["Nope"]})).is_err());
    assert!(s.execute("brush.saveLibrary", &json!({"user": true})).is_err());
    // A document whose brushes were all deleted has none to save.
    let all = brushes(&s);
    run(&mut s, "brush.delete", json!({ "names": all }));
    assert!(s.execute("brush.saveLibrary", &json!({})).is_err());
    // Without a path the library comes back as the file's bytes.
    let mut s = kit();
    let r = run(&mut s, "brush.saveLibrary", json!({"names": ["My Nib"]}));
    let bytes = etcfile::base64::decode(r["dataBase64"].as_str().unwrap()).unwrap();
    assert_eq!(vectorcraft_brush::library(&etcfile::load(&bytes).unwrap()).len(), 1);
}

/// A library file whose brushes repeat a name loads with the later brush numbered, and each one
/// can be added to a document by its own name.
#[test]
fn a_library_file_that_repeats_a_brush_name_loads_with_the_later_one_numbered() {
    use vectorcraft_brush::{Brush, Calligraphic};
    let nib = |size: f64| Brush { name: "Nib".into(), kind: BrushKind::Calligraphic(Calligraphic { size, ..Default::default() }) };
    let mut file = vectorcraft_doc::Document::new(100.0, 100.0);
    vectorcraft_brush::store(&mut file, &[nib(1.0), nib(2.0)]);
    let b64 = etcfile::base64::encode(&etcfile::save_file(&file));
    let mut s = session();
    let r = run(&mut s, "brush.loadLibrary", json!({"dataBase64": b64, "name": "Nibs.etc"}));
    let library = r["library"].as_str().unwrap().to_string();
    let listed = run(&mut s, "brush.library", json!({ "library": library }));
    let names: Vec<&str> = listed["brushes"].as_array().unwrap().iter().map(|b| b["name"].as_str().unwrap()).collect();
    assert_eq!(names, ["Nib", "Nib 2"]);
    let r = run(&mut s, "brush.addFromLibrary", json!({"library": library, "name": "Nib 2"}));
    assert_eq!(strs(&r["added"]), ["Nib 2"]);
    let size = |b: Brush| if let BrushKind::Calligraphic(c) = b.kind { c.size } else { 0.0 };
    assert_eq!(size(brush(&s, "Nib 2")), 2.0);
}
