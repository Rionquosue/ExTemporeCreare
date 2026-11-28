# craft-libs

The crates the engine uses from storytold/craft-libs, copied in so the engine builds from this repository alone. The engine's `Cargo.toml` (`engine/Cargo.toml`, and `engine/crates/text/Cargo.toml` for craft-font-catalog) names them as path dependencies.

- **Source:** https://github.com/storytold/craft-libs at `d20a82481b1f7b1d32c818b19c620338af9755d3` (2026-11-21), as `cargo vendor` packaged it.
- **Licence:** MIT OR Apache-2.0 (`LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`, from craft-libs' root). The hayro and linesweeper crates carry their own licence files.
- **Crates (28):** craft-affinity, craft-cdr, craft-cms, craft-codepage, craft-dds, craft-docpath, craft-eps, craft-font-catalog, craft-gif, craft-image-codecs, craft-jpeg, craft-jxl, craft-metadata, craft-mime, craft-pdf, craft-proof, craft-psd, craft-stream-codecs, craft-svg, craft-tiff, craft-xml, hayro-cmap, hayro-interpret, hayro-jbig2, hayro-postscript, hayro-syntax, krilla, linesweeper.

## Changes from the source

1. Each crate's tests, benches, examples and dev-dependencies are left out.
2. Dependencies between the crates are paths to each other (`../<crate>`).
3. craft-svg uses the engine's own `vectorcraft-geom` and `vectorcraft-pathops` (`engine/crates/geom`, `engine/crates/pathops`) instead of an older copy of VectorCraft. Its clip-path union in `src/paths.rs` takes the engine's `try_boolean_n` predicate, which lists the paths a point is inside (`!inside.is_empty()`), instead of one flag per path.
4. Optional dependencies outside this folder that the engine doesn't turn on are removed: craft-eps' `vectorcraft-doc` and `vectorcraft-eps`, and craft-image-codecs' `craft-heif` and `craft-video`.
5. Each manifest allows warnings (`[lints.rust] warnings = "allow"`), as Cargo does for any crate it downloads.
