package com.extemporecreare.app.tools.pencil

import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.DrawsLive
import com.extemporecreare.app.components.tools.EngineTool
import com.extemporecreare.app.components.tools.LivePath
import com.extemporecreare.app.components.tools.LivePathRecorder
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
 * The Pencil tool: the Marker's strokes ([StrokeMode]: the engine's Paintbrush or Blob Brush) with
 * a pencil tip and the paper's grain. The tip ([PencilTip]) is a cone, computed where the cone meets
 * the paper as the pen leans (the engine's cone nib), or a hard mechanical circle. Each stroke is one
 * undo step: with the Paintbrush it takes the size as its stroke weight, a hardness below 100 %
 * feathers it, and the grain is the engine's Pencil Grain effect (`stylize.pencilGrain`), fixed to
 * the canvas and sharp at any zoom.
 */
class PencilTool(store: ToolSettingsStore) : EngineTool<PencilToolSettings>(NAME, store, PencilToolSettings::fromJson), DrawsLive {

    /** The stroke in progress as it shows on the canvas. */
    private val live = LivePathRecorder()
    override val livePath: LivePath? get() = live.path

    /** The stroke in progress, as one undo step. */
    private val gesture = StrokeGesture(NAME)

    override fun engineToolId(): String = settings.mode.engineToolId

    override fun pushSettings(engine: EngineBridge): String? {
        val s = settings
        if (s.mode != StrokeMode.BLOB) return null
        return setOption(engine, "size", s.size)
            ?: setOption(engine, "angle", 0.0)
            ?: setOption(engine, "roundness", 100.0)
            ?: setOption(engine, "shape", "ellipse")
            ?: setOption(engine, "angleFrom", "fixed")
            ?: setOption(engine, "cone", if (s.tip == PencilTip.CONE) s.coneAngle else JSONObject.NULL)
    }

    override fun pointer(engine: EngineBridge, event: ToolPointer, viewJson: String): ToolResult {
        val s = settings
        live.follow(event, s.size)
        return gesture.pointer(engine, event, viewJson, beforeDraw = { useTip(engine, s) }) { created ->
            val weight = if (s.mode == StrokeMode.PAINTBRUSH) s.size else null
            val grain = if (s.grainStrength > 0) {
                listOf(GRAIN_EFFECT to JSONObject().put("scale", s.grainScale).put("strength", s.grainStrength))
            } else {
                emptyList()
            }
            StrokeFinish.apply(engine, created, weight, StrokeFinish.featherRadius(s.size, s.hardness), grain)
        }
    }

    override fun deactivate(engine: EngineBridge): ToolResult {
        gesture.close(engine)
        live.clear()
        return super.deactivate(engine)
    }

    /**
     * For the Paintbrush: make the document's Calligraphic brush [TIP_BRUSH] the tip, round and 1 pt
     * across at 1 pt stroke weight (the stroke weight sets the size), with the cone's half-angle for
     * a cone tip → an error message, or `null`.
     */
    private fun useTip(engine: EngineBridge, s: PencilToolSettings): String? {
        if (s.mode != StrokeMode.PAINTBRUSH) return null
        val params = JSONObject()
            .put("angle", 0.0)
            .put("roundness", 100.0)
            .put("size", 1.0)
            .put("shape", "ellipse")
            .put("variation", JSONArray().put(0.0).put(0.0).put(0.0))
            .put("modes", JSONArray().put("fixed").put("fixed").put("fixed"))
            .put("cone", if (s.tip == PencilTip.CONE) s.coneAngle else JSONObject.NULL)
        return TipBrush.use(engine, TIP_BRUSH, params)
    }

    companion object {
        /** The tool's dial name and settings key. */
        const val NAME = "Pencil"

        /** The name of the document Calligraphic brush that draws the Paintbrush's pencil tip. */
        const val TIP_BRUSH = "Pencil Tip"

        /** The engine effect that gives a stroke the paper's grain. */
        const val GRAIN_EFFECT = "stylize.pencilGrain"
    }
}

// GLOSSARY
// - PencilTool: the Pencil tool (engine Paintbrush or Blob Brush with a cone or mechanical tip and pencil grain), one undo step per stroke.
// - gesture: the stroke in progress. live, livePath: the stroke as it shows while drawn.
// - engineToolId(): the engine tool of the current mode.
// - pushSettings(engine): the Blob Brush's size and round tip, with the cone for a cone tip.
// - pointer(engine, event, viewJson): draw with the tip set first, then set the weight, feather and grain on release.
// - deactivate(engine): close a stroke left open.
// - useTip(engine, s): make the document's Pencil Tip Calligraphic brush the tip and current.
// - NAME: "Pencil". TIP_BRUSH: "Pencil Tip". GRAIN_EFFECT: "stylize.pencilGrain".
