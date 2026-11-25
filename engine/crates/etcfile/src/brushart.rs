//! The art of stored brushes, mounted on the document while a save or load step walks its art,
//! so images and placed documents inside brush art count like those in the layers.

use vectorcraft_brush::stored_art::StoredArt;
use vectorcraft_doc::Document;

/// Run `f` on `d` with its stored brushes' art mounted under the ids it is stored with; brushes
/// whose art `f` changed are written back into the library, and the art is unmounted again. When
/// art is mounted already, or no stored brush holds an image or a placed document, `f` runs on
/// `d` as it is.
pub fn with_brush_art<T>(d: &mut Document, f: impl FnOnce(&mut Document) -> T) -> T {
    if !d.mounted_art.is_empty() {
        return f(d);
    }
    let art = StoredArt::read(d);
    if art.is_empty() {
        return f(d);
    }
    art.mount_as_stored(d);
    let out = f(d);
    art.write_back(d);
    out
}

/// Replace every placed document in `d`'s art (brush art included) with a group of the art it
/// shows, with the resources that art uses. Without placed documents, the resources only
/// exporting adds are dropped.
pub fn placed_as_groups(d: &mut Document) {
    with_brush_art(d, |d| {
        if d.has_placed() {
            *d = d.with_placed_art().into_owned();
            d.placed_as_groups();
        } else {
            d.drop_placed_resources();
        }
    });
}

// GLOSSARY
// - `with_brush_art`: run a closure with the stored brushes' art mounted on the document.
// - `placed_as_groups`: turn placed documents into groups of their art.
