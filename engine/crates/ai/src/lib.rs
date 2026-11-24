//! Illustrator `.ai` editing layers: the editing copy of its art that a PDF-compatible `.ai` file
//! carries, read into a document with its layers, hidden objects and the art outside its
//! artboards ([`ai_alone_with`], [`ai_alone_on_page`], [`ai_layers_alone`], [`layered_ai_from`]).
//!
//! The editing copy is PostScript-like: it is read by a small clean-room PostScript interpreter.
//!
//! - `import`: the interpreter and the `.ai` editing-copy reader.
//! - `ps`: numbers, strings and the data encodings (ASCII85, run-length, Flate).
//! - `latin`: the Latin glyph names and encodings of legacy text.
//! - `markers`: markers of the files VectorCraft wrote, which the reader recognises.

mod import;
mod latin;
mod markers;
mod ps;

pub use import::{
    AiLayers, Imported, NameEncoding, ai_alone, ai_alone_on_page, ai_alone_with, ai_layers_alone, ai_opened_artboard, family_style, is_loss,
    layered_ai, layered_ai_from,
};
