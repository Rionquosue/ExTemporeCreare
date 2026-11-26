package com.extemporecreare.app.tools.brush

import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.EngineAnswer
import com.extemporecreare.app.components.tools.EngineTool
import com.extemporecreare.app.components.tools.ToolPointer
import com.extemporecreare.app.components.tools.ToolResult
import com.extemporecreare.app.components.tools.ToolSettingsStore
import org.json.JSONArray
import org.json.JSONObject
import java.nio.ByteBuffer

/**
 * The Brush tool: raster painting, as a raster painting app's brush paints; nothing it paints is
 * vector art. The moment the pen goes down the stroke's **splotch** is ready: a raster area the size
 * of the current view plus a buffer, at the zoom's resolution, or an existing splotch that already
 * covers the view at about this zoom. The stroke is painted dab by dab as the pen moves, its layer
 * ([live], [livePixels]) drawn by the canvas over the document, and kept in the splotch on release
 * as one undo step (engine `paint.stroke`'s live form through `EngineBridge.paintBegin` …
 * `paintEnd`).
 *
 * The size is in screen pixels. [setLocked] locks the splotch painted last as the canvas: every
 * stroke paints it whatever the view (it takes in the whole view when the view goes past it), and
 * the size is in its own pixels, so zooming no longer changes it. The engine's Hand tool stays
 * chosen underneath and gets no input. The canvas passes the view's frame in the view state as
 * `"viewport": {"x", "y", "width", "height"}` (document points) with its `"zoom"`.
 */
class BrushTool(store: ToolSettingsStore) : EngineTool<BrushToolSettings>(NAME, store, BrushToolSettings::fromJson) {

    /** A stroke being painted: its splotch's document area and pixel size, and the splotch pixels painted since the canvas last drew them. */
    data class LiveStroke(val x: Double, val y: Double, val width: Double, val height: Double, val pixelsWide: Int, val pixelsHigh: Int, val dirty: IntArray?)

    /** The stroke being painted, for the canvas to draw, or `null`. */
    var live: LiveStroke? = null
        private set

    /** The locked splotch's id, or `null` when the canvas isn't locked. */
    var locked: Long? = null
        private set

    /** The splotch the last stroke was kept in. */
    private var lastSplotch: Long? = null

    override fun engineToolId(): String = "hand"

    override fun pointer(engine: EngineBridge, event: ToolPointer, viewJson: String): ToolResult = when (event.kind) {
        ToolPointer.Kind.DOWN -> start(engine, event, viewJson)
        ToolPointer.Kind.DRAG -> if (live != null) paintTo(engine, event) else ToolResult.NONE
        ToolPointer.Kind.UP -> if (live != null) paintTo(engine, event).error?.let { cancel(engine); ToolResult.failed(it) } ?: finish(engine) else ToolResult.NONE
        ToolPointer.Kind.MOVE -> ToolResult.NONE
    }

    override fun deactivate(engine: EngineBridge): ToolResult {
        cancel(engine)
        return super.deactivate(engine)
    }

    /** Lock the splotch painted last as the canvas ([on]), or unlock it → an error message when there is none to lock. */
    fun setLocked(on: Boolean): ToolResult {
        if (!on) {
            locked = null
            return ToolResult.NONE
        }
        val id = lastSplotch ?: return ToolResult.failed("Paint a stroke first: the lock keeps the splotch painted last")
        locked = id
        return ToolResult.NONE
    }

    /**
     * Copy the live stroke's layer over splotch pixels ([x], [y], [width] × [height]) into [out] (a
     * direct buffer of `width × height × 4` bytes, premultiplied RGBA) → whether it was filled.
     */
    fun livePixels(engine: EngineBridge, x: Int, y: Int, width: Int, height: Int, out: ByteBuffer): Boolean =
        live != null && engine.paintPixels(x, y, width, height, out)

    /** The canvas drew what was painted: forget the dirty pixels. */
    fun drawn() {
        live = live?.copy(dirty = null)
    }

    /** The pen went down: get the stroke and its splotch ready and paint the first dab. */
    private fun start(engine: EngineBridge, event: ToolPointer, viewJson: String): ToolResult {
        cancel(engine)
        val view = runCatching { JSONObject(viewJson) }.getOrNull()
        val frame = view?.optJSONObject("viewport") ?: return ToolResult.failed("The brush needs the view's frame to make its canvas")
        val s = settings
        val params = JSONObject()
            .put("view", frame)
            .put("zoom", view.optDouble("zoom", 1.0))
            .put("size", s.size)
            .put("hardness", s.hardness)
            .put("spacing", s.spacing)
            .put("flow", s.flow)
            .put("opacity", s.opacity)
            .put("pressureSize", s.pressureSize)
        locked?.let { params.put("locked", it) }
        val begun = EngineAnswer(engine.paintBegin(params.toString()))
        val f = begun.okObject ?: run {
            // A locked splotch that is gone (deleted, undone) unlocks the canvas.
            if (locked != null) locked = null
            return ToolResult.failed(begun.error ?: "can't start the brush stroke")
        }
        live = LiveStroke(f.optDouble("x"), f.optDouble("y"), f.optDouble("width"), f.optDouble("height"), f.optInt("pixelsWide"), f.optInt("pixelsHigh"), null)
        return paintTo(engine, event)
    }

    /** Paint the stroke on to [event]'s point; the painted pixels join the dirty area for the canvas. */
    private fun paintTo(engine: EngineBridge, event: ToolPointer): ToolResult {
        val pts = JSONArray().put(JSONArray().put(event.x).put(event.y).put(event.pressure.toDouble()))
        val answer = EngineAnswer(engine.paintAdd(pts.toString()))
        val ok = answer.okObject ?: return ToolResult.failed(answer.error ?: "can't paint the stroke")
        val r = ok.optJSONArray("painted") ?: return ToolResult.NONE
        val now = IntArray(4) { r.optInt(it) }
        live = live?.let { l ->
            val d = l.dirty
            val merged = if (d == null) now else {
                val x0 = minOf(d[0], now[0])
                val y0 = minOf(d[1], now[1])
                intArrayOf(x0, y0, maxOf(d[0] + d[2], now[0] + now[2]) - x0, maxOf(d[1] + d[3], now[1] + now[3]) - y0)
            }
            l.copy(dirty = merged)
        }
        return ToolResult.NONE
    }

    /** The pen was released: keep the stroke in its splotch, one undo step. */
    private fun finish(engine: EngineBridge): ToolResult {
        live = null
        val answer = EngineAnswer(engine.paintEnd())
        val ok = answer.okObject ?: return ToolResult.failed(answer.error ?: "can't keep the brush stroke")
        lastSplotch = ok.optLong("id")
        val created = if (ok.optBoolean("created")) JSONArray().put(ok.optLong("id")) else JSONArray()
        return ToolResult(created = created, changed = true)
    }

    /** Drop a stroke left open. */
    private fun cancel(engine: EngineBridge) {
        if (live != null) {
            live = null
            engine.paintCancel()
        }
    }

    companion object {
        /** The tool's dial name and settings key. */
        const val NAME = "Brush"
    }
}

// GLOSSARY
// - BrushTool: the raster Brush tool, painting live strokes into splotches (view-sized raster areas), one undo step each.
// - LiveStroke: the live stroke's splotch area (document points), pixel size, and pixels painted since the canvas last drew.
// - live: the stroke being painted, for the canvas.
// - locked: the locked splotch's id, or null.
// - lastSplotch: the splotch the last stroke was kept in.
// - engineToolId(): "hand" (gets no input).
// - pointer(engine, event, viewJson): start, paint and keep the stroke.
// - deactivate(engine): drop a stroke left open.
// - setLocked(on): lock the splotch painted last as the canvas, or unlock.
// - livePixels(engine, x, y, width, height, out): the live stroke's layer for the canvas.
// - drawn(): forget the dirty pixels once drawn.
// - start(engine, event, viewJson): the pen went down: splotch ready, first dab.
// - paintTo(engine, event): paint on to a point, growing the dirty area.
// - finish(engine): keep the stroke, one undo step.
// - cancel(engine): drop a stroke left open.
// - NAME: "Brush".
