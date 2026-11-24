//! PostScript writing primitives: numbers, strings, paths, the prolog's short operator names, and
//! the data encodings images and embedded data use (ASCII85, run-length, Flate).

/// Most bytes [`ascii85_decode`] makes: what one filter or data section may hold.
pub(crate) const MAX_ASCII85: usize = 1 << 28;

/// ASCII85 `text` (whitespace ignored, up to `~>`) decoded; `None` when it is malformed or longer
/// than [`MAX_ASCII85`].
pub(crate) fn ascii85_decode(text: &str) -> Option<Vec<u8>> {
    ascii85_decode_within(text, MAX_ASCII85)
}

/// [`ascii85_decode`] of at most `max` bytes: `z` is four bytes for one, so the length of the
/// text doesn't bound what it becomes (#1715).
pub(crate) fn ascii85_decode_within(text: &str, max: usize) -> Option<Vec<u8>> {
    craft_eps::ascii85::decode(text.as_bytes(), max)
}

