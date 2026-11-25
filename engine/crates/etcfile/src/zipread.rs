//! Reads ZIP archives for `.etc` and `.etcr` files and checks them as `filestandard.md` §3 and
//! §11 require: safe relative UTF-8 names, no duplicate entries, STORED or DEFLATE only, and every
//! size checked against limits before anything is unpacked (decompression bombs are refused).

use std::io::Read;

use flate2::read::DeflateDecoder;

/// What is wrong with an archive.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ZipError {
    #[error("not a ZIP archive")]
    NotZip,
    #[error("the archive is damaged: {0}")]
    Damaged(String),
    #[error("the archive has an unsafe entry name: {0}")]
    UnsafeName(String),
    #[error("the archive has two entries named {0}")]
    Duplicate(String),
    #[error("the entry {0} uses a compression method this app doesn't read")]
    Method(String),
    #[error("the entry {0} unpacks to more than this app reads")]
    TooBig(String),
}

/// Most entries an archive may list.
pub const MAX_ENTRIES: usize = 100_000;

/// One entry of the central directory.
#[derive(Clone, Debug)]
pub struct Entry {
    pub name: String,
    /// ZIP method: 0 stored, 8 deflate.
    pub method: u16,
    pub crc: u32,
    pub packed: u64,
    pub size: u64,
    /// Where its local header starts.
    offset: u64,
}

/// An archive's entries over its bytes.
pub struct ZipArchive<'a> {
    bytes: &'a [u8],
    entries: Vec<Entry>,
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    b.get(at..at + 2).map(|s| u16::from_le_bytes([s[0], s[1]]))
}

fn u32_at(b: &[u8], at: usize) -> Option<u32> {
    b.get(at..at + 4).map(|s| u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

fn u64_at(b: &[u8], at: usize) -> Option<u64> {
    b.get(at..at + 8).map(|s| u64::from_le_bytes(s.try_into().expect("8 bytes")))
}

fn damaged(why: &str) -> ZipError {
    ZipError::Damaged(why.to_owned())
}

/// Whether `name` is a safe relative archive path: not empty, no leading `/`, no `\`, no drive
/// letter, no `.` or `..` part and no empty part (a trailing `/` marks a folder entry).
pub fn safe_name(name: &str) -> bool {
    let body = name.strip_suffix('/').unwrap_or(name);
    !body.is_empty()
        && !name.starts_with('/')
        && !name.contains('\\')
        && !name.contains('\0')
        && !body.split('/').next().is_some_and(|first| first.len() == 2 && first.ends_with(':'))
        && body.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
}

impl<'a> ZipArchive<'a> {
    /// Read the central directory of `bytes`, checking every entry's name.
    pub fn new(bytes: &'a [u8]) -> Result<Self, ZipError> {
        if !bytes.starts_with(b"PK\x03\x04") {
            return Err(ZipError::NotZip);
        }
        // The end record is in the last 22 + 65535 (comment) bytes.
        let from = bytes.len().saturating_sub(22 + 0xffff);
        let eocd = (from..bytes.len().saturating_sub(21)).rev().find(|&i| u32_at(bytes, i) == Some(0x0605_4b50)).ok_or_else(|| damaged("no end record"))?;
        let mut count = u64::from(u16_at(bytes, eocd + 10).ok_or_else(|| damaged("end record"))?);
        let mut cd_size = u64::from(u32_at(bytes, eocd + 12).ok_or_else(|| damaged("end record"))?);
        let mut cd_start = u64::from(u32_at(bytes, eocd + 16).ok_or_else(|| damaged("end record"))?);
        // ZIP64: the locator sits just before the end record.
        if eocd >= 20 && u32_at(bytes, eocd - 20) == Some(0x0706_4b50) {
            let at = usize::try_from(u64_at(bytes, eocd - 12).ok_or_else(|| damaged("ZIP64 locator"))?).map_err(|_| damaged("ZIP64 locator"))?;
            if u32_at(bytes, at) != Some(0x0606_4b50) {
                return Err(damaged("ZIP64 end record"));
            }
            count = u64_at(bytes, at + 32).ok_or_else(|| damaged("ZIP64 end record"))?;
            cd_size = u64_at(bytes, at + 40).ok_or_else(|| damaged("ZIP64 end record"))?;
            cd_start = u64_at(bytes, at + 48).ok_or_else(|| damaged("ZIP64 end record"))?;
        }
        if count > MAX_ENTRIES as u64 {
            return Err(damaged("too many entries"));
        }
        let start = usize::try_from(cd_start).map_err(|_| damaged("central directory"))?;
        let end = start.checked_add(usize::try_from(cd_size).map_err(|_| damaged("central directory"))?).ok_or_else(|| damaged("central directory"))?;
        let cd = bytes.get(start..end).ok_or_else(|| damaged("central directory out of range"))?;
        let mut entries: Vec<Entry> = Vec::with_capacity(count as usize);
        let mut at = 0usize;
        for _ in 0..count {
            if u32_at(cd, at) != Some(0x0201_4b50) {
                return Err(damaged("central directory entry"));
            }
            let field = |o: usize| u16_at(cd, at + o).ok_or_else(|| damaged("central directory entry"));
            let flags = field(8)?;
            let method = field(10)?;
            let crc = u32_at(cd, at + 16).ok_or_else(|| damaged("central directory entry"))?;
            let mut packed = u64::from(u32_at(cd, at + 20).ok_or_else(|| damaged("central directory entry"))?);
            let mut size = u64::from(u32_at(cd, at + 24).ok_or_else(|| damaged("central directory entry"))?);
            let (name_len, extra_len, comment_len) = (field(28)? as usize, field(30)? as usize, field(32)? as usize);
            let mut offset = u64::from(u32_at(cd, at + 42).ok_or_else(|| damaged("central directory entry"))?);
            let name_bytes = cd.get(at + 46..at + 46 + name_len).ok_or_else(|| damaged("entry name"))?;
            if flags & 1 != 0 {
                return Err(ZipError::Damaged("encrypted entries aren't read".into()));
            }
            let name = String::from_utf8(name_bytes.to_vec()).map_err(|_| ZipError::UnsafeName(String::from_utf8_lossy(name_bytes).into_owned()))?;
            // ZIP64 extra field: the fields at their maximum, in this order.
            let extra = cd.get(at + 46 + name_len..at + 46 + name_len + extra_len).ok_or_else(|| damaged("entry extra field"))?;
            let mut e = 0;
            while e + 4 <= extra.len() {
                let (id, len) = (u16_at(extra, e).unwrap_or(0), u16_at(extra, e + 2).unwrap_or(0) as usize);
                if id == 0x0001 {
                    let mut f = e + 4;
                    for value in [&mut size, &mut packed, &mut offset] {
                        if *value == 0xffff_ffff {
                            *value = u64_at(extra, f).ok_or_else(|| damaged("ZIP64 extra field"))?;
                            f += 8;
                        }
                    }
                }
                e += 4 + len;
            }
            if !safe_name(&name) {
                return Err(ZipError::UnsafeName(name));
            }
            if entries.iter().any(|x| x.name == name) {
                return Err(ZipError::Duplicate(name));
            }
            entries.push(Entry { name, method, crc, packed, size, offset });
            at += 46 + name_len + extra_len + comment_len;
        }
        Ok(Self { bytes, entries })
    }

    /// The entries in archive order.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// The entry named `name`.
    pub fn entry(&self, name: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.name == name)
    }

    /// The packed bytes of `e` as they are in the archive.
    fn raw(&self, e: &Entry) -> Result<&'a [u8], ZipError> {
        let at = usize::try_from(e.offset).map_err(|_| damaged("entry offset"))?;
        if u32_at(self.bytes, at) != Some(0x0403_4b50) {
            return Err(ZipError::Damaged(format!("no local header for {}", e.name)));
        }
        let name_len = u16_at(self.bytes, at + 26).ok_or_else(|| damaged("local header"))? as usize;
        let extra_len = u16_at(self.bytes, at + 28).ok_or_else(|| damaged("local header"))? as usize;
        let start = at + 30 + name_len + extra_len;
        let end = start.checked_add(usize::try_from(e.packed).map_err(|_| damaged("entry size"))?).ok_or_else(|| damaged("entry size"))?;
        self.bytes.get(start..end).ok_or_else(|| ZipError::Damaged(format!("{} runs past the end of the archive", e.name)))
    }

    /// The unpacked bytes of the entry named `name`, refused when its size passes `max`.
    pub fn read(&self, name: &str, max: u64) -> Result<Option<Vec<u8>>, ZipError> {
        let Some(e) = self.entry(name) else { return Ok(None) };
        self.read_entry(e, max).map(Some)
    }

    /// The unpacked bytes of `e`, refused when its stated or actual size passes `max`; the CRC is
    /// checked.
    pub fn read_entry(&self, e: &Entry, max: u64) -> Result<Vec<u8>, ZipError> {
        if e.size > max {
            return Err(ZipError::TooBig(e.name.clone()));
        }
        let raw = self.raw(e)?;
        let out = match e.method {
            0 => raw.to_vec(),
            8 => {
                let mut out = Vec::with_capacity(e.size as usize);
                // One byte more than stated shows an entry that lies about its size.
                DeflateDecoder::new(raw).take(e.size + 1).read_to_end(&mut out).map_err(|err| ZipError::Damaged(format!("{}: {err}", e.name)))?;
                out
            }
            _ => return Err(ZipError::Method(e.name.clone())),
        };
        if out.len() as u64 != e.size {
            return Err(ZipError::Damaged(format!("{} has the wrong size", e.name)));
        }
        if crc32fast::hash(&out) != e.crc {
            return Err(ZipError::Damaged(format!("{} fails its checksum", e.name)));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zipwrite::{Method, ZipWriter};

    fn archive(files: &[(&str, &[u8], Method)]) -> Vec<u8> {
        let mut w = ZipWriter::new(Vec::new());
        for (n, b, m) in files {
            w.add(n, b, *m).unwrap();
        }
        w.finish().unwrap()
    }

    #[test]
    fn writes_and_reads_back_stored_and_deflated_entries() {
        let text = "x".repeat(10_000);
        let zip = archive(&[("mimetype", b"application/x-ex-tempore-creare", Method::Stored), ("project.json", text.as_bytes(), Method::Deflate)]);
        let a = ZipArchive::new(&zip).unwrap();
        assert_eq!(a.entries()[0].name, "mimetype");
        assert_eq!(a.entries()[0].method, 0);
        assert_eq!(a.read("mimetype", 100).unwrap().unwrap(), b"application/x-ex-tempore-creare");
        assert_eq!(a.read("project.json", 1 << 20).unwrap().unwrap(), text.as_bytes());
        assert!(a.entries()[1].packed < 1000, "deflated");
        assert_eq!(a.read("project.json", 100), Err(ZipError::TooBig("project.json".into())));
        assert_eq!(a.read("missing", 100), Ok(None));
    }

    #[test]
    fn refuses_unsafe_names_duplicates_and_damage() {
        for bad in ["../x", "/abs", "a/../b", "a\\b", "C:/x", "", "a//b", "./a"] {
            assert!(!safe_name(bad), "{bad}");
        }
        assert!(safe_name("data/raster/a.png") && safe_name("Thumbnails/"));
        let mut w = ZipWriter::new(Vec::new());
        w.add("a", b"1", Method::Stored).unwrap();
        assert!(w.add("a", b"2", Method::Stored).is_err());
        let mut zip = archive(&[("a.txt", b"hello", Method::Stored)]);
        let at = zip.windows(5).position(|w| w == b"hello").unwrap();
        zip[at] = b'j';
        let a = ZipArchive::new(&zip).unwrap();
        assert!(matches!(a.read("a.txt", 100), Err(ZipError::Damaged(_))));
        assert_eq!(ZipArchive::new(b"not a zip").err(), Some(ZipError::NotZip));
    }
}

// GLOSSARY
// - `ZipError`: what is wrong with an archive (`NotZip`, `Damaged`, `UnsafeName`, `Duplicate`, `Method`, `TooBig`).
// - `MAX_ENTRIES`: most entries an archive may list.
// - `Entry`: one central-directory entry (name, method, CRC, packed and unpacked size, local header offset).
// - `ZipArchive`: an archive's checked entries over its bytes; `new`, `entries`, `entry`, `read`, `read_entry`.
// - `u16_at`, `u32_at`, `u64_at`: little-endian reads that return `None` past the end.
// - `damaged`: a `ZipError::Damaged` with a reason.
// - `safe_name`: whether an entry name is a safe relative archive path.
