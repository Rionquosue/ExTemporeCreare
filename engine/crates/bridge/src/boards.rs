//! Reference boards (`.etcr`, `filestandard.md` §7) for the UI: new, read and write boards, and
//! their linked images' status, relinking and finding missing files. Boards are plain data the UI
//! edits as JSON (`board.json`); no engine session is involved. Files cross as paths: on Android
//! `EngineBoards.kt` copies content URIs to and from temporary files.

use std::path::{Path, PathBuf};

use etcfile::boardlinks::{find_missing, link_statuses, relink};
use etcfile::etcr::{Board, EMBEDDED_KIND, board_assets, read_board, write_board};
use serde_json::{Value, json};

/// `{"ok": …}` or `{"error": {"kind", "message"}}` JSON text.
fn answer(r: Result<Value, String>) -> String {
    match r {
        Ok(v) => json!({ "ok": v }).to_string(),
        Err(message) => json!({ "error": { "kind": "board", "message": message } }).to_string(),
    }
}

/// Run `f`, catching panics.
fn guarded(f: impl FnOnce() -> Result<Value, String>) -> String {
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
        Ok(r) => answer(r),
        Err(_) => json!({ "error": { "kind": "panic", "message": "the board call stopped" } }).to_string(),
    }
}

fn parse(board: &str) -> Result<Board, String> {
    let b: Board = serde_json::from_str(board).map_err(|e| format!("board: {e}"))?;
    match b.problem() {
        Some(p) => Err(p),
        None => Ok(b),
    }
}

fn links_json(board: &Board, dir: &Path) -> Value {
    Value::Array(
        link_statuses(board, dir)
            .into_iter()
            .map(|(id, st)| {
                let mut v = serde_json::to_value(&st).unwrap_or(Value::Null);
                v["id"] = Value::String(id);
                v
            })
            .collect(),
    )
}

/// A new empty board titled `title` → `{"ok": board}`.
pub fn new_board(title: &str) -> String {
    guarded(|| serde_json::to_value(Board::new(title)).map_err(|e| e.to_string()))
}

/// The folder board `b`'s embedded images are kept in, under `embedded_dir`: one per board id.
fn embedded_folder(embedded_dir: &str, b: &Board) -> PathBuf {
    let id: String = b.board_id.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-').collect();
    Path::new(embedded_dir).join(if id.is_empty() { "board".to_string() } else { id })
}

/// The local file of embedded image `asset_path` in `folder`.
fn embedded_file(folder: &Path, asset_path: &str) -> PathBuf {
    folder.join(asset_path.rsplit('/').next().unwrap_or(asset_path))
}

/// Read the `.etcr` file at `path`; links are checked from `board_dir` (the folder of the board's
/// own file, which a temporary copy is not in). Its embedded images are written to
/// `embedded_dir/<boardId>/` (when `embedded_dir` is not empty) and found there →
/// `{"ok": {"board", "links": [{"id", "status", …}]}}`.
pub fn read(path: &str, board_dir: &str, embedded_dir: &str) -> String {
    guarded(|| {
        let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
        let board = read_board(&bytes).map_err(|e| e.to_string())?;
        let mut links = links_json(&board, Path::new(board_dir));
        if !embedded_dir.is_empty() {
            let folder = embedded_folder(embedded_dir, &board);
            std::fs::create_dir_all(&folder).map_err(|e| format!("{}: {e}", folder.display()))?;
            for (asset, data) in board_assets(&bytes, &board).map_err(|e| e.to_string())? {
                let file = embedded_file(&folder, &asset);
                std::fs::write(&file, data).map_err(|e| format!("{}: {e}", file.display()))?;
            }
            for (i, image) in board.images.iter().enumerate() {
                let Some(asset) = image.source.asset_path.as_deref().filter(|_| image.source.kind == EMBEDDED_KIND) else { continue };
                let file = embedded_file(&folder, asset);
                if let Some(row) = links.get_mut(i) {
                    *row = json!({ "id": image.id, "status": if file.is_file() { "found" } else { "missing" }, "path": file.to_string_lossy() });
                }
            }
        }
        Ok(json!({ "board": board, "links": links }))
    })
}

/// Write `board` (JSON) as an `.etcr` file at `path`, atomically, with the PNG at `thumbnail`
/// when that is not empty, and its embedded images' bytes from `embedded_dir/<boardId>/` →
/// `{"ok": true}`.
pub fn write(path: &str, board: &str, thumbnail: &str, embedded_dir: &str) -> String {
    guarded(|| {
        let b = parse(board)?;
        let thumb = if thumbnail.is_empty() { None } else { Some(std::fs::read(thumbnail).map_err(|e| format!("{thumbnail}: {e}"))?) };
        let folder = embedded_folder(embedded_dir, &b);
        let mut assets: Vec<(String, Vec<u8>)> = vec![];
        for image in b.images.iter().filter(|i| i.source.kind == EMBEDDED_KIND) {
            let asset = image.source.asset_path.clone().unwrap_or_default();
            if assets.iter().any(|(p, _)| *p == asset) {
                continue;
            }
            let file = embedded_file(&folder, &asset);
            let data = std::fs::read(&file).map_err(|e| format!("embedded image {}: {e}", file.display()))?;
            assets.push((asset, data));
        }
        let bytes = write_board(&b, thumb.as_deref(), &assets).map_err(|e| e.to_string())?;
        etcfile::write_atomic(Path::new(path), &bytes).map_err(|e| format!("{path}: {e}"))?;
        Ok(Value::Bool(true))
    })
}

/// Where each image of `board` (JSON) is, from `board_dir` → `{"ok": [{"id", "status", …}]}`.
pub fn links(board: &str, board_dir: &str) -> String {
    guarded(|| Ok(links_json(&parse(board)?, Path::new(board_dir))))
}

/// `board` (JSON) with image `id` linked to the file at `file` → `{"ok": board}`.
pub fn relink_image(board: &str, id: &str, file: &str, board_dir: &str) -> String {
    guarded(|| {
        let mut b = parse(board)?;
        relink(&mut b, id, Path::new(file), Path::new(board_dir))?;
        serde_json::to_value(b).map_err(|e| e.to_string())
    })
}

/// Files in `folders` (a JSON array of paths) that are `board`'s missing images →
/// `{"ok": [{"id", "path"}]}`.
pub fn find(board: &str, board_dir: &str, folders: &str) -> String {
    guarded(|| {
        let b = parse(board)?;
        let folders: Vec<PathBuf> = serde_json::from_str::<Vec<String>>(folders).map_err(|e| format!("folders: {e}"))?.into_iter().map(PathBuf::from).collect();
        let found = find_missing(&b, Path::new(board_dir), &folders);
        Ok(Value::Array(found.into_iter().map(|(id, path)| json!({ "id": id, "path": path })).collect()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok(s: &str) -> Value {
        let v: Value = serde_json::from_str(s).unwrap();
        v.get("ok").cloned().unwrap_or_else(|| panic!("{s}"))
    }

    #[test]
    fn a_board_is_made_written_read_and_relinked() {
        let d = std::env::temp_dir().join(format!("etc-bridge-boards-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("pose.png"), b"pixels").unwrap();
        let mut board = ok(&new_board("Refs"));
        board["images"] = json!([{
            "id": "r1",
            "source": {"kind": "linked-file", "relativePath": "gone.png", "originalFilename": "pose.png"},
            "transform": {"x": 0, "y": 0, "width": 100, "height": 80}
        }]);
        let text = board.to_string();
        let file = d.join("refs.etcr");
        assert_eq!(ok(&write(&file.to_string_lossy(), &text, "", "")), true);
        let dir = d.to_string_lossy();
        let read_back = ok(&read(&file.to_string_lossy(), &dir, ""));
        assert_eq!(read_back["links"][0]["status"], "missing");
        let found = ok(&find(&text, &dir, &json!([dir]).to_string()));
        assert_eq!(found[0]["id"], "r1");
        let relinked = ok(&relink_image(&text, "r1", found[0]["path"].as_str().unwrap(), &dir));
        assert_eq!(relinked["images"][0]["source"]["relativePath"], "pose.png");
        assert_eq!(ok(&links(&relinked.to_string(), &dir))[0]["status"], "found");
        let bad: Value = serde_json::from_str(&write(&file.to_string_lossy(), "{}", "", "")).unwrap();
        assert!(bad.get("error").is_some());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn an_embedded_image_travels_inside_the_board() {
        let d = std::env::temp_dir().join(format!("etc-bridge-embedded-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let (here, there) = (d.join("here"), d.join("there"));
        let mut board = ok(&new_board("Refs"));
        let id = board["boardId"].as_str().unwrap().to_string();
        std::fs::create_dir_all(here.join(&id)).unwrap();
        std::fs::write(here.join(&id).join("pose.png"), b"pixels").unwrap();
        board["images"] = json!([{
            "id": "r1",
            "source": {"kind": "embedded", "assetPath": "References/pose.png", "originalFilename": "pose.png"},
            "transform": {"x": 0, "y": 0, "width": 100, "height": 80}
        }]);
        let file = d.join("refs.etcr");
        assert_eq!(ok(&write(&file.to_string_lossy(), &board.to_string(), "", &here.to_string_lossy())), true);
        let read_back = ok(&read(&file.to_string_lossy(), "", &there.to_string_lossy()));
        assert_eq!(read_back["links"][0]["status"], "found");
        let path = read_back["links"][0]["path"].as_str().unwrap();
        assert_eq!(std::fs::read(path).unwrap(), b"pixels");
        assert!(path.starts_with(&*there.to_string_lossy()));
        let _ = std::fs::remove_dir_all(&d);
    }
}


// GLOSSARY
// - `answer`: a result as `{"ok"}` / `{"error"}` JSON text.
// - `guarded`: run a board call with panics caught.
// - `parse`: a board from JSON, checked against §7.
// - `links_json`: every image's link status as JSON.
// - `new_board`: a new empty board.
// - `embedded_folder`, `embedded_file`: where a board's embedded images are kept locally.
// - `read`: read an `.etcr` file, check its links and put its embedded images in their folder.
// - `write`: write a board as an `.etcr` file atomically, its embedded images taken from their folder.
// - `links`: the link status of every image of a board.
// - `relink_image`: link an image to another file.
// - `find`: files in folders that are a board's missing images.
