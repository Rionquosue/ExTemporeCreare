package com.extemporecreare.app.tools.crop

import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.DragBox
import com.extemporecreare.app.components.tools.EngineAnswer
import com.extemporecreare.app.components.tools.EngineTool
import com.extemporecreare.app.components.tools.NoToolSettings
import com.extemporecreare.app.components.tools.ToolPointer
import com.extemporecreare.app.components.tools.ToolResult
import com.extemporecreare.app.components.tools.ToolSettingsStore
import org.json.JSONArray
import org.json.JSONObject

/**
 * The Crop tool. With one placed image selected it crops that image, built on the engine's Crop
 * Image tool (its box, handles and dimmed outside; [apply] runs `object.cropImage`). Otherwise, on
 * a finite canvas, it crops the canvas: a [DragBox] starts on an artboard, a tap on another
 * artboard moves it there, and [apply] gives the artboard the box's bounds (`artboard.setProps`,
 * the art left where it is). An infinite canvas has no bounds to crop. Each apply is one undo step.
 */
class CropTool(store: ToolSettingsStore) : EngineTool<NoToolSettings>(NAME, store, { NoToolSettings }) {

    /** What the tool crops while active. */
    var mode: CropMode = CropMode.UNAVAILABLE
        private set

    /** The canvas crop box (canvas mode), for the canvas to draw. */
    var canvasBox: DragBox? = null
        private set

    /** The artboard the canvas crop box is on. */
    var canvasIndex: Int = 0
        private set

    /** The artboards of the document when the tool became active, as `[{"index", "x", "y", "width", "height"}]`. */
    private var artboards = JSONArray()

    /** The canvas box's grip held by the drag in progress, and where the drag was last. */
    private var held: Pair<DragBox.Grip, Pair<Double, Double>>? = null

    /** The engine tool for the mode: Crop Image for an image, else the Hand tool, which gets no input. */
    override fun engineToolId(): String = if (mode == CropMode.IMAGE) "cropImage" else "hand"

    override fun activate(engine: EngineBridge, viewJson: String): ToolResult {
        val state = EngineAnswer(engine.state()).okObject
        artboards = state?.optJSONArray("artboards") ?: JSONArray()
        mode = when {
            state == null || !state.optBoolean("document") -> CropMode.UNAVAILABLE
            (state.optJSONArray("images")?.length() ?: 0) == 1 && (state.optJSONArray("selection")?.length() ?: 0) == 1 -> CropMode.IMAGE
            state.optString("canvas") == "finite" && artboards.length() > 0 -> CropMode.CANVAS
            else -> CropMode.UNAVAILABLE
        }
        canvasBox = if (mode == CropMode.CANVAS) boxFor(0) else null
        held = null
        val result = super.activate(engine, viewJson)
        return if (mode == CropMode.UNAVAILABLE && result.error == null) {
            ToolResult.failed("Select one image to crop it; an infinite canvas has no bounds to crop")
        } else {
            result
        }
    }

    override fun pointer(engine: EngineBridge, event: ToolPointer, viewJson: String): ToolResult = when (mode) {
        CropMode.IMAGE -> super.pointer(engine, event, viewJson)
        CropMode.CANVAS -> canvasPointer(event, viewJson)
        CropMode.UNAVAILABLE -> ToolResult.NONE
    }

    /** Crop to the box: the image (image mode) or the artboard (canvas mode), as one undo step. */
    fun apply(engine: EngineBridge, viewJson: String = "{}"): ToolResult = when (mode) {
        CropMode.IMAGE -> {
            val options = EngineAnswer(engine.toolOptions()).okObject?.optJSONObject("options")
            val rect = options?.optJSONArray("rect") ?: return ToolResult.failed("There is no crop box")
            val crop = EngineAnswer(engine.execute("object.cropImage", JSONObject().put("rect", rect).toString()))
            if (!crop.isOk) ToolResult.failed(crop.error ?: "can't crop the image") else activate(engine, viewJson)
        }
        CropMode.CANVAS -> {
            val box = canvasBox ?: return ToolResult.failed("There is no crop box")
            val params = JSONObject()
                .put("index", canvasIndex)
                .put("x", box.left).put("y", box.top)
                .put("width", box.width).put("height", box.height)
                .put("moveArt", false).put("scaleArt", false)
            val set = EngineAnswer(engine.execute("artboard.setProps", params.toString()))
            if (!set.isOk) ToolResult.failed(set.error ?: "can't crop the canvas") else activate(engine, viewJson)
        }
        CropMode.UNAVAILABLE -> ToolResult.NONE
    }

    /** Put the box back where it started, cropping nothing. */
    fun cancel(engine: EngineBridge): ToolResult = when (mode) {
        CropMode.IMAGE -> ToolResult.failedOrNone(setOption(engine, "rect", JSONObject.NULL))
        CropMode.CANVAS -> {
            canvasBox = boxFor(canvasIndex)
            held = null
            ToolResult.NONE
        }
        CropMode.UNAVAILABLE -> ToolResult.NONE
    }

    /** Canvas mode: drag the box's edges, corners or inside; a tap on another artboard moves the box there. */
    private fun canvasPointer(event: ToolPointer, viewJson: String): ToolResult {
        val box = canvasBox ?: return ToolResult.NONE
        when (event.kind) {
            ToolPointer.Kind.DOWN -> {
                val grip = box.gripAt(event.x, event.y, DragBox.reach(REACH_PX, viewJson))
                if (grip != null) {
                    held = grip to (event.x to event.y)
                } else {
                    artboardAt(event.x, event.y)?.let { canvasBox = boxFor(it) }
                }
            }
            ToolPointer.Kind.DRAG, ToolPointer.Kind.UP -> {
                val (grip, at) = held ?: return ToolResult.NONE
                box.drag(grip, event.x - at.first, event.y - at.second)
                held = if (event.kind == ToolPointer.Kind.UP) null else grip to (event.x to event.y)
            }
            ToolPointer.Kind.MOVE -> {}
        }
        return ToolResult.NONE
    }

    /** A box on artboard [index] as it was when the tool became active (and [canvasIndex] set to it), or `null` when there is none. */
    private fun boxFor(index: Int): DragBox? {
        val a = artboards.optJSONObject(index) ?: return null
        canvasIndex = index
        return DragBox.of(a.optDouble("x"), a.optDouble("y"), a.optDouble("width"), a.optDouble("height"))
    }

    /** The index of the artboard under ([x], [y]), or `null`. */
    private fun artboardAt(x: Double, y: Double): Int? = (0 until artboards.length()).firstOrNull { i ->
        val a = artboards.getJSONObject(i)
        val ax = a.optDouble("x")
        val ay = a.optDouble("y")
        x in ax..ax + a.optDouble("width") && y in ay..ay + a.optDouble("height")
    }

    companion object {
        /** The tool's dial name and settings key. */
        const val NAME = "Crop"

        /** How near an edge or corner a press grips it, in screen pixels. */
        const val REACH_PX = 24.0
    }
}

// GLOSSARY
// - CropTool: the Crop tool: the selected image (engine Crop Image) or the finite canvas's artboard.
// - mode: what the tool crops while active (CropMode).
// - canvasBox: the canvas crop box, for drawing.
// - canvasIndex: the artboard the canvas box is on.
// - artboards: the document's artboards when the tool became active.
// - held: the grip and last position of the canvas box drag in progress.
// - engineToolId(): "cropImage" in image mode, else "hand" (gets no input).
// - activate(engine, viewJson): decide the mode and start the box.
// - pointer(engine, event, viewJson): the engine tool (image) or the canvas box drag.
// - apply(engine, viewJson): crop the image (object.cropImage) or the artboard (artboard.setProps).
// - cancel(engine): put the box back.
// - canvasPointer(event, viewJson): drag the canvas box, or move it to the artboard tapped.
// - boxFor(index): a box on an artboard (sets canvasIndex).
// - artboardAt(x, y): the artboard under a point.
// - NAME: "Crop".
// - REACH_PX: the grip reach in screen pixels, 24.
