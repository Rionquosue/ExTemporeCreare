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
- Pointer event JSON: `{"kind": "down"|"drag"|"up"|"move"|"doubleclick"|"tripleclick", "pos": {"x", "y"}, "pressure"?, "mods"?: {"shift", "alt", "cmd", "ctrl", "space"}, "pen"?: {"tilt": [x, y] (degrees, −90..90, document axes), "rotation" (degrees clockwise)}}`.
- View JSON: `viewport` (`{"x", "y", "width", "height"}`, the screen in document points; read by the app's Fill tool), `zoom`, `outline`, `smartGuides`, `guides`, `snapToGrid`, `pixelPreview`, `showBbox`, `snapToPixel`, `snapToPoint`, `snapToGlyph`, `artboard`.
- Tool request JSON: `{"type": "dialog"|"panel"|"switchTool"|"status"|"place"|"heldPress", …}`.
- Pointer answer JSON: `{"requests": [tool requests], "created": [ids], "changed": bool}`; on `up`, `created` lists the outermost objects added since the `down` and `changed` whether the document changed.
- Tool options answer JSON: `{"tool": engine tool id, "options": {…}}`.
- State JSON: `{"tool", "document", "canvas": "finite"|"infinite", "artboards": [{"index", "x", "y", "width", "height"}], "selection": [ids], "images": [ids], "editing"}` (points; finite = the document has artboards).
- Overlay JSON: `{"type": "marquee"|"path"|"line"|"anchor"|"curvePoint"|"handle"|"label"|"measure"|"distance"|"highlight"|"swatch"|"gridLine", …}`; `path.d` is SVG path data, points are `[x, y]`, colours `[r, g, b]` or `[r, g, b, a]`.
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

## Dial tools

- Tool names (dial names and settings keys): `Brush` (raster), `Marker` (the earlier vector Brush), `Pencil`, `Pen`, `Eraser`, `Fill`, `Selection`, `Shapes`, `Move`, `Crop`, `Export`.
- Engine tools used: Marker and Pencil `paintbrush` (mode `PAINTBRUSH`) / `blobBrush` (mode `BLOB`); Pen `pen`; Eraser `eraser`; Fill `fillArea`; Selection `selection`; Shapes `rectangle`, `ellipse`, `polygon`, `star`, `lineSegment`; Move `selection` (frame only); Crop `cropImage` (one image selected) or `hand` (canvas mode, no input); Export `hand` (no input); Brush `hand` (no input; paints with `paint.stroke`).
- Tool settings file: SharedPreferences `tool-settings` (`ToolSettingsStore.FILE`), one JSON string per tool name.
- Marker settings JSON: `{"mode": "PAINTBRUSH"|"BLOB", "size": pt 0.1–1000 (5), "hardness": % 0–100 (100), "brushLibrary", "brushName", "tip": "ROUND"|"OVAL"|"CHISEL", "tipAngle": ° −180–180 (45), "tipThickness": % 1–100 (30), "orientation": "FIXED"|"TILT"|"ROTATION"}`.
- Marker tip brush: document Calligraphic brush `Marker Tip` (`MarkerTool.TIP_BRUSH`): size 1 (the stroke weight sizes it), `shape` `ellipse`|`rectangle`, angle mode `fixed`|`bearing`|`rotation` with variation 180.
- Calligraphic brush field `shape`: `ellipse` (default, not saved) | `rectangle`; field `cone`: half-angle 1–80° or absent (`CONE_ANGLES`).
- Pencil settings JSON: `{"mode": "PAINTBRUSH"|"BLOB", "size": pt 0.1–1000 (1), "hardness": % (100), "tip": "CONE"|"MECHANICAL", "coneAngle": ° 1–80 (20), "grainScale": pt 0.05–50 (0.6), "grainStrength": % 0–100 (60)}`.
- Pencil tip brush: document Calligraphic brush `Pencil Tip` (`PencilTool.TIP_BRUSH`): round, size 1, `cone` for a cone tip.
- Effect `stylize.pencilGrain` (`PencilTool.GRAIN_EFFECT`): `{scale: pt 0.05–50 (0.6), strength: 0–100 (60)}`; grain constants `OCTAVES` (feature shares 1, 0.47, 0.23; weights 0.55, 0.30, 0.15) and `TOOTH` (0.38–0.62).
- `path.blob` params `shape` (`ellipse`|`rectangle`), `angleFrom` (`fixed`|`tilt`|`rotation`) and `cone` (1–80°); Blob Brush tool options `shape`, `angleFrom`, `cone` (kept with the tool settings).
- Pen settings JSON: `{"size": pt 0.1–1000 (1), "hardness": % (100)}`.
- Eraser settings JSON: `{"size": pt 0.1–1000 (10)}`.
- Fill settings JSON: `{"tolerance": pt ≥ 0 (0)}` → engine `gapDetection` (tolerance > 0) and `gapLength`.
- Fill failsafe (infinite canvas, tap changed nothing): `shape.rectangle` over `viewport`, `paint.setStroke {none}`, `select.set`, `object.arrange.sendToBack`, in undo group `Fill`.
- Shapes settings JSON: `{"kind": "RECTANGLE"|"ELLIPSE"|"POLYGON"|"STAR"|"LINE", "sides": 3–1000 (6), "points": 3–1000 (5)}`.
- Engine tool options set: Blob Brush `size`, `angle`, `roundness`, `shape`, `angleFrom`; Eraser `size`; Fill Area `gapDetection`, `gapLength`; Polygon `sides`; Star `points`.
- Hardness: feather radius = size / 2 × (1 − hardness / 100) pt, applied as effect `stylize.feather` to the objects a Marker stroke or a new Pen path added.
- Undo group labels: `Marker`, `Pencil`, `Pen`, `Move`.
- Crop: canvas box grip reach `CropTool.REACH_PX` = 24 screen px; minimum box `CanvasCropBox.MIN` = 1 pt; canvas apply `artboard.setProps {index, x, y, width, height, moveArt: false, scaleArt: false}`; image apply `object.cropImage {rect}` from the Crop Image tool's `rect` option; cancel sets `rect` to null (image) or restores the artboard's bounds (canvas).
- Export data: `Document::extra["export"]` (`EXPORT_KEY` in engine `cmd/exportslices.rs` and in etcfile), mirrored to `project.json` `export`: `{"grid": {"visible", "snapEnabled", "spacingPx" (32)}, "slices": [{"id": "slice-NNN", "name": "Slice <n>", "rectPx": {"x", "y", "width", "height"}, "aspectRatio": {"x", "y"}, "format": "png"|"jpg"|"webp", "scale" (1), "exportProfileRef" ("builtin:sRGB"), "visible"}]}`; document pixels = points.
- Export commands: `export.slice.list` (→ slices, grid, `contentBounds`), `export.slice.add`, `export.slice.update`, `export.slice.delete`, `export.slice.duplicate`, `export.grid.set`; undo labels `New Export Slice`, `Edit Export Slice`, `Delete Export Slice`, `Duplicate Export Slice`, `Export Grid`.
- Export tool: grip reach `ExportTool.REACH_PX` = 24 screen px; automatic content slice name `Content` (png, scale 1); writer limits `ExportSliceWriter.MAX_PIXELS` = 100 000 000, `QUALITY` = 95; JPEG is flattened on white.
- Shared box: `DragBox.MIN` = 1 pt.
- Export scopes: `Auto Content Slice`, `Whole Document` (finite canvas only), `Selected Slice`, `All Slices`; until chosen, `All Slices` when slices exist, else `Auto Content Slice`. Units: `px` (72 per inch), `in`, `cm`, `mm`.

## Tool settings windows

- UI package: `com.extemporecreare.app.ui` (folder `interface/`).
- Slider ranges: Brush size 0.5–5000 px (cubic), spacing 1–1000 % (squared), flow and opacity 1–100 %; Marker, Pencil, Pen and Eraser size 0.1–1000 pt (cubic track); Pencil cone sharpness 1–80°, grain size 0.05–50 pt (cubic), grain strength 0–100 %; hardness 0–100 %; Fill tolerance 0–72 pt (`MAX_TOLERANCE`, squared track); Shapes sides and points 3–100 (`MAX_COUNT`).
- Settings window width 240–340 dp, corner radius 16 dp; choices and buttons radius 8 dp.
- Selection and Move have no settings window (`uistandards.md` §6.2).
- Brush settings JSON: `{"size": screen px (locked: the splotch's px) 0.5–5000 (20), "hardness": % (80), "spacing": % 1–1000 (25), "flow": % 1–100 (100), "opacity": % 1–100 (100), "pressureSize": bool (true)}`.
- Splotches (paint blocks): image objects whose image key starts `paint-block-` (`BLOCK_KEY_PREFIX`), named `Paint`; made on pen-down from the view's frame plus `BUFFER_SHARE` (10 %) each side, at the zoom in pixels per point clamped 0.25–64 (`RESOLUTIONS`); reused without the lock when they cover the view at a resolution within `ZOOM_TOLERANCE` (5 %) of the zoom; a locked one grows to take in the view and buffer; limits `MAX_SIDE` 16 384 px, `MAX_PIXELS` 64 Mi; stored as PNG; undo label `Brush`.
- `paint.stroke` params: `points` ([x, y, pressure?]), `view` ({x, y, width, height}), `zoom`, `locked` (splotch id), `size`, `hardness`, `spacing`, `flow`, `opacity`, `pressureSize`, `color` (`#rrggbb`; default stroke, else fill, else black) → `{id, created, width, height}`.
- Live Brush stroke bridge answers: `paintBegin` → `{x, y, width, height, pixelsWide, pixelsHigh, splotch}`; `paintAdd` → `{painted: [x, y, w, h] | null}`; `paintPixels` fills premultiplied RGBA; `paintEnd` → `{id, created, width, height}`.
- View JSON `viewport` is also read by the Brush tool (the splotch's frame).
- Layers rows JSON: `{"rows": [{"id", "name", "kind": "folder"|"raster"|"vector"|"guide", "visible", "locked", "opacity" 0–100, "blend" (engine name: Normal, Multiply, Screen, Overlay, SoftLight, HardLight, ColorDodge, ColorBurn, Darken, Lighten, Difference, Exclusion, Hue, Saturation, Color, Luminosity), "current", "children"?}]}`, topmost first.
- Colours: primary = the stroke default, alternate = the fill default; `paintDefaults` / `setPaintDefaults` JSON `{"stroke": "#rrggbb"|null, "fill": "#rrggbb"|null}`.
