//! Opening and saving documents on Android, where files are content URIs the Kotlin side reaches
//! (`components/engine/EngineFiles.kt`). The engine never opens a URI itself:
//!
//! - Open: Kotlin copies the URI's bytes into a temporary file; [`open`] reads it and opens it with
//!   the URI as the document's path.
//! - Save: [`save`] plans the save with the URI as the path and writes every file into a temporary
//!   file in the folder Kotlin gives, atomically (`etcfile::write_atomic`); the save is kept pending.
//!   Kotlin checks each `.etc` with [`validate`], copies the temporary files into their URIs, then
//!   calls [`save_finish`]: on success the document is marked saved at its URI, otherwise nothing
//!   about the document changes and the original files are untouched.

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use vectorcraft_engine::cmd::fileio::{SaveJob, SaveMode, open_bytes, save_job, save_plan};

use crate::calls::guarded;

/// A save written to temporary files, waiting for Kotlin to copy them into place.
pub struct PendingSave {
    job: SaveJob,
    uid: u64,
}

/// The save mode named `mode`: `save`, `saveAs`, `copy` or `template`.
fn mode_of(mode: &str) -> Result<SaveMode, String> {
    match mode {
        "save" => Ok(SaveMode::Save),
        "saveAs" => Ok(SaveMode::SaveAs),
        "copy" => Ok(SaveMode::Copy),
        "template" => Ok(SaveMode::Template),
        other => Err(format!("unknown save mode `{other}` (save, saveAs, copy or template)")),
    }
}

/// Open the file copied to `temp` (its display name `name`, for its title and type) as a document
/// whose path is `uri`.
pub fn open(handle: i64, temp: &str, name: &str, uri: &str) -> String {
    guarded(handle, |b| {
        let bytes = std::fs::read(temp).map_err(|e| format!("can't read the copied file: {e}"))?;
        let path = (!uri.is_empty()).then(|| uri.to_string());
        open_bytes(&mut b.session, name, &bytes, path).map_err(|e| e.to_string())
    })
}

/// Plan and write a save of the active document: `mode` as [`mode_of`] reads it, `params` the save
/// params (JSON: `path` is the target URI, `format`, options…), each file written to a temporary
/// file in `temp_dir`. → `{"ok": {"files": [{"target", "temp"}], …the save's reply}}`; the save
/// waits for [`save_finish`]. A save already pending is dropped.
pub fn save(handle: i64, mode: &str, params: &str, temp_dir: &str) -> String {
    guarded(handle, |b| {
        b.pending = None;
        let mode = mode_of(mode)?;
        let p: Value = if params.trim().is_empty() { json!({}) } else { serde_json::from_str(params).map_err(|e| format!("params: {e}"))? };
        let plan = save_plan(&b.session, mode, &p).map_err(|e| e.to_string())?;
        let uid = b.session.doc().map_err(|e| e.to_string())?.uid;
        let job = save_job(&mut b.session, plan).map_err(|e| e.to_string())?;
        let mut files = vec![];
        let dir = PathBuf::from(temp_dir);
        let mut reply = job
            .write(|target, bytes| {
                let temp = dir.join(format!("save-{uid}-{}.tmp", files.len()));
                etcfile::write_atomic(&temp, bytes).map_err(|e| vectorcraft_engine::EngineError::Other(format!("{}: {e}", temp.display())))?;
                files.push(json!({ "target": target, "temp": temp.to_string_lossy() }));
                Ok(())
            })
            .map_err(|e| e.to_string())?;
        if let Some(o) = reply.as_object_mut() {
            o.insert("files".into(), Value::Array(files));
        }
        b.pending = Some(PendingSave { job, uid });
        Ok(reply)
    })
}

/// End the pending save: `ok` when every file reached its URI (the document is then saved there),
/// else it is dropped and the document is left as it was.
pub fn save_finish(handle: i64, ok: bool) -> String {
    guarded(handle, |b| {
        let Some(PendingSave { job, uid }) = b.pending.take() else { return Err("no save is pending".into()) };
        if ok {
            job.complete(&mut b.session, uid);
        }
        Ok(json!({ "saved": ok }))
    })
}

/// Whether the file at `path` is a valid `.etc` project → `{"ok": true}` or the reason it isn't.
pub fn validate(path: &str) -> String {
    let checked = std::fs::read(Path::new(path)).map_err(|e| e.to_string()).and_then(|b| etcfile::validate(&b).map_err(|e| e.to_string()));
    match checked {
        Ok(()) => json!({ "ok": true }).to_string(),
        Err(message) => json!({ "error": { "kind": "invalid", "message": message } }).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{calls, sessions};

    fn parse(s: &str) -> Value {
        serde_json::from_str(s).unwrap()
    }

    #[test]
    fn save_to_a_uri_through_temporary_files_then_open_it_again() {
        let dir = std::env::temp_dir().join(format!("etc-bridge-files-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let h = sessions::create();
        calls::execute(h, "file.new", r#"{"width": 120, "height": 80}"#);
        calls::execute(h, "shape.rectangle", r#"{"x": 10, "y": 10, "width": 30, "height": 20}"#);
        let uri = "content://com.example.documents/document/42";
        let r = parse(&save(h, "saveAs", &json!({"path": uri, "format": "etc"}).to_string(), &dir.to_string_lossy()));
        let files = r["ok"]["files"].as_array().unwrap_or_else(|| panic!("{r}")).clone();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0]["target"], uri);
        let temp = files[0]["temp"].as_str().unwrap().to_string();
        assert_eq!(parse(&validate(&temp))["ok"], true);
        // Not saved there until Kotlin says the copy reached the URI.
        let path = |h| sessions::with(h, |b| b.session.doc().ok().and_then(|st| st.path.clone())).flatten();
        assert_eq!(path(h), None);
        assert_eq!(parse(&save_finish(h, true))["ok"]["saved"], true);
        assert_eq!(path(h).as_deref(), Some(uri));
        assert_eq!(parse(&save_finish(h, true))["error"]["message"], "no save is pending");
        // Open the copy again with the URI as its path.
        let h2 = sessions::create();
        let opened = parse(&open(h2, &temp, "Poster.etc", uri));
        assert!(opened.get("ok").is_some(), "{opened}");
        assert_eq!(path(h2).as_deref(), Some(uri));
        assert!(parse(&validate(&dir.join("missing.etc").to_string_lossy())).get("error").is_some());
        sessions::destroy(h);
        sessions::destroy(h2);
        let _ = std::fs::remove_dir_all(&dir);
    }
}

// GLOSSARY
// - `PendingSave`: a save written to temporary files, waiting for the copy into its URIs.
// - `mode_of`: the engine save mode from its name.
// - `open`: open a file copied to a temporary file, with its URI as the document's path.
// - `save`: plan a save and write its files into temporary files; the save stays pending.
// - `save_finish`: complete or drop the pending save.
// - `validate`: whether a file is a valid `.etc` project.
