# Ex-Tempore Creare — File Standard

**Document:** `filestandard.md`  
**Status:** Proposed alpha specification (not yet implemented or ratified)  
**Native extension:** `.etc`  
**Working format version:** `0.4.0` (draft; infinite canvas and export slices added)  
**Reference-board extension:** `.etcr` (independent sidecar document)  
**Structural foundation for `.etc`:** OpenRaster (ORA), specification 0.0.6

## 1. Purpose and design rules

An `.etc` file is the **native, lossless editable project** for Ex-Tempore Creare. Its primary responsibility is to preserve the complete document, not merely the flattened picture.

1. **ORA-derived, not an ORA file:** reuse OpenRaster's readable ZIP/XML/image architecture, while using an ETC-specific manifest and file identity. Renaming `.etc` to `.ora` must **not** be presented as valid interchange.
2. **Non-destructive by default:** raster/vector assets, masks, layer effects, textures, settings and cascading filters remain editable on reopen.
3. **Top-to-bottom layer order:** save the visible layer hierarchy in UI order, with the first item at the top. Rendering must respect global filters over all visually lower items, including across folder boundaries.
4. **Extensible and versioned:** every custom node/filter and its parameters need stable identifiers and schema versions. Preserve unknown metadata where safe; never silently discard unsupported editing features.
5. **Portable and recoverable:** use open, documented encodings where practical, include a flattened preview, and validate the archive before replacing the original during saving. Reference imagery deliberately stays separate and linked; portability depends on whether referenced files remain accessible.
6. **Independent references:** artwork lives in `.etc`; the reference board lives in `.etcr` and can also be opened independently. The artwork may link to a board but **MUST NOT** contain reference-board images or their canvas data.
7. **Interchange is separate:** export to `.ora`, `.svg`, `.png`, etc. through explicit converters with loss-of-editability warnings where appropriate.

**Terminology:** **MUST** = required for the alpha format; **SHOULD** = recommended; **MAY** = optional or future capability.

## 2. OpenRaster foundation

OpenRaster 0.0.6 establishes the foundation for the ETC container:

| ORA concept | ETC treatment |
| --- | --- |
| ZIP container and relative asset paths | **Adopt**; UTF-8 paths, ZIP64 when needed. |
| `mimetype` as first uncompressed archive entry | **Adopt layout, change identity** to ETC's distinct provisional MIME value. |
| `stack.xml` with topmost item first | **Retain as a compatibility projection**; it is not authoritative for ETC-only features. |
| PNG raster assets, SVG vector assets | **Adopt** for RGB/greyscale raster and vector interchange; add lossless TIFF or a documented equivalent for native CMYK and other samples PNG cannot represent. |
| Nested `<stack>` groups | **Adopt representation** for conventional groups; native manifest controls ETC-specific scope. |
| Visibility, opacity, offsets, `composite-op` | **Map** to ETC properties where the semantics agree. |
| `mergedimage.png` | **Require** a current flattened preview. |
| `Thumbnails/thumbnail.png` | **Require** a preview of at most 256 × 256 pixels, colour-converted to sRGB. |
| Editable filter nodes and styles | **Extend natively**; OpenRaster's filter-effects specification remains a proposal rather than baseline. |
| Independent Fill opacity, masks, reference-board links and UI metadata | **Extend natively**; do not pretend these are universal ORA baseline properties. The board itself resides in `.etcr`. |

Official references:
- [OpenRaster specification home](https://www.openraster.org/)
- [OpenRaster file layout specification](https://www.openraster.org/baseline/file-layout-spec.html)
- [OpenRaster layer stack specification](https://www.openraster.org/baseline/layer-stack-spec.html)
- [OpenRaster filter effects proposal — **not** baseline](https://www.openraster.org/proposals/filter-effects.html)

## 3. `.etc` container and internal layout

An ETC document **MUST** be a ZIP archive with the following baseline layout. Suggested archive entries:

```text
Artwork.etc
├── mimetype                        # First entry, STORED (not compressed)
├── project.json                    # AUTHORITATIVE ETC document manifest
├── stack.xml                       # ORA-style compatibility layer projection
├── mergedimage.png                 # Complete flattened image
├── Thumbnails/
│   └── thumbnail.png               # Thumbnail, <=256x256
├── data/
│   ├── raster/                     # Lossless PNG (RGB/grey) or TIFF (CMYK) assets
│   ├── vector/                     # Editable SVG documents
│   ├── masks/                      # Editable grayscale/alpha mask images
│   └── textures/                   # Embedded texture assets
├── profiles/                       # Embedded ICC profiles and colour-management references
└── workspace/                      # Optional panel positions and scratchpad (not reference-board data)
```

### File identification

- **Extension:** `.etc` (deliberately chosen despite possible extension collisions with unrelated software).
- **Provisional, unregistered MIME:** `application/x-ex-tempore-creare`.
- The first ZIP entry **MUST** be `mimetype`, use the ZIP **STORED** method, and contain exactly `application/x-ex-tempore-creare` with no trailing newline.
- `project.json` **MUST** include `format: "ex-tempore-creare"` and `formatVersion`.
- Loading software **MUST** verify the archive structure and these identity fields; the filename extension alone is insufficient.
- `stack.xml` **MUST NOT** be treated as a replacement for `project.json`. Some native ETC nodes cannot be expressed in baseline ORA.

The MIME name is a **proposed ETC identifier**, not an IANA registration or a claim of OpenRaster compatibility.

### Encoding requirements

- Archive entries **MUST** use safe, relative paths; no `../`, absolute paths, duplicated conflicting entries, or escaping the archive root.
- JSON **MUST** use UTF-8; XML **MUST** use UTF-8.
- PNG and other already-compressed images **SHOULD** use ZIP STORED; JSON/XML **MAY** use DEFLATE. Use ZIP64 when necessary.
- Each stored resource **MUST** have a stable ID and path; assets referenced in the manifest **MUST** exist or have an explicit unresolved-external-reference status.
- `mergedimage.png` **MUST** represent the current visible finite document, or, for an infinite canvas, a documented finite preview extent (e.g. current saved view/content bounds) with that extent recorded in `project.json` (with ETC filters and compositing baked in), converted to sRGB with an appropriate colour tag/profile. It **MUST NOT** be mistaken for an editable layer or a replacement for native CMYK/wide-gamut data.
- The thumbnail **MUST** be a non-interlaced 8-bit/channel sRGB PNG, at most 256 × 256 pixels, preserving aspect ratio and carrying an appropriate colour tag.

## 4. Canonical document model

`project.json` is authoritative. `stack.xml` is derived from it whenever a file is saved.

### 4.1 Document metadata

| Property | Requirement |
| --- | --- |
| Format name and schema version | **MUST** |
| Project UUID and title | **MUST** |
| Canvas mode (`finite` or `infinite`) | **MUST**; a new or existing document declares its mode explicitly. |
| Canvas width/height in integer pixels | **MUST** for `finite`; **MUST NOT** imply clipping bounds in `infinite` mode. |
| Resolution (DPI/PPI) | **MUST** |
| Color space, channel depth, alpha model and profile reference | **MUST** |
| Canvas background (transparent or explicit color) | **MUST** |
| Source app/version and modification timestamp | **SHOULD** |
| Active layer, workspace state, zoom and floating panel positions | **MAY**; convenience metadata only |
| Export slices, export output settings and dot-grid preferences | **MUST** be persisted if present, as versioned, editable `export` metadata distinct from the layer stack |

### 4.2 Layer node types

| Node type | Data preserved |
| --- | --- |
| **Raster** | Full-resolution pixels or reconstructible raster tiles, offsets, transforms, brush-created marks. |
| **Vector** | Editable paths, shapes, control points, stroke/fill properties, transforms; SVG-backed where representable. |
| **Folder** | Ordered child nodes, collapse state, group compositing properties, visibility and locks. |
| **Filter** | Non-destructive effect identifier, parameters, mask, ordering, enabled state and global scope. |
| **Texture** | Source/preset ID, embedded pattern, tiling/scale, transform, blend mode, opacity and fill. |

Every node **MUST** preserve:

- A stable, unique ID; user-facing name; type; visible/locked state; and explicit position in the top-to-bottom hierarchy.
- **Blend mode**, **Opacity**, and **Fill** as independent editable values. Store Opacity and Fill at normalized precision (`0.0`–`1.0`); the UI shows percentages (`0`–`100%`).
- Editable masks and modifier references, including their ordering and enable state, when present.
- Node transform and asset references as applicable to the node type.
- The originating feature/renderer identifier and version when needed for consistent reopening.

**Fill is not an alias for Opacity.** For layer styles, Fill adjusts the layer's base content without necessarily reducing its separately rendered styles; Opacity adjusts the final layer contribution. The precise blend pipeline **MUST** be documented by the ETC compositor and round-trip tests.

### 4.3 Layer modifiers and effects

The layer's hamburger menu exposes **per-layer modifiers**. The format **MUST** retain an ordered editable list of modifiers, each with:

- Stable effect type ID and effect schema/implementation version.
- Enabled/disabled state, named parameters, input/output colorspace assumptions and optional masks.
- Blend mode, opacity and other relevant effect properties.
- Enough information to reproduce the result or flag an unavailable renderer without destroying its metadata.

Alpha modifier targets include **Stroke, Bevel/Emboss, Color Overlay, Gradient Overlay, and Drop Shadow**. Additional modifiers can be added without changing the container format.

### 4.4 Masks

- A node **MAY** have one or more editable masks; mask assets and attachment order **MUST** be saved.
- The layer-card mask control **MUST** reference the same mask state saved in the document.
- Alpha/luminance interpretation, inversion, enabled state, transform and feathering **MUST** be encoded when used.
- Raster masks can be PNG-backed for alpha; vector masks can use SVG plus native metadata.

### 4.5 Reference-board links and textures

- Reference boards **MUST** be serialized in separate `.etcr` files as specified in §7. Artwork `.etc` files **MUST NOT** embed reference images, reference-board arrangements or board viewport state.
- An `.etc` project **MAY** contain `referenceBoards` descriptors with a board ID and a relative path or URI to a separate `.etcr` file. The link is not a dependency for rendering artwork.
- Reference images **MUST NOT** become canvas layers unless explicitly inserted by the user as artwork; doing so creates a separate artwork resource rather than retaining an implicit link to the reference board.
- Unavailable or moved `.etcr` files **MUST** be reported as missing and remain relinkable without damaging the `.etc` document.
- Texture presets **MUST** save their actual recipe/asset, not rely solely on a library display name. Texture assets are **artwork resources**, not `.etcr` reference images.
- Custom/imported textures **SHOULD** be embedded within `.etc`, with deduplication by hash where possible.
- The color-mixing sketchpad **MAY** be saved as workspace data if the user elects to retain it; it is not automatically a compositing layer.

## 5. Layer hierarchy, compositing and cascading filters

This section is **ETC-specific**, not baseline ORA behavior.

### Ordering

1. The manifest's display order is **top to bottom**, matching the Layers panel.
2. Folder children retain their own top-to-bottom order. Folder collapse is a UI state, not a change in image order.
3. Compositing proceeds from underlying content toward the top; the final visual result must match the displayed stack and documented blend semantics.
4. A filter layer applies to the **composited content of every applicable node below its position**, **globally across folder boundaries**. A filter inside a folder still has global scope.
5. Multiple filters cascade deterministically: the **lower** filter processes its lower content first; the **higher** filter then processes that result.
6. Filter layers do not retroactively modify layers above their position. Hidden/disabled filters have no render effect but retain parameters.
7. Layer masks, filter masks, group compositing, opacity and fill are preserved. If group isolation conflicts with a global filter boundary, the renderer **MUST** honor global scope, using intermediate surfaces as needed; it **MUST NOT** silently restrict the filter to its folder.

Example visual order (top first):

```text
[Raster: Highlights]          # Not processed by filters below
[Filter: Color Balance]       # Applies to everything below
[Folder: Illustration]
    [Vector: Linework]
    [Raster: Paint]
[Filter: Blur]                # Applies to everything below
[Texture: Paper Grain]
[Raster: Background]
```

In this example, **Blur** affects Paper Grain and Background. **Color Balance** affects the combined result of the folder and everything beneath it, including the result of Blur, but not Highlights.

### Rendering metadata

Filters **MUST** serialize their scope as `all-below` for the alpha, and retain their parameters and stable implementation versions. Other scopes are **not in alpha scope**. GPU-optimized processing or preview approximations must not silently change the saved project's intended render; `mergedimage.png` must reflect the final canonical render.

## 6. Colour space, ICC profiles and print workflows

This section is part of the **native `.etc` format contract**, not a claim that baseline ORA provides complete colour management. A colour model (RGB, CMYK, greyscale) is **not**, by itself, a colour space: correct colour reproduction also depends on the actual colour encoding or ICC profile. A generic `"CMYK"` label does not identify a valid print condition.

### 6.1 Supported spaces and print conditions

| Working space / profile family | Colour model | ETC requirement | Typical use |
| --- | --- | --- | --- |
| **sRGB IEC 61966-2-1** | RGB | **MUST**; default for new projects and flattened previews | Standard web, app and general illustration output. |
| **Display P3** | RGB | **MUST** support through a valid ICC profile | Wide-gamut tablet displays and artwork. |
| **Adobe RGB (1998)** | RGB | **MUST** support through a valid ICC profile | Photography and print-oriented RGB workflows. |
| **ProPhoto RGB / ROMM RGB** | RGB | **SHOULD** support through ICC import | Wide-gamut photographic workflows. |
| **ICC-profiled greyscale** | Grey | **MUST** preserve and transform | Monochrome artwork and masks (masks keep their own opacity semantics). |
| **PSO Coated v3 / FOGRA51** | CMYK | **MUST** support through ICC import | European coated offset print condition. |
| **PSO Uncoated v3 / FOGRA52** | CMYK | **MUST** support through ICC import | European uncoated offset print condition. |
| **GRACoL 2013 / CRPC6** | CMYK | **MUST** support through ICC import | North American coated commercial printing. |
| **SWOP 2013 / CRPC5** | CMYK | **SHOULD** support through ICC import | North American web-offset print condition. |
| **ISO Coated v2 / FOGRA39, legacy SWOP profiles** | CMYK | **SHOULD** support via ICC import | Compatibility with older production files. |
| **Other valid ICC v2/v4 RGB, grey or CMYK profiles** | Varies | **MUST** retain the original profile bytes and unmodified source samples; **SHOULD** permit conversions if the colour-management engine supports them | User-supplied printer, camera and document profiles. |

The named print conditions are **examples of recognized profile families**, not universal CMYK defaults. The printer or print service's requested profile and substrate **MUST** determine the export target. The file format **MUST NOT** assume that one CMYK conversion is valid for every printing process.

### 6.2 Profile identity, embedding and persistence

- The native manifest **MUST** describe the document's **colour model**, **working profile**, **channel bit depth**, **alpha convention**, **rendering intent**, and whether black-point compensation is enabled.
- Profiles **MUST** be identified by a stable manifest ID. A profile reference **MUST** resolve to an embedded `profiles/*.icc` file or a precisely defined built-in profile (for example, `builtin:sRGB` meaning IEC 61966-2-1). Profile display names alone **MUST NOT** be treated as unique identifiers.
- User-supplied profiles used by document content **MUST** be retained byte-for-byte in `profiles/` where redistribution/embedding is permitted, with a SHA-256 content digest and the originating profile version. Where a licence prohibits embedding, the manifest **MUST** retain a hash and unambiguous reference and the application **MUST** warn that portability is degraded.
- Both ICC **v2 and v4** profile inputs **MUST** be recognised; ICC v4 is **preferred** for newly authored profiles when available. Unsupported or invalid profiles **MUST NOT** be silently replaced with sRGB.
- Each raster asset and each vector object with explicit colour values **MUST** resolve its colour encoding through its own profile reference or the document profile. Mixed-profile layers are permitted; their original samples and profiles **MUST** survive a save/reopen round trip.
- The import workflow **MUST** distinguish **Assign Profile** (reinterpret unchanged sample values) from **Convert to Profile** (transform values to maintain appearance). Import/open **MUST NOT** implicitly reassign or convert embedded profiles.
- Missing profiles, mismatched hashes, unsupported ICC types, and colour-model/profile incompatibilities **MUST** raise an explicit diagnostic. The editor **MUST** avoid destructive conversion on open and retain the original asset for recovery.

### 6.3 Colour transformations and compositor behaviour

- A colour-managed renderer **MUST** transform source colours into the declared **document compositing space** before applying blending, layer modifiers and global cascading filters; it **MUST NOT** blend raw CMYK samples directly as if they were RGB.
- The manifest **MUST** identify the compositing space and transfer-function policy separately from the source profile (e.g. `linear-light` versus `profile-encoded` processing). The compositor's defined calculation domain, alpha convention, blend-mode semantics and effect input/output colour assumptions **MUST** be versioned and reproducible.
- Source raster samples and embedded profiles **MUST** remain unchanged until the user applies a destructive conversion or edits those samples. Transformed display/proof/compositing buffers are intermediate data, not replacements for the stored source.
- The colour engine **MUST** support **perceptual, relative colorimetric, saturation and absolute colorimetric** ICC rendering intents, plus a separately recorded black-point-compensation setting when supported by the profile/engine.
- Colour pickers and the physical-pigment mixing pad **MUST** make clear which colour space their stored samples use; pigment-mixing calculations **MUST NOT** be represented as standard ICC RGB interpolation unless that is actually the algorithm used.
- Tablet previews **MUST** be colour-managed to the active display profile when available; missing display-profile information **MAY** fall back to sRGB with an explicit assumption. Screen display and export profile are independent.
- A CMYK soft-proof preview **SHOULD** support a separately chosen proof ICC profile, rendering intent, black-point compensation and out-of-gamut warning. A proof view **MUST NOT** rewrite document pixels.

### 6.4 Raster storage and preview compatibility

| Stored asset | Alpha baseline encoding | Requirements |
| --- | --- | --- |
| RGB / RGBA raster | PNG, 8 or 16 bits/channel | Lossless colour samples; profile embedded in `iCCP` when permitted or referenced in the manifest. |
| Greyscale / greyscale + alpha | PNG, 8 or 16 bits/channel | Lossless samples; explicit grey profile when colours are interpreted as artwork. |
| Native CMYK raster | Lossless TIFF with CMYK photometric interpretation and an ICC profile, **or** a documented lossless multi-channel tile encoding | Preserve C, M, Y, K samples, profile association, and separate alpha/mask data. **PNG is not valid native CMYK storage.** |
| Vector artwork | SVG with ETC manifest colour metadata | Preserve editable geometry and source colour values/profiles; do not rely on inconsistent SVG ICC support alone. |
| `mergedimage.png` and thumbnail | sRGB PNG | Colour-converted **previews only**; may differ in channels and bit depth from the native working image. |

CMYK transparency **MUST** be stored without converting CMYK samples to RGB solely to retain an alpha channel. Separate alpha/masks are permitted and preferable when the chosen TIFF decoder or exporter cannot preserve the required association safely.

### 6.5 Print metadata and export rules

- A print-oriented project **SHOULD** retain the target ICC profile, intended medium/print condition, resolution in PPI, rendering intent, black-point compensation and optional ink-coverage warning threshold.
- Export commands **MUST** declare the destination profile and whether the operation **embeds**, **assigns** or **converts** the profile. Conversion to the chosen CMYK output profile **MUST NOT** overwrite the editable native project without consent.
- An exported PNG **MUST** be RGB or greyscale with an appropriate colour tag; it **MUST NOT** be described as CMYK PNG. A lossless CMYK output can use TIFF. PDF/X delivery requirements (e.g. PDF/X-4 output intents and overprint behaviour) are **not in alpha scope** unless separately approved.
- Export conversion **SHOULD** preserve original source artwork in `.etc` and emit explicit warnings when output gamut, bit depth, channel separation, transparency or blend mode differs from the native document.
- External profile downloads are **not** part of the `.etc` file-format contract. Distributed sample profiles **MUST** be checked for redistribution licences before inclusion in the application or project archives.

### 6.6 Manifest shape (colour-management subset)

The `canvas.colorManagement` object is the **authoritative** description of document colour. The legacy standalone `canvas.colorSpace` label is deprecated in favour of this object.

```json
{
  "canvas": {
    "width": 2048,
    "height": 1536,
    "resolutionPpi": 300,
    "bitDepthPerChannel": 8,
    "alpha": "straight",
    "colorManagement": {
      "colorModel": "RGB",
      "workingProfileRef": "builtin:sRGB",
      "compositingSpace": "working-profile",
      "compositingTransfer": "linear-light",
      "renderingIntent": "relative-colorimetric",
      "blackPointCompensation": true,
      "previewProfileRef": "builtin:sRGB",
      "softProof": {
        "enabled": false,
        "proofProfileRef": null
      }
    }
  },
  "colorProfiles": {
    "builtin:sRGB": {
      "type": "builtin",
      "standard": "IEC 61966-2-1"
    },
    "printer-profile-001": {
      "type": "embedded",
      "path": "profiles/printer-001.icc",
      "sha256": "<64-hex-character-digest>",
      "profileVersion": "4"
    }
  }
}
```

This fragment is **illustrative** (the SHA-256 field is a placeholder, not a valid digest). Asset descriptors **MUST** also provide their `colorModel`, `bitDepthPerChannel`, `profileRef` and storage encoding where these differ from the document defaults. Profile references and available transfer functions will be fixed by the versioned JSON Schema before implementation.

### 6.7 Colour-management acceptance tests

- [ ] Open/save/reopen sRGB, Display P3, Adobe RGB and valid ICC-profiled greyscale documents while retaining profile identity and original sample values.
- [ ] Open/save/reopen a CMYK document with FOGRA51, FOGRA52 and GRACoL 2013 profiles without converting the native CMYK samples to RGB.
- [ ] Verify that a vector object, a texture and a raster layer can carry different source profiles and remain editable.
- [ ] Verify assign-versus-convert workflows and missing-profile warnings; opening a file must never silently substitute sRGB.
- [ ] Verify normal and multiply compositing, masks, independent Fill/Opacity, modifiers and cross-folder cascading filters under managed colour conversion.
- [ ] Verify that previews are sRGB-tagged while native profile and sample precision are preserved; compare proof against a trusted ICC colour-management engine.
- [ ] Verify conversion to a user-selected print profile and no unintended modification to the source `.etc` project.

**Colour-management references:**
- [International Color Consortium — ICC profile specifications](https://www.color.org/icc_specs2/)
- [ICC colour profile registry and registered print conditions](https://registry.color.org/profile-registry/)
- [ICC RGB/sRGB profiles](https://registry.color.org/rgb-registry/srgbprofiles)
- [European Color Initiative — FOGRA/PSO colour profiles](https://eci.org/doku.php_id%3Den_downloads.html)
- [PRINTING United Alliance — GRACoL print conditions](https://www.printing.org/library/standards/specifications-for-print-production/gracol)
- [W3C — PNG Specification](https://www.w3.org/TR/png-3/)
- [Library of Congress — TIFF tags](https://www.loc.gov/preservation/digital/formats/content/tiff_tags.shtml)

## 7. Separate reference-board format: `.etcr`

`.etcr` (**Ex-Tempore Creare Reference**) is the native file for the **PureRef-style reference board**, distinct from the editable artwork `.etc` file. A board may be attached to a project or opened on its own. **The alpha requires editable board state, external image links and an unbounded canvas.** It does not require that source images be copied into either document.

### 7.1 Container, file identity and separation

The `.etcr` file **MUST** be a versioned ZIP container with a small UTF-8 JSON manifest. Its file identification **MUST** differ from `.etc`:

```text
References.etcr
├── mimetype                    # First ZIP entry; STORED, no trailing newline
├── board.json                  # Authoritative reference-board state
└── Thumbnails/
    └── thumbnail.png           # Optional sRGB board preview
```

- **Extension:** `.etcr`; **provisional MIME:** `application/x-ex-tempore-creare-reference`.
- The `mimetype` entry **MUST** have exactly that MIME string; `board.json` **MUST** include `format: "ex-tempore-creare-reference"`, `formatVersion`, and a stable `boardId`.
- ZIP entry validation, UTF-8 encoding, size limits, atomic saves and safe relative archive paths follow the `.etc` security rules in §3.
- `.etcr` **MUST NOT** be saved as `.etc`, nor merged into `.etc` automatically. A project link in `.etc` **MUST** point to a separately stored `.etcr` document.
- No `stack.xml`, artwork raster layers, compositing stack, or ORA interchange claim is required for `.etcr`.

### 7.2 Reference image sourcing — external by default

- For an image **already present on the device**, `.etcr` **MUST** save a **link to the existing file**, not duplicate its image bytes in `.etcr` or `.etc`.
- A source descriptor **MUST** provide a resolvable reference such as an external file URI and/or a relative file path, plus a stable source ID. Store the original filename and **SHOULD** store image dimensions and a SHA-256 hash for reliable relinking.
- Resolve portable **relative paths** relative to the `.etcr` document where possible. A device-specific absolute URI **MAY** be saved as a fallback. External paths may point outside the `.etcr` ZIP, but archive entry paths must still satisfy §3; external files are **read-only by default**.
- On mobile platforms with scoped file access, the app **MUST** use the operating system's permitted document/file-access mechanism. Device-specific grants, security tokens or credentials **MUST NOT** be embedded in a shareable `.etcr`; keep them in protected app-local state, associated with the source ID.
- An image pasted from clipboard, produced in-app or obtained from a temporary source **MUST** first be retained as a separate accessible image file (for example in an app-managed reference-assets directory) and linked. Do not silently embed it in the artwork or reference board.
- **Embedding/packing reference images is not part of the alpha contract.** A future, explicitly user-requested portable package operation **MAY** copy assets, but must not silently replace existing linked sources.
- If a link is broken, permission is revoked or a file moved, **MUST** retain its original image entry, transformations and source metadata, show a missing-image placeholder and support a **Relink** operation. Neither the board nor the artwork may fail destructively.
- Linked image updates on disk **SHOULD** be detectable using available file metadata or hash checks; reloading the source **MUST NOT** reset the object's crop, position, size or rotation.

### 7.3 Infinite-canvas board and image operations

The reference workspace uses an **unbounded 2D coordinate system**; it is not constrained by the pixel dimensions, DPI or colour mode of the artwork canvas.

| Property | Requirement |
| --- | --- |
| Position/reposition | **MUST** retain each reference image's x/y position in board coordinates. |
| Resize | **MUST** retain width/height or equivalent scale, with optional locked aspect ratio. |
| Rotate | **MUST** retain an explicit rotation angle and pivot/anchor. |
| Crop | **MUST** retain a **non-destructive** crop rectangle in source-image coordinates; cropping never modifies the linked original. |
| Pan and zoom | **MUST** allow navigation across the infinite board and preserve the last camera centre/pan and zoom. |
| Ordering | **MUST** preserve front-to-back reference object order and visibility. |
| Reopen | **MUST** restore all transformations, crop, camera and source links exactly within floating-point tolerance. |

- Board coordinates and transforms **MUST** be stored as finite, double-precision-capable numeric values; a renderer **MUST NOT** allocate an image or texture spanning the infinite canvas. Only viewport-relevant content should be rendered or cached.
- Crop coordinates **SHOULD** be normalized (`x`, `y`, `width`, `height` in `[0,1]`) against the original source image dimensions; invalid crops **MUST** be rejected or diagnosed.
- Pan and zoom are **board-camera** operations, independent from moving/rotating/scaling a particular image. The saved camera is a convenience view and does not change object geometry.

- Existing image source files **MUST NOT** be modified by board operations. Image colour profiles **SHOULD** be honoured for accurate reference display, but the board does not inherit the artwork's working colour space.
- Multiple `.etcr` boards **MAY** be linked to one `.etc` document; each board remains independently editable and saveable.

#### Camera-state persistence and save triggers

- The `.etcr` board camera **MUST** be persisted as the latest observed `viewport.center` (`x`, `y` in board coordinates) and `viewport.zoom`, including when the only change since the previous save is a pan or zoom. **MUST NOT** reset the view to the board origin or fit-to-content on normal reopen.
- **Explicit Save:** When the user chooses **Save** (or a Save shortcut) while the reference board is open, the application **MUST** capture the camera state at the time of the save request and write it to that board's `.etcr` `board.json` alongside its other current board state.
- **App/tab lifecycle:** When the application or reference-board tab is backgrounded, loses visibility, is suspended, is closed, or the user switches away from it, the application **MUST** attempt to persist the latest camera and unsaved board state automatically. This includes browser tab switches and mobile app backgrounding where the platform exposes a lifecycle signal. The user **MUST NOT** have to press Save to retain the last navigated camera position.
- **Recovery under forced termination:** Since mobile operating systems and browsers can terminate processes without a final event or adequate time for a file write, the application **MUST** also checkpoint camera changes promptly into protected app-local recovery state (for example, after a short debounce), keyed to the board ID. On reopening, if that state is newer than the saved `.etcr`, the application **MUST** offer or automatically perform a non-destructive recovery of the newer camera state. Do not claim guaranteed final writes after forced termination.
- **Atomicity:** Automatic `.etcr` writes **MUST** follow the same atomic-save and validation rules as manual saves. A failed or interrupted save **MUST NOT** corrupt the last valid board file. The application **MUST** keep pending changes recoverable and report persistent save failures.
- Camera-only saves **MUST NOT** modify linked source images or the associated `.etc` artwork file. The reference board remains independently saveable.

### 7.4 `.etcr` manifest example

Illustrative structure, not a finalized JSON Schema:

```json
{
  "format": "ex-tempore-creare-reference",
  "formatVersion": "0.1.0",
  "boardId": "7dff0ebf-e0d4-405e-9bfa-a9ae090b33c8",
  "title": "Character References",
  "coordinateSystem": { "unit": "board-unit", "origin": [0, 0] },
  "viewport": { "center": [250.0, -125.0], "zoom": 0.85 },
  "images": [
    {
      "id": "reference-001",
      "source": {
        "kind": "linked-file",
        "relativePath": "assets/pose-reference.png",
        "originalFilename": "pose-reference.png"
      },
      "transform": {
        "x": 150.0, "y": -80.0,
        "width": 600.0, "height": 420.0,
        "rotationDegrees": -12.0,
        "pivot": [0.5, 0.5]
      },
      "crop": { "x": 0.1, "y": 0.0, "width": 0.8, "height": 1.0 },
      "visible": true
    }
  ]
}
```

`relativePath` above identifies a **file outside the ZIP**, located relative to the `.etcr` file. It is **not** an image stored inside the archive. The application **MUST** obtain user/OS authorization before reading external files and **MUST** treat the reference board as untrusted input.

### 7.5 Link from `.etc` to `.etcr`

The `.etc` `project.json` **MAY** contain:

```json
"referenceBoards": [
  {
    "boardId": "7dff0ebf-e0d4-405e-9bfa-a9ae090b33c8",
    "path": "References.etcr",
    "missingPolicy": "relink"
  }
]
```

- Board paths **SHOULD** be relative to the `.etc` document. If project or board is moved, the app **MUST** offer relinking and **MUST NOT** silently import the board as an artwork layer.
- The `.etc` thumbnail, merged preview, output export and filter stack **MUST NOT** include the reference board. Saving `.etc` **MUST NOT** implicitly change or duplicate the `.etcr` or its linked images.
- Closing or editing an `.etcr` board **MUST** not depend on having a paired `.etc` project open.

### 7.6 `.etcr` alpha acceptance criteria

- [ ] Create, save and reopen an independent `.etcr` board with more than one externally linked image and no duplicate source image bytes.
- [ ] Reposition, resize, rotate and non-destructively crop each image; reproduce the same result after reopening.
- [ ] Pan/zoom over widely separated positive/negative coordinates without a finite canvas boundary or massive backing allocation.
- [ ] Pan/zoom the board, then use **Save**; reopen the `.etcr` and verify the exact last camera centre and zoom are restored. Repeat when **only** the camera changed.
- [ ] Pan/zoom and switch tabs, background/minimize/suspend the app, or close normally without pressing Save; on reopening, verify the latest recorded camera is restored from the `.etcr` or recovered checkpoint.
- [ ] Simulate interrupted background saving/forced termination: the previous `.etcr` remains valid, the newer camera position is recoverable when a checkpoint exists, and neither linked images nor `.etc` is modified.
- [ ] Move/revoke access to a source image: retain the layout, display a missing placeholder, then relink successfully.
- [ ] Open `.etc` with linked `.etcr`, then move/remove the `.etcr`: artwork remains editable and renders unchanged, with a clear reference-board warning.
- [ ] Verify neither `.etc` nor `.etcr` silently embeds existing on-device reference images.
- [ ] Validate file identity, archive safety and atomic-save behavior for `.etcr` separately from `.etc`.

## 7A. Canvas mode and non-destructive export slices

This section extends `.etc` without modifying `.etcr`. See [`applicationstandard.md`](applicationstandard.md) for the corresponding UI and gesture rules.

- `canvas.mode` **MUST** be `finite` or `infinite`. Finite mode **MUST** include positive integer `width` and `height` pixel dimensions; infinite mode **MUST** allow signed, unbounded document-space layer coordinates and MUST NOT use finite artboard dimensions as clipping limits.
- Infinite mode MAY save a finite default viewport, camera centre/zoom and a finite `mergedimage.png` **preview**, but those dimensions **MUST NOT** be interpreted as document bounds. Manifest metadata MUST identify the preview region.
- The native `export` object **MUST** contain an ordered `slices` list when the user defines slices. Each slice MUST have a stable ID, name, rectangle (`x`,`y`,`width`,`height`) in document pixels, a positive aspect ratio pair, and its output configuration (format, scale/resolution and colour profile).
- Slice rectangles **MAY** be outside the finite canvas bounds and use negative coordinates in infinite mode. Width and height MUST be positive finite numbers. Store geometry without destructive clipping.
- `export.grid` **SHOULD** preserve dot-grid visibility, document-pixel spacing and snapping state. Grid visuals, selection frames, labels, centre handles and control overlays MUST NOT be stored as rendered raster layers or included in flattened/exported artwork.
- The export tool's active slice selection and other temporary inspector state MAY be stored as workspace data; editable slice geometry MUST be stored as document data.
- **Whole document** exports are well-defined for finite mode only. Infinite mode **MUST** use an explicit finite extent (content bounds or one/more user-defined export slices), never an implied arbitrary raster size.
- Slice exports MUST render the full applicable cascading filter stack and preserve off-slice effect contributions before cropping. References linked by `.etcr` MUST NOT be composited into `.etc` exports.
- ORA interchange SHOULD flatten/omit slice metadata with explicit limitations; an ORA file does not natively encode ETC's export slice workflow.

Example `project.json` excerpt:

```json
{
  "canvas": { "mode": "infinite", "resolutionPpi": 300 },
  "export": {
    "grid": { "visible": true, "snapEnabled": false, "spacingPx": 32 },
    "slices": [
      {
        "id": "slice-001",
        "name": "Illustration detail",
        "rectPx": { "x": -250, "y": 175, "width": 1600, "height": 900 },
        "aspectRatio": { "x": 16, "y": 9 },
        "format": "png",
        "scale": 1,
        "exportProfileRef": "builtin:sRGB"
      }
    ]
  }
}
```

## 8. Minimal manifest example (illustrative, not a complete JSON Schema)

```json
{
  "format": "ex-tempore-creare",
  "formatVersion": "0.4.0",
  "projectId": "37bc0d92-f718-4e1c-8dfd-415af5876e84",
  "title": "Untitled Artwork",
  "canvas": {
    "width": 2048,
    "height": 1536,
    "resolutionPpi": 144,
    "bitDepthPerChannel": 8,
    "alpha": "straight",
    "colorManagement": {
      "colorModel": "RGB",
      "workingProfileRef": "builtin:sRGB",
      "compositingSpace": "working-profile",
      "compositingTransfer": "linear-light",
      "renderingIntent": "relative-colorimetric",
      "blackPointCompensation": true,
      "previewProfileRef": "builtin:sRGB"
    },
    "background": "transparent"
  },
  "colorProfiles": {
    "builtin:sRGB": { "type": "builtin", "standard": "IEC 61966-2-1" }
  },
  "displayOrder": "top-to-bottom",
  "nodes": [
    {
      "id": "filter-001",
      "type": "filter",
      "name": "Color Balance",
      "visible": true,
      "locked": false,
      "blendMode": "normal",
      "opacity": 1.0,
      "fill": 1.0,
      "scope": "all-below",
      "effect": {
        "typeId": "etc.color-balance",
        "version": 1,
        "parameters": { "cyanRed": 0.08 }
      }
    },
    {
      "id": "folder-001",
      "type": "folder",
      "name": "Illustration",
      "visible": true,
      "locked": false,
      "blendMode": "normal",
      "opacity": 1.0,
      "fill": 1.0,
      "children": [
        {
          "id": "raster-001",
          "type": "raster",
          "name": "Painting",
          "visible": true,
          "locked": false,
          "blendMode": "multiply",
          "opacity": 0.85,
          "fill": 0.75,
          "source": "data/raster/raster-001.png",
          "sourceProfileRef": "builtin:sRGB",
          "masks": [],
          "modifiers": []
        }
      ]
    }
  ],
  "referenceBoards": [
    {
      "boardId": "7dff0ebf-e0d4-405e-9bfa-a9ae090b33c8",
      "path": "References.etcr",
      "missingPolicy": "relink"
    }
  ],
  "textures": []
}
```

The canonical alpha implementation **MUST** publish a machine-validatable JSON Schema before compatibility is declared stable. This example records structure and intent, **not** all eventual brush/filter parameters or the full raster tile model.

## 9. Interchange/export policy

| Target | Alpha scope | Behavior |
| --- | --- | --- |
| `.etc` | **Native save/open — MUST** | Lossless editable round-trip of supported artwork features; optional link to separate `.etcr` board. |
| `.etcr` | **Native reference-board save/open — MUST** | Separate editable infinite reference canvas with linked source images, transforms and cropping; never bundled inside `.etc`. |
| `.ora` | **Planned interoperability** | Create a separately valid ORA archive with `image/openraster`, valid `stack.xml` and required preview files. Preserve representable layers; flatten or otherwise bake unsupported filters/styles with a clear warning. Do not claim a renamed `.etc` is ORA. |
| `.svg` | **Planned vector interchange** | Export editable vector geometry where representable. Raster and unsupported effects require embedding or rasterization. |
| `.png` | **Planned image export** | Export flattened RGB/grey image with correct transparency and ICC/profile tagging; CMYK requires another format. |
| `.tif` / `.tiff` | **Planned print interchange** | Lossless CMYK export with a selected and embedded ICC print profile; distinct from native project storage. |
| `.kra` | **Out of project scope** | See §10. No native support, parser, writer or converter required. |

Interchange exports **MUST NOT** overwrite or downgrade the source `.etc` project without explicit user consent. When an export would lose masks, styles, filters, color precision, or editability, the app **MUST** disclose the affected features.

## 10. Alternative format: Krita `.kra` — NOT IN SCOPE

**Official Krita file-format reference:** [Krita `.kra` documentation](https://docs.krita.org/en/general_concepts/file_formats/file_kra.html)

Krita's format is an **alternative for possible future evaluation only**. ETC will **not** implement `.kra` import, export, native project storage, or compatibility mode during the current alpha scope.

**Conditional future rule:** If a user explicitly selects a Krita-standard project mode in a later release, that mode **MUST** be limited to the tools, filters, compositing operations and other features that Krita's supported file standard can faithfully preserve. Unsupported ETC-specific functionality **MUST** be disabled/hidden or rejected for that mode; it **MUST NOT** be saved as if it were valid editable `.kra` content. Conversion that bakes unsupported features into raster data is a separate, explicitly warned operation, not preservation of editable compatibility.

No `.kra` implementation requirements are imposed by this document.

## 11. Alpha validation and acceptance criteria

An alpha implementation of `.etc` is acceptable when it passes all of the following:

- [ ] Save and reopen raster, vector, folder, texture and filter nodes with the same hierarchy and top-to-bottom order.
- [ ] Preserve independent Blend Mode, Opacity and Fill values for each node and reproduce their appearance.
- [ ] Preserve layer effects, their parameters/order, masks and enabled states; no silent flattening of the native document.
- [ ] Preserve **global** filter effects across folder boundaries, including multiple cascading filters.
- [ ] Meet the ICC/RGB/grey/CMYK round-trip, proof and conversion tests in §6.7.
- [ ] Preserve `.etcr` board links in `.etc` without embedding reference images or their placement metadata; verify `.etcr` separately against §7.6.
- [ ] Include correct `mimetype`, valid `project.json`, `stack.xml` projection, merged preview, and thumbnail.
- [ ] Handle missing assets, unknown filter IDs and unsupported schema versions with explicit diagnostics; do not overwrite the source on failure.
- [ ] Validate archive paths and resource sizes before extraction and reject unsafe paths or decompression bombs.
- [ ] Use an atomic save strategy (write to a temporary file, validate, then replace) so interrupted saves do not corrupt the existing project.
- [ ] Test save/load/save round trips for raster content, effects, masks, blends, folder nesting, opacity/fill and workspace metadata.
- [ ] Treat `.ora` export as a separate conforming artifact and verify it against an independent ORA reader/validator when implemented.

## 12. Deferred decisions (outside the minimum format contract)

- HDR, scene-referred colour, 32-bit floating-point pixel storage, and advanced spot/DeviceN colour workflows (core ICC-managed RGB, greyscale and CMYK are covered in §6).
- Tiled raster storage versus per-layer lossless PNG/TIFF assets; any tile codec requires versioned asset descriptors and exact sample/profile preservation.
- Brush-stroke replay and undo/redo history persistence.
- Non-reference linked artwork resources, font embedding and content-addressed asset deduplication. `.etcr` external image links are required by §7, not deferred.
- Optional preview or editing cache files for large tablet documents.
- A published JSON Schema, canonical effect-parameter schemas and cross-version migration tools.
- `.kra` compatibility: explicitly **out of scope** until separately approved.

---

**Design boundary:** `.etc` is the source of truth for **artwork**, and `.etcr` is the independent source of truth for **reference boards**. ORA supplies the `.etc` container and basic layer-stack model; ETC defines the additional layer, filter, texture, mask and modifier semantics. Existing on-device reference images remain linked external resources, not duplicated project payloads.
