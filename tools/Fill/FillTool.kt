package com.extemporecreare.app.tools.fill

import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.EngineAnswer
import com.extemporecreare.app.components.tools.EngineTool
import com.extemporecreare.app.components.tools.ToolPointer
import com.extemporecreare.app.components.tools.ToolResult
import com.extemporecreare.app.components.tools.ToolSettingsStore
import com.extemporecreare.app.components.tools.ToolUndoGroup
import org.json.JSONArray
import org.json.JSONObject

/**
 * The Fill tool, built on the engine's Fill Area tool: a tap fills the area enclosed by paths
 * around it with the current fill. The tolerance is Fill Area's gap detection: gaps up to that
 * length close the area.
 *
 * Failsafe on an infinite canvas: when the tap finds no closed area (the document is unchanged),
 * only the current screen is filled, as a rectangle of the screen's bounds with the current fill
 * and no stroke, at the back of the active layer, behind the artwork; one undo step. The canvas
 * passes the screen's bounds in the view state as `"viewport": {"x", "y", "width", "height"}`, in
 * document points.
 */
class FillTool(store: ToolSettingsStore) : EngineTool<FillToolSettings>(NAME, store, FillToolSettings::fromJson) {

    override fun engineToolId(): String = "fillArea"

    override fun pushSettings(engine: EngineBridge): String? {
        val tolerance = settings.tolerance
        setOption(engine, "gapDetection", tolerance > 0)?.let { return it }
        return if (tolerance > 0) setOption(engine, "gapLength", tolerance) else null
    }

    override fun pointer(engine: EngineBridge, event: ToolPointer, viewJson: String): ToolResult {
        val result = super.pointer(engine, event, viewJson)
        if (event.kind != ToolPointer.Kind.UP || result.error != null || result.changed) return result
        val state = EngineAnswer(engine.state()).okObject ?: return result
        if (state.optString("canvas") != "infinite") return result
        return fillScreen(engine, viewJson)?.let { result.copy(error = it) } ?: result.copy(changed = true)
    }

    /** Fill the screen's bounds ([viewJson]'s `viewport`) behind the active layer's artwork, as one undo step → an error message, or `null`. */
    private fun fillScreen(engine: EngineBridge, viewJson: String): String? {
        val v = runCatching { JSONObject(viewJson).optJSONObject("viewport") }.getOrNull()
            ?: return "No closed area here, and the screen's bounds are unknown"
        val width = v.optDouble("width")
        val height = v.optDouble("height")
        if (!(width > 0 && height > 0)) return "No closed area here, and the screen's bounds are unknown"
        val group = ToolUndoGroup()
        group.begin(engine, NAME)?.let { return it }
        val rect = JSONObject().put("x", v.optDouble("x")).put("y", v.optDouble("y")).put("width", width).put("height", height)
        val made = EngineAnswer(engine.execute("shape.rectangle", rect.toString()))
        val id = made.okObject?.opt("id")
        val error = if (id == null) {
            made.error ?: "can't fill the screen"
        } else {
            val ids = JSONArray().put(id)
            EngineAnswer(engine.execute("paint.setStroke", JSONObject().put("ids", ids).put("none", true).toString())).error
                ?: EngineAnswer(engine.execute("select.set", JSONObject().put("ids", ids).toString())).error
                ?: EngineAnswer(engine.execute("object.arrange.sendToBack", "{}")).error
        }
        group.end(engine, cancel = error != null)
        return error
    }

    companion object {
        /** The tool's dial name and settings key. */
        const val NAME = "Fill"
    }
}

// GLOSSARY
// - FillTool: the Fill tool (engine Fill Area).
// - engineToolId(): "fillArea".
// - pushSettings(engine): tolerance as gap detection and gap length.
// - pointer(engine, event, viewJson): Fill Area, with the infinite-canvas failsafe on a release that changed nothing.
// - fillScreen(engine, viewJson): fill the screen's bounds behind the artwork, one undo step.
// - NAME: "Fill".
