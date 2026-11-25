package com.extemporecreare.app.tools.shapes

import com.extemporecreare.app.components.tools.ToolSettings
import org.json.JSONObject

/** The shapes the Shapes tool draws, with the engine tool for each. */
enum class ShapeKind(val engineToolId: String) {
    RECTANGLE("rectangle"),
    ELLIPSE("ellipse"),
    POLYGON("polygon"),
    STAR("star"),
    LINE("lineSegment"),
}

/** The Shapes tool's settings: the [kind] drawn, a polygon's [sides] (3 to 1000) and a star's [points] (3 to 1000). */
data class ShapesToolSettings(val kind: ShapeKind = ShapeKind.RECTANGLE, val sides: Int = 6, val points: Int = 5) : ToolSettings {

    override fun toJson(): JSONObject = JSONObject().put("kind", kind.name).put("sides", sides).put("points", points)

    companion object {
        /** The settings saved as [json]; missing or unreadable values take the defaults. */
        fun fromJson(json: JSONObject): ShapesToolSettings {
            val d = ShapesToolSettings()
            return ShapesToolSettings(
                kind = ShapeKind.entries.firstOrNull { it.name == json.optString("kind") } ?: d.kind,
                sides = json.optInt("sides", d.sides).coerceIn(3, 1000),
                points = json.optInt("points", d.points).coerceIn(3, 1000),
            )
        }
    }
}

// GLOSSARY
// - ShapeKind: RECTANGLE, ELLIPSE, POLYGON, STAR, LINE, with their engine tools.
// - ShapesToolSettings: kind, polygon sides and star points of the Shapes tool.
// - toJson(): the settings as JSON.
// - fromJson(json): the settings from JSON, defaults for what is missing.
