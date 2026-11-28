# Resume: hand-off notes

Where the work stands and how to carry on. Read this first in a new session, then keep it current: update it whenever a step finishes or a decision is made.

## Read first

1. The owner's rules, which apply to every change: `docs/for_agents/for_agents.md`, `docs/for_agents/script_structure.md`, `docs/for_agents/bugcheck.md` and `docs/filestructure.md`.
2. The standards: `docs/Standardsdocs/UIStandards/uistandards.md` (UI), `docs/Standardsdocs/filestandard.md` (`.etc` and `.etcr` files), `docs/Standardsdocs/applicationstandard.md`, `docs/Standardsdocs/Toolstandards/`.
3. What exists and what is planned: `docs/featurelist.md` (the app's features, from its UI) and `docs/planned.md` (missing features, marked engine-ready or new).

## Standing rules

- **Follow instructions literally.** Treat what the owner specifies as requirements. Don't reinterpret, narrow or extend them, and add nothing unrequested; propose it and wait. When something is ambiguous, ask one short question. When a constraint blocks a requirement, name the conflict in one sentence and ask.
- **Current standards, never memory:** training data is out of date, so never pick a version or an API from memory.
  - Before adding or changing any SDK level, plugin, library, toolchain, NDK, Gradle or Rust version, look up the latest stable release at its primary source that day (Maven metadata on `dl.google.com` or Maven Central, `services.gradle.org/versions/current`, the Android SDK repository XML, official release notes), and use it unless the owner says otherwise.
  - Use the current, non-deprecated API for each tool and library at that version: read its release notes or migration guide for the version in use, not older examples.
  - Build with deprecation warnings shown (`--warning-mode all`, and compiler warnings) and fix every deprecation before committing.
  - When reporting, name each version chosen and where it was checked. When a source can't be reached, say so and ask; don't fall back to memory.
- **Git:**
  - Commit and push only to `beta`.
  - Run `git remote -v` before every push. Never force-push.
  - Push after every finished step ("push per step"), and give the commit's link afterwards.
- **Commit messages** end with `Made with assistance from machine coding tools` and nothing else. No Co-Authored-By, no session links, no model names, never the owner's email.
- **Code files:** every new code file ends with a `GLOSSARY` block listing its members. Files copied from VectorCraft get none.
- **Docs:** keep `docs/classes.md`, `docs/variables.md`, `docs/filestructure.md` and `docs/featurelist.md` current with every change. Don't edit the standards (`docs/Standardsdocs/`) unless the owner asks.
- **Replies to the owner:** brief, with paragraphs double-spaced, numbered or indented lists rather than tables (unless asked), and no filler. Say plainly whether the result does exactly what was asked, and if not, what differs and why.
- **Colours:** the UI uses the `EtcColors` blue-grey and gold tokens. Purple or pink in a pasted screenshot is a clipboard artifact; ignore it and use the screenshot for layout only.

## The project

Ex-Tempore Creare: a tablet-first Android art app.

1. **Engine:** Rust, under `engine/`, forked from VectorCraft (an Illustrator-like engine). Its parts are `crates/engine` (the session and its commands), `crates/render`, `crates/effects`, `crates/doc` (the document model), `crates/etcfile` (`.etc` and `.etcr` files) and `crates/bridge` (JNI, `libetcbridge`).
2. **App:** Kotlin and Compose, package `com.extemporecreare.app`.
   - `interface/` holds the UI (package `…app.ui`, because `interface` is a Kotlin keyword).
   - `components/` holds shared logic.
   - `tools/` holds the dial tools, plus `tools/Libraries/` for presets and textures, packaged as app assets.
   - `app/` is the Gradle module.
3. **Bridge:** `components/engine/EngineBridge.kt` calls the Rust bridge. Answers are JSON, either `{"ok": …}` or `{"error": {"kind", "message"}}`.

## State (2026-11-27)

Everything is pushed to `beta`. In order, the most recent work:

1. UI polish to the owner's mockup (layout only, `EtcColors`): labelled top bar pills, a 640 dp frosted dial centred on the bottom edge with a size slider, readout and colours inside it and a settings chevron, layer cards with thumbnails and lock, collapsed panels as pills, opaque panels (the §3.2 fallback), a restyled splash.
2. Live strokes for Marker, Pencil and Eraser; no flicker after a stroke; the Layers panel lists layers only (layer roles); the app laid out between the system bars (`43b078c`).
3. Build on API 37 with current build tools and a Gradle 9.9.3 wrapper (`6a61da2`); the Kotlin compiles against API 37, the full APK has not been built on it yet.
4. The debug APK builds on the owner's VM (`assembleDebug`, about 17 minutes from clean) and runs on the `Medium_Tablet_API_35` emulator to the start screen. This needed the text crate's `build/web_fonts.rs`, which `.gitignore`'s `build/` had kept out of the repository (`0c1882b`).
5. Dial grip docks the dial to any edge as a toolbar (`5173c19`).
6. `docs/featurelist.md` and `docs/planned.md` (`72a04f5`).
7. Reference board Embed stores images inside the `.etcr` (`5612780`).
8. Texture library and texture layers, filter layers, Fill and masks on object rows (`9ce6d50`).
9. Layer Fill, greyscale layer masks, layer styles: Stroke, Bevel & Emboss, Color Overlay, Gradient Overlay (`80c2a71`).
10. Artwork view twist rotation; rotation and View Flip saved per document (`df0f8bf`).
11. Reference board handles, view twist, image menu (`5c35261`).
12. `uistandards.md` §6 lists 11 tools (`c1fd51b`).

The full feature set is in `docs/featurelist.md`. Beyond the start screen, none of the UI has been tried on a device yet. The next job is to go through `featurelist.md` on the emulator and on a real tablet.

## Waiting on the owner

1. **Docked panels and the toolbar:** with the dial docked as a toolbar on the left or right, the panels docked in the top corners can overlap it. Should they shift inward?
2. **`uistandards.md` §6.1:** it still says tapping an inactive tool must not select it, which now applies only to the rotary dial (the toolbar uses tap to choose). Add the toolbar mode to §6.1?
3. **Settings screens:** layer styles and filter layers apply with default values; there are no settings screens for them yet.
4. **Masks on filter layers:** not built.
5. **Resetting view rotation:** there is no control to reset the artwork view's rotation to 0°. Twisting back is the only way.
6. **Pigment mixing:** the colour workspace's mixing pad uses a provisional pigment model (`components/color/ColorMath.kt`). The owner hasn't chosen the final model.
7. **Release build:** it should strip debug symbols. This was deferred until there is a release build.

## Known failing engine tests (from before this work)

- `engine/crates/engine/tests/command_sweep.rs` `catalog_path_keys_match_location_policy`: `svgInteractivity.info` no longer names `files`.
- Tests that read raw save bytes for strings of VectorCraft's old native format, which `.etc` replaced:
  - `tests::inline::saved_documents_keep_inline_graphics`
  - `tests::links::a_placed_link_records_the_file_and_the_document_saves_only_a_preview`
  - `tests::brushlib::image_brush_stamps_draw_and_carry_their_pixels_when_added_and_saved`
  - `tests::nativefile::a_pinned_date_makes_saves_byte_identical`
  - `cmd::fileio::tests::a_native_save_streams_the_bytes_the_encoder_makes`
  - `cmd::fileio::tests::a_save_refuses_disagreeing_options_and_names_unread_native_ones`

## Building and testing

- **Versions:** `compileSdk` 37.2 (Compose 1.13 needs 37.1 or later) and `targetSdk` 37, build-tools 37.0.0, Android Gradle plugin 9.5.3 (built-in Kotlin), Kotlin 2.4.21, Compose BOM 2026.11.01, Gradle 9.9.3 through the wrapper (`gradlew`), JDK 17 or newer, NDK 30.0.16248370 (`ndkVersion`; `buildEngine` passes it to cargo-ndk as `ANDROID_NDK_HOME`, so none needs setting).
- **Kotlin check:**
  ```bash
  ./gradlew --no-daemon -q compileDebugKotlin -x buildEngine
  ```
- **Full APK:** the `assembleDebug` task. Gradle's `buildEngine` task builds the Rust bridge for Android, which needs the Android NDK.
- **Engine tests:**
  ```bash
  CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=<outside the repo> cargo test -q -p etc-bridge -p etcfile -p vectorcraft-render -p vectorcraft-effects -p vectorcraft-doc
  ```
  The engine crate's own suite (`-p vectorcraft-engine`) is long and includes the known failures above.
- **Where to run cargo:** from `engine/`, or with `--manifest-path engine/Cargo.toml`. Keep the target directory outside the repo; `.gitignore` has no `target/` entry.
- **Dependencies:** the crates from storytold/craft-libs are copied into `dependencies/craft-libs/` (its `README.md` lists them and what changed); no engine dependency comes from GitHub. crates.io crates and the Android libraries still download, pinned by `Cargo.lock` and the Gradle files.
- **Debug mode on a device:**
  ```bash
  adb shell setprop log.tag.ETCDebug DEBUG
  ```
  ```bash
  adb logcat -s ETCDebug
  ```
  Turn it off with `INFO`.
- **Pen testing:** a real tablet with a stylus is needed for tilt, barrel rotation and pressure; the emulator doesn't pass those through.

## The VM

- **Set up (2026-11-27):** Debian 13 with XFCE on Unraid, with a fixed 10 GB of memory (ballooning had held it at 545 MB, too little to build the engine).
  - Android Studio is in `/opt/android-studio`, and the SDK is in `~/Android/Sdk` (API 35, build-tools 34, NDK 27.0.12077973).
  - Gradle 8.10.2 and JDK 21 are in `~/.local/opt`. Android Studio's bundled JDK is Java 25, which Gradle 8.10 can't run on. There is no Gradle wrapper.
  - Rust and cargo-ndk are installed with rustup. `~/.profile` puts cargo and `ANDROID_HOME` on the path.
  - Build with `JAVA_HOME=~/.local/opt/jdk-21 gradle assembleDebug` from a login shell.
  - The emulator is `Medium_Tablet_API_35`.
- **Emulator:** needs KVM inside the VM, so enable nested virtualisation on the host. Otherwise use a real tablet with `adb` (Wi-Fi debugging, or USB passed through).
- **Network:** with restricted outbound access, these hosts are needed:
  - Gradle and Android: `dl.google.com`, `maven.google.com`, `repo.maven.apache.org`, `plugins.gradle.org`, `services.gradle.org`
  - Rust: `static.rust-lang.org`, `index.crates.io`, `static.crates.io`
  - Git: `github.com`
  - Claude Code: `api.anthropic.com`, `claude.ai`, `downloads.claude.ai`

  This list may be incomplete; watch what gets refused on the first build.
- **Credentials:** keep the release keystore and its passwords off the VM. Use a GitHub credential that can push to this repository only.

## Owner's decisions (requirements; don't revisit)

- **Removed from the engine:** EPS, CAD and metafiles; scripts and the JavaScript runtime; plug-ins; the Actions panel (the command journal is kept); all print functions (`print*`, `preflight`, `overprint`; Vector Halftone is kept); web export, interactivity, CSS and variables; VectorCraft's own slices; `help`, `apiref`, `shortcutset`, `toast`; the `format` crate (the `.vectorcraft` serializer).
- **Kept:** `svg`; PDF import and export; Affinity and CorelDRAW import; every image decoder (HEIF, JPEG XL, JPEG 2000, PSD, TGA, WebP, GIF, BMP, JPEG, PNG, TIFF); `testkit`; `menucmds`, `docmenu`, `panelcmds`, `tabs`, `wireframe`, `views`, `prefscmds`.
- **Presets** are data files under `tools/Libraries/presets`, loaded at run time.
- **Tools:**
  - **Dial order:** Brush, Marker, Pencil, Pen, Eraser, Fill, Selection, Shapes, Move, Crop, Export.
  - **Brush:** raster painting into blocks the size of the view plus a buffer, reused when the view and zoom match within a tolerance. Its size is in screen pixels; Lock canvas keeps painting into one block, its size in that block's pixels.
  - **Marker:** vector strokes (Paintbrush or Blob Brush modes). Tips are round, oval or chisel (a true rectangle). The tip follows its angle, the pen's tilt or the barrel rotation, falling back to tilt when the pen reports no rotation.
  - **Pencil:** the Marker with pencil grain. Its tip is a cone that widens as the pen leans, or a mechanical circle.
  - **Fill:** on an infinite canvas with no closed area, it fills the current screen behind the artwork.
  - **Crop:** crops a selected image, otherwise the canvas.
  - **Hardness:** works through the Feather effect.
- **Layers:**
  - **Fill:** as in Photoshop, it fades the content but not its styles.
  - **Masks:** greyscale (white shows, black hides), shown as a sublayer. Tapping a mask with something selected makes it the selection.
  - **Palette:** app-wide.
  - **Textures:** come from `tools/Libraries/textures`, which the owner fills with freely licensed images.
  - **Filter layers:** apply to everything visually below them, across folders.
- **Artwork camera:** saved per document, including View Flip and rotation.
- **Reference board:** `.etcr`, separate from `.etc`.
  - Images are linked, or embedded inside the `.etcr` when the owner chooses Embed.
  - Tapping an image shows corner handles: on a handle resizes, just outside a corner rotates.
  - Holding an image opens Delete, Embed, Flip (Horizontal or Vertical) and Crop.
  - Two-finger pinch and twist always move the view, never an image.
- **Dial toolbar:** the dial's grip docks it to any of the four edges as a toolbar, where a tap chooses a tool.
