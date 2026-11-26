package com.extemporecreare.app.tools.brush

import com.extemporecreare.app.components.tools.ToolSettings
import org.json.JSONObject

/**
 * The Brush tool's settings, as a raster painting app's brush has them: the dab's [size] in points
 * (0.1 to 5000), its [hardness] (percent of the dab that is solid before it fades), the [spacing]
 * between dabs (percent of the dab's diameter, 1 to 1000), the [flow] each dab adds (percent, 1 to
 * 100), the most the stroke covers, [opacity] (percent, 1 to 100), and whether the pen's pressure
 * scales the dabs ([pressureSize]).
 */
data class BrushToolSettings(
    val size: Double = 20.0,
    val hardness: Double = 80.0,
    val spacing: Double = 25.0,
    val flow: Double = 100.0,
    val opacity: Double = 100.0,
    val pressureSize: Boolean = true,
) : ToolSettings {

    override fun toJson(): JSONObject = JSONObject()
        .put("size", size)
        .put("hardness", hardness)
        .put("spacing", spacing)
        .put("flow", flow)
        .put("opacity", opacity)
        .put("pressureSize", pressureSize)

    companion object {
        /** The settings saved as [json]; missing values take the defaults. */
        fun fromJson(json: JSONObject): BrushToolSettings {
            val d = BrushToolSettings()
            return BrushToolSettings(
                size = json.optDouble("size", d.size).coerceIn(0.1, 5000.0),
                hardness = json.optDouble("hardness", d.hardness).coerceIn(0.0, 100.0),
                spacing = json.optDouble("spacing", d.spacing).coerceIn(1.0, 1000.0),
                flow = json.optDouble("flow", d.flow).coerceIn(1.0, 100.0),
                opacity = json.optDouble("opacity", d.opacity).coerceIn(1.0, 100.0),
                pressureSize = json.optBoolean("pressureSize", d.pressureSize),
            )
        }
    }
}

// GLOSSARY
// - BrushToolSettings: size (pt), hardness, spacing, flow, opacity (%) and pressureSize of the raster Brush tool.
// - toJson(): the settings as JSON.
// - fromJson(json): the settings from JSON, defaults for what is missing.
