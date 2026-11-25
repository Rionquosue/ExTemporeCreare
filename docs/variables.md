# Variables

Stored keys, paths, constants and identifiers.

## `.etc` files (engine/crates/etcfile)

- `EXTENSION` = `etc`: the native file extension.
- `MIME` = `application/x-ex-tempore-creare`: the `mimetype` entry, first in the archive and stored.
- `FORMAT` = `ex-tempore-creare`: `project.json` `format`.
- `FORMAT_VERSION` = `0.4.0`: `project.json` `formatVersion` written, and the newest read.
- `PROJECT_JSON` = `project.json`, `STACK_XML` = `stack.xml`, `MERGED_IMAGE` = `mergedimage.png`, `THUMBNAIL` = `Thumbnails/thumbnail.png`: archive entries.
- `data/raster/image-NNNN.<ext>`: embedded images; `data/vector/image-NNNN.<ext>`: embedded non-image documents (PDF, SVG); `profiles/profile-NNN.icc`: embedded ICC profiles.
- `THUMBNAIL_MAX` = 256: the thumbnail's longest side in pixels.
- `MERGED_MAX` = 8192 (engine `cmd/fileio/native.rs`): the longest side of `mergedimage.png` in pixels.
- `MAX_PRECISION` = 9: the most decimal places a save rounds path geometry to.
- `PROJECT_ID_KEY` = `etcProjectId`: the document `extra` key holding the project id; `project.json` `projectId` repeats it.
- `MAX_JSON` = 1 GiB, `MAX_ASSET` = 1 GiB, `MAX_UNPACKED` = 4 GiB: unpacked size limits when reading.
- `MAX_ENTRIES` = 100000: most entries an archive may list.
- `PRETTY_MAX_OBJECTS` = 2000, `PRETTY_MAX_BYTES` = 1 MiB: limits for an indented `project.json`.
- Native save options (`document.save`): `precision`, `separateArtboards`, `range`, `includeLinked`, `embedProfiles`.
- Engine format id of `.etc`: `etc` (`SAVE_FORMATS`, `TEMPLATE_FORMATS`).

## Preset libraries (engine/crates/engine/src/presets.rs)

- `tools/Libraries/presets/`: the presets root in the repository; the host sets the root at run time (`presets::set_root`), on Android the folder the app copies its bundled presets into.
- Kind folders: `brushes/` (`.etc`), `symbols/` (`.etc`), `patterns/` (`.etc`), `swatches/` (`.vcswatches`), `gradients/` (`.vcswatches`), `graphic-styles/` (`.vcstyles`).
- `index.json` in each kind folder: `{"libraries": [{"id", "name", "file"}]}`, in menu order. `id` is the library id the commands take; `file` is a plain file name in the same folder.

## Interface layer

- Android package / application id: `com.extemporecreare.app`.
- Native library: `libetcbridge.so` (`System.loadLibrary("etcbridge")`), crate `etc-bridge` in `engine/crates/bridge`.
- Bridge class: `com.extemporecreare.app.components.engine.EngineBridge`.
- Answers: `{"ok": <result>}` or `{"error": {"kind": "engine"|"noSession"|"panic", "message"}}`.
- Pointer event JSON: `{"kind": "down"|"drag"|"up"|"move"|"doubleclick"|"tripleclick", "pos": {"x", "y"}, "pressure"?, "mods"?: {"shift", "alt", "cmd", "ctrl", "space"}}`.
- View JSON: `zoom`, `outline`, `smartGuides`, `guides`, `snapToGrid`, `pixelPreview`, `showBbox`, `snapToPixel`, `snapToPoint`, `snapToGlyph`, `artboard`.
- Tool request JSON: `{"type": "dialog"|"panel"|"switchTool"|"status"|"place"|"heldPress", …}`.
- Render pixels: premultiplied RGBA8, rows top to bottom (Android `ARGB_8888`).
- Temporary files for open and save: `<cacheDir>/engine-files/` (`open-*.tmp`, `save-<uid>-<n>.tmp`).
- Save modes the bridge takes: `save`, `saveAs`, `copy`, `template`.

## Android build

- `applicationId` / `namespace`: `com.extemporecreare.app`; `minSdk` 26, `targetSdk` 35, `compileSdk` 35; ABIs `arm64-v8a`, `x86_64`.
- Source sets of `app/`: Kotlin from `interface/`, `components/`, `tools/`; assets from `tools/Libraries` (so `presets/`); native libraries from `app/build/engine/jniLibs` (task `buildEngine`, cargo-ndk).
- Presets on the device: `filesDir/presets/`, with `.version` (the app's last update time) marking the copy.

## Reference boards (`.etcr`)

- `ETCR_EXTENSION` = `etcr`; `ETCR_MIME` = `application/x-ex-tempore-creare-reference`; `ETCR_FORMAT` = `ex-tempore-creare-reference`; `ETCR_FORMAT_VERSION` = `0.1.0`; `BOARD_JSON` = `board.json`.
- `REFERENCE_BOARDS_KEY` = `referenceBoards`: the `.etc` document `extra` key of its board links; saves mirror it to `project.json` `referenceBoards` (§7.5).
- Link statuses: `found`, `changed` (SHA-256 differs), `uri` (only a content URI locates it), `missing`.
- Find-missing limits: 6 folders deep, 20 000 files.
- Temporary board copies on Android: `<cacheDir>/engine-boards/`.
