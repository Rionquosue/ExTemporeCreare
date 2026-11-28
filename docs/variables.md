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

- `applicationId` / `namespace`: `com.extemporecreare.app`; `minSdk` 26, `targetSdk` 37, `compileSdk` 37.2; build-tools 37.0.0; NDK 30.0.16248370 (`ndkVersion`); ABIs `arm64-v8a`, `x86_64`.
- Source sets of `app/`: Kotlin from `interface/`, `components/`, `tools/`; assets from `tools/Libraries` (so `presets/`); native libraries from `app/build/engine/jniLibs` (task `buildEngine`, cargo-ndk).
- Presets on the device: `filesDir/presets/`, with `.version` (the app's last update time) marking the copy.

## Reference boards (`.etcr`)

- `ETCR_EXTENSION` = `etcr`; `ETCR_MIME` = `application/x-ex-tempore-creare-reference`; `ETCR_FORMAT` = `ex-tempore-creare-reference`; `ETCR_FORMAT_VERSION` = `0.1.0`; `BOARD_JSON` = `board.json`.
- `REFERENCE_BOARDS_KEY` = `referenceBoards`: the `.etc` document `extra` key of its board links; saves mirror it to `project.json` `referenceBoards` (§7.5).
- Link statuses: `found`, `changed` (SHA-256 differs), `uri` (only a content URI locates it), `missing`.
- Find-missing limits: 6 folders deep, 20 000 files.
- Temporary board copies on Android: `<cacheDir>/engine-boards/`.
- Board handles: resize reach 18 dp from a corner (`HANDLE_DP`), rotate reach 48 dp beyond it (`ROTATE_DP`), handle dot 6 dp; crop corners reach 1.6 × 18 dp; smallest crop 2 % of the source (`BoardCrop.MIN_FRACTION`); smallest image side 1 board unit.
- Embedded board images: source `kind: "embedded"`, `assetPath` `References/ref-<16 hex of SHA-256>.<ext>` inside the `.etcr` (stored, bytes unchanged); kept locally in `<filesDir>/board-embedded/<boardId>/` while the board is open (Open extracts them, Save packs them).
- Board panel: pictures decoded at most 1024 px a side (`ReferenceBoard.MAX_SIDE`); a new image placed 280 px across at the board's zoom (`PLACED_SIZE`), centred on the camera and upright in the view, ids `ref-` + 8 characters; board zoom 0.01–64; view 240 dp high.
- Board recovery: SharedPreferences `reference-board` (`PREFS`), key `current` (`{uri, modified, board}`), and `camera:<boardId>` for a camera not saved since; written at the end of each board gesture and on file changes. Going to the background saves a board with a file and unsaved changes to that file. Save As name `References.etcr`; picker MIME `ETCR_MIME`.

## Dial tools

- Tool names (dial names and settings keys): `Brush` (raster), `Marker` (the earlier vector Brush), `Pencil`, `Pen`, `Eraser`, `Fill`, `Selection`, `Shapes`, `Move`, `Crop`, `Export`.
- Engine tools used: Marker and Pencil `paintbrush` (mode `PAINTBRUSH`) / `blobBrush` (mode `BLOB`); Pen `pen`; Eraser `eraser`; Fill `fillArea`; Selection `selection`; Shapes `rectangle`, `ellipse`, `polygon`, `star`, `lineSegment`; Move `selection` (frame only); Crop `cropImage` (one image selected) or `hand` (canvas mode, no input); Export `hand` (no input); Brush `hand` (no input; paints with `paint.stroke`).
- Tool settings file: SharedPreferences `tool-settings` (`ToolSettingsStore.FILE`), one JSON string per tool name.
- Marker settings JSON: `{"mode": "PAINTBRUSH"|"BLOB", "size": pt 0.1–1000 (5), "hardness": % 0–100 (100), "brushLibrary", "brushName", "tip": "ROUND"|"OVAL"|"CHISEL", "tipAngle": ° −180–180 (45), "tipThickness": % 1–100 (30), "orientation": "FIXED"|"TILT"|"ROTATION"}`; ROTATION falls back to TILT for a stroke whose press reports no barrel rotation, and TILT without tilt keeps the tip angle.
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

## Editor UI

- Activity: `com.extemporecreare.app.ui.MainActivity` (launcher; keeps itself across configuration changes). Save As MIME `application/x-ex-tempore-creare` (`ETC_MIME`), default name `Untitled.etc`.
- Startup: a cold launch shows the splash; configuration changes keep the editor. Recent files: SharedPreferences `recent-files` key `recent`, at most 10 (`RecentFiles.MAX`), thumbnails from `Thumbnails/thumbnail.png`.
- New Document: `EngineBridge.newDocument` with `{name, infinite, width, height (px = pt), units: "Pixels", colorMode rgb|cmyk, ppi 1–2400, background transparent|white|#rrggbb}`; in/cm/mm convert to pixels at the ppi; defaults kept in SharedPreferences `document-defaults` key `settings` (default 1920 × 1080 px, 144 ppi, RGB, transparent). Presets: Infinite canvas, Screen 1920 × 1080, Tablet 2560 × 1600, Square 2048, A4 300 ppi, Letter 300 ppi.
- Infinite canvas: a document with no artboards (the artboard `file.new` makes is removed before its history starts).
- Artwork camera: kept with the document (`setView {x, y, zoom, rotation, flipped}`, saved as the document's view, named `Artwork`, `SavedView.rotation` and `flipped`) before each save and on Home; restored on open per document (a document without one opens upright, unflipped, fitted). Two-finger twist rotates the view; the render covers the rotated view's bounding box.
- Panel layout: SharedPreferences `panel-layout` (`layersOpen`, `texturesOpen`, `boardOpen`, and per panel `<key>.dock` topStart|topEnd|bottomEnd|float, `<key>.x`, `<key>.y`); keys `textures`, `layers`, `board`; default docks textures topStart, layers topEnd, board bottomEnd; dock reach 70 dp (5 × the 14 dp margin); focus mode is not kept.
- Dial order: Brush, Marker, Pencil, Pen, Eraser, Fill, Selection, Shapes, Move, Crop, Export. Active sight 45° (`SIGHT`); ring radius 0.74 of the dial (`RING`); dial 340 dp, offset −110 dp, +110 dp into the lower-left corner.
- Dial toolbar: the grip docks the dial to the left, right, top or bottom edge within 48 dp (`DOCK_REACH`); kept as `panel-layout` `toolbarDock` (`left`|`right`|`top`|`bottom`, absent: rotary dial), cleared by Reset; tool buttons 48 dp, colour control shown at 52 dp, the toolbar reaching 76 dp in from its edge (`TOOLBAR_REACH`); undocked, the rotary dial's grip lands where it was released (within 60 % of the workspace from its corner); the rotary grip's centre is 40 dp from the left and 272 dp from the bottom before the dial moves.
- Top bar height `BAR_HEIGHT` 52 dp; Layers panel width 368 dp; Texture library 260 dp; Reference board 320 dp.
- Camera zoom range `MIN_ZOOM` 0.01 – `MAX_ZOOM` 64; Fit artwork leaves 18 % margin.
- Workspace surround colour `#0C131B` (`EditorCanvasView.SURROUND`); overlay accents `#EFC98C`, `#83C5BB`.
- Messages (results) show 3.5 s; errors stay until dismissed (`EditorState.error`).
- Main menu: Home, New document…, Open…, Save as…, Zoom in (×1.25), Zoom out (×0.8), Fit artwork, Actual size (100 %), Focus mode, Reset panel layout.
- Layers panel commands: `layer.setCurrent`, `layer.setProps {visible}`, `transparency.set {opacity|blend}`, `layer.new` (`Vector layer` role `vector`, `Raster layer` role `raster`, `Folder` role `folder`; `NodeExtras::layer_role`, saved as `layer_role`), rows are layers only (a layer's objects aren't listed; a layer without a role is a folder when it holds layers, raster when it holds only images, else vector); folders, texture and filter layers refuse new art, `layer.delete` (a folder with contents asks first), `layer.move {ids, target, place: above|below|inside}` (drag: a card's upper half above, lower half below, a folder's middle third inside).
- Layer styles (modifier menu, `effect.apply` on the layer): Stroke → `style.stroke` (3 pt outside, black); Bevel & Emboss → `style.bevelEmboss` (inner bevel, depth 100 %, 5 pt, light 120° at 30°, highlight white screen 75 %, shadow black multiply 75 %); Color Overlay → `style.colorOverlay` (#ff0000 100 %); Gradient Overlay → `style.gradientOverlay` (black to white, 90°); Drop Shadow → `stylize.dropShadow`; Outer Glow → `stylize.outerGlow`. Styles are drawn from the content's silhouette at full Fill, over the content.
- Fill: `layer.setFill {ids, fill 0–100}` (`NodeExtras::fill_opacity`, saved as `fill_opacity` only below 100 %); fades a layer or folder's content but not its styles; below 100 % the layer's shadows and glows are knocked out under its content.
- Texture layers: `texture.new {path, name, scale?}`; images from `tools/Libraries/textures` (assets `textures/`, PNG, JPEG, WebP, GIF, BMP; name = file name), copied to `<cacheDir>/textures/`; a locked layer of its own directly above the current layer, at its level, holding a rectangle over the artboards, or ±100 000 pt (`INFINITE_EXTENT`) on an infinite canvas; image keys `texture-` + 16 hex of the SHA-256; pattern named `Texture <name>`; scale 1 = one image pixel a point, the card's Scale 1–1000 %; at most 64 MB; export bounds leave textures out; library previews 96 px, three a row. The Layers panel's Texture button opens the Texture library.
- Fill and masks apply to any row (a layer or a folder).
- Filter layers: `filter.new {effect, params?, name?}` (effects: the 57 pixel filters, `filter.list`); an empty layer directly above the current layer, at its level, a row of kind `filter` (`effect` in its row); it filters everything drawn before it on the page in paint order, across folders, including the artboards' paper; hidden, it filters nothing; its Opacity mixes filtered and unfiltered; the card shows Opacity only (no blend, Fill or mask). The page is drawn once per filter layer plus once.
- Layer masks: the node's opacity mask; Reveal All = empty art, white background (`clip` off); Hide All = empty art, black background (`clip` on); From Selection = the selected objects' shapes in white, black background. The mask button adds one (from the selection when something is selected, else Reveal All; hold for Hide All); tapping the mask sublayer with something selected makes the mask the selection, else starts or stops painting it (`transparency.editOpacityMask` / `stopEditingOpacityMask`). Layer rows carry `fill`, `mask` ({enabled, inverted, background}), and the stack `selected` and `editingMask`.

## Debug mode

- Switch: the log level property of tag `ETCDebug` (`DebugMode.TAG`). On: `adb shell setprop log.tag.ETCDebug DEBUG`; off: `adb shell setprop log.tag.ETCDebug INFO`; read: `adb logcat -s ETCDebug`. The editor checks it every second.
- Log areas: `[pen]` (pointer readings, hover included), `[tool]` (tool events, timings, results, tool choice), `[engine]` (panel commands and their errors), `[render]` (render size, zoom, time).
- Readout: tool type, pressure, tilt and orientation (degrees, as Android reports them), hover distance, buttons, the tilt the tools get, screen and document position, active tool, zoom, last render time. Android has no standard barrel rotation axis.
- Colour workspace: opens with a double tap on the colour control, near the dial; edits the primary (stroke default) or alternate (fill default). Library: SharedPreferences `colour-library` (`swatches`, `history`), app-wide, history of 12 (`ColorLibrary.HISTORY`). Mixing pad 240 × 96 px, dab radius 7 px, strength 0.35; mix = weighted geometric mean of linear-light RGB (provisional, §13.3 open).
- Recovery: engine prefs `recoveryFolder` = `<filesDir>/recovery` (`EditorState.RECOVERY_FOLDER`), `autosaveRecovery` true; a copy every 2 min (`RECOVERY_EVERY_MS`) and when the app goes to the background; a cold launch with copies left behind (not open, not running) restores them and opens the editor.
- Export grid on the canvas: dots every `spacingPx` document points while the Export tool is active and the grid is shown, doubled until at least 10 screen px apart (`MIN_DOT_GAP`), colour `#A4AEB7`.
- Dial grip: hold and drag; offset kept as `dialX`, `dialY` (dp) in `panel-layout`, within 60 % of the screen; reset by Reset panel layout. Stepper: previous and next tool buttons beside the readout. Touch targets 44 dp.
