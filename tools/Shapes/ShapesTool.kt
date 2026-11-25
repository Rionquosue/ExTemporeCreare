package com.extemporecreare.app.tools.shapes

import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.EngineTool
import com.extemporecreare.app.components.tools.ToolSettingsStore

/**
 * The Shapes tool, built on the engine's Rectangle, Ellipse, Polygon, Star and Line Segment tools
 * ([ShapeKind]): dragging draws the chosen shape as one undo step.
 */
class ShapesTool(store: ToolSettingsStore) : EngineTool<ShapesToolSettings>(NAME, store, ShapesToolSettings::fromJson) {

    override fun engineToolId(): String = settings.kind.engineToolId

    override fun pushSettings(engine: EngineBridge): String? = when (settings.kind) {
        ShapeKind.POLYGON -> setOption(engine, "sides", settings.sides)
        ShapeKind.STAR -> setOption(engine, "points", settings.points)
        else -> null
    }

    companion object {
        /** The tool's dial name and settings key. */
        const val NAME = "Shapes"
    }
}

// GLOSSARY
// - ShapesTool: the Shapes tool (engine shape tools).
// - engineToolId(): the engine tool of the chosen kind.
// - pushSettings(engine): polygon sides or star points.
// - NAME: "Shapes".
