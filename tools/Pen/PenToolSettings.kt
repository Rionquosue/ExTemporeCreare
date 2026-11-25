package com.extemporecreare.app.tools.pen

import com.extemporecreare.app.components.tools.ToolSettings
import org.json.JSONObject

/** The Pen tool's settings: the new path's stroke weight [size] in points and [hardness] in percent. */
data class PenToolSettings(val size: Double = 1.0, val hardness: Double = 100.0) : ToolSettings {

    override fun toJson(): JSONObject = JSONObject().put("size", size).put("hardness", hardness)

    companion object {
        /** The settings saved as [json]; missing values take the defaults. */
        fun fromJson(json: JSONObject): PenToolSettings {
            val d = PenToolSettings()
            return PenToolSettings(
                size = json.optDouble("size", d.size).coerceIn(0.1, 1000.0),
                hardness = json.optDouble("hardness", d.hardness).coerceIn(0.0, 100.0),
            )
        }
    }
}

// GLOSSARY
// - PenToolSettings: size (stroke weight, pt) and hardness (%) of the Pen tool.
// - toJson(): the settings as JSON.
// - fromJson(json): the settings from JSON, defaults for what is missing.
