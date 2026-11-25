package com.extemporecreare.app.tools.crop

/** What the Crop tool crops, decided when it becomes active. */
enum class CropMode {
    /** The one selected placed image (the engine's Crop Image tool). */
    IMAGE,

    /** The finite canvas: an artboard's bounds. */
    CANVAS,

    /** Nothing: no image is selected and the canvas is infinite, or no document is open. */
    UNAVAILABLE,
}

// GLOSSARY
// - CropMode: IMAGE (the selected image), CANVAS (an artboard of a finite canvas) or UNAVAILABLE.
