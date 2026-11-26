package com.extemporecreare.app.tools.marker

import com.extemporecreare.app.components.tools.StrokeMode
import com.extemporecreare.app.components.tools.ToolSettings
import org.json.JSONObject

/** The Marker's tip shapes, with the nib shape the engine draws each with. */
enum class MarkerTip(val nibShape: String) {
    /** A round tip. */
    ROUND("ellipse"),

    /** An oval tip, [MarkerToolSettings.tipThickness] percent as thick as it is long. */
    OVAL("ellipse"),

    /** A chisel tip: a flat rectangle with square corners, [MarkerToolSettings.tipThickness] percent as thick as it is long. */
    CHISEL("rectangle"),
}

/** What turns an oval or chisel tip, with the engine's names for it. */
enum class TipOrientation(val angleFrom: String, val calligraphicMode: String) {
    /** The tip stays at [MarkerToolSettings.tipAngle]. */
    FIXED("fixed", "fixed"),

    /** The tip turns with the way the pen leans, [MarkerToolSettings.tipAngle] added. */
    TILT("tilt", "bearing"),

    /** The tip turns with the pen's barrel rotation, [MarkerToolSettings.tipAngle] added. */
    ROTATION("rotation", "rotation"),
}

/**
 * The Marker tool's settings: its [mode], [size] in points (the stroke weight for the Paintbrush,
 * the tip size for the Blob Brush, 0.1 to 1000), [hardness] in percent (below 100 the stroke gets
 * a feather), the [tip] shape with its [tipAngle] (degrees counter-clockwise, −180 to 180),
 * [tipThickness] (percent of its length, 1 to 100, for oval and chisel tips) and [orientation], and,
 * for the Paintbrush, the library brush [brushName] from library [brushLibrary], which draws in
 * place of the tip when chosen.
 */
data class MarkerToolSettings(
    val mode: StrokeMode = StrokeMode.PAINTBRUSH,
    val size: Double = 5.0,
    val hardness: Double = 100.0,
    val brushLibrary: String? = null,
    val brushName: String? = null,
    val tip: MarkerTip = MarkerTip.ROUND,
    val tipAngle: Double = 45.0,
    val tipThickness: Double = 30.0,
    val orientation: TipOrientation = TipOrientation.FIXED,
) : ToolSettings {

    /** The tip's thickness over its length in percent: 100 for a round tip. */
    val tipRoundness: Double get() = if (tip == MarkerTip.ROUND) 100.0 else tipThickness

    override fun toJson(): JSONObject = JSONObject()
        .put("mode", mode.name)
        .put("size", size)
        .put("hardness", hardness)
        .put("brushLibrary", brushLibrary ?: JSONObject.NULL)
        .put("brushName", brushName ?: JSONObject.NULL)
        .put("tip", tip.name)
        .put("tipAngle", tipAngle)
        .put("tipThickness", tipThickness)
        .put("orientation", orientation.name)

    companion object {
        /** The settings saved as [json]; missing or unreadable values take the defaults. */
        fun fromJson(json: JSONObject): MarkerToolSettings {
            val d = MarkerToolSettings()
            return MarkerToolSettings(
                mode = StrokeMode.entries.firstOrNull { it.name == json.optString("mode") } ?: d.mode,
                size = json.optDouble("size", d.size).coerceIn(0.1, 1000.0),
                hardness = json.optDouble("hardness", d.hardness).coerceIn(0.0, 100.0),
                brushLibrary = json.optString("brushLibrary").takeIf { it.isNotEmpty() && !json.isNull("brushLibrary") },
                brushName = json.optString("brushName").takeIf { it.isNotEmpty() && !json.isNull("brushName") },
                tip = MarkerTip.entries.firstOrNull { it.name == json.optString("tip") } ?: d.tip,
                tipAngle = json.optDouble("tipAngle", d.tipAngle).coerceIn(-180.0, 180.0),
                tipThickness = json.optDouble("tipThickness", d.tipThickness).coerceIn(1.0, 100.0),
                orientation = TipOrientation.entries.firstOrNull { it.name == json.optString("orientation") } ?: d.orientation,
            )
        }
    }
}

// GLOSSARY
// - MarkerTip: ROUND, OVAL, CHISEL, with the engine nib shape of each.
// - TipOrientation: FIXED, TILT, ROTATION, with the engine's angleFrom and calligraphic mode names.
// - MarkerToolSettings: mode, size (pt), hardness (%), brushLibrary, brushName, tip, tipAngle, tipThickness and orientation of the Marker tool.
// - tipRoundness: the tip's thickness over its length, 100 for round.
// - toJson(): the settings as JSON.
// - fromJson(json): the settings from JSON, defaults for what is missing.
