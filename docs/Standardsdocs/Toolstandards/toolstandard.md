# Ex-Tempore Creare — Tool Standard

**Document:** `toolstandard.md`  
**Status:** Proposed alpha specification  
**Related documents:** `uistandards.md`, `applicationstandard.md`, `filestandard.md`, `graphic_design_tools_and_filters.md` (research inventory)

## 1. Scope

This document lists the app's own tools and the engine components that provide them. Each tool's code lives in its own folder under `tools/`, and its preset data under `tools/<ToolName>/presets/`, as required by `filestructure.md` and `script_structure.md`. Engine code stays in `engine/`.

## 2. Tools

### 2.1 Graph

- Creates and edits charts (column, stacked, bar, line, area, scatter, pie, radar) from entered data.
- Engine component: `engine/crates/engine` graph commands.

### 2.2 Perspective Grid

- Constructs one-, two- and three-point perspective planes and draws or places artwork onto them.
- Engine component: `engine/crates/engine` perspective grid commands.

### 2.3 3D effects

- Extrude & Bevel, Revolve, Inflate and Rotate, applied to vector artwork as editable effects with projected shading.
- Engine component: `engine/crates/three-d`.

### 2.4 Image Trace

- Converts a raster image into vector shapes (colour quantize, boundary follow, polygon and Bezier fit).
- Engine component: `engine/crates/trace`.

### 2.5 Lens Flare

- Adds an editable vector lens flare (centre, halo, rays, rings).
- Engine component: `engine/crates/engine` flare commands.

### 2.6 Magic Wand

- Selects objects whose attributes (fill, stroke, stroke weight, opacity, blend mode) match a sampled object within set tolerances.
- Engine component: `engine/crates/engine` magic wand commands.

### 2.7 Recolor Artwork

- Reassigns, reduces or harmonizes the colours of selected artwork.
- Engine component: `engine/crates/engine` recolor commands.

### 2.8 Artboard

- Available on **finite canvas** only, as part of the Export tool's slices (`applicationstandard.md` §4, `uistandards.md` §9).
- The user adds an additional board the same way they add a layer.
- Board placement: boards are placed left to right in rows of **5**. The 6th board starts a new row below the first, and so on (boards 1–5 in row 1, 6–10 in row 2).
- Engine component: `engine/crates/doc` and `engine/crates/engine` artboard commands.

### 2.9 Text

- Point text, area text, text on a path, vertical text, Touch Type, character and paragraph styles, and text outlines.
- Composite fonts and kinsoku (Japanese line-break) sets.
- Spell check, Retype (font replacement) and text wrap around objects.
- Engine component: `engine/crates/text` and `engine/crates/engine` type commands.

### 2.10 Fonts

- Bundled fonts live in `fonts/` at the repository root, outside `engine/`.
- Android system fonts in `/system/fonts` are also available.
- Engine component: `engine/crates/text` font database.

### 2.11 Preset libraries

- Built-in brush, symbol, swatch, pattern and gradient presets and graphic styles, plus the Libraries panel.
- Preset data is stored as data files under `tools/<ToolName>/presets/` and loaded by the engine at run time.
- Engine component: `engine/crates/engine` library commands.

### 2.12 File handling

Adapted to the app's own `.etc` and `.etcr` formats (`filestandard.md`, `applicationstandard.md`):

- **Recovery:** recovery checkpoints for unsaved edits, per `applicationstandard.md` §2.1.
- **Package:** copies a saved `.etc` document with the files it links to into a folder of its own.
- **Links:** linked image files, including watching them for changes on disk.
- **Find files:** locates missing linked files for relinking.
