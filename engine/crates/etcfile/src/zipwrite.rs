//! Writes ZIP archives for `.etc` and `.etcr` files (`filestandard.md` §3): UTF-8 names, each
//! entry STORED or DEFLATE, ZIP64 records when an entry, an offset or the entry count passes the
//! classic ZIP limits.

use std::io::{self, Write};

use flate2::Compression;
use flate2::write::DeflateEncoder;

/// How an entry's bytes are kept in the archive.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    /// Uncompressed (ZIP method 0): `mimetype` and already-compressed images.
    Stored,
    /// DEFLATE (ZIP method 8): JSON, XML and other text.
    Deflate,
}

/// Fixed DOS date (1980-01-01) and time (00:00): the same document writes the same bytes.
const DOS_DATE: u16 = (1 << 5) | 1;
const DOS_TIME: u16 = 0;
/// General-purpose flag 11: names are UTF-8.
const UTF8_NAMES: u16 = 1 << 11;
/// "Version needed to extract": 2.0 for plain entries, 4.5 for ZIP64 ones.
const VERSION_PLAIN: u16 = 20;
const VERSION_ZIP64: u16 = 45;
/// Classic ZIP fields hold at most these; larger values go into ZIP64 records.
const MAX_U32: u64 = 0xffff_ffff;
const MAX_U16: u64 = 0xffff;

/// One entry already written: what the central directory repeats about it.
struct Written {
    name: String,
    method: Method,
    crc: u32,
    packed: u64,
    size: u64,
    offset: u64,
}

/// A ZIP archive being written to `W` entry by entry; [`ZipWriter::finish`] writes the central
/// directory.
pub struct ZipWriter<W: Write> {
    out: W,
    at: u64,
    entries: Vec<Written>,
}

impl<W: Write> ZipWriter<W> {
    pub fn new(out: W) -> Self {
        Self { out, at: 0, entries: Vec::new() }
    }

    /// Add `name` holding `bytes`, kept as `method` says.
    pub fn add(&mut self, name: &str, bytes: &[u8], method: Method) -> io::Result<()> {
        if self.entries.iter().any(|e| e.name == name) {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("the archive already has an entry named {name}")));
        }
        let crc = crc32fast::hash(bytes);
        let packed_bytes;
        let data: &[u8] = match method {
            Method::Stored => bytes,
            Method::Deflate => {
                let mut enc = DeflateEncoder::new(Vec::with_capacity(bytes.len() / 2), Compression::default());
                enc.write_all(bytes)?;
                packed_bytes = enc.finish()?;
                &packed_bytes
            }
        };
        let (size, packed, offset) = (bytes.len() as u64, data.len() as u64, self.at);
        let zip64 = size >= MAX_U32 || packed >= MAX_U32;
        let name_bytes = name.as_bytes();
        let mut extra = Vec::new();
        if zip64 {
            // Local ZIP64 extra field: both sizes.
            extra.extend_from_slice(&0x0001u16.to_le_bytes());
            extra.extend_from_slice(&16u16.to_le_bytes());
            extra.extend_from_slice(&size.to_le_bytes());
            extra.extend_from_slice(&packed.to_le_bytes());
        }
        let mut h = Vec::with_capacity(30 + name_bytes.len() + extra.len());
        h.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
        h.extend_from_slice(&(if zip64 { VERSION_ZIP64 } else { VERSION_PLAIN }).to_le_bytes());
        h.extend_from_slice(&UTF8_NAMES.to_le_bytes());
        h.extend_from_slice(&method_code(method).to_le_bytes());
        h.extend_from_slice(&DOS_TIME.to_le_bytes());
        h.extend_from_slice(&DOS_DATE.to_le_bytes());
        h.extend_from_slice(&crc.to_le_bytes());
        h.extend_from_slice(&(if zip64 { MAX_U32 as u32 } else { packed as u32 }).to_le_bytes());
        h.extend_from_slice(&(if zip64 { MAX_U32 as u32 } else { size as u32 }).to_le_bytes());
        h.extend_from_slice(&u16::try_from(name_bytes.len()).map_err(|_| too_long(name))?.to_le_bytes());
        h.extend_from_slice(&(extra.len() as u16).to_le_bytes());
        h.extend_from_slice(name_bytes);
        h.extend_from_slice(&extra);
        self.out.write_all(&h)?;
        self.out.write_all(data)?;
        self.at += h.len() as u64 + packed;
        self.entries.push(Written { name: name.to_owned(), method, crc, packed, size, offset });
        Ok(())
    }

    /// Write the central directory and the end records; returns the writer.
    pub fn finish(mut self) -> io::Result<W> {
        let start = self.at;
        for e in &self.entries {
            let zip64 = e.size >= MAX_U32 || e.packed >= MAX_U32 || e.offset >= MAX_U32;
            let mut extra = Vec::new();
            if zip64 {
                extra.extend_from_slice(&0x0001u16.to_le_bytes());
                extra.extend_from_slice(&24u16.to_le_bytes());
                extra.extend_from_slice(&e.size.to_le_bytes());
                extra.extend_from_slice(&e.packed.to_le_bytes());
                extra.extend_from_slice(&e.offset.to_le_bytes());
            }
            let version = if zip64 { VERSION_ZIP64 } else { VERSION_PLAIN };
            let mut c = Vec::with_capacity(46 + e.name.len() + extra.len());
            c.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
            c.extend_from_slice(&version.to_le_bytes());
            c.extend_from_slice(&version.to_le_bytes());
            c.extend_from_slice(&UTF8_NAMES.to_le_bytes());
            c.extend_from_slice(&method_code(e.method).to_le_bytes());
            c.extend_from_slice(&DOS_TIME.to_le_bytes());
            c.extend_from_slice(&DOS_DATE.to_le_bytes());
            c.extend_from_slice(&e.crc.to_le_bytes());
            c.extend_from_slice(&(if zip64 { MAX_U32 as u32 } else { e.packed as u32 }).to_le_bytes());
            c.extend_from_slice(&(if zip64 { MAX_U32 as u32 } else { e.size as u32 }).to_le_bytes());
            c.extend_from_slice(&(e.name.len() as u16).to_le_bytes());
            c.extend_from_slice(&(extra.len() as u16).to_le_bytes());
            c.extend_from_slice(&0u16.to_le_bytes()); // comment length
            c.extend_from_slice(&0u16.to_le_bytes()); // disk number
            c.extend_from_slice(&0u16.to_le_bytes()); // internal attributes
            c.extend_from_slice(&0u32.to_le_bytes()); // external attributes
            c.extend_from_slice(&(if zip64 { MAX_U32 as u32 } else { e.offset as u32 }).to_le_bytes());
            c.extend_from_slice(e.name.as_bytes());
            c.extend_from_slice(&extra);
            self.out.write_all(&c)?;
            self.at += c.len() as u64;
        }
        let (count, size) = (self.entries.len() as u64, self.at - start);
        if count >= MAX_U16 || size >= MAX_U32 || start >= MAX_U32 {
            let eocd64 = self.at;
            let mut z = Vec::with_capacity(76);
            z.extend_from_slice(&0x0606_4b50u32.to_le_bytes());
            z.extend_from_slice(&44u64.to_le_bytes());
            z.extend_from_slice(&VERSION_ZIP64.to_le_bytes());
            z.extend_from_slice(&VERSION_ZIP64.to_le_bytes());
            z.extend_from_slice(&0u32.to_le_bytes());
            z.extend_from_slice(&0u32.to_le_bytes());
            z.extend_from_slice(&count.to_le_bytes());
            z.extend_from_slice(&count.to_le_bytes());
            z.extend_from_slice(&size.to_le_bytes());
            z.extend_from_slice(&start.to_le_bytes());
            // ZIP64 end-of-central-directory locator.
            z.extend_from_slice(&0x0706_4b50u32.to_le_bytes());
            z.extend_from_slice(&0u32.to_le_bytes());
            z.extend_from_slice(&eocd64.to_le_bytes());
            z.extend_from_slice(&1u32.to_le_bytes());
            self.out.write_all(&z)?;
        }
        let mut end = Vec::with_capacity(22);
        end.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
        end.extend_from_slice(&0u16.to_le_bytes());
        end.extend_from_slice(&0u16.to_le_bytes());
        end.extend_from_slice(&(count.min(MAX_U16) as u16).to_le_bytes());
        end.extend_from_slice(&(count.min(MAX_U16) as u16).to_le_bytes());
        end.extend_from_slice(&(size.min(MAX_U32) as u32).to_le_bytes());
        end.extend_from_slice(&(start.min(MAX_U32) as u32).to_le_bytes());
        end.extend_from_slice(&0u16.to_le_bytes());
        self.out.write_all(&end)?;
        self.out.flush()?;
        Ok(self.out)
    }
}

/// The ZIP method number of `m`.
fn method_code(m: Method) -> u16 {
    match m {
        Method::Stored => 0,
        Method::Deflate => 8,
    }
}

fn too_long(name: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, format!("the entry name is too long: {}…", name.chars().take(40).collect::<String>()))
}

// GLOSSARY
// - `Method`: how an entry is kept, `Stored` or `Deflate`.
// - `DOS_DATE`, `DOS_TIME`: the fixed timestamp written on every entry.
// - `UTF8_NAMES`: ZIP flag bit 11, names are UTF-8.
// - `VERSION_PLAIN`, `VERSION_ZIP64`: "version needed to extract" for plain and ZIP64 entries.
// - `MAX_U32`, `MAX_U16`: classic ZIP field limits; at or past them ZIP64 records are written.
// - `Written`: an entry already written, as the central directory repeats it.
// - `ZipWriter`: an archive being written; `new`, `add` (one entry), `finish` (central directory and end records).
// - `method_code`: the ZIP method number of a `Method`.
// - `too_long`: the error for an entry name longer than a ZIP field holds.
