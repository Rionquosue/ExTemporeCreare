# Study notes: adaptively sampled distance fields (ADFs) for the infinite canvas

**Purpose:** how Mischief's drawing technology worked, from its published sources, and how the same ideas can drive Ex-Tempore Creare's infinite canvas mode.
**Status:** study notes, 2026-11-26. Not a specification. Statements marked *our design* are proposals, not taken from Mischief.

## 1. Plan these notes serve

1. **Pencil tool (primary):** vector strokes rendered through distance fields on the infinite canvas, resolution-independent at any zoom.
2. **Brush tool:** stores raster pixels in a block area on the infinite canvas, for a traditional painted look. Raster blocks are bounded regions, so they are not ADF-based (see §9).

## 2. Background: Mischief

- Mischief was made by 61 Solutions; Foundry bought it and discontinued it in 2019. Owners had until 31 December 2019 to get a permanent activation key; no official download remains.
- Its drawing technology was ADFs, invented by Sarah Frisken and Ronald Perry at Mitsubishi Electric Research Laboratories (MERL) and developed further at 61 Solutions. Mischief's own text claimed more than 50 patents.
- Claimed benefits: sharp antialiasing, fast rendering, small files, variable-width strokes, crisp strokes at any zoom.
- Brushes: a review of version 2.1 describes six basic brushes with mild customisation and **no textures**. Textured ADF brushes are therefore not something Mischief is known to have done; §7 is our own design.

## 3. Distance fields

- A distance field gives, for every point, the minimum distance to a shape's edge. A **signed** field uses the sign to tell inside from outside.
- Sign conventions differ between sources: the 2000 paper's examples are positive inside (h = 1 − r for a unit sphere); the 2009 MERL stroke and outline patents make outside negative. *Our design:* positive inside, negative outside, distance in document points. Then union of shapes is `max`, intersection `min`, subtraction `min(a, −b)`.
- Boolean operations and blends work directly on distance values; the 2000 paper builds CSG shapes this way.

## 4. ADF structure (Frisken, Perry, Rockwood, Jones, SIGGRAPH 2000)

- **Idea:** sample the distance field densely only where it has detail, sparsely where it varies smoothly, and store the samples in a spatial hierarchy. In 2D: a quadtree.
- **Cell contents (bi-linear ADF):** the distance at the cell's 4 corners, plus parent and child links. Distance inside a cell is reconstructed by bi-linear interpolation; the gradient (the edge normal) by the analytic gradient of that interpolation.
- **Subdivision rule:** a cell is split only when interpolation of its corners does not reproduce the true distance within an error tolerance. The paper tests the centre and the face and edge centres of each 3D cell (19 points); the 2D equivalent is the cell centre and the 4 edge midpoints. Also stop when the cell cannot contain the edge or a maximum depth is reached.
- **Generation:**
  - *Top-down:* compute the root's corners, then subdivide recursively by the rule above.
  - *Bottom-up:* start from a full regular grid and merge groups of children whose samples the parent reproduces within tolerance, level by level.
- **Size figures from the paper:**
  - The letter "R": 1,713 ADF cells versus 23,573 cells for a 3-colour quadtree (a tree that subdivides every edge cell to full depth).
  - Straight edges stay in large cells; only corners and cusps force deep subdivision.
  - A sphere (r = 0.4) at error 6.25 × 10⁻⁵: 16,201 cells and 24,809 samples, against 32,768 triangles at the same accuracy.

## 5. Bi-quadratic cells (MERL 2003 patent family)

- Each cell stores **9 distances** (corners, edge midpoints, centre). Distance and gradient are reconstructed with a bivariate interpolating polynomial that is continuous along shared edges of equal-size neighbours.
- Continuity between neighbours of different sizes is held to tolerance by the error predicate during generation.
- Result: **5–20 times fewer cells** than bi-linear ADFs for typical glyphs (Times Roman "a": 457 cells versus 17,393 for a 3-colour quadtree; "D": 399 versus 20,813).
- Status: this family's patents have all expired (§10).

## 6. Rendering and antialiasing

- **Distance to coverage:** for each pixel, sample the field, then map the distance through a filter to an intensity. One sample per pixel already gives good antialiasing, because the distance says how far the edge is.
- **Adaptive supersampling (patent figures):** a Gaussian-weighted filter of radius **1.3 × the pixel spacing**. Samples per pixel component: 1 when the cell is larger than the radius, 5 when it is between half the radius and the radius, 13 when it is at most half the radius.
- **Distance effects for free:** offset (thicker or thinner) edges by adding a constant; soft edges by widening the filter; glows and outlines from distance bands.
- **GPU path:** one patent (US 6,917,369) renders cells by texture mapping. On Android this maps naturally to a fragment shader evaluating cells, or to caching rendered tiles as textures.

## 7. Strokes

### 7.1 Pen stroke to distance field (US 7,106,332, expired)

1. Sample the pen state during the stroke: position, and optionally nib geometry and nib orientation.
2. Fit a minimal set of G2-continuous curves to the pen path, to a user-set accuracy.
3. Generate two lists of offset points from the path using the tool's size and orientation; fit curves to them to get the stroke's outlines.
4. Build the distance field from those outlines: a spatial hierarchy over the outline curves, an unsigned distance query at each sample point, and the sign from an intersection cache and a fill rule.

### 7.2 Stylized strokes: distance straight from a centreline (US 8,269,776, expired)

- Each stroke has a **centreline**, **profiles** (thickness along it) and **terminals** (end shapes).
- In cells along the centreline, distance at a point = (function of the profile thicknesses at the closest centreline point) − (distance from that closest point). Terminals get their own small distance fields.
- No outline fitting is needed, which suits pressure-varying width well. *Our design:* this is the better basis for the Pencil tool: the stroke's centreline samples and per-sample radius (from pressure) give distance directly.

### 7.3 Many strokes

- *Our design:* each stroke keeps its own field (or is evaluated on demand); a tile composites strokes in paint order with their colours and opacity, rather than merging all strokes into one shape. Merging into one union field is only right for strokes of the same colour on the same layer.

## 8. Textured strokes (*our design*)

Sources only go as far as the 2001 MERL rendering report: texture inside a variable-width surface, as colour or transparency variation, is "straightforward", and distance values can feed procedural shaders. A design that keeps the canvas resolution-independent:

1. **Stroke coordinates:** for a point near a stroke, `u` = arc length to the closest centreline point, `v` = signed distance across the stroke ÷ local half-width (−1 to 1). Both come out of the centreline distance query in §7.2.
2. **Coverage:** `alpha = edge(d) × grain(u, v, p) × opacity`, where `edge(d)` is the antialiasing filter of §6 and `p` is the canvas position.
3. **Grain sources:**
   - *Procedural noise* (for example fractal value or simplex noise) evaluated at canvas position `p` for paper-tooth effects (pencil on paper), or at `(u, v)` for marks that follow the stroke. Infinitely detailed at any zoom.
   - *Image textures* with mipmaps: only as detailed as the image, so zooming past their resolution softens them. Use for stamps and patterns, not for the main pencil grain.
4. **Pressure and tilt:** scale the width profile, and bias grain contrast (light pressure catches only the paper's peaks).
5. **Stability:** grain is a pure function of document coordinates and stroke parameters, so it never shimmers between redraws, at any pan or zoom.

## 9. Infinite canvas

- **Unbounded quadtree:** when content lands outside the root, add a new root that is twice the size and holds the old one as a child. World coordinates in f64; cell-local math in f32.
- **Level of detail:** draw cells about one screen pixel across; deeper cells only where the view is zoomed in. Mischief claimed a zoom range of 50 trillion to one.
- **Tiles:** cache rendered tiles keyed by (zoom level, x, y); invalidate only tiles a stroke's bounds touch.
- **What the file stores:** the stroke source data (centreline samples, pressure, tool settings, colour). Distance fields and tiles are caches, rebuilt on load. This keeps `.etc` files small and exact.
- **Brush raster blocks:** a block is a bounded raster region at a fixed resolution, placed in document coordinates on the infinite canvas. Outside its resolution it is resampled like an image. Where blocks are stored in `.etc` (for example `data/raster/` with their placement), and how block resolution is chosen, need decisions before the Brush tool is rebuilt.

## 10. Patent status

Checked on Google Patents, 2026-11-26. Not legal advice; the active ones need a proper check before shipping anything close to their claims.

| Patent | Subject | Priority | Status |
| --- | --- | --- | --- |
| US 6,396,492 | Detail-directed hierarchical distance fields (core ADF) | 1999-08-06 | Expired |
| US 7,106,332 | Pen strokes to distance fields | 2003-03-25 | Expired (2024-03-18) |
| US 7,034,845 | Antialiasing a 2D distance field, image order (bi-quadratic cells) | 2003-03-25 | Expired (2023-09-27) |
| US 6,982,724 | Antialiasing a 2D distance field, object order | 2003-03-25 | Expired (2023-09-04) |
| US 7,123,271 | Antialiasing a set of 2D distance fields | 2003-03-25 | Expired (2023-06-03) |
| US 7,030,881 | 2D objects to distance fields | 2003-03-25 | Expired (2024-04-14) |
| US 7,034,830 | Distance field in a corner cell | 2003-03-25 | Expired (2023-05-13) |
| US 6,917,369 | Rendering cell-based distance fields by texture mapping | 2003-03-25 | Expired |
| US 7,006,108 | Composite glyphs | 2003-03-25 | Expired |
| US 7,176,926 | Animating 2D objects | 2003-03-25 | Expired (2023-10-29) |
| US 8,269,776 | Distance field from stylized strokes (centreline, profiles, terminals) | 2009-03-31 | Expired (unpaid fees) |
| US 7,813,555 | Distance field from outlines, with outside/boundary/inside cell reconstruction methods | 2009-03-31 | **Active**, likely to about 2029 |
| US 9,122,270 | Hybrid ADFs: cells holding both distance samples and distance functions | 2012-01-13 | **Active**, to 2034-02-02 |
| US 12,159,370 | Converting a raster image into a layered distance-field image | 2021-12-13 | **Active**, to 2043-01-17 |

- No patents assigned to 61 Solutions turned up, by citation search or web search. The "more than 50 patents" Mischief cited appear to be the MERL families above.
- *Our design* stays clear of the active ones by:
  - Sampled cells only, with no distance functions stored in cells (avoids US 9,122,270's hybrid cells).
  - Distance from stroke centrelines, not per-cell outside/boundary/inside reconstruction methods for outlines (avoids US 7,813,555).
  - No conversion of raster images into distance fields (avoids US 12,159,370).

## 11. Open questions for the owner

1. Pencil tool: is it added to the rotary dial (currently Brush, Pen, Eraser, Fill, Selection, Shapes, Move, Crop, Export) or does it replace one?
2. Pencil on a finite canvas: same distance-field renderer, or only in infinite mode?
3. Brush raster blocks: fixed resolution per block, or chosen from the zoom when the block is created? Can blocks grow as you paint past their edge?
4. Should the Brush tool's current vector modes (Paintbrush, Blob Brush) stay, or does Brush become raster only?

## Sources

- Frisken, Perry, Rockwood, Jones, "Adaptively Sampled Distance Fields: A General Representation of Shape for Computer Graphics", SIGGRAPH 2000, MERL TR2000-15: https://www.merl.com/publications/docs/TR2000-15.pdf
- Perry, Frisken, "A New Framework for Non-Photorealistic Rendering", MERL TR2001-12: https://www.merl.com/publications/docs/TR2001-12.pdf
- US 7,106,332: https://patents.google.com/patent/US7106332B2/en
- US 7,034,845: https://patents.google.com/patent/US7034845B2/en
- US 8,269,776: https://patents.google.com/patent/US8269776B2/en
- US 7,813,555: https://patents.google.com/patent/US7813555B1/en
- US 9,122,270: https://patents.google.com/patent/US9122270B2/en
- US 12,159,370: https://patents.google.com/patent/US12159370B2/en
- US 6,396,492: https://patents.google.com/patent/US6396492B1/en
- What makes Mischief tick? (Hacker News): https://news.ycombinator.com/item?id=6522255
- Foundry discontinues Mischief (CG Channel): https://www.cgchannel.com/2019/08/foundry-discontinues-mischief
- Is Mischief 2.1 more than just an infinite canvas? (Creative Bloq): https://www.creativebloq.com/software/mischiefs-infinite-canvas-91516770
- Mischief, ArtStation Magazine: https://magazine.artstation.com/2015/08/mischief-typical-drawing-package/
