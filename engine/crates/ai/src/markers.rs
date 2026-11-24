//! Markers VectorCraft wrote into the files it saved, which reading those files recognises: the
//! creator line of the editing data, the keys of dynamic symbols and their overrides, and the
//! procedure set of its legacy `.ai` files.

/// `%%Creator`: the creator line VectorCraft wrote in the header of the editing data. The reader
/// reads the symbols of data with this creator at their own size.
pub(crate) const CREATOR_TEXT: &str = "VectorCraft, editing data for Illustrator"; // brand-ok: the creator line VectorCraft wrote

/// The key that marks a symbol as dynamic in the editing data.
pub(crate) const DYNAMIC_KEY: &str = "VectorCraftDynamicSymbol";

/// The key under which a symbol instance's overrides are kept, hex-encoded.
pub(crate) const OVERRIDES_KEY: &str = "VectorCraftSymbolOverrides";

/// The most bytes of overrides read for one instance.
const MAX_OVERRIDES_BYTES: usize = 1 << 20;

/// `s`, hex digits two per byte, as bytes; `None` when it isn't hex or holds too many.
pub(crate) fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) || s.len() / 2 > MAX_OVERRIDES_BYTES {
        return None;
    }
    s.as_bytes().chunks(2).map(|c| u8::from_str_radix(std::str::from_utf8(c).ok()?, 16).ok()).collect()
}

/// The preset (position in the layer colour list) of the colour index `index` in an `.ai` layer's
/// `Lb`.
pub(crate) fn preset_of_ai_layer_index(index: u8) -> u8 {
    match index {
        6 => 4,
        4 => 5,
        5 => 6,
        i => i,
    }
}
