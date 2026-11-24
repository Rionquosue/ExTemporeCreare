//! The layers of an Illustrator EPS and `.ai`: read from the editing copy of the art the file
//! carries (see `native`). The files here are small ones written by hand for the tests, with the
//! operators seen in files their users own.

use vectorcraft_doc::{Document, Node, NodeKind};



/// `data` as ASCII85, up to the `~>`.
fn ascii85(data: &[u8]) -> String {
    let mut out = String::new();
    for chunk in data.chunks(4) {
        let v = chunk.iter().enumerate().fold(0u32, |v, (i, b)| v | u32::from(*b) << (24 - 8 * i));
        let mut d = [0u8; 5];
        let mut x = v;
        for c in d.iter_mut().rev() {
            *c = (x % 85) as u8 + b'!';
            x /= 85;
        }
        out.extend(d.iter().take(chunk.len() + 1).map(|c| char::from(*c)));
    }
    out.push_str("~>");
    out
}

fn zstd(data: &str) -> Vec<u8> {
    ruzstd::encoding::compress_to_vec(data.as_bytes(), ruzstd::encoding::CompressionLevel::Fastest)
}

/// The editing copy of a file with `layers` (made by [`layer`]), 100 × 100 points.
fn native(layers: &str) -> String {
    format!(
        "%!PS-Adobe-3.0 \n%%BoundingBox: 0 0 100 100\n%%HiResBoundingBox: 0 0 100 100\n%AI3_Cropmarks: 0 0 100 100\n%AI5_NumLayers: 2\n{layers}%%Trailer\n"
    )
}

/// The editing copy of a Large Canvas file (#2031): art and boxes written at 1/`scale`, declared
/// with `%AI24_LargeCanvasScale` as Illustrator writes it. The file's page is `size × scale`
/// points.
fn native_large_canvas(size: f64, scale: f64, layers: &str) -> String {
    format!(
        "%!PS-Adobe-3.0 \n%%BoundingBox: 0 0 {size} {size}\n%%HiResBoundingBox: 0 0 {size} {size}\n%AI3_Cropmarks: 0 0 {size} {size}\n%AI24_LargeCanvasScale: {scale}\n%AI5_NumLayers: 1\n{layers}%%Trailer\n"
    )
}

/// A layer as the format writes it.
pub(super) fn layer(name: &str, visible: bool, body: &str) -> String {
    format!("%AI5_BeginLayer\n{} 1 1 1 0 0 1 0 79 128 255 0 50 0 Lb\n({name}) Ln\n{body}\nLB\n%AI5_EndLayer--\n", u8::from(visible))
}

/// A yellow square of 4 points at (`x`, `y`) in the format's operators.
pub(super) fn square(x: u32, y: u32) -> String {
    format!("0 0 1 0 k\n{x} {y} m\n{} {y} L\n{} {} L\n{x} {} L\nf", x + 4, x + 4, y + 4, y + 4)
}

fn square_path(x: u32, y: u32) -> String {
    format!("{x} {y} m\n{} {y} L\n{} {} L\n{x} {} L\nf", x + 4, x + 4, y + 4, y + 4)
}



fn names(d: &Document) -> Vec<String> {
    d.layers.iter().map(|l| l.name.clone().unwrap_or_default()).collect()
}

/// The ruler guides of `d` by their line, artboard and layer.
fn guide_lines(d: &Document) -> Vec<vectorcraft_doc::Guide> {
    d.ruler_guides()
        .into_iter()
        .map(|g| vectorcraft_doc::Guide { artboard: g.artboard, layer: g.layer, ..vectorcraft_doc::Guide::new(g.vertical, g.pos) })
        .collect()
}




fn kinds(l: &Node) -> Vec<&'static str> {
    l.children()
        .into_iter()
        .flatten()
        .map(|n| match &n.kind {
            NodeKind::Group { clip: true, children } => {
                ["clip[0]", "clip[1]", "clip[2]", "clip[3]"].get(children.len()).copied().unwrap_or("clip[many]")
            }
            NodeKind::Group { children, .. } => {
                ["group[0]", "group[1]", "group[2]", "group[3]"].get(children.len()).copied().unwrap_or("group[many]")
            }
            NodeKind::Compound { children, .. } => {
                ["compound[0]", "compound[1]", "compound[2]", "compound[3]"].get(children.len()).copied().unwrap_or("compound[many]")
            }
            NodeKind::Path { clipping: true, .. } => "clippath",
            NodeKind::Path { .. } => "path",
            NodeKind::Text(_) => "text",
            _ => "other",
        })
        .collect()
}









/// The `.ai`'s editing data: its marker, then the Zstandard stream.
fn private(native: &str) -> Vec<u8> {
    [b"%AI24_ZStandard_Data".as_slice(), &zstd(native)].concat()
}

/// A page of 100 × 100 with a square at (`x`, `y`) in one layer, as a PDF part would give.
fn pdf_part(x: f64, y: f64) -> Document {
    let mut d = Document::new(100.0, 100.0);
    let id = d.alloc_id();
    let path = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(x, y, x + 4.0, y + 4.0));
    let paint = vectorcraft_color::Paint::solid(vectorcraft_color::Color::cmyk(0.0, 0.0, 1.0, 0.0));
    let item = vectorcraft_doc::AppearanceItem::Fill(vectorcraft_doc::FillLayer::new(paint));
    let n = Node::path(id, path, vectorcraft_doc::Appearance { items: vec![item], ..Default::default() });
    let layer = d.layers[0].id;
    d.insert(Some(layer), 0, n).unwrap();
    d
}

fn pdf_two_squares() -> Document {
    let mut page = pdf_part(10.0, 86.0);
    let other = pdf_part(50.0, 86.0);
    let layer = page.layers[0].id;
    let mut square = page.reid(&other.layers[0].children().unwrap()[0]);
    square.appearance = vectorcraft_doc::Appearance {
        items: vec![vectorcraft_doc::AppearanceItem::Fill(vectorcraft_doc::FillLayer::new(vectorcraft_color::Paint::solid(
            vectorcraft_color::Color::cmyk(1.0, 0.0, 0.0, 0.0),
        )))],
        ..Default::default()
    }
    .into();
    page.insert(Some(layer), 1, square).unwrap();
    page
}

fn pdf_three_squares() -> Document {
    let mut page = pdf_two_squares();
    let other = pdf_part(70.0, 86.0);
    let layer = page.layers[0].id;
    let square = page.reid(&other.layers[0].children().unwrap()[0]);
    page.insert(Some(layer), 2, square).unwrap();
    page
}

fn pdf_split_square() -> Document {
    let mut page = pdf_part(10.0, 86.0);
    let layer = page.layers[0].id;
    for (i, color) in [vectorcraft_color::Color::cmyk(1.0, 0.0, 0.0, 0.0), vectorcraft_color::Color::cmyk(0.0, 1.0, 0.0, 0.0)].into_iter().enumerate()
    {
        let x = 50.0 + 2.0 * i as f64;
        let path = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(x, 86.0, x + 2.0, 90.0));
        let appearance = vectorcraft_doc::Appearance {
            items: vec![vectorcraft_doc::AppearanceItem::Fill(vectorcraft_doc::FillLayer::new(vectorcraft_color::Paint::solid(color)))],
            ..Default::default()
        };
        let node = Node::path(page.alloc_id(), path, appearance);
        page.insert(Some(layer), i + 1, node).unwrap();
    }
    page
}

fn os1_group(body: &str) -> String {
    format!("u\n{body}\nU\n")
}

fn replacement_is_cyan(doc: &Document, layer: usize, child: usize) {
    let group = &doc.layers[layer].children().unwrap()[child];
    let art = &group.children().unwrap()[0];
    assert!(matches!(art.kind, NodeKind::Path { .. }));
    assert!(matches!(art.appearance.items.first(), Some(vectorcraft_doc::AppearanceItem::Fill(fill))
        if fill.paint == vectorcraft_color::Paint::solid(vectorcraft_color::Color::cmyk(1.0, 0.0, 0.0, 0.0))));
}

fn replacement_has_two_halves(doc: &Document) {
    let group = &doc.layers[1].children().unwrap()[0];
    let art = group.children().unwrap();
    assert_eq!(art.len(), 2);
    assert_eq!(art[0].geometric_bounds().unwrap().x0, 50.0);
    assert_eq!(art[1].geometric_bounds().unwrap().x0, 52.0);
    assert!(matches!(art[0].appearance.items.first(), Some(vectorcraft_doc::AppearanceItem::Fill(fill))
        if fill.paint == vectorcraft_color::Paint::solid(vectorcraft_color::Color::cmyk(1.0, 0.0, 0.0, 0.0))));
    assert!(matches!(art[1].appearance.items.first(), Some(vectorcraft_doc::AppearanceItem::Fill(fill))
        if fill.paint == vectorcraft_color::Paint::solid(vectorcraft_color::Color::cmyk(0.0, 1.0, 0.0, 0.0))));
}

#[test]
fn top_level_symbol_uses_page_art_beside_live_path() {
    let symbol = format!("%AI14_BeginSymbol\n(Dot) 0 Ae\n{}\n%AI10_EndSymbol\n", square(0, 0));
    let instance = "/SymbolInstance :\n(Dot) /SymbolRef ,\n[1 0 0 1 50 10] /Matrix ,\n;";
    let art = native(&(symbol + &layer("Live", true, &square(10, 10)) + &layer("Mixed", true, &format!("{instance}\n{}", square(70, 10)))));
    let (doc, notes, read) = crate::layered_ai(&private(&art), pdf_three_squares(), vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    assert_eq!(names(&doc), ["Live", "Mixed"]);
    assert_eq!(kinds(&doc.layers[1]), ["group[1]", "path"]);
    replacement_is_cyan(&doc, 1, 0);
    assert!(notes.iter().any(|n| n.contains("symbols (1)")), "{notes:?}");
}

#[test]
fn group_with_only_symbol_instance_uses_page_art() {
    let symbol = format!("%AI14_BeginSymbol\n(Dot) 0 Ae\n{}\n%AI10_EndSymbol\n", square(0, 0));
    let instance = "/SymbolInstance :\n(Dot) /SymbolRef ,\n[1 0 0 1 50 10] /Matrix ,\n;";
    let art = native(&(symbol + &layer("Live", true, &square(10, 10)) + &layer("Mixed", true, &os1_group(instance))));
    let (doc, notes, read) = crate::layered_ai(&private(&art), pdf_two_squares(), vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    assert_eq!(names(&doc), ["Live", "Mixed"]);
    assert_eq!(kinds(&doc.layers[1]), ["group[1]"]);
    replacement_is_cyan(&doc, 1, 0);
    assert!(notes.iter().any(|n| n.contains("symbols (1)")), "{notes:?}");
}

#[test]
fn unread_group_uses_page_opacity_once() {
    let symbol = format!("%AI14_BeginSymbol\n(Dot) 0 Ae\n{}\n%AI10_EndSymbol\n", square(0, 0));
    let instance = "/SymbolInstance :\n(Dot) /SymbolRef ,\n[1 0 0 1 50 10] /Matrix ,\n;";
    let group = format!("0 Ae\nu\n{instance}\nU\n0 0.5 0 0 0 Xy\n0 0 Xd\n6 () XW");
    let art = native(&(symbol + &layer("Live", true, &square(10, 10)) + &layer("Mixed", true, &group)));
    let mut page = pdf_two_squares();
    let id = page.layers[0].children().unwrap()[1].id;
    page.node_mut(id).unwrap().opacity = 0.5;
    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    assert_eq!(names(&doc), ["Live", "Mixed"]);
    let replacement = &doc.layers[1].children().unwrap()[0];
    assert_eq!(replacement.opacity, 1.0);
    assert_eq!(replacement.children().unwrap()[0].opacity, 0.5);
    let rendered = vectorcraft_render::Renderer::new().render_region(&doc, vectorcraft_geom::Rect::new(50.0, 86.0, 54.0, 90.0), 4.0, false);
    let pixel = rendered.pixels.as_chunks::<4>().0.get(8 * rendered.width as usize + 8).unwrap();
    assert!((120..=136).contains(&pixel[3]), "the final document draws the group at 50% once: {pixel:?}");
}

#[test]
fn unread_group_with_unidentified_transparency_uses_page_document() {
    let symbol = format!("%AI14_BeginSymbol\n(Dot) 0 Ae\n{}\n%AI10_EndSymbol\n", square(0, 0));
    let instance = "/SymbolInstance :\n(Dot) /SymbolRef ,\n[1 0 0 1 50 10] /Matrix ,\n;";
    let group = format!("0 Ae\nu\n{instance}\nU\n0 0.5 0 0 0 Xy\n0 0 Xd\n6 () XW");
    let art = native(&(symbol + &layer("Live", true, &square(10, 10)) + &layer("Mixed", true, &group)));
    let (doc, notes, read) = crate::layered_ai(&private(&art), pdf_two_squares(), vec![], false, false, 1.0);
    assert!(!read, "{notes:?}");
    assert_eq!(names(&doc), ["Layer 1"]);
    assert!(notes.iter().any(|n| n.contains("appearance is not identifiable")), "{notes:?}");
}

#[test]
fn group_with_only_placed_file_without_preview_uses_page_art() {
    let placed = "%AI5_BeginPlace\n[1 0 0 1 0 0] 50 10 54 14 (missing) `\n~\n%AI5_EndPlace";
    let art = native(&(layer("Live", true, &square(10, 10)) + &layer("Mixed", true, &os1_group(placed))));
    let (doc, notes, read) = crate::layered_ai(&private(&art), pdf_two_squares(), vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    assert_eq!(names(&doc), ["Live", "Mixed"]);
    assert_eq!(kinds(&doc.layers[1]), ["group[1]"]);
    replacement_is_cyan(&doc, 1, 0);
    assert!(notes.iter().any(|n| n.contains("placed files (1)")), "{notes:?}");
}

/// A linked image whose pixels the editing data doesn't keep (`XF`) shows a placeholder where the
/// PDF part draws the image: the layers are compared with the PDF part there, and the file comes in
/// as its PDF part when they differ.
#[test]
fn a_linked_image_without_pixels_is_compared_with_the_pdf_part() {
    let linked = "%AI5_File:\n%AI5_BeginRaster\n(photo.tif) 0 XG\n[ 10 0 0 10 50 54 ] 0 0 4 4 4 4 8 3 0 0 0 0 XF\n%AI5_EndRaster";
    let art = native(&(layer("Live", true, &square(10, 10)) + &layer("Linked", true, linked)));
    let (doc, notes, read) = crate::layered_ai(&private(&art), pdf_two_squares(), vec![], false, false, 1.0);
    assert!(!read, "{notes:?}");
    assert_eq!(names(&doc), ["Layer 1"]);
}

#[test]
fn top_level_pattern_uses_page_art_beside_live_path() {
    let body = format!("(Absent) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p\n{}\n{}", square_path(50, 10), square(70, 10));
    let art = native(&(layer("Live", true, &square(10, 10)) + &layer("Mixed", true, &body)));
    let (doc, notes, read) = crate::layered_ai(&private(&art), pdf_three_squares(), vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    assert_eq!(names(&doc), ["Live", "Mixed"]);
    assert_eq!(kinds(&doc.layers[1]), ["group[1]", "path"]);
    replacement_is_cyan(&doc, 1, 0);
    assert!(notes.iter().any(|n| n.contains("pattern fills (1)")), "{notes:?}");
}

#[test]
fn pattern_set_in_plain_group_replaces_later_sibling_from_page_art() {
    let group = os1_group(&format!("(Absent) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p\n{}", square_path(50, 10)));
    let art = native(&(layer("Live", true, &square(10, 10)) + &layer("Mixed", true, &format!("{group}\n{}", square_path(70, 10)))));
    let (doc, notes, read) = crate::layered_ai(&private(&art), pdf_three_squares(), vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    assert_eq!(names(&doc), ["Live", "Mixed"]);
    assert_eq!(kinds(&doc.layers[1]), ["group[1]", "group[1]"]);
    replacement_is_cyan(&doc, 1, 0);
    assert!(notes.iter().any(|n| n.contains("pattern fills (2)")), "{notes:?}");
}

#[test]
fn top_level_placed_file_without_preview_uses_page_art() {
    let placed = "%AI5_BeginPlace\n[1 0 0 1 0 0] 50 10 54 14 (missing) `\n~\n%AI5_EndPlace";
    let art = native(&(layer("Live", true, &square(10, 10)) + &layer("Mixed", true, &format!("{placed}\n{}", square(70, 10)))));
    let (doc, notes, read) = crate::layered_ai(&private(&art), pdf_three_squares(), vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    assert_eq!(names(&doc), ["Live", "Mixed"]);
    assert_eq!(kinds(&doc.layers[1]), ["group[1]", "path"]);
    replacement_is_cyan(&doc, 1, 0);
    assert!(notes.iter().any(|n| n.contains("placed files (1)")), "{notes:?}");
}

#[test]
fn nested_previewless_placed_file_at_box_corner_falls_back() {
    let place = "%AI5_BeginPlace\n[1 0 0 -1 -7970 8070] 8000 8000 8040 8040 4 4 0\n(missing.png)`\n~\n%AI5_EndPlace";
    let art = native(&layer("Placed", true, &os1_group(place))).replace("%AI5_NumLayers", "%AI3_TemplateBox: 100 50 100 50\n%AI5_NumLayers");
    let mut page = Document::new(100.0, 100.0);
    let id = page.alloc_id();
    let path = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(30.0, 30.0, 70.0, 70.0));
    let fill = vectorcraft_doc::FillLayer::new(vectorcraft_color::Paint::solid(vectorcraft_color::Color::gray(0.5)));
    page.insert(
        Some(page.layers[0].id),
        0,
        Node::path(id, path, vectorcraft_doc::Appearance { items: vec![vectorcraft_doc::AppearanceItem::Fill(fill)], ..Default::default() }),
    )
    .unwrap();
    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(!read, "a nested placeholder has no group-wide corner alternative: {notes:?}");
    assert_eq!(names(&doc), ["Layer 1"]);
}

#[test]
fn group_with_previewless_placed_file_and_readable_sibling_keeps_sibling() {
    let place = "%AI5_BeginPlace\n[1 0 0 -1 -7970 8070] 8000 8000 8040 8040 4 4 0\n(missing.png)`\n~\n%AI5_EndPlace";
    let group = os1_group(&format!("{place}\n{}", square(70, 10)));
    let art = native(&layer("Mixed", true, &group)).replace("%AI5_NumLayers", "%AI3_TemplateBox: 100 50 100 50\n%AI5_NumLayers");
    let mut page = pdf_part(70.0, 86.0);
    let id = page.alloc_id();
    let path = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(30.0, 30.0, 70.0, 70.0));
    let fill = vectorcraft_doc::FillLayer::new(vectorcraft_color::Paint::solid(vectorcraft_color::Color::gray(0.5)));
    page.insert(
        Some(page.layers[0].id),
        1,
        Node::path(id, path, vectorcraft_doc::Appearance { items: vec![vectorcraft_doc::AppearanceItem::Fill(fill)], ..Default::default() }),
    )
    .unwrap();
    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    if read {
        let mixed = doc.layers.iter().find(|layer| layer.name.as_deref() == Some("Mixed")).unwrap();
        assert!(
            mixed.children().unwrap().iter().any(|child| {
                child.children().is_some_and(|children| children.iter().any(|n| n.geometric_bounds().is_some_and(|b| b.x0 == 70.0)))
            }),
            "readable sibling was dropped: {notes:?}"
        );
    } else {
        assert_eq!(names(&doc), ["Layer 1"], "{notes:?}");
    }
}

#[test]
fn previewless_placed_file_tries_corner_when_first_box_contains_live_art() {
    let place = "%AI5_BeginPlace\n[1 0 0 -1 -7970 8070] 8000 8000 8040 8040 4 4 0\n(missing.png)`\n~\n%AI5_EndPlace";
    let live = "0 0 1 0 k\n130 -180 m\n134 -180 L\n134 -184 L\n130 -184 L\nf";
    let art = native(&(layer("Live", true, live) + &layer("Placed", true, place)))
        .replace("%AI5_NumLayers", "%AI3_TemplateBox: 100 50 100 50\n%AI5_NumLayers");
    let mut page = Document::new(100.0, 100.0);
    let id = page.alloc_id();
    let path = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(30.0, 30.0, 70.0, 70.0));
    let fill = vectorcraft_doc::FillLayer::new(vectorcraft_color::Paint::solid(vectorcraft_color::Color::gray(0.5)));
    page.insert(
        Some(page.layers[0].id),
        0,
        Node::path(id, path, vectorcraft_doc::Appearance { items: vec![vectorcraft_doc::AppearanceItem::Fill(fill)], ..Default::default() }),
    )
    .unwrap();
    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    assert_eq!(names(&doc), ["Live", "Placed"]);
    assert_eq!(kinds(&doc.layers[0]), ["path"]);
    assert_eq!(kinds(&doc.layers[1]), ["group[1]"]);
    assert!(notes.iter().any(|note| note.contains("placed files (1)")), "{notes:?}");
}

#[test]
fn replaced_page_art_reids_opacity_mask_art() {
    let place = "%AI5_BeginPlace\n[1 0 0 1 0 0] 50 10 54 14 (missing) `\n~\n%AI5_EndPlace";
    let art = native(&layer("Placed", true, place));
    let mut page = pdf_part(50.0, 86.0);
    let page_node_id = page.layers[0].children().unwrap()[0].id;
    let mask_id = page.alloc_id();
    let mask_path = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(50.0, 86.0, 54.0, 90.0));
    let mask_fill = vectorcraft_doc::FillLayer::new(vectorcraft_color::Paint::solid(vectorcraft_color::Color::WHITE));
    let mask_art = Node::path(
        mask_id,
        mask_path,
        vectorcraft_doc::Appearance { items: vec![vectorcraft_doc::AppearanceItem::Fill(mask_fill)], ..Default::default() },
    );
    page.node_mut(page_node_id).unwrap().extras_mut().mask = Some(Box::new(vectorcraft_doc::OpacityMask::new(mask_art, true)));
    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    let replacement = &doc.layers[0].children().unwrap()[0].children().unwrap()[0];
    let mask = replacement.extras().mask.as_ref().unwrap();
    assert_ne!(mask.art.id, mask_id);
    assert_ne!(mask.art.id, replacement.id);
    assert!(mask.art.id.0 < doc.peek_next_id());
}

#[test]
fn top_level_placed_file_replacement_keeps_page_image_data() {
    use vectorcraft_doc::{ImageBlob, ImageObject};

    let placed = "%AI5_BeginPlace\n[1 0 0 1 0 0] 50 10 54 14 (missing) `\n~\n%AI5_EndPlace";
    let art = native(&(layer("Live", true, &square(10, 10)) + &layer("Mixed", true, placed)));
    let mut page = pdf_part(10.0, 86.0);
    let pixels = image::RgbaImage::from_pixel(4, 4, image::Rgba([0, 255, 255, 255]));
    let mut png = Vec::new();
    {
        let __dyn: image::DynamicImage = pixels.into();
        let (__w, __h) = (__dyn.width(), __dyn.height());
        let (__layout, __sample, __raw) = match __dyn {
            image::DynamicImage::ImageLuma8(b) => (craft_image_codecs::ChannelLayout::Gray, craft_image_codecs::SampleType::U8, b.into_raw()),
            image::DynamicImage::ImageLumaA8(b) => (craft_image_codecs::ChannelLayout::GrayA, craft_image_codecs::SampleType::U8, b.into_raw()),
            image::DynamicImage::ImageRgb8(b) => (craft_image_codecs::ChannelLayout::Rgb, craft_image_codecs::SampleType::U8, b.into_raw()),
            image::DynamicImage::ImageRgba8(b) => (craft_image_codecs::ChannelLayout::Rgba, craft_image_codecs::SampleType::U8, b.into_raw()),
            image::DynamicImage::ImageLuma16(b) => (
                craft_image_codecs::ChannelLayout::Gray,
                craft_image_codecs::SampleType::U16,
                b.into_raw().into_iter().flat_map(|s| s.to_ne_bytes()).collect(),
            ),
            image::DynamicImage::ImageLumaA16(b) => (
                craft_image_codecs::ChannelLayout::GrayA,
                craft_image_codecs::SampleType::U16,
                b.into_raw().into_iter().flat_map(|s| s.to_ne_bytes()).collect(),
            ),
            image::DynamicImage::ImageRgb16(b) => (
                craft_image_codecs::ChannelLayout::Rgb,
                craft_image_codecs::SampleType::U16,
                b.into_raw().into_iter().flat_map(|s| s.to_ne_bytes()).collect(),
            ),
            image::DynamicImage::ImageRgba16(b) => (
                craft_image_codecs::ChannelLayout::Rgba,
                craft_image_codecs::SampleType::U16,
                b.into_raw().into_iter().flat_map(|s| s.to_ne_bytes()).collect(),
            ),
            other => (craft_image_codecs::ChannelLayout::Rgba, craft_image_codecs::SampleType::U8, other.into_rgba8().into_raw()),
        };
        let __cimg = craft_image_codecs::Image::from_raw(__w, __h, __layout, __sample, __raw).unwrap();
        craft_image_codecs::encode(&__cimg, craft_image_codecs::Format::Png, &craft_image_codecs::EncodeOptions::default())
            .map(|__b| png.extend_from_slice(&__b))
    }
    .unwrap();
    let blob = ImageBlob::new("image/png", png);
    let key = blob.content_key();
    let expected = blob.bytes.clone();
    page.images.insert(key.clone(), blob);
    let image = Node::new(
        page.alloc_id(),
        NodeKind::Image(Box::new(ImageObject {
            key: key.clone(),
            width: 4,
            height: 4,
            xf: vectorcraft_geom::Affine::translate(vectorcraft_geom::Vec2::new(50.0, 86.0)),
            link: None,
            placement: Default::default(),
            pixels: None,
        })),
    );
    page.insert(Some(page.layers[0].id), 1, image).unwrap();

    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    assert_eq!(names(&doc), ["Live", "Mixed"]);
    let replacement = &doc.layers[1].children().unwrap()[0];
    let [image] = replacement.children().unwrap().as_slice() else { panic!("{replacement:?}") };
    let NodeKind::Image(im) = &image.kind else { panic!("{image:?}") };
    assert!(im.pixels.is_none(), "PDF-imported images use document data");
    assert_eq!(doc.images.get(&im.key).map(|blob| &blob.bytes), Some(&expected));
    assert!(notes.iter().any(|note| note.contains("placed files (1)")), "{notes:?}");
}

#[test]
fn top_level_pattern_replacement_keeps_gradient_stop_swatch() {
    use vectorcraft_color::gradient::{GradientKind, GradientPaint, GradientStop};
    use vectorcraft_color::{Color, Gradient, Paint, Swatch};

    let body = format!("(Absent) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p\n{}", square_path(50, 10));
    let art = native(&(layer("Live", true, &square(10, 10)) + &layer("Mixed", true, &body)));
    let mut page = pdf_two_squares();
    let cyan = Color::cmyk(1.0, 0.0, 0.0, 0.0);
    page.swatches.push(Swatch { name: "Linked Cyan".into(), paint: Paint::solid(cyan), global: true, spot: false });
    let mut stop = GradientStop::new(0.0, cyan);
    stop.swatch = Some("Linked Cyan".into());
    let gradient = Gradient::new(GradientKind::Linear, vec![stop, GradientStop::new(1.0, cyan)]);
    let id = page.layers[0].children().unwrap()[1].id;
    let node = page.node_mut(id).unwrap();
    node.appearance = vectorcraft_doc::Appearance {
        items: vec![vectorcraft_doc::AppearanceItem::Fill(vectorcraft_doc::FillLayer::new(Paint::Gradient(Box::new(GradientPaint::new(gradient)))))],
        ..Default::default()
    }
    .into();

    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    let group = &doc.layers[1].children().unwrap()[0];
    let replacement = &group.children().unwrap()[0];
    assert!(matches!(replacement.appearance.fill().map(|f| &f.paint), Some(Paint::Gradient(g))
        if g.gradient.stops[0].swatch.as_deref() == Some("Linked Cyan")));
    assert!(matches!(doc.swatch("Linked Cyan"), Some(s) if s.global && s.paint == Paint::solid(cyan)));
}

/// Two independently imported copies of the same tile have different node IDs.
fn page_with_named_pattern(native_art: &str, different: bool) -> Document {
    use vectorcraft_color::Paint;

    let (editing, _) = crate::ai_alone(&private(&native_art.replace("(Absent)", "(Dots)"))).unwrap();
    let source = editing.pattern("Dots").unwrap();
    let mut page = pdf_two_squares();
    let mut pattern = source.clone();
    pattern.art = source.art.iter().map(|node| std::sync::Arc::new(page.reid(node))).collect();
    if different {
        pattern.tile.x1 += 1.0;
    }
    page.patterns.push(pattern);
    let id = page.layers[0].children().unwrap()[1].id;
    let node = page.node_mut(id).unwrap();
    node.appearance = vectorcraft_doc::Appearance {
        items: vec![vectorcraft_doc::AppearanceItem::Fill(vectorcraft_doc::FillLayer::new(Paint::Pattern {
            pattern: "Dots".into(),
            xf: vectorcraft_geom::Affine::IDENTITY,
        }))],
        ..Default::default()
    }
    .into();
    page
}

fn named_pattern_replacement_art() -> String {
    let dots = "%AI3_BeginPattern: (Dots)\n(Dots) 0 0 4 4\n%_0 0 0 1 k\n%_1 1 m\n%_3 1 L\n%_3 3 L\n%_1 3 L\n%_f\nE\n%AI3_EndPattern\n";
    let unread = format!("(Absent) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p\n{}", square_path(50, 10));
    native(&format!("{dots}{}{}", layer("Live", true, &square(10, 10)), layer("Mixed", true, &unread)))
}

#[test]
fn same_named_pattern_in_page_art_keeps_editing_layers() {
    let art = named_pattern_replacement_art();
    let page = page_with_named_pattern(&art, false);
    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    assert_eq!(names(&doc), ["Live", "Mixed"]);
    assert_eq!(doc.patterns.iter().filter(|p| p.name == "Dots").count(), 1);
    let replacement = &doc.layers[1].children().unwrap()[0].children().unwrap()[0];
    assert!(matches!(replacement.appearance.fill().map(|f| &f.paint), Some(vectorcraft_color::Paint::Pattern { pattern, .. }) if pattern == "Dots"));
}

#[test]
fn copied_page_pattern_reids_its_mask_art() {
    let source_art = named_pattern_replacement_art();
    let mut page = page_with_named_pattern(&source_art, false);
    let mask_id = page.alloc_id();
    let path = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(0.0, 0.0, 4.0, 4.0));
    let fill = vectorcraft_doc::FillLayer::new(vectorcraft_color::Paint::solid(vectorcraft_color::Color::WHITE));
    let mask_art =
        Node::path(mask_id, path, vectorcraft_doc::Appearance { items: vec![vectorcraft_doc::AppearanceItem::Fill(fill)], ..Default::default() });
    std::sync::Arc::make_mut(&mut page.patterns[0].art[0]).extras_mut().mask = Some(Box::new(vectorcraft_doc::OpacityMask::new(mask_art, true)));
    let art = native(&format!(
        "{}{}",
        layer("Live", true, &square(10, 10)),
        layer("Mixed", true, &format!("(Absent) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p\n{}", square_path(50, 10)))
    ));
    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    let mask = doc.pattern("Dots").unwrap().art[0].extras().mask.as_ref().unwrap();
    assert_ne!(mask.art.id, mask_id);
    assert!(mask.art.id.0 < doc.peek_next_id());
}

#[test]
fn different_named_pattern_in_page_art_falls_back() {
    let art = named_pattern_replacement_art();
    let page = page_with_named_pattern(&art, true);
    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(!read, "{notes:?}");
    assert_eq!(names(&doc), ["Layer 1"]);
    assert!(notes.iter().any(|n| n.contains("page art conflicts with pattern Dots")), "{notes:?}");
}

#[test]
fn partly_overlapping_page_art_does_not_replace_an_object() {
    let art = native(&(layer("Live", true, &square(10, 10)) + &layer("Mixed", true, &os1_group(&format!("{}\nZZ", square(50, 10))))));
    let mut page = pdf_part(10.0, 86.0);
    let other = pdf_part(48.0, 86.0);
    let id = page.layers[0].id;
    let square = page.reid(&other.layers[0].children().unwrap()[0]);
    page.insert(Some(id), 1, square).unwrap();
    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(!read, "{notes:?}");
    assert_eq!(names(&doc), ["Layer 1"]);
    assert!(notes.iter().any(|n| n.contains("page art wasn't found")), "{notes:?}");
}

/// Page art for the unread square and its neighbor under one PDF group.
fn grouped_page_across_unread_and_neighbor(clipped: bool) -> Document {
    let mut page = pdf_two_squares();
    let layer = page.layers[0].id;
    let unread = page.remove(page.layers[0].children().unwrap()[1].id).unwrap();
    let other = pdf_part(70.0, 86.0);
    let neighbor = page.reid(&other.layers[0].children().unwrap()[0]);
    let mut children = Vec::new();
    if clipped {
        let path = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(50.0, 86.0, 74.0, 90.0));
        let mut clip = Node::path(page.alloc_id(), path, vectorcraft_doc::Appearance::default());
        if let NodeKind::Path { clipping, .. } = &mut clip.kind {
            *clipping = true;
        }
        children.push(std::sync::Arc::new(clip));
    }
    children.extend([unread, std::sync::Arc::new(neighbor)]);
    let mut group = Node::group(page.alloc_id(), children);
    group.set_clips(clipped);
    page.insert(Some(layer), 1, group).unwrap();
    page
}

#[test]
fn clipped_pdf_group_across_unread_and_neighbor_falls_back() {
    let body = format!("{}\n{}", os1_group(&format!("{}\nZZ", square(50, 10))), square(70, 10));
    let art = native(&(layer("Live", true, &square(10, 10)) + &layer("Mixed", true, &body)));
    let (doc, notes, read) = crate::layered_ai(&private(&art), grouped_page_across_unread_and_neighbor(true), vec![], false, false, 1.0);
    assert!(!read, "{notes:?}");
    assert_eq!(names(&doc), ["Layer 1"]);
    assert!(notes.iter().any(|n| n.contains("clipping or appearance ancestor")), "{notes:?}");
}

#[test]
fn plain_pdf_group_across_unread_and_neighbor_uses_child() {
    let body = format!("{}\n{}", os1_group(&format!("{}\nZZ", square(50, 10))), square(70, 10));
    let art = native(&(layer("Live", true, &square(10, 10)) + &layer("Mixed", true, &body)));
    let (doc, notes, read) = crate::layered_ai(&private(&art), grouped_page_across_unread_and_neighbor(false), vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    assert_eq!(names(&doc), ["Live", "Mixed"]);
    replacement_is_cyan(&doc, 1, 0);
    assert!(notes.iter().any(|n| n.contains("`ZZ` (1)")), "{notes:?}");
}

#[test]
fn live_art_inside_unread_bounds_falls_back_instead_of_drawing_twice() {
    let body = format!("{}\n{}", os1_group(&format!("{}\nZZ", square(50, 10))), "0 0 1 0 k\n51 11 m\n53 11 L\n53 13 L\n51 13 L\nf");
    let art = native(&(layer("Live", true, &square(10, 10)) + &layer("Mixed", true, &body)));
    let mut page = pdf_two_squares();
    let inner = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(51.0, 87.0, 53.0, 89.0));
    let appearance = (*page.layers[0].children().unwrap()[0].appearance).clone();
    let node = Node::path(page.alloc_id(), inner, appearance);
    let layer_id = page.layers[0].id;
    page.insert(Some(layer_id), 2, node).unwrap();
    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(!read, "{notes:?}");
    assert_eq!(names(&doc), ["Layer 1"]);
    assert!(notes.iter().any(|n| n.contains("bounds also contain live art")), "{notes:?}");
}

#[test]
fn live_art_crossing_unread_bounds_by_half_a_point_falls_back() {
    let neighbor = "0 0 1 0 k\n53.5 11 m\n54.5 11 L\n54.5 13 L\n53.5 13 L\nf";
    let art = native(&layer("Mixed", true, &format!("{}\n{neighbor}", os1_group(&format!("{}\nZZ", square(50, 10))))));
    let mut page = pdf_part(50.0, 86.0);
    let path = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(53.5, 87.0, 54.5, 89.0));
    let appearance = (*page.layers[0].children().unwrap()[0].appearance).clone();
    let node = Node::path(page.alloc_id(), path, appearance);
    let layer = page.layers[0].id;
    page.insert(Some(layer), 1, node).unwrap();
    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(!read, "{notes:?}");
    assert_eq!(names(&doc), ["Layer 1"]);
    assert!(notes.iter().any(|n| n.contains("bounds also contain live art")), "{notes:?}");
}

#[test]
fn overlapping_unread_objects_cannot_claim_the_same_page_art() {
    let first = os1_group(&format!("{}\nZZ", square(50, 10)));
    let second = os1_group(&format!("{}\nZZ", square(52, 10)));
    let art = native(&layer("Mixed", true, &format!("{first}\n{second}")));
    let mut page = pdf_part(52.0, 86.0);
    let path = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(52.0, 86.0, 54.0, 90.0));
    let node = page.layers[0].children().unwrap()[0].as_ref().clone();
    let layer = page.layers[0].id;
    page.remove(node.id).unwrap();
    let replacement = Node::path(page.alloc_id(), path, (*node.appearance).clone());
    page.insert(Some(layer), 0, replacement).unwrap();
    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(!read, "{notes:?}");
    assert_eq!(names(&doc), ["Layer 1"]);
    assert!(notes.iter().any(|n| n.contains("more than one unread object")), "{notes:?}");
}

#[test]
fn top_level_pattern_without_page_art_falls_back() {
    let body = format!("(Absent) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p\n{}", square_path(50, 10));
    let art = native(&(layer("Live", true, &square(10, 10)) + &layer("Mixed", true, &body)));
    let (doc, notes, read) = crate::layered_ai(&private(&art), pdf_part(10.0, 86.0), vec![], false, false, 1.0);
    assert!(!read, "{notes:?}");
    assert_eq!(names(&doc), ["Layer 1"]);
    assert!(notes.iter().any(|n| n.contains("page art wasn't found")), "{notes:?}");
}

#[test]
fn top_level_placed_file_without_page_art_falls_back() {
    let placed = "%AI5_BeginPlace\n[1 0 0 1 0 0] 50 10 54 14 (missing) `\n~\n%AI5_EndPlace";
    let art = native(&(layer("Live", true, &square(10, 10)) + &layer("Mixed", true, placed)));
    let (doc, notes, read) = crate::layered_ai(&private(&art), pdf_part(10.0, 86.0), vec![], false, false, 1.0);
    assert!(!read, "{notes:?}");
    assert_eq!(names(&doc), ["Layer 1"]);
    assert!(notes.iter().any(|n| n.contains("page art wasn't found")), "{notes:?}");
}

const BLACK: &str = "0 0 0 1";
const YELLOW: &str = "0 0 1 0";
const PAGE_CYAN: [f32; 4] = [1.0, 0.0, 0.0, 0.0];
const PAGE_YELLOW: [f32; 4] = [0.0, 0.0, 1.0, 0.0];

/// A square of 16 points at (`x`, 10) in the format's operators, painted in CMYK `cmyk`.
fn square_16(x: u32, cmyk: &str) -> String {
    format!("{cmyk} k\n{x} 10 m\n{} 10 L\n{} 26 L\n{x} 26 L\nf", x + 16, x + 16)
}

/// A 100 × 100 page with a square of 16 points at each (`x`, `y`, CMYK) of `squares`: at `y` 74
/// where [`square_16`] puts the layers' squares.
fn page_16(squares: &[(f64, f64, [f32; 4])]) -> Document {
    let mut d = Document::new(100.0, 100.0);
    let layer = d.layers[0].id;
    for (i, (x, y, [c, m, ye, k])) in squares.iter().enumerate() {
        let path = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(*x, *y, x + 16.0, y + 16.0));
        let paint = vectorcraft_color::Paint::solid(vectorcraft_color::Color::cmyk(*c, *m, *ye, *k));
        let item = vectorcraft_doc::AppearanceItem::Fill(vectorcraft_doc::FillLayer::new(paint));
        let n = Node::path(d.alloc_id(), path, vectorcraft_doc::Appearance { items: vec![item], ..Default::default() });
        d.insert(Some(layer), i, n).unwrap();
    }
    d
}

/// Three squares on two layers, the middle one black in the layers and cyan on the page.
fn middle_differs() -> (String, Document) {
    let front = format!("{}\n{}", square_16(40, BLACK), square_16(76, YELLOW));
    let art = native(&(layer("Back", true, &square_16(4, YELLOW)) + &layer("Front", true, &front)));
    (art, page_16(&[(4.0, 74.0, PAGE_YELLOW), (40.0, 74.0, PAGE_CYAN), (76.0, 74.0, PAGE_YELLOW)]))
}

#[test]
fn an_object_that_draws_differently_comes_in_as_the_page_art() {
    let (art, page) = middle_differs();
    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    assert_eq!(names(&doc), ["Back", "Front"]);
    assert_eq!(kinds(&doc.layers[0]), ["path"]);
    assert_eq!(kinds(&doc.layers[1]), ["group[1]", "path"]);
    replacement_is_cyan(&doc, 1, 0);
    let note = "objects that drew differently from the file's own page came in as its art: Front (1)";
    assert!(notes.iter().any(|n| n == note && crate::is_loss(n)), "{notes:?}");
    assert!(!notes.iter().any(|n| n.contains("weren't read")), "{notes:?}");
}

#[test]
fn two_differing_objects_on_different_layers_are_both_replaced() {
    let back = format!("{}\n{}", square_16(4, BLACK), square_16(28, YELLOW));
    let front = format!("{}\n{}", square_16(52, YELLOW), square_16(76, BLACK));
    let art = native(&(layer("Back", true, &back) + &layer("Front", true, &front)));
    let page = page_16(&[(4.0, 74.0, PAGE_CYAN), (28.0, 74.0, PAGE_YELLOW), (52.0, 74.0, PAGE_YELLOW), (76.0, 74.0, PAGE_CYAN)]);
    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    assert_eq!(names(&doc), ["Back", "Front"]);
    assert_eq!(kinds(&doc.layers[0]), ["group[1]", "path"]);
    assert_eq!(kinds(&doc.layers[1]), ["path", "group[1]"]);
    replacement_is_cyan(&doc, 0, 0);
    replacement_is_cyan(&doc, 1, 1);
    let note = "objects that drew differently from the file's own page came in as its art: Back (1), Front (1)";
    assert!(notes.iter().any(|n| n == note), "{notes:?}");
}

/// The text wrap options of a 2-point offset, for the object before them.
const WRAP_OPTIONS: &str = "%_/ArtDictionary :\n%_0 /Bool (AI11TextWrapInvert) ,\n%_2 /Real (AI11TextWrapOffset) ,\n%_;\n%_\n";

#[test]
fn a_differing_wrap_object_keeps_its_wrap_options() {
    let front = format!("{}\n{WRAP_OPTIONS}{}", square_16(40, BLACK), square_16(76, YELLOW));
    let art = native(&(layer("Back", true, &square_16(4, YELLOW)) + &layer("Front", true, &front)));
    let page = page_16(&[(4.0, 74.0, PAGE_YELLOW), (40.0, 74.0, PAGE_CYAN), (76.0, 74.0, PAGE_YELLOW)]);
    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    assert_eq!(kinds(&doc.layers[1]), ["group[1]", "path"]);
    replacement_is_cyan(&doc, 1, 0);
    let group = &doc.layers[1].children().unwrap()[0];
    assert_eq!(group.extras().wrap, Some(vectorcraft_doc::TextWrap { offset: 2.0, invert: false }));
}

#[test]
fn a_differing_group_holding_a_wrap_object_still_falls_back() {
    let group = os1_group(&format!("{}\n{WRAP_OPTIONS}", square_16(40, BLACK)));
    let front = format!("{group}\n{}", square_16(76, YELLOW));
    let art = native(&(layer("Back", true, &square_16(4, YELLOW)) + &layer("Front", true, &front)));
    let page = page_16(&[(4.0, 74.0, PAGE_YELLOW), (40.0, 74.0, PAGE_CYAN), (76.0, 74.0, PAGE_YELLOW)]);
    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(!read, "{notes:?}");
    assert_eq!(names(&doc), ["Layer 1"]);
    assert!(!notes.iter().any(|n| n.starts_with("objects that drew")), "{notes:?}");
}

#[test]
fn a_page_that_differs_everywhere_still_comes_in_as_its_pdf_part() {
    let front = format!("{}\n{}", square_16(40, BLACK), square_16(76, BLACK));
    let art = native(&(layer("Back", true, &square_16(4, BLACK)) + &layer("Front", true, &front)));
    let page = page_16(&[(4.0, 74.0, PAGE_CYAN), (40.0, 74.0, PAGE_CYAN), (76.0, 74.0, PAGE_CYAN)]);
    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(!read, "{notes:?}");
    assert_eq!(names(&doc), ["Layer 1"]);
    assert!(notes.iter().any(|n| n.contains("weren't read") && n.contains("differs from the page's")), "{notes:?}");
    assert!(!notes.iter().any(|n| n.starts_with("objects that drew")), "{notes:?}");
}

#[test]
fn a_page_object_outside_every_layer_object_still_falls_back() {
    let front = format!("{}\n{}", square_16(40, YELLOW), square_16(76, YELLOW));
    let art = native(&(layer("Back", true, &square_16(4, YELLOW)) + &layer("Front", true, &front)));
    let page = page_16(&[(4.0, 74.0, PAGE_YELLOW), (40.0, 74.0, PAGE_YELLOW), (76.0, 74.0, PAGE_YELLOW), (40.0, 20.0, PAGE_CYAN)]);
    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(!read, "{notes:?}");
    assert_eq!(names(&doc), ["Layer 1"]);
    assert!(notes.iter().any(|n| n.contains("differs from the page's")), "{notes:?}");
}

#[test]
fn a_differing_object_with_live_art_inside_falls_back() {
    // The group reaches over the square between its own two.
    let group = os1_group(&format!("{}\n{}", square_16(4, BLACK), square_16(40, YELLOW)));
    let front = format!("{group}\n{}\n{}", square_16(22, YELLOW), square_16(76, YELLOW));
    let art = native(&layer("Front", true, &front));
    let page = page_16(&[(4.0, 74.0, PAGE_CYAN), (22.0, 74.0, PAGE_YELLOW), (40.0, 74.0, PAGE_YELLOW), (76.0, 74.0, PAGE_YELLOW)]);
    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(!read, "{notes:?}");
    assert_eq!(names(&doc), ["Layer 1"]);
    assert!(notes.iter().any(|n| n.contains("differs from the page's")), "{notes:?}");
}

#[test]
fn trusted_editing_data_keeps_a_differing_object() {
    let (art, page) = middle_differs();
    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, true, 1.0);
    assert!(read, "{notes:?}");
    assert_eq!(names(&doc), ["Back", "Front"]);
    assert_eq!(kinds(&doc.layers[1]), ["path", "path"]);
    assert!(notes.iter().any(|n| n.ends_with("they come from its editing data all the same")), "{notes:?}");
    assert!(!notes.iter().any(|n| n.starts_with("objects that drew")), "{notes:?}");
}

#[test]
fn a_differing_object_on_an_unnamed_layer_is_counted_under_its_panel_name() {
    let front = format!("{}\n{}", square_16(40, BLACK), square_16(76, YELLOW));
    let art = native(&(layer("Back", true, &square_16(4, YELLOW)) + &layer("", true, &front)));
    let page = page_16(&[(4.0, 74.0, PAGE_YELLOW), (40.0, 74.0, PAGE_CYAN), (76.0, 74.0, PAGE_YELLOW)]);
    let (_, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    let note = "objects that drew differently from the file's own page came in as its art: <Layer> (1)";
    assert!(notes.iter().any(|n| n == note), "{notes:?}");
}

#[test]
fn at_most_64_differing_objects_come_in_as_the_page_art() {
    // 130 squares of 4 points, the first `black` of them black in the layers and cyan on the page.
    let grid = |black: usize| {
        let at = |i: usize| (4 + 7 * (i % 13) as u32, 4 + 7 * (i / 13) as u32);
        let art: String = (0..130)
            .map(|i| {
                let (x, y) = at(i);
                if i < black { format!("{BLACK} k\n{}", square_path(x, y)) } else { square(x, y) }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let mut page = Document::new(100.0, 100.0);
        let layer_id = page.layers[0].id;
        for i in 0..130 {
            let (x, y) = at(i);
            let (x, y) = (f64::from(x), 96.0 - f64::from(y));
            let path = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(x, y, x + 4.0, y + 4.0));
            let color =
                if i < black { vectorcraft_color::Color::cmyk(1.0, 0.0, 0.0, 0.0) } else { vectorcraft_color::Color::cmyk(0.0, 0.0, 1.0, 0.0) };
            let item = vectorcraft_doc::AppearanceItem::Fill(vectorcraft_doc::FillLayer::new(vectorcraft_color::Paint::solid(color)));
            let n = Node::path(page.alloc_id(), path, vectorcraft_doc::Appearance { items: vec![item], ..Default::default() });
            page.insert(Some(layer_id), i, n).unwrap();
        }
        crate::layered_ai(&private(&native(&layer("Grid", true, &art))), page, vec![], false, false, 1.0)
    };
    let (_, notes, read) = grid(64);
    assert!(read, "{notes:?}");
    let note = "objects that drew differently from the file's own page came in as its art: Grid (64)";
    assert!(notes.iter().any(|n| n == note), "{notes:?}");
    let (doc, notes, read) = grid(65);
    assert!(!read, "{notes:?}");
    assert_eq!(names(&doc), ["Layer 1"]);
    assert!(!notes.iter().any(|n| n.starts_with("objects that drew")), "{notes:?}");
}

#[test]
fn a_differing_object_under_the_type_of_a_text_object_that_stays_falls_back() {
    // A black rectangle under area type whose frame reaches past it; the page draws the rectangle
    // cyan, with the type on it.
    let big = format!("{BLACK} k\n10 2 m\n60 2 L\n60 40 L\n10 40 L\nf");
    let doc = document_of(&[area_frame(34.0, 26.0, 95.0, 38.0)], &[story("Hi\\r", &[0], "/0 0", "/1 8.0")]);
    let art = native_with_text(&(layer("Art", true, &format!("{big}\n{}", frame_object(0, 0))) + &layer("Top", true, &square_16(76, YELLOW))), &doc);
    let mut page = page_16(&[(76.0, 74.0, PAGE_YELLOW)]);
    let layer_id = page.layers[0].id;
    let path = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(10.0, 60.0, 60.0, 98.0));
    let paint = vectorcraft_color::Paint::solid(vectorcraft_color::Color::cmyk(1.0, 0.0, 0.0, 0.0));
    let item = vectorcraft_doc::AppearanceItem::Fill(vectorcraft_doc::FillLayer::new(paint));
    let square = Node::path(page.alloc_id(), path, vectorcraft_doc::Appearance { items: vec![item], ..Default::default() });
    page.insert(Some(layer_id), 0, square).unwrap();
    let style = vectorcraft_doc::CharStyle { size: 8.0, ..Default::default() };
    let text = vectorcraft_doc::TextObject::point(vectorcraft_geom::Point::new(34.0, 69.0), "Hi", style);
    let text = Node::new(page.alloc_id(), NodeKind::Text(Box::new(text)));
    page.insert(Some(layer_id), 1, text).unwrap();
    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(!read, "{notes:?}");
    assert_eq!(names(&doc), ["Layer 1"]);
    assert!(!notes.iter().any(|n| n.starts_with("objects that drew")), "{notes:?}");
}

#[test]
fn unread_object_on_second_artboard_takes_second_page_art() {
    let boards = "%_/Document :\n%_/Dictionary :\n%_/Array :\n\
        %_/Dictionary :\n%_0 100 /RealPointRelToROrigin\n%_ (PositionPoint1) ,\n%_100 0 /RealPointRelToROrigin\n%_ (PositionPoint2) ,\n%_; ,\n\
        %_/Dictionary :\n%_0 -50 /RealPointRelToROrigin\n%_ (PositionPoint1) ,\n%_100 -150 /RealPointRelToROrigin\n%_ (PositionPoint2) ,\n%_; ,\n\
        %_; (ArtboardArray) ,\n%_; /NotRecorded ,\n%_;\n";
    let second = "u\n0 0 1 0 k\n50 -100 m\n54 -100 L\n54 -96 L\n50 -96 L\nf\nZZ\nU";
    let art = native(&(boards.to_string() + &layer("Live", true, &square(10, 10)) + &layer("Mixed", true, second)));
    let mut page = pdf_part(10.0, 86.0);
    let mut board = page.artboards[0].clone();
    (board.id, board.rect) = (2, vectorcraft_geom::Rect::new(120.0, 0.0, 220.0, 100.0));
    page.artboards.push(board);
    let path = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(170.0, 46.0, 174.0, 50.0));
    let paint = vectorcraft_color::Paint::solid(vectorcraft_color::Color::cmyk(1.0, 0.0, 0.0, 0.0));
    let appearance = vectorcraft_doc::Appearance {
        items: vec![vectorcraft_doc::AppearanceItem::Fill(vectorcraft_doc::FillLayer::new(paint))],
        ..Default::default()
    };
    let node = Node::path(page.alloc_id(), path, appearance);
    let layer = page.layers[0].id;
    page.insert(Some(layer), 1, node).unwrap();
    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    assert_eq!(names(&doc), ["Live", "Mixed"]);
    replacement_is_cyan(&doc, 1, 0);
    let bounds = doc.layers[1].children().unwrap()[0].visual_bounds().unwrap();
    assert_eq!(bounds, vectorcraft_geom::Rect::new(50.0, 196.0, 54.0, 200.0));
}

#[test]
fn page_art_from_second_artboard_moves_its_pattern_tile() {
    let boards = "%_/Document :\n%_/Dictionary :\n%_/Array :\n\
        %_/Dictionary :\n%_0 100 /RealPointRelToROrigin\n%_ (PositionPoint1) ,\n%_100 0 /RealPointRelToROrigin\n%_ (PositionPoint2) ,\n%_; ,\n\
        %_/Dictionary :\n%_0 -50 /RealPointRelToROrigin\n%_ (PositionPoint1) ,\n%_100 -150 /RealPointRelToROrigin\n%_ (PositionPoint2) ,\n%_; ,\n\
        %_; (ArtboardArray) ,\n%_; /NotRecorded ,\n%_;\n";
    let dots = "%AI3_BeginPattern: (Dots)\n(Dots) 0 0 4 4\n%_0 0 0 1 k\n%_1 1 m\n%_3 1 L\n%_3 3 L\n%_1 3 L\n%_f\nE\n%AI3_EndPattern\n";
    let second = "u\n0 0 1 0 k\n50 -100 m\n54 -100 L\n54 -96 L\n50 -96 L\nf\nZZ\nU";
    let art = native(&(boards.to_string() + dots + &layer("Live", true, &square(10, 10)) + &layer("Mixed", true, second)));
    let (editing, _) = crate::ai_alone(&private(&native(&format!("{dots}{}", layer("Live", true, &square(10, 10)))))).unwrap();
    let source = editing.pattern("Dots").unwrap();
    let mut page = pdf_part(10.0, 86.0);
    let mut pattern = source.clone();
    pattern.art = source.art.iter().map(|node| std::sync::Arc::new(page.reid(node))).collect();
    page.patterns.push(pattern);
    let mut board = page.artboards[0].clone();
    (board.id, board.rect) = (2, vectorcraft_geom::Rect::new(120.0, 0.0, 220.0, 100.0));
    page.artboards.push(board);
    let path = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(170.0, 46.0, 174.0, 50.0));
    let paint = vectorcraft_color::Paint::Pattern { pattern: "Dots".into(), xf: vectorcraft_geom::Affine::IDENTITY };
    let appearance = vectorcraft_doc::Appearance {
        items: vec![vectorcraft_doc::AppearanceItem::Fill(vectorcraft_doc::FillLayer::new(paint))],
        ..Default::default()
    };
    let node = Node::path(page.alloc_id(), path, appearance);
    let layer = page.layers[0].id;
    page.insert(Some(layer), 1, node).unwrap();
    let (doc, notes, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    let replacement = &doc.layers[1].children().unwrap()[0].children().unwrap()[0];
    assert_eq!(replacement.visual_bounds().unwrap(), vectorcraft_geom::Rect::new(50.0, 196.0, 54.0, 200.0));
    let Some(vectorcraft_color::Paint::Pattern { xf, .. }) = replacement.appearance.fill().map(|f| &f.paint) else {
        panic!("the page art's pattern fill is kept: {:?}", replacement.appearance);
    };
    assert_eq!(*xf, vectorcraft_geom::Affine::translate(vectorcraft_geom::Vec2::new(-120.0, 150.0)));
}

#[test]
fn unknown_operator_between_objects_falls_back() {
    let body = format!("{}\nZZ\n{}", square(50, 10), square(70, 10));
    let art = native(&(layer("Live", true, &square(10, 10)) + &layer("Mixed", true, &body)));
    let (doc, notes, read) = crate::layered_ai(&private(&art), pdf_three_squares(), vec![], false, false, 1.0);
    assert!(!read, "{notes:?}");
    assert_eq!(names(&doc), ["Layer 1"]);
    assert!(notes.iter().any(|n| n.contains("`ZZ`")), "{notes:?}");
}

#[test]
fn unread_object_without_page_art_names_what_is_unread() {
    let art = native(&layer("Mixed", true, &os1_group(&format!("{}\nZZ", square(50, 10)))));
    let e = crate::ai_alone(&private(&art)).unwrap_err();
    assert_eq!(e, "it has operators VectorCraft doesn't know (`ZZ`), which VectorCraft doesn't read from it yet");
    let pattern = os1_group(&format!("(Absent) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p\n{}", square_path(70, 10)));
    let art = native(&layer("Mixed", true, &format!("{}\n{pattern}", os1_group(&format!("{}\nZZ", square(50, 10))))));
    let e = crate::ai_alone(&private(&art)).unwrap_err();
    assert_eq!(e, "it has operators VectorCraft doesn't know (`ZZ`) and pattern fills, which VectorCraft doesn't read from it yet");
}

#[test]
fn unread_pattern_inside_one_object_uses_page_art_and_keeps_layers() {
    let art = native(
        &(layer("Live", true, &square(10, 10))
            + &layer(
                "Mixed",
                true,
                &format!("{}\n{}", os1_group(&format!("(Absent) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p\n{}", square_path(50, 10))), square(70, 10)),
            )),
    );
    let (doc, notes, read) = crate::layered_ai(&private(&art), pdf_three_squares(), vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    assert_eq!(names(&doc), ["Live", "Mixed"]);
    assert_eq!(kinds(&doc.layers[0]), ["path"]);
    assert_eq!(kinds(&doc.layers[1]), ["group[1]", "path"]);
    replacement_is_cyan(&doc, 1, 0);
    assert!(notes.iter().any(|n| n.contains("pattern fills (1)") && n.contains("page's art")), "{notes:?}");
}

#[test]
fn unknown_operator_inside_group_uses_page_art() {
    let art = native(&(layer("Live", true, &square(10, 10)) + &layer("Mixed", true, &os1_group(&format!("{}\nZZ", square(50, 10))))));
    let (doc, notes, read) = crate::layered_ai(&private(&art), pdf_split_square(), vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    assert_eq!(names(&doc), ["Live", "Mixed"]);
    assert_eq!(kinds(&doc.layers[1]), ["group[2]"]);
    replacement_has_two_halves(&doc);
    assert!(notes.iter().any(|n| n.contains("`ZZ` (1)")), "{notes:?}");
}

#[test]
fn unread_object_on_hidden_layer_stays_left_out() {
    let art = native(&(layer("Live", true, &square(10, 10)) + &layer("Hidden", false, &os1_group(&format!("{}\nZZ", square(50, 10))))));
    let (doc, notes, read) = crate::layered_ai(&private(&art), pdf_part(10.0, 86.0), vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    assert_eq!(names(&doc), ["Live", "Hidden"]);
    assert!(notes.iter().any(|n| n.contains("left out of them")), "{notes:?}");
}

#[test]
fn unread_object_without_page_art_falls_back() {
    let art = native(&(layer("Live", true, &square(10, 10)) + &layer("Mixed", true, &os1_group(&format!("{}\nZZ", square(50, 10))))));
    let (doc, notes, read) = crate::layered_ai(&private(&art), pdf_part(10.0, 86.0), vec![], false, false, 1.0);
    assert!(!read, "{notes:?}");
    assert_eq!(names(&doc), ["Layer 1"]);
    assert!(notes.iter().any(|n| n.contains("page art wasn't found")), "{notes:?}");
}

#[test]
fn unclosed_unread_group_falls_back() {
    let art = native(&(layer("Live", true, &square(10, 10)) + &layer("Mixed", true, &format!("u\n{}\nZZ", square(50, 10)))));
    let (doc, notes, read) = crate::layered_ai(&private(&art), pdf_two_squares(), vec![], false, false, 1.0);
    assert!(!read, "{notes:?}");
    assert_eq!(names(&doc), ["Layer 1"]);
    assert!(notes.iter().any(|n| n.contains("closes something it didn't open")), "{notes:?}");
}


/// The PDF part `d` as read in points from pages of `k` points a unit (their `UserUnit`): its
/// pages and art `k` times as large.
fn in_points(mut d: Document, k: f64) -> Document {
    for layer in &mut d.layers {
        std::sync::Arc::make_mut(layer).transform(vectorcraft_geom::Affine::scale(k), true);
    }
    for a in &mut d.artboards {
        a.rect = a.rect.scale_from_origin(k);
    }
    d
}

/// A 1-bit gray image of `w` × `h` pixels in the format's operators, its samples `rows` (binary,
/// rows padded to whole bytes, a set bit white), placed by the matrix `m`.
fn raster_1bit(m: &str, (w, h): (u32, u32), rows: &str) -> String {
    format!(
        "%AI5_File:\n%AI5_BeginRaster\n() 1 XG\n/DeviceGray XN\n[ {m} ] {w} {h} 0 Xh\n\
         [ {m} ] 0 0 {w} {h} {w} {h} 1 1 0 0 1 0 4 4 0 0\n%%BeginData: {}\rXI\n{rows}%%EndData\r\nXH\r\n%AI5_EndRaster\r\nN",
        rows.len()
    )
}

/// #1479: an image past 2^25 pixels keeps the layers, at full size; a 1-bit one is a 1-bit PNG of
/// its rows.
#[test]
fn an_ai_file_with_an_image_past_2_25_pixels_keeps_its_layers() {
    // 6,000 × 6,000 pixels, each row black and white in turn ('U' is 01010101).
    let image = raster_1bit("0.0005 0 0 0.0005 50 14", (6000, 6000), &"U".repeat(750 * 6000));
    let art = native(&(layer("Live", true, &square(10, 10)) + &layer("Texture", true, &image)));
    let (d, warnings, read) = crate::layered_ai(&private(&art), pdf_part(10.0, 86.0), vec![], false, true, 1.0);
    assert!(read, "{warnings:?}");
    assert_eq!(names(&d), ["Live", "Texture"]);
    assert_eq!(kinds(&d.layers[0]), ["path"]);
    let [image] = d.layers[1].children().unwrap().as_slice() else { panic!("{:?}", d.layers[1]) };
    let NodeKind::Image(im) = &image.kind else { panic!("{image:?}") };
    assert_eq!((im.width, im.height), (6000, 6000));
    let png = &d.images[&im.key].bytes;
    // Bit depth 1, colour type 0 (gray).
    assert_eq!(png.get(24..26), Some([1, 0].as_slice()));
    let gray = craft_image_codecs::decode(png)
        .map(|__cimg| {
            let (__w, __h) = __cimg.dimensions();
            match (__cimg.layout(), __cimg.sample_type()) {
                (craft_image_codecs::ChannelLayout::Gray, craft_image_codecs::SampleType::U8) => {
                    image::DynamicImage::ImageLuma8(image::GrayImage::from_raw(__w, __h, __cimg.into_data()).unwrap())
                }
                (craft_image_codecs::ChannelLayout::GrayA, craft_image_codecs::SampleType::U8) => {
                    image::DynamicImage::ImageLumaA8(image::GrayAlphaImage::from_raw(__w, __h, __cimg.into_data()).unwrap())
                }
                (craft_image_codecs::ChannelLayout::Rgb, craft_image_codecs::SampleType::U8) => {
                    image::DynamicImage::ImageRgb8(image::RgbImage::from_raw(__w, __h, __cimg.into_data()).unwrap())
                }
                _ => image::DynamicImage::ImageRgba8(image::RgbaImage::from_raw(__w, __h, __cimg.to_rgba8()).unwrap()),
            }
        })
        .unwrap()
        .to_luma8();
    assert_eq!((gray.get_pixel(0, 0)[0], gray.get_pixel(1, 0)[0], gray.get_pixel(5999, 5999)[0]), (0, 255, 255));
    assert!(warnings.iter().all(|w| !w.contains("image")), "{warnings:?}");
}

/// An image past the target size is averaged down by the smallest whole step that fits, a 1-bit
/// one into gray coverage, the blocks at its edges over the pixels they have; it covers the box
/// the full size would.
#[test]
fn an_ai_image_past_the_target_size_is_reduced_and_keeps_the_layers() {
    // 17 × 13 pixels: rows black, and in turn (the first one) white but for columns 0, 8 and 16.
    let rows: String = (0..13).map(|y| if y % 2 == 0 { "\u{7f}\u{7f}\0" } else { "\0\0\0" }).collect();
    let image = raster_1bit("1 0 0 1 50 14", (17, 13), &rows);
    let art = native(&(layer("Live", true, &square(10, 10)) + &layer("Texture", true, &image)));
    let (d, warnings, read) = crate::import::ai::with_image_limits(20, 1 << 31, 512 << 20, || {
        crate::layered_ai(&private(&art), pdf_part(10.0, 86.0), vec![], false, true, 1.0)
    });
    assert!(read, "{warnings:?}");
    assert_eq!(names(&d), ["Live", "Texture"]);
    assert!(warnings.contains(&"an embedded image of 17 × 13 pixels was reduced to 5 × 4 pixels to keep the layers".to_string()), "{warnings:?}");
    let [image] = d.layers[1].children().unwrap().as_slice() else { panic!("{:?}", d.layers[1]) };
    let NodeKind::Image(im) = &image.kind else { panic!("{image:?}") };
    assert_eq!((im.width, im.height), (5, 4));
    let full = im.xf.transform_rect_bbox(vectorcraft_geom::Rect::new(0.0, 0.0, 5.0, 4.0));
    assert!(
        (full.x0 - 50.0).abs() < 1e-9 && (full.x1 - 67.0).abs() < 1e-9 && (full.y0 - 86.0).abs() < 1e-9 && (full.y1 - 99.0).abs() < 1e-9,
        "{full:?}"
    );
    let gray = craft_image_codecs::decode(&d.images[&im.key].bytes)
        .map(|__cimg| {
            let (__w, __h) = __cimg.dimensions();
            match (__cimg.layout(), __cimg.sample_type()) {
                (craft_image_codecs::ChannelLayout::Gray, craft_image_codecs::SampleType::U8) => {
                    image::DynamicImage::ImageLuma8(image::GrayImage::from_raw(__w, __h, __cimg.into_data()).unwrap())
                }
                (craft_image_codecs::ChannelLayout::GrayA, craft_image_codecs::SampleType::U8) => {
                    image::DynamicImage::ImageLumaA8(image::GrayAlphaImage::from_raw(__w, __h, __cimg.into_data()).unwrap())
                }
                (craft_image_codecs::ChannelLayout::Rgb, craft_image_codecs::SampleType::U8) => {
                    image::DynamicImage::ImageRgb8(image::RgbImage::from_raw(__w, __h, __cimg.into_data()).unwrap())
                }
                _ => image::DynamicImage::ImageRgba8(image::RgbaImage::from_raw(__w, __h, __cimg.to_rgba8()).unwrap()),
            }
        })
        .unwrap()
        .to_luma8();
    // Steps of 4: blocks of rows 0–11 are half white rows, the last block row 12 alone; blocks of
    // columns 0–3 and 8–11 have 3 white columns of 4, and the last block column 16 alone.
    let rows: Vec<Vec<u8>> = (0..4).map(|y| (0..5).map(|x| gray.get_pixel(x, y)[0]).collect()).collect();
    assert_eq!(rows, [vec![96, 128, 96, 128, 0], vec![96, 128, 96, 128, 0], vec![96, 128, 96, 128, 0], vec![191, 255, 191, 255, 0]]);
}

/// An image past the hard limits is a box that the page's art inside it replaces, and the
/// layers stay; without page art (the editing data alone), it is left out with a note.
#[test]
fn an_ai_image_past_the_hard_limits_comes_in_as_the_pages_art() {
    let image = raster_1bit("1 0 0 1 50 14", (4, 4), &"\0".repeat(4));
    let art = native(&(layer("Live", true, &square(10, 10)) + &layer("Texture", true, &image)));
    let (d, notes, read) =
        crate::import::ai::with_image_limits(20, 15, 512 << 20, || crate::layered_ai(&private(&art), pdf_two_squares(), vec![], false, false, 1.0));
    assert!(read, "{notes:?}");
    assert_eq!(names(&d), ["Live", "Texture"]);
    assert_eq!(kinds(&d.layers[1]), ["group[1]"]);
    replacement_is_cyan(&d, 1, 0);
    assert!(notes.iter().any(|n| n.contains("very large images") && n.contains("they came in as the page's art")), "{notes:?}");
    // Past the bytes the samples may take, too.
    let (_, notes, read) =
        crate::import::ai::with_image_limits(20, 1 << 31, 1, || crate::layered_ai(&private(&art), pdf_two_squares(), vec![], false, false, 1.0));
    assert!(read && notes.iter().any(|n| n.contains("very large images")), "{notes:?}");
    let (d, notes) = crate::import::ai::with_image_limits(20, 15, 512 << 20, || crate::ai_alone(&private(&art))).unwrap();
    assert_eq!(names(&d), ["Live", "Texture"]);
    assert!(d.layers[1].children().unwrap().is_empty());
    assert!(notes.iter().any(|n| n == "an image too large to read from the editing data was left out"), "{notes:?}");
}

/// A `.ai` whose layers hold a placed PDF and no text object: its PDF part draws the placed file,
/// type and all, where the layers show the file's preview. That type is the placed file's and needs
/// no text object, so the file comes in from its layers, the placed file where its matrix takes the
/// canvas point of art space's origin or at the corner of its box (see
/// `a_placed_file_at_its_boxs_corner_matches_the_page`). Type outside the placed file still needs
/// one, and without it the file comes in as its PDF part.
#[test]
fn type_of_the_page_inside_a_placed_file_needs_no_text_object() {
    // A file placed on (30, 30)–(70, 70) whose preview is 2 × 2 pixels of half-transparent black,
    // as in `a_placed_file_at_its_boxs_corner_matches_the_page`; with the template box, the canvas
    // point of art space's origin puts it off the page, and the box's corner on it.
    let place = format!(
        "%AI5_BeginPlace\n%%FileType: 1\n[1 0 0 -1 -7970 8070] 8000 8000 8040 8040 4 4 0\n(/files/pages.pdf)`\n\
         %AI26_BeginPlacedObjectPreview\n2 2 8 32 17 {}\n%AI26_EndPlacedObjectPreview\n~\n%AI5_EndPlace\n",
        "\x7f\0\0\0".repeat(4)
    );
    let at_origin = native(&layer("Pages", true, &place));
    let at_corner = at_origin.replace("%AI5_NumLayers", "%AI3_TemplateBox: 100 50 100 50\n%AI5_NumLayers");
    // The PDF part: the placed file's page in 50% grey, and type at (`x`, 55).
    let pdf = |x: f64| {
        let mut d = Document::new(100.0, 100.0);
        let grey = vectorcraft_doc::FillLayer::new(vectorcraft_color::Paint::solid(vectorcraft_color::Color::gray(0.5)));
        let rect = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(30.0, 30.0, 70.0, 70.0));
        let page = Node::path(
            d.alloc_id(),
            rect,
            vectorcraft_doc::Appearance { items: vec![vectorcraft_doc::AppearanceItem::Fill(grey)], ..Default::default() },
        );
        let style = vectorcraft_doc::CharStyle { size: 14.0, ..Default::default() };
        let text = vectorcraft_doc::TextObject::point(vectorcraft_geom::Point::new(x, 55.0), "Hi", style);
        let text = Node::new(d.alloc_id(), NodeKind::Text(Box::new(text)));
        let layer = d.layers[0].id;
        d.insert(Some(layer), 0, page).unwrap();
        d.insert(Some(layer), 1, text).unwrap();
        d
    };
    for data in [&at_origin, &at_corner] {
        let (d, warnings, read) = crate::layered_ai(&private(data), pdf(38.0), vec![], false, false, 1.0);
        assert!(read, "{warnings:?}");
        assert_eq!(names(&d), ["Pages"]);
        let mut links = vec![];
        d.visit_images(|_, im| links.extend(im.link.as_ref().map(|l| l.path.clone())));
        assert_eq!(links, ["/files/pages.pdf"], "{warnings:?}");
        assert!(warnings.iter().all(|w| !w.contains("no text object")), "the type is the placed file's, not left out: {warnings:?}");
        // Type beside the placed file is a text object's, which the layers don't have.
        let (_, warnings, read) = crate::layered_ai(&private(data), pdf(5.0), vec![], false, false, 1.0);
        assert!(!read, "{warnings:?}");
        assert!(warnings.iter().any(|w| w.contains("its layers have no text objects")), "{warnings:?}");
    }
    // With a text object the text document makes beside it, the placed file's type is still its
    // own, not type left out.
    // The template box's centre (100, 50) is the canvas centre: the anchor is at (5, 10) in the art.
    let doc = text_document(&[("Note\r", 0, (8096.5, 8231.5), (0.0, 0.0), 0.0)]);
    let both = native_with_text(&layer("Pages", true, &format!("{place}{}", text_object(0))), &doc)
        .replace("%AI3_TemplateBox: 50 50 50 50", "%AI3_TemplateBox: 100 50 100 50");
    let mut page = pdf(38.0);
    let style = vectorcraft_doc::CharStyle { size: 14.0, ..Default::default() };
    let note = Node::new(
        page.alloc_id(),
        NodeKind::Text(Box::new(vectorcraft_doc::TextObject::point(vectorcraft_geom::Point::new(5.0, 90.0), "Note", style))),
    );
    let layer_id = page.layers[0].id;
    page.insert(Some(layer_id), 2, note).unwrap();
    let (d, warnings, read) = crate::layered_ai(&private(&both), page, vec![], false, false, 1.0);
    assert!(read, "{warnings:?}");
    assert_eq!(text_of(&d.layers[0]).iter().map(|t| t.plain_text()).collect::<Vec<_>>(), ["Note"], "{warnings:?}");
    assert!(!warnings.iter().any(|w| w.contains("left out")), "{warnings:?}");
}

/// A palette of `[None]` and White, and a color group `Inks` holding a spot color, Red ink.
const PALETTE: &str =
    "%AI5_BeginPalette\n0 0 Pb\n0 0 0 0 1 1 1 Xa\n(White)\nPc\n1 (Inks) 1 Pg\n0.06 0.98 1 0 (Red ink) 0 x\n(Red ink)\nPc\nPB\n%AI5_EndPalette\n";

/// A `.ai` that comes in as its PDF part, whose layers can't be used, still gets the swatches and
/// color groups of its editing data's palette in place of the defaults, and keeps the inks its PDF
/// part's art paints with that the palette doesn't have.
#[test]
fn an_ai_file_that_comes_in_as_its_pdf_part_keeps_the_swatches_of_its_palette() {
    let image = "%AI5_File:\n%AI5_BeginRaster\n() 1 XG\n/DeviceGray XN\n[ 1 0 0 1 0 100 ] 6000 6000 0 Xh\n\
         [ 1 0 0 1 0 100 ] 0 0 6000 6000 6000 6000 8 1 0 0 1 0 4 4 0 0\n%%BeginData: 1\rXI\n\u{0}%%EndData\r\nXH\r\n%AI5_EndRaster\r\nN";
    let art = native(&format!("{PALETTE}{}", layer("Texture", true, &format!("{}\n{image}", square(10, 10)))));
    let mut part = pdf_part(10.0, 86.0);
    let gold = vectorcraft_color::Paint::solid(vectorcraft_color::Color::cmyk(0.0, 0.2, 0.9, 0.1));
    part.swatches.push(vectorcraft_color::swatch::Swatch { name: "Gold ink".into(), paint: gold, global: true, spot: true });
    // An image past 2^25 pixels isn't read here, and the page has no art of its own in its box.
    let (d, warnings, read) =
        crate::import::ai::with_image_limits(1 << 26, 1 << 25, 512 << 20, || crate::layered_ai(&private(&art), part, vec![], false, true, 1.0));
    assert!(!read, "the PDF part is used: {warnings:?}");
    let swatches: Vec<&str> = d.swatches.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(swatches, ["[None]", "White", "Gold ink"], "{warnings:?}");
    let groups: Vec<&str> = d.swatch_groups.iter().map(|g| g.name.as_str()).collect();
    assert_eq!(groups, ["Inks"]);
    assert!(d.swatch("Red ink").is_some_and(|s| s.spot));
}

/// A `.ai` gradient library (a palette of gradient swatches and no layers) gets the palette's
/// gradient swatches, each holding its gradient without a link to a swatch.
#[test]
fn an_ai_file_without_layers_keeps_the_gradients_of_its_palette_unlinked() {
    let sky = "%AI5_BeginGradient: (Sky)\n(Sky) 0 2 Bd\n[\n%_1 0 50 0 Bs\n%_0 0 50 100 Bs\nBD\n%AI5_EndGradient\n";
    let palette = "%AI5_BeginPalette\n0 0 Pb\nBb\n2 (Sky) 0 0 0 1 1 0 0 1 0 0 1 Bg\n0 BB\n(Sky)\nPc\nPB\n%AI5_EndPalette\n";
    let art = native(&format!("{sky}{palette}"));
    let (d, warnings, read) = crate::layered_ai(&private(&art), pdf_part(10.0, 86.0), vec![], false, true, 1.0);
    assert!(!read, "{warnings:?}");
    let swatches: Vec<&str> = d.swatches.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(swatches, ["[None]", "Sky"], "{warnings:?}");
    let Some(vectorcraft_color::Paint::Gradient(g)) = d.swatch("Sky").map(|s| &s.paint) else { panic!("{:?}", d.swatch("Sky")) };
    assert_eq!(g.swatch, None, "the swatch's own paint");
    assert_eq!(g.gradient.stops.len(), 2);
}

/// A file that comes in as its PDF part reads the palette's names that aren't UTF-8 in the encoding
/// picked, with the notes on them; a layer name that isn't UTF-8, in layers that aren't used, gets
/// none.
#[test]
fn an_ai_file_that_comes_in_as_its_pdf_part_notes_its_palette_names_in_a_legacy_encoding() {
    let image = "%AI5_File:\n%AI5_BeginRaster\n() 1 XG\n/DeviceGray XN\n[ 1 0 0 1 0 100 ] 6000 6000 0 Xh\n\
         [ 1 0 0 1 0 100 ] 0 0 6000 6000 6000 6000 8 1 0 0 1 0 4 4 0 0\n%%BeginData: 1\rXI\n\u{0}%%EndData\r\nXH\r\n%AI5_EndRaster\r\nN";
    // 빨강 (red) in code page 949, for the group and its ink; 그라 for the layer.
    let palette = PALETTE.replace("Inks", r"\273\241\260\255").replace("Red ink", r"\273\241\260\255 ink");
    let art = native(&format!("{palette}{}", layer(r"\261\327\266\363", true, &format!("{}\n{image}", square(10, 10)))));
    // An image past 2^25 pixels isn't read here, and the page has no art of its own in its box.
    let (d, warnings, read) = crate::import::ai::with_image_limits(1 << 26, 1 << 25, 512 << 20, || {
        crate::layered_ai_from(crate::AiLayers::decode(&private(&art)), pdf_part(10.0, 86.0), vec![], false, true, 1.0, crate::NameEncoding::Korean)
    });
    assert!(!read, "the PDF part is used: {warnings:?}");
    let groups: Vec<&str> = d.swatch_groups.iter().map(|g| g.name.as_str()).collect();
    assert_eq!(groups, ["빨강"]);
    assert!(d.swatch("빨강 ink").is_some_and(|s| s.spot));
    let notes: Vec<&String> = warnings.iter().filter(|w| w.contains("isn't UTF-8")).collect();
    assert_eq!(notes.len(), 2, "{warnings:?}");
    assert!(notes.iter().all(|n| n.contains("windows-949") && !n.contains("그라")), "{notes:?}");
}

/// A library without layers reads the names of its brushes, graphic styles, symbols, variables and
/// data sets that aren't UTF-8 in the encoding picked, with a note on each name it brings in.
#[test]
fn an_ai_library_without_layers_reads_its_names_in_a_legacy_encoding() {
    // 빨강 and 그라 in code page 949.
    let (red, gra) = (r"\273\241\260\255", r"\261\327\266\363");
    let art = format!("%AI8_BeginBrushPattern\n({gra})\n0 0 m\n4 0 L\n4 4 L\nf\n%AI8_EndBrushPattern\n");
    let spray = format!(
        "%AI8_BeginPluginObject\n(Adobe Scatter Brush Tool)\n({red})\n(2 /{gra}/ 5 1 0 0 / 0 0.5 1.5 1 1.2 2 0 -0.25 0.25 2 -45 30 1 1 0) .\n%AI8_EndPluginObject\n"
    );
    let order = format!(
        "%AI8_BeginPluginObject\n(Adobe Brush Manager Order)\n(Adobe Brush Manager Order)\n( Adobe Scatter Brush Tool/ {red}/) .\n%AI8_EndPluginObject\n"
    );
    let styles =
        format!("%AI9_BeginArtStyles\n{}%AI9_EndArtStyles\n%AI9_BeginArtStyleList\n({red})\n%AI9_EndArtStyleList\n", stroke_art_style(red, 1.0, ""));
    let symbols = format!("%AI5_Begin_NonPrinting\n{}%AI5_End_NonPrinting--\n", symbol_def(gra, &symbol_square(-2.0, -2.0, 2.0, 2.0), 1));
    let variables = vectorcraft_testkit::ai::variable_library(&["QVARQ"], &["QSETQ"]).replace("QVARQ", red).replace("QSETQ", gra);
    let library = format!(
        "%!PS-Adobe-3.0 \n%%BoundingBox: 0 0 100 100\n%%BeginSetup\n{art}{order}{spray}{styles}{symbols}{variables}%%EndSetup\n%%PageTrailer\n%%Trailer\n"
    );
    let (d, warnings, read) = crate::layered_ai_from(
        crate::AiLayers::decode(&private(&library)),
        pdf_part(10.0, 86.0),
        vec![],
        false,
        true,
        1.0,
        crate::NameEncoding::Korean,
    );
    assert!(!read, "{warnings:?}");
    assert_eq!(vectorcraft_brush::library(&d).iter().map(|b| b.name.as_str()).collect::<Vec<_>>(), ["빨강"], "{warnings:?}");
    assert_eq!(d.graphic_styles.iter().map(|g| g.name.as_str()).collect::<Vec<_>>(), ["빨강"], "{warnings:?}");
    assert_eq!(d.symbols.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["그라"], "{warnings:?}");
    assert_eq!(d.variables.variables.iter().map(|v| v.name.as_str()).collect::<Vec<_>>(), ["빨강"], "{warnings:?}");
    assert_eq!(d.variables.datasets.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(), ["그라"], "{warnings:?}");
    let notes: Vec<&String> = warnings.iter().filter(|w| w.contains("isn't UTF-8") || w.contains("aren't UTF-8")).collect();
    assert_eq!(notes.len(), 2, "{warnings:?}");
    assert!(notes.iter().all(|n| n.contains("windows-949")), "{notes:?}");
}

/// A pattern of a file without layers that takes a free name beside a pattern of the PDF part has
/// a note on the name it was read under.
#[test]
fn a_renamed_pattern_of_a_file_without_layers_has_a_note_on_its_legacy_name() {
    // 빨강 in code page 949.
    let red = r"\273\241\260\255";
    let tile = format!("%AI3_BeginPattern: ({red})\n({red}) 0 0 4 4\n%_0 0 0 1 k\n%_1 1 m\n%_3 1 L\n%_3 3 L\n%_1 3 L\n%_f\nE\n%AI3_EndPattern\n");
    let palette = format!("%AI5_BeginPalette\n0 0 Pb\n({red}) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p\n(Tile)\nPc\nPB\n%AI5_EndPalette\n");
    let art = native(&format!("{tile}{palette}"));
    let mut part = pdf_part(10.0, 86.0);
    part.patterns.push(vectorcraft_doc::PatternDef::new("빨강", vec![]));
    let (d, warnings, read) =
        crate::layered_ai_from(crate::AiLayers::decode(&private(&art)), part, vec![], false, true, 1.0, crate::NameEncoding::Korean);
    assert!(!read, "{warnings:?}");
    assert!(d.pattern("빨강 2").is_some(), "{:?}", d.patterns.iter().map(|p| &p.name).collect::<Vec<_>>());
    let notes: Vec<&String> = warnings.iter().filter(|w| w.contains("isn't UTF-8")).collect();
    assert_eq!(notes.len(), 1, "{warnings:?}");
    assert!(notes[0].contains("`빨강`"), "{notes:?}");
}

/// A `.ai` pattern library (a palette of pattern swatches and no layers) whose PDF part defines no
/// pattern gets the patterns its editing data defines with the palette's pattern swatches.
#[test]
fn an_ai_file_without_layers_keeps_the_patterns_of_its_palette() {
    let dots = "%AI3_BeginPattern: (Dots)\n(Dots) 0 0 4 4\n%_0 0 0 1 k\n%_1 1 m\n%_3 1 L\n%_3 3 L\n%_1 3 L\n%_f\nE\n%AI3_EndPattern\n";
    let palette =
        "%AI5_BeginPalette\n0 0 Pb\n0 0 0 0 1 1 1 Xa\n(White)\nPc\n(Dots) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p\n(Dots)\nPc\nPB\n%AI5_EndPalette\n";
    let art = native(&format!("{dots}{palette}"));
    let (d, warnings, read) = crate::layered_ai(&private(&art), pdf_part(10.0, 86.0), vec![], false, true, 1.0);
    assert!(!read, "{warnings:?}");
    let swatches: Vec<&str> = d.swatches.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(swatches, ["[None]", "White", "Dots"], "{warnings:?}");
    assert!(matches!(&d.swatch("Dots").map(|s| &s.paint), Some(vectorcraft_color::Paint::Pattern { pattern, .. }) if pattern == "Dots"));
    assert!(d.pattern("Dots").is_some_and(|p| !p.art.is_empty()));

    // A pattern that shows an image brings the image's data with it.
    let pic = "%AI3_BeginPattern: (Pic)\n(Pic) 0 0 4 4\n%AI5_File:\n%AI5_BeginRaster\n() 1 XG\n/DeviceGray XN\n[ 4 0 0 4 0 4 ] 1 1 0 Xh\n\
               [ 4 0 0 4 0 4 ] 0 0 1 1 1 1 8 1 0 0 1 0 4 4 0 0\n%%BeginData: 1\rXI\n\u{0}%%EndData\r\nXH\r\n%AI5_EndRaster\r\nN\nE\n%AI3_EndPattern\n";
    let art = native(&format!("{pic}{}", palette.replace("(Dots)", "(Pic)")));
    let (d, warnings, _) = crate::layered_ai(&private(&art), pdf_part(10.0, 86.0), vec![], false, true, 1.0);
    let mut keys = vec![];
    d.visit_images(|_, im| keys.push(im.key.clone()));
    assert_eq!(keys.len(), 1, "{warnings:?}");
    assert!(keys.iter().all(|k| d.images.contains_key(k)), "{keys:?}");
}

/// The art of a pattern with an opacity mask that a palette swatch or a brush's art paints with,
/// in a file without layers, takes ids of the document, its mask's art included.
#[test]
fn a_pattern_of_a_file_without_layers_takes_ids_for_its_mask_art() {
    let mask_art: String = symbol_square(0.0, 0.0, 4.0, 4.0).replace("0 1 1 0 1 0 0 Xa", "1 g").lines().map(|l| format!("%_{l}\n")).collect();
    let mask = format!("%_/ArtDictionary :\n%_/Mask :\n%_X=\n{mask_art}%_X+\n%_(Art) ,\n%_1 /Bool (Clipping) ,\n%_; (AI9_Mask) ,\n%_;\n");
    let hidden = |art: String| -> String { art.lines().map(|l| format!("%_{l}\n")).collect() };
    let tile = format!("{}{mask}{}", hidden(symbol_square(0.0, 0.0, 4.0, 4.0)), hidden(symbol_square(1.0, 1.0, 3.0, 3.0)));
    let dots = format!("%AI3_BeginPattern: (Dots)\n(Dots) 0 0 4 4\n{tile}E\n%AI3_EndPattern\n");
    let palette = "%AI5_BeginPalette\n0 0 Pb\n(Dots) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p\n(Dots)\nPc\nPB\n%AI5_EndPalette\n";
    let order = "%AI8_BeginPluginObject\n(Adobe Brush Manager Order)\n(Adobe Brush Manager Order)\n( Adobe Scatter Brush Tool/ Spray/) .\n%AI8_EndPluginObject\n";
    let spray = "%AI8_BeginPluginObject\n(Adobe Scatter Brush Tool)\n(Spray)\n(2 /Dot art/ 5 1 0 0 / 0 0.5 1.5 1 ) -\n\
                 (1.2 2 0 -0.25 0.25 2 -45 30 1 1 0) .\n%AI8_EndPluginObject\n";
    let dot = "%AI8_BeginBrushPattern\n(Dot art)\n(Dots) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p\n0 0 m\n4 0 L\n4 4 L\nf\n%AI8_EndBrushPattern\n";
    for setup in [format!("{dots}{palette}"), format!("{dots}{dot}{order}{spray}")] {
        let library = format!("%!PS-Adobe-3.0 \n%%BoundingBox: 0 0 100 100\n%%BeginSetup\n{setup}%%EndSetup\n%%PageTrailer\n%%Trailer\n");
        let (d, warnings, read) = crate::layered_ai(&private(&library), pdf_part(10.0, 86.0), vec![], false, true, 1.0);
        assert!(!read, "{warnings:?}");
        let art = d.pattern("Dots").map(|p| p.art.clone()).unwrap_or_default();
        let mut masked = art.iter().flat_map(|n| std::iter::once(n).chain(n.children().into_iter().flatten()));
        assert!(masked.any(|n| n.extras().mask.is_some()), "the mask is read: {art:#?} {warnings:?}");
        let mut ids = vec![];
        for n in d.layers.iter().chain(d.patterns.iter().flat_map(|p| &p.art)) {
            all_ids(n, &mut ids);
        }
        let unique: std::collections::HashSet<_> = ids.iter().copied().collect();
        assert_eq!(unique.len(), ids.len(), "{ids:?}");
        assert!(ids.iter().all(|id| id.0 < d.peek_next_id()), "{ids:?}");
    }
}

/// A brush library (brush objects and their art, no layers) comes in as its PDF part with the
/// library's brushes in place of the default ones: Other Library lists them.
#[test]
fn an_ai_brush_library_without_layers_keeps_its_brushes() {
    let order = "%AI8_BeginPluginObject\n(Adobe Brush Manager Order)\n(Adobe Brush Manager Order)\n\
                 ( Adobe Scatter Brush Tool/ Spray/ Adobe Calli) -\n(graphic Brush Tool/ Nib/) .\n%AI8_EndPluginObject\n";
    let nib = "%AI8_BeginPluginObject\n(Adobe Calligraphic Brush Tool)\n(Nib)\n(1 2 6 40 60 -30 10 1 0 2 0) .\n%AI8_EndPluginObject\n";
    let spray = "%AI8_BeginPluginObject\n(Adobe Scatter Brush Tool)\n(Spray)\n(2 /Dot art/ 5 1 0 0 / 0 0.5 1.5 1 ) -\n\
                 (1.2 2 0 -0.25 0.25 2 -45 30 1 1 0) .\n%AI8_EndPluginObject\n";
    // The art is filled with a pattern no swatch paints with: it comes in with the brush.
    let dots = "%AI3_BeginPattern: (Dots)\n(Dots) 0 0 4 4\n%_0 0 0 1 k\n%_1 1 m\n%_3 1 L\n%_3 3 L\n%_1 3 L\n%_f\nE\n%AI3_EndPattern\n";
    let dot = "%AI8_BeginBrushPattern\n(Dot art)\n(Dots) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p\n0 0 m\n4 0 L\n4 4 L\nf\n%AI8_EndBrushPattern\n";
    let library =
        format!("%!PS-Adobe-3.0 \n%%BoundingBox: 0 0 0 0\n%%BeginSetup\n{dots}{dot}{order}{nib}{spray}%%EndSetup\n%%PageTrailer\n%%Trailer\n");
    let (d, warnings, read) = crate::layered_ai(&private(&library), pdf_part(10.0, 86.0), vec![], false, true, 1.0);
    assert!(!read, "{warnings:?}");
    let lib = vectorcraft_brush::library(&d);
    assert_eq!(lib.iter().map(|b| b.name.as_str()).collect::<Vec<_>>(), ["Spray", "Nib"], "{warnings:?}");
    assert!(warnings.is_empty(), "{warnings:?}");
    assert!(d.pattern("Dots").is_some_and(|p| !p.art.is_empty()), "the pattern the brush's art paints with");
}

/// An art style that strokes with black at `width` units, after the effects `effects` (filters as
/// the format writes them), as the format writes it.
fn stroke_art_style(name: &str, width: f64, effects: &str) -> String {
    format!(
        "/KnownStyle :\n({name}) /Name ,\n/ActiveStyle :\n/CompoundFilter :\n(Chain Style Filter) 0 0 /Filter ,\n1 /Visible ,\n{effects}\
         /BasicFilter :\n(Stroke Style Filter) 0 0 /Filter ,\n1 /Visible ,\n/Dictionary : /NotRecorded ,\n\
         /StrokeStyle : 0 R\n0 0 0 0 0 0 1 XA\n0 J 0 j {width} w 10 M []0 d\n0 XR\n1 0 Xd\n/Def ;\n (StrokeStyle) ,\n; /Dict ;\n /Part ,\n;\n"
    )
}

/// The art styles and the style list of a graphic style library: Thin, Medium and Thick, listed in
/// that order, 1, 2 and 3 units wide, Medium after an Offset Path of 4 units, and Unlisted, which
/// the list leaves out, with Thin listed twice.
fn style_list() -> String {
    let offset = "/BasicFilter :\n(Adobe Offset Path) 1 0 /Filter ,\n1 /Visible ,\n(OffsetPath.aip) /PluginFileName ,\n(Offset Path) /Title ,\n\
        /Dictionary : /NotRecorded ,\n4 /Real (ofst) ,\n6 /Real (mlim) ,\n1 /Int (jntp) ,\n; /Dict ;\n /Part ,\n";
    format!(
        "%AI9_BeginArtStyles\n{}{}{}{}%AI9_EndArtStyles\n%AI9_BeginArtStyleList\n(Thin)\n(Medium)\n(Thick)\n(Thin)\n%AI9_EndArtStyleList\n",
        stroke_art_style("Thick", 3.0, ""),
        stroke_art_style("Unlisted", 4.0, ""),
        stroke_art_style("Thin", 1.0, ""),
        stroke_art_style("Medium", 2.0, offset)
    )
}

/// The offset of each Offset Path in the graphic styles of `d`.
fn style_offsets(d: &Document) -> Vec<f64> {
    let effects = d.graphic_styles.iter().flat_map(|g| &g.appearance.effects);
    effects.filter(|e| e.id == "path.offsetPath").filter_map(|e| e.params["offset"].as_f64()).collect()
}

/// A graphic style library (the art styles and list of [`style_list`], and no layers) with the
/// editing data's setup `setup` before them.
fn style_library(setup: &str) -> String {
    let styles = style_list();
    format!("%!PS-Adobe-3.0 \n%%BoundingBox: 0 0 100 100\n%%BeginSetup\n{setup}{styles}%%EndSetup\n%%PageTrailer\n%%Trailer\n")
}

/// A graphic style library (art styles and their list, no layers) comes in as its PDF part with the
/// library's graphic styles in place of the default ones, in the list's order: Other Library lists them.
#[test]
fn an_ai_graphic_style_library_without_layers_keeps_its_styles() {
    let (d, warnings, read) = crate::layered_ai(&private(&style_library("")), pdf_part(10.0, 86.0), vec![], false, true, 1.0);
    assert!(!read, "{warnings:?}");
    let styles: Vec<(&str, u32, f64)> = d.graphic_styles.iter().map(|g| (g.name.as_str(), g.id, g.appearance.stroke_width())).collect();
    assert_eq!(styles, [("Thin", 1, 1.0), ("Medium", 2, 2.0), ("Thick", 3, 3.0)], "{warnings:?}");
    assert_eq!(style_offsets(&d), [4.0]);
    assert!(warnings.is_empty(), "{warnings:?}");
    // Without a style list, the default styles stay.
    let (d, warnings, _) = crate::layered_ai(&private(&native("")), pdf_part(10.0, 86.0), vec![], false, true, 1.0);
    assert_eq!(d.graphic_styles, Document::new(1.0, 1.0).graphic_styles, "{warnings:?}");
}

/// A graphic style library whose artboard is its page in the units of a `UserUnit` of 10 comes in
/// with its styles' stroke weights and effects ten times as large, as the page's art is; one whose
/// artboard is the page in points keeps them. Without a `UserUnit`, a Large Canvas library whose
/// artboard is the page at its declared scale of 10 comes in with them ten times as large too.
#[test]
fn an_ai_graphic_style_library_in_the_units_of_a_user_unit_page_comes_in_at_the_pages_size() {
    let board = "%_/Document :\n%_/Dictionary :\n%_/Array :\n\
        %_/Dictionary :\n%_0 100 /RealPointRelToROrigin\n%_ (PositionPoint1) ,\n%_100 0 /RealPointRelToROrigin\n%_ (PositionPoint2) ,\n%_; ,\n\
        %_; (ArtboardArray) ,\n%_; /NotRecorded ,\n%_;\n";
    let widths = |d: &Document| d.graphic_styles.iter().map(|g| g.appearance.stroke_width()).collect::<Vec<_>>();
    let page = in_points(pdf_part(10.0, 86.0), 10.0);
    let (d, warnings, read) = crate::layered_ai(&private(&style_library(board)), page, vec![], false, true, 10.0);
    assert!(!read, "{warnings:?}");
    assert_eq!(widths(&d), [10.0, 20.0, 30.0], "{warnings:?}");
    assert_eq!(style_offsets(&d), [40.0]);
    let points = board.replace("0 100 /Real", "0 1000 /Real").replace("100 0 /Real", "1000 0 /Real");
    let page = in_points(pdf_part(10.0, 86.0), 10.0);
    let (d, warnings, _) = crate::layered_ai(&private(&style_library(&points)), page, vec![], false, true, 10.0);
    assert_eq!(widths(&d), [1.0, 2.0, 3.0], "{warnings:?}");
    assert_eq!(style_offsets(&d), [4.0]);
    let large = format!("%AI24_LargeCanvasScale: 10\n{board}");
    let page = in_points(pdf_part(10.0, 86.0), 10.0);
    let (d, warnings, _) = crate::layered_ai(&private(&style_library(&large)), page, vec![], false, true, 1.0);
    assert_eq!(widths(&d), [10.0, 20.0, 30.0], "{warnings:?}");
    assert_eq!(style_offsets(&d), [40.0]);
}



/// A symbol named `name` whose art is `art` (in the symbol's space, y down), with the registration
/// point `registration` (1 to 9 from the top left, row by row), as the editing data defines it.
fn symbol_def(name: &str, art: &str, registration: u32) -> String {
    format!(
        "%AI14_BeginSymbol\n({name})\n0 A\n0 Xw\n0 Ae\nu\n{art}U\n%_/ArtDictionary :\n%_{registration} /Int (AI13PatternRegistrationType) ,\n%_;\n%_\n9 () XW\n%AI10_EndSymbol\n"
    )
}

/// A red square from (`x0`, `y0`) to (`x1`, `y1`), as a symbol's art.
fn symbol_square(x0: f64, y0: f64, x1: f64, y1: f64) -> String {
    format!("0 1 1 0 1 0 0 Xa\n{x0} {y0} m\n{x0} {y1} L\n{x1} {y1} L\n{x1} {y0} L\n{x0} {y0} L\nf\n")
}

/// The symbols Dot (a square of 4 about the origin, registered at its top left), Bar (12 × 2 about
/// the origin) and Masked (a square with an opacity mask), listed as Bar, Masked, Dot.
fn symbol_defs() -> String {
    let mask_art: String = symbol_square(-2.0, -2.0, 2.0, 2.0).replace("0 1 1 0 1 0 0 Xa", "1 g").lines().map(|l| format!("%_{l}\n")).collect();
    let mask = format!("%_/ArtDictionary :\n%_/Mask :\n%_X=\n{mask_art}%_X+\n%_(Art) ,\n%_1 /Bool (Clipping) ,\n%_; (AI9_Mask) ,\n%_;\n");
    let masked = format!("{}{mask}{}", symbol_square(-2.0, -2.0, 2.0, 2.0), symbol_square(-1.0, -1.0, 1.0, 1.0));
    let defs = symbol_def("Dot", &symbol_square(-2.0, -2.0, 2.0, 2.0), 1)
        + &symbol_def("Bar", &symbol_square(-6.0, -1.0, 6.0, 1.0), 5)
        + &symbol_def("Masked", &masked, 5);
    format!("%AI5_Begin_NonPrinting\n{defs}%AI5_End_NonPrinting--\n%AI24_BeginSymbolList\n(Bar)\n(Masked)\n(Dot)\n%AI24_EndSymbolList\n")
}

/// A symbol library (the symbols of [`symbol_defs`], and no layers).
fn symbol_library() -> String {
    format!("%!PS-Adobe-3.0 \n%%BoundingBox: 0 0 100 100\n%%BeginSetup\n{}%%EndSetup\n%%PageTrailer\n%%Trailer\n", symbol_defs())
}

/// The ids of `n`, what it holds and the art of its opacity masks.
fn all_ids(n: &Node, out: &mut Vec<vectorcraft_doc::NodeId>) {
    out.push(n.id);
    if let Some(m) = n.extras().mask.as_ref() {
        all_ids(&m.art, out);
    }
    for c in n.children().into_iter().flatten() {
        all_ids(c, out);
    }
}

/// The names of the symbols of `d`, in the Symbols panel's order.
fn symbol_names(d: &Document) -> Vec<&str> {
    d.symbols.iter().map(|s| s.name.as_str()).collect()
}

/// A symbol library comes in as its PDF part with the library's symbols, in the list's order, with
/// their registration points: Other Library lists them. Their art takes ids of the document, the
/// art of an opacity mask in it included.
#[test]
fn an_ai_symbol_library_without_layers_keeps_its_symbols() {
    let (d, warnings, read) = crate::layered_ai(&private(&symbol_library()), pdf_part(10.0, 86.0), vec![], false, true, 1.0);
    assert!(!read, "{warnings:?}");
    assert_eq!(symbol_names(&d), ["Bar", "Masked", "Dot"], "{warnings:?}");
    assert_eq!(d.symbol_registration("Dot"), vectorcraft_doc::SymbolRegistration(0));
    assert!(d.symbol_registration("Bar").is_center());
    let boxes = d.symbol_boxes();
    let bar = d.symbols.first().and_then(|s| s.art.visual_bounds_with(&boxes));
    assert_eq!(bar, Some(vectorcraft_geom::Rect::new(-6.0, -1.0, 6.0, 1.0)));
    drop(boxes);
    let mut ids = vec![];
    for n in d.layers.iter().chain(d.symbols.iter().map(|s| &s.art)) {
        all_ids(n, &mut ids);
    }
    let masked = d.symbols.get(1).map(|s| &s.art);
    assert!(masked.is_some_and(|a| a.children().into_iter().flatten().any(|c| c.extras().mask.is_some())), "the mask is read");
    let unique: std::collections::HashSet<_> = ids.iter().copied().collect();
    assert_eq!(unique.len(), ids.len(), "{ids:?}");
    assert!(warnings.is_empty(), "{warnings:?}");
    // Without symbols, the PDF part's (none) stay.
    let (d, warnings, _) = crate::layered_ai(&private(&native("")), pdf_part(10.0, 86.0), vec![], false, true, 1.0);
    assert!(d.symbols.is_empty(), "{warnings:?}");
}





/// A symbol library's symbols bring the spot colors their art is linked to. A color whose name a
/// swatch of the library's palette has for another color comes in under a free variant of the
/// name, which the art then names, as it does for an ink of the PDF part; without a palette, it
/// takes the place of the default swatch of that name.
#[test]
fn an_ai_symbol_librarys_symbols_keep_their_spot_colors() {
    let library = |setup: &str, ink: &str| {
        let art = format!("0 0 1 0 ({ink}) 1 x\n-2 -2 m\n-2 2 L\n2 2 L\n2 -2 L\n-2 -2 L\nf\n");
        let defs = format!("{setup}%AI5_Begin_NonPrinting\n{}%AI5_End_NonPrinting--\n", symbol_def("Inked", &art, 5));
        format!("%!PS-Adobe-3.0 \n%%BoundingBox: 0 0 100 100\n%%BeginSetup\n{defs}%%EndSetup\n%%PageTrailer\n%%Trailer\n")
    };
    let open = |text: &str| crate::layered_ai(&private(text), pdf_part(10.0, 86.0), vec![], false, true, 1.0);
    let link = |d: &Document| match d.symbols.first().and_then(|s| s.art.appearance.fill()).map(|f| &f.paint) {
        Some(vectorcraft_color::Paint::Solid { swatch, .. }) => swatch.clone(),
        _ => None,
    };
    let ink = |d: &Document, name: &str| d.swatch(name).map(|s| (s.paint.color(), s.spot));
    let yellow = Some(vectorcraft_color::Color::cmyk(0.0, 0.0, 1.0, 0.0));
    let (d, warnings, _) = open(&library("", "Brand Ink"));
    assert_eq!(link(&d).as_deref(), Some("Brand Ink"), "{warnings:?}");
    assert_eq!(ink(&d, "Brand Ink"), Some((yellow, true)));
    let palette = "%AI5_BeginPalette\n0 0 Pb\n0 0 0 0 0 1 0 Xa\n(Clash)\nPc\nPB\n%AI5_EndPalette\n";
    let (d, warnings, _) = open(&library(palette, "Clash"));
    assert_eq!(link(&d).as_deref(), Some("Clash 2"), "{warnings:?}");
    assert_eq!(ink(&d, "Clash 2"), Some((yellow, true)));
    assert_eq!(ink(&d, "Clash"), Some((Some(vectorcraft_color::Color::rgb(0.0, 1.0, 0.0)), false)));
    let (d, warnings, _) = open(&library("", "Red"));
    assert_eq!(link(&d).as_deref(), Some("Red"), "{warnings:?}");
    assert_eq!(ink(&d, "Red"), Some((yellow, true)));
    assert_eq!(d.swatches.iter().filter(|s| s.name == "Red").count(), 1);
    // An ink of the PDF part keeps its swatch: the library's color of that name is renamed.
    let mut part = pdf_part(10.0, 86.0);
    let green = vectorcraft_color::Paint::solid(vectorcraft_color::Color::cmyk(1.0, 0.0, 1.0, 0.0));
    part.swatches.push(vectorcraft_color::Swatch { name: "Ink".into(), paint: green.clone(), global: true, spot: true });
    let (d, warnings, _) = crate::layered_ai(&private(&library("", "Ink")), part, vec![], false, true, 1.0);
    assert_eq!(link(&d).as_deref(), Some("Ink 2"), "{warnings:?}");
    assert_eq!(ink(&d, "Ink 2"), Some((yellow, true)));
    assert_eq!(d.swatch("Ink").map(|s| &s.paint), Some(&green));
    // A color group of the PDF part keeps its name: the library's color of that name is renamed.
    let mut part = pdf_part(10.0, 86.0);
    part.swatch_groups.push(vectorcraft_color::SwatchGroup { name: "Kit".into(), swatches: vec![] });
    let (d, warnings, _) = crate::layered_ai(&private(&library("", "Kit")), part, vec![], false, true, 1.0);
    assert_eq!(link(&d).as_deref(), Some("Kit 2"), "{warnings:?}");
    assert_eq!(ink(&d, "Kit 2"), Some((yellow, true)));
    assert!(d.swatch("Kit").is_none());
}

/// A symbol library's symbols bring a global process color their art is filled with as a global
/// swatch that isn't an ink, and a spot color a stop of their gradient names as an ink.
#[test]
fn an_ai_symbol_librarys_symbols_keep_their_global_and_gradient_stop_colors() {
    let gradient = "%AI5_BeginGradient: (Fade)\n(Fade) 0 2 Bd\n[\n\
                    %_0 1 1 0 (Brand) 0 0 5 1 6 50 0 Bs\n%_0 0 0 0 1 0 0 2 1 6 50 100 Bs\nBD\n%AI5_EndGradient\n";
    let global = "0 0 1 0 (Process Yellow) 0 0 Xk\n-2 -2 m\n-2 2 L\n2 2 L\n2 -2 L\n-2 -2 L\nf\n";
    let graded = "Bb\n1 (Fade) 0 0 0 4 1 0 0 1 0 0 Bg\n-2 -2 m\n-2 2 L\n2 2 L\n2 -2 L\n-2 -2 L\nf\n0 BB\n";
    let defs = symbol_def("Global", global, 5) + &symbol_def("Graded", graded, 5);
    let text = format!(
        "%!PS-Adobe-3.0 \n%%BoundingBox: 0 0 100 100\n%%BeginSetup\n{gradient}%AI5_Begin_NonPrinting\n{defs}%AI5_End_NonPrinting--\n\
         %%EndSetup\n%%PageTrailer\n%%Trailer\n"
    );
    let (d, warnings, _) = crate::layered_ai(&private(&text), pdf_part(10.0, 86.0), vec![], false, true, 1.0);
    assert_eq!(symbol_names(&d), ["Global", "Graded"], "{warnings:?}");
    let swatch = |name: &str| d.swatch(name).map(|s| (s.paint.color(), s.global, s.spot));
    assert_eq!(swatch("Process Yellow"), Some((Some(vectorcraft_color::Color::cmyk(0.0, 0.0, 1.0, 0.0)), true, false)), "{warnings:?}");
    assert_eq!(swatch("Brand"), Some((Some(vectorcraft_color::Color::cmyk(0.0, 1.0, 1.0, 0.0)), true, true)), "{warnings:?}");
}

/// A symbol library's symbol shows its image from the document's images, and an object of a
/// symbol's art linked to a style of the library's style list links to that style as the document
/// has it.
#[test]
fn an_ai_symbol_librarys_symbols_bring_their_images_and_style_links() {
    // The template box's center is (100.5, 50.5): art space's origin is at canvas (8091, 8242).
    let m = "[ 5 0 0 5 -8096 8244.5 ]";
    let raster = format!(
        "%AI5_File:\n%AI5_BeginRaster\n() 1 XG\n/DeviceRGB XN\n{m} 2 1 0 Xh\n{m} 0 0 2 1 2 1 8 3 0 0 1 0 4 4 0 0\n\
         %%BeginData: 6\rXI\nABCDEF%%EndData\r\nXH\r\n%AI5_EndRaster\r\nN\n"
    );
    let path = "-2 -2 m\n-2 2 L\n2 2 L\n2 -2 L\n-2 -2 L\n";
    let hidden: String = path.lines().map(|l| format!("%_{l}\n")).collect();
    let styled = format!("0 Ae\nu\n0 0 0 0 0 0 1 XA\n0 J 0 j 3 w 10 M []0 d\n{path}S\nU\n{hidden}%_S\n1 (Thick) XW\n");
    let defs = format!("%AI5_Begin_NonPrinting\n{}{}%AI5_End_NonPrinting--\n", symbol_def("Pic", &raster, 5), symbol_def("Line", &styled, 5));
    let text = style_library(&defs).replacen("%%BeginSetup", "%AI3_TemplateBox: 100.5 50.5 100.5 50.5\n%%BeginSetup", 1);
    let (d, warnings, read) = crate::layered_ai(&private(&text), pdf_part(10.0, 86.0), vec![], false, true, 1.0);
    assert!(!read, "{warnings:?}");
    assert_eq!(symbol_names(&d), ["Pic", "Line"], "{warnings:?}");
    let key = d.symbols.first().and_then(|s| if let NodeKind::Image(im) = &s.art.kind { Some(im.key.clone()) } else { None });
    let key = key.unwrap_or_else(|| panic!("an image: {:#?}", d.symbols.first()));
    assert!(d.images.contains_key(&key), "{key}");
    let mut linked = vec![];
    let mut stack: Vec<&Node> = d.symbols.get(1).map(|s| &*s.art).into_iter().collect();
    while let Some(n) = stack.pop() {
        linked.extend(n.extras().graphic_style);
        stack.extend(n.children().into_iter().flatten().map(|c| &**c));
    }
    let named: Vec<&str> = linked.iter().filter_map(|id| d.graphic_styles.iter().find(|g| g.id == *id)).map(|g| g.name.as_str()).collect();
    assert_eq!(named, ["Thick"], "{linked:?}");
}

/// The instances of a symbol library's symbols that would draw more than
/// [`vectorcraft_doc::depth::MAX_DRAWN_DEPTH`] objects deep go, with the note saying so, and a
/// document that has symbols already keeps them.
#[test]
fn an_ai_symbol_librarys_instances_drawn_too_deep_go() {
    // C0 is a square, and each later symbol holds an instance of the one before it. The template
    // box's center is (100.5, 50.5): art space's origin is at canvas (8091, 8242).
    let instance = |name: &str| format!("/SymbolInstance :\n({name}) /SymbolRef ,\n1 0 0 -1 -8091 8242 /RTransformMatrix\n;\n9 () XW\n");
    let count = vectorcraft_doc::depth::MAX_DRAWN_DEPTH + 10;
    let mut defs = symbol_def("C0", &symbol_square(-2.0, -2.0, 2.0, 2.0), 5);
    for i in 1..count {
        defs.push_str(&symbol_def(&format!("C{i}"), &instance(&format!("C{}", i - 1)), 5));
    }
    let text = format!(
        "%!PS-Adobe-3.0 \n%%BoundingBox: 0 0 100 100\n%AI3_TemplateBox: 100.5 50.5 100.5 50.5\n%%BeginSetup\n\
         %AI5_Begin_NonPrinting\n{defs}%AI5_End_NonPrinting--\n%%EndSetup\n%%PageTrailer\n%%Trailer\n"
    );
    let (d, warnings, _) = crate::layered_ai(&private(&text), pdf_part(10.0, 86.0), vec![], false, true, 1.0);
    assert_eq!(d.symbols.len(), count, "{warnings:?}");
    let instances = d.symbols.iter().filter(|s| matches!(s.art.kind, NodeKind::SymbolInstance { .. })).count();
    assert!(instances < count - 1, "{instances}");
    assert!(warnings.contains(&vectorcraft_doc::depth::symbols_note()), "{warnings:?}");
    // A document with symbols keeps its own.
    let mut part = pdf_part(10.0, 86.0);
    let mine = Node::group(part.alloc_id(), vec![]);
    part.symbols.push(vectorcraft_doc::Symbol { name: "Mine".into(), art: std::sync::Arc::new(mine), dynamic: false });
    let (d, warnings, _) = crate::layered_ai(&private(&text), part, vec![], false, true, 1.0);
    assert_eq!(symbol_names(&d), ["Mine"], "{warnings:?}");
}

/// A PDF part whose tiling has the name of a pattern its editing data defines (the PDF part names
/// its tilings `Pattern 1`, `Pattern 2` and so on): the editing data's pattern comes in under a free
/// variant of the name, and the palette's swatch and the brush's art paint with it, not with the
/// tiling.
#[test]
fn an_ai_file_without_layers_keeps_its_patterns_apart_from_the_pdf_parts_tilings() {
    let dots = "%AI3_BeginPattern: (Pattern 1)\n(Pattern 1) 0 0 4 4\n%_0 0 0 1 k\n%_1 1 m\n%_3 1 L\n%_3 3 L\n%_1 3 L\n%_f\nE\n%AI3_EndPattern\n";
    let palette = "%AI5_BeginPalette\n0 0 Pb\n(Pattern 1) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p\n(Dots)\nPc\nPB\n%AI5_EndPalette\n";
    let order = "%AI8_BeginPluginObject\n(Adobe Brush Manager Order)\n(Adobe Brush Manager Order)\n( Adobe Scatter Brush Tool/ Spray/) .\n%AI8_EndPluginObject\n";
    let spray = "%AI8_BeginPluginObject\n(Adobe Scatter Brush Tool)\n(Spray)\n(2 /Dot art/ 5 1 0 0 / 0 0.5 1.5 1 ) -\n\
                 (1.2 2 0 -0.25 0.25 2 -45 30 1 1 0) .\n%AI8_EndPluginObject\n";
    let dot = "%AI8_BeginBrushPattern\n(Dot art)\n(Pattern 1) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p\n0 0 m\n4 0 L\n4 4 L\nf\n%AI8_EndBrushPattern\n";
    let part = || {
        let mut part = pdf_part(10.0, 86.0);
        let id = part.alloc_id();
        let square = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(0.0, 0.0, 9.0, 9.0));
        let tiling = std::sync::Arc::new(Node::path(id, square, vectorcraft_doc::Appearance::default()));
        let tile = vectorcraft_geom::Rect::new(0.0, 0.0, 9.0, 9.0);
        part.patterns.push(vectorcraft_doc::PatternDef { tile, ..vectorcraft_doc::PatternDef::new("Pattern 1", vec![tiling]) });
        part
    };
    let width = |d: &Document, name: &str| d.pattern(name).map(|p| p.tile.width());
    let swatch_pattern = |d: &Document| match d.swatch("Dots").map(|s| &s.paint) {
        Some(vectorcraft_color::Paint::Pattern { pattern, .. }) => Some(pattern.clone()),
        _ => None,
    };
    fn painted(n: &Node, out: &mut Vec<String>) {
        for p in vectorcraft_doc::swatches::node_paints(n) {
            if let vectorcraft_color::Paint::Pattern { pattern, .. } = p {
                out.push(pattern.clone());
            }
        }
        for c in n.children().into_iter().flatten() {
            painted(c, out);
        }
    }
    let brushed = |d: &Document| {
        let mut out = vec![];
        for b in vectorcraft_brush::library(d) {
            for a in b.art() {
                painted(a, &mut out);
            }
        }
        out
    };

    // The palette alone.
    let art = native(&format!("{dots}{palette}"));
    let (d, warnings, read) = crate::layered_ai(&private(&art), part(), vec![], false, true, 1.0);
    assert!(!read, "{warnings:?}");
    assert_eq!(width(&d, "Pattern 1"), Some(9.0), "the PDF part's tiling stays");
    assert_eq!(swatch_pattern(&d).as_deref(), Some("Pattern 1 2"), "{warnings:?}");
    assert_eq!(width(&d, "Pattern 1 2"), Some(4.0));

    // A brush library: the brush's art paints with the editing data's pattern.
    let library = format!("%!PS-Adobe-3.0 \n%%BoundingBox: 0 0 0 0\n%%BeginSetup\n{dots}{dot}{order}{spray}%%EndSetup\n%%PageTrailer\n%%Trailer\n");
    let (d, warnings, read) = crate::layered_ai(&private(&library), part(), vec![], false, true, 1.0);
    assert!(!read, "{warnings:?}");
    assert_eq!(brushed(&d), ["Pattern 1 2"], "{warnings:?}");
    assert_eq!(width(&d, "Pattern 1"), Some(9.0));
    assert_eq!(width(&d, "Pattern 1 2"), Some(4.0));

    // Both: the palette's swatch and the brush paint with the one pattern, added once.
    let both =
        format!("%!PS-Adobe-3.0 \n%%BoundingBox: 0 0 0 0\n%%BeginSetup\n{dots}{palette}{dot}{order}{spray}%%EndSetup\n%%PageTrailer\n%%Trailer\n");
    let (d, warnings, _) = crate::layered_ai(&private(&both), part(), vec![], false, true, 1.0);
    assert_eq!(swatch_pattern(&d).as_deref(), Some("Pattern 1 2"), "{warnings:?}");
    assert_eq!(brushed(&d), ["Pattern 1 2"], "{warnings:?}");
    let names: Vec<&str> = d.patterns.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["Pattern 1", "Pattern 1 2"]);
}




#[test]
fn an_ai_file_gets_its_layers_and_the_art_outside_its_artboard() {
    // A square on the artboard, and one above it (the artboard is y 0 to 100 here).
    let art = native(&(layer("Jig", true, &square(10, 10)) + &layer("Parts", true, &square(30, 110)) + &layer("Old", false, &square(60, 60))));
    let (d, warnings, read) = crate::layered_ai(&private(&art), pdf_part(10.0, 86.0), vec!["a note".into()], false, false, 1.0);
    assert_eq!(names(&d), ["Jig", "Parts", "Old"], "{warnings:?}");
    assert!(read);
    assert_eq!(warnings, ["a note"], "nothing else to say");
    let above = d.layers[1].children().unwrap()[0].visual_bounds().unwrap();
    assert!(above.y1 <= 0.0 + 1e-6, "the art above the artboard is kept, outside it: {above:?}");
    assert!(!d.layers[2].visible);
}

/// A `.ai` of versions 10 to 13, whose symbol instances are placed by a `TransformMatrix` on the
/// canvas, opens from its layers: the art outside its artboard, an instance among it, comes in,
/// rather than the PDF part alone, which has none of it.
#[test]
fn an_older_ai_files_symbols_keep_its_layers_and_the_art_outside_its_artboard() {
    // The template box's center is (50.5, 50.5): art space's origin is at canvas (8141, 8242). The
    // instance puts Box at (150, 50) in art space, right of the artboard (x 0 to 100).
    let instance = "/SymbolInstance :\n(Box) /SymbolRef ,\n1 0 0 1 8291 8192 /TransformMatrix\n;\n9 () XW";
    let layers = format!(
        "{}{}{}",
        stroked_box_symbol(),
        layer("Jig", true, &square(10, 10)),
        layer("Parts", true, &format!("{}\n{instance}", square(30, 110)))
    );
    let art = native(&layers).replacen("%AI5_NumLayers", "%AI3_TemplateBox: 50.5 50.5 50.5 50.5\n%AI5_NumLayers", 1);
    let (d, warnings, read) = crate::layered_ai(&private(&art), pdf_part(10.0, 86.0), vec![], false, false, 1.0);
    assert!(read, "{warnings:?}");
    assert_eq!(names(&d), ["Jig", "Parts"], "{warnings:?}");
    let parts = d.layers[1].children().unwrap();
    let above = parts[0].visual_bounds().unwrap();
    assert!(above.y1 <= 1e-6, "the square above the artboard is kept, outside it: {above:?}");
    let NodeKind::SymbolInstance { symbol, .. } = &parts[1].kind else { panic!("an instance: {:?} {warnings:?}", parts[1].kind) };
    assert_eq!(symbol, "Box");
    let b = parts[1].visual_bounds_with(&d.symbol_boxes()).unwrap();
    let near = [b.x0 - 149.0, b.y0 - 49.5, b.x1 - 151.0, b.y1 - 50.5].iter().all(|v| v.abs() < 0.3);
    assert!(near, "the instance right of the artboard: {b:?}");
}

/// A `.ai` whose PDF part was read in points from a page of 10 points a unit (its `UserUnit`), and
/// whose editing data has the page in those units: its layers come in at the page's size, ten
/// times as large as the editing data has them, with stroke weights and dashes ten times as large.
/// Editing data with the page in points comes in as it is.
#[test]
fn editing_data_in_the_units_of_a_user_unit_page_comes_in_at_the_pages_size() {
    // A dashed line 2 units wide on a hidden layer, which the page doesn't draw.
    let line = "0 0 1 0 K\n2 w\n[4 2] 1 d\n10 50 m\n50 50 L\nS";
    let art = native(&(layer("Jig", true, &square(10, 10)) + &layer("Parts", true, &square(30, 110)) + &layer("Old", false, line)));
    let page = in_points(pdf_part(10.0, 86.0), 10.0);
    let first = |d: &Document, l: usize| d.layers[l].children().unwrap()[0].visual_bounds().unwrap();
    let near = |r: vectorcraft_geom::Rect, want: [f64; 4]| [r.x0, r.y0, r.x1, r.y1].iter().zip(want).all(|(a, b)| (a - b).abs() < 1e-6);
    let (d, w, read) = crate::layered_ai(&private(&art), page.clone(), vec![], false, false, 10.0);
    assert!(read, "{w:?}");
    assert_eq!(names(&d), ["Jig", "Parts", "Old"]);
    assert_eq!(d.artboards[0].rect, vectorcraft_geom::Rect::new(0.0, 0.0, 1000.0, 1000.0));
    // The square on the artboard and the one above it, ten times as large and as far from its corner.
    assert!(near(first(&d, 0), [100.0, 860.0, 140.0, 900.0]), "{:?}", first(&d, 0));
    assert!(near(first(&d, 1), [300.0, -140.0, 340.0, -100.0]), "{:?}", first(&d, 1));
    let stroke = d.layers[2].children().unwrap()[0].appearance.stroke().cloned().unwrap();
    let dash = stroke.dash.unwrap();
    assert_eq!((stroke.width, dash.pattern, dash.offset), (20.0, vec![40.0, 20.0], 10.0));
    // Read as if its page had no `UserUnit`, the editing data is for another page.
    let (_, w, read) = crate::layered_ai(&private(&art), page.clone(), vec![], false, false, 1.0);
    assert!(!read && w.iter().any(|w| w.contains("for a 100 × 100 pt page, the file's 1000 × 1000 pt page at 1:10")), "{w:?}");
    // Editing data with the page in points.
    let points = native(&layer("Jig", true, "0 0 1 0 k\n100 100 m\n140 100 L\n140 140 L\n100 140 L\nf")).replace(" 0 0 100 100", " 0 0 1000 1000");
    let (d, w, read) = crate::layered_ai(&private(&points), page, vec![], false, false, 10.0);
    assert!(read, "{w:?}");
    assert_eq!(d.artboards[0].rect, vectorcraft_geom::Rect::new(0.0, 0.0, 1000.0, 1000.0));
    assert!(near(first(&d, 0), [100.0, 860.0, 140.0, 900.0]), "{:?}", first(&d, 0));
}

/// The graphic styles of a `.ai` whose editing data is in the units of a `UserUnit` page come in
/// with their stroke weights as large as the art's; the default styles stay as they are.
#[test]
fn graphic_styles_in_the_units_of_a_user_unit_page_come_in_at_the_pages_size() {
    let line = "/KnownStyle :\n(Line) /Name ,\n/ActiveStyle :\n/CompoundFilter :\n(Chain Style Filter) 0 0 /Filter ,\n1 /Visible ,\n\
                /BasicFilter :\n(Stroke Style Filter) 0 0 /Filter ,\n1 /Visible ,\n/Dictionary : /NotRecorded ,\n\
                /StrokeStyle : 0 R\n0 0 0 0 0 0 1 XA\n0 J 0 j 2 w 10 M []0 d\n0 XR\n1 0 Xd\n/Def ;\n (StrokeStyle) ,\n; /Dict ;\n /Part ,\n;\n";
    let styles = format!("%AI9_BeginArtStyles\n{line}%AI9_EndArtStyles\n%AI9_BeginArtStyleList\n(Line)\n%AI9_EndArtStyleList\n");
    let page = in_points(pdf_part(10.0, 86.0), 10.0);
    let art = native(&format!("{styles}{}", layer("Jig", true, &square(10, 10)) + &layer("Parts", true, &square(30, 110))));
    let (d, w, read) = crate::layered_ai(&private(&art), page.clone(), vec![], false, false, 10.0);
    assert!(read, "{w:?}");
    let widths: Vec<(&str, f64)> = d.graphic_styles.iter().map(|g| (g.name.as_str(), g.appearance.stroke_width())).collect();
    assert_eq!(widths, [("Line", 20.0)]);
    let art = native(&(layer("Jig", true, &square(10, 10)) + &layer("Parts", true, &square(30, 110))));
    let (d, w, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 10.0);
    assert!(read, "{w:?}");
    assert_eq!(d.graphic_styles, Document::new(1.0, 1.0).graphic_styles);
}

/// #779: a `.ai`'s ruler guides come in as guides of their layers, at the page's size when the
/// editing data is in the units of a `UserUnit` page.
#[test]
fn an_ai_files_ruler_guides_come_in_on_their_layers() {
    // Across the canvas (16,383 points around the template box's centre, 50 50) at y 40, and
    // across the artboard at x 20.
    let guides = "-8141.5 40 m\n8241.5 40 L\n(N) *\n20 0 m\n20 100 L\n(N) *\n";
    let art = native(&layer("Jig", true, &format!("{}\n{guides}", square(10, 10))));
    let art = art.replacen("%AI5_NumLayers", "%AI3_TemplateBox: 50 50 50 50\n%AI5_NumLayers", 1);
    for k in [1.0, 10.0] {
        let page = in_points(pdf_part(10.0, 86.0), k);
        let (d, w, read) = crate::layered_ai(&private(&art), page, vec![], false, false, k);
        assert!(read, "{w:?}");
        let jig = d.layers[0].id;
        let want = [
            vectorcraft_doc::Guide { layer: Some(jig), ..vectorcraft_doc::Guide::new(false, 60.0 * k) },
            vectorcraft_doc::Guide { artboard: Some(d.artboards[0].id), layer: Some(jig), ..vectorcraft_doc::Guide::new(true, 20.0 * k) },
        ];
        assert_eq!(guide_lines(&d), want);
        assert_eq!(d.layers[0].children().map_or(0, |c| c.iter().filter(|c| !c.is_ruler_guide()).count()), 1, "the square alone");
    }
}


/// A PDF part read from a `UserUnit` page cropped to the art: the art's box, not the artboard, is
/// the page in the editing data's units, so the layers are read at the page's size, with their
/// artboard.
#[test]
fn a_user_unit_page_cropped_to_the_art_is_read_in_its_units() {
    // A 50 × 50 artboard, art over 100 × 100 (its bounding box), and a page of that box at 10 pt a
    // unit: 1000 × 1000 pt.
    let art = native(&layer("Jig", true, &square(10, 10))).replace("%AI3_Cropmarks: 0 0 100 100", "%AI3_Cropmarks: 0 0 50 50");
    let page = in_points(pdf_part(10.0, 86.0), 10.0);
    let (d, w, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 10.0);
    assert!(read, "{w:?}");
    assert_eq!(d.artboards[0].rect.size(), vectorcraft_geom::Size::new(500.0, 500.0));
    // The square, 36 down the artboard, ten times as large and as far.
    let b = d.layers[0].children().unwrap()[0].visual_bounds().unwrap();
    assert!([b.x0 - 100.0, b.y0 - 360.0, b.x1 - 140.0, b.y1 - 400.0].iter().all(|v| v.abs() < 1e-6), "{b:?}");
}

/// A PDF part whose pages are each cropped to all of the art holds the art once a page: the type
/// of the pages after the first is that type again, and only the first page's goes into the
/// layers.
#[test]
fn type_on_the_copies_of_a_page_cropped_to_the_art_is_read_once() {
    // A 50 × 50 artboard, art over 100 × 100, a text object with no story in the file, and two
    // pages of the art's box in a row, 20 apart, each with the square and the type.
    let art = native(&layer("Art", true, &(square(10, 10) + &text_object(0)))).replace("%AI3_Cropmarks: 0 0 100 100", "%AI3_Cropmarks: 0 0 50 50");
    let mut pdf = pdf_part(10.0, 86.0);
    let mut second = pdf.artboards[0].clone();
    (second.id, second.rect) = (2, vectorcraft_geom::Rect::new(120.0, 0.0, 220.0, 100.0));
    pdf.artboards.push(second);
    let copy = pdf_part(130.0, 86.0).layers[0].children().unwrap()[0].as_ref().clone();
    let layer_id = pdf.layers[0].id;
    let id = pdf.alloc_id();
    pdf.insert(Some(layer_id), 1, Node { id, ..copy }).unwrap();
    for x in [30.0, 150.0] {
        let style = vectorcraft_doc::CharStyle { size: 14.0, ..Default::default() };
        let text = vectorcraft_doc::TextObject::point(vectorcraft_geom::Point::new(x, 40.0), "Hi", style);
        let id = pdf.alloc_id();
        pdf.insert(Some(layer_id), 2, Node::new(id, NodeKind::Text(Box::new(text)))).unwrap();
    }
    let (d, w, read) = crate::layered_ai(&private(&art), pdf, vec![], false, false, 1.0);
    assert!(read, "{w:?}");
    let texts = text_of(&d.layers[0]);
    // The first page's type, where the art has it: 50 above the artboard's top.
    assert_eq!(texts.iter().map(|t| t.plain_text()).collect::<Vec<_>>(), ["Hi"], "{w:?}");
    let [.., e, f] = texts[0].xf.as_coeffs();
    assert!((e - 30.0).abs() < 1e-6 && (f + 10.0).abs() < 1e-6, "{:?}", texts[0].xf);
}

/// A PDF saved with only the second of its two artboards has a page for that one, while its
/// editing data keeps both: the layers are compared with the page on that artboard, and the page's
/// type goes there.
#[test]
fn a_pdf_of_its_second_artboard_is_compared_on_that_artboard() {
    // Two 100-point artboards side by side, 50 apart (art space, y up), a square on each, and a
    // text object with no story in the file.
    let board = |x0: i32| {
        format!(
            "%_/Dictionary :\n%_{x0} 100 /RealPointRelToROrigin\n%_ (PositionPoint1) ,\n%_{} 0 /RealPointRelToROrigin\n%_ (PositionPoint2) ,\n%_; ,\n",
            x0 + 100
        )
    };
    let boards = format!("%_/Document :\n%_/Dictionary :\n%_/Array :\n{}{}%_; (ArtboardArray) ,\n%_; /NotRecorded ,\n%_;\n", board(0), board(150));
    let art = native(&(boards + &layer("Art", true, &(square(10, 10) + "\n" + &square(180, 50) + "\n" + &text_object(0)))));
    // The page: the second artboard, its square 30 from the left and 46 down, and the type.
    let mut pdf = pdf_part(30.0, 46.0);
    let style = vectorcraft_doc::CharStyle { size: 14.0, ..Default::default() };
    let text = vectorcraft_doc::TextObject::point(vectorcraft_geom::Point::new(40.0, 20.0), "Hi", style);
    let (layer_id, id) = (pdf.layers[0].id, pdf.alloc_id());
    pdf.insert(Some(layer_id), 1, Node::new(id, NodeKind::Text(Box::new(text)))).unwrap();
    let (d, w, read) = crate::layered_ai(&private(&art), pdf, vec![], false, false, 1.0);
    assert!(read, "{w:?}");
    assert_eq!(d.artboards.len(), 2, "both artboards are kept");
    let texts = text_of(&d.layers[0]);
    assert_eq!(texts.iter().map(|t| t.plain_text()).collect::<Vec<_>>(), ["Hi"], "{w:?}");
    let [.., e, f] = texts[0].xf.as_coeffs();
    assert!((e - 190.0).abs() < 1e-6 && (f - 20.0).abs() < 1e-6, "on the second artboard: {:?}", texts[0].xf);
}

/// Two 100-point artboards, the second 50 below the first (art space, y up), as the editing data
/// lists them.
const TWO_BOARDS: &str = "%_/Document :\n%_/Dictionary :\n%_/Array :\n\
    %_/Dictionary :\n%_0 100 /RealPointRelToROrigin\n%_ (PositionPoint1) ,\n%_100 0 /RealPointRelToROrigin\n%_ (PositionPoint2) ,\n%_; ,\n\
    %_/Dictionary :\n%_0 -50 /RealPointRelToROrigin\n%_ (PositionPoint1) ,\n%_100 -150 /RealPointRelToROrigin\n%_ (PositionPoint2) ,\n%_; ,\n\
    %_; (ArtboardArray) ,\n%_; /NotRecorded ,\n%_;\n";

/// The PDF part of a file with [`TWO_BOARDS`]: a page for each artboard in a row, the first with a
/// square at (10, 86), and the second with one at (`x`, `y`) of its own when given.
fn two_pages(second: Option<(f64, f64)>) -> Document {
    let mut page = pdf_part(10.0, 86.0);
    let mut board = page.artboards[0].clone();
    (board.id, board.rect) = (2, vectorcraft_geom::Rect::new(120.0, 0.0, 220.0, 100.0));
    page.artboards.push(board);
    if let Some((x, y)) = second {
        let other = pdf_part(120.0 + x, y);
        let square = page.reid(&other.layers[0].children().unwrap()[0]);
        let layer = page.layers[0].id;
        page.insert(Some(layer), 1, square).unwrap();
    }
    page
}

/// The layers are compared with every page: art that they draw on the second artboard and its page
/// doesn't sends the file to its PDF part, as it would on the first.
#[test]
fn art_only_the_layers_draw_on_the_second_artboard_makes_the_file_come_in_as_its_pdf_part() {
    // A 60-point square on the second artboard, 20 from its left and 20 down.
    let big = "0 0 1 0 k\n20 -130 m\n80 -130 L\n80 -70 L\n20 -70 L\nf";
    let art = native(&(TWO_BOARDS.to_string() + &layer("Live", true, &square(10, 10)) + &layer("Second", true, big)));
    let (d, w, read) = crate::layered_ai(&private(&art), two_pages(None), vec![], false, false, 1.0);
    assert!(!read, "{w:?}");
    assert_ne!(names(&d), ["Live", "Second"], "{w:?}");
    assert!(w.iter().any(|w| w.contains("differs from the page's")), "{w:?}");
    // The same square on the first artboard, which its page doesn't draw either.
    let big = "0 0 1 0 k\n20 20 m\n80 20 L\n80 80 L\n20 80 L\nf";
    let art = native(&(TWO_BOARDS.to_string() + &layer("Live", true, &square(10, 10)) + &layer("Second", true, big)));
    let (_, w, read) = crate::layered_ai(&private(&art), two_pages(None), vec![], false, false, 1.0);
    assert!(!read && w.iter().any(|w| w.contains("differs from the page's")), "{w:?}");
}

/// Art that the second page draws and the layers don't sends the file to its PDF part too.
#[test]
fn art_only_the_second_page_draws_makes_the_file_come_in_as_its_pdf_part() {
    let art = native(&(TWO_BOARDS.to_string() + &layer("Live", true, &square(10, 10))));
    // A 60-point square on the second page, 20 from its left and 20 down.
    let mut page = two_pages(None);
    let path = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(140.0, 20.0, 200.0, 80.0));
    let paint = vectorcraft_color::Paint::solid(vectorcraft_color::Color::cmyk(0.0, 0.0, 0.0, 1.0));
    let appearance = vectorcraft_doc::Appearance {
        items: vec![vectorcraft_doc::AppearanceItem::Fill(vectorcraft_doc::FillLayer::new(paint))],
        ..Default::default()
    };
    let node = Node::path(page.alloc_id(), path, appearance);
    let layer_id = page.layers[0].id;
    page.insert(Some(layer_id), 1, node).unwrap();
    let (_, w, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(!read, "{w:?}");
    assert!(w.iter().any(|w| w.contains("differs from the page's")), "{w:?}");
}

/// A PDF saved with some of the artboards, not starting with the first, can hold any of the
/// others after it: its later pages aren't compared with the artboards that follow the first.
#[test]
fn the_later_pages_of_a_pdf_of_some_artboards_are_not_compared() {
    // Four 100-point artboards side by side, 50 apart (art space, y up), a 4-point square on the
    // first, second and fourth, and a 60-point square on the third.
    let board = |x0: i32| {
        format!(
            "%_/Dictionary :\n%_{x0} 100 /RealPointRelToROrigin\n%_ (PositionPoint1) ,\n%_{} 0 /RealPointRelToROrigin\n%_ (PositionPoint2) ,\n%_; ,\n",
            x0 + 100
        )
    };
    let boards = format!(
        "%_/Document :\n%_/Dictionary :\n%_/Array :\n{}{}{}{}%_; (ArtboardArray) ,\n%_; /NotRecorded ,\n%_;\n",
        board(0),
        board(150),
        board(300),
        board(450)
    );
    let big = "0 0 1 0 k\n320 20 m\n380 20 L\n380 80 L\n320 80 L\nf";
    let art = native(&(boards + &layer("Art", true, &(square(10, 10) + "\n" + &square(180, 50) + "\n" + big + "\n" + &square(460, 30)))));
    // The PDF part: a page for the second artboard, then one for the fourth.
    let mut pdf = pdf_part(30.0, 46.0);
    let mut second = pdf.artboards[0].clone();
    (second.id, second.rect) = (2, vectorcraft_geom::Rect::new(120.0, 0.0, 220.0, 100.0));
    pdf.artboards.push(second);
    let other = pdf_part(130.0, 66.0);
    let fourth = pdf.reid(&other.layers[0].children().unwrap()[0]);
    let layer_id = pdf.layers[0].id;
    pdf.insert(Some(layer_id), 1, fourth).unwrap();
    let (d, w, read) = crate::layered_ai(&private(&art), pdf, vec![], false, false, 1.0);
    assert!(read, "{w:?}");
    assert_eq!(d.artboards.len(), 4, "{w:?}");
}

/// The editing data of four 100-point artboards side by side, 50 apart (art space, y up), with a
/// 4-point square on each at (`squares[i]`, 10 + 20 × i), and a text object with no story in the
/// file.
fn four_boards_with_type(squares: [u32; 4]) -> String {
    let board = |x0: u32| {
        format!(
            "%_/Dictionary :\n%_{x0} 100 /RealPointRelToROrigin\n%_ (PositionPoint1) ,\n%_{} 0 /RealPointRelToROrigin\n%_ (PositionPoint2) ,\n%_; ,\n",
            x0 + 100
        )
    };
    let boards = (0..4u32).map(|i| board(i * 150)).collect::<String>();
    let boards = format!("%_/Document :\n%_/Dictionary :\n%_/Array :\n{boards}%_; (ArtboardArray) ,\n%_; /NotRecorded ,\n%_;\n");
    let art: Vec<String> = (0..4u32).map(|i| square(i * 150 + squares[i as usize], 10 + 20 * i)).collect();
    native(&(boards + &layer("Art", true, &(art.join("\n") + "\n" + &text_object(0)))))
}

/// The PDF part of a file with [`four_boards_with_type`] saved with two of its artboards: a page
/// for each in a row, page `i` with a square `squares[i].0` from its left and `squares[i].1` up
/// from its bottom, and "Hi" 30 from the second page's left and 20 down.
fn two_pages_with_type(squares: [(f64, u32); 2]) -> Document {
    let at = |(x, y): (f64, u32), page: f64| pdf_part(page + x, 100.0 - f64::from(y) - 4.0);
    let mut pdf = at(squares[0], 0.0);
    let mut second = pdf.artboards[0].clone();
    (second.id, second.rect) = (2, vectorcraft_geom::Rect::new(120.0, 0.0, 220.0, 100.0));
    pdf.artboards.push(second);
    let other = at(squares[1], 120.0);
    let square = pdf.reid(&other.layers[0].children().unwrap()[0]);
    let layer_id = pdf.layers[0].id;
    pdf.insert(Some(layer_id), 1, square).unwrap();
    let style = vectorcraft_doc::CharStyle { size: 14.0, ..Default::default() };
    let text = vectorcraft_doc::TextObject::point(vectorcraft_geom::Point::new(150.0, 20.0), "Hi", style);
    let id = pdf.alloc_id();
    pdf.insert(Some(layer_id), 2, Node::new(id, NodeKind::Text(Box::new(text)))).unwrap();
    pdf
}

/// A PDF saved with artboards that don't follow one another has a page for each: the type on a
/// later page goes to the artboard that page shows, not to the one after the previous page's.
#[test]
fn type_on_a_later_page_of_a_pdf_of_some_artboards_goes_to_its_artboard() {
    // The squares 10, 20, 30 and 40 from each artboard's left.
    let art = four_boards_with_type([10, 20, 30, 40]);
    // Pages for the second and fourth artboards. The fourth artboard starts at x 450, so the type
    // 30 from its left is at x 480.
    let (d, w, read) = crate::layered_ai(&private(&art), two_pages_with_type([(20.0, 30), (40.0, 70)]), vec![], false, false, 1.0);
    assert!(read, "{w:?}");
    assert_eq!(d.artboards.len(), 4, "{w:?}");
    let texts = text_of(&d.layers[0]);
    assert_eq!(texts.iter().map(|t| t.plain_text()).collect::<Vec<_>>(), ["Hi"], "{w:?}");
    let [.., e, f] = texts[0].xf.as_coeffs();
    assert!((e - 480.0).abs() < 1e-6 && (f - 20.0).abs() < 1e-6, "on the fourth artboard: {:?}", texts[0].xf);
}

/// A PDF saved with the first and third of four artboards: the type on its second page goes to
/// the third artboard.
#[test]
fn type_on_the_second_page_of_a_pdf_of_the_first_and_third_artboards_goes_to_the_third() {
    let art = four_boards_with_type([10, 20, 30, 40]);
    // The third artboard starts at x 300, so the type 30 from its left is at x 330.
    let (d, w, read) = crate::layered_ai(&private(&art), two_pages_with_type([(10.0, 10), (30.0, 50)]), vec![], false, false, 1.0);
    assert!(read, "{w:?}");
    assert_eq!(d.artboards.len(), 4, "{w:?}");
    let texts = text_of(&d.layers[0]);
    assert_eq!(texts.iter().map(|t| t.plain_text()).collect::<Vec<_>>(), ["Hi"], "{w:?}");
    let [.., e, f] = texts[0].xf.as_coeffs();
    assert!((e - 330.0).abs() < 1e-6 && (f - 20.0).abs() < 1e-6, "on the third artboard: {:?}", texts[0].xf);
}


/// Layers that draw each page's art on its artboard come in.
#[test]
fn layers_that_match_every_page_come_in() {
    // A square on the second artboard, 50 from its left and 46 down, which its page draws too.
    let second = "0 0 1 0 k\n50 -100 m\n54 -100 L\n54 -96 L\n50 -96 L\nf";
    let art = native(&(TWO_BOARDS.to_string() + &layer("Live", true, &square(10, 10)) + &layer("Second", true, second)));
    let (d, w, read) = crate::layered_ai(&private(&art), two_pages(Some((50.0, 46.0))), vec![], false, false, 1.0);
    assert!(read, "{w:?}");
    assert_eq!(names(&d), ["Live", "Second"], "{w:?}");
    assert!(w.is_empty(), "{w:?}");
}

/// Layers that draw each page's art come in from [`TWO_BOARDS`] changed to `boards`.
fn two_boards_come_in(boards: String) {
    let second = "0 0 1 0 k\n50 -100 m\n54 -100 L\n54 -96 L\n50 -96 L\nf";
    let art = native(&(boards + &layer("Live", true, &square(10, 10)) + &layer("Second", true, second)));
    let (d, w, read) = crate::layered_ai(&private(&art), two_pages(Some((50.0, 46.0))), vec![], false, false, 1.0);
    assert!(read, "{w:?}");
    assert_eq!(names(&d), ["Live", "Second"], "{w:?}");
}

/// A first artboard up to half a point larger than its page is compared on an area of the page's
/// size.
#[test]
fn a_first_artboard_a_little_wider_than_its_page_is_compared_on_the_pages_size() {
    two_boards_come_in(TWO_BOARDS.replace("%_100 0 /", "%_100.3 0 /"));
}

/// A later artboard up to half a point smaller than its page is compared on an area of the page's
/// size.
#[test]
fn a_second_artboard_a_little_shorter_than_its_page_is_compared_on_the_pages_size() {
    two_boards_come_in(TWO_BOARDS.replace("%_100 -150", "%_100 -149.7"));
}

/// A text object the second page doesn't draw is left out even when the first page differs by more
/// than the comparison allows.
#[test]
fn an_unseen_text_object_on_the_second_page_is_weighed_while_the_first_page_differs() {
    let doc = text_document(&[("Ghost\r", 0, (8200.0, 8330.0), (0.0, 0.0), 0.0)]);
    let big = "0 0 1 0 k\n20 20 m\n80 20 L\n80 80 L\n20 80 L\nf";
    let art = native_with_text(&(TWO_BOARDS.to_string() + &layer("Art", true, &(square(10, 10) + "\n" + big + "\n" + &text_object(0)))), &doc);
    // The first page draws the 60-point square in black.
    let mut page = two_pages(None);
    let path = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(20.0, 20.0, 80.0, 80.0));
    let paint = vectorcraft_color::Paint::solid(vectorcraft_color::Color::cmyk(0.0, 0.0, 0.0, 1.0));
    let appearance = vectorcraft_doc::Appearance {
        items: vec![vectorcraft_doc::AppearanceItem::Fill(vectorcraft_doc::FillLayer::new(paint))],
        ..Default::default()
    };
    let node = Node::path(page.alloc_id(), path, appearance);
    let layer_id = page.layers[0].id;
    page.insert(Some(layer_id), 1, node).unwrap();
    let (d, w, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(read, "{w:?}");
    let mut texts = vec![];
    d.walk(|n| {
        if let NodeKind::Text(t) = &n.kind {
            texts.push(t.plain_text());
        }
    });
    assert!(texts.is_empty(), "{w:?}");
    assert!(w.iter().any(|w| w.contains("doesn't draw")), "{w:?}");
}

/// A text object on the second artboard that its page doesn't draw is weighed as one on the first
/// is: it is taken out, unless its layer doesn't print.
#[test]
fn a_text_object_the_second_page_doesnt_draw_is_left_out_unless_its_layer_doesnt_print() {
    // "Ghost" 38.5 down the second artboard (150 below the first's anchor in the canvas).
    let doc = text_document(&[("Ghost\r", 0, (8200.0, 8330.0), (0.0, 0.0), 0.0)]);
    let all_text = |d: &Document| {
        let mut all = vec![];
        d.walk(|n| {
            if let NodeKind::Text(t) = &n.kind {
                all.push(t.plain_text());
            }
        });
        all
    };
    let art = native_with_text(&(TWO_BOARDS.to_string() + &layer("Art", true, &(square(10, 10) + "\n" + &text_object(0)))), &doc);
    let (d, w, read) = crate::layered_ai(&private(&art), two_pages(None), vec![], false, false, 1.0);
    assert!(read, "{w:?}");
    assert!(all_text(&d).is_empty(), "{w:?}");
    assert!(w.iter().any(|w| w.contains("doesn't draw")), "{w:?}");
    // On a layer that shows and doesn't print, it stays.
    let tips = layer("Tips", true, &text_object(0)).replace("1 1 1 1 0 0 1 0 79", "1 1 1 0 0 0 1 0 79");
    let art = native_with_text(&(TWO_BOARDS.to_string() + &layer("Art", true, &square(10, 10)) + &tips), &doc);
    let (d, w, read) = crate::layered_ai(&private(&art), two_pages(None), vec![], false, false, 1.0);
    assert!(read, "{w:?}");
    assert_eq!(names(&d), ["Art", "Tips"], "{w:?}");
    assert_eq!(all_text(&d), ["Ghost"], "{w:?}");
    assert!(!w.iter().any(|w| w.contains("doesn't draw")), "{w:?}");
}

#[test]
fn an_ai_file_that_cant_be_read_through_its_editing_data_keeps_its_pdf_part() {
    let pdf = pdf_part(10.0, 86.0);
    // No editing data this reads.
    let (d, w, read) = crate::layered_ai(b"nothing", pdf.clone(), vec![], false, false, 1.0);
    assert_eq!(d.layers.len(), pdf.layers.len());
    assert!(w.is_empty() && !read);
    // Editing data that isn't for this page: neither its artboard nor its art's box is the page.
    let other = native(&layer("Jig", true, &square(10, 10))).replace(" 0 0 100 100", " 0 0 50 50");
    let (d, w, _) = crate::layered_ai(&private(&other), pdf.clone(), vec![], false, false, 1.0);
    assert_eq!(names(&d), names(&pdf));
    assert!(w.iter().any(|w| w.contains("for a 50 × 50 pt page, the file's 100 × 100 pt page at 1:2")), "{w:?}");
    // A page cropped to the art's box, not the artboard, is that art's page: the layers are read,
    // with their artboard.
    let cropped = native(&layer("Jig", true, &square(10, 10))).replace("%AI3_Cropmarks: 0 0 100 100", "%AI3_Cropmarks: 0 0 50 50");
    let (d, w, read) = crate::layered_ai(&private(&cropped), pdf.clone(), vec![], false, false, 1.0);
    assert!(read, "{w:?}");
    assert_eq!(names(&d), ["Jig"]);
    assert_eq!(d.artboards[0].rect.size(), vectorcraft_geom::Size::new(50.0, 50.0));
}




/// Editing data that can't be decompressed leaves the `.ai` as its PDF part, after its notes, with a
/// note saying why its layers weren't read, which counts as a loss.
#[test]
fn an_ai_file_whose_editing_data_cant_be_decompressed_says_so() {
    let pdf = pdf_part(10.0, 86.0);
    for (private, why) in [
        (&b"%AI12_CompressedDataxx"[..], "its zlib compression is damaged"),
        (b"%AI24_ZStandard_Data nothing", "its Zstandard compression is damaged"),
    ] {
        let (d, w, read) = crate::layered_ai(private, pdf.clone(), vec!["a note".into()], false, false, 1.0);
        assert!(!read);
        assert_eq!(names(&d), names(&pdf));
        assert_eq!(w, ["a note".to_string(), format!("the file's layers weren't read from its editing data ({why}): they come from its PDF part")]);
        assert!(crate::is_loss(&w[1]), "{w:?}");
    }
}



/// The definition of the symbol Box: a rectangle 2 × 1 about the origin, stroked 0.4 wide.
fn stroked_box_symbol() -> String {
    let art = "0 0 0 1 K\n0.4 w\n-1 -0.5 m\n-1 0.5 L\n1 0.5 L\n1 -0.5 L\n-1 -0.5 L\nS\n";
    format!("%AI5_Begin_NonPrinting\n{}%AI5_End_NonPrinting--\n", symbol_def("Box", art, 5))
}

/// The art of `d`'s symbol Box ([`stroked_box_symbol`]) is ten times as large, its stroke 4 wide,
/// and its instance on `d`'s second layer, placed in the file at twice its size about (`x`, `y`),
/// keeps that scale and draws a box of 40 × 20 about `center`.
fn box_symbol_at_ten_times(d: &Document, center: (f64, f64), notes: &[String]) {
    let art = &d.symbols.iter().find(|s| s.name == "Box").unwrap_or_else(|| panic!("{notes:?}")).art;
    let b = art.geometric_bounds().unwrap();
    let close = |r: vectorcraft_geom::Rect, e: [f64; 4]| [r.x0, r.y0, r.x1, r.y1].iter().zip(e).all(|(a, b)| (a - b).abs() < 1e-6);
    assert!(close(b, [-10.0, -5.0, 10.0, 5.0]), "the art at the page's size: {b:?} {notes:?}");
    let mut widths = vec![];
    art.walk(&mut |n| {
        if matches!(n.kind, NodeKind::Path { .. }) {
            widths.push(n.appearance.stroke_width());
        }
    });
    assert_eq!(widths, [4.0], "{notes:?}");
    let instance = d.layers[1].children().unwrap().first().unwrap_or_else(|| panic!("{:#?} {notes:?}", d.layers));
    let NodeKind::SymbolInstance { xf, .. } = &instance.kind else { panic!("an instance: {:?}", instance.kind) };
    let [a, b0, c, d0, ..] = xf.as_coeffs();
    assert!((a.hypot(b0) - 2.0).abs() < 1e-9 && (c.hypot(d0) - 2.0).abs() < 1e-9, "the instance's own scale: {xf:?}");
    let (x, y) = center;
    assert!(close(xf.transform_rect_bbox(b), [x - 20.0, y - 10.0, x + 20.0, y + 10.0]), "{:?}", xf.transform_rect_bbox(b));
}


/// A large canvas `.ai` 100 units square at a scale of 10, with the symbol Box
/// ([`stroked_box_symbol`]) and an instance of it at twice its size about (50, 50), its header
/// carrying `creator` when given, read with its PDF part, on a page whose PDF part can't be read,
/// and without a page.
fn large_canvas_ai_box_reads(creator: Option<&str>) -> Vec<(&'static str, Document, Vec<String>)> {
    let instance = "/SymbolInstance :\n(Box) /SymbolRef ,\n2 0 0 -2 50 50 /RTransformMatrix\n;\n9 () XW";
    let layers = format!("{}{}{}", stroked_box_symbol(), layer("Art", true, &square(10, 10)), layer("Box", false, instance));
    let mut art = native_large_canvas(100.0, 10.0, &layers);
    if let Some(creator) = creator {
        art = art.replacen("%!PS-Adobe-3.0 \n", &format!("%!PS-Adobe-3.0 \n%%Creator: {creator}\n"), 1);
    }
    let data = private(&art);
    let (d, notes, read) = crate::layered_ai(&data, in_points(pdf_part(10.0, 86.0), 10.0), vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    let on_page = crate::ai_alone_on_page(&data, (1000.0, 1000.0), 10.0, Default::default()).unwrap();
    let no_page = crate::ai_alone(&data).unwrap();
    vec![("PDF part", d, notes), ("on a page", on_page.0, on_page.1), ("no page", no_page.0, no_page.1)]
}

/// A `.ai` whose editing data is read at its declared large canvas scale has its symbols scaled
/// with the art, as an EPS has, however the file opens.
#[test]
fn a_large_canvas_ai_keeps_its_symbols_at_the_pages_size() {
    for (how, d, notes) in large_canvas_ai_box_reads(None) {
        let notes: Vec<String> = notes.into_iter().chain([how.to_string()]).collect();
        box_symbol_at_ten_times(&d, (500.0, 500.0), &notes);
    }
}

/// A large canvas `.ai` that VectorCraft wrote (its `%%Creator`) has its symbols defined at their
/// own size, as its `.ai` save writes them: their art keeps that size, and the instance, scaled with
/// the layers, draws the same box, however the file opens.
#[test]
fn a_large_canvas_ai_vectorcraft_wrote_keeps_its_symbols_at_their_own_size() {
    for (how, d, notes) in large_canvas_ai_box_reads(Some(crate::markers::CREATOR_TEXT)) {
        let art = &d.symbols.iter().find(|s| s.name == "Box").unwrap_or_else(|| panic!("{how}: {notes:?}")).art;
        let b = art.geometric_bounds().unwrap();
        assert_eq!(b, vectorcraft_geom::Rect::new(-1.0, -0.5, 1.0, 0.5), "{how}: {notes:?}");
        let instance = d.layers[1].children().unwrap().first().unwrap_or_else(|| panic!("{how}: {:#?} {notes:?}", d.layers));
        let NodeKind::SymbolInstance { xf, .. } = &instance.kind else { panic!("{how}: an instance: {:?}", instance.kind) };
        let drawn = xf.transform_rect_bbox(b);
        let close = [drawn.x0 - 480.0, drawn.y0 - 490.0, drawn.x1 - 520.0, drawn.y1 - 510.0].iter().all(|v| v.abs() < 1e-6);
        assert!(close, "{how}: {drawn:?}");
    }
}

#[test]
fn type_with_a_listed_art_style_is_linked_to_its_graphic_style() {
    let doc = text_document(&[("Bang\r", 0, (8200.0, 8180.0), (0.0, 0.0), 0.0)]);
    let drawing = "0 0 1 0 k\n54 56 m\n100 56 L\n100 80 L\n54 80 L\nf";
    let look = format!("u\n/AI11Text :\n1 /FreeUndo ,\n0 /FrameIndex ,\n0 /StoryIndex ,\n;\n{drawing}\nU\n9 () XW\n");
    let own: String = text_object(0).lines().map(|l| format!("%_{l}\n")).collect();
    let list = "%AI9_BeginArtStyleList\n(Comic)\n%AI9_EndArtStyleList\n";
    let art = native_with_text(&(comic_style() + list + &layer("Art", true, &format!("{look}{own}1 (Comic) XW\n"))), &doc);
    let (d, notes) = crate::ai_alone(&private(&art)).unwrap();
    assert_eq!(d.graphic_styles.iter().map(|g| (g.name.as_str(), g.id)).collect::<Vec<_>>(), [("Comic", 1)]);
    let mut texts = vec![];
    d.walk(|n| {
        if matches!(n.kind, NodeKind::Text(_)) {
            texts.push(n.extras().graphic_style);
        }
    });
    assert_eq!(texts, [Some(1)], "{notes:?}");
}

/// A `.ai` whose layers can't be read comes in as its PDF part, a page per artboard in a row; its
/// pages still go where its editing data puts its artboards, with their art, and keep their names (#1068).
#[test]
fn an_ai_file_read_as_its_pdf_part_keeps_its_artboard_layout() {
    // Two 100-point artboards, the second below the first (art space, y up).
    let board = |x0: i32, y0: i32| {
        format!(
            "%_/Dictionary :\n%_{x0} {} /RealPointRelToROrigin\n%_ (PositionPoint1) ,\n%_{} {y0} /RealPointRelToROrigin\n%_ (PositionPoint2) ,\n%_(at {x0} {y0}) /UnicodeString (Name) ,\n%_; ,\n",
            y0 + 100,
            x0 + 100
        )
    };
    let boards = |list: &[(i32, i32)]| {
        let all: String = list.iter().map(|(x, y)| board(*x, *y)).collect();
        format!("%_/Document :\n%_/Dictionary :\n%_/Array :\n{all}%_; (ArtboardArray) ,\n%_; /NotRecorded ,\n%_;\n")
    };
    // An operator this doesn't know, so the layers aren't read.
    let unread = |list: &[(i32, i32)]| native(&(boards(list) + &layer("Art", true, &(square(10, 10) + "\n1 Zz"))));
    // The PDF part: the pages side by side, 20 points apart, a square on each.
    let mut pdf = pdf_part(10.0, 10.0);
    let second = pdf_part(130.0, 10.0).layers[0].children().unwrap()[0].clone();
    let mut second = (*second).clone();
    second.id = pdf.alloc_id();
    let layer_id = pdf.layers[0].id;
    pdf.insert(Some(layer_id), 1, second).unwrap();
    let mut page = pdf.artboards[0].clone();
    (page.id, page.rect) = (2, vectorcraft_geom::Rect::new(120.0, 0.0, 220.0, 100.0));
    pdf.artboards.push(page);
    let squares = |d: &Document| d.layers[0].children().unwrap().iter().map(|n| n.visual_bounds().unwrap().origin()).collect::<Vec<_>>();
    let (d, w, read) = crate::layered_ai(&private(&unread(&[(0, 0), (0, -150)])), pdf.clone(), vec![], false, false, 1.0);
    assert!(!read && w.iter().any(|w| w.contains("layers weren't read")), "{w:?}");
    let rects: Vec<_> = d.artboards.iter().map(|a| a.rect).collect();
    assert_eq!(rects, [vectorcraft_geom::Rect::new(0.0, 0.0, 100.0, 100.0), vectorcraft_geom::Rect::new(0.0, 150.0, 100.0, 250.0)]);
    let board_names: Vec<_> = d.artboards.iter().map(|a| a.name.as_str()).collect();
    assert_eq!(board_names, ["at 0 0", "at 0 -150"]);
    let moved = squares(&d);
    assert!((moved[0] - squares(&pdf)[0]).hypot() < 1e-6, "the first page's art stays: {moved:?}");
    assert!((moved[1] - vectorcraft_geom::Point::new(10.0, 160.0)).hypot() < 1e-6, "the second's goes with its page: {moved:?}");
    // Artboards that don't match the pages leave them in a row.
    for list in [&[(0, 0)][..], &[(0, 0), (0, -150), (200, 0)]] {
        let (d, _, _) = crate::layered_ai(&private(&unread(list)), pdf.clone(), vec![], false, false, 1.0);
        assert_eq!(d.artboards[1].rect, pdf.artboards[1].rect, "{list:?}");
        assert_eq!(d.artboards[1].name, pdf.artboards[1].name, "{list:?}");
    }
    // Pages read in points from a user space of 10 points a unit, with artboards in those units:
    // the same layout, ten times as large.
    let (d, _, read) = crate::layered_ai(&private(&unread(&[(0, 0), (0, -150)])), in_points(pdf.clone(), 10.0), vec![], false, false, 10.0);
    assert!(!read);
    let rects: Vec<_> = d.artboards.iter().map(|a| a.rect).collect();
    assert_eq!(rects, [vectorcraft_geom::Rect::new(0.0, 0.0, 1000.0, 1000.0), vectorcraft_geom::Rect::new(0.0, 1500.0, 1000.0, 2500.0)]);
    let moved = squares(&d);
    assert!((moved[1] - vectorcraft_geom::Point::new(100.0, 1600.0)).hypot() < 1e-6, "the second's art goes with its page: {moved:?}");
}

/// A `.ai` read as its PDF part opens on the page of the artboard its editing data names as the
/// active one (`CropAreaActive`), also when the reading stops before the artboards.
#[test]
fn an_ai_file_read_as_its_pdf_part_keeps_its_active_artboard() {
    let board = |x0: i32| {
        format!(
            "%_/Dictionary :\n%_{x0} 100 /RealPointRelToROrigin\n%_ (PositionPoint1) ,\n%_{} 0 /RealPointRelToROrigin\n%_ (PositionPoint2) ,\n%_; ,\n",
            x0 + 100
        )
    };
    // `before` in the document's data, before the artboards; an operator this doesn't know, so the
    // layers aren't read.
    let unread = |before: &str, active: &str| {
        let boards = format!(
            "%_/Document :\n%_/Dictionary :\n{before}%_/Array :\n{}{}%_; (ArtboardArray) ,\n{active}%_; /NotRecorded ,\n%_;\n",
            board(0),
            board(120)
        );
        native(&(boards + &layer("Art", true, &(square(10, 10) + "\n1 Zz"))))
    };
    let mut pdf = pdf_part(10.0, 10.0);
    let mut page = pdf.artboards[0].clone();
    (page.id, page.rect) = (2, vectorcraft_geom::Rect::new(120.0, 0.0, 220.0, 100.0));
    pdf.artboards.push(page);
    let opened = |before: &str, active: &str| {
        let (d, w, read) = crate::layered_ai(&private(&unread(before, active)), pdf.clone(), vec![], false, false, 1.0);
        assert!(!read && w.iter().any(|w| w.contains("layers weren't read")), "{w:?}");
        d.opened_artboard
    };
    let stops = "%_/Array :\n%_/Dictionary :\nLB\n";
    // Another entry, a dictionary of its own, between the array and the entry.
    let between = "%_/Dictionary :\n%_1 /Int (Other) ,\n%_; (Inner) ,\n%_1 /Int (CropAreaActive) ,\n";
    for before in ["", stops] {
        assert_eq!(opened(before, "%_1 /Int (CropAreaActive) ,\n"), Some(2), "{before}");
        assert_eq!(opened(before, between), Some(2), "{before}");
        assert_eq!(opened(before, "%_0 /Int (CropAreaActive) ,\n"), None, "{before}");
        assert_eq!(opened(before, ""), None, "{before}");
        // A `CropAreaActive` of a dictionary inside the holder is not the holder's.
        assert_eq!(opened(before, "%_/Dictionary :\n%_1 /Int (CropAreaActive) ,\n%_; (Inner) ,\n"), None, "{before}");
    }
    // The entry before the array, after what stops the reading.
    assert_eq!(opened(&format!("{stops}%_; ,\n%_; (Art) ,\n%_1 /Int (CropAreaActive) ,\n"), ""), Some(2));
    // What stops the reading between the array and the entry.
    assert_eq!(opened("", "%_/Dictionary :\nLB\n%_; (Inner) ,\n%_1 /Int (CropAreaActive) ,\n"), Some(2));
    // Lines ended by carriage returns.
    let (d, ..) =
        crate::layered_ai(&private(&unread(stops, "%_1 /Int (CropAreaActive) ,\n").replace('\n', "\r")), pdf.clone(), vec![], false, false, 1.0);
    assert_eq!(d.opened_artboard, Some(2));
}

/// A `.ai` whose editing data has artboards and no layers comes in as its PDF part, open on the
/// page of the artboard the editing data names as the active one when there is a page per artboard
/// of its size.
#[test]
fn an_ai_file_with_no_layers_keeps_its_active_artboard() {
    let board = |x0: i32| {
        format!(
            "%_/Dictionary :\n%_{x0} 100 /RealPointRelToROrigin\n%_ (PositionPoint1) ,\n%_{} 0 /RealPointRelToROrigin\n%_ (PositionPoint2) ,\n%_; ,\n",
            x0 + 100
        )
    };
    let no_layers = |active: &str| {
        native(&format!(
            "%_/Document :\n%_/Dictionary :\n%_/Array :\n{}{}%_; (ArtboardArray) ,\n{active}%_; /NotRecorded ,\n%_;\n",
            board(0),
            board(120)
        ))
    };
    let mut pdf = pdf_part(10.0, 10.0);
    let mut page = pdf.artboards[0].clone();
    (page.id, page.rect) = (2, vectorcraft_geom::Rect::new(120.0, 0.0, 220.0, 100.0));
    pdf.artboards.push(page.clone());
    let opened = |pdf: &Document, active: &str| {
        let data = no_layers(active);
        assert!(!data.contains("%AI5_BeginLayer"));
        let (d, w, read) = crate::layered_ai(&private(&data), pdf.clone(), vec![], false, false, 1.0);
        assert!(!read && !w.iter().any(|w| w.contains("layers weren't read")), "{w:?}");
        assert_eq!(d.artboards.iter().map(|a| a.rect).collect::<Vec<_>>(), pdf.artboards.iter().map(|a| a.rect).collect::<Vec<_>>());
        d.opened_artboard
    };
    assert_eq!(opened(&pdf, "%_1 /Int (CropAreaActive) ,\n"), Some(2));
    assert_eq!(opened(&pdf, "%_0 /Int (CropAreaActive) ,\n"), None);
    assert_eq!(opened(&pdf, ""), None);
    // Not a page per artboard, or a page of another size: the pages aren't the artboards.
    let mut three = pdf.clone();
    (page.id, page.rect) = (3, vectorcraft_geom::Rect::new(240.0, 0.0, 340.0, 100.0));
    three.artboards.push(page);
    assert_eq!(opened(&three, "%_1 /Int (CropAreaActive) ,\n"), None);
    let mut other_size = pdf.clone();
    other_size.artboards[1].rect = vectorcraft_geom::Rect::new(120.0, 0.0, 320.0, 100.0);
    assert_eq!(opened(&other_size, "%_1 /Int (CropAreaActive) ,\n"), None);
}

#[test]
fn art_moved_with_its_page_moves_its_pattern_tile() {
    // Two 100-point artboards, the second 50 below the first; an operator this doesn't know, so
    // the layers aren't read.
    let boards = "%_/Document :\n%_/Dictionary :\n%_/Array :\n\
        %_/Dictionary :\n%_0 100 /RealPointRelToROrigin\n%_ (PositionPoint1) ,\n%_100 0 /RealPointRelToROrigin\n%_ (PositionPoint2) ,\n%_; ,\n\
        %_/Dictionary :\n%_0 -50 /RealPointRelToROrigin\n%_ (PositionPoint1) ,\n%_100 -150 /RealPointRelToROrigin\n%_ (PositionPoint2) ,\n%_; ,\n\
        %_; (ArtboardArray) ,\n%_; /NotRecorded ,\n%_;\n";
    let art = native(&(boards.to_string() + &layer("Art", true, &(square(10, 10) + "\n1 Zz"))));
    // The PDF part: the second page 20 points right of the first, with a pattern-filled square.
    let mut pdf = pdf_part(10.0, 10.0);
    let mut page = pdf.artboards[0].clone();
    (page.id, page.rect) = (2, vectorcraft_geom::Rect::new(120.0, 0.0, 220.0, 100.0));
    pdf.artboards.push(page);
    let path = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(130.0, 10.0, 134.0, 14.0));
    let paint = vectorcraft_color::Paint::Pattern { pattern: "Dots".into(), xf: vectorcraft_geom::Affine::IDENTITY };
    let appearance = vectorcraft_doc::Appearance {
        items: vec![vectorcraft_doc::AppearanceItem::Fill(vectorcraft_doc::FillLayer::new(paint))],
        ..Default::default()
    };
    let (id, layer_id) = (pdf.alloc_id(), pdf.layers[0].id);
    pdf.insert(Some(layer_id), 1, Node::path(id, path, appearance)).unwrap();
    let (d, w, read) = crate::layered_ai(&private(&art), pdf, vec![], false, false, 1.0);
    assert!(!read, "{w:?}");
    let square = &d.layers[0].children().unwrap()[1];
    assert_eq!(square.visual_bounds().unwrap().origin(), vectorcraft_geom::Point::new(10.0, 160.0));
    let Some(vectorcraft_color::Paint::Pattern { xf, .. }) = square.appearance.fill().map(|f| &f.paint) else {
        panic!("the square keeps its pattern fill: {:?}", square.appearance);
    };
    assert_eq!(*xf, vectorcraft_geom::Affine::translate(vectorcraft_geom::Vec2::new(-120.0, 150.0)));
}





/// A text document of `stories` (each as the text, its alignment code, its anchor and the frame's
/// translation, and the start of its first line from the anchor), one font and a 14 point default.
/// A story: its text, alignment code, anchor, frame translation and first line's start.
pub(super) type Story<'a> = (&'a str, u8, (f64, f64), (f64, f64), f64);

pub(super) fn text_document(stories: &[Story<'_>]) -> String {
    let story = |(text, align, anchor, _t, start): &Story<'_>| {
        let n = text.chars().count();
        format!(
            "<< /0 << /0 ({text}) /5 << /0 [ << /0 << /0 << /0 () /5 << /0 {align} >> /6 0 >> >> /1 {n} >> ] >> \
             /6 << /0 [ << /0 << /0 << /0 () /5 0 /6 << /0 0 /1 14.0 /53 << /99 /CAITextPaint /0 << /0 2 /1 [ 1.0 0.0 1.0 0.0 0.0 ] >> >> >> >> >> /1 {n} >> ] >> >> \
             /1 << /0 [ << /0 0 >> ] /2 [ << /99 /PC /6 [ << /99 /F /0 << /0 [ {} {} ] >> /6 [ << /99 /R /6 [ << /99 /R /6 [ << /99 /L /6 [ \
             << /99 /S /0 << /0 [ {start} 0.0 ] >> /15 << /0 {n} >> >> ] >> ] >> ] >> ] >> ] >> ] >> >>",
            anchor.0, anchor.1
        )
    };
    let frame =
        |(_, _, _, t, _): &(&str, u8, (f64, f64), (f64, f64), f64)| format!("<< /0 << /0 [ 0 0 ] /2 << /2 [ 1 0 0 1 {} {} ] >> >> >>", t.0, t.1);
    format!(
        "/0 << /1 << /0 [ << /0 << /0 << /0 (Helvetica) >> >> >> ] >> /8 << /0 [ {} ] >> >>\n/1 << /1 [ {} ] /2 << /1 12.0 >> >>\n",
        stories.iter().map(frame).collect::<Vec<_>>().join(" "),
        stories.iter().map(story).collect::<Vec<_>>().join(" ")
    )
}


/// The editing copy of a 100 × 100 file with `layers` and the text `document`; its template box is
/// centred at (50, 50).
pub(super) fn native_with_text(layers: &str, document: &str) -> String {
    let lines: Vec<String> = ascii85(document.as_bytes()).as_bytes().chunks(60).map(|c| format!("%{}", String::from_utf8_lossy(c))).collect();
    format!(
        "%!PS-Adobe-3.0 \n%%BoundingBox: 0 0 100 100\n%%HiResBoundingBox: 0 0 100 100\n%AI3_Cropmarks: 0 0 100 100\n%AI3_TemplateBox: 50 50 50 50\n\
         %AI5_NumLayers: 2\n{layers}%AI11_BeginTextDocument\n/AI11TextDocument : /ASCII85Decode ,\n{}\n%AI11_EndTextDocument\n%%Trailer\n",
        lines.join("\n")
    )
}

/// [`text_document`] of `stories`, padded so that [`native_with_text`] doesn't split the end of its
/// data, `~>`, across two lines.
pub(super) fn unsplit_text_document(stories: &[Story<'_>]) -> String {
    let mut doc = text_document(stories);
    while ascii85(doc.as_bytes()).len() % 60 == 1 {
        doc.push('\n');
    }
    doc
}

/// A text object of story `story`, as the layers hold it.
pub(super) fn text_object(story: u32) -> String {
    format!("/AI11Text :\n0 /FreeUndo ,\n0 /FrameIndex ,\n{story} /StoryIndex ,\n2 /TextAntialiasing ,\n;\n")
}

fn text_of(l: &Node) -> Vec<&vectorcraft_doc::TextObject> {
    l.children().into_iter().flatten().filter_map(|n| if let NodeKind::Text(t) = &n.kind { Some(&**t) } else { None }).collect()
}






























/// `doc` (from [`text_document`], whose stories all name the first frame) with its last story in
/// its own frame, the last, of a kind this doesn't read.
fn last_frame_unread(doc: &str) -> String {
    let frames = doc.matches("/2 << /2 [").count();
    let mut doc = doc.to_string();
    let at = doc.rfind("/1 << /0 [ << /0 0 >> ]").unwrap();
    doc.replace_range(at..at + 23, &format!("/1 << /0 [ << /0 {} >> ]", frames - 1));
    let at = doc.rfind("/2 << /2 [").unwrap() + "/2 << ".len();
    doc.insert_str(at, "/0 9 ");
    doc
}

/// The plain text of every text object in `d`, in order.
fn all_texts(d: &Document) -> Vec<String> {
    let mut all = vec![];
    d.walk(|n| {
        if let NodeKind::Text(t) = &n.kind {
            all.push(t.plain_text());
        }
    });
    all
}







/// A box from (`x0`, `y0`) to (`x1`, `y1`) in the format's operators, filled yellow.
fn filled_box(x0: u32, y0: u32, x1: u32, y1: u32) -> String {
    format!("0 0 1 0 k\n{x0} {y0} m\n{x1} {y0} L\n{x1} {y1} L\n{x0} {y1} L\nf\n")
}

/// The type in a drawn look (`1 /FreeUndo`): a story of another text document.
const LOOK_TYPE: &str = "/AI11Text :\n1 /FreeUndo ,\n0 /FrameIndex ,\n0 /StoryIndex ,\n;\n";

/// The text of each text object in `n`, and whether it is in a group whose box is `around`.
fn texts_in(n: &Node, around: (f64, f64), out: &mut Vec<(String, bool)>) {
    for c in n.children().into_iter().flatten() {
        match &c.kind {
            NodeKind::Text(t) => {
                let inside = n.visual_bounds().is_some_and(|b| (b.x0 - around.0).abs() < 0.5 && (b.y0 - around.1).abs() < 7.0);
                out.push((t.plain_text(), inside));
            }
            _ => texts_in(c, around, out),
        }
    }
}


/// A `.ai` whose page (its PDF part) has a yellow box from (10, 70) to (50, 90), and the type
/// `text` at (12, 84) when there is one.
fn pdf_box_with_type(text: Option<&str>) -> Document {
    let mut d = Document::new(100.0, 100.0);
    let layer_id = d.layers[0].id;
    let path = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(10.0, 70.0, 50.0, 90.0));
    let paint = vectorcraft_color::Paint::solid(vectorcraft_color::Color::cmyk(0.0, 0.0, 1.0, 0.0));
    let item = vectorcraft_doc::AppearanceItem::Fill(vectorcraft_doc::FillLayer::new(paint));
    let square = Node::path(d.alloc_id(), path, vectorcraft_doc::Appearance { items: vec![item], ..Default::default() });
    d.insert(Some(layer_id), 0, square).unwrap();
    if let Some(text) = text {
        let style = vectorcraft_doc::CharStyle { size: 6.0, ..Default::default() };
        let text = vectorcraft_doc::TextObject::point(vectorcraft_geom::Point::new(12.0, 84.0), text, style);
        let id = d.alloc_id();
        d.insert(Some(layer_id), 1, Node::new(id, NodeKind::Text(Box::new(text)))).unwrap();
    }
    d
}

#[test]
fn a_copy_that_a_look_draws_in_an_ai_file_takes_the_type_of_its_pdf_part() {
    let look = format!("0 Ae\nu\nu\n{LOOK_TYPE}{}U\nU\n", filled_box(10, 10, 50, 30));
    let art = native(&layer("Art", true, &look));
    let (d, w, read) = crate::layered_ai(&private(&art), pdf_box_with_type(Some("Copy")), vec![], false, false, 1.0);
    assert!(read, "{w:?}");
    let mut texts = vec![];
    texts_in(&d.layers[0], (10.0, 70.0), &mut texts);
    assert_eq!(texts, [("Copy".to_string(), true)], "{w:?}");
    assert!(!w.iter().any(|w| w.contains("left out") || w.contains("order the page paints")), "{w:?}");
    // With its type asked for as outlines, the copy has no type to hold, and the layers still come in.
    let (d, w, read) = crate::layered_ai(&private(&art), pdf_box_with_type(None), vec![], true, false, 1.0);
    assert!(read, "{w:?}");
    assert!(all_texts(&d).is_empty(), "{w:?}");
}
















/// A text object on a layer that shows and doesn't print, which the page doesn't draw, isn't taken
/// out: it comes in from the text document, on its layer.
#[test]
fn a_shown_text_object_on_a_layer_that_doesnt_print_stays_on_its_layer() {
    let doc = text_document(&[("Hi there\r", 0, (8200.0, 8180.0), (0.0, 0.0), 0.0)]);
    // The layer's options with its fourth flag, printing, off.
    let tips = layer("Tips", true, &text_object(0)).replace("1 1 1 1 0 0 1 0 79", "1 1 1 0 0 0 1 0 79");
    let art = native_with_text(&(layer("Art", true, &square(10, 10)) + &tips), &doc);
    let (d, w, read) = crate::layered_ai(&private(&art), pdf_part(10.0, 86.0), vec![], false, false, 1.0);
    assert!(read, "{w:?}");
    assert_eq!(names(&d), ["Art", "Tips"], "{w:?}");
    let tips = &d.layers[1];
    assert!(tips.visible && matches!(tips.kind, NodeKind::Layer { printable: false, .. }));
    let texts: Vec<String> = text_of(tips).iter().map(|t| t.plain_text()).collect();
    assert_eq!(texts, ["Hi there"], "{w:?}");
    assert!(!w.iter().any(|w| w.contains("doesn't draw")), "{w:?}");
}





#[test]
fn an_ai_file_without_its_pdf_part_reads_its_layers_and_makes_its_type() {
    // Point type from the text document, shown or not; a story it can't read is left out, with a
    // note that is a loss.
    let mut doc = text_document(&[("Hi there\r", 0, (8200.0, 8180.0), (0.0, 0.0), 0.0), ("Area\r", 0, (8200.0, 8160.0), (0.0, 0.0), 0.0)]);
    // Story 1 in frame 1, a frame of a kind this doesn't know.
    let at = doc.rfind("/1 << /0 [ << /0 0 >> ]").unwrap();
    doc.replace_range(at..at + 23, "/1 << /0 [ << /0 1 >> ]");
    let at = doc.rfind("/2 << /2 [ 1 0 0 1 0 0 ] >>").unwrap();
    doc.replace_range(at..at + 27, "/2 << /0 9 /2 [ 1 0 0 1 0 0 ] >>");
    let art = native_with_text(&(layer("Back", true, &square(10, 10)) + &layer("Words", true, &(text_object(0) + &text_object(1)))), &doc);
    let (d, notes) = crate::ai_alone(&private(&art)).unwrap();
    assert_eq!(names(&d), ["Back", "Words"], "{notes:?}");
    let texts: Vec<String> = text_of(&d.layers[1]).iter().map(|t| t.plain_text()).collect();
    assert_eq!(texts, ["Hi there"], "{notes:?}");
    let left = notes.iter().find(|n| n.contains("left out")).unwrap();
    assert!(left.starts_with("1 ") && crate::is_loss(left), "{notes:?}");
    assert!(!notes.iter().any(|n| n.contains("order the page paints")), "{notes:?}");
    assert!(crate::ai_alone(b"%AI24_ZStandard_Data nothing").is_err());
}

/// A `.ai`'s PDF part has a page for each artboard, in a row: the type on a later page goes to
/// its text object where the layers have that artboard, not by the first page's place.
#[test]
fn type_on_a_later_artboard_of_an_ai_file_goes_to_its_text_object() {
    // Two 100-point artboards, the second 50 below the first; the type is 50 down the second
    // (art space: (58.5, -100); the canvas centre 8191.5 is the template box's centre, (50, 50)).
    let boards = "%_/Document :\n%_/Dictionary :\n%_/Array :\n\
        %_/Dictionary :\n%_0 100 /RealPointRelToROrigin\n%_ (PositionPoint1) ,\n%_100 0 /RealPointRelToROrigin\n%_ (PositionPoint2) ,\n%_; ,\n\
        %_/Dictionary :\n%_0 -50 /RealPointRelToROrigin\n%_ (PositionPoint1) ,\n%_100 -150 /RealPointRelToROrigin\n%_ (PositionPoint2) ,\n%_; ,\n\
        %_; (ArtboardArray) ,\n%_; /NotRecorded ,\n%_;\n";
    let doc = text_document(&[("Hi there\r", 0, (8200.0, 8341.5), (0.0, 0.0), 0.0)]);
    let art = native_with_text(&(boards.to_string() + &layer("Art", true, &(square(10, 10) + &text_object(0)))), &doc);
    // The PDF part: the second page 20 points right of the first, the type 50 down it.
    let mut pdf = pdf_part(10.0, 86.0);
    let mut page = pdf.artboards[0].clone();
    (page.id, page.rect) = (2, vectorcraft_geom::Rect::new(120.0, 0.0, 220.0, 100.0));
    pdf.artboards.push(page);
    let style = vectorcraft_doc::CharStyle { size: 14.0, ..Default::default() };
    let text = vectorcraft_doc::TextObject::point(vectorcraft_geom::Point::new(178.5, 50.0), "Hi there", style);
    let (id, layer_id) = (pdf.alloc_id(), pdf.layers[0].id);
    pdf.insert(Some(layer_id), 1, Node::new(id, NodeKind::Text(Box::new(text)))).unwrap();
    let (d, w, read) = crate::layered_ai(&private(&art), pdf, vec![], false, false, 1.0);
    assert!(read, "{w:?}");
    assert_eq!(d.artboards[1].rect, vectorcraft_geom::Rect::new(0.0, 150.0, 100.0, 250.0));
    let texts = text_of(&d.layers[0]);
    assert_eq!(texts.len(), 1, "{w:?}");
    let [.., e, f] = texts[0].xf.as_coeffs();
    assert!((e - 58.5).abs() < 1e-6 && (f - 200.0).abs() < 1e-6, "{:?}", texts[0].xf);
    assert!(!w.iter().any(|w| w.contains("with another's") || w.contains("doesn't draw")), "{w:?}");
}

#[test]
fn page_type_on_a_later_artboard_moves_its_pattern_tile() {
    // As above, with a frame this doesn't read, so the text object takes the page's type, which
    // has a pattern fill.
    let boards = "%_/Document :\n%_/Dictionary :\n%_/Array :\n\
        %_/Dictionary :\n%_0 100 /RealPointRelToROrigin\n%_ (PositionPoint1) ,\n%_100 0 /RealPointRelToROrigin\n%_ (PositionPoint2) ,\n%_; ,\n\
        %_/Dictionary :\n%_0 -50 /RealPointRelToROrigin\n%_ (PositionPoint1) ,\n%_100 -150 /RealPointRelToROrigin\n%_ (PositionPoint2) ,\n%_; ,\n\
        %_; (ArtboardArray) ,\n%_; /NotRecorded ,\n%_;\n";
    let doc = last_frame_unread(&text_document(&[("Hi there\r", 0, (8200.0, 8341.5), (0.0, 0.0), 0.0)]));
    let art = native_with_text(&(boards.to_string() + &layer("Art", true, &(square(10, 10) + &text_object(0)))), &doc);
    let mut pdf = pdf_part(10.0, 86.0);
    let mut page = pdf.artboards[0].clone();
    (page.id, page.rect) = (2, vectorcraft_geom::Rect::new(120.0, 0.0, 220.0, 100.0));
    pdf.artboards.push(page);
    let style = vectorcraft_doc::CharStyle { size: 14.0, ..Default::default() };
    let text = vectorcraft_doc::TextObject::point(vectorcraft_geom::Point::new(178.5, 50.0), "Hi there", style);
    let (id, layer_id) = (pdf.alloc_id(), pdf.layers[0].id);
    let mut node = Node::new(id, NodeKind::Text(Box::new(text)));
    let paint = vectorcraft_color::Paint::Pattern { pattern: "Dots".into(), xf: vectorcraft_geom::Affine::IDENTITY };
    node.appearance = vectorcraft_doc::Appearance {
        items: vec![vectorcraft_doc::AppearanceItem::Fill(vectorcraft_doc::FillLayer::new(paint))],
        ..Default::default()
    }
    .into();
    pdf.insert(Some(layer_id), 1, node).unwrap();
    let (d, w, read) = crate::layered_ai(&private(&art), pdf, vec![], false, false, 1.0);
    assert!(read, "{w:?}");
    let texts: Vec<&Node> = d.layers[0].children().unwrap().iter().filter(|n| matches!(n.kind, NodeKind::Text(_))).map(|n| &**n).collect();
    assert_eq!(texts.len(), 1, "{w:?}");
    let Some(vectorcraft_color::Paint::Pattern { xf, .. }) = texts[0].appearance.fill().map(|f| &f.paint) else {
        panic!("the page's type keeps its pattern fill: {:?}", texts[0].appearance);
    };
    assert_eq!(*xf, vectorcraft_geom::Affine::translate(vectorcraft_geom::Vec2::new(-120.0, 150.0)));
}

/// A global process colour (`Xk`) or a pattern fill on a sublayer no longer sends a `.ai` to its
/// PDF part, which has only the top-level layers (#1025).
#[test]
fn a_global_colour_or_a_pattern_fill_keeps_the_sublayers_of_an_ai_file() {
    // The PDF part's yellow square, in a global yellow at full tint, on a sublayer; a hidden
    // sublayer with a pattern fill (the page doesn't have it).
    let yellow = "0 0 1 0 (Yellow global) 0 0 Xk\n10 10 m\n14 10 L\n14 14 L\n10 14 L\nf";
    let dots = "%AI3_BeginPattern: (Dots)\n(Dots) 0 0 4 4\n%_0 0 0 1 k\n%_1 1 m\n%_3 1 L\n%_3 3 L\n%_1 3 L\n%_f\nE\n%AI3_EndPattern\n";
    let patterned = "(Dots) 0 0 1 1 0 0 0 0 0 [1 0 0 1 0 0] p\n50 50 m\n70 50 L\n70 70 L\n50 70 L\nf";
    let ground = layer("Ground", true, &(layer("Lawn", true, yellow) + &layer("Paving", false, patterned)));
    let art = native(&format!("{dots}{ground}"));
    let (d, warnings, _) = crate::layered_ai(&private(&art), pdf_part(10.0, 86.0), vec![], false, false, 1.0);
    assert_eq!(names(&d), ["Ground"], "{warnings:?}");
    let subs: Vec<_> = d.layers[0].children().unwrap().iter().map(|l| (l.name.clone().unwrap_or_default(), kinds(l))).collect();
    assert_eq!(subs, [("Lawn".to_string(), vec!["path"]), ("Paving".to_string(), vec!["path"])]);
    assert!(warnings.is_empty(), "{warnings:?}");
    assert!(d.swatch("Yellow global").is_some_and(|s| s.global && !s.spot));
    assert!(d.pattern("Dots").is_some() && d.swatch("Dots").is_some());
    // Without a PDF part, the pattern fill shows.
    let shown = native(&format!("{dots}{}", layer("Ground", true, &layer("Lawn", true, patterned))));
    let (d, notes) = crate::ai_alone(&private(&shown)).unwrap();
    let lawn = &d.layers[0].children().unwrap()[0];
    assert_eq!((lawn.name.as_deref(), kinds(lawn)), (Some("Lawn"), vec!["path"]), "{notes:?}");
    let fill = lawn.children().unwrap()[0].appearance.fill().map(|f| f.paint.clone());
    assert!(matches!(&fill, Some(vectorcraft_color::Paint::Pattern { pattern, .. }) if pattern == "Dots"), "{fill:?}");
}

/// A `.ai` whose PDF part is older than its editing copy draws less than its layers: trusted, the
/// layers are used anyway, and the difference is noted.
#[test]
fn a_trusted_editing_copy_is_used_where_it_draws_differently_from_the_pdf_part() {
    let extra = "0 0 1 0 k\n50 50 m\n70 50 L\n70 70 L\n50 70 L\nf";
    let art = native(&layer("Back", true, &(square(10, 10) + "\n" + extra)));
    let (d, w, _) = crate::layered_ai(&private(&art), pdf_part(10.0, 86.0), vec![], false, false, 1.0);
    assert_ne!(names(&d), ["Back"], "untrusted, the PDF part is used: {w:?}");
    let (d, w, _) = crate::layered_ai(&private(&art), pdf_part(10.0, 86.0), vec![], false, true, 1.0);
    assert_eq!(names(&d), ["Back"], "{w:?}");
    assert_eq!(d.layers[0].children().unwrap().len(), 2);
    assert!(w.iter().any(|w| w.contains("differs from the page's") && w.contains("all the same")), "{w:?}");
}









#[test]
fn type_asked_for_as_outlines_leaves_a_file_whose_type_shows_as_its_pdf_part() {
    let doc = text_document(&[("Hi\r", 0, (8200.0, 8180.0), (0.0, 0.0), 0.0)]);
    // Type that shows: the PDF part's outlines are its type, which the text objects can't hold.
    let shown = native_with_text(&layer("Art", true, &(square(10, 10) + "\n" + &text_object(0))), &doc);
    let (d, w, _) = crate::layered_ai(&private(&shown), pdf_part(10.0, 86.0), vec![], true, false, 1.0);
    assert_eq!(names(&d), ["Layer 1"], "{w:?}");
    assert!(w.iter().any(|w| w.contains("layers weren't read") && w.contains("outlines")), "{w:?}");
    // Type only on a hidden layer: the layers come in.
    let hidden = native_with_text(&(layer("Art", true, &square(10, 10)) + &layer("Notes", false, &text_object(0))), &doc);
    let (d, w, _) = crate::layered_ai(&private(&hidden), pdf_part(10.0, 86.0), vec![], true, false, 1.0);
    assert_eq!(names(&d), ["Art", "Notes"], "{w:?}");
}

/// A point of the art (y up) on the canvas of a file whose template box is centred at (50, 50).
fn canvas(x: f64, y: f64) -> (f64, f64) {
    (x + 8141.5, 8241.5 - y)
}

/// The segments (four points each) of the rectangle `x0 y0 x1 y1` of the art, on the canvas.
fn canvas_rect(x0: f64, y0: f64, x1: f64, y1: f64) -> String {
    let corners = [canvas(x0, y1), canvas(x0, y0), canvas(x1, y0), canvas(x1, y1), canvas(x0, y1)];
    corners
        .windows(2)
        .map(|w| format!("{} {} {} {} {} {} {} {}", w[0].0, w[0].1, w[0].0, w[0].1, w[1].0, w[1].1, w[1].0, w[1].1))
        .collect::<Vec<_>>()
        .join(" ")
}

/// A story of `text` in `frames`, one paragraph (`para`: its keys) and one style (`style`).
fn story(text: &str, frames: &[usize], para: &str, style: &str) -> String {
    let n = text.chars().count();
    let frames: String = frames.iter().map(|f| format!("<< /0 {f} >> ")).collect();
    format!(
        "<< /0 << /0 ({text}) /5 << /0 [ << /0 << /0 << /0 () /5 << {para} >> /6 0 >> >> /1 {n} >> ] >> \
         /6 << /0 [ << /0 << /0 << /0 () /5 0 /6 << {style} >> >> >> /1 {n} >> ] >> >> /1 << /0 [ {frames}] >> >>"
    )
}

/// A text document of `frames` and `stories` (written by [`story`]).
fn document_of(frames: &[String], stories: &[String]) -> String {
    format!(
        "/0 << /1 << /0 [ << /0 << /0 << /0 (Helvetica) >> >> >> ] >> /8 << /0 [ {} ] >> >>\n/1 << /1 [ {} ] /2 << /1 12.0 >> >>\n",
        frames.join(" "),
        stories.join(" ")
    )
}

/// A text object of frame `frame` of story `story`.
fn frame_object(story: u32, frame: u32) -> String {
    format!("/AI11Text :\n0 /FreeUndo ,\n{frame} /FrameIndex ,\n{story} /StoryIndex ,\n;\n")
}






/// The text objects of `d`: each one's text and kind, in the layers' order.
fn text_kinds(d: &Document) -> Vec<(String, &'static str)> {
    let mut out = vec![];
    d.walk(|n| {
        if let NodeKind::Text(t) = &n.kind {
            let kind = match t.kind {
                vectorcraft_doc::TextKind::Area { .. } => "area",
                vectorcraft_doc::TextKind::OnPath { .. } => "path",
                _ => "point",
            };
            out.push((t.plain_text(), kind));
        }
    });
    out
}



/// An unread group holding a frame of a blank story's thread is replaced by its page art as for
/// any other text object (the frame holds no type): the layers are read, and the thread, left with
/// one frame, isn't one any more.
#[test]
fn unread_group_holding_a_blank_threaded_frame_is_replaced() {
    let frames = [
        format!("<< /0 << /0 [ 0 0 ] /1 << /0 [ {} ] >> /2 << /0 1 >> >> >>", canvas_rect(50.0, 10.0, 54.0, 14.0)),
        format!("<< /0 << /0 [ 0 0 ] /1 << /0 [ {} ] >> /2 << /0 1 >> >> >>", canvas_rect(70.0, 10.0, 74.0, 14.0)),
    ];
    let text = document_of(&frames, &[story("\\r", &[0, 1], "/0 0", "/1 10.0")]);
    for removed in [0, 1] {
        let grouped = os1_group(&format!("{}\n{}\nZZ", square(if removed == 0 { 50 } else { 70 }, 10), frame_object(0, removed)));
        let other = frame_object(0, 1 - removed);
        let art = native_with_text(&(layer("Back", true, &square(10, 10)) + &layer("Mixed", true, &format!("{grouped}\n{other}"))), &text);
        let (doc, notes, read) = crate::layered_ai(&private(&art), pdf_three_squares(), vec![], false, true, 1.0);
        assert!(read, "removed frame {removed}: {notes:?}");
        assert!(doc.text_threads.is_empty(), "removed frame {removed}: {:?} {notes:?}", doc.text_threads);
        assert_eq!(text_kinds(&doc), [("".to_string(), "area")], "removed frame {removed}: {notes:?}");
    }
}



/// Without a page (the editing data alone), a blank area story comes in as its empty frame and
/// blank point type is left out, without the note for type of kinds that aren't read.
#[test]
fn blank_stories_without_a_page() {
    let frames = [area_frame(10.0, 50.0, 90.0, 90.0), point_frame(30.0, 30.0)];
    let doc = document_of(&frames, &[story("\\r", &[0], "/0 0", "/1 6.0"), story(" \\r", &[1], "/0 0", "/1 6.0")]);
    let art = native_with_text(&layer("Art", true, &(text_object(0) + &text_object(1))), &doc);
    let (d, notes) = crate::ai_alone(&private(&art)).unwrap();
    assert_eq!(text_kinds(&d), [("".to_string(), "area")], "{notes:?}");
    assert!(!notes.iter().any(|w| w.contains("left out")), "{notes:?}");
}




/// A blank story's thread whose first frame an unread group takes with it leaves its other frame
/// alone, no longer threaded: in a warp, that frame is weighed against its look as any warped text
/// object is (the look draws a square the empty frame doesn't, so the look takes its place).
#[test]
fn a_blank_frame_an_unread_group_leaves_alone_in_a_warp_is_weighed_as_lone_type() {
    let frames = [
        format!("<< /0 << /0 [ 0 0 ] /1 << /0 [ {} ] >> /2 << /0 1 >> >> >>", canvas_rect(50.0, 10.0, 54.0, 14.0)),
        format!("<< /0 << /0 [ 0 0 ] /1 << /0 [ {} ] >> /2 << /0 1 >> >> >>", canvas_rect(70.0, 10.0, 74.0, 14.0)),
    ];
    let text = document_of(&frames, &[story("\\r", &[0, 1], "/0 0", "/1 10.0")]);
    let grouped = os1_group(&format!("{}\n{}\nZZ", square(50, 10), frame_object(0, 0)));
    let own: String = frame_object(0, 1).lines().map(|l| format!("%_{l}\n")).collect();
    let warped = format!("u\n0 1 1 0 1 0 0 Xa\n70 10 m\n74 10 L\n74 14 L\n70 14 L\nf\nU\n9 () XW\n{own}1 (Warped) XW\n");
    let art = native_with_text(
        &(warped_style(40.0, WarpOn::Object) + &layer("Back", true, &square(10, 10)) + &layer("Mixed", true, &format!("{grouped}\n{warped}"))),
        &text,
    );
    let (doc, notes, read) = crate::layered_ai(&private(&art), pdf_three_squares(), vec![], false, true, 1.0);
    assert!(read, "{notes:?}");
    assert!(doc.text_threads.is_empty(), "{:?} {notes:?}", doc.text_threads);
    assert!(notes.iter().any(|w| w.starts_with(crate::import::ai::WARPED_TYPE_DRAWN)), "{notes:?}");
    // The look took the frame's place: no text object is left.
    assert!(text_kinds(&doc).is_empty(), "{:?} {notes:?}", text_kinds(&doc));
}

/// A point type frame at the point (`x`, `y`) of the art.
fn point_frame(x: f64, y: f64) -> String {
    let (cx, cy) = canvas(x, y);
    format!("<< /0 << /0 [ 0 0 ] /2 << /2 [ 1 0 0 1 {cx} {cy} ] >> >> >>")
}

/// An area type frame of the rectangle `x0 y0 x1 y1` of the art.
fn area_frame(x0: f64, y0: f64, x1: f64, y1: f64) -> String {
    format!("<< /0 << /0 [ 0 0 ] /1 << /0 [ {} ] >> /2 << /0 1 >> >> >>", canvas_rect(x0, y0, x1, y1))
}

/// The text of `n` when it is a text object, or a group of one object whose text that is.
fn lone_text(n: &Node) -> Option<String> {
    match &n.kind {
        NodeKind::Text(t) => Some(t.plain_text()),
        NodeKind::Group { .. } => match n.children()?.as_slice() {
            [only] => lone_text(only),
            _ => None,
        },
        _ => None,
    }
}

/// What each object of layer `l` holds as one text object ([`lone_text`]).
fn lone_texts(l: &Node) -> Vec<Option<String>> {
    l.children().into_iter().flatten().map(|n| lone_text(n)).collect()
}

/// `text` as a string of the text document: UTF-16 after its byte order mark, each byte in octal.
fn utf16(text: &str) -> String {
    let mut out = String::from("\\376\\377");
    for unit in text.encode_utf16() {
        for b in unit.to_be_bytes() {
            out.push_str(&format!("\\{b:03o}"));
        }
    }
    out
}


/// The page's type isn't matched with the text objects when the text document makes the type of
/// every one: a file of more of them than are matched still comes in with its layers.
#[test]
fn six_thousand_point_stories_open_with_layers() {
    let stories: Vec<(String, (f64, f64))> =
        (0..6000).map(|i| (format!("S{i}\r"), (8150.0 + f64::from(i % 60), 8150.0 + f64::from(i / 60) * 0.8))).collect();
    let list: Vec<Story<'_>> = stories.iter().map(|(t, at)| (t.as_str(), 0, *at, (0.0, 0.0), 0.0)).collect();
    let doc = text_document(&list);
    let objects: String = (0..6000).map(text_object).collect();
    let art = native_with_text(&(layer("Back", true, &square(10, 10)) + &layer("Words", true, &objects)), &doc);
    let (d, notes) = crate::ai_alone(&private(&art)).unwrap();
    assert_eq!(names(&d), ["Back", "Words"], "{notes:?}");
    assert_eq!(text_of(&d.layers[1]).len(), 6000, "{notes:?}");
}




/// A hidden text object whose type the text document can't make takes the page's hidden type, as
/// it does when some shown text object's type can't be made either, but not the page's hidden type
/// of one whose type is made.
#[test]
fn a_hidden_text_object_that_cant_be_made_takes_the_pages_hidden_type() {
    let doc = last_frame_unread(&text_document(&[
        ("Hi there\r", 0, (8200.0, 8180.0), (0.0, 0.0), 0.0),
        ("Known\rMore\r", 0, (8200.0, 8200.0), (0.0, 0.0), 0.0),
        ("Secret\r", 0, (8200.0, 8160.0), (0.0, 0.0), 0.0),
    ]));
    let spare = layer("Spare", false, &(text_object(1) + &text_object(2)));
    let art = native_with_text(&(layer("Art", true, &format!("{}\n{}", square(10, 10), text_object(0))) + &spare), &doc);
    let style = vectorcraft_doc::CharStyle { size: 14.0, ..Default::default() };
    let text = |d: &mut Document, s: &str, y: f64| {
        Node::new(d.alloc_id(), NodeKind::Text(Box::new(vectorcraft_doc::TextObject::point(vectorcraft_geom::Point::new(58.5, y), s, style.clone()))))
    };
    // The last text object's type over "Known", in words or in marks only.
    for secret in ["Secret", "?!"] {
        let mut page = pdf_part(10.0, 86.0);
        let shown = text(&mut page, "Hi there", 38.5);
        let first = page.layers[0].id;
        page.insert(Some(first), 1, shown).unwrap();
        let spare = page.add_layer(Some("Spare"));
        // The two lines of "Known More" where its story puts them, and the other over them, painted
        // first.
        for (i, (s, y)) in [(secret, 60.0), ("Known", 58.5), ("More", 73.0)].into_iter().enumerate() {
            let n = text(&mut page, s, y);
            page.insert(Some(spare), i, n).unwrap();
        }
        if let Some(l) = page.node_mut(spare) {
            l.visible = false;
        }
        let (d, w, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
        assert!(read, "{secret}: {w:?}");
        assert_eq!(names(&d), ["Art", "Spare"], "{w:?}");
        let texts = |l: usize| text_of(&d.layers[l]).iter().map(|t| t.plain_text()).collect::<Vec<_>>();
        assert_eq!((texts(0), texts(1)), (vec!["Hi there".to_string()], vec!["Known\nMore".to_string(), secret.to_string()]), "{w:?}");
    }
}



/// A line break inside a paragraph (`\x03` in the text document) stays a line break in it.
#[test]
fn a_soft_return_is_a_line_break_in_its_paragraph() {
    let doc = text_document(&[("One\\003Two\r", 0, (8200.0, 8180.0), (0.0, 0.0), 0.0)]);
    let art = native_with_text(&(layer("Back", true, &square(10, 10)) + &layer("Words", true, &text_object(0))), &doc);
    let (d, notes) = crate::ai_alone(&private(&art)).unwrap();
    let texts = text_of(&d.layers[1]);
    assert_eq!(texts.len(), 1, "{notes:?}");
    assert_eq!(texts[0].plain_text(), "One\u{2028}Two");
    assert!(texts[0].paras.is_empty(), "one paragraph: {:?}", texts[0].paras);
}

/// A Thai story is one text object however the page splits its type: by word, and its marks
/// above apart from their letters, above the top of its frame.
#[test]
fn a_thai_story_stays_one_text_object() {
    let thai = "\u{0E2A}\u{0E38}\u{0E02}\u{0E2A}\u{0E31}\u{0E19}\u{0E15}\u{0E4C} \u{0E27}\u{0E31}\u{0E19}\u{0E40}\u{0E01}\u{0E34}\u{0E14}";
    let doc = document_of(&[area_frame(10.0, 30.0, 90.0, 60.0)], &[story(&utf16(&format!("{thai}\r")), &[0], "/0 0", "/1 14.0")]);
    let art = native_with_text(&layer("Art", true, &format!("{}\nu\n{}U\n", square(10, 10), frame_object(0, 0))), &doc);
    let mut page = pdf_part(10.0, 86.0);
    let layer_id = page.layers[0].id;
    let style = vectorcraft_doc::CharStyle { size: 14.0, ..Default::default() };
    // Doc y 40 is the frame's top: the words on its first line, the marks a little above it.
    let pieces = [
        ("\u{0E2A}\u{0E38}\u{0E02}\u{0E2A}\u{0E19}\u{0E15}", 10.0, 53.0),
        ("\u{0E27}\u{0E19}\u{0E40}\u{0E01}\u{0E14}", 50.0, 53.0),
        ("\u{0E31}\u{0E4C}", 24.0, 41.0),
        ("\u{0E31}\u{0E34}", 58.0, 41.0),
    ];
    for (i, (text, x, y)) in pieces.iter().enumerate() {
        let t = vectorcraft_doc::TextObject::point(vectorcraft_geom::Point::new(*x, *y), text, style.clone());
        let n = Node::new(page.alloc_id(), NodeKind::Text(Box::new(t)));
        page.insert(Some(layer_id), 1 + i, n).unwrap();
    }
    let (d, w, read) = crate::layered_ai(&private(&art), page, vec![], false, false, 1.0);
    assert!(read, "{w:?}");
    assert_eq!(names(&d), ["Art"], "{w:?}");
    assert_eq!(all_texts(&d), [thai], "{w:?}");
    assert_eq!(lone_texts(&d.layers[0]).last(), Some(&Some(thai.to_string())), "{w:?}");
}







/// A filled bar of the art from x 40 to 60, the whole page high, made a wrap object with an offset
/// of 2 points when `wrap` is set.
fn wrap_bar(wrap: bool) -> String {
    let dict = "%_/ArtDictionary :\n%_0 /Bool (AI11TextWrapInvert) ,\n%_2 /Real (AI11TextWrapOffset) ,\n%_;\n%_\n";
    format!("0 0 1 0 k\n40 0 m\n40 100 L\n60 100 L\n60 0 L\nf\n{}", if wrap { dict } else { "" })
}

/// A story long enough to fill several lines of a 10-point area type frame.
const WRAP_STORY: &str = "Text flows around the bar in the middle of its frame when the bar is a wrap object\\r";

/// The `.ai` editing data of area type from x 10 to 90 of the art, with `below` before it and
/// `above` after it in its layer, and `other` on a layer above → the document read from it alone.
fn wrapped_area(below: &str, above: &str, other: Option<&str>) -> Document {
    let doc = document_of(&[area_frame(10.0, 10.0, 90.0, 90.0)], &[story(WRAP_STORY, &[0], "/0 0", "/1 10.0")]);
    let mut layers = layer("Art", true, &format!("{below}\n{}{above}", frame_object(0, 0)));
    if let Some(other) = other {
        layers += &layer("Top", true, other);
    }
    let (d, notes) = crate::ai_alone(&private(&native_with_text(&layers, &doc))).unwrap();
    assert!(!notes.iter().any(|n| n.contains("couldn't be read")), "{notes:?}");
    d
}

/// The area type of `d` (its first text object).
fn area_text(d: &Document) -> vectorcraft_doc::TextObject {
    let mut found = None;
    d.walk(|n| {
        if let NodeKind::Text(t) = &n.kind
            && found.is_none()
        {
            found = Some((**t).clone());
        }
    });
    found.expect("area type")
}

#[test]
fn wrap_area_after_open() {
    let d = wrapped_area("", &wrap_bar(true), None);
    let t = area_text(&d);
    assert_eq!(t.wrap.len(), 1, "one wrap shape, the bar's");
    assert_eq!(t.wrap[0].wrap, vectorcraft_doc::TextWrap { offset: 2.0, invert: false });
    // Each line keeps to one side of the bar and its 2-point offset.
    let lay = vectorcraft_text::layout(vectorcraft_text::FontDb::global(), &t);
    assert!(lay.lines.len() > 1);
    let spans: Vec<(f64, f64)> = lay.lines.iter().map(|l| l.avail).collect();
    assert!(spans.iter().all(|(a, b)| *b <= 38.0 + 1e-6 || *a >= 62.0 - 1e-6), "{spans:?}");
    assert!(spans.iter().any(|(_, b)| *b <= 38.0 + 1e-6) && spans.iter().any(|(a, _)| *a >= 62.0 - 1e-6), "{spans:?}");
    // A wrap object below the type, or on another layer, doesn't wrap it.
    assert!(area_text(&wrapped_area(&wrap_bar(true), "", None)).wrap.is_empty());
    assert!(area_text(&wrapped_area("", "", Some(&wrap_bar(true)))).wrap.is_empty());
}

#[test]
fn no_wrap_metadata_unchanged() {
    let d = wrapped_area("", &wrap_bar(false), None);
    let t = area_text(&d);
    assert!(t.wrap.is_empty());
    let mut wraps = 0;
    d.walk(|n| wraps += usize::from(n.extras().wrap.is_some()));
    assert_eq!(wraps, 0);
    // The lines span the frame, through the bar.
    let lay = vectorcraft_text::layout(vectorcraft_text::FontDb::global(), &t);
    assert!(
        lay.lines.iter().all(|l| l.avail.0 <= 10.0 + 1e-6 && l.avail.1 >= 90.0 - 1e-6),
        "{:?}",
        lay.lines.iter().map(|l| l.avail).collect::<Vec<_>>()
    );
}

/// A text object that is itself a wrap object keeps its options when its type is made from the
/// text document.
#[test]
fn a_wrapped_text_object_stays_a_wrap_object() {
    let doc = document_of(
        &[area_frame(10.0, 10.0, 90.0, 90.0), point_frame(45.0, 50.0)],
        &[story(WRAP_STORY, &[0], "/0 0", "/1 10.0"), story("Mid\\r", &[1], "/0 0", "/1 10.0")],
    );
    let dict = "%_/ArtDictionary :\n%_0 /Bool (AI11TextWrapInvert) ,\n%_4 /Real (AI11TextWrapOffset) ,\n%_;\n%_\n";
    let layers = layer("Art", true, &format!("{}{}{dict}", frame_object(0, 0), frame_object(1, 0)));
    let (d, notes) = crate::ai_alone(&private(&native_with_text(&layers, &doc))).unwrap();
    let kids = d.layers[0].children().unwrap();
    assert_eq!(kids.len(), 2, "{notes:?}");
    assert_eq!(kids[1].extras().wrap, Some(vectorcraft_doc::TextWrap { offset: 4.0, invert: false }), "{notes:?}");
    let NodeKind::Text(area) = &kids[0].kind else { panic!("area type") };
    assert_eq!(area.wrap.len(), 1, "the area type wraps around the text object above it");
}


/// Replacing a group containing either threaded frame must use the PDF page instead.
#[test]
fn unread_group_holding_threaded_frame_falls_back() {
    let frames = [
        format!("<< /0 << /0 [ 0 0 ] /1 << /0 [ {} ] >> /2 << /0 1 >> >> >>", canvas_rect(50.0, 10.0, 54.0, 14.0)),
        format!("<< /0 << /0 [ 0 0 ] /1 << /0 [ {} ] >> /2 << /0 1 >> >> >>", canvas_rect(70.0, 10.0, 74.0, 14.0)),
    ];
    let text = document_of(&frames, &[story("Threaded\\r", &[0, 1], "/0 0", "/1 10.0")]);
    for removed in [0, 1] {
        let grouped = os1_group(&format!("{}\n{}\nZZ", square(if removed == 0 { 50 } else { 70 }, 10), frame_object(0, removed)));
        let other = frame_object(0, 1 - removed);
        let art = native_with_text(&(layer("Back", true, &square(10, 10)) + &layer("Mixed", true, &format!("{grouped}\n{other}"))), &text);
        let (doc, notes, read) = crate::layered_ai(&private(&art), pdf_three_squares(), vec![], false, true, 1.0);
        assert!(!read, "removed frame {removed}: {notes:?}");
        assert_eq!(names(&doc), ["Layer 1"]);
        assert!(notes.iter().any(|n| n.contains("an unread object holds a frame of threaded type")), "removed frame {removed}: {notes:?}");
    }
}

/// Replacing an unread group may still discard a single-frame text object.
#[test]
fn unread_group_holding_single_frame_uses_page_art() {
    let frame = format!("<< /0 << /0 [ 0 0 ] /1 << /0 [ {} ] >> /2 << /0 1 >> >> >>", canvas_rect(50.0, 10.0, 54.0, 14.0));
    let text = document_of(&[frame], &[story("Single\\r", &[0], "/0 0", "/1 10.0")]);
    let grouped = os1_group(&format!("{}\n{}\nZZ", square(50, 10), frame_object(0, 0)));
    let art = native_with_text(&(layer("Back", true, &square(10, 10)) + &layer("Mixed", true, &grouped)), &text);
    let (doc, notes, read) = crate::layered_ai(&private(&art), pdf_two_squares(), vec![], false, false, 1.0);
    assert!(read, "{notes:?}");
    assert_eq!(names(&doc), ["Back", "Mixed"]);
    assert!(doc.text_threads.is_empty());
}



























/// What `f` returns, with how many times it read a text document and scanned the text objects'
/// frames of an editing copy on this thread; with `afresh`, each check and build reads them anew,
/// keeping nothing for the next.
fn text_reads<T>(afresh: bool, f: impl FnOnce() -> T) -> (T, u64, u64) {
    crate::import::ate::TEXTS_READS.set(0);
    crate::import::ai::TEXT_FRAME_SCANS.set(0);
    crate::import::native::READ_AFRESH.set(afresh);
    let out = f();
    crate::import::native::READ_AFRESH.set(false);
    (out, crate::import::ate::TEXTS_READS.get(), crate::import::ai::TEXT_FRAME_SCANS.get())
}

/// A `.ai` that opens without its PDF part reads its text document and scans its text objects'
/// frames once, for the check that its text document makes all of its type and for the build, and
/// comes in as it does when each reads them anew.
#[test]
fn layers_that_open_alone_read_their_text_once() {
    let doc = unsplit_text_document(&[("Hi there\r", 0, (8200.0, 8180.0), (0.0, 0.0), 0.0), ("Mid\r", 2, (8191.5, 8191.5), (0.0, 0.0), -10.0)]);
    let art = layer("Art", true, &format!("{}\n{}{}", square(10, 10), text_object(0), text_object(1)));
    let file = private(&native_with_text(&art, &doc));
    let open = || crate::ai_layers_alone(&file, &[(100.0, 100.0)], 1.0, crate::NameEncoding::default()).ok();
    let (opened, texts, frames) = text_reads(false, open);
    let Some((d, notes)) = opened else { panic!("didn't open without the page") };
    assert_eq!((texts, frames), (1, 1), "{notes:?}");
    assert_eq!(text_of(&d.layers[0]).iter().map(|t| t.plain_text()).collect::<Vec<_>>(), ["Hi there", "Mid"], "{notes:?}");
    let (afresh, texts, frames) = text_reads(true, open);
    assert_eq!((texts, frames), (2, 2));
    assert_eq!(afresh, Some((d, notes)));
}

/// Layers whose text document can't be read wait on the PDF part, which then doesn't read the text
/// document or scan the frames again, and come in as they do when each check and build reads them
/// anew.
#[test]
fn layers_whose_text_document_cannot_be_read_read_it_once_with_the_page() {
    let art = layer("Art", true, &format!("{}\n{}", square(10, 10), text_object(0)));
    let file = private(&native_with_text(&art, "not a text document"));
    let names = crate::NameEncoding::default();
    let open = || {
        let Err(layers) = crate::ai_layers_alone(&file, &[(100.0, 100.0)], 1.0, names) else { panic!("opened without the page") };
        crate::layered_ai_from(layers, pdf_part(10.0, 86.0), vec![], false, false, 1.0, names)
    };
    let (opened, texts, frames) = text_reads(false, open);
    assert_eq!((texts, frames), (1, 1), "{:?}", opened.1);
    let (afresh, texts, frames) = text_reads(true, open);
    assert_eq!((texts, frames), (2, 2), "{:?}", afresh.1);
    assert_eq!(opened, afresh);
}































/// An art style section with style `Comic`: a white fill offset and repeated by Transform under a
/// red fill with its own blend mode, the characters at the bottom.
fn comic_style() -> String {
    let basic = |name: &str, body: &str| {
        format!("/BasicFilter :\n({name}) 0 0 /Filter ,\n1 /Visible ,\n/Dictionary : /NotRecorded ,\n{body}; /Dict ;\n /Part ,\n")
    };
    let fill = |paint: &str| basic("Fill Style Filter", &format!("/FillStyle : 0 O\n{paint}\n0 1 0 0 0 Xy\n0 1 Xd\n/Def ;\n (FillStyle) ,\n"));
    let compound = |name: &str, kids: &[String]| format!("/CompoundFilter :\n({name}) 0 0 /Filter ,\n1 /Visible ,\n{};\n /Part ,\n", kids.concat());
    let rim = compound(
        "Chain Style Filter",
        &[
            basic("Adobe Offset Path", "3 /Real (ofst) ,\n0 /Int (jntp) ,\n4 /Real (mlim) ,\n"),
            fill("0 0 0 0 1 1 1 Xa"),
            basic("Adobe Transform", "6 /Int (numCopies) ,\n1 /Real (moveH_Pts) ,\n"),
        ],
    );
    let red = compound(
        "Chain Style Filter",
        &[
            basic("Adobe Outline Type", ""),
            fill("0 1 1 0 1 0 0 Xa"),
            basic("Blend Style Filter", "/BlendStyle : 1 0.5 0 0 0 Xy\n/Def ;\n (BlendStyle) ,\n"),
        ],
    );
    let stack = compound("Stack Style Filter", &[red, rim, basic("Conduit Filter", "")]);
    format!("%AI9_BeginArtStyles\n/KnownStyle :\n(Comic) /Name ,\n/ActiveStyle :\n{}%AI9_EndArtStyles\n", compound("Chain Style Filter", &[stack]))
}


/// Where [`warped_style`] puts its Warp.
#[derive(Clone, Copy)]
enum WarpOn {
    Object,
    Fill,
    Stroke,
}

/// An art style section with style `Warped`: a red fill with Effect › Warp › Arc at `bend` percent on
/// `on`.
fn warped_style(bend: f64, on: WarpOn) -> String {
    let basic = |name: &str, body: &str| {
        format!("/BasicFilter :\n({name}) 0 0 /Filter ,\n1 /Visible ,\n/Dictionary : /NotRecorded ,\n{body}; /Dict ;\n /Part ,\n")
    };
    let compound = |name: &str, kids: &[String]| format!("/CompoundFilter :\n({name}) 0 0 /Filter ,\n1 /Visible ,\n{};\n /Part ,\n", kids.concat());
    let fill = basic("Fill Style Filter", "/FillStyle : 0 O\n0 1 1 0 1 0 0 Xa\n0 1 0 0 0 Xy\n0 1 Xd\n/Def ;\n (FillStyle) ,\n");
    let warp = basic(
        "Adobe Deform",
        &format!(
            "1 /Int (DeformStyle) ,\n{} /Real (DeformValue) ,\n0 /Real (DeformHoriz) ,\n0 /Real (DeformVert) ,\n0 /Bool (Rotate) ,\n",
            bend / 100.0
        ),
    );
    let stroke = || {
        basic("Stroke Style Filter", "/StrokeStyle : 0 1 0 0 0 Xy\n0 J 0 j 2 w 10 M []0 d0 XR\n0 0 0 1 0 0 0 XA\n1 0 Xd\n/Def ;\n (StrokeStyle) ,\n")
    };
    let conduit = || basic("Conduit Filter", "");
    let chain = match on {
        WarpOn::Object => compound("Chain Style Filter", &[fill, warp]),
        WarpOn::Fill => {
            compound("Chain Style Filter", &[compound("Stack Style Filter", &[compound("Chain Style Filter", &[warp, fill]), conduit()])])
        }
        WarpOn::Stroke => {
            compound("Chain Style Filter", &[compound("Stack Style Filter", &[compound("Chain Style Filter", &[warp, stroke()]), fill, conduit()])])
        }
    };
    format!("%AI9_BeginArtStyles\n/KnownStyle :\n(Warped) /Name ,\n/ActiveStyle :\n{chain}%AI9_EndArtStyles\n")
}

/// `outline`, a path in the document's coordinates, as filled paths in a 100 × 100 file's art space,
/// in the red of [`warped_style`].
fn drawn_as_operators(outline: &kurbo::BezPath) -> String {
    use kurbo::PathEl;
    let mut out = String::from("0 1 1 0 1 0 0 Xa\n");
    let y = |v: f64| 100.0 - v;
    let mut last = kurbo::Point::ZERO;
    for el in outline.elements() {
        match *el {
            PathEl::MoveTo(p) => out += &format!("{} {} m\n", p.x, y(p.y)),
            PathEl::LineTo(p) => out += &format!("{} {} L\n", p.x, y(p.y)),
            PathEl::QuadTo(c, p) => {
                let (a, b) = (last + (c - last) * (2.0 / 3.0), p + (c - p) * (2.0 / 3.0));
                out += &format!("{} {} {} {} {} {} C\n", a.x, y(a.y), b.x, y(b.y), p.x, y(p.y));
            }
            PathEl::CurveTo(a, b, p) => out += &format!("{} {} {} {} {} {} C\n", a.x, y(a.y), b.x, y(b.y), p.x, y(p.y)),
            PathEl::ClosePath => out += "f\n",
        }
        last = match *el {
            PathEl::MoveTo(p) | PathEl::LineTo(p) | PathEl::QuadTo(_, p) | PathEl::CurveTo(_, _, p) => p,
            PathEl::ClosePath => last,
        };
    }
    out
}

/// What `n` holds as art, its paths joined, in the document's coordinates.
fn outlines_of(n: &Node) -> kurbo::BezPath {
    let mut out = kurbo::BezPath::new();
    if let NodeKind::Path { path, .. } = &n.kind {
        out.extend(path.to_bezpath());
    }
    for c in n.children().into_iter().flatten() {
        out.extend(outlines_of(c));
    }
    out
}

/// The editing copy of `objects` text objects of `text` under style `style`, each with the look
/// `drawing`.
fn warped_file(style: &str, text: &str, objects: u32, drawing: &str) -> String {
    let stories: Vec<_> = (0..objects).map(|_| (text, 0u8, (8200.0, 8180.0), (0.0, 0.0), 0.0)).collect();
    let doc = text_document(&stories);
    let art: String = (0..objects)
        .map(|n| {
            let look = format!("u\n{drawing}\nU\n9 () XW\n");
            let own: String = text_object(n).lines().map(|l| format!("%_{l}\n")).collect();
            format!("{look}{own}1 (Warped) XW\n")
        })
        .collect();
    native_with_text(&(style.to_string() + &layer("Art", true, &art)), &doc)
}

/// The type of `doc`, and whether it has a warp.
fn warped_texts(doc: &Document) -> Vec<(String, bool)> {
    let mut out = vec![];
    doc.walk(|n| {
        if let NodeKind::Text(t) = &n.kind {
            out.push((
                t.plain_text(),
                n.appearance.effects.iter().chain(n.appearance.items.iter().flat_map(|i| i.effects())).any(|e| e.id.starts_with("warp.")),
            ));
        }
    });
    out
}

#[test]
fn warped_type_that_draws_as_its_look_stays_live_type() {
    let art = warped_file(&warped_style(40.0, WarpOn::Object), "TILT\r", 1, &plain_warped_look());
    let (d, notes) = crate::ai_alone(&private(&art)).unwrap();
    assert_eq!(warped_texts(&d), [("TILT".to_string(), true)], "{notes:?}");
    assert!(!notes.iter().any(|w| w.starts_with(crate::import::ai::WARPED_TYPE_DRAWN)), "{notes:?}");
}

/// What the live Arc at 40% draws of the type `TILT` that [`warped_file`] places, as look operators.
fn plain_warped_look() -> String {
    let doc = text_document(&[("TILT\r", 0, (8200.0, 8180.0), (0.0, 0.0), 0.0)]);
    let plain = layer("Art", true, &text_object(0));
    let (d, notes) = crate::ai_alone(&private(&native_with_text(&plain, &doc))).unwrap();
    let mut text = None;
    d.walk(|n| {
        if matches!(n.kind, NodeKind::Text(_)) {
            text = Some(n.clone());
        }
    });
    let mut text = text.unwrap_or_else(|| panic!("no type: {notes:?}"));
    text.appearance = warped_appearance().into();
    drawn_as_operators(&outlines_of(&vectorcraft_effects::expand_outlined(None, &text, None).unwrap()))
}

/// The appearance that [`warped_style`] at 40% on the object reads as.
fn warped_appearance() -> vectorcraft_doc::Appearance {
    let art = warped_file(&warped_style(40.0, WarpOn::Object), "TILT\r", 1, "");
    let (d, notes) = crate::ai_alone(&private(&art)).unwrap();
    let mut out = None;
    d.walk(|n| {
        if matches!(n.kind, NodeKind::Text(_)) {
            out = Some((*n.appearance).clone());
        }
    });
    out.unwrap_or_else(|| panic!("no type: {notes:?}"))
}

#[test]
fn warped_type_that_draws_apart_from_its_look_comes_in_as_the_look() {
    // GUARDS: a warp on type opened live with no check, so a live warp that draws another shape from
    // the file's moved and resized the type.
    let drawing = "0 1 1 0 1 0 0 Xa\n54 56 m\n100 56 L\n100 80 L\n54 80 L\nf";
    for on in [WarpOn::Object, WarpOn::Fill, WarpOn::Stroke] {
        let art = warped_file(&warped_style(40.0, on), "TILT\r", 2, drawing);
        let (d, notes) = crate::ai_alone(&private(&art)).unwrap();
        assert_eq!(warped_texts(&d), [], "{notes:?}");
        let looks = d.layers[0].children().unwrap();
        assert_eq!(looks.len(), 2, "{looks:#?}");
        assert!(looks.iter().all(|l| matches!(l.kind, NodeKind::Group { .. }) && l.visual_bounds().is_some()), "{looks:#?}");
        // One note, with how many, that counts as a loss.
        let noted: Vec<&String> = notes.iter().filter(|w| w.starts_with(crate::import::ai::WARPED_TYPE_DRAWN)).collect();
        assert_eq!(noted, ["warped text objects that came in as drawn art, not as editable type: 2"], "{notes:?}");
        assert!(crate::is_loss(noted[0]));
    }
}

/// An art style section with style `Dashed`: a black stroke 2 units wide with butt caps, miter joins
/// and dashes of 6 and gaps of 4, aligned to corners and path ends (`BSDashAdj`).
fn aligned_dash_style() -> String {
    "%AI9_BeginArtStyles\n/KnownStyle :\n(Dashed) /Name ,\n/ActiveStyle :\n/CompoundFilter :\n(Chain Style Filter) 0 0 /Filter ,\n1 /Visible ,\n\
     /BasicFilter :\n(Stroke Style Filter) 0 0 /Filter ,\n1 /Visible ,\n2 /FillOrStroke ,\n/Dictionary : /NotRecorded ,\n\
     /Dictionary : /NotRecorded ,\n1 /Bool (BSDashAdj) ,\n; (kAIBeautifulStrokesParamsDictKey) ,\n\
     /StrokeStyle : 0 R\n0 0 0 1 0 0 0 XA\n0 J 0 j 2 w 10 M [6 4 ]0 d\n0 XR\n1 0 Xd\n/Def ;\n (StrokeStyle) ,\n; /Dict ;\n /Part ,\n;\n%AI9_EndArtStyles\n"
        .to_string()
}

/// The open path of [`aligned_dash_file`], a corner at (80, 20), in the document's coordinates.
fn aligned_dash_path() -> kurbo::BezPath {
    let mut bp = kurbo::BezPath::new();
    bp.move_to((20.0, 20.0));
    bp.line_to((80.0, 20.0));
    bp.line_to((80.0, 70.0));
    bp
}

/// What [`aligned_dash_style`] draws along [`aligned_dash_path`], with its dashes aligned to corners
/// and path ends by VectorCraft or (`aligned` false) not, as look operators in black.
fn aligned_dash_look(aligned: bool) -> String {
    use vectorcraft_color::{Color, Paint};
    use vectorcraft_doc::{Dash, LineCap, LineJoin, StrokeLayer};
    let mut stroke = StrokeLayer::new(Paint::solid(Color::BLACK), 2.0);
    (stroke.cap, stroke.join, stroke.miter_limit) = (LineCap::Butt, LineJoin::Miter, 10.0);
    stroke.dash = Some(Dash { pattern: vec![6.0, 4.0], offset: 0.0, align_corners: aligned });
    let outline = vectorcraft_effects::stroke::line_outline(&aligned_dash_path(), &stroke, 2.0, 0.01);
    assert!(!outline.elements().is_empty());
    drawn_as_operators(&outline).replace("0 1 1 0 1 0 0 Xa", "0 0 0 1 0 0 0 Xa")
}

/// The editing copy of [`aligned_dash_path`] under [`aligned_dash_style`], with the look `drawing`.
fn aligned_dash_file(drawing: &str) -> String {
    aligned_dash_files(drawing, 1)
}

/// [`aligned_dash_file`] with `n` such paths.
fn aligned_dash_files(drawing: &str, n: usize) -> String {
    let own = "%_0 0 0 1 0 0 0 XA\n%_2 w\n%_20 80 m\n%_80 80 L\n%_80 30 L\n%_S\n";
    let one = format!("u\n{drawing}\nU\n9 () XW\n{own}1 (Dashed) XW\n");
    native(&(aligned_dash_style() + &layer("Art", true, &one.repeat(n))))
}

/// The paths of `d` that stroke with dashes, each with whether its dashes are aligned to corners.
fn dashed_paths(d: &Document) -> Vec<bool> {
    let mut out = vec![];
    d.walk(|n| {
        if matches!(n.kind, NodeKind::Path { .. }) {
            for i in &n.appearance.items {
                if let vectorcraft_doc::AppearanceItem::Stroke(s) = i
                    && let Some(dash) = &s.dash
                {
                    out.push(dash.align_corners);
                }
            }
        }
    });
    out
}

/// A stroke whose dashes are aligned to corners and path ends comes in as the path with that stroke,
/// editable, where VectorCraft's fitting draws as its drawn look.
#[test]
fn dashes_aligned_to_corners_that_draw_as_the_look_come_in_live() {
    let (d, notes) = crate::ai_alone(&private(&aligned_dash_file(&aligned_dash_look(true)))).unwrap();
    assert_eq!(dashed_paths(&d), [true], "{notes:?}");
    assert!(!notes.iter().any(|w| w.contains("drawn look")), "{notes:?}");
}

/// Where the look draws the dashes elsewhere (here, not fitted), the path keeps its drawn look.
#[test]
fn dashes_aligned_to_corners_that_draw_apart_from_the_look_keep_it() {
    let (d, notes) = crate::ai_alone(&private(&aligned_dash_file(&aligned_dash_look(false)))).unwrap();
    assert!(dashed_paths(&d).is_empty(), "{notes:?}");
    let looks = d.layers[0].children().unwrap();
    assert!(matches!(looks.as_slice(), [l] if matches!(l.kind, NodeKind::Group { .. }) && l.visual_bounds().is_some()), "{looks:#?}");
}

/// The strokes with aligned dashes one import checks against their looks are limited
/// ([`crate::import::ai::MAX_DASH_CHECKS`]), apart from the envelopes' checks: past the
/// limit, a path that would draw as its look keeps the look.
#[test]
fn dashes_aligned_to_corners_past_their_checks_keep_the_look() {
    let n = crate::import::ai::MAX_DASH_CHECKS;
    let (d, notes) = crate::ai_alone(&private(&aligned_dash_files(&aligned_dash_look(true), n + 1))).unwrap();
    assert_eq!(dashed_paths(&d).len(), n, "{notes:?}");
    let looks = d.layers[0].children().unwrap().iter().filter(|c| matches!(c.kind, NodeKind::Group { .. })).count();
    assert_eq!(looks, 1);
}

/// The reads of a file apart from its layers check no stroke with dashes aligned to corners in its
/// layers against its look: the import's checks ([`crate::import::ai::MAX_DASH_CHECKS`]) are its
/// layers' reader's, though those reads go over all of the data. Its artboards and variable library
/// are read with no checks at all, and its swatches (and so its brushes, styles and symbols, read
/// the same way) with none once their reader reaches the layers. The layers' reader still checks
/// the one such stroke.
#[test]
fn the_reads_apart_from_the_layers_check_no_aligned_dashes() {
    use vectorcraft_testkit::ai::{sample_variable_sets, variable_library_data};
    let library = variable_library_data(&sample_variable_sets("variableAndTypes", &[]));
    let own = "%_0 0 0 1 0 0 0 XA\n%_2 w\n%_20 80 m\n%_80 80 L\n%_80 30 L\n%_S\n";
    let one = format!("u\n{}\nU\n9 () XW\n{own}1 (Dashed) XW\n", aligned_dash_look(true));
    let data = native(&(library + &aligned_dash_style() + &layer("Art", true, &one)));
    let checks = |f: &dyn Fn()| {
        crate::import::ai::DASH_CHECKS_MADE.set(0);
        f();
        crate::import::ai::DASH_CHECKS_MADE.get()
    };
    let names = crate::import::ai::Names::default;
    assert_eq!(checks(&|| drop(crate::ai_alone(&private(&data)))), 1);
    assert_eq!(checks(&|| drop(crate::import::ai::artboards(data.as_bytes(), &mut names()))), 0);
    assert_eq!(checks(&|| assert!(crate::import::ai::variables(data.as_bytes(), &mut names()).present)), 0);
    assert_eq!(checks(&|| drop(crate::import::ai::palette(data.as_bytes(), &[], &mut names()))), 0);
}

/// Type under a style with dashes aligned to corners keeps its drawn look, as before, and takes no
/// such stroke: the look check isn't made for type.
#[test]
fn type_with_dashes_aligned_to_corners_keeps_its_drawn_look() {
    let doc = text_document(&[("TILT\r", 0, (8200.0, 8180.0), (0.0, 0.0), 0.0)]);
    let own: String = text_object(0).lines().map(|l| format!("%_{l}\n")).collect();
    let look = format!("u\n{}\nU\n9 () XW\n", aligned_dash_look(true));
    let file = native_with_text(&(aligned_dash_style() + &layer("Art", true, &format!("{look}{own}1 (Dashed) XW\n"))), &doc);
    let (d, notes) = crate::ai_alone(&private(&file)).unwrap();
    // The look stays, and the type takes no stroke with aligned dashes.
    let kids = d.layers[0].children().unwrap();
    assert!(kids.iter().any(|c| matches!(c.kind, NodeKind::Group { .. }) && c.visual_bounds().is_some()), "{notes:?}");
    let mut aligned = false;
    d.walk(|n| {
        if matches!(n.kind, NodeKind::Text(_)) {
            aligned |= n
                .appearance
                .items
                .iter()
                .any(|i| matches!(i, vectorcraft_doc::AppearanceItem::Stroke(s) if s.dash.as_ref().is_some_and(|d| d.align_corners)));
        }
    });
    assert!(!aligned, "{notes:?}");
}

/// Warped type that a text variable binds stays type with the effect live and keeps its binding,
/// though its live warp draws apart from its look.
#[test]
fn warped_type_bound_to_a_text_variable_stays_bound_type() {
    use vectorcraft_testkit::ai::{XmlValue, named, variable_library_data, xml_element, xml_node};
    let attributes =
        [("trait", XmlValue::Str("textcontent")), ("varName", XmlValue::Unicode("v_text")), ("docRef", XmlValue::Raw("/XMLUIDREF : (type_x5F_1) ;"))];
    let declared = [xml_node(1, "variable", XmlValue::Str(""), &attributes, &[])];
    let set = xml_element("variableSet", &[("locked", "none")], &[xml_element("variables", &[], &declared)]);
    let library = variable_library_data(&xml_element("variableSets", &[], &[set]));
    let drawing = "0 1 1 0 1 0 0 Xa\n54 56 m\n100 56 L\n100 80 L\n54 80 L\nf";
    let own: String = text_object(0).lines().map(|l| format!("%_{l}\n")).collect();
    let art = format!("u\n{drawing}\nU\n9 () XW\n{own}{}1 (Warped) XW\n", named("type_x5F_1"));
    let doc = text_document(&[("TILT\r", 0, (8200.0, 8180.0), (0.0, 0.0), 0.0)]);
    let file = native_with_text(&(library + &warped_style(40.0, WarpOn::Object) + &layer("Art", true, &art)), &doc);
    let (d, notes) = crate::ai_alone(&private(&file)).unwrap();
    assert_eq!(warped_texts(&d), [("TILT".to_string(), true)], "{notes:?}");
    assert!(!notes.iter().any(|w| w.starts_with(crate::import::ai::WARPED_TYPE_DRAWN)), "{notes:?}");
    let mut text = None;
    d.walk(|n| {
        if matches!(n.kind, NodeKind::Text(_)) {
            text = Some(n.id);
        }
    });
    assert_eq!(d.variables.objects_of("v_text"), [text.unwrap()], "{notes:?}");
}



/// Warped type spends the page's share of the budget one object after another: with enough of them
/// the later ones open as their looks, though each draws close enough on its own.
#[test]
fn warped_type_shares_the_page_budget() {
    let look = plain_warped_look();
    let style = warped_style(40.0, WarpOn::Object);
    let art = warped_file(&style, "TILT\r", 12, &look);
    let (d, notes) = crate::ai_alone(&private(&art)).unwrap();
    assert_eq!(warped_texts(&d).len(), 12, "{notes:?}");
    // The look draws a small square more than the live warp does.
    let art = warped_file(&style, "TILT\r", 12, &format!("{look}0 1 1 0 1 0 0 Xa\n2 2 m\n3.5 2 L\n3.5 3.5 L\n2 3.5 L\nf"));
    let (d, notes) = crate::ai_alone(&private(&art)).unwrap();
    let live = warped_texts(&d).len();
    assert!(live > 0 && live < 12, "{live}: {notes:?}");
    assert!(notes.iter().any(|w| w == &format!("{}: {}", crate::import::ai::WARPED_TYPE_DRAWN, 12 - live)), "{live}: {notes:?}");
}


#[test]
fn warped_type_with_a_look_that_draws_nothing_stays_type() {
    // The look only held the type, which the slot's text object replaces, or a path that paints
    // nothing: there is no art to stand in.
    for drawing in ["", LOOK_TYPE, "54 56 m\n100 56 L\n100 80 L\n54 80 L\nn"] {
        let art = warped_file(&warped_style(40.0, WarpOn::Object), "TILT\r", 1, drawing);
        let (d, notes) = crate::ai_alone(&private(&art)).unwrap();
        assert_eq!(warped_texts(&d), [("TILT".to_string(), true)], "{notes:?}");
        assert!(!notes.iter().any(|w| w.starts_with(crate::import::ai::WARPED_TYPE_DRAWN)), "{notes:?}");
    }
}

/// A look that draws type as well as art is missing the type once the slot's text object takes it,
/// so it can't stand in: the object stays type, and its letters stay on the page.
#[test]
fn warped_type_whose_look_also_draws_type_stays_type() {
    let drawing = format!("{LOOK_TYPE}0 1 1 0 1 0 0 Xa\n54 56 m\n100 56 L\n100 80 L\n54 80 L\nf");
    let art = warped_file(&warped_style(40.0, WarpOn::Fill), "TILT\r", 1, &drawing);
    let (d, notes) = crate::ai_alone(&private(&art)).unwrap();
    assert_eq!(warped_texts(&d), [("TILT".to_string(), true)], "{notes:?}");
    assert!(!notes.iter().any(|w| w.starts_with(crate::import::ai::WARPED_TYPE_DRAWN)), "{notes:?}");
}


/// Warped frames of threaded type stay type, and stay one thread, whatever their looks draw.
#[test]
fn warped_threaded_type_stays_threaded() {
    let frames = [
        format!("<< /0 << /0 [ 0 0 ] /1 << /0 [ {} ] >> /2 << /0 1 >> >> >>", canvas_rect(10.0, 60.0, 40.0, 80.0)),
        format!("<< /0 << /0 [ 0 0 ] /1 << /0 [ {} ] >> /2 << /0 1 >> >> >>", canvas_rect(50.0, 60.0, 80.0, 80.0)),
    ];
    let doc = document_of(&frames, &[story("A story in two frames\\r", &[0, 1], "/0 0", "/1 10.0")]);
    let body: String = (0..2)
        .map(|frame| {
            let own: String = frame_object(0, frame).lines().map(|l| format!("%_{l}\n")).collect();
            format!("u\n0 1 1 0 1 0 0 Xa\n2 2 m\n9 2 L\n9 9 L\n2 9 L\nf\nU\n9 () XW\n{own}1 (Warped) XW\n")
        })
        .collect();
    let art = native_with_text(&(warped_style(40.0, WarpOn::Object) + &layer("Art", true, &body)), &doc);
    let (d, notes) = crate::ai_alone(&private(&art)).unwrap();
    assert_eq!(d.text_threads.len(), 1, "{notes:?}");
    assert_eq!(warped_texts(&d).len(), 2, "{notes:?}");
    assert!(!notes.iter().any(|w| w.starts_with(crate::import::ai::WARPED_TYPE_DRAWN)), "{notes:?}");
}



/// An art style section with style `Bent`: a blue fill over the characters, and effect `name`, its
/// dictionary holding `keys`, on the whole object.
fn whole_object_style(name: &str, keys: &str) -> String {
    let basic = |name: &str, body: &str| {
        format!("/BasicFilter :\n({name}) 0 0 /Filter ,\n1 /Visible ,\n/Dictionary : /NotRecorded ,\n{body}; /Dict ;\n /Part ,\n")
    };
    let compound = |name: &str, kids: &[String]| format!("/CompoundFilter :\n({name}) 0 0 /Filter ,\n1 /Visible ,\n{};\n /Part ,\n", kids.concat());
    let fill = basic("Fill Style Filter", "/FillStyle : 0 O\n0 0 0 0 0 0 1 Xa\n0 1 Xd\n/Def ;\n (FillStyle) ,\n");
    let stack = compound("Stack Style Filter", &[fill, basic("Conduit Filter", "")]);
    let root = compound("Chain Style Filter", &[stack, basic(name, keys)]);
    format!("%AI9_BeginArtStyles\n/KnownStyle :\n(Bent) /Name ,\n/ActiveStyle :\n{root}%AI9_EndArtStyles\n")
}

#[test]
fn type_with_a_distort_effect_comes_in_as_type_with_it_live() {
    // GUARDS: Free Distort, Pucker & Bloat, Twist, Zig Zag and Tweak weren't read, and type with them came in
    // without its art style, the style's drawing beside it as paths.
    use serde_json::json;
    let doc = text_document(&[("Bang\r", 0, (8200.0, 8180.0), (0.0, 0.0), 0.0)]);
    let drawing = "0 0 1 0 k\n54 56 m\n100 56 L\n100 80 L\n54 80 L\nf";
    let look = format!("u\n/AI11Text :\n1 /FreeUndo ,\n0 /FrameIndex ,\n0 /StoryIndex ,\n;\n{drawing}\nU\n9 () XW\n");
    let own: String = text_object(0).lines().map(|l| format!("%_{l}\n")).collect();
    let open = |name: &str, keys: &str| {
        let art = native_with_text(&(whole_object_style(name, keys) + &layer("Art", true, &format!("{look}{own}1 (Bent) XW\n"))), &doc);
        let (d, _) = crate::ai_alone(&private(&art)).unwrap();
        let (mut texts, mut paths) = (vec![], 0);
        d.walk(|n| match &n.kind {
            NodeKind::Text(t) => texts.push((t.plain_text(), n.appearance.effects.clone())),
            NodeKind::Path { .. } => paths += 1,
            _ => {}
        });
        (texts, paths)
    };
    for (name, keys, id, param, want) in [
        ("Adobe Twirl", "45 /Real (angle) ,\n", "distort.twist", "angle", json!(45.0)),
        (
            "Adobe Transform",
            "5 /Real (moveH_Pts) ,\n-7 /Real (moveV_Pts) ,\n0 /Bool (transformPatterns) ,\n",
            "distort.transform",
            "moveV",
            json!(7.0),
        ),
        (
            "Adobe Scribble and Tweak",
            "30 /Real (horz) ,\n20 /Real (vert) ,\n0 /Bool (anch) ,\n0 /Bool (in) ,\n0 /Bool (out) ,\n",
            "distort.tweak",
            "h",
            json!(30.0),
        ),
        ("Adobe Punk and Bloat", "-50 /Real (d_factor) ,\n", "distort.puckerBloat", "amount", json!(-50.0)),
        ("Adobe Zigzag", "3 /Real (amount) ,\n1 /Real (absoluteness) ,\n7 /Real (ridges) ,\n", "distort.zigZag", "ridges", json!(7)),
        ("Adobe Free Distort", "", "distort.freeDistort", "corners", json!([[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]])),
    ] {
        let (texts, paths) = open(name, keys);
        let [(text, effects)] = texts.as_slice() else { panic!("{name}: {texts:?}") };
        assert_eq!((text.as_str(), paths), ("Bang", 0), "{name}");
        let [e] = effects.as_slice() else { panic!("{name}: {effects:?}") };
        assert_eq!((e.id.as_str(), &e.params[param]), (id, &want), "{name}");
    }
    // An angle that isn't a number: the drawing as paths, and no effect on the type.
    let (texts, paths) = open("Adobe Twirl", "NaN /Real (angle) ,\n");
    assert_eq!(paths, 1, "{texts:?}");
    assert!(texts.iter().all(|(_, effects)| effects.is_empty()), "{texts:?}");
}

#[test]
fn type_with_convert_to_shape_comes_in_as_type_with_it_live() {
    // GUARDS: `Adobe Shape Effects` wasn't read, and type with it came in without its art style,
    // the style's drawing beside it as paths.
    let shape = "/BasicFilter :\n(Adobe Shape Effects) 1 0 /Filter ,\n1 /Visible ,\n/Dictionary : /NotRecorded ,\n\
        (RoundedRectangle) /UnicodeString (DisplayString) ,\n1 /Int (Shape) ,\n0 /Real (Absolute) ,\n4 /Real (RelWidth) ,\n\
        2 /Real (RelHeight) ,\n3 /Real (CornerRadius) ,\n; /Dict ;\n /Part ,\n";
    let stack = "/CompoundFilter :\n(Stack Style Filter) 0 0 /Filter ,\n1 /Visible ,\n/BasicFilter :\n(Fill Style Filter) 0 0 /Filter ,\n1 /Visible ,\n\
        /Dictionary : /NotRecorded ,\n/FillStyle : 0 O\n0 0 0 0 0 0 1 Xa\n0 1 Xd\n/Def ;\n (FillStyle) ,\n; /Dict ;\n /Part ,\n\
        /BasicFilter :\n(Conduit Filter) 0 0 /Filter ,\n1 /Visible ,\n;\n /Part ,\n;\n /Part ,\n";
    let styles = format!(
        "%AI9_BeginArtStyles\n/KnownStyle :\n(Boxed) /Name ,\n/ActiveStyle :\n/CompoundFilter :\n(Chain Style Filter) 0 0 /Filter ,\n1 /Visible ,\n\
        {shape}{stack};\n /Part ,\n%AI9_EndArtStyles\n"
    );
    let doc = text_document(&[("Bang\r", 0, (8200.0, 8180.0), (0.0, 0.0), 0.0)]);
    let drawing = "0 0 1 0 k\n54 56 m\n100 56 L\n100 80 L\n54 80 L\nf";
    let look = format!("u\n/AI11Text :\n1 /FreeUndo ,\n0 /FrameIndex ,\n0 /StoryIndex ,\n;\n{drawing}\nU\n9 () XW\n");
    let own: String = text_object(0).lines().map(|l| format!("%_{l}\n")).collect();
    let art = native_with_text(&(styles + &layer("Art", true, &format!("{look}{own}1 (Boxed) XW\n"))), &doc);
    let (d, notes) = crate::ai_alone(&private(&art)).unwrap();
    let (mut texts, mut paths) = (vec![], 0);
    d.walk(|n| match &n.kind {
        NodeKind::Text(t) => texts.push((t.plain_text(), n.appearance.effects.clone())),
        NodeKind::Path { .. } => paths += 1,
        _ => {}
    });
    let [(text, effects)] = texts.as_slice() else { panic!("{texts:?} {notes:?}") };
    assert_eq!((text.as_str(), paths), ("Bang", 0), "{notes:?}");
    let [e] = effects.as_slice() else { panic!("{effects:?}") };
    assert_eq!(
        (e.id.as_str(), &e.params),
        (
            "convertToShape.roundedRectangle",
            &serde_json::json!({"relative": true, "extraW": 8.0, "extraH": 4.0, "width": 36.0, "height": 36.0, "radius": 3.0})
        )
    );
}

#[test]
fn type_with_a_variable_width_stroke_keeps_it_live() {
    // GUARDS: other objects with a variable-width stroke come in as their drawn look; type keeps the
    // stroke, since its drawn look can't be retyped.
    use vectorcraft_doc::AppearanceItem;
    let doc = text_document(&[("Bang\r", 0, (8200.0, 8180.0), (0.0, 0.0), 0.0)]);
    let style = "%AI9_BeginArtStyles\n/KnownStyle :\n(Tapered) /Name ,\n/ActiveStyle :\n/CompoundFilter :\n(Chain Style Filter) 0 0 /Filter ,\n1 /Visible ,\n\
        /CompoundFilter :\n(Stack Style Filter) 0 0 /Filter ,\n1 /Visible ,\n/BasicFilter :\n(Stroke Style Filter) 0 0 /Filter ,\n1 /Visible ,\n\
        2 /FillOrStroke ,\n/Dictionary : /NotRecorded ,\n/StrokeStyle : 0 R\n0 0 0 1 0 0 0 XA\n0 J 0 j 2 w 10 M []0 d\n0 XR\n1 0 Xd\n/Def ;\n (StrokeStyle) ,\n\
        /Dictionary : /NotRecorded ,\n/Array :\n0 /Real ,\n0.5 /Real ,\n1 /Real ,\n; (BSVariableWidthTValuesArray) ,\n\
        /Array :\n0 /Real ,\n1 /Real ,\n0 /Real ,\n; (BSCenteredWidthsArray) ,\n; (kAIBeautifulStrokesParamsDictKey) ,\n; /Dict ;\n /Part ,\n\
        /BasicFilter :\n(Conduit Filter) 0 0 /Filter ,\n1 /Visible ,\n;\n /Part ,\n;\n /Part ,\n;\n%AI9_EndArtStyles\n";
    let drawing = "0 0 0 1 k\n54 56 m\n100 56 L\n100 80 L\n54 80 L\nf";
    let look = format!("u\n/AI11Text :\n1 /FreeUndo ,\n0 /FrameIndex ,\n0 /StoryIndex ,\n;\n{drawing}\nU\n9 () XW\n");
    let own: String = text_object(0).lines().map(|l| format!("%_{l}\n")).collect();
    let art = native_with_text(&(style.to_string() + &layer("Art", true, &format!("{look}{own}1 (Tapered) XW\n"))), &doc);
    let (d, notes) = crate::ai_alone(&private(&art)).unwrap();
    let mut texts = vec![];
    d.walk(|n| {
        if let NodeKind::Text(t) = &n.kind {
            texts.push((t.plain_text(), n.appearance.clone()));
        }
    });
    let [(text, a)] = texts.as_slice() else { panic!("{texts:?} {notes:?}") };
    assert_eq!(text, "Bang");
    let [AppearanceItem::Stroke(stroke)] = a.items.as_slice() else { panic!("{a:?}") };
    assert_eq!(stroke.profile.as_ref().map(|p| p.points.clone()), Some(vec![(0.0, 0.0, 0.0), (0.5, 1.0, 1.0), (1.0, 0.0, 0.0)]));
    assert!(!notes.iter().any(|w| w.contains("drawn look")), "{notes:?}");
}

#[test]
fn point_type_without_a_matrix_is_placed_by_its_layout() {
    // GUARDS: Illustrator 24 leaves an unturned, unscaled point type's matrix out of its frame; such
    // stories weren't read, and the type was left out.
    let with = text_document(&[("Hi there\r", 0, (8200.0, 8180.0), (0.0, 0.0), 0.0)]);
    let without = with.replace("/2 << /2 [ 1 0 0 1 0 0 ] >>", "/2 << >>");
    assert_ne!(with, without);
    let place = |doc: &str| {
        let art = native_with_text(&layer("Spare", false, &text_object(0)), doc);
        let (d, notes) = crate::ai_alone(&private(&art)).unwrap();
        let texts = text_of(&d.layers[0]);
        assert_eq!(texts.len(), 1, "{notes:?}");
        (texts[0].plain_text(), texts[0].xf)
    };
    assert_eq!(place(&without), place(&with));
}

/// A placed file is put where its matrix takes the canvas point of art space's origin (the
/// centre of `%AI3_TemplateBox` on the canvas); many files put it at the corner of the box it is
/// written with instead. When the layers don't draw as the page with the first, the second is
/// tried.
#[test]
fn a_placed_file_at_its_boxs_corner_matches_the_page() {
    // A 2 × 2 preview of half-transparent black (premultiplied ARGB), of a file whose box is
    // 8000 8000 8040 8040 on the canvas: 40 × 40 units, which the matrix takes to (30, 70)–(70,
    // 30) in art space, a square at (30, 30)–(70, 70) on the 100 × 100 page. The template box's
    // canvas point (8091.5, 8241.5) would put it off the page.
    let pixel = "\x7f\0\0\0";
    let place = format!(
        "%AI5_BeginPlace\n%%FileType: 1\n[1 0 0 -1 -7970 8070] 8000 8000 8040 8040 4 4 0\n(photo.png)`\n\
         %AI26_BeginPlacedObjectPreview\n2 2 8 32 17 {}\n%AI26_EndPlacedObjectPreview\n~\n%AI5_EndPlace\n",
        pixel.repeat(4)
    );
    let data = native(&layer("Jig", true, &place)).replace("%AI5_NumLayers", "%AI3_TemplateBox: 100 50 100 50\n%AI5_NumLayers");
    // The page: the square in 50% grey, as the PDF part draws the image.
    let mut page = Document::new(100.0, 100.0);
    let id = page.alloc_id();
    let path = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(30.0, 30.0, 70.0, 70.0));
    let grey = vectorcraft_doc::FillLayer::new(vectorcraft_color::Paint::solid(vectorcraft_color::Color::gray(0.5)));
    let n = Node::path(id, path, vectorcraft_doc::Appearance { items: vec![vectorcraft_doc::AppearanceItem::Fill(grey)], ..Default::default() });
    let l = page.layers[0].id;
    page.insert(Some(l), 0, n).unwrap();
    let (d, w, read) = crate::layered_ai(&private(&data), page, vec![], false, false, 1.0);
    assert!(read, "{w:?}");
    assert_eq!(names(&d), ["Jig"]);
    let image = &d.layers[0].children().unwrap()[0];
    assert!(matches!(image.kind, NodeKind::Image(_)), "{:?}", image.kind);
    let b = image.geometric_bounds().unwrap();
    assert!([b.x0 - 30.0, b.y0 - 30.0, b.x1 - 70.0, b.y1 - 70.0].iter().all(|v| v.abs() < 0.01), "on the page's square: {b:?}");
}

/// A placed file of [`a_placed_file_at_its_boxs_corner_matches_the_page`] whose dictionary names
/// page 2 and its media box: `%_` lines when `hidden` (art inside another object's dictionary).
fn placed_page_two(hidden: bool) -> String {
    let lines = [
        "%AI5_BeginPlace".to_string(),
        "%%FileType: 1".into(),
        "[1 0 0 -1 -7970 8070] 8000 8000 8040 8040 4 4 0".into(),
        "(/files/pages.pdf)`".into(),
        "%AI26_BeginPlacedObjectPreview".into(),
        format!("2 2 8 32 17 {}", "\x7f\0\0\0".repeat(4)),
        "%AI26_EndPlacedObjectPreview".into(),
        "~".into(),
        "%AI5_EndPlace".into(),
    ];
    let hide = if hidden { "%_" } else { "" };
    let place: String = lines.iter().map(|l| format!("{hide}{l}\n")).collect();
    format!(
        "{place}%_/ArtDictionary :\n%_/XMLUID : (pages) ; (AI10_ArtUID) ,\n%_2 /Int (AI11 PlacedPDFPageIndex) ,\n%_4 /Int (AI11 PlacedPDFCropTo) ,\n%_;\n%_\n"
    )
}

/// The placed document of `n`, with its bounds.
fn placed_document(n: &Node) -> (&vectorcraft_doc::PlacedDocument, vectorcraft_geom::Rect) {
    let NodeKind::PlacedDocument(p) = &n.kind else { panic!("{:?}", n.kind) };
    (p, n.geometric_bounds().unwrap())
}

/// A placed PDF page comes in as a placed document of its page and box, where its linked image
/// would have gone: at its box's corner when that is where the layers draw as the page does.
#[test]
fn a_placed_pdf_page_is_a_placed_document_where_its_image_would_be() {
    let data = native(&layer("Jig", true, &placed_page_two(false))).replace("%AI5_NumLayers", "%AI3_TemplateBox: 100 50 100 50\n%AI5_NumLayers");
    let mut page = Document::new(100.0, 100.0);
    let id = page.alloc_id();
    let path = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(30.0, 30.0, 70.0, 70.0));
    let grey = vectorcraft_doc::FillLayer::new(vectorcraft_color::Paint::solid(vectorcraft_color::Color::gray(0.5)));
    let n = Node::path(id, path, vectorcraft_doc::Appearance { items: vec![vectorcraft_doc::AppearanceItem::Fill(grey)], ..Default::default() });
    let l = page.layers[0].id;
    page.insert(Some(l), 0, n).unwrap();
    let (d, w, read) = crate::layered_ai(&private(&data), page, vec![], false, false, 1.0);
    assert!(read, "{w:?}");
    let (p, b) = placed_document(&d.layers[0].children().unwrap()[0]);
    assert_eq!((p.link.path.as_str(), p.link.page, p.page_box, p.bounding), ("/files/pages.pdf", Some(2), vectorcraft_doc::PageBox::Media, false));
    assert_eq!((p.width, p.height), (40.0, 40.0));
    assert!([b.x0 - 30.0, b.y0 - 30.0, b.x1 - 70.0, b.y1 - 70.0].iter().all(|v| v.abs() < 0.01), "on the page's square: {b:?}");
    assert!(d.images[&p.key].is_proxy(), "the preview until the file is read");
}

/// A placed PDF page in an object's opacity mask is a placed document there too.
#[test]
fn a_placed_pdf_page_in_an_opacity_mask_is_a_placed_document() {
    let mask = format!("%_/ArtDictionary :\n%_/Mask :\n%_X=\n{}%_X+\n%_ (Art) ,\n%_; (AI9_Mask) ,\n%_;\n%_\n", placed_page_two(true));
    let data = native(&layer("Masked", true, &format!("{}\n{mask}", square(10, 10))));
    let (d, notes) = crate::ai_alone(&private(&data)).unwrap();
    let square = &d.layers[0].children().unwrap()[0];
    let mask = square.extras().mask.clone().unwrap_or_else(|| panic!("{notes:?}"));
    let art = if let NodeKind::Group { children, .. } = &mask.art.kind { children[0].clone() } else { mask.art.clone() };
    let (p, _) = placed_document(&art);
    assert_eq!((p.link.page, p.page_box), (Some(2), vectorcraft_doc::PageBox::Media));
}

/// A placed PDF page whose box is too small for a finite placement stays a linked image.
#[test]
fn a_placed_pdf_page_too_small_to_place_stays_a_linked_image() {
    let place = placed_page_two(false).replace("[1 0 0 -1 -7970 8070] 8000 8000 8040 8040", "[1e308 0 0 -1 0 200] 0 0 1e-309 2");
    let data = native(&layer("Jig", true, &place));
    let (d, notes) = crate::ai_alone(&private(&data)).unwrap();
    let mut kinds = vec![];
    d.walk(|n| match &n.kind {
        NodeKind::Image(im) => kinds.push(("image", im.xf.as_coeffs().iter().all(|v| v.is_finite()))),
        NodeKind::PlacedDocument(p) => kinds.push(("placed", p.xf.as_coeffs().iter().all(|v| v.is_finite()))),
        _ => {}
    });
    assert_eq!(kinds, [("image", true)], "{notes:?}");
}

/// A placed PDF page bound to an image variable stays a linked image, the object an image variable
/// binds, so the binding is kept.
#[test]
fn a_placed_pdf_page_bound_to_an_image_variable_stays_a_linked_image() {
    use vectorcraft_testkit::ai::{XmlValue, variable_library_data, xml_element, xml_node};
    let declared = vec![xml_node(
        1,
        "variable",
        XmlValue::Str(""),
        &[("trait", XmlValue::Str("fileref")), ("varName", XmlValue::Unicode("v_file")), ("docRef", XmlValue::Raw("/XMLUIDREF : (pages) ;"))],
        &[],
    )];
    let set = xml_element("variableSet", &[("locked", "none")], &[xml_element("variables", &[], &declared)]);
    let library = variable_library_data(&xml_element("variableSets", &[], &[set]));
    let data = native(&(library + &layer("Jig", true, &placed_page_two(false))));
    let (d, notes) = crate::ai_alone(&private(&data)).unwrap();
    let n = &d.layers[0].children().unwrap()[0];
    assert!(matches!(&n.kind, NodeKind::Image(im) if im.link.as_ref().is_some_and(|l| l.page.is_none())), "{:?} {notes:?}", n.kind);
    assert_eq!(d.variables.objects_of("v_file"), [n.id], "{notes:?}");
}

#[test]
fn a_small_placed_file_goes_where_it_draws_closer_to_the_page() {
    // A 2 × 2 placed file (a small logo on a large page) whose box corner the matrix takes to
    // (30, 30)–(32, 32), as the page draws it. The template box's canvas point puts it at
    // (10, 10)–(12, 12): on the page too, and small enough that the layers pass the comparison
    // there all the same.
    let place = format!(
        "%AI5_BeginPlace\n%%FileType: 1\n[1 0 0 -1 -7970 8070] 8000 8000 8002 8002 4 4 0\n(logo.png)`\n\
         %AI26_BeginPlacedObjectPreview\n2 2 8 32 17 {}\n%AI26_EndPlacedObjectPreview\n~\n%AI5_EndPlace\n",
        "\x7f\0\0\0".repeat(4)
    );
    let data = native(&layer("Jig", true, &place)).replace("%AI5_NumLayers", "%AI3_TemplateBox: 211.5 -211.5 211.5 -211.5\n%AI5_NumLayers");
    let mut page = Document::new(100.0, 100.0);
    let id = page.alloc_id();
    let path = vectorcraft_geom::shapes::rectangle(vectorcraft_geom::Rect::new(30.0, 30.0, 32.0, 32.0));
    let grey = vectorcraft_doc::FillLayer::new(vectorcraft_color::Paint::solid(vectorcraft_color::Color::gray(0.5)));
    let n = Node::path(id, path, vectorcraft_doc::Appearance { items: vec![vectorcraft_doc::AppearanceItem::Fill(grey)], ..Default::default() });
    let l = page.layers[0].id;
    page.insert(Some(l), 0, n).unwrap();
    let (d, w, read) = crate::layered_ai(&private(&data), page, vec![], false, false, 1.0);
    assert!(read, "{w:?}");
    let b = d.layers[0].children().unwrap()[0].geometric_bounds().unwrap();
    assert!([b.x0 - 30.0, b.y0 - 30.0, b.x1 - 32.0, b.y1 - 32.0].iter().all(|v| v.abs() < 0.01), "on the page's square: {b:?}");
}

#[test]
fn text_a_placed_file_covers_only_at_its_first_placement_is_kept() {
    const OPAQUE: &str = "<opaque>";
    // An opaque black 2 × 2 preview of a file whose box corner the matrix takes to (30, 30)–(70,
    // 70), as the page draws it. The template box's canvas point puts it 10 to the left instead,
    // over the 8 pt text object "Hi" at (21, 48): there the text can't be seen, and the page has
    // no type for it (it draws it as outlines), so the text looks like type the app doesn't draw.
    // The layers then differ from the page by little more than `compare` allows, so that is
    // weighed. With the file at its box's corner the text shows, as the page has it.
    let place = format!(
        "%AI5_BeginPlace\n%%FileType: 1\n[1 0 0 -1 -7970 8070] 8000 8000 8040 8040 4 4 0\n(photo.png)`\n\
         %AI26_BeginPlacedObjectPreview\n2 2 8 32 17 {}\n%AI26_EndPlacedObjectPreview\n~\n%AI5_EndPlace\n",
        OPAQUE.repeat(4)
    );
    // Canvas centre 8191.5 is the template box's centre (201.5, -191.5): the anchor (8011, 7952) is
    // (21, 48) in art space.
    let document = text_document(&[("Hi\r", 0, (8011.0, 7952.0), (0.0, 0.0), 0.0)]).replace("/1 14.0", "/1 8.0");
    let data = native_with_text(&layer("Art", true, &(text_object(0) + &place)), &document)
        .replace("%AI3_TemplateBox: 50 50 50 50", "%AI3_TemplateBox: 201.5 -191.5 201.5 -191.5");
    // The page (y down): the image's black square, and the type's outlines, a magenta box where
    // its glyphs are.
    let mut page = Document::new(100.0, 100.0);
    let l = page.layers[0].id;
    for (rect, color) in [
        (vectorcraft_geom::Rect::new(30.0, 30.0, 70.0, 70.0), vectorcraft_color::Color::BLACK),
        (vectorcraft_geom::Rect::new(21.5, 46.5, 28.0, 52.0), vectorcraft_color::Color::Cmyk { c: 0.0, m: 1.0, y: 0.0, k: 0.0 }),
    ] {
        let id = page.alloc_id();
        let fill = vectorcraft_doc::FillLayer::new(vectorcraft_color::Paint::solid(color));
        let n = Node::path(
            id,
            vectorcraft_geom::shapes::rectangle(rect),
            vectorcraft_doc::Appearance { items: vec![vectorcraft_doc::AppearanceItem::Fill(fill)], ..Default::default() },
        );
        page.insert(Some(l), 0, n).unwrap();
    }
    // The preview's pixels are opaque black (ARGB `ff 00 00 00`), bytes a `str` can't hold.
    let bytes =
        data.replace(OPAQUE, "\u{1}").into_bytes().into_iter().flat_map(|b| if b == 1 { vec![0xff, 0, 0, 0] } else { vec![b] }).collect::<Vec<u8>>();
    let private =
        [b"%AI24_ZStandard_Data".as_slice(), &ruzstd::encoding::compress_to_vec(&bytes[..], ruzstd::encoding::CompressionLevel::Fastest)].concat();
    let (d, w, read) = crate::layered_ai(&private, page, vec![], false, false, 1.0);
    assert!(read, "{w:?}");
    let art = d.layers[0].children().unwrap();
    let image = art.iter().find(|n| matches!(n.kind, NodeKind::Image(_))).expect("the image");
    let b = image.geometric_bounds().unwrap();
    assert!([b.x0 - 30.0, b.y0 - 30.0, b.x1 - 70.0, b.y1 - 70.0].iter().all(|v| v.abs() < 0.01), "on the page's square: {b:?}");
    assert_eq!(text_of(&d.layers[0]).iter().map(|t| t.plain_text()).collect::<Vec<_>>(), ["Hi"], "{w:?}");
    assert!(!w.iter().any(|w| w.contains("doesn't draw")), "{w:?}");
}

/// #6865: explicitly trusting editing data keeps an artboard larger than its PDF page.
#[test]
fn trusted_editing_data_keeps_an_artboard_taller_than_its_pdf_page() {
    let art = native(&layer("Oversized", true, &square(10, 10))).replace(" 0 0 100 100", " 0 0 12992.726 14637.147");
    let (d, w, read) = crate::layered_ai(&private(&art), Document::new(12992.7, 14400.0), vec![], false, true, 1.0);
    assert!(read, "{w:?}");
    assert_eq!(names(&d), ["Oversized"], "{w:?}");
    assert!((d.artboards[0].rect.height() - 14637.147).abs() < 0.001);
    assert_eq!(kinds(&d.layers[0]), ["path"]);
    assert!(w.iter().any(|w| w.contains("artboard is larger than the PDF page")), "{w:?}");
    let child = &d.layers[0].children().unwrap()[0];
    assert!(child.visual_bounds().unwrap().y0 > 14400.0, "art beyond the PDF page is kept");
    let (_, w, read) = crate::layered_ai(&private(&art), Document::new(12992.7, 14400.0), vec![], false, false, 1.0);
    assert!(!read, "default import still rejects the size mismatch: {w:?}");
    let (_, w, read) = crate::layered_ai(&private(&art), Document::new(12000.0, 14400.0), vec![], false, true, 1.0);
    assert!(!read, "the uncapped dimension must still match: {w:?}");
}
