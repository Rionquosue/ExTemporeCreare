package com.extemporecreare.app.tools.eraser

import com.extemporecreare.app.components.tools.ToolSettings
import org.json.JSONObject

/** The Eraser tool's settings: its tip [size] in points (0.1 to 1000). */
data class EraserToolSettings(val size: Double = 10.0) : ToolSettings {

    override fun toJson(): JSONObject = JSONObject().put("size", size)

    companion object {
        /** The settings saved as [json]; missing values take the defaults. */
        fun fromJson(json: JSONObject): EraserToolSettings =
            EraserToolSettings(json.optDouble("size", EraserToolSettings().size).coerceIn(0.1, 1000.0))
    }
}

// GLOSSARY
// - EraserToolSettings: the Eraser's tip size (pt).
// - toJson(): the settings as JSON.
// - fromJson(json): the settings from JSON, defaults for what is missing.
