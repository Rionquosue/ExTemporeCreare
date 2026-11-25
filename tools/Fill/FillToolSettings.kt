package com.extemporecreare.app.tools.fill

import com.extemporecreare.app.components.tools.ToolSettings
import org.json.JSONObject

/**
 * The Fill tool's settings: its [tolerance] in points, the widest gap between paths that still
 * closes an area (0: only areas that are fully closed).
 */
data class FillToolSettings(val tolerance: Double = 0.0) : ToolSettings {

    override fun toJson(): JSONObject = JSONObject().put("tolerance", tolerance)

    companion object {
        /** The settings saved as [json]; missing values take the defaults. */
        fun fromJson(json: JSONObject): FillToolSettings =
            FillToolSettings(json.optDouble("tolerance", FillToolSettings().tolerance).coerceAtLeast(0.0))
    }
}

// GLOSSARY
// - FillToolSettings: the Fill tool's tolerance (largest gap closed, pt).
// - toJson(): the settings as JSON.
// - fromJson(json): the settings from JSON, defaults for what is missing.
