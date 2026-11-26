package com.extemporecreare.app.tools.marker

import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.EngineAnswer
import com.extemporecreare.app.components.tools.EngineTool
import com.extemporecreare.app.components.tools.StrokeFinish
import com.extemporecreare.app.components.tools.StrokeGesture
import com.extemporecreare.app.components.tools.ToolPointer
import com.extemporecreare.app.components.tools.ToolResult
import com.extemporecreare.app.components.tools.ToolSettingsStore
import org.json.JSONObject

/**
 * The Marker tool, built on the engine's Paintbrush and Blob Brush tools ([MarkerMode]). Each stroke
 * is one undo step: with the Paintbrush it takes the brush's size as its stroke weight, and in both
 * modes a hardness below 100 % feathers it ([StrokeFinish]).
 */
class MarkerTool(store: ToolSettingsStore) : EngineTool<MarkerToolSettings>(NAME, store, MarkerToolSettings::fromJson) {

    /** The stroke in progress, as one undo step. */
    private val gesture = StrokeGesture(NAME)

    override fun engineToolId(): String = settings.mode.engineToolId

    override fun pushSettings(engine: EngineBridge): String? =
        if (settings.mode == MarkerMode.BLOB) setOption(engine, "size", settings.size) else null

    override fun pointer(engine: EngineBridge, event: ToolPointer, viewJson: String): ToolResult {
        val s = settings
        return gesture.pointer(engine, event, viewJson, beforeDraw = { useLibraryBrush(engine, s) }) { created ->
            val weight = if (s.mode == MarkerMode.PAINTBRUSH) s.size else null
            StrokeFinish.apply(engine, created, weight, StrokeFinish.featherRadius(s.size, s.hardness))
        }
    }

    override fun deactivate(engine: EngineBridge): ToolResult {
        gesture.close(engine)
        return super.deactivate(engine)
    }

    /**
     * For the Paintbrush with a library brush chosen: copy it into the document (once; the engine
     * skips a brush it has) and make it the brush the Paintbrush paints with; with none chosen, a
     * plain stroke → an error message, or `null`.
     */
    private fun useLibraryBrush(engine: EngineBridge, s: MarkerToolSettings): String? {
        if (s.mode != MarkerMode.PAINTBRUSH) return null
        val name = s.brushName
        val library = s.brushLibrary
        if (name != null && library != null) {
            val add = JSONObject().put("library", library).put("name", name)
            EngineAnswer(engine.execute("brush.addFromLibrary", add.toString())).error?.let { return it }
        }
        val current = JSONObject().put("name", name ?: JSONObject.NULL)
        return EngineAnswer(engine.execute("brush.setCurrent", current.toString())).error
    }

    companion object {
        /** The tool's dial name and settings key. */
        const val NAME = "Marker"
    }
}

// GLOSSARY
// - MarkerTool: the Marker tool (engine Paintbrush or Blob Brush), one undo step per stroke.
// - gesture: the stroke in progress.
// - engineToolId(): the engine tool of the current mode.
// - pushSettings(engine): the Blob Brush's tip size.
// - pointer(engine, event, viewJson): draw, then set the stroke weight and feather on release.
// - deactivate(engine): close a stroke left open.
// - useLibraryBrush(engine, s): copy the chosen library brush in and make it current.
// - NAME: "Marker".
