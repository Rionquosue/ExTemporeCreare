# Ex-Tempore Creare — UI Standards

**Filename:** `uistandards.md`  
**Status:** Alpha UI specification  
**Reference implementation:** `Ex-Tempore-Creare_UI_Mockup_v10.html`  
**Companion specifications:** `applicationstandard.md`, `filestandard.md`  
**Product target:** Tablet-first drawing and design application, with desktop and smaller-screen adaptations.

## 1. Purpose and authority

This document specifies the appearance, structure, and expected interaction of the Ex-Tempore Creare user interface as established in the **v10 interactive mockup** and its approved rotary/layer conventions. It covers the launch screen, document creation, workspace chrome, rotary tools, floating panels, unified layer stack, textures, reference board, colour controls, and export workflow.

- **MUST**: Required for the alpha implementation.
- **SHOULD**: Preferred behavior where platform constraints permit.
- **MAY**: Optional or later enhancement.
- **Prototype-only**: Demonstrated in HTML but not yet a production renderer or file format.

The mockup establishes the UI layout and interaction contract. `filestandard.md` governs `.etc` and `.etcr` persistence, ICC colour handling, and document structure; `applicationstandard.md` governs startup/recovery and export workflow. A visual shortcut in the prototype must not remove an explicitly required feature.

## 2. Interface principles

1. **Canvas first.** Artwork receives the largest possible working area. Avoid permanent inspectors, duplicate status bars, and oversized tool palettes.
2. **Progressive disclosure.** Frequently used operations stay visible; detailed controls open only when requested and close without disrupting the workspace.
3. **Direct manipulation.** Tool rotation, drawing, slice selection, dragging, docking, and quick numeric adjustments should work with stylus or touch.
4. **Consistent control placement.** Do not arbitrarily move or replace the approved hamburger, mask, folder, Blend Mode, Opacity, Fill, or rotary affordances.
5. **Independent movable UI nodes.** Panels may be collapsed, repositioned, and edge-docked. Minimized items should use the least practical screen area.
6. **One authoritative layer stack.** Raster, vector, textures, global filters, and folders share one top-to-bottom compositing hierarchy.
7. **Non-destructive editing.** Tool settings, masks, effects, export slices, and reference transformations must remain editable.
8. **No redundant readouts.** Do not recreate the removed bottom-centre `100% / Pen 12 px` bar; tool identity and size belong in the rotary.

## 3. Launch and session flow

### 3.1 Cold launch

The opening screen MUST appear immediately when no active suspended session exists. Its primary composition is:

```text
                  [ ETC logo ]
              Ex-Tempore Creare
               CREATE ON THE FLY

          [ New Document ]  [ Open ]

                  Recent projects
        [ 1 ] [ 2 ] [ 3 ] [ 4 ] [ 5 ]
        [ 6 ] [ 7 ] [ 8 ] [ 9 ] [10 ]

                    [ Settings ]
```

- Show **up to 10 recent saved projects**, five columns by two rows in the tablet landscape layout; use newest-first ordering.
- Recent entries display a preview/thumbnail area, document name, and saved/modified information. The v10 mockup shows disabled placeholders for empty slots; those placeholders are not actual recent documents.
- `New Document` opens the document-creation dialog. `Open` invokes file selection. `Settings` opens application settings.
- Narrow layouts MAY reflow the grid (the mockup uses two columns on small screens).
- Avoid an intervening marketing, tutorial, or confirmation screen on ordinary launch.

### 3.2 Resume

If the application was backgrounded, tabbed out, minimized, suspended, or put to sleep with an active document, it MUST resume **directly to the prior workspace**, not return to the opening screen. Preserve the document, selected tool, panels, active controls, unsaved edits, and artwork camera/recovery state. An intentional `Home screen` command is different: it returns to the opening screen.

The prototype uses browser-local recovery on visibility loss and `pagehide`. Production resume and checkpoint guarantees belong to `applicationstandard.md`.

## 4. New Document and application settings

### 4.1 New Document dialog

The New Document dialog MUST contain the following controls:

| Control | Standard |
| --- | --- |
| Document name | Editable text; initial value `Untitled`. |
| Canvas mode | `Finite canvas` / `Infinite canvas` segmented choice. |
| Preset | `Custom`, `Full HD · 1920 × 1080`, `Square · 2048 × 2048`, `A4`, `US Letter`. |
| Width / Height | Editable positive values for a finite canvas. Disabled for infinite canvas. |
| Units | `px`, `in`, `cm`, `mm`. |
| Resolution | PPI input; v10 prototype defaults to `300`. |
| Colour profile | sRGB, Display P3, Adobe RGB (1998), Gray Gamma 2.2, FOGRA51 CMYK, GRACoL 2013 CMYK in the mockup. |
| Bit depth | `8-bit / channel` or `16-bit / channel`. |
| Background | Transparent or white. |
| Footer | `Cancel` and `Create Document`. |

- Show the resulting pixel dimensions and PPI as a compact, automatically updated conversion line.
- In infinite mode, disable finite width/height fields and explain that export regions define finite output.
- Keep the form in a focused, dismissible dialog rather than occupying the persistent editor workspace.
- Actual ICC conversion, high-bit-depth rendering, and CMYK storage are **not implemented by the HTML mockup**; their required behavior belongs to `filestandard.md`.

### 4.2 Application Settings

The currently visualized Settings dialog contains **Default colour profile** and **Default resolution (PPI)**, saved locally by the prototype. Additional settings should follow the same compact, task-specific dialog convention, rather than adding permanent controls to the canvas.

## 5. Editor workspace and persistent commands

### 5.1 General layout

The workspace uses a dark, low-distraction shell with a central artwork surface, a slim toolbar across the top, the corner rotary at bottom-left, a layer/export panel typically at upper-right, a texture library typically at upper-left, and a compact reference icon near the lower-right. Panel positions are movable and not fixed workflow constraints.

### 5.2 Top toolbar

The persistent primary controls MUST remain available and keep the established order:

```text
[ Hamburger ] [ Save ] [ Flip Horizontally ] [ Undo ] [ Redo ] [ Export ]
```

- **Hamburger:** Opens the collapsed command menu for secondary actions.
- **Save:** Saves the editable project and its current artwork camera state.
- **Flip Horizontally:** Flips the *view*; it must not silently mirror stored artwork.
- **Undo / Redo:** Reverses or reapplies eligible editing operations.
- **Export:** Opens the export confirmation/output workflow; it does not substitute for selecting the rotary's Export tool when creating/editing slices.
- The brand/logo may be displayed toward the opposite end of the header, as in v10.

The hamburger menu currently exposes Home screen, New composition, Open saved project, Rename composition, Colour laboratory, Layers & filters, Texture library, Reference board, Canvas focus mode, Fit canvas to view, Zoom in/out, Reset UI layout, and a demonstration-only clear-strokes command. Nonessential commands belong here rather than in the persistent toolbar.

### 5.3 Focus mode and canvas controls

- `Canvas focus mode` hides floating panels while leaving the rotary accessible.
- Zoom commands belong in the hamburger menu and gesture/keyboard interactions, **not** in a separate permanent bottom-centre zoom strip.
- Opening a popover must not permanently shift or shrink the artwork surface.
- Camera position and orientation must be restorable from `.etc` and session recovery; see §12.

## 6. Corner rotary — primary tool selector

### 6.1 Geometry and visual state

The rotary is a clipped circular dial emerging from the **bottom-left corner**. Most of its circumference is intentionally hidden outside the viewport; the cut-off edge is a functional visual cue that the ring can rotate.

- The **active tool sits in a fixed aperture at 45° from the bottom-left corner** (upward-right from the rotary centre).
- The tool ring rotates **under** that stationary active position.
- Only the active tool is crisp, prominent, and fully opaque, with a warm-highlight treatment.
- Inactive tool icons remain recognizable at substantially lower opacity beneath a translucent, glass-like rotary cover.
- Do not highlight an arbitrary tool elsewhere on the ring; do not use opacity as a second selection mechanism.
- The rotary retains the central **primary/alternate colour selector**.

### 6.2 Selection gestures

| Gesture | Required response |
| --- | --- |
| Rotate the wheel by touch/stylus drag | Advance or reverse through tools and snap the next selection to the fixed 45° aperture. |
| Scroll over the rotary (desktop) | Step through the tool list. |
| Use arrow keys when the rotary has keyboard focus | Step forward/backward. |
| Tap the **active** tool | Open or close its contextual settings. |
| Tap an **inactive** icon | **Do not change the selected tool.** Selection is by rotation only. |
| Long-press the dedicated drag grip | Reposition/dock the complete rotary node, without rotating the tool selection. |

The v10 tool list comprises **Brush, Pen, Eraser, Fill, Selection, Shapes, Move, Crop, and Export**. Additions must follow the same selection convention.

### 6.3 Tool readout and quick adjustment

- Within the rotary, show the selected tool's name and its primary size when applicable (for example, **Pen / 12 px**).
- Place a **dedicated quick-adjust drag button directly below** the readout. A vertical drag changes the active tool's size/width when relevant. The prototype uses 1–128 px for supported brush tools.
- The drag button is separate from the tool-selection ring and the UI-node drag grip.
- A tool lacking a size parameter should not display an irrelevant size input or quick-adjust handle.
- Frequently changed hardness or comparable quick parameters may be exposed through this area, as determined by the tool's available options.

### 6.4 Contextual expanded settings

The expanded tool window MUST be **closed by default**. Tapping the active tool or its expansion button opens a small, curved panel following the rotary contour. Changing tools closes the previous tool's window.

Current context examples:

- Pen / Brush / Eraser: size slider, brush-size preview, quick presets, hardness.
- Fill: tolerance, Contiguous, Sample all layers.
- Shapes: fill and stroke/outline toggles.
- Tools without relevant options: no empty controls.

The expanded window is not another permanently visible inspector. It may scroll if its controls exceed available space.

### 6.5 Colour core

- **Single tap:** Swap the primary and alternate colours.
- **Double tap:** Open the Colour laboratory panel.
- Display the two colours as overlapping swatches in the rotary centre.
- Keep the centre colour control available even when other panels are hidden.

## 7. Unified Layers / Filters / Folders panel

### 7.1 Structure and ordering

The upper-right floating panel is normally titled **Layers** and contains:

1. A collapsible header, with a drag affordance and item count.
2. Four layer-creation actions: **Vector**, **Raster**, **Filter**, **Texture**.
3. A clearly labelled **COMPOSITING STACK — TOP ↓ BOTTOM**.
4. A scrollable list of layer/folder cards.
5. An optional collapsed `Selected layer · advanced` section.
6. Footer: **Delete** on the far-left and **New Folder** on the far-right.

The **bottom-right footer button creates a folder, never a layer**. The four creation buttons and all stack controls collapse with the parent panel.

The same stack contains raster layers, vector layers, texture layers, global filter layers, and nested folders. Items can be renamed, toggled visible, reordered, and moved between folders. Folders can be expanded/collapsed and can contain mixed layer types. Selection and drag/drop must remain distinguishable.

### 7.2 Required layer-card layout

Every individual layer card MUST preserve this arrangement:

```text
┌──────────────────────────────────────────────────────────────────┐
│ [≡]  Layer name                 [Type]              [Visibility] │
│                                                                  │
│ [Type icon]  Blend Mode     Opacity       Fill          [Mask]     │
│              [dropdown]     [100% ▾]     [100% ▾]                │
└──────────────────────────────────────────────────────────────────┘
```

- **Top-left card hamburger** opens modifiers and layer effects for **that specific layer**.
- **Name and type** remain identifiable without opening a separate inspector.
- **Visibility** is accessible directly on the card.
- **Blend Mode, Opacity, and Fill are always visible**, individually editable, and not buried in the layer modifier menu.
- **Mask action belongs at the bottom-right of the individual layer card.** It adds/removes/edits the associated mask and must visibly indicate whether a mask is active.
- The card's selection state should be visibly distinct without obscuring these controls.

### 7.3 Opacity and Fill interactions

These controls are **independent**, not synonyms. For both properties:

- Display a compact field containing the current percentage and a small dropdown arrow; do **not** put a permanent inline slider in every layer card.
- **Tap the percentage:** Direct numeric entry, 0–100%, with validation/clamping.
- **Tap the adjacent arrow:** Open a larger, touch-friendly anchored slider; update the visible percentage as it moves.
- The temporary popup should be headed **only `Opacity` or `Fill`**, with the current percentage and scale. **Do not repeat the layer name in the popup.** The popover is already anchored to its layer.
- Closing the popup does not change selection, mask, blending, or hierarchy.

Target compositing semantics distinguish complete layer opacity from the fill/content contribution; precise rendering is the engine's responsibility. The HTML preview approximates these values and does not prove Photoshop-equivalent rendering.

### 7.4 Blend modes

The v10 dropdown includes: Normal, Multiply, Screen, Overlay, Soft Light, Hard Light, Color Dodge, Color Burn, Darken, Lighten, Difference, Hue, Saturation, Color, and Luminosity.

Blend selections remain visible per layer. Preserve their names and distinct intent even where a prototype preview cannot correctly demonstrate every mode.

### 7.5 Per-layer modifiers

Tapping the **layer-card hamburger** opens a contextual, non-destructive effects menu. The current mockup includes **Stroke, Bevel & Emboss, Color Overlay, Gradient Overlay, Drop Shadow, and Outer Glow**.

Effects are attached to the selected layer, editable independently, and represented in that layer's controls/status. A full modifier UI may expand later; it must not displace the always-visible Blend Mode, Opacity, or Fill controls.

### 7.6 Cascading global filter layers

- A **Filter layer** affects all applicable layer content **beneath it in top-to-bottom stack order**, regardless of parent folder boundaries.
- A folder boundary MUST NOT terminate a global filter's reach.
- Filter layers can reproduce effects also offered as per-layer modifiers, but with a different scope.
- Reordering a filter changes the layers to which its cascade applies.
- Filter visibility and parameters remain editable.
- Avoid conflating **a filter layer**, **a layer-local modifier**, and **a texture layer**.

The v10 browser demonstration uses simplified SVG/CSS approximations rather than a full stack-aware compositor.

### 7.7 Advanced controls

`Selected layer · advanced` is reserved for less frequently edited properties such as texture size/tiling, parent-folder reassignment, and rename. It MUST NOT replace the controls required directly on every layer card.

## 8. Texture library

The floating **Texture library** is a collapsible panel that can add a procedural overlay as an **editable texture layer**. The v10 library contains eight entries:

| Texture | Intended appearance |
| --- | --- |
| Paper grain | Fine organic surface grain |
| Fine linen | Subtle woven fibres |
| Halftone dots | Print-like dot screen |
| Stipple | Irregular ink specks |
| Crosshatch | Directional sketch lines |
| Watercolor wash | Diffuse pigment wash |
| Crackle | Distressed cracked surface |
| Brushed metal | Linear industrial finish |

Selecting a texture creates or configures a texture layer in the unified stack. Its opacity, Blend Mode, and scale are adjusted through normal layer controls and the advanced inspector. The library itself is **not** a second independent compositor.

## 9. Reference board

### 9.1 Collapsed state

- The reference panel MUST **collapse to a single compact icon**, approximately **46 × 46 px** in the established layout.
- It starts collapsed in the space-saving mockup and remains available near the lower-right by default.
- **Single tap** expands or minimizes it.
- **Long-press** its handle/icon to reposition and edge-dock it.
- Do not leave a wide reference title bar visible when the board is collapsed.

### 9.2 Expanded state

The expanded panel shows imported reference images, captions, an import action, and a compact draggable header. It should remain substantially narrower than the main artwork and not obstruct the canvas unnecessarily.

The full reference system is a separate **`.etcr` reference board**, not a set of embedded `.etc` artwork layers. Its dedicated infinite canvas must support image positioning, cropping, resizing, rotation, independent pan/zoom, and restored reference-camera state. Device-resident images are linked rather than needlessly embedded. These are required product behaviors in `filestandard.md`, **not yet fully interactive in the v10 browser mockup**.

## 10. Colour laboratory

The colour picker opens as a movable panel only when requested. It contains:

- Saturation/value selection square and hue control.
- Primary-colour hexadecimal entry and native colour picker.
- Saved swatches and recent colour history.
- A small **pigment-mixing sketch area**, with Clear, Sample mixed colour, and primary/alternate brush selection.

The mixing pad is intended as a practical, visual colour-blending surface rather than a static gradient swatch. The v10 preview labels its mixing as **K–M-inspired and uncalibrated**; physical pigment accuracy is not established by the mockup.

## 11. Export tool, slices, and grid

### 11.1 Entering export mode

**Export is a tool on the rotary**. Rotate it into the fixed 45° active position to enter export-editing mode.

- The existing **Layers panel transforms in place into `Export Settings`**. Do not open a second persistent sidebar.
- Returning to another tool restores the Layers view without destroying the stack or saved slice definitions.
- The persistent top-toolbar Export button remains a separate **execute/export** action.

### 11.2 Export Settings controls

The v10 panel displays:

| Control | Behavior |
| --- | --- |
| Export scope | Auto/content bounds, All slices, Selected slice, Whole document (finite canvas only). |
| Format | PNG and SVG in the prototype; future formats depend on the actual renderer/export implementation. |
| Scale | 1× and 2× in the prototype. |
| Export slices | List and select named regions; Fit artwork, Duplicate, Delete. |
| Aspect ratio | Two independently editable positive numbers separated by `:`. |
| Dimensions | Editable Width and Height. |
| Units | Pixels, inches, centimetres, millimetres. |
| Show dot grid | Overlay visibility toggle. |
| Snap to grid | Independent snapping toggle. |
| Grid size | Slider with readout; current prototype defaults to 25 px, range 5–200 px. |
| Export action | Start output of the selected scope. |

The dimension editor is disabled or subdued when no slice is selected. When a user draws a new region, selection and its aspect-ratio/dimension controls become available automatically.

### 11.3 Slice interaction

- Drag across the artwork to create a non-destructive rectangular export slice.
- Show the slice outline and an obvious selected state.
- **Drag the central handle to move** an existing slice.
- **Drag the boundary/corner handle to resize** it; the current mockup uses a lower-right resize grip.
- Edit either aspect-ratio value to constrain the box. Edit width or height numerically; switch units without unexpectedly changing the underlying geometry.
- If snapping is enabled, apply it when creating, moving, and resizing a slice.
- The visible grid is a **field of dots**, not an opaque lattice of continuous lines. Grid dots, handles, selections, and panel UI do not export.

### 11.4 Automatic slice when none are defined

On export with no user-defined slices, the application MUST generate **exactly one content-following slice** enclosing all currently visible, exportable artwork. It includes the visible contribution of effects, masks, textures, and global filters and excludes hidden content, references, and UI overlays.

- The automatically generated slice grows/shrinks with changes in visible artwork.
- Moving, resizing, or editing it converts it into a manual slice with fixed bounds.
- If manual slices already exist, do not add another automatic one.
- On an empty document, explain that there is nothing to export and allow manual region definition; do not create a zero-sized file.
- A finite canvas may explicitly export its Whole Document bounds. An infinite canvas cannot export an unbounded area.
- Store slice definitions and grid preferences in `.etc`; they are **not** layer nodes.

The v10 prototype implements a simplified content-bounds estimator. The production requirement is the accurate final rendered extent specified in `applicationstandard.md`.

## 12. Floating nodes, gestures, and persistence

### 12.1 Node movement and docking

- Applicable panels and the rotary have a visible drag affordance.
- **Long-press the drag handle**, then move the node. The HTML demonstration uses approximately 390 ms as its hold threshold; that timing is a prototype value rather than a fixed platform requirement.
- Drag toward any workspace edge to reveal docking feedback. On release, dock or position the node accordingly.
- Panel contents still respond to ordinary taps without beginning a drag.
- Panels can collapse independently. The texture and layer panels collapse to compact headers; the reference board collapses specifically to an icon.
- `Reset UI layout` restores the default placement and collapse states.

### 12.2 Artwork and reference cameras

Save the artwork camera in the `.etc` project: **document-space centre X/Y, zoom, rotation, and horizontal-flip state**. Save on explicit Save and checkpoint on background/suspension for recovery. On reopening/resume, restore the appropriate saved or newer recovery view.

The reference-board camera is independent and belongs to `.etcr`; its pan/zoom/rotation and element transforms must not overwrite the artwork camera. The v10 prototype records zoom and flip but uses a fixed demo centre and zero rotation: **full camera manipulation/restoration remains an alpha implementation requirement**.

### 12.3 File types

- **`.etc`:** Native editable artwork, including layer/filter hierarchy, document/canvas settings, artwork camera, and export slices.
- **`.etcr`:** Separate linked-reference workspace, with external asset links and reference camera.
- **PNG/SVG:** Current browser-prototype export choices; these are outputs, not substitutes for an editable `.etc` project.
- **Krita `.kra`:** Documented alternative in `filestandard.md`, explicitly **out of current project scope**.

The v10 HTML Save action downloads a JSON demonstration file with an `.etc.json` name and stores browser-local data. It does **not** yet write the native `.etc` ZIP/archive format.

## 13. Responsive standards and visual language

### 13.1 Screen adaptation

- **Tablet landscape is the primary composition.** Keep the canvas central, layer/export panel to the side, rotary partially off-screen, and reference icon compact.
- Narrow screens may automatically collapse the texture library, reference board, and, when necessary, layers. Preserve direct access to the collapsed controls.
- Reduce toolbar labels before removing critical toolbar actions. At very narrow sizes, icon-only primary actions are permitted.
- The v10 prototype adapts at approximately **1150 px**, **820 px**, and **570/560 px** viewport widths; these are implementation breakpoints, not mandated device classes.
- Popovers must reposition or scroll rather than becoming inaccessible off-screen.

### 13.2 Colour, typography, and surface styling

Use the v10 look as the initial alpha visual direction:

| Token | Prototype value / treatment |
| --- | --- |
| Background | Deep blue-black `#0c131b` with restrained navy gradients. |
| Shell | `#141e29`. |
| Panel | Translucent dark slate `#17222eeb`, with backdrop blur. |
| Secondary panel | `#1d2a37`. |
| Border | Muted blue-grey `#34404f` / `#2a3644`. |
| Text | Warm off-white `#f1eee9`. |
| Secondary text | Grey-blue `#a4aeb7`. |
| Primary accent | Warm gold `#efc98c`; secondary gold `#dba767`. |
| Secondary status accent | Muted teal `#83c5bb`. |
| Artwork paper (demo) | `#e7ddc6`. |
| Panels and cards | Soft rounded rectangles; dark translucent surfaces, restrained borders and shadows. |
| Headings/branding | Refined serif branding contrasted against compact, legible sans-serif controls. |

Gold indicates active/selected actions and the rotary's sight window. Low opacity indicates inactive rotary positions; these visual states must not be confused. Do not add decoration that reduces the artboard's usable area.

### 13.3 Input and feedback

- Support touch/stylus as first-class input and pointer/keyboard where available.
- Give controls meaningful accessible names, focus states, and values. Keyboard shortcuts demonstrated in v10 include Save, Undo/Redo, zoom, and rotary arrow navigation.
- Use temporary contextual feedback for completed actions and invalid operations. Do not permanently add another status bar.
- Respect platform safe areas and keep active controls reachable, including on compact layouts.

## 14. Alpha acceptance checklist

### Startup and document setup

- [ ] Cold start displays New Document, Open, up to ten recent projects arranged 5×2 on tablet, and Settings.
- [ ] Background/foreground restores the editor without showing the opening screen.
- [ ] Finite/infinite mode, units, PPI, colour profile, bit depth, and background are selectable.

### Workspace and rotary

- [ ] Permanent actions are Hamburger, Save, Flip, Undo, Redo, Export.
- [ ] No duplicate bottom-centre zoom/tool status bar exists.
- [ ] The visible rotary remains clipped into the bottom-left corner.
- [ ] The active tool is always positioned at the 45° sight window; other tools are frosted/dimmed.
- [ ] Rotating selects tools; tapping inactive tools does not.
- [ ] Tapping the active tool opens contextual settings; those settings start closed.
- [ ] Quick vertical drag adjusts supported tool sizes; central colour tap swaps colours and double tap opens the picker.

### Layers and panels

- [ ] Every layer card continuously displays hamburger, name/type, visibility, Blend Mode, Opacity, Fill, and bottom-right mask.
- [ ] Percent value is directly editable; arrow opens a large slider headed **only** Opacity or Fill.
- [ ] Layer hamburgers open the appropriate per-layer modifiers.
- [ ] Global filter layers cascade through every following layer, regardless of folders.
- [ ] Vector/Raster/Filter/Texture creation buttons exist; bottom-right footer creates **New Folder**.
- [ ] Reference board collapses to a **single icon**, not a title bar; expanded board can import and show references.
- [ ] Floating nodes long-press move, dock and collapse without changing unrelated settings.

### Export and persistence

- [ ] Rotating to Export replaces Layers **in the same panel**.
- [ ] A slice can be drawn, ratio/dimensions edited, moved using its central grip, and resized.
- [ ] Dot-grid visibility, snapping and grid spacing work independently.
- [ ] No defined slices produces one automatic content-bounds slice, excluding references and UI chrome.
- [ ] Manual slice edits stop automatic bounds-following for that slice.
- [ ] Infinite canvas export requires finite content bounds/slices; no unbounded export is attempted.
- [ ] `.etc` saves artwork camera, layer state, document settings and export slices. `.etcr` independently saves reference layout/camera.

## 15. Prototype limits and unresolved implementation work

The v10 HTML file is a **functional interface prototype**, not the full alpha renderer. In particular:

- It uses a fixed demonstration drawing surface, even when the new-document UI selects an infinite canvas or other sizes.
- Layer effects, blending, Fill, masks, and global filters are visually approximated; production compositing semantics and precision are not established by the prototype.
- Colour mixing is an experimental preview; ICC conversion, CMYK, calibrated print proofing, and full 16-bit processing remain engine/file-format work.
- It saves JSON demonstration projects, not actual `.etc` archive files. Recent-project entries are browser-local.
- The full `.etcr` infinite reference board, external-file relinking, and independent reference camera are specified but not implemented in the current visual mockup.
- The complete artwork camera, true infinite document navigation, accurate post-filter export bounds, and production export formats require implementation.

**Non-negotiable UI constraint:** Future rendering, file-format, or feature additions must preserve the approved rotary selection model, direct layer-card customization, compact reference icon, and in-place Layers/Export Settings switch unless a later specification explicitly changes them.
