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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radix_number_with_sign_after_hash_is_not_a_number() {
        assert!(number("16#+ff").is_none());
        assert!(number("16#-ff").is_none());
        assert!(matches!(number("16#ff"), Some(Obj::Int(255))));
    }
}
