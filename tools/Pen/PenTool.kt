package com.extemporecreare.app.tools.pen

import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.EngineTool
import com.extemporecreare.app.components.tools.StrokeFinish
import com.extemporecreare.app.components.tools.StrokeGesture
import com.extemporecreare.app.components.tools.ToolPointer
import com.extemporecreare.app.components.tools.ToolResult
import com.extemporecreare.app.components.tools.ToolSettingsStore

/**
 * The Pen tool, built on the engine's Pen tool: each tap or drag places an anchor, as one undo
 * step. The tap that starts a path gives it the size as its stroke weight and, below 100 %
 * hardness, a feather ([StrokeFinish]).
 */
class PenTool(store: ToolSettingsStore) : EngineTool<PenToolSettings>(NAME, store, PenToolSettings::fromJson) {

    /** The anchor placement in progress, as one undo step. */
    private val gesture = StrokeGesture(NAME)

    override fun engineToolId(): String = "pen"

    override fun pointer(engine: EngineBridge, event: ToolPointer, viewJson: String): ToolResult {
        val s = settings
        return gesture.pointer(engine, event, viewJson) { created ->
            StrokeFinish.apply(engine, created, s.size, StrokeFinish.featherRadius(s.size, s.hardness))
        }
    }

    override fun deactivate(engine: EngineBridge): ToolResult {
        gesture.close(engine)
        return super.deactivate(engine)
    }

    companion object {
        /** The tool's dial name and settings key. */
        const val NAME = "Pen"
    }
}

// GLOSSARY
// - PenTool: the Pen tool (engine Pen), one undo step per anchor; new paths get weight and feather.
// - gesture: the anchor placement in progress.
// - engineToolId(): "pen".
// - pointer(engine, event, viewJson): place anchors; finish a new path on release.
// - deactivate(engine): close a placement left open.
// - NAME: "Pen".
