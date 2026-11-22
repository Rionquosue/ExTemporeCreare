# Graphic design tools and filters

Research date: **21 November 2026**. **369 grouped tool and filter entries.**

## Scope and interpretation

- Programs: Photoshop desktop, Clip Studio Paint, Adobe Illustrator desktop, [VectorCraft by storytold / ArtCraft](https://github.com/storytold/vectorcraft), Canva's design/photo editor, and **Affinity Designer 2**.
- This is a broad built-in feature inventory, grouped by behaviour. It includes tools, filters, adjustments and closely related editing commands. It does not enumerate every brush preset, downloadable asset, third-party extension, animation command or administrative menu item.
- “Programs present” records documented implementations or clearly labelled functional equivalents. Shared rows do **not** imply identical algorithms, controls, output or file semantics. Omission is not a definitive claim that a program cannot achieve a similar result.
- Affinity Designer is assessed as the named standalone product. Affinity Photo and the newer unified Affinity application's additional pixel tools are not silently attributed to Designer.
- Clip Studio means **Clip Studio Paint**, primarily Studio Mode with PRO/EX capabilities. DEBUT, older perpetual releases and Simple Mode have restrictions. The detailed feature list and filter manual do not always update together.
- VectorCraft was checked against its public documentation and current repository. Some effects are independent approximations; current-main documentation can run ahead of installed releases. A listed command is not treated as proof of exact Illustrator parity.
- Canva features can depend on plan, region, rollout and AI credits. Marketplace apps are outside the core inventory.
- Blending sections distinguish layer compositing, group behaviour, brush transfer and pigment mixing. Availability varies with colour mode and bit depth. Canva transparency is included without assuming a Photoshop-style blend-mode menu. “Transfer” here includes brush deposition dynamics, not file transfer.
- Descriptions explain practical purpose; behaviour summarizes the operation. Source keys below each section link to the corresponding official references at the end. This is a documentation review, not a hands-on compatibility test.

## Selection, navigation and layout

Sources: [PS1], [AI1], [VC1], [AD1], [CS1], [CA1].

| Tool name | Behaviour | Programs present | Description |
| --- | --- | --- | --- |
| Move / object selection | Selects and translates artwork. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer, Canva | Works on layers, objects or elements according to the application. |
| Direct Selection / Node / Control Point | Edits individual points and curve handles. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Reshapes a path without moving the entire object. |
| Group Selection | Selects progressively larger nested groups. | Adobe Illustrator, VectorCraft | Useful for reaching components inside grouped artwork. |
| Rectangular / elliptical marquee | Creates a geometric pixel selection. | Photoshop, Clip Studio Paint, Affinity Designer | Limits painting and edits to a rectangular or elliptical region. |
| Single-row / single-column marquee | Selects one pixel row or column. | Photoshop | Useful for precise pixel operations. |
| Lasso / polygonal selection | Defines a selection using a drawn outline. | Photoshop, Clip Studio Paint, Affinity Designer | Freehand and straight-segment selection variants. |
| Vector lasso | Selects anchors inside a drawn loop. | Adobe Illustrator, VectorCraft | Selects vector components rather than image pixels. |
| Magnetic Lasso | Follows contrast boundaries while selecting. | Photoshop | Assists tracing irregular subjects. |
| Magic Wand / Auto Select / Flood Select | Selects pixels similar to a sampled colour. | Photoshop, Clip Studio Paint, Affinity Designer | Tolerance and connectivity control the selected region. |
| Vector Magic Wand | Selects objects with matching appearance attributes. | Adobe Illustrator, VectorCraft | Matches fills, strokes or other supported properties. |
| Selection Brush / Selection Pen | Paints a selection mask. | Photoshop, Clip Studio Paint, Affinity Designer | Adds selected regions with brush gestures. |
| Object Selection / Select Subject | Detects image subjects automatically. | Photoshop | Produces a pixel selection for masking or editing. |
| Selection refinement | Adjusts selection edges and softness. | Photoshop, Clip Studio Paint, Affinity Designer | Photoshop Select and Mask and Affinity Refine offer additional edge controls. |
| Hand / pan | Moves the visible canvas area. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer, Canva | Changes the view rather than artwork coordinates. |
| Zoom | Changes view magnification. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer, Canva | Inspects details or the overall composition. |
| Rotate View | Rotates the working view. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Does not permanently rotate the artwork. |
| Artboard | Creates independent design areas within a document. | Photoshop, Adobe Illustrator, VectorCraft, Affinity Designer | Useful for related outputs and multiple sizes. |
| Page management | Adds and reorders document pages. | Clip Studio Paint, Canva | Clip Studio multi-page production is edition-dependent; Canva uses design pages. |
| Align / distribute | Aligns bounds or equalizes spacing. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer, Canva | Arranges multiple elements consistently. |
| Guides / snapping / grids | Constrains placement to reference geometry. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer, Canva | Available guide types and snapping targets differ. |
| Measure | Measures distances and angles. | Photoshop, Adobe Illustrator, VectorCraft, Affinity Designer | Photoshop uses its Ruler tool; other programs expose measuring tools. |
| Dimension | Creates dimension annotations linked to artwork. | Adobe Illustrator, VectorCraft | Labels lengths, angles or radii for technical drawings. |
| Area Tool | Reports enclosed shape area. | Affinity Designer | Useful for scaled plans and material estimates. |
| Slice / Slice Selection | Defines separately exportable rectangular regions. | Photoshop, Adobe Illustrator, VectorCraft, Affinity Designer | Affinity exposes slicing through Export Persona. |
| Print Tiling | Positions artwork relative to printable page tiles. | Adobe Illustrator, VectorCraft | Supports printing a large design across multiple sheets. |

## Vector drawing and path construction

Sources: [AI1], [VC1], [VC2], [AD1], [CS2], [CS3], [PS1], [CA2].

| Tool name | Behaviour | Programs present | Description |
| --- | --- | --- | --- |
| Bezier Curve / Pen | Constructs polynomial curves using anchors and tangent handles. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Commonly called a path tool. Ordinary cubic Bezier handles control tangency; they are not NURBS weights. Clip Studio uses its Bezier figure subtool. |
| Curvature Pen / Curvature | Fits smooth segments through clicked points. | Photoshop, Adobe Illustrator, VectorCraft | Reduces manual handle manipulation. |
| Freeform Pen | Converts a drawn gesture into a path. | Photoshop | Useful for tracing by hand. |
| Vector Pencil | Fits an editable path to a freehand stroke. | Adobe Illustrator, VectorCraft, Affinity Designer | Distinct from a raster pencil brush. |
| Add / delete / convert anchor | Changes path points and corner continuity. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | May be separate tools or modes of the node editor. |
| Straight line / polyline | Draws straight segments. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer, Canva | Canva provides lines and connectors; editable path semantics differ. |
| Rectangle / rounded rectangle | Creates rectangular forms with optional rounded corners. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer, Canva | May be a shape tool, figure subtool or library element. |
| Ellipse / circle | Creates circular or elliptical forms. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer, Canva | Common foundation for icons and diagrams. |
| Polygon / star | Creates regular polygons or star forms. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer, Canva | Parameter controls vary; Canva uses editable elements rather than full path construction. |
| Arc | Draws a curved segment. | Adobe Illustrator, VectorCraft | A dedicated tool distinct from drawing an arbitrary Pen curve. |
| Spiral | Creates a spiral path. | Adobe Illustrator, VectorCraft, Affinity Designer | Useful for ornamental and technical construction. |
| Rectangular Grid / Polar Grid | Creates a grid as artwork. | Adobe Illustrator, VectorCraft | Produces actual vector lines rather than view-only guides. |
| Parametric shape collection | Builds specialized shapes from adjustable parameters. | Affinity Designer | Includes triangle, diamond, trapezoid, double/square star, arrow, donut, pie, segment, crescent, cog, cloud, callouts, tear, heart and cat. |
| QR Code Tool | Creates a QR code from supplied content. | Affinity Designer | A dedicated Designer shape tool. |
| Vector brush | Applies a brush appearance along an editable path. | Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Clip Studio requires a vector layer. Affinity textured vector brushes may contain raster imagery. |
| Blob Brush | Paints filled vector regions. | Adobe Illustrator, VectorCraft | Touching compatible regions can merge into one shape. |
| Shaper / shape recognition | Converts rough gestures into geometric shapes. | Adobe Illustrator, VectorCraft, Canva | Illustrator and VectorCraft use Shaper; Canva offers Shape Assist with different editing behaviour. |
| Smooth / simplify path | Reduces irregularities or unnecessary anchors. | Clip Studio Paint, Adobe Illustrator, VectorCraft | Clip Studio exposes Simplify vector line; vector editors provide smoothing and simplification. |
| Join / connect paths | Connects compatible open endpoints. | Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Builds continuous contours from separate strokes. |
| Scissors | Splits a path at a chosen point. | Adobe Illustrator, VectorCraft | Creates independently editable path sections. |
| Knife | Cuts shapes along a drawn cut. | Adobe Illustrator, VectorCraft, Affinity Designer | Useful for dividing filled regions. |
| Path Eraser | Removes portions of a vector path. | Adobe Illustrator, VectorCraft | Edits geometry rather than painting transparency. |
| Vector Eraser | Deletes vector stroke portions or objects. | Clip Studio Paint, Adobe Illustrator, VectorCraft | Clip Studio can erase to intersections or remove whole strokes. |
| Boolean union / subtract / intersect / exclude | Combines shapes using set operations. | Photoshop, Adobe Illustrator, VectorCraft, Affinity Designer | Photoshop offers shape-path operations; vector editors expose fuller Boolean workflows. |
| Shape Builder | Merges or removes overlapping regions interactively. | Adobe Illustrator, VectorCraft, Affinity Designer | Constructs complex shapes from simple overlapping forms. |
| Offset Path / Contour | Creates a parallel inset or outset boundary. | Adobe Illustrator, VectorCraft, Affinity Designer | Useful for borders, cut lines and layered silhouettes. |
| Live corners / Corner Tool | Rounds or reshapes selected corners. | Adobe Illustrator, VectorCraft, Affinity Designer | Maintains adjustable corner geometry. |
| Outline / expand stroke | Converts stroke appearance into filled geometry. | Adobe Illustrator, VectorCraft, Affinity Designer | Useful before cutting, Boolean operations or final delivery. |
| Variable stroke width | Changes width along a path. | Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Clip Studio adjusts vector line width; Illustrator, VectorCraft and Designer have width controls. |
| Image Trace | Converts a raster image into vector shapes. | Adobe Illustrator, VectorCraft | An automatic tracing operation rather than manual Pen tracing. |
| Blend | Interpolates shapes and colours between objects. | Adobe Illustrator, VectorCraft | Creates intermediate steps along an editable spine. |
| Objects on Path | Distributes objects along a path. | Adobe Illustrator, VectorCraft | Arranges repeated artwork along a curve. |
| Symbol tools | Sprays and modifies collections of symbol instances. | Adobe Illustrator, VectorCraft | Includes Sprayer, Shifter, Scruncher, Sizer, Spinner, Stainer, Screener and Styler. |

## Painting, filling and retouching

Sources: [PS1], [CS1], [CS4], [AD1], [VC1], [CA2].

| Tool name | Behaviour | Programs present | Description |
| --- | --- | --- | --- |
| Raster Brush / Pen | Deposits coloured pixels along a stroke. | Photoshop, Clip Studio Paint, Affinity Designer, Canva | Canva Draw is a simpler drawing workflow; brush engines are not equivalent. |
| Raster Pencil / Pixel Tool | Draws hard-edged or textured pixel marks. | Photoshop, Clip Studio Paint, Affinity Designer | Affinity Pixel Tool targets hard-edged pixel art; Clip Studio pencil brushes emulate pencil media. |
| Airbrush | Builds soft colour with a spray-like brush. | Photoshop, Clip Studio Paint, Affinity Designer | A brush mode or preset rather than always a separate toolbar tool. |
| Mixer Brush / Blend | Mixes colours through brush strokes. | Photoshop, Clip Studio Paint | Photoshop Mixer Brush and Clip Studio Blend provide different mixing models. |
| Smudge | Drags existing colour through an image. | Photoshop, Clip Studio Paint, Affinity Designer | Useful for soft transitions and painterly blending. |
| Decoration / pattern brush | Repeats motifs along a stroke. | Clip Studio Paint, Adobe Illustrator, VectorCraft | Clip Studio Decoration and vector pattern/scatter brushes serve related purposes. |
| Eraser | Removes painted content. | Photoshop, Clip Studio Paint, Affinity Designer, Canva | Canva erases Draw marks; it is not a general pixel-layer eraser. |
| Background Eraser / Magic Eraser | Removes pixels based on sampled colour. | Photoshop | Useful for background cleanup with colour tolerance. |
| Flood Fill / Paint Bucket | Fills a connected region. | Photoshop, Clip Studio Paint, Affinity Designer | Clip Studio can use reference layers and gap-closing options. |
| Live Paint Bucket / Vector Flood Fill | Creates or colours bounded vector regions. | Adobe Illustrator, VectorCraft, Affinity Designer | Affinity Vector Flood Fill and Live Paint differ in resulting object structure. |
| Live Paint Selection | Selects individual faces or edges of Live Paint artwork. | Adobe Illustrator, VectorCraft | Supports recolouring and editing within a Live Paint group. |
| Eyedropper / Colour Picker | Samples a colour from artwork. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer, Canva | Sampling scope and desktop access depend on application and platform. |
| Style Picker / appearance sampling | Copies appearance attributes between objects. | Adobe Illustrator, VectorCraft, Affinity Designer, Canva | Affinity has Style Picker; Canva Copy style is more limited. |
| Gradient fill | Interpolates colours across a region. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer, Canva | Geometry and supported object types vary by program. |
| Gradient Mesh | Interpolates colours over a mesh of points. | Adobe Illustrator, VectorCraft | Supports smoothly shaded vector surfaces. |
| Freeform Gradient | Blends colours around distributed stops or lines. | Adobe Illustrator, VectorCraft | Less constrained than linear and radial gradients. |
| Transparency Tool | Edits a spatial opacity gradient. | Affinity Designer | Creates fades directly on vector or image objects. |
| Clone Stamp / Copy Stamp | Copies pixels from a sampled source. | Photoshop, Clip Studio Paint | Clip Studio calls this Copy stamp. |
| Healing Brush / Spot Healing | Repairs defects while blending with surroundings. | Photoshop | Combines sampled or inferred texture with local appearance. |
| Patch Tool | Repairs a selected region using another region. | Photoshop | Useful for larger blemishes and repeated texture. |
| Content-Aware Fill / Move | Reconstructs image content when removing or relocating an object. | Photoshop | Uses surrounding image information rather than a simple colour fill. |
| Dodge / Burn | Locally lightens or darkens pixels. | Photoshop, Affinity Designer | Useful for shaping tonal contrast. |
| Sponge Tool | Locally changes colour saturation. | Photoshop | Distinct from the artistic Sponge filter. |
| Blur / Sharpen brush | Locally softens or increases edge contrast. | Photoshop, Affinity Designer | Brush-based retouching rather than a whole-layer filter. |
| Red Eye | Corrects flash-induced red pupils. | Photoshop | A specialized photographic repair tool. |
| History Brush / Art History Brush | Paints from a saved image state. | Photoshop | Art History Brush adds stylized strokes. |
| Pattern Stamp | Paints using a repeating pattern. | Photoshop | Useful for patterned repair and decoration. |

## Transforms, masks, type and reusable structure

Sources: [PS1], [AI1], [VC1], [VC2], [AD1], [CS1], [CA1], [CA3].

| Tool name | Behaviour | Programs present | Description |
| --- | --- | --- | --- |
| Scale / rotate / reflect | Changes size, orientation or mirroring. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer, Canva | Basic object or image transformations. |
| Shear / skew / free distort | Moves corners or axes to distort geometry. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Raster and vector implementations differ. |
| Crop | Limits visible image bounds. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer, Canva | Affinity has Vector Crop; Illustrator and VectorCraft crop placed images. |
| Perspective transform | Maps artwork to a perspective quadrilateral. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Available through transforms, effects or dedicated controls. |
| Mesh / envelope warp | Deforms artwork using a mesh or envelope. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Affinity supports vector Warp Groups and documented pixel distortion filters. |
| Puppet Warp | Deforms artwork using positioned pins. | Photoshop, Adobe Illustrator, VectorCraft | Useful for posing or bending connected artwork. |
| Liquify | Pushes, expands, contracts or twists image regions. | Photoshop, Clip Studio Paint | A raster deformation workflow. |
| Vector deformation brushes | Pushes or distorts paths beneath a brush. | Adobe Illustrator, VectorCraft | Warp, Twirl, Pucker, Bloat, Scallop, Crystallize and Wrinkle. |
| Perspective Grid / Selection | Constructs and edits art on perspective planes. | Adobe Illustrator, VectorCraft | Keeps artwork aligned with a perspective grid. |
| Isometric grid / plane editing | Aligns construction to axonometric planes. | Affinity Designer | Useful for diagrammatic illustration. |
| Layer mask | Controls visibility with a mask. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Illustrator and VectorCraft commonly use opacity masks; raster editors use pixel masks. |
| Clipping mask / clipped content | Limits content to another object's boundary. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer, Canva | Canva frames provide a constrained equivalent, not arbitrary path-mask editing. |
| Blend modes | Combines layer or object colours mathematically. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Includes modes such as Multiply and Screen; available sets differ. |
| Opacity | Adjusts overall transparency. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer, Canva | Fades an element without changing its base colour. |
| Point / artistic text | Creates directly positioned editable text. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer, Canva | Common for titles and labels. |
| Area / frame text | Flows text within a bounded frame. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer, Canva | Text threading and overflow behaviour differ. |
| Text on a path | Places text along an editable vector path. | Photoshop, Adobe Illustrator, VectorCraft, Affinity Designer | More general than Canva's preset curved-text effect. |
| Vertical type | Arranges text vertically. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft | Supports vertical composition; language options vary. |
| Touch Type | Transforms individual characters while retaining text. | Adobe Illustrator, VectorCraft | Adjusts position, rotation or scale per character. |
| Text outlines | Converts text to editable vector contours. | Photoshop, Adobe Illustrator, VectorCraft, Affinity Designer | Loses normal text editing after conversion. |
| Character / paragraph styles | Reuses named text formatting. | Photoshop, Adobe Illustrator, VectorCraft, Affinity Designer | Supports consistent typography across a document. |
| Linked symbols | Reuses artwork with synchronized instances. | Adobe Illustrator, VectorCraft, Affinity Designer | Editing a symbol can update multiple placements. |
| Smart Object | Keeps embedded or linked source content for reversible editing. | Photoshop | Supports reusable transforms and Smart Filters. |
| Adjustment / correction layers | Applies editable colour changes through layers. | Photoshop, Clip Studio Paint, Affinity Designer | Keeps the original pixels available; Clip Studio is edition-dependent. |
| Appearance / graphic styles | Stacks and reuses fills, strokes and effects. | Adobe Illustrator, VectorCraft, Affinity Designer | Affinity's Appearance and Styles offer a different subset of controls. |
| Radial / grid / mirror Repeat | Creates editable repeated arrangements. | Adobe Illustrator, VectorCraft | Dedicated live-repeat objects. |
| Pattern creation | Defines artwork as a repeating pattern. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft | Clip Studio uses tiling materials; workflows are not identical. |

## Colour and tonal adjustments

Sources: [PS1], [CS5], [AD2], [VC4], [CA1].

| Tool name | Behaviour | Programs present | Description |
| --- | --- | --- | --- |
| Brightness / Contrast | Changes overall luminance and tonal separation. | Photoshop, Clip Studio Paint, Affinity Designer, Canva, VectorCraft | VectorCraft provides colour-adjustment effects. |
| Levels | Remaps black point, white point and midtones. | Photoshop, Clip Studio Paint, Affinity Designer, VectorCraft | Useful for correcting a compressed tonal range. |
| Curves / Tone Curve | Remaps tones using an editable curve. | Photoshop, Clip Studio Paint, Affinity Designer, VectorCraft | Supports selective control over different tonal regions. |
| Hue / Saturation / Lightness | Changes colour angle, intensity and brightness. | Photoshop, Clip Studio Paint, Affinity Designer, Canva, VectorCraft | Names and colour models differ; Canva exposes image adjustments. |
| Colour Balance | Shifts colours toward complementary colour pairs. | Photoshop, Clip Studio Paint, Affinity Designer | Useful for correcting casts or grading shadows and highlights. |
| Exposure | Changes exposure-related brightness. | Photoshop, Affinity Designer, VectorCraft | A dedicated adjustment distinct from simple brightness. |
| Vibrance | Adjusts saturation with selective weighting. | Photoshop, Affinity Designer, Canva | Usually affects muted colours more strongly. |
| Black and White / monochrome | Removes hue while controlling tonal rendering. | Photoshop, Clip Studio Paint, Affinity Designer, Canva, VectorCraft | Clip Studio uses saturation, expression-colour or monochrome options; controls differ. |
| Invert | Reverses colour or tone values. | Photoshop, Clip Studio Paint, Affinity Designer, VectorCraft | Produces a negative-style result. |
| Posterize | Reduces the number of tonal levels. | Photoshop, Clip Studio Paint, Affinity Designer | Creates stepped colour bands. |
| Threshold / Binarization | Converts tones to black or white at a cutoff. | Photoshop, Clip Studio Paint, Affinity Designer | Useful for masks and high-contrast artwork. |
| Gradient Map | Maps image brightness to gradient colours. | Photoshop, Clip Studio Paint, Affinity Designer | Common for colour grading and colouring grayscale paintings. |
| Channel Mixer | Builds output channels from weighted input channels. | Photoshop, Affinity Designer | Useful for colour correction and monochrome conversion. |
| Selective Colour | Adjusts selected colour families. | Photoshop, Affinity Designer | Allows targeted print-oriented colour correction. |
| LUT / Colour Lookup | Remaps colours through a lookup table. | Photoshop, Affinity Designer | Applies a repeatable colour grade. |
| White Balance / temperature and tint | Corrects warm/cool and green/magenta casts. | Photoshop, Affinity Designer, Canva, VectorCraft | Photoshop provides this in Camera Raw. |
| Shadows / Highlights | Targets dark and bright tonal regions separately. | Photoshop, Affinity Designer, Canva | Recovers or reshapes local tonal detail. |
| Lens / Photo Filter | Applies a photographic colour cast. | Photoshop, Affinity Designer | Simulates coloured lens filters. |
| Split Toning | Colours shadows and highlights separately. | Affinity Designer | Useful for stylized photographic grading. |
| Soft Proof | Previews an output colour profile. | Photoshop, Adobe Illustrator, VectorCraft, Affinity Designer | A colour-management preview, not a permanent image filter. |
| OpenColorIO adjustment | Transforms colours through an OCIO configuration. | Affinity Designer | Relevant to managed rendering and compositing pipelines. |
| Recolour artwork | Reassigns an illustration's palette. | Adobe Illustrator, VectorCraft | Edits multiple vector colours together. |

## Blur, sharpening, distortion and image cleanup

Sources: [PS2], [CS6], [AD3], [VC3], [AI2], [CA1].

| Tool name | Behaviour | Programs present | Description |
| --- | --- | --- | --- |
| Gaussian Blur | Smooths pixels with a Gaussian-weighted kernel. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Affinity Designer exposes this as a layer effect. |
| Blur / Blur More | Applies basic smoothing. | Photoshop, Clip Studio Paint | Simple blur commands with limited controls. |
| Motion Blur | Blurs along a direction. | Photoshop, Clip Studio Paint | Suggests movement or camera travel. |
| Radial Blur | Blurs around or outward from a centre. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft | Produces rotational or zoom-like blur. |
| Lens Blur | Simulates defocus and aperture-shaped highlights. | Photoshop, Clip Studio Paint | Clip Studio introduced this in version 4.0. |
| Smart Blur | Smooths regions while preserving selected edges. | Photoshop, Adobe Illustrator, VectorCraft | Useful for simplifying noise without uniformly softening contours. |
| Surface Blur | Smooths similar tones while preserving boundaries. | Photoshop | Often used for controlled retouching. |
| Box / Average / Shape Blur | Uses uniform, whole-region or shaped averaging. | Photoshop | Different kernels create different softening patterns. |
| Blur Gallery | Applies spatially controlled photographic blur. | Photoshop | Includes Field, Iris, Tilt-Shift, Path and Spin Blur. |
| Sharpen / Sharpen More | Increases edge contrast. | Photoshop, Clip Studio Paint | Basic sharpening commands. |
| Unsharp Mask | Enhances differences from a blurred image. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft | Radius, amount and threshold control the result. |
| Smart Sharpen | Applies controllable blur-aware sharpening. | Photoshop | Offers more targeted correction than basic Sharpen. |
| High Pass | Retains high-frequency detail around neutral gray. | Photoshop | Often blended with the source for sharpening. |
| Add Noise | Adds random brightness or colour variation. | Photoshop, Clip Studio Paint | Useful for grain, texture and breaking up smooth gradients. |
| Reduce Noise / Remove JPEG Noise | Suppresses noise or compression artifacts. | Photoshop, Clip Studio Paint | Clip Studio specifically offers Remove jpeg noise; Photoshop has broader noise controls. |
| Dust & Scratches / Remove Dust | Removes small isolated marks. | Photoshop, Clip Studio Paint | The two programs use different detection and cleanup controls. |
| Median / Despeckle | Suppresses isolated pixel variation. | Photoshop | Useful for cleaning scans and noisy flat regions. |
| Mosaic | Groups pixels into block-like cells. | Photoshop, Clip Studio Paint | Useful for stylization and obscuring detail. |
| Crystallize | Replaces image regions with irregular colour cells. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft | Clip Studio added its version in 4.0; distinguish this filter from vector deformation tools. |
| Displace | Offsets pixels according to another image. | Photoshop | Useful for mapping graphics onto surface texture. |
| Pinch | Contracts or expands pixels around a centre. | Photoshop, Clip Studio Paint | A localized radial distortion. |
| Ripple / Wave | Applies repeating spatial distortion. | Photoshop, Clip Studio Paint | Each program offers different wave and ripple implementations. |
| Twirl | Rotates pixels progressively around a centre. | Photoshop, Clip Studio Paint | Clip Studio also offers related spiral distortion controls. |
| Polar Coordinates | Converts between rectangular and polar layouts. | Photoshop | Useful for circular graphics and texture experiments. |
| Spherize / Curved Surface | Warps an image toward a curved surface appearance. | Photoshop, Clip Studio Paint | Clip Studio offers Curved surface; the parameterization differs. |
| ZigZag | Displaces pixels in concentric patterns. | Photoshop, Clip Studio Paint | Distinct from the vector Zig Zag path effect. |
| Lens Correction / Adaptive Wide Angle | Corrects photographic geometric distortion. | Photoshop | Targets lens curvature and perspective-related distortion. |
| Camera Raw Filter | Provides photographic colour, detail and lens controls. | Photoshop | A broad processing workspace within Photoshop. |
| Liquify Face-Aware controls | Detects and adjusts facial proportions. | Photoshop | A specialized subset of the Liquify workspace. |

## Vector effects and styling

Sources: [VC3], [AI1], [AI2], [PS1], [AD3], [CA3].

| Tool name | Behaviour | Programs present | Description |
| --- | --- | --- | --- |
| Drop / Outer Shadow | Renders an offset shadow from visible content. | Photoshop, Adobe Illustrator, VectorCraft, Affinity Designer, Canva | Canva exposes photo and text shadow variants. |
| Inner Shadow | Renders a shadow inside a boundary. | Photoshop, Affinity Designer | Suggests recessed content. |
| Inner / Outer Glow | Adds a soft halo near a boundary. | Photoshop, Adobe Illustrator, VectorCraft, Affinity Designer | Useful for lighting and separation from the background. |
| Bevel / Emboss | Shades edges to suggest raised or recessed relief. | Photoshop, Affinity Designer | A surface-style effect rather than full 3D geometry. |
| Colour / Gradient Overlay | Repaints layer content with a colour or gradient. | Photoshop, Affinity Designer | An editable layer-style operation. |
| Stroke / Outline effect | Adds an outline around visible content. | Photoshop, Affinity Designer, Canva | Canva outlines text through text effects; scope differs. |
| Feather | Softens an object's visible boundary. | Photoshop, Adobe Illustrator, VectorCraft | Useful for blending composited elements. |
| Roughen | Perturbs path contours. | Adobe Illustrator, VectorCraft | Creates irregular edges with adjustable character. |
| Zig Zag path effect | Alternates peaks and valleys along paths. | Adobe Illustrator, VectorCraft | Creates sawtooth or smooth wave outlines. |
| Pucker & Bloat effect | Pulls geometry inward or pushes it outward. | Adobe Illustrator, VectorCraft | Applies an adjustable vector distortion. |
| Twist / Tweak effect | Rotates or perturbs path geometry. | Adobe Illustrator, VectorCraft | Stylizes the object's vector contour. |
| Transform effect | Applies live transformations and repeated copies. | Adobe Illustrator, VectorCraft | Useful for repeated geometric constructions. |
| Warp presets | Bends artwork into predefined envelope forms. | Photoshop, Adobe Illustrator, VectorCraft, Affinity Designer | Available targets and presets differ; Photoshop also provides text warp. |
| Scribble | Replaces fill appearance with irregular strokes. | Adobe Illustrator, VectorCraft | Creates a hand-sketched vector appearance. |
| Convert to Shape effect | Encloses artwork in a live rectangle or ellipse. | Adobe Illustrator, VectorCraft | An appearance effect distinct from converting text to outlines. |
| SVG filter | Applies an SVG filter definition. | Adobe Illustrator, VectorCraft | Effects may rasterize or export differently depending on the filter chain. |
| 3D Extrude / Revolve / Rotate / Inflate | Builds a shaded 3D-style result from vector artwork. | Adobe Illustrator, VectorCraft | VectorCraft documents partial behavioural parity; not a general modelling replacement. |
| Crop Marks | Adds print-trimming reference marks. | Adobe Illustrator, VectorCraft | Used for production layout. |
| Rasterize effect | Renders vector content to pixels within an effect stack. | Adobe Illustrator, VectorCraft | Resolution influences subsequent raster effects. |

## Gallery filters: Artistic

Sources: [VC3], [AI2], [PS2].

| Tool name | Behaviour | Programs present | Description |
| --- | --- | --- | --- |
| Colored Pencil | Renders pencil-like coloured marks. | Photoshop, Adobe Illustrator, VectorCraft | Illustrative shading. VectorCraft is an independent approximation. |
| Cutout | Reduces imagery to flat colour regions. | Photoshop, Adobe Illustrator, VectorCraft | Poster-like simplification. VectorCraft is an independent approximation. |
| Dry Brush | Creates broken painted marks. | Photoshop, Adobe Illustrator, VectorCraft | Dry-media appearance. VectorCraft is an independent approximation. |
| Film Grain | Adds film-like texture. | Photoshop, Adobe Illustrator, VectorCraft | Photographic stylization. VectorCraft is an independent approximation. |
| Fresco | Creates coarse painted patches. | Photoshop, Adobe Illustrator, VectorCraft | Mural-like appearance. VectorCraft is an independent approximation. |
| Neon Glow | Adds coloured luminous shading. | Photoshop, Adobe Illustrator, VectorCraft | Stylized light treatment. VectorCraft is an independent approximation. |
| Paint Daubs | Groups detail into brush-like marks. | Photoshop, Adobe Illustrator, VectorCraft | Painted surface appearance. VectorCraft is an independent approximation. |
| Palette Knife | Flattens detail into broad painted facets. | Photoshop, Adobe Illustrator, VectorCraft | Impasto-style abstraction. VectorCraft is an independent approximation. |
| Plastic Wrap | Adds glossy relief-like highlights. | Photoshop, Adobe Illustrator, VectorCraft | Synthetic surface treatment. VectorCraft is an independent approximation. |
| Poster Edges | Combines flattened colour with dark edges. | Photoshop, Adobe Illustrator, VectorCraft | Graphic illustration effect. VectorCraft is an independent approximation. |
| Rough Pastels | Adds pastel-like strokes and texture. | Photoshop, Adobe Illustrator, VectorCraft | Chalky media appearance. VectorCraft is an independent approximation. |
| Smudge Stick | Smears detail into directional marks. | Photoshop, Adobe Illustrator, VectorCraft | Soft drawing treatment. VectorCraft is an independent approximation. |
| Sponge | Creates blotchy textured regions. | Photoshop, Adobe Illustrator, VectorCraft | Dabbed paint appearance. VectorCraft is an independent approximation. |
| Underpainting | Adds a textured painted foundation. | Photoshop, Adobe Illustrator, VectorCraft | Canvas-like treatment. VectorCraft is an independent approximation. |
| Watercolor | Simplifies detail into wash-like colour. | Photoshop, Adobe Illustrator, VectorCraft | Water-based media appearance. VectorCraft is an independent approximation. |

## Gallery filters: Brush Strokes

Sources: [VC3], [AI2], [PS2].

| Tool name | Behaviour | Programs present | Description |
| --- | --- | --- | --- |
| Accented Edges | Emphasizes boundaries. | Photoshop, Adobe Illustrator, VectorCraft | Graphic edge treatment. VectorCraft is an independent approximation. |
| Angled Strokes | Builds directional painted marks. | Photoshop, Adobe Illustrator, VectorCraft | Diagonal brush texture. VectorCraft is an independent approximation. |
| Crosshatch | Creates intersecting stroke patterns. | Photoshop, Adobe Illustrator, VectorCraft | Hatched shading. VectorCraft is an independent approximation. |
| Dark Strokes | Adds dark stroke-like shading. | Photoshop, Adobe Illustrator, VectorCraft | High-contrast painted texture. VectorCraft is an independent approximation. |
| Ink Outlines | Emphasizes ink-like contours. | Photoshop, Adobe Illustrator, VectorCraft | Illustrative line treatment. VectorCraft is an independent approximation. |
| Spatter | Creates sprayed patches. | Photoshop, Adobe Illustrator, VectorCraft | Airbrush-like texture. VectorCraft is an independent approximation. |
| Sprayed Strokes | Applies directional sprayed marks. | Photoshop, Adobe Illustrator, VectorCraft | Directional texture. VectorCraft is an independent approximation. |
| Sumi-e | Creates ink-wash-like contrast. | Photoshop, Adobe Illustrator, VectorCraft | Brush-and-ink appearance. VectorCraft is an independent approximation. |

## Gallery filters: Distort

Sources: [VC3], [AI2], [PS2].

| Tool name | Behaviour | Programs present | Description |
| --- | --- | --- | --- |
| Diffuse Glow | Spreads light into a grainy glow. | Photoshop, Adobe Illustrator, VectorCraft | Dreamlike highlights. VectorCraft is an independent approximation. |
| Glass | Displaces imagery through a glass-like surface. | Photoshop, Adobe Illustrator, VectorCraft | Refractive distortion. VectorCraft is an independent approximation. |
| Ocean Ripple | Applies small irregular ripples. | Photoshop, Adobe Illustrator, VectorCraft | Water-like distortion. VectorCraft is an independent approximation. |

## Gallery filters: Pixelate

Sources: [VC3], [AI2], [PS2].

| Tool name | Behaviour | Programs present | Description |
| --- | --- | --- | --- |
| Color Halftone | Converts tones into screened dots. | Photoshop, Adobe Illustrator, VectorCraft | Print-screen appearance. VectorCraft is an independent approximation. |
| Mezzotint | Replaces tones with patterned marks. | Photoshop, Adobe Illustrator, VectorCraft | Engraving-like texture. VectorCraft is an independent approximation. |
| Pointillize | Reconstructs colour with discrete dots. | Photoshop, Adobe Illustrator, VectorCraft | Pointillist appearance. VectorCraft is an independent approximation. |

## Gallery filters: Sketch

Sources: [VC3], [AI2], [PS2].

| Tool name | Behaviour | Programs present | Description |
| --- | --- | --- | --- |
| Bas Relief | Simulates shallow sculpted shading. | Photoshop, Adobe Illustrator, VectorCraft | Relief treatment. VectorCraft is an independent approximation. |
| Chalk & Charcoal | Combines light and dark drawing marks. | Photoshop, Adobe Illustrator, VectorCraft | Dry-media contrast. VectorCraft is an independent approximation. |
| Charcoal | Creates charcoal-like shading. | Photoshop, Adobe Illustrator, VectorCraft | Monochrome drawing effect. VectorCraft is an independent approximation. |
| Chrome | Creates reflective metallic shading. | Photoshop, Adobe Illustrator, VectorCraft | Metallic abstraction. VectorCraft is an independent approximation. |
| Conte Crayon | Creates textured crayon marks. | Photoshop, Adobe Illustrator, VectorCraft | Traditional drawing appearance. VectorCraft is an independent approximation. |
| Graphic Pen | Represents tones with pen strokes. | Photoshop, Adobe Illustrator, VectorCraft | Hatched illustration. VectorCraft is an independent approximation. |
| Halftone Pattern | Maps tone to a regular pattern. | Photoshop, Adobe Illustrator, VectorCraft | Screened texture. VectorCraft is an independent approximation. |
| Note Paper | Creates paper-like relief and grain. | Photoshop, Adobe Illustrator, VectorCraft | Textured paper treatment. VectorCraft is an independent approximation. |
| Photocopy | Emphasizes high-contrast copied contours. | Photoshop, Adobe Illustrator, VectorCraft | Copier-like appearance. VectorCraft is an independent approximation. |
| Plaster | Simulates a moulded surface. | Photoshop, Adobe Illustrator, VectorCraft | Relief abstraction. VectorCraft is an independent approximation. |
| Reticulation | Breaks tone into irregular grain. | Photoshop, Adobe Illustrator, VectorCraft | Photochemical texture. VectorCraft is an independent approximation. |
| Stamp | Simplifies content into bold regions. | Photoshop, Adobe Illustrator, VectorCraft | Rubber-stamp appearance. VectorCraft is an independent approximation. |
| Torn Edges | Creates irregular high-contrast boundaries. | Photoshop, Adobe Illustrator, VectorCraft | Torn-paper treatment. VectorCraft is an independent approximation. |
| Water Paper | Creates softened fibrous colour marks. | Photoshop, Adobe Illustrator, VectorCraft | Wet-paper appearance. VectorCraft is an independent approximation. |

## Gallery filters: Stylize

Sources: [VC3], [AI2], [PS2].

| Tool name | Behaviour | Programs present | Description |
| --- | --- | --- | --- |
| Glowing Edges | Highlights detected edges with luminous colour. | Photoshop, Adobe Illustrator, VectorCraft | Neon contour effect. VectorCraft is an independent approximation. |

## Gallery filters: Texture

Sources: [VC3], [AI2], [PS2].

| Tool name | Behaviour | Programs present | Description |
| --- | --- | --- | --- |
| Craquelure | Adds fine crack patterns. | Photoshop, Adobe Illustrator, VectorCraft | Aged paint surface. VectorCraft is an independent approximation. |
| Grain | Adds controllable granular texture. | Photoshop, Adobe Illustrator, VectorCraft | Surface roughness. VectorCraft is an independent approximation. |
| Mosaic Tiles | Builds tile-like regions and boundaries. | Photoshop, Adobe Illustrator, VectorCraft | Decorative tile effect. VectorCraft is an independent approximation. |
| Patchwork | Forms raised block-like patches. | Photoshop, Adobe Illustrator, VectorCraft | Quilt-like abstraction. VectorCraft is an independent approximation. |
| Stained Glass | Divides colours with pane-like boundaries. | Photoshop, Adobe Illustrator, VectorCraft | Glass-panel appearance. VectorCraft is an independent approximation. |
| Texturizer | Applies a textured surface impression. | Photoshop, Adobe Illustrator, VectorCraft | Canvas and related material treatments. VectorCraft is an independent approximation. |

## Additional raster, illustration and comic effects

Sources: [CS1], [CS2], [CS6], [PS2], [VC3].

| Tool name | Behaviour | Programs present | Description |
| --- | --- | --- | --- |
| Clouds / Difference Clouds / Perlin Noise | Generates procedural cloudy texture. | Photoshop, Clip Studio Paint | Clip Studio uses Perlin noise; Photoshop provides Clouds variants. |
| Lens Flare | Adds a simulated optical flare. | Photoshop, Adobe Illustrator, VectorCraft | Photoshop uses a raster filter; Illustrator and VectorCraft use a vector Flare tool. |
| Fibers | Generates streaked fibrous texture. | Photoshop | Useful for material backgrounds. |
| Emboss filter | Converts contrast into directional relief shading. | Photoshop | Different from the editable Bevel and Emboss layer style. |
| Find Edges / Trace Contour | Extracts boundaries from image contrast. | Photoshop | Useful for stylization and mask construction. |
| Oil Paint | Creates flowing painted strokes. | Photoshop | A dedicated filter beyond the traditional Filter Gallery. |
| Offset | Shifts image coordinates with edge handling. | Photoshop | Useful for inspecting texture seams. |
| Minimum / Maximum | Expands dark or light regions. | Photoshop | Useful for mask erosion and dilation. |
| Custom convolution | Applies a user-defined filter matrix. | Photoshop | Allows custom linear pixel operations. |
| De-Interlace / NTSC Colors | Processes legacy video-oriented imagery. | Photoshop, Adobe Illustrator, VectorCraft | Availability depends on document mode and current menu configuration. |
| Artistic / line extraction | Simplifies colour or extracts drawn-looking outlines. | Clip Studio Paint | Clip Studio's Artistic filter offers colour and line processing. |
| Pencil Drawing | Creates pencil-style contours and hatching. | Clip Studio Paint | Introduced in version 3.0. |
| Retro Film | Combines vintage colour and optical treatments. | Clip Studio Paint | Introduced in version 3.0. |
| Chromatic Aberration | Offsets colour channels spatially. | Clip Studio Paint | Introduced in version 3.0. |
| Normal Map | Converts an image into surface-normal texture data. | Clip Studio Paint | Useful for 3D material workflows; not listed as a current Photoshop 3D feature. |
| Quick Filters / Quick Effects | Applies preset colour or effect treatments. | Clip Studio Paint | Groups common adjustments in a preview interface. |
| Remove Dust tool | Cleans isolated marks using a targeted tool. | Clip Studio Paint | Useful for scanned linework. |
| Rulers / Perspective Ruler / Symmetry Ruler | Constrains brush strokes to constructed guides. | Clip Studio Paint | Provides drawing assistance rather than permanent image distortion. |
| Frame Border | Creates and divides comic panels. | Clip Studio Paint | Dedicated comic-production tools. |
| Balloon / Balloon Tail | Creates editable speech balloons and tails. | Clip Studio Paint | Supports comic lettering workflows. |
| Speed Lines / Focus Lines | Generates parallel or converging action lines. | Clip Studio Paint | Creates editable comic effects. |
| Screentone / layer tone | Converts shading to printable patterns. | Clip Studio Paint | A comic layer effect rather than the same operation as Color Halftone. |
| LT conversion | Separates image or 3D content into lines and tones. | Clip Studio Paint | EX-specific production feature. |
| 3D reference manipulation | Positions models, poses and cameras for drawing reference. | Clip Studio Paint | Supports illustration construction rather than general mesh modelling. |

## Canva-specific effects and assisted editing

Sources: [CA1], [CA2], [CA3], [CA4], [PS1].

| Tool name | Behaviour | Programs present | Description |
| --- | --- | --- | --- |
| Photo filters | Applies a preset photographic colour treatment. | Canva | Preset names and collections can change. |
| Duotone | Maps an image into a two-colour treatment. | Canva, VectorCraft | Canva provides this in Filters; VectorCraft offers a live colour effect. |
| Blur / Auto Focus | Softens images or emphasizes the subject. | Canva | Different controls from Gaussian Blur and optical blur workspaces. |
| Face Retouch | Applies portrait cleanup. | Canva | A simplified photographic retouching tool. |
| Shadow / Lift / Hollow / Splice / Echo / Glitch / Neon | Applies named text appearance effects. | Canva | Each preset has its own available controls. |
| Curve text | Bends a text line into a curved arrangement. | Canva | Does not provide a general editable Bezier text path. |
| Background text effect | Adds a coloured backing behind text. | Canva | Useful for labels and readable overlays. |
| BG Remover / Remove Background | Detects a subject and removes the surrounding background. | Photoshop, Canva | Access and automation differ; Canva availability is plan-dependent. |
| Magic Eraser / Remove Tool | Removes unwanted image content. | Photoshop, Canva | Photoshop's Remove Tool and Canva Magic Eraser are separate implementations. |
| Generative Fill / Magic Edit | Replaces or adds content using generative editing. | Photoshop, Canva | Cloud access, credits and plan restrictions may apply. |
| Generative Expand / Magic Expand | Creates image content beyond existing boundaries. | Photoshop, Canva | Useful for changing composition or aspect ratio. |
| Magic Grab | Separates a photo subject for repositioning. | Canva | An assisted extraction workflow. |
| Grab Text | Detects image text for editing. | Canva | Different from manually adding a text box. |
| Magic Layers | Separates image content into editable elements. | Canva | Availability depends on rollout and plan. |
| BG Generator | Generates a new photographic background. | Canva | An assisted photo-editing feature. |
| Upscale | Increases image dimensions using enhancement processing. | Photoshop, Canva | Specific models, limits and subscription requirements vary. |
| Style Match | Applies matching visual styling to supported content. | Canva | An assisted styling operation. |
| Magic Resize / Magic Switch | Adapts a design to another size or format. | Canva | May require a paid plan. |
| Frames / grids | Places images into predefined layout containers. | Canva | A simplified composition and clipping workflow. |
| Charts | Creates data-driven visual charts. | Adobe Illustrator, VectorCraft, Canva | Illustrator and VectorCraft include dedicated graph tools; Canva provides chart elements. |

## Additional current assisted and raster effects

Sources: [CS7], [AI3].

| Tool name | Behaviour | Programs present | Description |
| --- | --- | --- | --- |
| Glow filter | Adds luminous bloom to image content. | Clip Studio Paint | Added in Clip Studio Paint 5.1; distinct from manually painted glow. |
| Inflate filter | Creates a raised, inflated surface appearance. | Clip Studio Paint | Added in Clip Studio Paint 5.1. |
| Drop Shadow filter | Generates a shadow from layer content. | Clip Studio Paint | Added in Clip Studio Paint 5.1. |
| Text to Vector Graphic | Generates vector artwork from a prompt. | Adobe Illustrator | Cloud generation and usage limits apply. |
| Text to Pattern | Generates repeating patterns from a prompt. | Adobe Illustrator | A generative alternative to manual pattern construction. |
| Generative Shape Fill | Generates content within a selected shape. | Adobe Illustrator | Output and controls depend on the selected model. |
| Generative Recolor | Recolours artwork using a text prompt. | Adobe Illustrator | Produces palette alternatives for existing artwork. |
| Prompt to Edit | Changes generated artwork through text instructions. | Adobe Illustrator | Current documented feature; model and content restrictions apply. |

## Layer blend modes: individual operations

Sources: [BL1], [BL2], [BL3], [BL4], [BL5].

| Tool name | Behaviour | Programs present | Description |
| --- | --- | --- | --- |
| Normal blend mode | Composites foreground colour using its alpha. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Baseline layer compositing. |
| Dissolve blend mode | Converts partial coverage into scattered opaque pixels. | Photoshop | Produces a stippled fade. |
| Darken blend mode | Keeps the lower value per colour channel. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Useful for combining dark marks. |
| Multiply blend mode | Multiplies normalized colour channels. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | White is neutral; useful for shadows. |
| Color Burn blend mode | Darkens through a nonlinear channel operation. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Strong tonal contrast. |
| Linear Burn blend mode | Adds channel values and subtracts one. | Photoshop, Clip Studio Paint, Affinity Designer | Clamping and colour space affect the result. |
| Darker Color blend mode | Chooses the darker complete colour. | Photoshop, Clip Studio Paint, Affinity Designer | Unlike Darken, does not choose channels independently. |
| Lighten blend mode | Keeps the higher value per channel. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Useful for combining light marks. |
| Screen blend mode | Inverts, multiplies and reinverts channel values. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Black is neutral; useful for light overlays. |
| Color Dodge blend mode | Brightens through a nonlinear channel operation. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Can produce intense highlights. |
| Linear Dodge / Add blend mode | Adds channel values. | Photoshop, Clip Studio Paint, Affinity Designer | Photoshop calls this Linear Dodge (Add). |
| Glow Dodge blend mode | Applies Clip Studio's enhanced dodge variant. | Clip Studio Paint | Useful for painted light. |
| Add (Glow) blend mode | Applies Clip Studio's enhanced additive variant. | Clip Studio Paint | Distinct from Add and layer glow filters. |
| Lighter Color blend mode | Chooses the lighter complete colour. | Photoshop, Clip Studio Paint, Affinity Designer | Unlike Lighten, compares whole colours. |
| Overlay blend mode | Switches between darkening and lightening by base tone. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Useful for texture overlays. |
| Soft Light blend mode | Applies gentle tone-dependent contrast. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Algorithms can differ between applications. |
| Hard Light blend mode | Switches between Multiply and Screen by source tone. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Stronger contrast than Soft Light. |
| Vivid Light blend mode | Combines burn and dodge operations. | Photoshop, Clip Studio Paint, Affinity Designer | Strong contrast around neutral gray. |
| Linear Light blend mode | Applies a signed linear brightness change. | Photoshop, Clip Studio Paint, Affinity Designer | Useful for detail compositing. |
| Pin Light blend mode | Combines conditional darken and lighten operations. | Photoshop, Clip Studio Paint, Affinity Designer | Can replace intermediate tones. |
| Hard Mix blend mode | Produces strongly quantized channel colours. | Photoshop, Clip Studio Paint, Affinity Designer | Fill opacity can substantially change its appearance. |
| Difference blend mode | Calculates absolute channel differences. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Identical aligned pixels become black. |
| Exclusion blend mode | Uses a softer difference-like operation. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Useful for stylized inversion. |
| Subtract blend mode | Subtracts source values from base values. | Photoshop, Clip Studio Paint, Affinity Designer | Different from Boolean shape subtraction. |
| Divide blend mode | Divides base values by source values. | Photoshop, Clip Studio Paint, Affinity Designer | Useful for tonal correction experiments. |
| Hue blend mode | Transfers hue while retaining base saturation and luminance. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Changes colour family. |
| Saturation blend mode | Transfers saturation while retaining other base components. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Changes colour intensity. |
| Color blend mode | Transfers hue and saturation while preserving base luminance. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Useful for colouring grayscale work. |
| Luminosity / Brightness blend mode | Transfers luminance while preserving base colour components. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Clip Studio names its variant Brightness. |
| Average blend mode | Averages source and destination values. | Affinity Designer | Different from blurring an image to its average colour. |
| Negation blend mode | Applies a sum-based inversion operation. | Affinity Designer | Stylized compositing. |
| Reflect blend mode | Applies a nonlinear reflect operation. | Affinity Designer | Useful for reflective-looking highlights. |
| Glow blend mode | Applies the reversed-input counterpart of Reflect. | Affinity Designer | Not the same as Glow Dodge or a halo filter. |
| Contrast Negate blend mode | Inverts according to source and destination contrast. | Affinity Designer | Useful for graphic tonal effects. |
| Erase layer blend mode | Uses layer coverage to remove underlying visibility. | Affinity Designer | Provides editable erasing through a separate layer. |
| Pass Through / Through / Passthrough | Allows grouped children to interact with layers outside their group. | Photoshop, Clip Studio Paint, Affinity Designer | Group compositing behaviour; not a colour formula. |

## Opacity, transparency, masks and advanced compositing

Sources: [BL4], [BL6], [BL7], [BL8], [BL9], [BL10], [BL11].

| Tool name | Behaviour | Programs present | Description |
| --- | --- | --- | --- |
| Layer / object opacity control | Scales the contribution of a complete layer or object. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer, Canva | Canva calls its control Transparency. |
| Group opacity | Fades a collection through its parent container. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Isolated and pass-through groups can behave differently. |
| Fill opacity control | Changes fill contribution independently of overall layer opacity. | Photoshop, Affinity Designer | Photoshop can retain visible styles at zero Fill; Affinity exposes Fill Opacity in Blend Options. |
| Fill / stroke appearance opacity | Sets transparency for an individual fill or stroke. | Adobe Illustrator, VectorCraft, Affinity Designer | Does not necessarily fade the object's other appearance components. |
| Gradient-stop opacity | Assigns different alpha values to gradient stops. | Photoshop, Clip Studio Paint, Adobe Illustrator, VectorCraft, Affinity Designer | Creates spatially varying transparency. |
| Blend If | Makes layer visibility depend on source or backdrop channel values. | Photoshop | Split slider handles soften the transition. |
| Source / underlying blend ranges | Maps tonal values to transparency through curves. | Affinity Designer | A tonal masking control comparable in purpose to Blend If. |
| Per-channel blend ranges | Uses a chosen colour channel to control tonal blending. | Affinity Designer | Allows colour-dependent transparency. |
| Blend gamma | Changes the transfer curve used for layer blending. | Affinity Designer | RGB blending control; separate from brush Transfer dynamics. |
| Antialias coverage map | Remaps partially covered edge pixels. | Affinity Designer | Controls edge compositing and antialias inheritance. |
| Advanced blending channel exclusion | Disables selected channels in layer blending. | Photoshop | Useful for channel-specific composites. |
| Shallow / Deep knockout | Punches through a group or toward the background. | Photoshop | Controlled through advanced layer blending. |
| Blend Clipped Layers As Group | Controls how a clipping stack receives the base blend mode. | Photoshop | Changes interactions within clipped content. |
| Blend Interior Effects As Group | Applies layer blending to internal style effects together. | Photoshop | Affects effects such as inner glow and colour overlay. |
| Transparency Shapes Layer | Controls whether existing transparency bounds layer effects. | Photoshop | Useful for specialized style compositing. |
| Layer / Vector Mask Hides Effects | Controls whether a mask also hides layer styles. | Photoshop | Separates masking of content and effects. |
| Isolate Blending | Restricts child blending to a group or layer. | Adobe Illustrator, VectorCraft | Prevents blend modes interacting with outside artwork. |
| Knockout Group | Controls whether overlapping group members obscure one another. | Adobe Illustrator, VectorCraft | Different from deleting underlying geometry. |
| Opacity and Mask Define Knockout Shape | Uses transparency when determining a knockout boundary. | Adobe Illustrator, VectorCraft | Refines knockout behaviour. |
| Opacity mask controls | Uses mask luminance to control artwork visibility. | Adobe Illustrator, VectorCraft | Includes invert, clip, disable and unlink options. |
| Alpha lock / Lock Transparent Pixels | Restricts painting to existing coverage. | Photoshop, Clip Studio Paint, Affinity Designer | Useful for recolouring without expanding a silhouette. |
| Clip to Layer Below / clipping stack | Uses a lower layer's coverage to constrain upper layers. | Photoshop, Clip Studio Paint, Affinity Designer | Useful for shading a flat-colour base. |
| Mask density / feather | Weakens or softens a mask without repainting it. | Photoshop | Controls mask strength and edge softness separately. |
| Apply Image | Blends image or channel content into a target. | Photoshop | Supports channel-based masking and compositing. |
| Calculations | Combines channels into a new channel, selection or document. | Photoshop | Useful for constructing detailed masks. |
| Fade command | Changes the strength and mode of the last supported edit. | Photoshop | Useful immediately after a filter or paint operation. |
| Smart Filter blending options | Assigns opacity and a blend mode to a Smart Filter. | Photoshop | Keeps filter compositing editable. |
| Effect-specific opacity / blend mode | Controls compositing of individual effects. | Photoshop, Adobe Illustrator, VectorCraft, Affinity Designer | Supported controls depend on the effect. |

## Brush transfer, flow and colour mixing

Sources: [BL1], [BL2], [BL12], [BL13], [BL14], [BL15], [BL16].

| Tool name | Behaviour | Programs present | Description |
| --- | --- | --- | --- |
| Brush opacity / accumulation | Controls the opacity of deposited paint. | Photoshop, Clip Studio Paint, Affinity Designer, Canva | Affinity uses Accumulation; Canva Draw provides a transparency setting. |
| Brush Flow | Controls the rate at which paint builds up. | Photoshop, Affinity Designer | Separate from the overall stroke-opacity limit. |
| Transfer dynamics | Varies brush opacity and flow along a stroke. | Photoshop | Photoshop Brush Settings category. |
| Opacity jitter / pressure response | Varies stroke opacity using randomness or input dynamics. | Photoshop, Clip Studio Paint, Affinity Designer | Available controllers and minimum values differ. |
| Flow jitter / pressure response | Varies deposition rate along a stroke. | Photoshop, Affinity Designer | Useful for controlled buildup with a stylus. |
| Brush blending mode | Blends new marks with existing pixels on the same layer. | Photoshop, Clip Studio Paint, Affinity Designer | Separate from the layer's blend mode. |
| Behind / Background painting mode | Deposits new paint behind existing coverage. | Photoshop, Clip Studio Paint | A painting mode, not a layer stacking command. |
| Clear / Erase painting mode | Removes coverage using the painting tool. | Photoshop, Clip Studio Paint | Uses brush geometry to erase. |
| Black Burn tool mode | Darkens existing painted areas. | Clip Studio Paint | A tool-only mode. |
| White Burn tool mode | Lightens existing painted areas. | Clip Studio Paint | A tool-only mode. |
| Erase (Compare) tool mode | Erases according to an opacity comparison. | Clip Studio Paint | Different from ordinary erasing. |
| Compare Density tool mode | Paints only where the new mark has greater opacity. | Clip Studio Paint | Useful for controlled accumulation. |
| Replace Alpha tool mode | Replaces coverage with the tool's opacity. | Clip Studio Paint | Directly changes painted transparency. |
| Colour mixing method | Chooses how fresh colour mixes with existing pigment. | Clip Studio Paint | Blend, Running color and Smear have different behaviour. |
| Amount of Paint | Controls the fresh-colour contribution during mixing. | Clip Studio Paint | A brush-material control rather than layer opacity. |
| Density of Paint | Controls the density of paint deposited while mixing. | Clip Studio Paint | Can respond to stylus dynamics. |
| Color Stretch | Controls how far existing colour is dragged. | Clip Studio Paint | Useful for transitions and wet-looking strokes. |
| Mixer Brush Wet / Load / Mix | Controls pickup, paint supply and source mixing. | Photoshop | Simulates paint interaction during a stroke. |
| Mixer Brush clean / load | Clears or replenishes the brush reservoir. | Photoshop | Changes the colour carried into the next stroke. |
| Wet Edges | Accumulates stronger paint near stroke boundaries. | Photoshop, Affinity Designer | Creates watercolor-like edges without changing the layer blend mode. |
| Brush texture blending | Combines brush coverage with a texture. | Photoshop, Clip Studio Paint, Affinity Designer | Texture interaction changes the deposited mark. |

## Sources

Official vendor documentation and the VectorCraft project were checked on the research date. These are living documents.

[VC4]: https://craft.wiki/apps/vectorcraft/adjust-colors-with-effects/
- **VC4** — [VectorCraft live colour effects](https://craft.wiki/apps/vectorcraft/adjust-colors-with-effects/)
[CS7]: https://support.clip-studio.com/en-us/faq/articles/20240017
- **CS7** — [Clip Studio new filter availability](https://support.clip-studio.com/en-us/faq/articles/20240017)
[AI3]: https://helpx.adobe.com/illustrator/desktop/new-features/whats-new.html
- **AI3** — [Illustrator current features](https://helpx.adobe.com/illustrator/desktop/new-features/whats-new.html)
[PS1]: https://helpx.adobe.com/photoshop/desktop.html
- **PS1** — [Adobe Photoshop desktop help and tool index](https://helpx.adobe.com/photoshop/desktop.html)
[PS2]: https://helpx.adobe.com/photoshop/desktop/effects-filters/blur-sharpen-filters/filter-effects-reference.html
- **PS2** — [Photoshop filter effects reference](https://helpx.adobe.com/photoshop/desktop/effects-filters/blur-sharpen-filters/filter-effects-reference.html)
[AI1]: https://helpx.adobe.com/illustrator/desktop/get-started/learn-the-basics/illustrator-tools-reference.html
- **AI1** — [Illustrator tools reference](https://helpx.adobe.com/illustrator/desktop/get-started/learn-the-basics/illustrator-tools-reference.html)
[AI2]: https://helpx.adobe.com/illustrator/desktop/special-effects-styles/apply-filter-effects/raster-effects-reference.html
- **AI2** — [Illustrator raster effects reference](https://helpx.adobe.com/illustrator/desktop/special-effects-styles/apply-filter-effects/raster-effects-reference.html)
[VC1]: https://craft.wiki/apps/vectorcraft/tools-reference/
- **VC1** — [VectorCraft tools reference](https://craft.wiki/apps/vectorcraft/tools-reference/)
[VC2]: https://craft.wiki/apps/vectorcraft/overview/
- **VC2** — [VectorCraft overview and workflow documentation](https://craft.wiki/apps/vectorcraft/overview/)
[VC3]: https://github.com/storytold/vectorcraft/blob/main/docs/effects-parity.md
- **VC3** — [VectorCraft implemented effects and limitations, repository documentation](https://github.com/storytold/vectorcraft/blob/main/docs/effects-parity.md)
[AD1]: https://affinity.help/designer2/en-US.lproj/index.html
- **AD1** — [Affinity Designer 2 help, tools and feature index](https://affinity.help/designer2/en-US.lproj/index.html)
[AD2]: https://affinity.help/designer2/en-US.lproj/pages/Layers/adjustmentLayers.html
- **AD2** — [Affinity Designer adjustment layers](https://affinity.help/designer2/en-US.lproj/pages/Layers/adjustmentLayers.html)
[AD3]: https://affinity.help/designer2/en-US.lproj/pages/LayerFX/create_layerFX.html
- **AD3** — [Affinity Designer layer effects](https://affinity.help/designer2/en-US.lproj/pages/LayerFX/create_layerFX.html)
[CS1]: https://www.clipstudio.net/en/functional_list/
- **CS1** — [Clip Studio Paint detailed feature list and edition comparison](https://www.clipstudio.net/en/functional_list/)
[CS2]: https://help.clip-studio.com/en-us/manual_en/180_layers/Vector_layers.htm
- **CS2** — [Clip Studio vector layers and correction tools](https://help.clip-studio.com/en-us/manual_en/180_layers/Vector_layers.htm)
[CS3]: https://help.clip-studio.com/en-us/manual_en/450_figure/Complex_lines.htm
- **CS3** — [Clip Studio complex lines and Bezier curves](https://help.clip-studio.com/en-us/manual_en/450_figure/Complex_lines.htm)
[CS4]: https://help.clip-studio.com/en-us/manual_en/240_brushes/Drawing_and_painting.htm
- **CS4** — [Clip Studio drawing and painting](https://help.clip-studio.com/en-us/manual_en/240_brushes/Drawing_and_painting.htm)
[CS5]: https://help.clip-studio.com/en-us/manual_en/390_filters/Tonal_Correction_Effects.htm
- **CS5** — [Clip Studio tonal correction effects](https://help.clip-studio.com/en-us/manual_en/390_filters/Tonal_Correction_Effects.htm)
[CS6]: https://help.clip-studio.com/en-us/manual_en/390_filters/Filters.htm
- **CS6** — [Clip Studio filter reference](https://help.clip-studio.com/en-us/manual_en/390_filters/Filters.htm)
[CA1]: https://www.canva.com/help/image-editor/
- **CA1** — [Canva photo editor tools, effects and current locations](https://www.canva.com/help/image-editor/)
[CA2]: https://www.canva.com/help/draw-in-canva/
- **CA2** — [Canva drawing and Shape Assist](https://www.canva.com/help/draw-in-canva/)
[CA3]: https://www.canva.com/help/text-effects/
- **CA3** — [Canva text effects](https://www.canva.com/help/text-effects/)
[CA4]: https://www.canva.com/help/creating-using-gradients/
- **CA4** — [Canva gradient controls](https://www.canva.com/help/creating-using-gradients/)
[BL1]: https://helpx.adobe.com/photoshop/desktop/repair-retouch/adjust-light-tone/blending-mode-descriptions.html
- **BL1** — [Photoshop blend modes](https://helpx.adobe.com/photoshop/desktop/repair-retouch/adjust-light-tone/blending-mode-descriptions.html)
[BL2]: https://help.clip-studio.com/en-us/manual_en/180_layers/Blending_modes.htm
- **BL2** — [Clip Studio layer and tool blend modes](https://help.clip-studio.com/en-us/manual_en/180_layers/Blending_modes.htm)
[BL3]: https://helpx.adobe.com/illustrator/desktop/manage-colors/apply-transparency-and-blending/blending-modes-in-illustrator.html
- **BL3** — [Illustrator blend modes](https://helpx.adobe.com/illustrator/desktop/manage-colors/apply-transparency-and-blending/blending-modes-in-illustrator.html)
[BL4]: https://craft.wiki/apps/vectorcraft/fill-stroke-and-color/
- **BL4** — [VectorCraft transparency and blend controls](https://craft.wiki/apps/vectorcraft/fill-stroke-and-color/)
[BL5]: https://affinity.help/designer2/en-US.lproj/pages/Layers/layerBlendModes.html
- **BL5** — [Affinity Designer blend modes](https://affinity.help/designer2/en-US.lproj/pages/Layers/layerBlendModes.html)
[BL6]: https://helpx.adobe.com/photoshop/desktop/create-manage-layers/apply-layer-effects/set-layer-opacity-and-blending-modes.html
- **BL6** — [Photoshop opacity and Fill](https://helpx.adobe.com/photoshop/desktop/create-manage-layers/apply-layer-effects/set-layer-opacity-and-blending-modes.html)
[BL7]: https://helpx.adobe.com/photoshop/desktop/create-manage-layers/apply-layer-effects/use-advanced-blending-options.html
- **BL7** — [Photoshop advanced blending](https://helpx.adobe.com/photoshop/desktop/create-manage-layers/apply-layer-effects/use-advanced-blending-options.html)
[BL8]: https://affinity.help/designer2/en-US.lproj/pages/Layers/layerBlendRanges.html
- **BL8** — [Affinity Designer blend ranges](https://affinity.help/designer2/en-US.lproj/pages/Layers/layerBlendRanges.html)
[BL9]: https://helpx.adobe.com/illustrator/desktop/manage-colors/apply-transparency-and-blending/apply-transparency-to-artwork.html
- **BL9** — [Illustrator transparency controls](https://helpx.adobe.com/illustrator/desktop/manage-colors/apply-transparency-and-blending/apply-transparency-to-artwork.html)
[BL10]: https://helpx.adobe.com/photoshop/desktop/adjust-color/hdr-and-channel-manipulation/blend-channels-using-calculations.html
- **BL10** — [Photoshop Apply Image and Calculations](https://helpx.adobe.com/photoshop/desktop/adjust-color/hdr-and-channel-manipulation/blend-channels-using-calculations.html)
[BL11]: https://www.canva.com/en_gb/help/transparency/
- **BL11** — [Canva transparency](https://www.canva.com/en_gb/help/transparency/)
[BL12]: https://helpx.adobe.com/photoshop/desktop/apply-painting-techniques/fill-objects-selections-layers/brush-dynamics-settings.html
- **BL12** — [Photoshop Transfer dynamics](https://helpx.adobe.com/photoshop/desktop/apply-painting-techniques/fill-objects-selections-layers/brush-dynamics-settings.html)
[BL13]: https://affinity.help/designer2/en-US.lproj/pages/Painting/pixel_modify.html
- **BL13** — [Affinity Designer pixel brush controls](https://affinity.help/designer2/en-US.lproj/pages/Painting/pixel_modify.html)
[BL14]: https://help.clip-studio.com/en-us/manual_en/810_subtools/I.htm
- **BL14** — [Clip Studio Ink settings](https://help.clip-studio.com/en-us/manual_en/810_subtools/I.htm)
[BL15]: https://help.clip-studio.com/en-us/manual_en/240_brushes/Blending_tools.htm
- **BL15** — [Clip Studio blending brushes](https://help.clip-studio.com/en-us/manual_en/240_brushes/Blending_tools.htm)
[BL16]: https://helpx.adobe.com/photoshop/desktop/apply-painting-techniques/brushes-presets/paint-with-brush-or-pencil-tool.html
- **BL16** — [Photoshop brush flow and opacity](https://helpx.adobe.com/photoshop/desktop/apply-painting-techniques/brushes-presets/paint-with-brush-or-pencil-tool.html)
