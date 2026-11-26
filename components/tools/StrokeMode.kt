package com.extemporecreare.app.components.tools

/** How a stroke tool (Marker, Pencil) draws, with the engine tool each way uses. */
enum class StrokeMode(val engineToolId: String) {
    /** Stroked paths, the tip drawn by a Calligraphic brush (or a library brush). */
    PAINTBRUSH("paintbrush"),

    /** Filled shapes that merge as they are painted. */
    BLOB("blobBrush"),
}

// GLOSSARY
// - StrokeMode: PAINTBRUSH ("paintbrush") or BLOB ("blobBrush"), with the engine tool each uses.
