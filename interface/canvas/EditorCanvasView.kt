package com.extemporecreare.app.ui.canvas

import android.annotation.SuppressLint
import android.content.Context
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.Paint
import android.graphics.PorterDuff
import android.graphics.PorterDuffXfermode
import android.graphics.Rect
import android.os.Handler
import android.os.Looper
import android.view.MotionEvent
import android.view.View
import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.ToolPointer
import com.extemporecreare.app.tools.brush.BrushTool
import com.extemporecreare.app.tools.crop.CropTool
import com.extemporecreare.app.tools.export.ExportTool
import com.extemporecreare.app.ui.editor.EditorState
import java.util.concurrent.Executors
import kotlin.math.cos
import kotlin.math.hypot
import kotlin.math.sin

/**
 * The artwork canvas: the document rendered by the engine at the camera's zoom (on a worker thread;
 * while a render is pending the last one is drawn moved to the camera), with the active tool's
 * overlays, the Brush's live stroke, the Crop tool's canvas box and the Export tool's slices on
 * top. A stylus or one finger draws with the active tool; two fingers pan and zoom the view.
 */
@SuppressLint("ViewConstructor")
class EditorCanvasView(context: Context, private val state: EditorState) : View(context) {

    /** The last render, the camera and revision it was made for. */
    private var rendered: Bitmap? = null
    private var renderedCamera: Camera? = null
    private var renderedRevision = -1

    /** A render is under way. */
    private var rendering = false

    /** The renders' worker thread, and the main thread's handler. */
    private val worker = Executors.newSingleThreadExecutor()
    private val main = Handler(Looper.getMainLooper())

    /** The Brush's live stroke as a bitmap of its splotch, and the stroke it belongs to. */
    private var liveBitmap: Bitmap? = null
    private var liveFor: BrushTool.LiveStroke? = null

    /** What the current touch does: draw (pointer id) or navigate (two fingers' last centre and span). */
    private var drawing: Int? = null
    private var navigating: Triple<Float, Float, Float>? = null

    private val bitmapPaint = Paint(Paint.FILTER_BITMAP_FLAG)
    private val overlays = OverlayPainter()

    /** Follow the editor: refresh when the [revision] or [camera] the screen holds moved on. */
    fun follow(revision: Int, camera: Camera) {
        if (revision != renderedRevision || camera != renderedCamera) refresh()
    }

    /** Ask for a new render when the camera or the document changed. */
    fun refresh() {
        if (width == 0 || height == 0) return
        if (renderedCamera != state.camera || renderedRevision != state.revision) render()
        invalidate()
    }

    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) {
        state.onViewSize(w, h)
        renderedCamera = null
        refresh()
    }

    /** Render the document for the current camera and revision on the worker thread. */
    private fun render() {
        if (rendering) return
        rendering = true
        val (w, h, cam, rev) = Quad(width, height, state.camera, state.revision)
        worker.execute {
            val v = cam.viewport(w, h)
            val buffer = EngineBridge.renderBuffer(w, h)
            val ok = state.engine.render(v[0], v[1], w, h, cam.zoom, buffer)
            val bmp = if (ok) Bitmap.createBitmap(w, h, Bitmap.Config.ARGB_8888).also { buffer.rewind(); it.copyPixelsFromBuffer(buffer) } else null
            main.post {
                rendering = false
                if (bmp != null) {
                    rendered = bmp
                    renderedCamera = cam
                    renderedRevision = rev
                }
                refresh()
            }
        }
    }

    override fun onDraw(canvas: Canvas) {
        canvas.drawColor(SURROUND)
        val cam = state.camera
        // The last render, moved from its camera to the current one.
        val bmp = rendered
        val rc = renderedCamera
        if (bmp != null && rc != null) {
            canvas.save()
            val m = cam.matrix(width, height)
            val back = android.graphics.Matrix()
            rc.matrix(width, height).invert(back)
            back.postConcat(m)
            canvas.concat(back)
            canvas.drawBitmap(bmp, 0f, 0f, bitmapPaint)
            canvas.restore()
        }
        drawLiveStroke(canvas, cam)
        (state.activeTool as? CropTool)?.canvasBox?.let { b -> overlays.box(canvas, cam.toScreen(b.left, b.top, b.width, b.height, width, height), true) }
        (state.activeTool as? ExportTool)?.let { drawSlices(canvas, cam, it) }
        overlays.draw(canvas, state.engine.overlays(state.viewJson()), cam.matrix(width, height))
    }

    /** The Brush's live stroke: its splotch's dirty pixels copied in, the splotch drawn in place. */
    private fun drawLiveStroke(canvas: Canvas, cam: Camera) {
        val brush = state.activeTool as? BrushTool ?: return
        val live = brush.live ?: run {
            liveBitmap = null
            liveFor = null
            return
        }
        if (liveFor == null || liveFor?.pixelsWide != live.pixelsWide || liveFor?.pixelsHigh != live.pixelsHigh || liveFor?.x != live.x || liveFor?.y != live.y) {
            liveBitmap = Bitmap.createBitmap(live.pixelsWide.coerceAtLeast(1), live.pixelsHigh.coerceAtLeast(1), Bitmap.Config.ARGB_8888)
        }
        liveFor = live
        val target = liveBitmap ?: return
        live.dirty?.let { d ->
            val (x, y, w, h) = d.toList()
            if (w > 0 && h > 0) {
                val buffer = EngineBridge.renderBuffer(w, h)
                if (brush.livePixels(state.engine, x, y, w, h, buffer)) {
                    val part = Bitmap.createBitmap(w, h, Bitmap.Config.ARGB_8888)
                    buffer.rewind()
                    part.copyPixelsFromBuffer(buffer)
                    Canvas(target).drawBitmap(part, x.toFloat(), y.toFloat(), REPLACE)
                    part.recycle()
                }
            }
            brush.drawn()
        }
        canvas.drawBitmap(target, null, cam.toScreen(live.x, live.y, live.width, live.height, width, height), bitmapPaint)
    }

    /** The Export tool's slices, the automatic content slice, and the box being drawn or edited. */
    private fun drawSlices(canvas: Canvas, cam: Camera, tool: ExportTool) {
        for (s in tool.slices.slices) {
            if (!s.visible) continue
            overlays.box(canvas, cam.toScreen(s.x, s.y, s.width, s.height, width, height), s.id == tool.selectedId)
        }
        tool.autoSlice?.let { b -> overlays.box(canvas, cam.toScreen(b.left, b.top, b.width, b.height, width, height), false, dashed = true) }
        tool.editing?.let { b -> overlays.box(canvas, cam.toScreen(b.left, b.top, b.width, b.height, width, height), true) }
    }

    @SuppressLint("ClickableViewAccessibility")
    override fun onTouchEvent(e: MotionEvent): Boolean {
        when (e.actionMasked) {
            MotionEvent.ACTION_DOWN -> {
                drawing = e.getPointerId(0)
                send(ToolPointer.Kind.DOWN, e, 0)
            }
            MotionEvent.ACTION_POINTER_DOWN -> if (e.pointerCount == 2 && e.getToolType(0) != MotionEvent.TOOL_TYPE_STYLUS) {
                // A second finger: the touch navigates; a stroke the first finger began ends here.
                drawing?.let { id -> e.findPointerIndex(id).takeIf { it >= 0 }?.let { send(ToolPointer.Kind.UP, e, it) } }
                drawing = null
                navigating = centreAndSpan(e)
            }
            MotionEvent.ACTION_MOVE -> {
                val nav = navigating
                if (nav != null && e.pointerCount >= 2) {
                    val now = centreAndSpan(e)
                    var cam = state.camera.panned(now.first - nav.first, now.second - nav.second)
                    if (nav.third > 0f) cam = cam.zoomedAbout((now.third / nav.third).toDouble(), now.first, now.second, width, height)
                    state.camera = cam
                    navigating = now
                } else {
                    drawing?.let { id -> e.findPointerIndex(id).takeIf { it >= 0 }?.let { i -> sendHistory(e, i) } }
                }
            }
            MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> {
                drawing?.let { id -> e.findPointerIndex(id).takeIf { it >= 0 }?.let { send(ToolPointer.Kind.UP, e, it) } }
                drawing = null
                navigating = null
            }
            MotionEvent.ACTION_POINTER_UP -> if (navigating != null && e.pointerCount <= 2) navigating = null
        }
        refresh()
        return true
    }

    /** The two fingers' centre and the distance between them. */
    private fun centreAndSpan(e: MotionEvent): Triple<Float, Float, Float> {
        val (x0, y0, x1, y1) = listOf(e.getX(0), e.getY(0), e.getX(1), e.getY(1))
        return Triple((x0 + x1) / 2, (y0 + y1) / 2, hypot(x1 - x0, y1 - y0))
    }

    /** A drag's in-between samples, then its latest one. */
    private fun sendHistory(e: MotionEvent, i: Int) {
        for (k in 0 until e.historySize) {
            sendAt(ToolPointer.Kind.DRAG, e.getHistoricalX(i, k), e.getHistoricalY(i, k), e.getHistoricalPressure(i, k), e, i)
        }
        send(ToolPointer.Kind.DRAG, e, i)
    }

    private fun send(kind: ToolPointer.Kind, e: MotionEvent, i: Int) = sendAt(kind, e.getX(i), e.getY(i), e.getPressure(i), e, i)

    /** One event for the active tool, in document points, with a stylus's pressure and tilt. */
    private fun sendAt(kind: ToolPointer.Kind, sx: Float, sy: Float, pressure: Float, e: MotionEvent, i: Int) {
        val (x, y) = state.camera.toDoc(sx, sy, width, height)
        val stylus = e.getToolType(i) == MotionEvent.TOOL_TYPE_STYLUS
        var tiltX: Float? = null
        var tiltY: Float? = null
        if (stylus) {
            // Android: tilt from upright (radians) and orientation (radians, 0 pointing up, clockwise).
            val tilt = Math.toDegrees(e.getAxisValue(MotionEvent.AXIS_TILT, i).toDouble())
            val o = e.getAxisValue(MotionEvent.AXIS_ORIENTATION, i).toDouble()
            val mirror = if (state.camera.flipped) -1 else 1
            tiltX = (tilt * sin(o) * mirror).toFloat()
            tiltY = (-tilt * cos(o)).toFloat()
        }
        state.pointer(ToolPointer(kind, x, y, if (stylus) pressure.coerceIn(0f, 1f) else 1f, tiltX, tiltY))
    }

    override fun onDetachedFromWindow() {
        super.onDetachedFromWindow()
        worker.shutdown()
    }

    /** Four values. */
    private data class Quad(val w: Int, val h: Int, val camera: Camera, val revision: Int)

    companion object {
        /** The workspace round the artwork (`--bg`). */
        val SURROUND = Color.parseColor("#0C131B")

        /** Copies pixels as they are. */
        private val REPLACE = Paint().apply { xfermode = PorterDuffXfermode(PorterDuff.Mode.SRC) }
    }
}

// GLOSSARY
// - EditorCanvasView: the artwork canvas view: renders, draws tool feedback, routes touch to the tool or navigation.
// - rendered, renderedCamera, renderedRevision, rendering: the last render and whether one is under way.
// - worker, main: the render thread and the main thread's handler.
// - liveBitmap, liveFor: the Brush's live stroke bitmap and its stroke.
// - drawing, navigating: what the current touch does.
// - follow(revision, camera): refresh when the editor moved on.
// - refresh(): render again if needed and redraw.
// - render(): render the document on the worker thread.
// - onDraw: the render moved to the camera, the live stroke, the Crop box, Export slices, overlays.
// - drawLiveStroke(canvas, cam): copy the stroke's dirty pixels in and draw its splotch.
// - drawSlices(canvas, cam, tool): the Export tool's slices and boxes.
// - onTouchEvent: draw with one finger or a stylus; navigate with two fingers.
// - centreAndSpan(e), sendHistory(e, i), send(kind, e, i), sendAt(...): touch to tool events.
// - Quad: a render's size, camera and revision.
// - SURROUND: the workspace colour. REPLACE: a copy-as-is paint.
