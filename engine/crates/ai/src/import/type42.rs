//! Type 42 fonts' glyphs drawn from the TrueType font they carry, for the type a composite font
//! shows: its glyphs are picked by glyph index, often in a subset font without a character map,
//! so they can't be set again as type in an installed font. The font is the `sfnts` strings
//! joined, each string of odd length ending in a padding byte that isn't part of it (Adobe
//! Technical Note #5012, "The Type 42 Font Format Specification", on the `sfnts` array); a glyph
//! name maps to its glyph index through `CharStrings`.

