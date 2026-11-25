package com.extemporecreare.app.tools.export

/**
 * The units Export Settings shows dimensions in (`uistandards.md` §9.2). Document pixels are points,
 * 72 to the inch; switching units changes only the display, never the region (§9.3).
 */
enum class ExportUnits(val label: String, private val perInch: Double) {
    PX("px", 72.0),
    IN("in", 1.0),
    CM("cm", 2.54),
    MM("mm", 25.4);

    /** [px] document pixels in these units. */
    fun fromPx(px: Double): Double = px / 72.0 * perInch

    /** [value] in these units as document pixels. */
    fun toPx(value: Double): Double = value / perInch * 72.0
}

// GLOSSARY
// - ExportUnits: px, in, cm, mm (72 document pixels to the inch).
// - fromPx(px): document pixels in a unit.
// - toPx(value): a unit's value in document pixels.
