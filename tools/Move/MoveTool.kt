package com.extemporecreare.app.tools.move

import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.EngineAnswer
import com.extemporecreare.app.components.tools.EngineTool
import com.extemporecreare.app.components.tools.NoToolSettings
import com.extemporecreare.app.components.tools.ToolPointer
import com.extemporecreare.app.components.tools.ToolResult
import com.extemporecreare.app.components.tools.ToolSettingsStore
import com.extemporecreare.app.components.tools.ToolUndoGroup
import org.json.JSONObject

/**
 * The Move tool: dragging anywhere on the canvas moves the selected objects by the drag
 * (`object.move`), without changing the selection; the whole drag is one undo step. The engine's
 * Selection tool stays chosen underneath so its selection frame is drawn; it gets no input.
 */
class MoveTool(store: ToolSettingsStore) : EngineTool<NoToolSettings>(NAME, store, { NoToolSettings }) {

    /** The drag's undo group. */
    private val group = ToolUndoGroup()

    /** Where the drag was at its last event, or `null` when no drag is moving the selection. */
    private var last: Pair<Double, Double>? = null

    override fun engineToolId(): String = "selection"

    override fun pointer(engine: EngineBridge, event: ToolPointer, viewJson: String): ToolResult = when (event.kind) {
        ToolPointer.Kind.DOWN -> start(engine, event)
        ToolPointer.Kind.DRAG -> moveTo(engine, event)
        ToolPointer.Kind.UP -> moveTo(engine, event).also { stop(engine) }
        ToolPointer.Kind.MOVE -> ToolResult.NONE
    }

    override fun deactivate(engine: EngineBridge): ToolResult {
        stop(engine)
        return super.deactivate(engine)
    }

    /** Start a drag at [event] when something is selected. */
    private fun start(engine: EngineBridge, event: ToolPointer): ToolResult {
        stop(engine)
        val state = EngineAnswer(engine.state()).okObject ?: return ToolResult.failed("no document is open")
        if ((state.optJSONArray("selection")?.length() ?: 0) == 0) return ToolResult.failed("Nothing is selected to move")
        group.begin(engine, NAME)?.let { return ToolResult.failed(it) }
        last = event.x to event.y
        return ToolResult.NONE
    }

    /** Move the selection by how far the drag went since its last event. */
    private fun moveTo(engine: EngineBridge, event: ToolPointer): ToolResult {
        val (x, y) = last ?: return ToolResult.NONE
        val dx = event.x - x
        val dy = event.y - y
        if (dx == 0.0 && dy == 0.0) return ToolResult.NONE
        val moved = EngineAnswer(engine.execute("object.move", JSONObject().put("dx", dx).put("dy", dy).toString()))
        if (!moved.isOk) {
            group.end(engine, cancel = true)
            last = null
            return ToolResult.failed(moved.error ?: "can't move the selection")
        }
        last = event.x to event.y
        return ToolResult.NONE
    }

    /** End the drag, keeping what it moved as one undo step. */
    private fun stop(engine: EngineBridge) {
        last = null
        group.end(engine)
    }

    companion object {
        /** The tool's dial name and settings key. */
        const val NAME = "Move"
    }
}

// GLOSSARY
// - MoveTool: the Move tool, moving the selection by a drag as one undo step.
// - group: the drag's undo group.
// - last: the drag's last position, or null.
// - engineToolId(): "selection" (draws the selection frame; gets no input).
// - pointer(engine, event, viewJson): start, move and end the drag.
// - deactivate(engine): end a drag left open.
// - start(engine, event): start a drag when something is selected.
// - moveTo(engine, event): move the selection by the drag's step.
// - stop(engine): end the drag.
// - NAME: "Move".
