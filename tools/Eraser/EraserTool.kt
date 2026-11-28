package com.extemporecreare.app.tools.eraser

import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.DrawsLive
import com.extemporecreare.app.components.tools.EngineTool
import com.extemporecreare.app.components.tools.LivePath
import com.extemporecreare.app.components.tools.LivePathRecorder
import com.extemporecreare.app.components.tools.ToolPointer
import com.extemporecreare.app.components.tools.ToolResult
import com.extemporecreare.app.components.tools.ToolSettingsStore

/** The Eraser tool, built on the engine's Eraser tool: each stroke erases as one undo step. */
class EraserTool(store: ToolSettingsStore) : EngineTool<EraserToolSettings>(NAME, store, EraserToolSettings::fromJson), DrawsLive {

    /** The stroke in progress as it shows on the canvas. */
    private val live = LivePathRecorder()
    override val livePath: LivePath? get() = live.path

    override fun engineToolId(): String = "eraser"

    override fun pushSettings(engine: EngineBridge): String? = setOption(engine, "size", settings.size)

    override fun pointer(engine: EngineBridge, event: ToolPointer, viewJson: String): ToolResult {
        live.follow(event, settings.size, erases = true)
        return super.pointer(engine, event, viewJson)
    }

    override fun deactivate(engine: EngineBridge): ToolResult {
        live.clear()
        return super.deactivate(engine)
    }

    companion object {
        /** The tool's dial name and settings key. */
        const val NAME = "Eraser"
    }
}

// GLOSSARY
// - EraserTool: the Eraser tool (engine Eraser).
// - engineToolId(): "eraser".
// - pushSettings(engine): the tip size.
// - live, livePath: the stroke as it shows while drawn. pointer(engine, event, viewJson): records it, then erases. deactivate(engine): drops it.
// - NAME: "Eraser".
