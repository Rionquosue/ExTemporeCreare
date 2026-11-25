package com.extemporecreare.app.tools.export

/** What an export writes (`uistandards.md` §9.2 Scope). */
enum class ExportScope(val label: String) {
    /** The automatic content-following slice (§9.4). */
    AUTO_CONTENT("Auto Content Slice"),

    /** The finite canvas's artboards; unavailable for an infinite canvas. */
    WHOLE_DOCUMENT("Whole Document"),

    /** The selected slice. */
    SELECTED_SLICE("Selected Slice"),

    /** Every visible slice. */
    ALL_SLICES("All Slices"),
}

// GLOSSARY
// - ExportScope: AUTO_CONTENT, WHOLE_DOCUMENT, SELECTED_SLICE, ALL_SLICES, with their labels.
