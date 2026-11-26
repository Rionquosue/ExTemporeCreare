package com.extemporecreare.app.ui.board

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.calculateCentroid
import androidx.compose.foundation.gestures.calculatePan
import androidx.compose.foundation.gestures.calculateZoom
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.clipRect
import androidx.compose.ui.graphics.drawscope.rotate
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.drawText
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.sp
import com.extemporecreare.app.components.boards.ReferenceBoard
import com.extemporecreare.app.ui.theme.EtcColors
import org.json.JSONObject
import kotlin.math.cos
import kotlin.math.roundToInt
import kotlin.math.sin

/**
 * The reference board's view (`uistandards.md` §8.2, `filestandard.md` §7.3): the board's images at
 * their places, through the board's own camera. One finger on an image moves it, elsewhere pans; two
 * fingers zoom and pan the camera, or resize the selected image when they start on it (aspect kept).
 * A tap selects an image or clears the selection. A missing image shows as a dashed placeholder with
 * its file name. Only what is in view is drawn. [revision] redraws it; [onChanged] reports edits,
 * and each gesture's end checkpoints the board (`filestandard.md` §7.3).
 */
@Composable
fun BoardCanvas(board: ReferenceBoard, revision: Int, selected: String?, onSelect: (String?) -> Unit, onChanged: () -> Unit, modifier: Modifier = Modifier) {
    val text = rememberTextMeasurer()
    val current by rememberUpdatedState(selected)
    val select by rememberUpdatedState(onSelect)
    val changed by rememberUpdatedState(onChanged)
    Canvas(
        modifier.pointerInput(board) {
            awaitEachGesture {
                val down = awaitFirstDown()
                val view = Size(size.width.toFloat(), size.height.toFloat())
                val hit = hitTest(board, view, down.position)
                var travel = 0f
                var moved = false
                var resizing: String? = null
                do {
                    val event = awaitPointerEvent()
                    val pressed = event.changes.count { it.pressed }
                    val pan = event.calculatePan()
                    val (cx, cy, zoom) = board.camera()
                    if (pressed >= 2) {
                        moved = true
                        if (resizing == null && hit != null && hit == current) resizing = hit
                        val factor = event.calculateZoom()
                        if (resizing != null) {
                            board.scale(resizing, factor.toDouble())
                        } else {
                            val c = event.calculateCentroid()
                            val nz = (zoom * factor).coerceIn(0.01, 64.0)
                            val px = (c.x - view.width / 2) / zoom + cx
                            val py = (c.y - view.height / 2) / zoom + cy
                            board.setCamera(px - (c.x - view.width / 2) / nz - pan.x / nz, py - (c.y - view.height / 2) / nz - pan.y / nz, nz)
                        }
                        changed()
                    } else if (pressed == 1 && resizing == null) {
                        travel += pan.getDistance()
                        if (travel > viewConfiguration.touchSlop) moved = true
                        if (moved) {
                            if (hit != null) board.move(hit, pan.x / zoom, pan.y / zoom) else board.setCamera(cx - pan.x / zoom, cy - pan.y / zoom, zoom)
                            changed()
                        }
                    }
                    event.changes.forEach { it.consume() }
                } while (event.changes.any { it.pressed })
                if (moved) board.checkpoint() else select(hit)
            }
        },
    ) {
        @Suppress("UNUSED_EXPRESSION") revision
        val (cx, cy, zoom) = board.camera()
        clipRect {
            drawRect(EtcColors.Bg)
            for (image in board.images()) {
                if (!image.optBoolean("visible", true)) continue
                val id = image.optString("id")
                val t = image.optJSONObject("transform") ?: continue
                val left = ((t.optDouble("x") - cx) * zoom + size.width / 2).toFloat()
                val top = ((t.optDouble("y") - cy) * zoom + size.height / 2).toFloat()
                val w = (t.optDouble("width") * zoom).toFloat()
                val h = (t.optDouble("height") * zoom).toFloat()
                val reach = maxOf(w, h)
                if (left + reach < -reach || top + reach < -reach || left - reach > size.width + reach || top - reach > size.height + reach) continue
                val pivot = pivotOf(t, left, top, w, h)
                rotate(t.optDouble("rotationDegrees", 0.0).toFloat(), pivot) {
                    val picture = board.pictures[id]
                    if (picture != null && !board.isMissing(id)) {
                        val (src, srcSize) = cropOf(image.optJSONObject("crop"), picture.width, picture.height)
                        drawImage(picture.asImageBitmap(), src, srcSize, IntOffset(left.roundToInt(), top.roundToInt()), IntSize(w.roundToInt().coerceAtLeast(1), h.roundToInt().coerceAtLeast(1)))
                    } else {
                        drawRect(EtcColors.Panel2, Offset(left, top), Size(w, h))
                        drawRect(EtcColors.Muted, Offset(left, top), Size(w, h), style = Stroke(2f, pathEffect = PathEffect.dashPathEffect(floatArrayOf(10f, 8f))))
                        if (board.isMissing(id)) {
                            val name = image.optJSONObject("source")?.optString("originalFilename").orEmpty()
                            drawText(text, "Missing\n$name", Offset(left + 8f, top + 8f), TextStyle(color = EtcColors.Text, fontSize = 11.sp), overflow = TextOverflow.Ellipsis, size = Size((w - 16f).coerceAtLeast(1f), (h - 16f).coerceAtLeast(1f)))
                        }
                    }
                    if (id == selected) drawRect(EtcColors.Gold, Offset(left, top), Size(w, h), style = Stroke(3f))
                }
            }
        }
    }
}

/** The topmost image under [point] (view pixels), or null. */
private fun hitTest(board: ReferenceBoard, view: Size, point: Offset): String? {
    val (cx, cy, zoom) = board.camera()
    for (image in board.images().asReversed()) {
        if (!image.optBoolean("visible", true)) continue
        val t = image.optJSONObject("transform") ?: continue
        val left = ((t.optDouble("x") - cx) * zoom + view.width / 2).toFloat()
        val top = ((t.optDouble("y") - cy) * zoom + view.height / 2).toFloat()
        val w = (t.optDouble("width") * zoom).toFloat()
        val h = (t.optDouble("height") * zoom).toFloat()
        val pivot = pivotOf(t, left, top, w, h)
        val a = Math.toRadians(-t.optDouble("rotationDegrees", 0.0))
        val dx = point.x - pivot.x
        val dy = point.y - pivot.y
        val x = (dx * cos(a) - dy * sin(a)).toFloat() + pivot.x
        val y = (dx * sin(a) + dy * cos(a)).toFloat() + pivot.y
        if (x in left..left + w && y in top..top + h) return image.optString("id")
    }
    return null
}

/** The rotation pivot of an image drawn at ([left], [top]) sized [w]×[h]: its `pivot` ([x, y] fractions) or centre. */
private fun pivotOf(t: JSONObject, left: Float, top: Float, w: Float, h: Float): Offset {
    val p = t.optJSONArray("pivot")
    return Offset(left + w * (p?.optDouble(0, 0.5) ?: 0.5).toFloat(), top + h * (p?.optDouble(1, 0.5) ?: 0.5).toFloat())
}

/** The part of a [width]×[height] picture a normalized [crop] keeps (all of it without one). */
private fun cropOf(crop: JSONObject?, width: Int, height: Int): Pair<IntOffset, IntSize> {
    if (crop == null) return IntOffset.Zero to IntSize(width, height)
    val x = (crop.optDouble("x", 0.0) * width).roundToInt().coerceIn(0, width - 1)
    val y = (crop.optDouble("y", 0.0) * height).roundToInt().coerceIn(0, height - 1)
    val w = (crop.optDouble("width", 1.0) * width).roundToInt().coerceIn(1, width - x)
    val h = (crop.optDouble("height", 1.0) * height).roundToInt().coerceIn(1, height - y)
    return IntOffset(x, y) to IntSize(w, h)
}

// GLOSSARY
// - BoardCanvas(board, revision, selected, onSelect, onChanged, modifier): the reference board's view and its gestures.
// - hitTest(board, view, point): the topmost image under a point.
// - pivotOf(t, left, top, w, h): an image's rotation pivot on screen.
// - cropOf(crop, width, height): the part of a picture a crop keeps.
