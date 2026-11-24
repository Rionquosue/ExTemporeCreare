//! Gradients a program paints as a row of abutting strips, each in one colour (the way many
//! PostScript writers draw a smooth shading at Level 2, which has no `shfill`), read as one
//! linear gradient.
//!
//! Drawn one by one, abutting strips leave seams: a renderer that anti-aliases each fill lets the
//! art below show through along every shared edge (PDF 32000-1:2008 §10.6.4 notes the effect,
//! "conflation", for adjacent areas painted separately), which reads as thin stripes across the
//! gradient, and as a tint of the colour below over the whole of it where the strips are only a
//! pixel or two wide. A run of them under the same clips becomes one object filled with a linear
//! gradient through the strips' colours, which has no seams and stays editable.
//!
//! Only strips that clearly are a gradient merge: at least [`MIN_STRIPS`] parallelograms of one
//! length, side by side in one direction with no gap or overlap, each a small step in colour from
//! the one before ([`MAX_STEP`]). Abutting blocks of distinct colours (a bar chart, a row of
//! swatches) stay as they are.

