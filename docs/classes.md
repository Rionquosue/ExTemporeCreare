# Classes

Index of the project's own classes and types, by file. Copied VectorCraft code is not listed.

## engine/crates/etcfile (`.etc` project files)

- `src/lib.rs`: crate root. Constants `EXTENSION`, `MIME`, `FORMAT`, `FORMAT_VERSION`, entry names; `is_native_ext`, `is_native_name`.
- `src/error.rs`: `EtcError`, the read and write errors (`NotEtc`, `TooNew`, `BadAsset`, `TooBig`, `Zip`, `Encode`).
- `src/save.rs`: `SaveOptions` (how a document is written); `save`, `save_file`, `save_with`, `save_to`, `ensure_project_id`, `new_project_id`, `sha256_hex`.
- `src/load.rs`: `EtcFile` (a file read), `FileInfo` (`formatVersion`, `projectId`); `load`, `load_file`, `load_file_capped`, `validate`, `sniff`, `project_json`, `thumbnail`, `merged_image`.
- `src/manifest.rs`: `project.json` types: `Head` (written), `Identity` and `Project` (read), `Assets`, `AssetRef`, `ProfileRef`, `Region`; `canvas`, `artboard_bounds`, `project_json`, `version_parts`, `is_newer`.
- `src/stack.rs`: `stack_xml`, the ORA-style `stack.xml` projection.
- `src/zipwrite.rs`: `ZipWriter` and `Method` (`Stored`, `Deflate`): writes ZIP archives with ZIP64 when needed.
- `src/zipread.rs`: `ZipArchive`, `Entry`, `ZipError`: reads and checks ZIP archives; `safe_name`.
- `src/atomic.rs`: `write_atomic`, `write_atomic_with`, `write_new_with`, `sync_file`, `read_only_location`.
- `src/base64.rs`: `encode`, `decode`, `decoded_len`.
- `src/brushart.rs`: `with_brush_art`, `placed_as_groups`.
