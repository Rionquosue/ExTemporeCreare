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

## engine/crates/engine (run-time presets)

- `src/presets.rs`: built-in preset libraries read from `tools/Libraries/presets/<kind>/`. `Kind` (`Brushes`, `Symbols`, `Swatches`, `Gradients`, `Patterns`, `GraphicStyles`), `Entry` (one `index.json` row); `set_root`, `root`, `entries`, `entry`, `bytes`, `library`.

## engine/crates/bridge (interface layer, `libetcbridge.so`)

- `src/sessions.rs`: `Bridge` (one engine session); `create`, `destroy`, `with`: open sessions by handle.
- `src/calls.rs`: `execute`, `pointer`, `select_tool`, `render`, `set_presets_root`: what the UI asks of the engine, answered as `{"ok"}` / `{"error"}` JSON.
- `src/convert.rs`: `view_info` (view state JSON → engine `ViewInfo`), `ui_request`, `ui_requests` (tool requests → JSON).
- `src/files.rs`: `PendingSave`; `open` (a copied file with its URI as path), `save` (write into temporary files, keep pending), `save_finish` (with the final URI a rename gave), `validate`.
- `src/jni_exports.rs`: the JNI functions of `EngineBridge`.

## components/engine

- `EngineBridge.kt`: `com.extemporecreare.app.components.engine.EngineBridge`, one engine session for the UI: `execute`, `pointer`, `selectTool`, `render`, `openCopied`, `saveToTemp`, `saveFinish`, `close`; `renderBuffer`, `validate`, `setPresetsRoot`.
- `EngineFiles.kt`: `EngineFiles(context, engine)`: `open(uri)` and `save(uri, mode, options)` through content URIs, temporary files in `cacheDir/engine-files`, a check of each `.etc`, then the replace: renamed over the original where the provider allows (`replaceByRename`), else copied into the URI (`copyInto`).
- `EnginePresets.kt`: `EnginePresets.install(context)`: copies the bundled presets (assets `presets/`) into `filesDir/presets` when the app version changed and sets the engine's presets folder.
