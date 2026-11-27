# Planned

The tools, filters and blending features of [graphic_design_tools_and_filters.md](Standardsdocs/Toolstandards/graphic_design_tools_and_filters.md) that the app does not offer today (compared with [featurelist.md](featurelist.md)). Where a row is partly in the app, only the missing part is listed.

**Engine-ready**: the inherited engine already does it (VectorCraft lists it and it was not removed); the app has no UI for it yet. **New**: the engine has nothing for it.

## Selection, navigation and layout

| Feature | Behaviour | Engine |
| --- | --- | --- |
| Direct Selection / Node / Control Point | Edits individual points and curve handles. | Engine-ready |
| Group Selection | Selects progressively larger nested groups. | Engine-ready |
| Rectangular / elliptical marquee | Creates a geometric pixel selection. | New |
| Single-row / single-column marquee | Selects one pixel row or column. | New |
| Lasso / polygonal selection | Defines a selection using a drawn outline. | New |
| Vector lasso | Selects anchors inside a drawn loop. | Engine-ready |
| Magnetic Lasso | Follows contrast boundaries while selecting. | New |
| Magic Wand / Auto Select / Flood Select | Selects pixels similar to a sampled colour. | New |
| Vector Magic Wand | Selects objects with matching appearance attributes. | Engine-ready |
| Selection Brush / Selection Pen | Paints a selection mask. | New |
| Object Selection / Select Subject | Detects image subjects automatically. | New |
| Selection refinement | Adjusts selection edges and softness. | New |
| Artboard tool (several artboards) | Creates and edits independent design areas; the app makes one artboard per finite document and Crop resizes it. | Engine-ready |
| Page management | Adds and reorders document pages. | New |
| Align / distribute | Aligns bounds or equalizes spacing. | Engine-ready |
| Guides, grids and snapping to art | Constrains placement to reference geometry; the app has only the Export tool's dot grid. | Engine-ready |
| Measure | Measures distances and angles. | Engine-ready |
| Dimension | Creates dimension annotations linked to artwork. | Engine-ready |
| Area Tool | Reports enclosed shape area. | New |
| Print Tiling | Positions artwork relative to printable page tiles. | New (removed from the engine with the print functions) |

## Vector drawing and path construction

| Feature | Behaviour | Engine |
| --- | --- | --- |
| Curvature Pen | Fits smooth segments through placed points. | Engine-ready |
| Freeform Pen | Converts a drawn gesture into a path. | New |
| Vector Pencil | Fits an editable path to a freehand stroke, without a brush. | Engine-ready |
| Add / delete / convert anchor | Changes path points and corner continuity. | Engine-ready |
| Rounded rectangle | Rectangles with rounded corners. | Engine-ready |
| Arc | Draws a curved segment. | Engine-ready |
| Spiral | Creates a spiral path. | Engine-ready |
| Rectangular Grid / Polar Grid | Creates a grid as artwork. | Engine-ready |
| Parametric shape collection | Triangle, arrow, donut, pie, cog, cloud, callouts, heart and others from parameters. | New |
| QR Code Tool | Creates a QR code from supplied content. | New |
| Shaper / shape recognition | Converts rough gestures into geometric shapes. | Engine-ready |
| Smooth / simplify path | Reduces irregularities or unnecessary anchors. | Engine-ready |
| Join / connect paths | Connects compatible open endpoints. | Engine-ready |
| Scissors | Splits a path at a chosen point. | Engine-ready |
| Knife | Cuts shapes along a drawn cut. | Engine-ready |
| Path Eraser | Removes portions of a vector path. | Engine-ready |
| Boolean union / subtract / intersect / exclude | Combines shapes using set operations. | Engine-ready |
| Shape Builder | Merges or removes overlapping regions interactively. | Engine-ready |
| Offset Path / Contour | Creates a parallel inset or outset boundary. | Engine-ready |
| Live corners / Corner Tool | Rounds or reshapes selected corners. | Engine-ready |
| Outline / expand stroke | Converts stroke appearance into filled geometry. | Engine-ready |
| Variable stroke width | Changes width along a path. | Engine-ready |
| Image Trace | Converts a raster image into vector shapes. | Engine-ready |
| Blend | Interpolates shapes and colours between objects. | Engine-ready |
| Objects on Path | Distributes objects along a path. | Engine-ready |
| Symbol tools | Sprayer, Shifter, Scruncher, Sizer, Spinner, Stainer, Screener, Styler. | Engine-ready |

## Painting, filling and retouching

| Feature | Behaviour | Engine |
| --- | --- | --- |
| Raster Pencil / Pixel Tool | Draws hard-edged or textured pixel marks (the app's Pencil makes vector strokes). | New |
| Airbrush | Builds soft colour with a spray-like brush. | New |
| Mixer Brush / Blend | Mixes colours through brush strokes. | New |
| Smudge | Drags existing colour through an image. | New |
| Decoration / pattern brush | Repeats motifs along a stroke. | Engine-ready |
| Raster eraser | Removes painted pixels (the app's Eraser erases vector art). | New |
| Background Eraser / Magic Eraser | Removes pixels based on sampled colour. | New |
| Flood Fill / Paint Bucket (pixels) | Fills a connected pixel region. | New |
| Live Paint Selection | Selects individual faces or edges of Live Paint artwork. | Engine-ready |
| Eyedropper / Colour Picker | Samples a colour from artwork. | Engine-ready |
| Style Picker / appearance sampling | Copies appearance attributes between objects. | Engine-ready |
| Gradient fill | Interpolates colours across a region. | Engine-ready |
| Gradient Mesh | Interpolates colours over a mesh of points. | Engine-ready |
| Freeform Gradient | Blends colours around distributed stops or lines. | Engine-ready |
| Transparency Tool | Edits a spatial opacity gradient. | New |
| Clone Stamp / Copy Stamp | Copies pixels from a sampled source. | New |
| Healing Brush / Spot Healing | Repairs defects while blending with surroundings. | New |
| Patch Tool | Repairs a selected region using another region. | New |
| Content-Aware Fill / Move | Reconstructs image content when removing or relocating an object. | New |
| Dodge / Burn | Locally lightens or darkens pixels. | New |
| Sponge Tool | Locally changes colour saturation. | New |
| Blur / Sharpen brush | Locally softens or increases edge contrast. | New |
| Red Eye | Corrects flash-induced red pupils. | New |
| History Brush / Art History Brush | Paints from a saved image state. | New |
| Pattern Stamp | Paints using a repeating pattern. | New |

## Transforms, masks, type and reusable structure

| Feature | Behaviour | Engine |
| --- | --- | --- |
| Reflect | Mirrors artwork (scale and rotate are in the app). | Engine-ready |
| Shear / skew / free distort | Moves corners or axes to distort geometry. | Engine-ready |
| Perspective transform | Maps artwork to a perspective quadrilateral. | Engine-ready |
| Mesh / envelope warp | Deforms artwork using a mesh or envelope. | Engine-ready |
| Puppet Warp | Deforms artwork using positioned pins. | Engine-ready |
| Liquify | Pushes, expands, contracts or twists image regions. | New |
| Vector deformation brushes | Warp, Twirl, Pucker, Bloat, Scallop, Crystallize, Wrinkle. | Engine-ready |
| Perspective Grid / Selection | Constructs and edits art on perspective planes. | Engine-ready |
| Isometric grid / plane editing | Aligns construction to axonometric planes. | New |
| Clipping mask / clipped content | Limits content to another object's boundary. | Engine-ready |
| Point / artistic text | Creates directly positioned editable text. | Engine-ready |
| Area / frame text | Flows text within a bounded frame. | Engine-ready |
| Text on a path | Places text along an editable vector path. | Engine-ready |
| Vertical type | Arranges text vertically. | Engine-ready |
| Touch Type | Transforms individual characters while retaining text. | Engine-ready |
| Text outlines | Converts text to editable vector contours. | Engine-ready |
| Character / paragraph styles | Reuses named text formatting. | Engine-ready |
| Linked symbols | Reuses artwork with synchronized instances. | Engine-ready |
| Smart Object | Keeps embedded or linked source content for reversible editing. | New |
| Adjustment / correction layers | Applies editable colour changes through layers (the app's filter layers apply pixel filters, not colour adjustments). | New |
| Appearance / graphic styles | Stacks and reuses fills, strokes and effects. | Engine-ready |
| Radial / grid / mirror Repeat | Creates editable repeated arrangements. | Engine-ready |
| Pattern creation | Defines artwork as a repeating pattern. | Engine-ready |

## Colour and tonal adjustments

| Feature | Behaviour | Engine |
| --- | --- | --- |
| Brightness / Contrast | Changes overall luminance and tonal separation. | Engine-ready (vector colour effect) |
| Levels | Remaps black point, white point and midtones. | Engine-ready (vector colour effect) |
| Curves / Tone Curve | Remaps tones using an editable curve. | Engine-ready (vector colour effect) |
| Hue / Saturation / Lightness | Changes colour angle, intensity and brightness. | Engine-ready (vector colour effect) |
| Colour Balance | Shifts colours toward complementary colour pairs. | New |
| Exposure | Changes exposure-related brightness. | Engine-ready (vector colour effect) |
| Vibrance | Adjusts saturation with selective weighting. | New |
| Black and White / monochrome | Removes hue while controlling tonal rendering. | Engine-ready (vector colour effect) |
| Invert | Reverses colour or tone values. | Engine-ready (vector colour effect) |
| Posterize | Reduces the number of tonal levels. | New |
| Threshold / Binarization | Converts tones to black or white at a cutoff. | New |
| Gradient Map | Maps image brightness to gradient colours. | New |
| Channel Mixer | Builds output channels from weighted input channels. | New |
| Selective Colour | Adjusts selected colour families. | New |
| LUT / Colour Lookup | Remaps colours through a lookup table. | New |
| White Balance / temperature and tint | Corrects warm/cool and green/magenta casts. | Engine-ready (vector colour effect) |
| Shadows / Highlights | Targets dark and bright tonal regions separately. | New |
| Lens / Photo Filter | Applies a photographic colour cast. | New |
| Split Toning | Colours shadows and highlights separately. | New |
| Soft Proof | Previews an output colour profile. | Engine-ready |
| OpenColorIO adjustment | Transforms colours through an OCIO configuration. | New |
| Recolour artwork | Reassigns an illustration's palette. | Engine-ready |

## Blur, sharpening, distortion and image cleanup

| Feature | Behaviour | Engine |
| --- | --- | --- |
| Gaussian Blur | Smooths pixels with a Gaussian-weighted kernel. | Engine-ready (object effect; not a filter layer filter yet) |
| Blur / Blur More | Applies basic smoothing. | New |
| Motion Blur | Blurs along a direction. | New |
| Lens Blur | Simulates defocus and aperture-shaped highlights. | New |
| Surface Blur | Smooths similar tones while preserving boundaries. | New |
| Box / Average / Shape Blur | Uses uniform, whole-region or shaped averaging. | New |
| Blur Gallery | Field, Iris, Tilt-Shift, Path and Spin Blur. | New |
| Sharpen / Sharpen More | Increases edge contrast. | New |
| Smart Sharpen | Applies controllable blur-aware sharpening. | New |
| High Pass | Retains high-frequency detail around neutral gray. | New |
| Add Noise | Adds random brightness or colour variation. | New |
| Reduce Noise / Remove JPEG Noise | Suppresses noise or compression artifacts. | New |
| Dust & Scratches / Remove Dust | Removes small isolated marks. | New |
| Median / Despeckle | Suppresses isolated pixel variation. | New |
| Mosaic | Groups pixels into block-like cells. | New |
| Displace | Offsets pixels according to another image. | New |
| Pinch | Contracts or expands pixels around a centre. | New |
| Ripple / Wave | Applies repeating spatial distortion. | New |
| Twirl | Rotates pixels progressively around a centre. | New |
| Polar Coordinates | Converts between rectangular and polar layouts. | New |
| Spherize / Curved Surface | Warps an image toward a curved surface appearance. | New |
| ZigZag | Displaces pixels in concentric patterns. | New |
| Lens Correction / Adaptive Wide Angle | Corrects photographic geometric distortion. | New |
| Camera Raw Filter | Photographic colour, detail and lens controls. | New |
| Liquify Face-Aware controls | Detects and adjusts facial proportions. | New |

## Vector effects and styling

| Feature | Behaviour | Engine |
| --- | --- | --- |
| Inner Shadow | Renders a shadow inside a boundary. | New |
| Inner Glow | Adds a soft halo inside a boundary (Outer Glow is in the app). | Engine-ready |
| Feather | Softens an object's visible boundary (the app uses it only for stroke hardness). | Engine-ready |
| Roughen | Perturbs path contours. | Engine-ready |
| Zig Zag path effect | Alternates peaks and valleys along paths. | Engine-ready |
| Pucker & Bloat effect | Pulls geometry inward or pushes it outward. | Engine-ready |
| Twist / Tweak effect | Rotates or perturbs path geometry. | Engine-ready |
| Transform effect | Applies live transformations and repeated copies. | Engine-ready |
| Warp presets | Bends artwork into predefined envelope forms. | Engine-ready |
| Scribble | Replaces fill appearance with irregular strokes. | Engine-ready |
| Convert to Shape effect | Encloses artwork in a live rectangle or ellipse. | Engine-ready |
| SVG filter | Applies an SVG filter definition. | Engine-ready |
| 3D Extrude / Revolve / Rotate / Inflate | Builds a shaded 3D-style result from vector artwork. | Engine-ready |
| Crop Marks | Adds print-trimming reference marks. | Engine-ready |
| Rasterize effect | Renders vector content to pixels within an effect stack. | Engine-ready |

## Additional raster, illustration and comic effects

| Feature | Behaviour | Engine |
| --- | --- | --- |
| Clouds / Difference Clouds / Perlin Noise | Generates procedural cloudy texture. | New |
| Lens Flare | Adds a simulated optical flare. | Engine-ready (vector Flare tool) |
| Fibers | Generates streaked fibrous texture. | New |
| Emboss filter | Converts contrast into directional relief shading. | New |
| Find Edges / Trace Contour | Extracts boundaries from image contrast. | New |
| Oil Paint | Creates flowing painted strokes. | New |
| Offset | Shifts image coordinates with edge handling. | New |
| Minimum / Maximum | Expands dark or light regions. | New |
| Custom convolution | Applies a user-defined filter matrix. | New |
| Artistic / line extraction | Simplifies colour or extracts drawn-looking outlines. | New |
| Pencil Drawing filter | Creates pencil-style contours and hatching. | New |
| Retro Film | Combines vintage colour and optical treatments. | New |
| Chromatic Aberration | Offsets colour channels spatially. | New |
| Normal Map | Converts an image into surface-normal texture data. | New |
| Quick Filters / Quick Effects | Applies preset colour or effect treatments. | New |
| Remove Dust tool | Cleans isolated marks using a targeted tool. | New |
| Rulers / Perspective Ruler / Symmetry Ruler | Constrains brush strokes to constructed guides. | New |
| Frame Border | Creates and divides comic panels. | New |
| Balloon / Balloon Tail | Creates editable speech balloons and tails. | New |
| Speed Lines / Focus Lines | Generates parallel or converging action lines. | New |
| Screentone / layer tone | Converts shading to printable patterns. | New |
| LT conversion | Separates image or 3D content into lines and tones. | New |
| 3D reference manipulation | Positions models, poses and cameras for drawing reference. | New |

## Canva-style effects and assisted editing

| Feature | Behaviour | Engine |
| --- | --- | --- |
| Photo filters | Applies a preset photographic colour treatment. | New |
| Duotone | Maps an image into a two-colour treatment. | Engine-ready |
| Blur / Auto Focus | Softens images or emphasizes the subject. | New |
| Face Retouch | Applies portrait cleanup. | New |
| Text effects (Shadow, Lift, Hollow, Splice, Echo, Glitch, Neon) | Named text appearance effects. | New |
| Curve text | Bends a text line into a curved arrangement. | New |
| Background text effect | Adds a coloured backing behind text. | New |
| BG Remover / Remove Background | Removes the background around a detected subject. | New |
| Magic Eraser / Remove Tool | Removes unwanted image content. | New |
| Generative Fill / Magic Edit | Replaces or adds content using generative editing. | New |
| Generative Expand / Magic Expand | Creates image content beyond existing boundaries. | New |
| Magic Grab | Separates a photo subject for repositioning. | New |
| Grab Text | Detects image text for editing. | New |
| Magic Layers | Separates image content into editable elements. | New |
| BG Generator | Generates a new photographic background. | New |
| Upscale | Increases image dimensions using enhancement processing. | New |
| Style Match | Applies matching visual styling to supported content. | New |
| Magic Resize / Magic Switch | Adapts a design to another size or format. | New |
| Frames / grids | Places images into predefined layout containers. | New |
| Charts | Creates data-driven visual charts. | Engine-ready |

## Additional current assisted and raster effects

| Feature | Behaviour | Engine |
| --- | --- | --- |
| Glow filter | Adds luminous bloom to image content. | New |
| Inflate filter | Creates a raised, inflated surface appearance. | New |
| Text to Vector Graphic | Generates vector artwork from a prompt. | New |
| Text to Pattern | Generates repeating patterns from a prompt. | New |
| Generative Shape Fill | Generates content within a selected shape. | New |
| Generative Recolor | Recolours artwork using a text prompt. | New |
| Prompt to Edit | Changes generated artwork through text instructions. | New |

## Layer blend modes

| Feature | Behaviour | Engine |
| --- | --- | --- |
| Exclusion blend mode | A softer difference-like operation. | Engine-ready |
| Dissolve blend mode | Converts partial coverage into scattered opaque pixels. | New |
| Linear Burn blend mode | Adds channel values and subtracts one. | New |
| Darker Color blend mode | Chooses the darker complete colour. | New |
| Linear Dodge / Add blend mode | Adds channel values. | New |
| Glow Dodge blend mode | Clip Studio's enhanced dodge variant. | New |
| Add (Glow) blend mode | Clip Studio's enhanced additive variant. | New |
| Lighter Color blend mode | Chooses the lighter complete colour. | New |
| Vivid Light blend mode | Combines burn and dodge operations. | New |
| Linear Light blend mode | Applies a signed linear brightness change. | New |
| Pin Light blend mode | Combines conditional darken and lighten operations. | New |
| Hard Mix blend mode | Produces strongly quantized channel colours. | New |
| Subtract blend mode | Subtracts source values from base values. | New |
| Divide blend mode | Divides base values by source values. | New |
| Average blend mode | Averages source and destination values. | New |
| Negation blend mode | Applies a sum-based inversion operation. | New |
| Reflect blend mode | Applies a nonlinear reflect operation. | New |
| Glow blend mode | The reversed-input counterpart of Reflect. | New |
| Contrast Negate blend mode | Inverts according to source and destination contrast. | New |
| Erase layer blend mode | Uses layer coverage to remove underlying visibility. | New |
| Pass Through | Lets a folder's children blend with layers outside it. | New |

## Opacity, transparency, masks and advanced compositing

| Feature | Behaviour | Engine |
| --- | --- | --- |
| Fill / stroke appearance opacity | Sets transparency for an individual fill or stroke. | Engine-ready |
| Gradient-stop opacity | Assigns different alpha values to gradient stops. | Engine-ready |
| Blend If | Makes visibility depend on source or backdrop channel values. | New |
| Source / underlying blend ranges | Maps tonal values to transparency through curves. | New |
| Per-channel blend ranges | Uses a chosen colour channel to control tonal blending. | New |
| Blend gamma | Changes the transfer curve used for layer blending. | New |
| Antialias coverage map | Remaps partially covered edge pixels. | New |
| Advanced blending channel exclusion | Disables selected channels in layer blending. | New |
| Shallow / Deep knockout | Punches through a group or toward the background. | New |
| Blend Clipped Layers As Group | Controls how a clipping stack receives the base blend mode. | New |
| Blend Interior Effects As Group | Applies layer blending to internal style effects together. | New |
| Transparency Shapes Layer | Controls whether existing transparency bounds layer effects. | New |
| Layer / Vector Mask Hides Effects | Controls whether a mask also hides layer styles. | New |
| Isolate Blending | Restricts child blending to a group or layer. | Engine-ready |
| Knockout Group | Controls whether overlapping group members obscure one another. | Engine-ready |
| Opacity and Mask Define Knockout Shape | Uses transparency when determining a knockout boundary. | Engine-ready |
| Opacity mask clip and unlink | The mask's clip and unlink options (invert and disable are in the app). | Engine-ready |
| Alpha lock / Lock Transparent Pixels | Restricts painting to existing coverage. | New |
| Clip to Layer Below / clipping stack | Uses a lower layer's coverage to constrain upper layers. | New |
| Mask density / feather | Weakens or softens a mask without repainting it. | New |
| Apply Image | Blends image or channel content into a target. | New |
| Calculations | Combines channels into a new channel, selection or document. | New |
| Fade command | Changes the strength and mode of the last supported edit. | New |
| Smart Filter blending options | Assigns opacity and a blend mode to a filter. | New |
| Effect-specific opacity / blend mode | Controls compositing of individual effects (the engine takes them as parameters; the app has no settings for styles or filters yet). | Engine-ready |

## Brush transfer, flow and colour mixing

| Feature | Behaviour | Engine |
| --- | --- | --- |
| Transfer dynamics | Varies brush opacity and flow along a stroke (the Brush varies only its size by pressure). | New |
| Opacity jitter / pressure response | Varies stroke opacity using randomness or input dynamics. | New |
| Flow jitter / pressure response | Varies deposition rate along a stroke. | New |
| Brush blending mode | Blends new marks with existing pixels on the same layer. | New |
| Behind / Background painting mode | Deposits new paint behind existing coverage. | New |
| Clear / Erase painting mode | Removes coverage using the painting tool. | New |
| Black Burn tool mode | Darkens existing painted areas. | New |
| White Burn tool mode | Lightens existing painted areas. | New |
| Erase (Compare) tool mode | Erases according to an opacity comparison. | New |
| Compare Density tool mode | Paints only where the new mark has greater opacity. | New |
| Replace Alpha tool mode | Replaces coverage with the tool's opacity. | New |
| Colour mixing method | Blend, Running color and Smear mixing of fresh and existing colour. | New |
| Amount of Paint | Controls the fresh-colour contribution during mixing. | New |
| Density of Paint | Controls the density of paint deposited while mixing. | New |
| Color Stretch | Controls how far existing colour is dragged. | New |
| Mixer Brush Wet / Load / Mix | Controls pickup, paint supply and source mixing. | New |
| Mixer Brush clean / load | Clears or replenishes the brush reservoir. | New |
| Wet Edges | Accumulates stronger paint near stroke boundaries. | New |
| Brush texture blending (Brush and Marker) | Combines brush coverage with a texture (the Pencil's grain is in the app). | New |
