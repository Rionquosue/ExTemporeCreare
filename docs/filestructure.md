# Filestructure document

## File structure rules

All engine code lives in engine/

All UI code lives in interface/. Its Kotlin package is `com.extemporecreare.app.ui` (`interface` is a reserved word in Kotlin); dial tool settings windows are in interface/tools/<ToolName>/.

All tool code lives in tools/

All dependencies live in dependencies/

The Android app module lives in app/. Its Gradle build compiles the Kotlin in interface/, components/ and tools/, packages tools/Libraries/presets and tools/Libraries/textures (the Texture library's images) as assets, and packages the engine bridge library built from engine/.

## Existing file structure

The following is the existing file structure as a tree:

```
EXTEMPORECREARE
|-- .gitignore
|-- LICENSE
|-- README.md
|-- build.gradle.kts
|-- gradle.properties
|-- settings.gradle.kts
|-- app/
|   |-- build.gradle.kts
|   |-- src/
|       |-- main/
|           |-- AndroidManifest.xml
|-- components/
|   |-- readme.md
|   |-- boards/
|   |   |-- BoardGeometry.kt
|   |   |-- ReferenceBoard.kt
|   |-- color/
|   |   |-- ColorLibrary.kt
|   |   |-- ColorMath.kt
|   |-- debug/
|   |   |-- DebugMode.kt
|   |   |-- PenReading.kt
|   |-- documents/
|   |   |-- DocumentSettings.kt
|   |   |-- RecentFiles.kt
|   |-- engine/
|   |   |-- EngineBoards.kt
|   |   |-- EngineBridge.kt
|   |   |-- EngineFiles.kt
|   |   |-- EnginePresets.kt
|   |   |-- UriReplace.kt
|   |-- textures/
|   |   |-- TextureLibrary.kt
|   |-- tools/
|       |-- CanvasTool.kt
|       |-- DragBox.kt
|       |-- EngineAnswer.kt
|       |-- EngineTool.kt
|       |-- StrokeFinish.kt
|       |-- StrokeGesture.kt
|       |-- StrokeMode.kt
|       |-- TipBrush.kt
|       |-- ToolPointer.kt
|       |-- ToolResult.kt
|       |-- ToolSettings.kt
|       |-- ToolSettingsStore.kt
|       |-- ToolUndoGroup.kt
|-- dependencies/
|   |-- readme.md
|-- docs/
|   |-- Ex-Tempore-Creare_UI_Mockup_v10.html
|   |-- classes.md
|   |-- featurelist.md
|   |-- filestructure.md
|   |-- planned.md
|   |-- variables.md
|   |-- research/
|   |   |-- adf_study_notes.md
|   |-- Standardsdocs/
|   |   |-- applicationstandard.md
|   |   |-- filestandard.md
|   |   |-- Toolstandards/
|   |   |   |-- graphic_design_tools_and_filters.md
|   |   |   |-- toolstandard.md
|   |   |-- UIStandards/
|   |       |-- uistandards.md
|   |-- for_agents/
|       |-- bugcheck.md
|       |-- for_agents.md
|       |-- script_structure.md
|-- engine/
|   |-- ASSETS.md
|   |-- ATTRIBUTION.md
|   |-- Cargo.lock
|   |-- Cargo.toml
|   |-- LICENSE-APACHE
|   |-- LICENSE-MIT
|   |-- NOTICE
|   |-- readme.md
|   |-- licenses/
|   |-- xtask/
|   |-- crates/
|       |-- affinity/
|       |-- ai/
|       |-- apischema/
|       |-- bridge/
|       |-- brush/
|       |-- color/
|       |-- doc/
|       |-- effects/
|       |-- engine/
|       |-- etcfile/
|       |-- geom/
|       |-- pathops/
|       |-- pdf/
|       |-- render/
|       |-- svg/
|       |-- testkit/
|       |-- text/
|       |-- three-d/
|       |-- tools/
|       |-- trace/
|-- fonts/
|   |-- OFL-Inter.txt
|   |-- OFL-JetBrainsMono.txt
|   |-- OFL-SourceSans3.txt
|   |-- OFL-SourceSerif4.txt
|-- interface/
|   |-- MainActivity.kt
|   |-- readme.md
|   |-- board/
|   |   |-- BoardCanvas.kt
|   |-- canvas/
|   |   |-- Camera.kt
|   |   |-- EditorCanvasView.kt
|   |   |-- OverlayPainter.kt
|   |-- color/
|   |   |-- ColorWorkspace.kt
|   |   |-- MixingPad.kt
|   |-- common/
|   |   |-- ConfirmDialog.kt
|   |-- debug/
|   |   |-- DebugOverlay.kt
|   |-- dial/
|   |   |-- ColorControl.kt
|   |   |-- DialGrip.kt
|   |   |-- DialTools.kt
|   |   |-- RotaryDial.kt
|   |   |-- ToolReadout.kt
|   |-- document/
|   |   |-- DocumentPresets.kt
|   |   |-- NewDocumentDialog.kt
|   |-- editor/
|   |   |-- EditorScreen.kt
|   |   |-- EditorState.kt
|   |   |-- MovablePanel.kt
|   |   |-- PanelLayout.kt
|   |   |-- ToolSettingsHost.kt
|   |-- panels/
|   |   |-- FloatingPanel.kt
|   |   |-- LayerCard.kt
|   |   |-- LayersPanel.kt
|   |   |-- MaskRow.kt
|   |   |-- ReferenceBoardPanel.kt
|   |   |-- TextureLibraryPanel.kt
|   |-- theme/
|   |   |-- EtcColors.kt
|   |-- splash/
|   |   |-- SettingsDialog.kt
|   |   |-- SplashScreen.kt
|   |-- topbar/
|   |   |-- TopBar.kt
|   |-- tools/
|       |-- ToolSettingsControls.kt
|       |-- Brush/
|       |   |-- BrushToolSettingsPanel.kt
|       |-- Crop/
|       |   |-- CropToolSettingsPanel.kt
|       |-- Eraser/
|       |   |-- EraserToolSettingsPanel.kt
|       |-- Export/
|       |   |-- ExportToolSettingsPanel.kt
|       |-- Fill/
|       |   |-- FillToolSettingsPanel.kt
|       |-- Marker/
|       |   |-- MarkerToolSettingsPanel.kt
|       |-- Pen/
|       |   |-- PenToolSettingsPanel.kt
|       |-- Pencil/
|       |   |-- PencilToolSettingsPanel.kt
|       |-- Shapes/
|           |-- ShapesToolSettingsPanel.kt
|-- tools/
    |-- readme.md
    |-- Brush/
    |   |-- BrushTool.kt
    |   |-- BrushToolSettings.kt
    |-- Crop/
    |   |-- CropMode.kt
    |   |-- CropTool.kt
    |-- Eraser/
    |   |-- EraserTool.kt
    |   |-- EraserToolSettings.kt
    |-- Export/
    |   |-- ExportScope.kt
    |   |-- ExportSlice.kt
    |   |-- ExportSliceCommands.kt
    |   |-- ExportSliceWriter.kt
    |   |-- ExportTool.kt
    |   |-- ExportUnits.kt
    |-- Fill/
    |   |-- FillTool.kt
    |   |-- FillToolSettings.kt
    |-- Marker/
    |   |-- MarkerTool.kt
    |   |-- MarkerToolSettings.kt
    |-- Move/
    |   |-- MoveTool.kt
    |-- Pen/
    |   |-- PenTool.kt
    |   |-- PenToolSettings.kt
    |-- Pencil/
    |   |-- PencilTool.kt
    |   |-- PencilToolSettings.kt
    |-- Selection/
    |   |-- SelectionTool.kt
    |-- Shapes/
    |   |-- ShapesTool.kt
    |   |-- ShapesToolSettings.kt
    |-- Libraries/
        |-- presets/
            |-- brushes/
            |-- gradients/
            |-- graphic-styles/
            |-- patterns/
            |-- swatches/
            |-- symbols/
        |-- textures/
            |-- README.md
```
