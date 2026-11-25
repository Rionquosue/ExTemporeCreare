//! One-off: writes the built-in libraries as data files under tools/Libraries/presets/.
use crate::cmd::{brushlib, patternlib, symbollib};
use std::path::PathBuf;

fn write_kind(root: &PathBuf, kind: &str, items: Vec<(String, String, String, Vec<u8>)>) {
    let dir = root.join(kind);
    std::fs::create_dir_all(&dir).unwrap();
    let mut index = vec![];
    for (id, name, file, bytes) in items {
        std::fs::write(dir.join(&file), bytes).unwrap();
        index.push(serde_json::json!({"id": id, "name": name, "file": file}));
    }
    let text = serde_json::to_string_pretty(&serde_json::json!({"libraries": index})).unwrap() + "\n";
    std::fs::write(dir.join("index.json"), text).unwrap();
}

#[test]
#[ignore]
fn export_presets() {
    let root = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../tools/Libraries/presets"));
    let brushes = brushlib::BUILTIN
        .iter()
        .map(|b| {
            let lib = brushlib::builtin_library(b.id).unwrap();
            (b.id.to_string(), b.name.to_string(), format!("{}.etc", b.id), etcfile::save(&lib.doc, true))
        })
        .collect();
    write_kind(&root, "brushes", brushes);
    let symbols = symbollib::BUILTIN
        .iter()
        .map(|b| {
            let lib = symbollib::builtin_library(b.id).unwrap();
            (b.id.to_string(), b.name.to_string(), format!("{}.etc", b.id), etcfile::save(&lib.doc, true))
        })
        .collect();
    write_kind(&root, "symbols", symbols);
    let patterns = patternlib::PATTERN_LIBRARIES
        .iter()
        .map(|b| (b.id.to_string(), b.name.to_string(), format!("{}.etc", b.id), etcfile::save(&patternlib::library_doc(b.id).unwrap(), true)))
        .collect();
    write_kind(&root, "patterns", patterns);
    use vectorcraft_color::libraries::{GRADIENT_LIBRARIES, SWATCH_LIBRARIES, builtin_library};
    use vectorcraft_color::palette_io::{PaletteFormat, write};
    for (kind, list) in [("swatches", SWATCH_LIBRARIES), ("gradients", GRADIENT_LIBRARIES)] {
        let items = list
            .iter()
            .map(|b| {
                let lib = builtin_library(b.id).unwrap();
                (b.id.to_string(), b.name.to_string(), format!("{}.vcswatches", b.id), write(&lib, PaletteFormat::Native).unwrap())
            })
            .collect();
        write_kind(&root, kind, items);
    }
    use vectorcraft_doc::style_libs::{STYLE_LIBRARIES, STYLES_EXT, builtin_style_library, write as write_styles};
    let styles = STYLE_LIBRARIES
        .iter()
        .map(|b| {
            let lib = builtin_style_library(b.id).unwrap();
            (b.id.to_string(), b.name.to_string(), format!("{}.{STYLES_EXT}", b.id), write_styles(&lib).into_bytes())
        })
        .collect();
    write_kind(&root, "graphic-styles", styles);
}
