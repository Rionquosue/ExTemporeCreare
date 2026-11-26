package com.extemporecreare.app.ui.document

import com.extemporecreare.app.components.documents.DocumentSettings

/** The New Document presets (`uistandards.md` §5.3): each fills the settings it names, which stay editable. */
val DOCUMENT_PRESETS: List<Pair<String, (DocumentSettings) -> DocumentSettings>> = listOf(
    "Infinite canvas" to { s -> s.copy(infinite = true) },
    "Screen 1920 × 1080" to { s -> s.copy(infinite = false, width = 1920.0, height = 1080.0, units = "px", ppi = 144.0, colorMode = "rgb") },
    "Tablet 2560 × 1600" to { s -> s.copy(infinite = false, width = 2560.0, height = 1600.0, units = "px", ppi = 144.0, colorMode = "rgb") },
    "Square 2048" to { s -> s.copy(infinite = false, width = 2048.0, height = 2048.0, units = "px", ppi = 144.0, colorMode = "rgb") },
    "A4 300 ppi" to { s -> s.copy(infinite = false, width = 210.0, height = 297.0, units = "mm", ppi = 300.0) },
    "Letter 300 ppi" to { s -> s.copy(infinite = false, width = 8.5, height = 11.0, units = "in", ppi = 300.0) },
)

// GLOSSARY
// - DOCUMENT_PRESETS: the New Document presets (label, settings change).
