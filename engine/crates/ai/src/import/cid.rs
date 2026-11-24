//! CID-keyed fonts (PLRM 3rd ed., §5.11): CMaps built with the operators of the `CIDInit`
//! procedure set (and the predefined `Identity-H` and `Identity-V`), `composefont`, Type 0
//! fonts with `FMapType` 9 that map a show string's codes through their CMap to CIDs, and the
//! glyphs of the CIDFonts they select: Type 0 CIDFonts (Type 1 charstrings in the binary data
//! `StartData` reads, or in a `GlyphDirectory`: PLRM 3rd ed., §5.11.3, and Adobe Technical Note
//! #5014, "Adobe CMap and CIDFont Files Specification") and Type 2 CIDFonts (TrueType glyphs in
//! `sfnts`, picked through `CIDMap`). CIDs carry no characters: the glyphs are drawn from the
//! font's outlines, as a composite font's are.

