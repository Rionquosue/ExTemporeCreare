//! EPS and PostScript import: a small clean-room PostScript interpreter that turns what a file
//! draws into document objects.
//!
//! - `lex`: the scanner; `obj`: objects, operators and errors; `interp`: the stacks, running
//!   procedures, and the operators that don't draw.
//! - `graphics`: the graphics state, coordinates, paths, colours (grey, RGB, CMYK, spot inks as
//!   spot swatches at a tint, indexed, patterns), painting, clipping, axial and radial shadings as
//!   gradients and tiling patterns as pattern swatches.
//! - `shading`: function-based and mesh shadings as gradient meshes, sampled functions.
//! - `data`: data the program reads from itself through decoding filters, and images.
//! - `text`: fonts by name and type as point type; Type 3 fonts' glyphs drawn by their procedures.
//! - `preview`: the previews a file carries, for when its PostScript can't be read.
//! - `native`: the layers of an Illustrator file, from the editing copy of its art that it carries.
//!
//! Clipped art comes in as clipping groups. In a file in the legacy Illustrator format (versions
//! 3–8, with the prolog that defines its operators), the group operators `u` … `U` (nested) come in
//! as groups. The format's published specification documents its header comments (`%AI…`,
//! `%%Creator`) and these operators: Adobe Illustrator File Format Specification, version 7.0
//! (1998), listed by PRONOM at https://www.nationalarchives.gov.uk/PRONOM/fmt/423. A file in that
//! format whose program has its layers (`%AI5_BeginLayer`) comes in through them, as `native` reads
//! an Illustrator EPS's editing copy (#1027), and one without layers (Illustrator 3) comes in
//! through its art put on one layer, as its page when that can't be read. One VectorCraft wrote (its prolog the procedure set
//! [`crate::markers::PROCSET`], which prints its gradients flat and leaves its type out) comes in
//! from those layers alone, without its page.
//!
//! The page is the file's `%%BoundingBox` (`%%HiResBoundingBox` when it has one), the first page
//! of a PostScript file without one. A `.ai` in the legacy format ([`import_ai`]) that comes in
//! through its layers keeps the artboard its header gives, with its art outside it; an EPS comes in
//! on its page. A file whose program can't be read (an operator the
//! interpreter doesn't know, an error, a limit reached) or that draws nothing comes in from its
//! layers when its program has them (a `.ai` in PostScript form whose prolog is named but not
//! included, as other apps write it), or with its art on one layer when it is in the legacy format
//! without layers (Illustrator 3, as image editors still export their paths), else as its preview
//! (a TIFF image, a Windows metafile or an EPSI bitmap), with a warning that names the error, the
//! operator and the procedures it ran in; without a preview, what was drawn before the error is
//! kept (with that warning), else the file is refused.

mod ai;
mod ate;
mod cid;
mod graphics;
mod interp;
mod lex;
mod native;
mod obj;
mod predefined_cmap;
mod shading;
mod text;
mod type1;
mod type42;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_cid;
#[cfg(test)]
mod tests_composite;
#[cfg(test)]
mod tests_fontnames;
#[cfg(test)]
mod tests_generators;
#[cfg(test)]
mod tests_illustrator;
#[cfg(test)]
mod tests_images;
#[cfg(test)]
mod tests_native;
#[cfg(test)]
mod tests_reads;
#[cfg(test)]
mod tests_strings;
#[cfg(test)]
mod tests_type1;
#[cfg(test)]
mod tests_type42;

use vectorcraft_doc::Document;

pub use ai::NameEncoding;
#[cfg(test)]
pub(crate) use ai::objects_read;
#[cfg(test)]
pub(crate) use ai::{ART_STYLE_PARSES, ENVELOPE_DRAWN, ENVELOPE_TYPE_DRAWN};
pub use native::{AiLayers, ai_alone, ai_alone_on_page, ai_alone_with, ai_layers_alone, ai_opened_artboard, is_loss, layered_ai, layered_ai_from};
pub(crate) use native::scale_symbols;
pub use text::family_style;

/// The page of a PostScript file without a bounding box (US Letter).
const LETTER: [f64; 4] = [0.0, 0.0, 612.0, 792.0];

/// A read file.
#[derive(Debug)]
pub struct Imported {
    pub document: Document,
    pub warnings: Vec<String>,
    /// The document shows the file's preview: its PostScript couldn't be read.
    pub preview: bool,
}

