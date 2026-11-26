package com.extemporecreare.app.tools.brush

import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.EngineAnswer
import com.extemporecreare.app.components.tools.EngineTool
import com.extemporecreare.app.components.tools.ToolPointer
import com.extemporecreare.app.components.tools.ToolResult
import com.extemporecreare.app.components.tools.ToolSettingsStore
import org.json.JSONArray
import org.json.JSONObject

/**
 * The Brush tool: raster painting, as a raster painting app's brush paints. A stroke's points are
 * gathered while it is drawn ([stroke], for the canvas to preview) and on release painted with the
 * engine's `paint.stroke`: round dabs, as soft as the hardness and as far apart as the spacing,
 * building up by their flow to at most the opacity, into a **paint block**: the block under the
 * stroke's start, else a new one at the resolution of the zoom it is started at. A block grows to
 * take in paint past its edges. One undo step per stroke. The engine's Hand tool stays chosen
 * underneath and gets no input.
 */
class BrushTool(store: ToolSettingsStore) : EngineTool<BrushToolSettings>(NAME, store, BrushToolSettings::fromJson) {

    /** The stroke being drawn: its points in document points, with pressure. */
    val stroke: List<ToolPointer> get() = points

    /** The points gathered since the press. */
    private val points = mutableListOf<ToolPointer>()

    override fun engineToolId(): String = "hand"

    override fun pointer(engine: EngineBridge, event: ToolPointer, viewJson: String): ToolResult {
        when (event.kind) {
            ToolPointer.Kind.DOWN -> {
                points.clear()
                points.add(event)
            }
            ToolPointer.Kind.DRAG -> if (points.isNotEmpty()) points.add(event)
            ToolPointer.Kind.UP -> if (points.isNotEmpty()) {
                points.add(event)
                return paint(engine, viewJson).also { points.clear() }
            }
            ToolPointer.Kind.MOVE -> {}
        }
        return ToolResult.NONE
    }

    override fun deactivate(engine: EngineBridge): ToolResult {
        points.clear()
        return super.deactivate(engine)
    }

    /** Paint the gathered stroke into a block, a new one at the view's zoom → the result. */
    private fun paint(engine: EngineBridge, viewJson: String): ToolResult {
        val s = settings
        val zoom = runCatching { JSONObject(viewJson).optDouble("zoom", 1.0) }.getOrDefault(1.0)
        val pts = JSONArray()
        for (p in points) pts.put(JSONArray().put(p.x).put(p.y).put(p.pressure.toDouble()))
        val params = JSONObject()
            .put("points", pts)
            .put("size", s.size)
            .put("hardness", s.hardness)
            .put("spacing", s.spacing)
            .put("flow", s.flow)
            .put("opacity", s.opacity)
            .put("pressureSize", s.pressureSize)
            .put("resolution", zoom.coerceIn(MIN_RESOLUTION, MAX_RESOLUTION))
        val answer = EngineAnswer(engine.execute("paint.stroke", params.toString()))
        val ok = answer.okObject ?: return ToolResult.failed(answer.error ?: "can't paint the stroke")
        return ToolResult(created = if (ok.optBoolean("created")) JSONArray().put(ok.opt("id")) else JSONArray(), changed = true)
    }

    companion object {
        /** The tool's dial name and settings key. */
        const val NAME = "Brush"

        /** The lowest and highest resolution a new paint block takes, pixels per point (the engine's range). */
        const val MIN_RESOLUTION = 0.25
        const val MAX_RESOLUTION = 64.0
    }
}

// GLOSSARY
// - BrushTool: the raster Brush tool, painting strokes into growing paint blocks (paint.stroke), one undo step each.
// - stroke: the stroke being drawn, for the canvas's preview.
// - points: the points gathered since the press.
// - engineToolId(): "hand" (gets no input).
// - pointer(engine, event, viewJson): gather the stroke; paint it on release.
// - deactivate(engine): drop a stroke left open.
// - paint(engine, viewJson): paint the stroke into a block at the view's zoom.
// - NAME: "Brush". MIN_RESOLUTION, MAX_RESOLUTION: 0.25 and 64 px per point.
