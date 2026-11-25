//! The linked images of a reference board (`filestandard.md` §7.2) on a file system: where each
//! image's file is, relinking an image to another file, and finding missing files in folders by
//! their recorded hash or name. Content URIs are the Android side's (`EngineBoards.kt`): a source
//! whose only location is such a URI is reported as `uri`, for the caller to check.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::etcr::{Board, Source};
use crate::sha256_hex;

/// Where an image's file was found.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum LinkStatus {
    /// Its file is at `path`.
    Found { path: String },
    /// Its file is at `path`, but its bytes no longer match the recorded SHA-256.
    Changed { path: String },
    /// Only a non-file URI locates it (`content://…`): the caller checks it.
    Uri { uri: String },
    /// No file is where the board says.
    Missing,
}

/// The status of every image of `board`, saved in folder `board_dir`, in board order.
pub fn link_statuses(board: &Board, board_dir: &Path) -> Vec<(String, LinkStatus)> {
    board.images.iter().map(|i| (i.id.clone(), status(&i.source, board_dir))).collect()
}

/// The file path a `file://` URI names.
fn file_uri_path(uri: &str) -> Option<PathBuf> {
    uri.strip_prefix("file://").map(|p| PathBuf::from(percent_decode(p)))
}

/// `%XX` escapes decoded (invalid ones are kept as written).
fn percent_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(v);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Where `source` is, from `board_dir`: its relative path first, then a `file://` URI (§7.2).
pub fn status(source: &Source, board_dir: &Path) -> LinkStatus {
    let mut candidates = vec![];
    if let Some(rel) = &source.relative_path {
        candidates.push(board_dir.join(rel));
    }
    if let Some(p) = source.uri.as_deref().and_then(file_uri_path) {
        candidates.push(p);
    }
    for p in candidates {
        if p.is_file() {
            let path = p.to_string_lossy().into_owned();
            let changed = source.sha256.as_deref().is_some_and(|h| std::fs::read(&p).is_ok_and(|b| !h.eq_ignore_ascii_case(&sha256_hex(&b))));
            return if changed { LinkStatus::Changed { path } } else { LinkStatus::Found { path } };
        }
    }
    match &source.uri {
        Some(uri) if !uri.starts_with("file://") => LinkStatus::Uri { uri: uri.clone() },
        _ => LinkStatus::Missing,
    }
}

/// `path` relative to `base` when it is inside it, with `/` separators.
fn relative_to(path: &Path, base: &Path) -> Option<String> {
    let rel = path.strip_prefix(base).ok()?;
    Some(rel.components().map(|c| c.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/"))
}

/// Link image `id` of `board` (saved in `board_dir`) to the file at `file`: a relative path when
/// the file is inside the board's folder, a `file://` URI always, its name, and its SHA-256. The
/// image's transform and crop are kept (§7.2: reloading never resets them). → whether the image
/// exists and the file could be read.
pub fn relink(board: &mut Board, id: &str, file: &Path, board_dir: &Path) -> Result<(), String> {
    let bytes = std::fs::read(file).map_err(|e| format!("{}: {e}", file.display()))?;
    let image = board.image_mut(id).ok_or_else(|| format!("no image `{id}` on the board"))?;
    let abs = std::fs::canonicalize(file).unwrap_or_else(|_| file.to_path_buf());
    let dir = std::fs::canonicalize(board_dir).unwrap_or_else(|_| board_dir.to_path_buf());
    let s = &mut image.source;
    s.kind = "linked-file".into();
    s.relative_path = relative_to(&abs, &dir);
    s.uri = Some(format!("file://{}", abs.to_string_lossy()));
    s.original_filename = file.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    s.sha256 = Some(sha256_hex(&bytes));
    Ok(())
}

/// Most files [`find_missing`] looks at, and how deep it goes into folders.
const MAX_FILES: usize = 20_000;
const MAX_DEPTH: usize = 6;

/// For each missing image of `board` (saved in `board_dir`), a file in `folders` (and their
/// subfolders) that is it: one with the recorded SHA-256 first, else one with its original file
/// name. → (image id, path) for each image found; nothing is changed (see [`relink`]).
pub fn find_missing(board: &Board, board_dir: &Path, folders: &[PathBuf]) -> Vec<(String, String)> {
    let missing: Vec<&crate::etcr::BoardImage> =
        board.images.iter().filter(|i| matches!(status(&i.source, board_dir), LinkStatus::Missing)).collect();
    if missing.is_empty() {
        return vec![];
    }
    let mut files = vec![];
    for f in folders {
        collect(f, 0, &mut files);
    }
    let mut out = vec![];
    for image in missing {
        let by_hash = image.source.sha256.as_deref().and_then(|h| {
            files.iter().find(|p| {
                std::fs::metadata(p).is_ok_and(|m| m.len() < (1 << 30)) && std::fs::read(p).is_ok_and(|b| sha256_hex(&b).eq_ignore_ascii_case(h))
            })
        });
        let name = &image.source.original_filename;
        let by_name = || (!name.is_empty()).then(|| files.iter().find(|p| p.file_name().is_some_and(|n| n.to_string_lossy() == *name))).flatten();
        if let Some(p) = by_hash.or_else(by_name) {
            out.push((image.id.clone(), p.to_string_lossy().into_owned()));
        }
    }
    out
}

/// The files under `dir`, at most [`MAX_DEPTH`] folders deep and [`MAX_FILES`] in all.
fn collect(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth > MAX_DEPTH || out.len() >= MAX_FILES {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = entries.flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        if out.len() >= MAX_FILES {
            return;
        }
        if p.is_dir() {
            collect(&p, depth + 1, out);
        } else if p.is_file() {
            out.push(p);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::etcr::{BoardImage, Transform};
    use serde_json::Map;

    fn folder(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("etcfile-boardlinks-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(d.join("assets")).unwrap();
        d
    }

    fn image(id: &str, rel: &str) -> BoardImage {
        BoardImage {
            id: id.into(),
            source: Source { kind: "linked-file".into(), relative_path: Some(rel.into()), original_filename: rel.rsplit('/').next().unwrap().into(), ..Source::default() },
            transform: Transform { x: 0.0, y: 0.0, width: 10.0, height: 10.0, rotation_degrees: 0.0, pivot: [0.5, 0.5] },
            crop: None,
            visible: true,
            extra: Map::new(),
        }
    }

    #[test]
    fn statuses_relink_and_find_missing() {
        let d = folder("all");
        std::fs::write(d.join("assets/a.png"), b"first").unwrap();
        let mut b = Board::new("B");
        b.images.push(image("a", "assets/a.png"));
        b.images.push(image("b", "assets/b.png"));
        let st = link_statuses(&b, &d);
        assert!(matches!(&st[0].1, LinkStatus::Found { .. }));
        assert_eq!(st[1].1, LinkStatus::Missing);
        // The missing one turns up elsewhere under its name.
        let elsewhere = d.join("moved");
        std::fs::create_dir_all(&elsewhere).unwrap();
        std::fs::write(elsewhere.join("b.png"), b"second").unwrap();
        let found = find_missing(&b, &d, &[elsewhere.clone()]);
        assert_eq!(found, vec![("b".to_string(), elsewhere.join("b.png").to_string_lossy().into_owned())]);
        b.images[1].transform.x = 42.0;
        relink(&mut b, "b", &elsewhere.join("b.png"), &d).unwrap();
        assert_eq!(b.images[1].source.relative_path.as_deref(), Some("moved/b.png"));
        assert_eq!(b.images[1].transform.x, 42.0, "the placement is kept");
        assert!(matches!(status(&b.images[1].source, &d), LinkStatus::Found { .. }));
        // A changed file is told apart by its hash.
        std::fs::write(elsewhere.join("b.png"), b"edited").unwrap();
        assert!(matches!(status(&b.images[1].source, &d), LinkStatus::Changed { .. }));
        let mut u = image("c", "gone.png");
        u.source.relative_path = None;
        u.source.uri = Some("content://provider/document/9".into());
        assert_eq!(status(&u.source, &d), LinkStatus::Uri { uri: "content://provider/document/9".into() });
        assert_eq!(percent_decode("a%20b%zz"), "a b%zz");
        let _ = std::fs::remove_dir_all(&d);
    }
}

// GLOSSARY
// - `LinkStatus`: where an image's file was found (`Found`, `Changed`, `Uri`, `Missing`).
// - `link_statuses`: the status of every image of a board.
// - `file_uri_path`: the path a `file://` URI names.
// - `percent_decode`: `%XX` escapes decoded.
// - `status`: where one source's file is (relative path, then `file://` URI).
// - `relative_to`: a path relative to a folder it is inside.
// - `relink`: point an image at another file, keeping its placement.
// - `MAX_FILES`, `MAX_DEPTH`: limits of the search for missing files.
// - `find_missing`: files in folders that are the board's missing images (by SHA-256, else name).
// - `collect`: the files under a folder, within the limits.
