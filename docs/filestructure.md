# Filestructure document

## File structure rules

All engine code lives in engine/

All UI code lives in interface/

All tool code lives in tools/

All dependencies live in dependencies/

The Android app module lives in app/. Its Gradle build compiles the Kotlin in interface/, components/ and tools/, packages tools/Libraries/presets as assets, and packages the engine bridge library built from engine/.

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
|   |-- engine/
|   |   |-- EngineBoards.kt
|   |   |-- EngineBridge.kt
|   |   |-- EngineFiles.kt
|   |   |-- EnginePresets.kt
|   |   |-- UriReplace.kt
|   |-- tools/
|       |-- CanvasTool.kt
|       |-- EngineAnswer.kt
|       |-- EngineTool.kt
|       |-- StrokeFinish.kt
|       |-- StrokeGesture.kt
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
|   |-- filestructure.md
|   |-- variables.md
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
|   |-- readme.md
|-- tools/
    |-- readme.md
    |-- Brush/
    |   |-- BrushTool.kt
    |   |-- BrushToolSettings.kt
    |-- Crop/
    |   |-- CanvasCropBox.kt
    |   |-- CropMode.kt
    |   |-- CropTool.kt
    |-- Eraser/
    |   |-- EraserTool.kt
    |   |-- EraserToolSettings.kt
    |-- Fill/
    |   |-- FillTool.kt
    |   |-- FillToolSettings.kt
    |-- Move/
    |   |-- MoveTool.kt
    |-- Pen/
    |   |-- PenTool.kt
    |   |-- PenToolSettings.kt
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
```
