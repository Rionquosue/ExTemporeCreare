package com.extemporecreare.app.tools.marker

import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.EngineAnswer
import com.extemporecreare.app.components.tools.EngineTool
import com.extemporecreare.app.components.tools.StrokeFinish
import com.extemporecreare.app.components.tools.StrokeGesture
import com.extemporecreare.app.components.tools.StrokeMode
import com.extemporecreare.app.components.tools.TipBrush
import com.extemporecreare.app.components.tools.ToolPointer
import com.extemporecreare.app.components.tools.ToolResult
import com.extemporecreare.app.components.tools.ToolSettingsStore
import org.json.JSONArray
import org.json.JSONObject

/**
 * The Marker tool, built on the engine's Paintbrush and Blob Brush tools ([StrokeMode]). Each stroke
 * is one undo step: with the Paintbrush it takes the size as its stroke weight, and in both modes a
 * hardness below 100 % feathers it ([StrokeFinish]).
 *
 * The tip ([MarkerTip]): round, oval or chisel, turned by its angle, the pen's tilt or its barrel
 * rotation ([TipOrientation]). With the Paintbrush an oval or chisel tip is the document's
 * Calligraphic brush [TIP_BRUSH], kept to the settings; with the Blob Brush it is the tool's own tip
 * (`angle`, `roundness`, `shape`, `angleFrom`).
 */
class MarkerTool(store: ToolSettingsStore) : EngineTool<MarkerToolSettings>(NAME, store, MarkerToolSettings::fromJson) {

    /** The stroke in progress, as one undo step. */
    private val gesture = StrokeGesture(NAME)

    override fun engineToolId(): String = settings.mode.engineToolId

    override fun pushSettings(engine: EngineBridge): String? {
        val s = settings
        if (s.mode != StrokeMode.BLOB) return null
        return setOption(engine, "size", s.size)
            ?: setOption(engine, "angle", s.tipAngle)
            ?: setOption(engine, "roundness", s.tipRoundness)
            ?: setOption(engine, "shape", s.tip.nibShape)
            ?: setOption(engine, "angleFrom", if (s.tip == MarkerTip.ROUND) TipOrientation.FIXED.angleFrom else s.orientation.angleFrom)
    }

    override fun pointer(engine: EngineBridge, event: ToolPointer, viewJson: String): ToolResult {
        val s = settings
        return gesture.pointer(engine, event, viewJson, beforeDraw = { usePaintbrushTip(engine, s) }) { created ->
            val weight = if (s.mode == StrokeMode.PAINTBRUSH) s.size else null
            StrokeFinish.apply(engine, created, weight, StrokeFinish.featherRadius(s.size, s.hardness))
        }
    }

    override fun deactivate(engine: EngineBridge): ToolResult {
        gesture.close(engine)
        return super.deactivate(engine)
    }

    /**
     * For the Paintbrush, the brush it paints with: a chosen library brush (copied into the document
     * once; the engine skips a brush it has), else for an oval or chisel tip the document's
     * [TIP_BRUSH] set to the tip, else a plain stroke → an error message, or `null`.
     */
    private fun usePaintbrushTip(engine: EngineBridge, s: MarkerToolSettings): String? {
        if (s.mode != StrokeMode.PAINTBRUSH) return null
        val library = s.brushLibrary
        val name = when {
            s.brushName != null && library != null -> {
                val add = JSONObject().put("library", library).put("name", s.brushName)
                EngineAnswer(engine.execute("brush.addFromLibrary", add.toString())).error?.let { return it }
                s.brushName
            }
            s.tip == MarkerTip.ROUND -> null
            else -> return tipBrush(engine, s)
        }
        val current = JSONObject().put("name", name ?: JSONObject.NULL)
        return EngineAnswer(engine.execute("brush.setCurrent", current.toString())).error
    }

    /**
     * Make the document's Calligraphic brush [TIP_BRUSH] the tip ([TipBrush]):
     * 1 pt long at 1 pt stroke weight, so the stroke weight sets its size; with the pen turning it,
     * its angle follows the pen's bearing or barrel rotation, [MarkerToolSettings.tipAngle] added →
     * an error message, or `null`.
     */
    private fun tipBrush(engine: EngineBridge, s: MarkerToolSettings): String? {
        val turns = s.orientation != TipOrientation.FIXED
        val params = JSONObject()
            .put("angle", s.tipAngle)
            .put("roundness", s.tipRoundness)
            .put("size", 1.0)
            .put("shape", s.tip.nibShape)
            .put("variation", JSONArray().put(if (turns) 180.0 else 0.0).put(0.0).put(0.0))
            .put("modes", JSONArray().put(s.orientation.calligraphicMode).put("fixed").put("fixed"))
        return TipBrush.use(engine, TIP_BRUSH, params)
    }

    companion object {
        /** The tool's dial name and settings key. */
        const val NAME = "Marker"

        /** The name of the document Calligraphic brush that draws the Paintbrush's oval and chisel tips. */
        const val TIP_BRUSH = "Marker Tip"
    }
}

// GLOSSARY
// - MarkerTool: the Marker tool (engine Paintbrush or Blob Brush), one undo step per stroke.
// - gesture: the stroke in progress.
// - engineToolId(): the engine tool of the current mode.
// - pushSettings(engine): the Blob Brush's size and tip (angle, roundness, shape, angleFrom).
// - pointer(engine, event, viewJson): draw with the Paintbrush's brush set first, then set the stroke weight and feather on release.
// - deactivate(engine): close a stroke left open.
// - usePaintbrushTip(engine, s): make the library brush, the tip brush or a plain stroke the Paintbrush's.
// - tipBrush(engine, s): make the document's Marker Tip Calligraphic brush the tip and current.
// - TIP_BRUSH: "Marker Tip".
// - NAME: "Marker".
