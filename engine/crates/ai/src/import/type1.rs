//! Embedded fonts whose glyphs are drawn from the font program the file carries: Type 1 fonts,
//! whose encrypted part (`eexec`: Adobe Type 1 Font Format, chapter 7) holds their charstrings,
//! and Type 42 fonts (see `type42`). The interpreter doesn't run the encrypted part; it is read
//! by the Type 1 font parser, and the font dictionary gets the `CharStrings` dictionary the part
//! would have given it, each glyph name standing for its glyph's index, by which the glyph is
//! drawn. Its charstrings are run here, counting the bytes of the charstrings and subroutines
//! they enter, so that the work of drawing glyphs is paid for from the operation budget.

