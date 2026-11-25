package com.extemporecreare.app.components.tools

import com.extemporecreare.app.components.engine.EngineBridge
import org.json.JSONArray

/**
 * A stroke tool's gesture as one undo step: the press opens an undo group named [label], the
 * engine tool draws, and on release the objects it added are finished (weight, feather) before the
 * group closes. Brush and Pen use it.
 */
class StrokeGesture(private val label: String) {

    /** The undo group of the gesture in progress. */
    private val group = ToolUndoGroup()

    /**
     * Feed [event] to the active engine tool. [beforeDraw] runs after the group opens, before the
     * press reaches the tool (an error from it cancels the gesture); [finish] gets the objects the
     * gesture added, on release.
     */
    fun pointer(
        engine: EngineBridge,
        event: ToolPointer,
        viewJson: String,
        beforeDraw: () -> String? = { null },
        finish: (JSONArray) -> String?,
    ): ToolResult {
        if (event.kind == ToolPointer.Kind.DOWN) {
            group.begin(engine, label)?.let { return ToolResult.failed(it) }
            beforeDraw()?.let {
                group.end(engine, cancel = true)
                return ToolResult.failed(it)
            }
        }
        val result = ToolResult.fromPointer(engine.pointer(event.toJson(), viewJson))
        if (event.kind != ToolPointer.Kind.UP || !group.isOpen) return result
        val finished = if (result.error == null) finish(result.created) else null
        val closed = group.end(engine)
        return result.copy(error = result.error ?: finished ?: closed)
    }

    /** Close a gesture left open (the tool was switched mid-stroke). */
    fun close(engine: EngineBridge) {
        group.end(engine)
    }
}

// GLOSSARY
// - StrokeGesture: a stroke tool's press-to-release gesture as one undo step, finished on release.
// - label: the undo step's name.
// - group: the gesture's undo group.
// - pointer(engine, event, viewJson, beforeDraw, finish): feed an event, opening and closing the group.
// - close(engine): close a gesture left open.
