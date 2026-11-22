# Ex Tempore Creare — UI Standards

**File:** `UIStandards.md`  
**Status:** Draft for review · version 0.1  
**Date:** 2026-11-22  
**Baseline:** `Ex-Tempore-Creare_UI_Mockup_v10.html`

## 1. Scope and authority

This document defines ETC’s visual language, workspace composition, controls and interaction behaviour using the current v10 mockup. It incorporates the startup and export requirements in `applicationstandard.md` (0.1.2) and persistence requirements in `filestandard.md` (0.4.2).

**MUST** identifies required behaviour; **SHOULD** identifies a recommended implementation. Measurements below are mockup reference values, not mandatory physical device sizes. Proposed refinements and unresolved decisions are explicitly separated in §13.

The mockup includes inherited styles and an older v8 HTML title. Its final style overrides and v10 behaviour define the baseline. Earlier `ui-spec.MD` statements allowing tap-to-select tools or leaving global filter scope undecided are superseded here. This draft does not modify those source documents.

The mockup demonstrates interaction and appearance; it does not establish that every displayed feature is implemented in the production engine.

## 2. Core principles

- **Canvas first:** preserve drawing space with compact, collapsible controls.
- **Tablet first:** primary tasks MUST work with touch and stylus, without hover, right-click or a keyboard.
- **Modular workspace:** panels can float, collapse and dock; their state survives rearrangement.
- **Direct essentials:** keep Menu, Undo, Redo, Save, view Flip and Export accessible.
- **Contextual detail:** reveal settings near the relevant control instead of adding permanent inspectors.
- **Non-destructive editing:** distinguish document changes, view changes and output-region changes.
- **Consistent state:** active tools, selected items, disabled controls and save results must be intelligible.

## 3. Visual language

Use dark blue-grey surfaces, warm neutral text and restrained gold accents. Artwork remains the dominant visual content. Floating surfaces use translucent fills, thin borders, soft shadows and modest background blur.

### 3.1 Colour tokens

Values are taken from the mockup’s root palette; local component overrides may use related shades.

| Role | Reference token | Value |
| --- | --- | --- |
| Deep workspace background | `--bg` | `#0C131B` |
| Application shell | `--shell` | `#141E29` |
| Floating panel | `--panel` | `#17222EEB` |
| Secondary panel surface | `--panel2` | `#1D2A37` |
| Main border | `--line` | `#34404F` |
| Subtle divider | `--line-soft` | `#2A3644` |
| Primary text | `--text` | `#F1EEE9` |
| Secondary text | `--muted` | `#A4AEB7` |
| Subdued text | `--soft` | `#778492` |
| Active / primary accent | `--gold` | `#EFC98C` |
| Secondary gold | `--gold2` | `#DBA767` |
| Positive status accent | `--teal` | `#83C5BB` |
| Demonstration paper | `--paper` | `#E7DDC6` |

The paper colour is demonstration artwork styling, not a forced background for every document. Transparency and user-selected backgrounds remain supported.

### 3.2 Typography, icons and surfaces

| Element | Standard |
| --- | --- |
| Interface text | Inter where available; system sans-serif fallback |
| Branding and splash headings | Restrained serif treatment, matching the mockup’s Georgia/serif styling |
| Labels | Short, readable names; reserve uppercase tracking for small category labels |
| Numeric values | Tabular figures where practical; display units beside values |
| Icons | Consistent outline family, rounded strokes; approximately 17 px / 1.75 px stroke for ordinary icons |
| Panels | Rounded rectangles; root radius 16 px, Layers reference radius 14 px |
| Inputs and small buttons | Smaller, consistent corner radii; visible border and focus state |
| Top actions | Compact pill buttons; Menu uses a distinct rounded-square button |
| Active tool | Warm gold fill, bright outline and restrained glow |
| Selected layer | Warm tinted card and gold border |
| Disabled control | Reduced emphasis and no activation; retain legibility |

Blur and transparency MUST NOT make labels unreadable over artwork. Use an opaque fallback where blur is unavailable. Effects indicate hierarchy and state, not decoration on every control.

## 4. Workspace composition

| Component | Default location / presentation | Behaviour |
| --- | --- | --- |
| Artwork canvas | Central, largest available area | Finite artboard or infinite drawing surface |
| Essential actions | Compact top bar | Menu at far left; Undo, Redo, Save, Flip and Export grouped beside it |
| Branding | Right end of top bar | Can reduce or hide on narrow screens |
| Rotary tool dial | Cropped into the lower-left corner | Rotation selects the tool beneath a fixed active sight |
| Tool readout | Immediately beside the dial | Active tool name, relevant value and quick adjustment |
| Layers | Floating upper-right panel | Ordered cards with controls directly on each card |
| Export Settings | Reuses the Layers footprint | Replaces Layers while the Export tool is active |
| Texture library | Floating upper-left panel | Collapsible preset browser |
| Reference board | Lower-right panel | Collapsible independent reference workspace |
| Colour workspace | Opens near the dial | Picker, swatches, history and mixing pad |
| Main menu | Opens from the leftmost Menu control | Secondary commands, view controls and layout reset |

Do not require a permanent full-height sidebar. Do not restore the obsolete separate layer inspector or a permanent zoom strip; zoom commands remain available through the menu and supported navigation inputs.

The desktop mockup’s top bar is approximately 51 px high. The Layers panel is approximately 368 px wide, reducing at narrower widths. These are starting proportions, subject to touch readability.

**Canvas focus mode** hides floating panels while retaining the rotary dial, its colour control and essential actions. Exiting focus mode MUST restore the previous panel arrangement.

## 5. Opening splash and document setup

### 5.1 Startup routing

| Entry condition | Required destination |
| --- | --- |
| Cold launch without a resumable session | Opening splash |
| Return from another app, minimization, sleep or suspension | Previous editor state immediately |
| Explicit Home / document close | Opening splash |
| Interrupted session with valid recovery | Recover the session; surface recovery failure if necessary |

Resume MUST preserve unsaved work, document, camera, active tool and panel state. Backgrounding alone MUST NOT return the user to the splash.

### 5.2 Splash layout

Present, in order:

1. ETC identity and the primary actions **New Document** and **Open**.
2. Up to **ten recent saved files**, arranged in **two rows of five** on tablet landscape.
3. A separate **Settings** control below the recent-file area.

Recent files are ordered newest first and should show thumbnail, name and date. Empty slots may be unobtrusive placeholders. Missing files offer Locate / Remove from recents. Narrow screens may reflow the grid while preserving order and the ten-file display limit.

### 5.3 New Document

| Control | Required presentation |
| --- | --- |
| Name | Editable; default `Untitled` |
| Canvas mode | Finite / Infinite |
| Preset | Populates editable document settings |
| Width and height | Positive values; finite canvas only |
| Units | `px`, `in`, `cm`, `mm` |
| Resolution | PPI; show resulting pixel dimensions |
| Orientation | Swap finite width and height |
| Colour profile and bit depth | Only capabilities supported by the engine |
| Background | Transparent or selected solid colour |
| Create / Cancel | Clear commit and dismissal controls |

Infinite mode disables irrelevant outer-dimension fields and explains that slices define finite export output. Settings SHOULD support reusable document defaults. Device limits and validation errors must be visible before creation.

## 6. Rotary tool system

### 6.1 Selection contract

- The dial rotates beneath a stationary **45° active sight**, in the upper-right quadrant relative to its centre.
- The tool arriving at that sight becomes active. **Tapping an inactive tool MUST NOT select it.**
- Inactive positions remain muted beneath the frosted ring; the active position stays sharp and highlighted.
- The dial is partially clipped by the workspace edge by design. Its active sight, central colour control and tool settings MUST remain reachable.
- Dragging the ring rotates tools. Mouse-wheel and keyboard stepping may supplement this interaction.
- Repositioning uses the dial grip; ring rotation, colour input and node dragging must not conflict.

The current inventory is **Brush, Pen, Eraser, Fill, Selection, Shapes, Move, Crop and Export**. This inventory does not imply that all tools are functional in the HTML prototype. Slot spacing must accommodate the actual inventory; do not hard-code an eight-slot geometry.

### 6.2 Tool settings

The active tool name and applicable size appear adjacent to the dial. The quick-adjust control supports vertical dragging to change size for applicable tools. Tapping the active tool or its settings trigger opens a compact contextual settings window.

Show only relevant settings: brush/pen/eraser size and hardness, fill tolerance, or other supported tool properties. Preserve each tool’s settings when switching. Do not present a brush size as though it controls Export, Move or Selection.

### 6.3 Central colour control

| Gesture | Behaviour |
| --- | --- |
| Single tap | Swap primary and alternate colours |
| Double tap | Open or close the colour workspace |
| Dial-grip hold and drag | Reposition the toolbar node |

Recognize double taps without first committing an accidental colour swap. Keep both colour samples visible in the centre.

## 7. Layers, folders and filters

### 7.1 Unified stack and cards

Display the hierarchy top to bottom, with the topmost item first. Each card provides its frequently used properties directly; selection MUST NOT be required merely to reveal basic opacity and blend controls.

| Card element | Behaviour |
| --- | --- |
| Name and type | Identify the item; truncate visually without losing access to the full name |
| Visibility | Toggle rendering without deleting content |
| Drag grip | Reorder with a clear drop indicator |
| Folder chevron | Expand/collapse children without changing artwork |
| Blend mode | Select an applicable blend/transfer operation |
| Opacity | Direct percentage entry plus separate dropdown slider |
| Fill | Separate direct percentage entry plus dropdown slider |
| Mask control | Expose applicable mask state and operations |
| Modifier menu | Open non-destructive layer styles |

Opacity and Fill are separate properties and MUST NOT be aliases. Both use a bounded 0–100% value, a visible percent symbol and a distinct arrow button opening a larger touch-friendly slider. Numeric typing must remain possible without dragging a slider. Popovers stay within the workspace and identify the edited property.

The creation row contains **Vector, Raster, Filter and Texture**. The footer provides **Delete** and **Add Folder**. Avoid duplicating these with a second persistent inspector.

### 7.2 Blend modes and modifiers

The mockup lists Normal, Multiply, Screen, Overlay, Soft Light, Hard Light, Color Dodge, Color Burn, Darken, Lighten, Difference, Hue, Saturation, Color and Luminosity.

Its modifier menu lists Stroke, Bevel & Emboss, Color Overlay, Gradient Overlay, Drop Shadow and Outer Glow. These are the current UI inventory, not a complete engine capability specification. Unsupported operations must not silently behave like a different operation.

### 7.3 Filter scope and folders

**Filter layers cascade over every visually lower item, including items in other folders.** Folder boundaries do not silently restrict this global scope. Per-layer styles remain attached to their own item and are distinct from cascading filter layers.

Folders may contain layers, textures, filters and other folders. Visibility and opacity affect descendants. Dragging a folder moves its descendants; self-parenting and moves into descendants are invalid. Preserve child order, provide clear nesting/drop feedback, and confirm deletion of a nonempty folder.

### 7.4 Texture library

Show named preview tiles. Selecting a preset inserts an editable texture layer with visibility, opacity, blend and scale/tiling controls. It does not permanently bake texture into the artwork. Texture layers participate in ordering, folders, saves and exports.

## 8. Colour and reference workspaces

### 8.1 Colour workspace

Provide a saturation/value picker, hue control, numeric colour input, saved swatches, recent history and a small mixing pad. The pad supports paint gestures, primary/alternate colour use, sampling and reset.

Mixing is intended to simulate pigment interaction rather than simply average RGB values. The production mixing algorithm remains unresolved; the prototype’s approximation is not a final rendering requirement.

### 8.2 Reference board

The board is a PureRef-inspired workspace for importing, arranging and viewing reference images. Tapping its header collapses or restores it; holding the header allows repositioning. Collapsing MUST preserve its images, arrangement and camera.

Reference images and the reference camera belong to **`.etcr`**, independently of the artwork **`.etc`** document. Missing linked imagery must remain identifiable and relinkable. Reference imagery MUST NOT appear in artwork exports.

## 9. Export tool and slices

### 9.1 Context switch

Rotating Export into the active sight replaces **Layers** with **Export Settings in the same panel footprint**. Rotating away restores Layers with its prior selection, folder expansion and scroll position. Preserve export-panel state on return.

The top-bar Export action remains available. The radial Export tool supplies region editing; the top-bar action enters the output flow.

### 9.2 Export Settings

| Control | Standard |
| --- | --- |
| Scope | Auto Content Slice; Whole Document for finite canvas; Selected Slice / All Slices when applicable |
| Slice list | Named selectable regions; visibility, rename, duplicate and delete |
| Ratio | Two editable positive values separated by `:` |
| Dimensions | Width / height with `px`, `in`, `cm`, `mm` |
| Output | Supported format, scale/resolution and final raster pixel dimensions |
| Colour / transparency | Options consistent with the document and export capabilities |
| Grid | Dotted overlay, visibility, snapping and grid size |

Whole Document is unavailable for infinite canvases. Grid controls belong to Export Settings, not every tool’s default controls.

### 9.3 Slice editing

1. Drag on the canvas while Export is active to preview a rectangle.
2. Release to create a slice and automatically reveal its ratio/dimensions editor.
3. Drag the centre handle to move the slice; use boundary/corner handles to resize.
4. Edit either ratio value to resize about the existing centre unless a handle determines the anchor.
5. Switch display units without unexpectedly changing the actual region.

Grid snapping uses document coordinates and remains stable under pan and zoom. Slices are editable document objects, not layer cards, painted pixels or destructive crops. Geometry edits participate in Undo/Redo and saves.

### 9.4 Automatic content slice

When exporting without user-defined slices, automatically create **one content-following slice** around all visible exportable artwork, including visible effect extents. No region-selection confirmation is required when valid bounds exist.

Exclude hidden content, reference images, guides, grid dots and interface controls. A configured background alone does not establish content bounds. The automatic region updates as artwork changes until the user moves, resizes or changes its ratio/dimensions; that edit converts it into a fixed manual slice.

Existing manual slices remain authoritative. With no visible exportable content, show **Nothing to export** and offer slice creation. Finite documents may explicitly export Whole Document; this is not the automatic fallback.

All exports exclude UI overlays and use the same intended layer/filter composition as the artwork view. Extended effects must not be clipped by naïve geometry-only bounds.

## 10. Panel movement and input precedence

| Interaction | Required response |
| --- | --- |
| Tap a control | Perform its labelled action |
| Hold a panel header or node grip | Enter repositioning mode |
| Drag the held node | Move it with visible drag feedback |
| Approach a dock target | Show a placement preview |
| Release over a dock target | Dock the node |
| Release elsewhere | Leave it floating |
| Collapse / restore | Preserve component state |
| Reset UI layout | Restore default placement without altering artwork |

Canvas strokes, layer reordering, slider adjustment and dial rotation MUST NOT accidentally move panels. Buttons and form fields must not trigger header dragging. Following a drag, suppress the accidental click that could collapse the panel.

Keep controls reachable after window resize, orientation changes and undocking. The deliberate corner crop of the rotary is an exception to full-node containment, not permission to hide its essential controls.

## 11. Responsive behaviour and feedback

- Landscape is the reference layout; portrait and narrow layouts retain the same interaction model.
- Reduce decorative branding and redundant visible labels before removing essential actions.
- Constrain panel widths to the viewport and scroll panel contents when needed.
- Reflow recent files and document fields on narrow screens; keep dialogs scrollable.
- Reposition popovers to avoid clipping. Maintain usable targets rather than simply shrinking the entire interface.
- Provide accessible names for icon buttons, visible keyboard focus and clear selected/expanded states.
- Use brief toasts for successful actions; use persistent, actionable messages for errors needing correction.
- Save status must reflect actual save completion. Never report an unsuccessful operation as saved.
- View Flip mirrors the view only; it does not rewrite artwork geometry or mirror the exported image.

## 12. State and persistence

| State | Owner / required behaviour |
| --- | --- |
| Artwork, layer hierarchy, effects and export slices | `.etc` document |
| Saved artwork camera | `.etc`, under `workspace.artworkViewport`; document-space centre, zoom and supported view rotation/flip |
| Reference images, layout and reference camera | Independent `.etcr` board |
| Panel positions, docking and collapse state | Workspace/application preferences |
| Tool preferences and recent files | Application/session state |
| Unsaved edits and most recent session camera | Recovery/session state |

Opening a saved document restores its artwork camera. Resuming an interrupted session may use a newer valid recovery camera without silently rewriting the saved document. UI layout changes and reference-board navigation MUST NOT become artwork pixels.

## 13. Prototype gaps and proposed refinements

### 13.1 Known prototype limitations

The HTML uses simplified drawing and compositing, JSON project downloads, a fixed demonstration drawing surface and partial camera restoration. Its automatic bounds use approximations. Several tools and displayed properties are placeholders or simplified implementations.

Production requirements therefore include real finite/infinite geometry, correct effect bounds and compositing, native file persistence, complete camera restore, complete undoable slice editing and supported colour-management behaviour. Displayed format/profile options do not prove engine support.

Some requirements inherited from the application standard—such as exact panel-state restoration, ratio preservation, slice renaming/visibility and complete resize affordances—need validation beyond the mockup. Do not reproduce prototype omissions as standards.

### 13.2 Proposed implementation refinements

These recommendations are additions for review, not measurements claimed from the mockup:

- Target approximately 44 × 44 logical pixels for primary touch hit areas, including invisible padding where compact visuals require it.
- Provide reduced-motion and reduced-transparency alternatives.
- Preserve form labels and focus order when layout reflows.
- Offer an accessible stepped rotary interaction while retaining the rotation-based selection model.
- Establish contrast, minimum text size and interaction timing targets during device testing.

### 13.3 Decisions still open

- Production type scale and minimum readable text size; the mockup includes very small labels.
- Final touch target sizes, gesture thresholds and handedness configurations.
- Exact pigment-mixing model and palette persistence scope.
- Complete tool inventory, overflow/paging behaviour and production tool parameters.
- Detailed mask interactions and which blend/style operations ship in alpha.
- Adjacent-panel docking rules beyond the demonstrated edge docking.

## 14. Acceptance checklist

- [ ] Cold launch shows New Document, Open, up to ten recents and Settings; returning from sleep resumes directly.
- [ ] Finite and infinite document setup clearly distinguish dimensions and export bounds.
- [ ] Rotating the dial changes tools at the fixed 45° sight; tapping inactive slots does not.
- [ ] Colour single/double taps do not conflict; quick size adjustment does not rotate or move the dial.
- [ ] Layer cards expose blend, Opacity and Fill directly; percentage entry and slider popovers stay synchronized.
- [ ] Reordering and folder operations preserve the intended hierarchy and global downward filter scope.
- [ ] Export Settings replaces Layers in place and restores its state on exit.
- [ ] Slice drawing reveals dimensions; moving, resizing, ratios and units behave consistently.
- [ ] Export without manual slices generates one content-following region; manual edits make it fixed.
- [ ] Hidden content, references and UI overlays are excluded from output; visible effect extents are included.
- [ ] Save/reopen restores the artwork camera independently of the reference-board camera.
- [ ] Panels, dialogs and popovers remain reachable in landscape and portrait.
- [ ] Focus mode and layout reset preserve artwork and recover the expected controls.
- [ ] Unsupported prototype features are not presented as completed production capabilities.
