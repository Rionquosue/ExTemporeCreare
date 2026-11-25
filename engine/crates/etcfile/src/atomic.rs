//! Atomic saving: a file is written to a temporary file in the same folder, flushed to disk, then
//! renamed over the target, so an interrupted or failed save never damages the existing file
//! (`filestandard.md` §11: "write to a temporary file, validate, then replace").
//!
//! On Android the Kotlin side reaches content URIs itself; these functions are for paths the
//! engine writes directly (app storage, recovery checkpoints, preset and library files).

use std::fs::{File, OpenOptions};
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// Counts temporary files made by this process, so two saves never pick the same name.
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Whether a write error means the location can't be written at all (read-only storage, or no
/// permission), as opposed to a failure of this one write.
pub fn read_only_location(error: &io::Error) -> bool {
    matches!(error.kind(), io::ErrorKind::ReadOnlyFilesystem | io::ErrorKind::PermissionDenied)
}

/// Write `bytes` to `path` atomically.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    write_atomic_with(path, |f| f.write_all(bytes))
}

/// Write `path` atomically with `fill`, which writes the content into the temporary file. When
/// `fill` or the flush fails, the temporary file is removed and `path` is left as it was.
pub fn write_atomic_with(path: &Path, fill: impl FnOnce(&mut File) -> io::Result<()>) -> io::Result<()> {
    let target = resolve_link(path)?;
    let tmp = temp_path(&target)?;
    let result = (|| {
        let mut f = OpenOptions::new().write(true).create_new(true).open(&tmp)?;
        fill(&mut f)?;
        f.flush()?;
        sync_file(&f)?;
        drop(f);
        keep_permissions(&target, &tmp)?;
        std::fs::rename(&tmp, &target)?;
        sync_parent(&target);
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// Create `path` with `fill`, never replacing a file: the content is written to a temporary file
/// and then linked into place, which fails with `AlreadyExists` when `path` exists. Where the
/// folder can't hard-link, the finished temporary file is copied into a file created with
/// `create_new`; a failed copy removes that partial file again.
pub fn write_new_with(path: &Path, fill: impl FnOnce(&mut File) -> io::Result<()>) -> io::Result<()> {
    let tmp = temp_path(path)?;
    let staged = (|| {
        let mut f = OpenOptions::new().write(true).create_new(true).open(&tmp)?;
        fill(&mut f)?;
        f.flush()?;
        sync_file(&f)
    })();
    if let Err(e) = staged {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    let placed = match std::fs::hard_link(&tmp, path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => Err(e),
        // No hard links in this folder: copy the finished content into a new file.
        Err(_) => copy_new(&tmp, path),
    };
    let _ = std::fs::remove_file(&tmp);
    if placed.is_ok() {
        sync_parent(path);
    }
    placed
}

/// Copy `from` into `path`, which must not exist yet; a failed copy removes the partial file.
fn copy_new(from: &Path, path: &Path) -> io::Result<()> {
    let mut out = OpenOptions::new().write(true).create_new(true).open(path)?;
    let copied = (|| {
        io::copy(&mut File::open(from)?, &mut out)?;
        out.flush()?;
        sync_file(&out)
    })();
    if copied.is_err() {
        drop(out);
        let _ = std::fs::remove_file(path);
    }
    copied
}

/// Flush `file`'s content and metadata to the storage device.
pub fn sync_file(file: &File) -> io::Result<()> {
    file.sync_all()
}

/// Flush the folder holding `path`, so the rename itself survives a power loss. Folders can't be
/// opened for this on every system; a failure is not an error of the save.
fn sync_parent(path: &Path) {
    #[cfg(unix)]
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty())
        && let Ok(d) = File::open(dir)
    {
        let _ = d.sync_all();
    }
    #[cfg(not(unix))]
    let _ = path;
}

/// The file a symbolic link `path` points to (the link itself stays a link), else `path`.
fn resolve_link(path: &Path) -> io::Result<PathBuf> {
    match std::fs::symlink_metadata(path) {
        Ok(m) if m.file_type().is_symlink() => std::fs::canonicalize(path),
        _ => Ok(path.to_path_buf()),
    }
}

/// A temporary file name beside `target`: `.<name>.<pid>-<n>.tmp`.
fn temp_path(target: &Path) -> io::Result<PathBuf> {
    let name = target.file_name().ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "the path names no file"))?;
    let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let tmp_name = format!(".{}.{}-{n}.tmp", name.to_string_lossy(), std::process::id());
    Ok(target.with_file_name(tmp_name))
}

/// Give the temporary file the permissions of the file it replaces (a new file keeps the default).
fn keep_permissions(target: &Path, tmp: &Path) -> io::Result<()> {
    match std::fs::metadata(target) {
        Ok(m) => std::fs::set_permissions(tmp, m.permissions()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("etcfile-atomic-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn replaces_and_leaves_no_temporary_file() {
        let d = folder("replace");
        let p = d.join("a.etc");
        write_atomic(&p, b"one").unwrap();
        write_atomic(&p, b"two").unwrap();
        assert_eq!(std::fs::read(&p).unwrap(), b"two");
        assert_eq!(std::fs::read_dir(&d).unwrap().count(), 1);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn a_failed_fill_keeps_the_old_file() {
        let d = folder("fail");
        let p = d.join("a.etc");
        write_atomic(&p, b"old").unwrap();
        let e = write_atomic_with(&p, |f| {
            f.write_all(b"partial")?;
            Err(io::Error::other("stopped"))
        });
        assert!(e.is_err());
        assert_eq!(std::fs::read(&p).unwrap(), b"old");
        assert_eq!(std::fs::read_dir(&d).unwrap().count(), 1);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn write_new_never_replaces() {
        let d = folder("new");
        let p = d.join("a.etc");
        write_new_with(&p, |f| f.write_all(b"first")).unwrap();
        let e = write_new_with(&p, |f| f.write_all(b"second")).unwrap_err();
        assert_eq!(e.kind(), io::ErrorKind::AlreadyExists);
        assert_eq!(std::fs::read(&p).unwrap(), b"first");
        let _ = std::fs::remove_dir_all(&d);
    }
}

// GLOSSARY
// - `TEMP_COUNTER`: per-process counter that keeps temporary file names unique.
// - `read_only_location`: whether a write error means the location can't be written at all.
// - `write_atomic`: write bytes to a path through a temporary file and a rename.
// - `write_atomic_with`: `write_atomic` with a closure that fills the temporary file.
// - `write_new_with`: create a file that must not exist yet, through a temporary file and a hard link.
// - `copy_new`: copy a finished temporary file into a new file where hard links aren't available.
// - `sync_file`: flush a file's content and metadata to the device.
// - `sync_parent`: flush the folder after a rename (best effort).
// - `resolve_link`: the file a symbolic link points to, so the link itself is kept.
// - `temp_path`: the temporary file name beside a target.
// - `keep_permissions`: copy the replaced file's permissions onto the temporary file.
