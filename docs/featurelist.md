# Feature list

What the app does today, from its UI. The inherited engine can do more; what it can do but the app does not offer yet is listed in [planned.md](planned.md) as engine-ready.

## Documents and files

- **Opening splash** on a cold launch: New Document, Open, up to ten recent files with thumbnails (Locate a moved file, remove an entry), Settings. Returning from the background resumes the editor.
- **New Document**: finite canvas (width, height in px, in, cm or mm, ppi, RGB or CMYK, transparent, white or a custom background) or **infinite canvas**; presets (Infinite canvas, Screen 1920 × 1080, Tablet 2560 × 1600, Square 2048, A4 300 ppi, Letter 300 ppi); the defaults are kept and editable in Settings.
- **`.etc` files**: Save As, Open, Save to the same file; the artwork camera (centre, zoom, view rotation, View Flip) is saved with each document.
- **Data recovery**: a recovery copy every two minutes and when the app goes to the background; a launch after an interruption restores it.
- **Export**: the Export tool's slices to a chosen folder as PNG, JPEG or WebP at a scale; scopes Auto Content Slice, Whole Document, Selected Slice, All Slices.

## Workspace

- **Canvas**: one finger or the pen uses the active tool; two fingers pan, pinch-zoom and twist-rotate the view. Rotation and flip turn the view only. Marker, Pencil and Eraser strokes show as they are drawn (at the tool's size, in the primary colour; the Eraser's in translucent grey), and a finished stroke, the Brush's included, stays shown until the new render includes it.
- **System bars**: the app is laid out between the status bar, the navigation bar and any display cutout.
- **Top bar**: menu (Home, New document…, Open…, Save as…, Zoom in, Zoom out, Fit artwork, Actual size, Focus mode, Reset panel layout), Save, Flip, Undo, Redo, Export.
- **Panels**: Layers (or Export Settings while the Export tool is active), Texture library and Reference board float over the canvas; tap a header to collapse it, hold it to move the panel, which docks at the workspace's edges; Focus mode hides them; Reset restores the layout.
- **Rotary dial** in the lower-left corner: turn it to bring a tool to the 45° sight (tapping an inactive tool does nothing), tap the active tool for its settings, hold the grip to move the dial, step buttons as an alternative to turning; the tool's name and size show beside it, and a vertical drag on the size changes it.
- **Dial toolbar**: hold the dial's grip and drag it to any edge (left, right, top or bottom) to dock the dial there as a toolbar (a gold preview shows where); tap a tool to choose it, tap the active one for its settings; the colour control, readout, settings and colour workspace stay beside it; drag the toolbar's grip away from its edge for the rotary dial again. The choice is kept with the panel layout.
- **Colour control** in the dial: tap swaps primary and alternate, double tap opens the colour workspace.
- **Colour workspace**: saturation/value square, hue bar, `#rrggbb` and R, G, B entry, saved swatches (app-wide), recent colours, a mixing pad (paint with primary or alternate, sample, reset; the pigment model is provisional).
- **Messages**: brief results disappear; errors stay until dismissed.
- **Debug mode** switched from ADB (`log.tag.ETCDebug`): logcat logging and an on-screen readout of pen pressure, tilt, orientation, hover and render time.

## Tools (the dial)

- **Brush**: raster painting. Each stroke paints a block of pixels the size of the view plus a buffer, at the zoom's resolution; size (screen pixels), hardness, spacing, flow, opacity, pressure-sized; **Lock canvas** keeps painting into the same block, its size in its pixels.
- **Marker**: vector strokes as a Paintbrush (an editable path with a brush) or a Blob Brush (filled shapes); round, oval or chisel tip; tip angle, thickness, orientation fixed, from pen tilt or from barrel rotation (falling back to tilt); size, hardness (feather), brush choice.
- **Pencil**: the Marker's strokes with a pencil tip (a cone that widens as the pen leans, or a hard mechanical circle) and live pencil grain (scale, strength), fixed to the canvas and sharp at any zoom.
- **Pen**: Bézier paths, anchor by anchor; size and hardness.
- **Eraser**: erases vector art along the stroke; size.
- **Fill**: fills the area enclosed by paths with the current fill (gap tolerance); on an infinite canvas with no enclosed area it fills the current screen behind the artwork.
- **Selection**: tap or marquee to select objects; drag to move, the frame's handles resize or rotate.
- **Shapes**: rectangle, ellipse, polygon (sides), star (points), line.
- **Move**: drag anywhere to move the selection.
- **Crop**: a selected placed image (non-destructive), or the canvas's artboard on a finite canvas.
- **Export**: draw, move and resize export slices; name, aspect ratio, format, scale; a dotted grid with snapping; units px, in, cm, mm; the automatic content slice when there are none.

## Layers

- **Rows**: folders, vector, raster, texture and filter layers, topmost first; a layer's strokes and objects are not listed; folders, texture and filter layers can't be drawn on; drag a card's grip above, below or into a folder; collapse folders; a nonempty folder's deletion is confirmed.
- **Create**: Vector layer, Raster layer, Folder, Texture (opens the Texture library), Filter (a menu of filters).
- **Per row**: visibility, blend mode (Normal, Multiply, Screen, Overlay, Soft Light, Hard Light, Color Dodge, Color Burn, Darken, Lighten, Difference, Hue, Saturation, Color, Luminosity), **Opacity** and **Fill** (typed or by slider; Fill fades the content but not its styles, and knocks a shadow out under it), current-layer selection.
- **Layer styles** (modifier menu): Stroke (outside, inside or centre), Bevel & Emboss (inner, outer, emboss, pillow), Color Overlay, Gradient Overlay, Drop Shadow, Outer Glow. They apply with default settings and are drawn from the content at full Fill.
- **Layer masks**: a greyscale mask per row shown as its sublayer (white shows, black hides, grey partly shows); add from the selection or Reveal All (hold for Hide All); tap the mask with a selection to make it the selection, without one to paint into it; Invert, Disable / Enable, Reset to Reveal All or Hide All, Delete.
- **Texture layers**: the Texture library's images (`tools/Libraries/textures`) as named preview tiles; a texture layer tiles its image as a live pattern over the canvas, with a Scale control; export bounds leave it out.
- **Filter layers**: one of 57 pixel filters applied to everything below on the page, across folders, mixed at the layer's Opacity: Artistic (Colored Pencil, Cutout, Dry Brush, Film Grain, Fresco, Neon Glow, Paint Daubs, Palette Knife, Plastic Wrap, Poster Edges, Rough Pastels, Smudge Stick, Sponge, Underpainting, Watercolor), Blur (Radial, Smart), Brush Strokes (Accented Edges, Angled Strokes, Crosshatch, Dark Strokes, Ink Outlines, Spatter, Sprayed Strokes, Sumi-e), Distort (Diffuse Glow, Glass, Ocean Ripple), Pixelate (Color Halftone, Crystallize, Mezzotint, Pointillize), Sharpen (Unsharp Mask), Sketch (Bas Relief, Chalk & Charcoal, Charcoal, Chrome, Conte Crayon, Graphic Pen, Halftone Pattern, Note Paper, Photocopy, Plaster, Reticulation, Stamp, Torn Edges, Water Paper), Stylize (Glowing Edges, Pencil Grain), Texture (Craquelure, Grain, Mosaic Tiles, Patchwork, Stained Glass, Texturizer), Video (De-Interlace, NTSC Colors). They apply with default settings.

## Reference board

- A separate `.etcr` board in its own panel: New, Open, Save; images added from files are linked, not copied.
- Its own camera: two fingers pan, zoom and twist-rotate the board view, never an image; saved with the board.
- Tap an image for four corner handles (drag a handle to resize, just outside a corner to rotate); drag an image to move it; Front, Remove.
- Hold an image for Delete, Embed (stores the image inside the `.etcr`), Flip (Horizontal, Vertical), Crop (non-destructive, by its corners).
- Missing images show as placeholders with their file name and can be relinked; unsaved board state and camera are recovered after a restart; a board with a file is saved when the app goes to the background.
- Reference images never appear in artwork exports.
