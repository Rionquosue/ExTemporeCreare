package com.extemporecreare.app.tools.export

import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.DragBox
import com.extemporecreare.app.components.tools.EngineAnswer
import org.json.JSONObject

/**
 * The engine's export slice commands (`export.slice.*`, `export.grid.set`), each change one undo
 * step. Each call answers the engine's result object, or throws [IllegalStateException] with the
 * engine's message.
 */
object ExportSliceCommands {

    /** Run [command] with [params] → its result object. */
    private fun run(engine: EngineBridge, command: String, params: JSONObject): JSONObject {
        val answer = EngineAnswer(engine.execute(command, params.toString()))
        return answer.okObject ?: throw IllegalStateException(answer.error ?: "$command failed")
    }

    /** The slices, the grid and the content bounds. */
    fun list(engine: EngineBridge): ExportSliceList = ExportSliceList.fromJson(run(engine, "export.slice.list", JSONObject()))

    /** Add a slice over [box] → its id. */
    fun add(engine: EngineBridge, box: DragBox): String =
        run(engine, "export.slice.add", rect(box)).getString("id")

    /** Give slice [id] the region [box]. */
    fun move(engine: EngineBridge, id: String, box: DragBox) {
        run(engine, "export.slice.update", rect(box).put("id", id))
    }

    /** Change slice [id]'s output fields in [fields] (`name`, `aspectRatio`, `format`, `scale`, `exportProfileRef`, `visible`, and `x`, `y`, `width`, `height`). */
    fun update(engine: EngineBridge, id: String, fields: JSONObject) {
        run(engine, "export.slice.update", JSONObject(fields.toString()).put("id", id))
    }

    /** Remove slice [id]. */
    fun delete(engine: EngineBridge, id: String) {
        run(engine, "export.slice.delete", JSONObject().put("id", id))
    }

    /** Copy slice [id] → the copy's id. */
    fun duplicate(engine: EngineBridge, id: String): String =
        run(engine, "export.slice.duplicate", JSONObject().put("id", id)).getString("id")

    /** Set the export grid to [grid]. */
    fun setGrid(engine: EngineBridge, grid: ExportGrid) {
        val params = JSONObject().put("visible", grid.visible).put("snapEnabled", grid.snapEnabled).put("spacingPx", grid.spacingPx)
        run(engine, "export.grid.set", params)
    }

    /** [box] as `{x, y, width, height}`. */
    private fun rect(box: DragBox): JSONObject =
        JSONObject().put("x", box.left).put("y", box.top).put("width", box.width).put("height", box.height)
}

// GLOSSARY
// - ExportSliceCommands: the engine's export slice and grid commands.
// - run(engine, command, params): run a command → its result object, or throw.
// - list(engine): slices, grid and content bounds.
// - add(engine, box): add a slice → id.
// - move(engine, id, box): set a slice's region.
// - update(engine, id, fields): change a slice's fields.
// - delete(engine, id): remove a slice.
// - duplicate(engine, id): copy a slice → id.
// - setGrid(engine, grid): set the export grid.
// - rect(box): a region as JSON.
