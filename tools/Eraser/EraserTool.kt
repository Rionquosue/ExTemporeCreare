package com.extemporecreare.app.tools.eraser

import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.EngineTool
import com.extemporecreare.app.components.tools.ToolSettingsStore

/** The Eraser tool, built on the engine's Eraser tool: each stroke erases as one undo step. */
class EraserTool(store: ToolSettingsStore) : EngineTool<EraserToolSettings>(NAME, store, EraserToolSettings::fromJson) {

    override fun engineToolId(): String = "eraser"

    override fun pushSettings(engine: EngineBridge): String? = setOption(engine, "size", settings.size)

    companion object {
        /** The tool's dial name and settings key. */
        const val NAME = "Eraser"
    }
}

// GLOSSARY
// - EraserTool: the Eraser tool (engine Eraser).
// - engineToolId(): "eraser".
// - pushSettings(engine): the tip size.
// - NAME: "Eraser".
