//! The scanner: PostScript source into objects (numbers, names, strings, procedures), with the
//! position the program's own data (`currentfile`) is read from.
//!
//! Backed by `craft_ps::read::lex`.

/// Where `needle` first starts in `hay` (uses `memmem` so large inputs stay linear-time).
pub(crate) fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    memchr::memmem::find(hay, needle)
}

