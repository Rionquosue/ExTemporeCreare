# Classes

Index of the project's own classes and types, by file. Copied VectorCraft code is not listed.

## engine/crates/etcfile (`.etc` project files)

- `src/lib.rs`: crate root. Constants `EXTENSION`, `MIME`, `FORMAT`, `FORMAT_VERSION`, entry names, `REFERENCE_BOARDS_KEY`, `EXPORT_KEY`; `is_native_ext`, `is_native_name`.
- `src/error.rs`: `EtcError`, the read and write errors (`NotEtc`, `TooNew`, `BadAsset`, `TooBig`, `Zip`, `Encode`).
- `src/save.rs`: `SaveOptions` (how a document is written); `save`, `save_file`, `save_with`, `save_to`, `ensure_project_id`, `new_project_id`, `sha256_hex`.
- `src/load.rs`: `EtcFile` (a file read), `FileInfo` (`formatVersion`, `projectId`); `load`, `load_file`, `load_file_capped`, `validate`, `sniff`, `project_json`, `thumbnail`, `merged_image`.
- `src/manifest.rs`: `project.json` types: `Head` (written; `referenceBoards`, `export`), `Identity` and `Project` (read), `Assets`, `AssetRef`, `ProfileRef`, `Region`; `canvas`, `artboard_bounds`, `project_json`, `version_parts`, `is_newer`.
- `src/stack.rs`: `stack_xml`, the ORA-style `stack.xml` projection.
- `src/zipwrite.rs`: `ZipWriter` and `Method` (`Stored`, `Deflate`): writes ZIP archives with ZIP64 when needed.
- `src/zipread.rs`: `ZipArchive`, `Entry`, `ZipError`: reads and checks ZIP archives; `safe_name`.
- `src/atomic.rs`: `write_atomic`, `write_atomic_with`, `write_new_with`, `sync_file`, `read_only_location`.
- `src/base64.rs`: `encode`, `decode`, `decoded_len`.
- `src/brushart.rs`: `with_brush_art`, `placed_as_groups`.
- `src/etcr.rs`: reference boards (`.etcr`): `Board` (`new`, `image`, `image_mut`, `problem`), `BoardImage`, `Source`, `Transform`, `Crop`, `Viewport`; `read_board`, `write_board`, `sniff_board`, `board_thumbnail`.
- `src/boardlinks.rs`: board image links on a file system: `LinkStatus` (`Found`, `Changed`, `Uri`, `Missing`); `link_statuses`, `status`, `relink`, `find_missing`.

## engine/crates/engine (run-time presets, export slices)

- `src/cmd/exportslices.rs`: export slices and grid as document data (`Document::extra["export"]`): `export.slice.list`, `export.slice.add`, `export.slice.update`, `export.slice.delete`, `export.slice.duplicate`, `export.grid.set`; `EXPORT_KEY`, `SLICE_FORMATS`, `DEFAULT_GRID_SPACING`.
- `src/presets.rs`: built-in preset libraries read from `tools/Libraries/presets/<kind>/`. `Kind` (`Brushes`, `Symbols`, `Swatches`, `Gradients`, `Patterns`, `GraphicStyles`), `Entry` (one `index.json` row); `set_root`, `root`, `entries`, `entry`, `bytes`, `library`.

## engine/crates/bridge (interface layer, `libetcbridge.so`)

- `src/sessions.rs`: `Bridge` (one engine session, its pending save, and the objects and document at the last press: `gesture_ids`, `gesture_doc`); `create`, `destroy`, `with`: open sessions by handle.
- `src/boards.rs`: reference boards as JSON: `new_board`, `read`, `write`, `links`, `relink_image`, `find`.
- `src/calls.rs`: `execute`, `pointer` (reports the objects a gesture added and whether it changed the document), `select_tool`, `tool_options`, `set_tool_option`, `state`, `overlays`, `render`, `set_presets_root`: what the UI asks of the engine, answered as `{"ok"}` / `{"error"}` JSON.
- `src/convert.rs`: `view_info` (view state JSON → engine `ViewInfo`), `ui_request`, `ui_requests` (tool requests → JSON), `overlay_json`, `overlays_json` (tool overlays → JSON).
- `src/files.rs`: `PendingSave`; `open` (a copied file with its URI as path), `save` (write into temporary files, keep pending), `save_finish` (with the final URI a rename gave), `validate`.
- `src/jni_exports.rs`: the JNI functions of `EngineBridge`.

## components/engine

- `EngineBridge.kt`: `com.extemporecreare.app.components.engine.EngineBridge`, one engine session for the UI: `execute`, `pointer`, `selectTool`, `toolOptions`, `setToolOption`, `state`, `overlays`, `render`, `openCopied`, `saveToTemp`, `saveFinish`, `close`; `renderBuffer`, `validate`, `setPresetsRoot`.
- `EngineFiles.kt`: `EngineFiles(context, engine)`: `open(uri)` and `save(uri, mode, options)` through content URIs, temporary files in `cacheDir/engine-files`, a check of each `.etc`, then the replace: renamed over the original where the provider allows (`replaceByRename`), else copied into the URI (`copyInto`).
- `EngineBoards.kt`: `EngineBoards(context)`: reference boards through content URIs: `newBoard`, `open`, `save`, `links`, `relink`, `findMissing`.
- `UriReplace.kt`: `UriReplace(context)`: puts finished files in place of content URIs (`replace`: rename over the original where the provider allows, else `copyInto`), `supports`, `displayName`.
- `EnginePresets.kt`: `EnginePresets.install(context)`: copies the bundled presets (assets `presets/`) into `filesDir/presets` when the app version changed and sets the engine's presets folder.

## components/tools (shared by the dial tools)

- `CanvasTool.kt`: `CanvasTool`, a rotary dial tool: `name`, `activate`, `pointer`, `deactivate`.
- `EngineTool.kt`: `EngineTool<S>`, a dial tool built on an engine tool, with saved settings: `settings`, `isActive`, `engineToolId`, `pushSettings`, `update`, `setOption`.
- `DragBox.kt`: `DragBox`, a rectangle a drag edits (`Grip`: `INSIDE`, `BOTTOM_RIGHT`; `gripAt`, `drag`, `snap`, `resizeAboutCentre`, `contains`, `copy`, `sameAs`; `of`, `reach`). Crop's canvas box and Export's slices.
- `EngineAnswer.kt`: `EngineAnswer`, one bridge answer parsed: `isOk`, `okObject`, `error`.
- `ToolPointer.kt`: `ToolPointer` (`Kind`: `DOWN`, `DRAG`, `UP`, `MOVE`), a pointer event in document points: `toJson`.
- `ToolResult.kt`: `ToolResult` (`requests`, `created`, `error`, `changed`): `NONE`, `failed`, `failedOrNone`, `fromPointer`.
- `ToolSettings.kt`: `ToolSettings` (`toJson`); `NoToolSettings`.
- `ToolSettingsStore.kt`: `ToolSettingsStore(context)`: `load`, `save` per tool name.
- `ToolUndoGroup.kt`: `ToolUndoGroup`: `begin`, `end`, `isOpen` around `edit.undoGroup.begin` / `end`.
- `StrokeFinish.kt`: `StrokeFinish`: `featherRadius`, `apply` (stroke weight and Feather on created objects).
- `StrokeGesture.kt`: `StrokeGesture(label)`: a stroke tool's gesture as one undo step, finished on release: `pointer`, `close`.

## tools (dial tools)

- `Marker/MarkerTool.kt`: `MarkerTool` (engine `paintbrush` / `blobBrush`; was the Brush tool); `Marker/MarkerToolSettings.kt`: `MarkerMode` (`PAINTBRUSH`, `BLOB`), `MarkerToolSettings`.
- `Pen/PenTool.kt`: `PenTool` (engine `pen`); `Pen/PenToolSettings.kt`: `PenToolSettings`.
- `Eraser/EraserTool.kt`: `EraserTool` (engine `eraser`); `Eraser/EraserToolSettings.kt`: `EraserToolSettings`.
- `Fill/FillTool.kt`: `FillTool` (engine `fillArea`; infinite-canvas failsafe `fillScreen`); `Fill/FillToolSettings.kt`: `FillToolSettings`.
- `Selection/SelectionTool.kt`: `SelectionTool` (engine `selection`).
- `Shapes/ShapesTool.kt`: `ShapesTool` (engine `rectangle`, `ellipse`, `polygon`, `star`, `lineSegment`); `Shapes/ShapesToolSettings.kt`: `ShapeKind`, `ShapesToolSettings`.
- `Crop/CropTool.kt`: `CropTool` (engine `cropImage` for one selected image; else a canvas box applied with `artboard.setProps`): `mode`, `canvasBox`, `apply`, `cancel`; `canvasIndex`; `Crop/CropMode.kt`: `CropMode` (`IMAGE`, `CANVAS`, `UNAVAILABLE`).
- `Export/ExportTool.kt`: `ExportTool` (slice editing; engine `hand` chosen, no input): `slices`, `selectedId`, `editing`, `revealDimensions`, `autoSlice`, `refresh`, `select`, `dimensionsShown`, `setAspect`, `setDimensions`, `setOutput`, `duplicate`, `delete`, `setGrid`; `Export/ExportSlice.kt`: `ExportSlice`, `ExportGrid`, `ExportSliceList`; `Export/ExportSliceCommands.kt`: `ExportSliceCommands` (`list`, `add`, `move`, `update`, `delete`, `duplicate`, `setGrid`); `Export/ExportSliceWriter.kt`: `ExportSliceWriter(context)` (`Region`, `regions(list, scope, selectedId, artboards)`, `write`); `Export/ExportScope.kt`: `ExportScope` (`AUTO_CONTENT`, `WHOLE_DOCUMENT`, `SELECTED_SLICE`, `ALL_SLICES`); `Export/ExportUnits.kt`: `ExportUnits` (`PX`, `IN`, `CM`, `MM`; `fromPx`, `toPx`). `ExportTool` also has `scope`, `chooseScope`.
- `Move/MoveTool.kt`: `MoveTool` (`object.move` per drag step in one undo group; engine `selection` chosen for its frame).

## interface (package `com.extemporecreare.app.ui`)

- `theme/EtcColors.kt`: `EtcColors`, the `uistandards.md` §3.1 colour tokens.
- `tools/ToolSettingsControls.kt`: `ToolSettingsWindow`, `SettingNote`, `SettingSlider`, `SettingChoice`, `SettingSwitch`, `SettingNumberField`, `SettingButton`, `formatNumber`: the controls of the tools' settings windows.
- `tools/Marker/MarkerToolSettingsPanel.kt`: `MarkerToolSettingsPanel` (mode, size, hardness, library brush dropdown).
- `tools/Pen/PenToolSettingsPanel.kt`: `PenToolSettingsPanel` (size, hardness).
- `tools/Eraser/EraserToolSettingsPanel.kt`: `EraserToolSettingsPanel` (size).
- `tools/Fill/FillToolSettingsPanel.kt`: `FillToolSettingsPanel` (tolerance).
- `tools/Shapes/ShapesToolSettingsPanel.kt`: `ShapesToolSettingsPanel` (shape, sides, points).
- `tools/Crop/CropToolSettingsPanel.kt`: `CropToolSettingsPanel` (what it crops, Apply, Cancel, image-crop warning with Save first).
- `tools/Export/ExportToolSettingsPanel.kt`: `ExportToolSettingsPanel` (Export Settings: scope, slices, ratio, dimensions and units, output, grid).
