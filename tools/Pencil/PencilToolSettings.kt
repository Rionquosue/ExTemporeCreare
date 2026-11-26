package com.extemporecreare.app.tools.pencil

import com.extemporecreare.app.components.tools.StrokeMode
import com.extemporecreare.app.components.tools.ToolSettings
import org.json.JSONObject

/** The Pencil's tip. */
enum class PencilTip {
    /** A sharpened cone: where it meets the paper widens and stretches toward the lean as the pen tilts. */
    CONE,

    /** A hard round tip that stays the same whatever the pen does (a mechanical pencil's lead). */
    MECHANICAL,
}

/**
 * The Pencil tool's settings: its [mode], [size] in points (the line's width upright, 0.1 to 1000),
 * [hardness] in percent (below 100 the line gets a feather), the [tip] with its [coneAngle] (the
 * cone's half-angle in degrees, 1 to 80: smaller is sharper), and the paper grain's [grainScale]
 * (points, 0.05 to 50) and [grainStrength] (percent, 0 to 100; 0: no grain).
 */
data class PencilToolSettings(
    val mode: StrokeMode = StrokeMode.PAINTBRUSH,
    val size: Double = 1.0,
    val hardness: Double = 100.0,
    val tip: PencilTip = PencilTip.CONE,
    val coneAngle: Double = 20.0,
    val grainScale: Double = 0.6,
    val grainStrength: Double = 60.0,
) : ToolSettings {

    override fun toJson(): JSONObject = JSONObject()
        .put("mode", mode.name)
        .put("size", size)
        .put("hardness", hardness)
        .put("tip", tip.name)
        .put("coneAngle", coneAngle)
        .put("grainScale", grainScale)
        .put("grainStrength", grainStrength)

    companion object {
        /** The settings saved as [json]; missing or unreadable values take the defaults. */
        fun fromJson(json: JSONObject): PencilToolSettings {
            val d = PencilToolSettings()
            return PencilToolSettings(
                mode = StrokeMode.entries.firstOrNull { it.name == json.optString("mode") } ?: d.mode,
                size = json.optDouble("size", d.size).coerceIn(0.1, 1000.0),
                hardness = json.optDouble("hardness", d.hardness).coerceIn(0.0, 100.0),
                tip = PencilTip.entries.firstOrNull { it.name == json.optString("tip") } ?: d.tip,
                coneAngle = json.optDouble("coneAngle", d.coneAngle).coerceIn(1.0, 80.0),
                grainScale = json.optDouble("grainScale", d.grainScale).coerceIn(0.05, 50.0),
                grainStrength = json.optDouble("grainStrength", d.grainStrength).coerceIn(0.0, 100.0),
            )
        }
    }
}

// GLOSSARY
// - PencilTip: CONE (widens with the pen's tilt) or MECHANICAL (a hard fixed circle).
// - PencilToolSettings: mode, size (pt), hardness (%), tip, coneAngle (°), grainScale (pt) and grainStrength (%) of the Pencil tool.
// - toJson(): the settings as JSON.
// - fromJson(json): the settings from JSON, defaults for what is missing.
