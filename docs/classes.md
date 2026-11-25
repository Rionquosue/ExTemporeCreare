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
- `src/etcr.rs`: reference boards (`.etcr`): `Board` (`new`, `image`, `image_mut`, `problem`), `BoardImage`, `Source`, `Transform`, `Crop`, `Viewport`; `read_board`, `write_board`, `sniff_board`, `board_thumbnail`.
- `src/boardlinks.rs`: board image links on a file system: `LinkStatus` (`Found`, `Changed`, `Uri`, `Missing`); `link_statuses`, `status`, `relink`, `find_missing`.

## engine/crates/engine (run-time presets)

- `src/presets.rs`: built-in preset libraries read from `tools/Libraries/presets/<kind>/`. `Kind` (`Brushes`, `Symbols`, `Swatches`, `Gradients`, `Patterns`, `GraphicStyles`), `Entry` (one `index.json` row); `set_root`, `root`, `entries`, `entry`, `bytes`, `library`.

## engine/crates/bridge (interface layer, `libetcbridge.so`)

- `src/sessions.rs`: `Bridge` (one engine session, its pending save and the objects at the last press, `gesture_ids`); `create`, `destroy`, `with`: open sessions by handle.
- `src/boards.rs`: reference boards as JSON: `new_board`, `read`, `write`, `links`, `relink_image`, `find`.
- `src/calls.rs`: `execute`, `pointer` (reports the objects a gesture added), `select_tool`, `tool_options`, `set_tool_option`, `state`, `overlays`, `render`, `set_presets_root`: what the UI asks of the engine, answered as `{"ok"}` / `{"error"}` JSON.
- `src/convert.rs`: `view_info` (view state JSON → engine `ViewInfo`), `ui_request`, `ui_requests` (tool requests → JSON), `overlay_json`, `overlays_json` (tool overlays → JSON).
- `src/files.rs`: `PendingSave`; `open` (a copied file with its URI as path), `save` (write into temporary files, keep pending), `save_finish` (with the final URI a rename gave), `validate`.
- `src/jni_exports.rs`: the JNI functions of `EngineBridge`.

## components/engine

- `EngineBridge.kt`: `com.extemporecreare.app.components.engine.EngineBridge`, one engine session for the UI: `execute`, `pointer`, `selectTool`, `toolOptions`, `setToolOption`, `state`, `overlays`, `render`, `openCopied`, `saveToTemp`, `saveFinish`, `close`; `renderBuffer`, `validate`, `setPresetsRoot`.
- `EngineFiles.kt`: `EngineFiles(context, engine)`: `open(uri)` and `save(uri, mode, options)` through content URIs, temporary files in `cacheDir/engine-files`, a check of each `.etc`, then the replace: renamed over the original where the provider allows (`replaceByRename`), else copied into the URI (`copyInto`).
- `EngineBoards.kt`: `EngineBoards(context)`: reference boards through content URIs: `newBoard`, `open`, `save`, `links`, `relink`, `findMissing`.
- `UriReplace.kt`: `UriReplace(context)`: puts finished files in place of content URIs (`replace`: rename over the original where the provider allows, else `copyInto`), `supports`, `displayName`.
- `EnginePresets.kt`: `EnginePresets.install(context)`: copies the bundled presets (assets `presets/`) into `filesDir/presets` when the app version changed and sets the engine's presets folder.
