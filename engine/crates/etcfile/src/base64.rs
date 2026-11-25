//! Standard base64 (RFC 4648, `+` and `/`, `=` padding). The engine's command replies carry file
//! bytes as `dataBase64`; this module encodes and decodes them.

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Marks a byte that is not in [`ALPHABET`] in [`DECODE`].
const INVALID: u8 = 0xff;

/// The 6-bit value of each base64 character, [`INVALID`] for the others.
const DECODE: [u8; 256] = {
    let mut table = [INVALID; 256];
    let mut i = 0;
    while i < 64 {
        table[ALPHABET[i] as usize] = i as u8;
        i += 1;
    }
    table
};

/// `bytes` as padded base64 text.
pub fn encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [chunk[0], chunk.get(1).copied().unwrap_or(0), chunk.get(2).copied().unwrap_or(0)];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(ALPHABET[(n >> 18) as usize & 63] as char);
        out.push(ALPHABET[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { ALPHABET[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { ALPHABET[n as usize & 63] as char } else { '=' });
    }
    out
}

/// The bytes `text` decodes to, at most (`None`: its length can't be base64). ASCII whitespace
/// is skipped by [`decode`], so this is an upper bound for text holding any.
pub fn decoded_len(text: &str) -> Option<usize> {
    let len = text.len();
    Some(len / 4 * 3 + match len % 4 {
        0 => 0,
        2 => 1,
        3 => 2,
        _ => return None,
    })
}

/// The bytes of base64 `text` (`None`: not base64). Padding is optional, and ASCII whitespace
/// (line breaks in pasted data) is skipped.
pub fn decode(text: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(text.len() / 4 * 3 + 2);
    let mut acc: u32 = 0;
    let mut bits = 0u32;
    let mut padding = 0usize;
    for &c in text.as_bytes() {
        if c.is_ascii_whitespace() {
            continue;
        }
        if c == b'=' {
            padding += 1;
            continue;
        }
        if padding > 0 {
            // Data after padding.
            return None;
        }
        let v = DECODE[c as usize];
        if v == INVALID {
            return None;
        }
        acc = (acc << 6) | u32::from(v);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    // 6 bits left over can't come from whole bytes; leftover bits must be zero.
    if bits >= 6 || acc != 0 || padding > 2 {
        return None;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_every_length() {
        let data: Vec<u8> = (0..=255).collect();
        for n in 0..data.len() {
            let text = encode(&data[..n]);
            assert_eq!(decode(&text).as_deref(), Some(&data[..n]), "{n}");
            assert!(decoded_len(&text).unwrap() >= n);
        }
    }

    #[test]
    fn known_values_and_refusals() {
        assert_eq!(encode(b"Man"), "TWFu");
        assert_eq!(encode(b"Ma"), "TWE=");
        assert_eq!(decode("TWE"), Some(b"Ma".to_vec()));
        assert_eq!(decode("TW\nFu"), Some(b"Man".to_vec()));
        assert_eq!(decode("T!Fu"), None);
        assert_eq!(decode("TWE=x"), None);
        assert_eq!(decode("T"), None);
    }
}

// GLOSSARY
// - `ALPHABET`: the 64 base64 characters in value order.
// - `INVALID`: the marker for a byte outside `ALPHABET` in `DECODE`.
// - `DECODE`: the 6-bit value of every byte, `INVALID` for non-base64 bytes.
// - `encode`: bytes to padded base64 text.
// - `decoded_len`: the most bytes a base64 text decodes to; `None` for an impossible length.
// - `decode`: base64 text to bytes, padding optional, ASCII whitespace skipped; `None` when invalid.
