package com.extemporecreare.app.ui.canvas

import android.graphics.Canvas
import android.graphics.Color
import android.graphics.DashPathEffect
import android.graphics.Matrix
import android.graphics.Paint
import android.graphics.RectF
import androidx.core.graphics.PathParser
import com.extemporecreare.app.components.tools.EngineAnswer
import org.json.JSONArray
import org.json.JSONObject

/** Draws the active tool's overlays (`EngineBridge.overlays`) and the canvas's boxes (Crop, Export slices). */
class OverlayPainter {

    private val stroke = Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.STROKE; strokeWidth = 1.5f }
    private val fill = Paint(Paint.ANTI_ALIAS_FLAG).apply { style = Paint.Style.FILL }
    private val text = Paint(Paint.ANTI_ALIAS_FLAG).apply { textSize = 28f }
    private val dash = DashPathEffect(floatArrayOf(8f, 6f), 0f)

    /** Draw the overlays in [answer] with document → screen [m]. */
    fun draw(canvas: Canvas, answer: String, m: Matrix) {
        val list = EngineAnswer(answer).let { a -> if (a.isOk) runCatching { JSONObject(answer).optJSONArray("ok") }.getOrNull() else null } ?: return
        for (i in 0 until list.length()) {
            val o = list.optJSONObject(i) ?: continue
            when (o.optString("type")) {
                "path" -> path(canvas, o, m)
                "marquee" -> {
                    val r = RectF(o.optDouble("x").toFloat(), o.optDouble("y").toFloat(), (o.optDouble("x") + o.optDouble("width")).toFloat(), (o.optDouble("y") + o.optDouble("height")).toFloat())
                    m.mapRect(r)
                    box(canvas, r, false, dashed = true)
                }
                "line", "gridLine", "distance" -> line(canvas, o, m)
                "anchor" -> point(canvas, o, m, square = true)
                "handle", "curvePoint" -> point(canvas, o, m, square = false)
                "label", "measure" -> label(canvas, o, m)
            }
        }
    }

    /** A box on screen, gold when [active], dashed when [dashed]. */
    fun box(canvas: Canvas, r: RectF, active: Boolean, dashed: Boolean = false) {
        stroke.color = if (active) GOLD else TEAL
        stroke.pathEffect = if (dashed) dash else null
        canvas.drawRect(r, stroke)
        stroke.pathEffect = null
    }

    private fun colorOf(o: JSONObject, fallback: Int = TEAL): Int {
        val c = o.optJSONArray("color") ?: return fallback
        return Color.argb(if (c.length() > 3) c.optInt(3) else 255, c.optInt(0), c.optInt(1), c.optInt(2))
    }

    private fun pt(a: JSONArray?, m: Matrix): FloatArray {
        val p = floatArrayOf(a?.optDouble(0)?.toFloat() ?: 0f, a?.optDouble(1)?.toFloat() ?: 0f)
        m.mapPoints(p)
        return p
    }

    private fun path(canvas: Canvas, o: JSONObject, m: Matrix) {
        val p = runCatching { PathParser.createPathFromPathData(o.optString("d")) }.getOrNull() ?: return
        p.transform(m)
        stroke.color = colorOf(o)
        stroke.strokeWidth = o.optDouble("width", 1.0).toFloat().coerceAtLeast(1f)
        stroke.pathEffect = if (o.optBoolean("dashed")) dash else null
        canvas.drawPath(p, stroke)
        stroke.pathEffect = null
        stroke.strokeWidth = 1.5f
    }

    private fun line(canvas: Canvas, o: JSONObject, m: Matrix) {
        val a = pt(o.optJSONArray("a"), m)
        val b = pt(o.optJSONArray("b"), m)
        stroke.color = colorOf(o)
        canvas.drawLine(a[0], a[1], b[0], b[1], stroke)
    }

    private fun point(canvas: Canvas, o: JSONObject, m: Matrix, square: Boolean) {
        val p = pt(o.optJSONArray("p"), m)
        val r = 5f
        fill.color = if (o.optBoolean("filled")) colorOf(o) else Color.WHITE
        stroke.color = colorOf(o)
        if (square) {
            canvas.drawRect(p[0] - r, p[1] - r, p[0] + r, p[1] + r, fill)
            canvas.drawRect(p[0] - r, p[1] - r, p[0] + r, p[1] + r, stroke)
        } else {
            canvas.drawCircle(p[0], p[1], r, fill)
            canvas.drawCircle(p[0], p[1], r, stroke)
        }
    }

    private fun label(canvas: Canvas, o: JSONObject, m: Matrix) {
        val p = pt(o.optJSONArray("p"), m)
        text.color = colorOf(o, Color.WHITE)
        canvas.drawText(o.optString("text"), p[0] + 8f, p[1] - 8f, text)
    }

    companion object {
        /** The accent colours (`--gold`, `--teal`). */
        val GOLD = Color.parseColor("#EFC98C")
        val TEAL = Color.parseColor("#83C5BB")
    }
}

// GLOSSARY
// - OverlayPainter: draws tool overlays and canvas boxes.
// - stroke, fill, text, dash: the paints.
// - draw(canvas, answer, m): the overlays of an EngineBridge.overlays answer.
// - box(canvas, r, active, dashed): a box on screen.
// - colorOf(o, fallback), pt(a, m): an overlay's colour and point.
// - path, line, point, label: one overlay of each kind.
// - GOLD, TEAL: the accent colours.
