package com.extemporecreare.app.tools.brush

import com.extemporecreare.app.components.tools.ToolSettings
import org.json.JSONObject

/** The engine tool each Brush mode uses. */
enum class BrushMode(val engineToolId: String) {
    /** Stroked paths, painted with the chosen library brush (none: a plain stroke). */
    PAINTBRUSH("paintbrush"),

    /** Filled shapes that merge as they are painted. */
    BLOB("blobBrush"),
}

/**
 * The Brush tool's settings: its [mode], [size] in points (the stroke weight for the Paintbrush,
 * the tip size for the Blob Brush, 0.1 to 1000), [hardness] in percent (below 100 the stroke gets
 * a feather) and, for the Paintbrush, the library brush [brushName] from library [brushLibrary].
 */
data class BrushToolSettings(
    val mode: BrushMode = BrushMode.PAINTBRUSH,
    val size: Double = 5.0,
    val hardness: Double = 100.0,
    val brushLibrary: String? = null,
    val brushName: String? = null,
) : ToolSettings {

    override fun toJson(): JSONObject = JSONObject()
        .put("mode", mode.name)
        .put("size", size)
        .put("hardness", hardness)
        .put("brushLibrary", brushLibrary ?: JSONObject.NULL)
        .put("brushName", brushName ?: JSONObject.NULL)

    companion object {
        /** The settings saved as [json]; missing or unreadable values take the defaults. */
        fun fromJson(json: JSONObject): BrushToolSettings {
            val d = BrushToolSettings()
            return BrushToolSettings(
                mode = BrushMode.entries.firstOrNull { it.name == json.optString("mode") } ?: d.mode,
                size = json.optDouble("size", d.size).coerceIn(0.1, 1000.0),
                hardness = json.optDouble("hardness", d.hardness).coerceIn(0.0, 100.0),
                brushLibrary = json.optString("brushLibrary").takeIf { it.isNotEmpty() && !json.isNull("brushLibrary") },
                brushName = json.optString("brushName").takeIf { it.isNotEmpty() && !json.isNull("brushName") },
            )
        }
    }
}

// GLOSSARY
// - BrushMode: PAINTBRUSH ("paintbrush") or BLOB ("blobBrush"), with the engine tool each uses.
// - BrushToolSettings: mode, size (pt), hardness (%), brushLibrary and brushName of the Brush tool.
// - toJson(): the settings as JSON.
// - fromJson(json): the settings from JSON, defaults for what is missing.
