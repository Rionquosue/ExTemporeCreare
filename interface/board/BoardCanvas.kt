package com.extemporecreare.app.ui.board

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.calculateCentroid
import androidx.compose.foundation.gestures.calculateRotation
import androidx.compose.foundation.gestures.calculateZoom
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.clipRect
import androidx.compose.ui.graphics.drawscope.withTransform
import androidx.compose.ui.input.pointer.PointerEvent
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.text.TextMeasurer
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.drawText
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.sp
import com.extemporecreare.app.components.boards.BoardCamera
import com.extemporecreare.app.components.boards.BoardCrop
import com.extemporecreare.app.components.boards.BoardPoint
import com.extemporecreare.app.components.boards.ImagePlace
import com.extemporecreare.app.components.boards.ReferenceBoard
import com.extemporecreare.app.ui.theme.EtcColors
import kotlinx.coroutines.withTimeoutOrNull
import kotlin.math.atan2
import kotlin.math.roundToInt

/**
 * The reference board's view (`uistandards.md` §8.2, `filestandard.md` §7.3) through the board's
 * own camera. Two fingers always move the view: pinch zooms, twist rotates, both pan; they never
 * act on an image. One finger: on an image it moves it, elsewhere it pans. A tap selects an image,
 * which shows four corner handles: dragging on a handle resizes the image about its pivot (aspect
 * kept), dragging just outside a corner rotates it. Holding an image opens its menu ([onMenu]).
 * While [cropping] an image, its whole picture shows faintly and its corners move the crop's
 * edges. A missing image shows as a dashed placeholder with its file name. [revision] redraws;
 * [onChanged] reports edits, and each gesture's end checkpoints the board.
 */
@Composable
fun BoardCanvas(
    board: ReferenceBoard,
    revision: Int,
    selected: String?,
    cropping: String?,
    onSelect: (String?) -> Unit,
    onMenu: (String, Offset) -> Unit,
    onChanged: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val text = rememberTextMeasurer()
    val current by rememberUpdatedState(selected)
    val crop by rememberUpdatedState(cropping)
    val select by rememberUpdatedState(onSelect)
    val menu by rememberUpdatedState(onMenu)
    val changed by rememberUpdatedState(onChanged)
    Canvas(
        modifier.pointerInput(board) {
            val handle = HANDLE_DP * density
            val ring = ROTATE_DP * density
            awaitEachGesture {
                val down = awaitFirstDown()
                val w = size.width.toDouble()
                val h = size.height.toDouble()
                val start = BoardPoint(down.position.x.toDouble(), down.position.y.toDouble())
                val cam0 = board.camera()
                val hit = hitTest(board, cam0, w, h, start)
                var mode: Mode = when {
                    crop != null -> cropHandle(board, crop!!, cam0, w, h, start, handle)?.let { Mode.Crop(crop!!, it) } ?: Mode.Pan
                    else -> current?.let { id -> imageHandle(board, id, cam0, w, h, start, handle, ring) } ?: hit?.let { Mode.Move(it) } ?: Mode.Pan
                }
                var last = start
                var travel = 0.0
                var moved = false
                var held = false
                var navigating = false
                var prev: PointerEvent? = null
                while (true) {
                    val event = if (!moved && !navigating && !held && hit != null && crop == null) {
                        withTimeoutOrNull(viewConfiguration.longPressTimeoutMillis) { awaitPointerEvent() }
                    } else {
                        awaitPointerEvent()
                    }
                    if (event == null) {
                        held = true
                        menu(hit!!, down.position)
                        continue
                    }
                    val pressed = event.changes.count { it.pressed }
                    if (pressed == 0) break
                    if (pressed >= 2) {
                        // Two fingers: the view, never an image.
                        navigating = true
                        moved = true
                        val before = prev?.takeIf { it.changes.count { c -> c.pressed } >= 2 }
                        if (before != null) {
                            val c0 = before.calculateCentroid()
                            val c1 = event.calculateCentroid()
                            val cam = board.camera()
                            board.setCamera(cam.navigated(BoardPoint(c0.x.toDouble(), c0.y.toDouble()), BoardPoint(c1.x.toDouble(), c1.y.toDouble()), event.calculateZoom().toDouble(), event.calculateRotation().toDouble(), w, h))
                            changed()
                        }
                    } else if (!navigating && !held) {
                        val pos = event.changes.first { it.pressed }.position
                        val now = BoardPoint(pos.x.toDouble(), pos.y.toDouble())
                        travel += (now - last).length()
                        if (travel > viewConfiguration.touchSlop) moved = true
                        if (moved) {
                            val cam = board.camera()
                            val dBoard = cam.toBoard(now, w, h) - cam.toBoard(last, w, h)
                            when (val m = mode) {
                                is Mode.Move -> board.move(m.id, dBoard.x, dBoard.y)
                                is Mode.Pan -> board.setCamera(cam.copy(x = cam.x - dBoard.x, y = cam.y - dBoard.y))
                                is Mode.Resize -> board.placeOf(m.id)?.let { p ->
                                    val pivot = cam.toView(p.pivot, w, h)
                                    val was = (last - pivot).length()
                                    if (was > 1.0) board.scale(m.id, (now - pivot).length() / was)
                                }
                                is Mode.Rotate -> board.placeOf(m.id)?.let { p ->
                                    val pivot = cam.toView(p.pivot, w, h)
                                    board.rotate(m.id, p.rotation + angle(now - pivot) - angle(last - pivot))
                                }
                                is Mode.Crop -> dragCrop(board, m.id, m.corner, dBoard)
                            }
                            changed()
                        }
                        last = now
                    }
                    prev = event
                    event.changes.forEach { it.consume() }
                }
                if (moved) board.checkpoint() else if (!held && crop == null) select(hit)
            }
        },
    ) {
        @Suppress("UNUSED_EXPRESSION") revision
        val cam = board.camera()
        val w = size.width.toDouble()
        val h = size.height.toDouble()
        clipRect {
            drawRect(EtcColors.Bg)
            for (image in board.images()) {
                if (!image.optBoolean("visible", true)) continue
                val id = image.optString("id")
                val place = ImagePlace.of(image.optJSONObject("transform") ?: continue)
                if (!onScreen(place, cam, w, h)) continue
                val picture = board.pictures[id]?.takeIf { !board.isMissing(id) }
                val c = board.cropOf(id)
                if (picture != null) {
                    if (id == cropping) {
                        val full = fullPlace(place, c)
                        drawPicture(picture.asImageBitmap(), full, BoardCrop(), cam, w, h, 0.35f)
                    }
                    drawPicture(picture.asImageBitmap(), place, c, cam, w, h, 1f)
                } else {
                    val name = image.optJSONObject("source")?.optString("originalFilename").orEmpty()
                    drawPlaceholder(text, place, cam, w, h, if (board.isMissing(id)) "Missing\n$name" else "")
                }
                if (id == selected || id == cropping) {
                    val corners = place.corners().map { cam.toView(it, w, h).offset() }
                    val outline = Path().apply { moveTo(corners[0].x, corners[0].y); corners.drop(1).forEach { lineTo(it.x, it.y) }; close() }
                    drawPath(outline, EtcColors.Gold, style = Stroke(2f * density, pathEffect = if (id == cropping) PathEffect.dashPathEffect(floatArrayOf(10f, 8f)) else null))
                    for (corner in corners) {
                        drawCircle(EtcColors.Bg, (HANDLE_DOT_DP + 2f) * density, corner)
                        drawCircle(EtcColors.Gold, HANDLE_DOT_DP * density, corner)
                    }
                }
            }
        }
    }
}

/** What a one-finger drag does. */
private sealed interface Mode {
    data object Pan : Mode
    data class Move(val id: String) : Mode
    data class Resize(val id: String) : Mode
    data class Rotate(val id: String) : Mode
    data class Crop(val id: String, val corner: Int) : Mode
}

/** The handle of selected image [id] at [p]: a corner (resize) or just outside one (rotate), or null. */
private fun imageHandle(board: ReferenceBoard, id: String, cam: BoardCamera, w: Double, h: Double, p: BoardPoint, handle: Float, ring: Float): Mode? {
    val place = board.placeOf(id) ?: return null
    val corners = place.corners().map { cam.toView(it, w, h) }
    if (corners.any { (it - p).length() <= handle }) return Mode.Resize(id)
    val inside = place.contains(cam.toBoard(p, w, h))
    if (!inside && corners.any { (it - p).length() <= ring }) return Mode.Rotate(id)
    return null
}

/** The corner of the cropped image [id] at [p] (0 top-left, 1 top-right, 2 bottom-right, 3 bottom-left), or null. */
private fun cropHandle(board: ReferenceBoard, id: String, cam: BoardCamera, w: Double, h: Double, p: BoardPoint, handle: Float): Int? {
    val place = board.placeOf(id) ?: return null
    return place.corners().map { cam.toView(it, w, h) }.indexOfFirst { (it - p).length() <= handle * 1.6 }.takeIf { it >= 0 }
}

/** Move crop corner [corner] of image [id] by [d] board units: the crop's edges follow the finger. */
private fun dragCrop(board: ReferenceBoard, id: String, corner: Int, d: BoardPoint) {
    val place = board.placeOf(id) ?: return
    val c = board.cropOf(id)
    val local = d.rotated(-place.rotation)
    val du = local.x / (place.w / c.w)
    val dv = local.y / (place.h / c.h)
    // In display orientation (a flip mirrors the source).
    var left = if (place.flipX) 1 - c.x - c.w else c.x
    var top = if (place.flipY) 1 - c.y - c.h else c.y
    var right = left + c.w
    var bottom = top + c.h
    val min = BoardCrop.MIN_FRACTION
    if (corner == 0 || corner == 3) left = (left + du).coerceIn(0.0, right - min) else right = (right + du).coerceIn(left + min, 1.0)
    if (corner == 0 || corner == 1) top = (top + dv).coerceIn(0.0, bottom - min) else bottom = (bottom + dv).coerceIn(top + min, 1.0)
    val x = if (place.flipX) 1 - right else left
    val y = if (place.flipY) 1 - bottom else top
    board.setCrop(id, BoardCrop(x, y, right - left, bottom - top))
}

/** The placement of the whole source of an image placed at [place] with crop [c]. */
private fun fullPlace(place: ImagePlace, c: BoardCrop): ImagePlace {
    val fullW = place.w / c.w
    val fullH = place.h / c.h
    val sx = if (place.flipX) 1 - c.x - c.w else c.x
    val sy = if (place.flipY) 1 - c.y - c.h else c.y
    return place.reframed(-sx * fullW, -sy * fullH, fullW, fullH)
}

/** The topmost image under view point [p], or null. */
private fun hitTest(board: ReferenceBoard, cam: BoardCamera, w: Double, h: Double, p: BoardPoint): String? {
    val at = cam.toBoard(p, w, h)
    return board.images().asReversed().firstOrNull { i ->
        i.optBoolean("visible", true) && i.optJSONObject("transform")?.let { ImagePlace.of(it).contains(at) } == true
    }?.optString("id")
}

/** Whether image [place] may be in view (a generous circle test). */
private fun onScreen(place: ImagePlace, cam: BoardCamera, w: Double, h: Double): Boolean {
    val centre = cam.toView(place.toBoard(place.w / 2, place.h / 2), w, h)
    val reach = (place.w + place.h) * cam.zoom
    return centre.x > -reach && centre.y > -reach && centre.x < w + reach && centre.y < h + reach
}

/** Draw [picture]'s crop [c] at [place], through [cam], at [alpha]. */
private fun DrawScope.drawPicture(picture: androidx.compose.ui.graphics.ImageBitmap, place: ImagePlace, c: BoardCrop, cam: BoardCamera, w: Double, h: Double, alpha: Float) {
    val pivot = cam.toView(place.pivot, w, h).offset()
    val pw = (place.w * cam.zoom).toFloat()
    val ph = (place.h * cam.zoom).toFloat()
    val sx = (c.x * picture.width).roundToInt().coerceIn(0, picture.width - 1)
    val sy = (c.y * picture.height).roundToInt().coerceIn(0, picture.height - 1)
    val sw = (c.w * picture.width).roundToInt().coerceIn(1, picture.width - sx)
    val sh = (c.h * picture.height).roundToInt().coerceIn(1, picture.height - sy)
    withTransform({
        translate(pivot.x, pivot.y)
        rotate((cam.rotation + place.rotation).toFloat(), Offset.Zero)
        scale(if (place.flipX) -1f else 1f, if (place.flipY) -1f else 1f, Offset(((0.5 - place.px) * pw).toFloat(), ((0.5 - place.py) * ph).toFloat()))
    }) {
        drawImage(
            picture, IntOffset(sx, sy), IntSize(sw, sh),
            IntOffset((-place.px * pw).roundToInt(), (-place.py * ph).roundToInt()), IntSize(pw.roundToInt().coerceAtLeast(1), ph.roundToInt().coerceAtLeast(1)),
            alpha = alpha,
        )
    }
}

/** A dashed placeholder at [place] with [label]. */
private fun DrawScope.drawPlaceholder(text: TextMeasurer, place: ImagePlace, cam: BoardCamera, w: Double, h: Double, label: String) {
    val pivot = cam.toView(place.pivot, w, h).offset()
    val pw = (place.w * cam.zoom).toFloat()
    val ph = (place.h * cam.zoom).toFloat()
    withTransform({
        translate(pivot.x, pivot.y)
        rotate((cam.rotation + place.rotation).toFloat(), Offset.Zero)
    }) {
        val topLeft = Offset((-place.px * pw).toFloat(), (-place.py * ph).toFloat())
        drawRect(EtcColors.Panel2, topLeft, Size(pw, ph))
        drawRect(EtcColors.Muted, topLeft, Size(pw, ph), style = Stroke(2f, pathEffect = PathEffect.dashPathEffect(floatArrayOf(10f, 8f))))
        if (label.isNotEmpty() && pw > 24f && ph > 24f) {
            drawText(text, label, topLeft + Offset(8f, 8f), TextStyle(color = EtcColors.Text, fontSize = 11.sp), overflow = TextOverflow.Ellipsis, size = Size(pw - 16f, ph - 16f))
        }
    }
}

private fun BoardPoint.offset(): Offset = Offset(x.toFloat(), y.toFloat())

/** [d]'s direction in degrees, clockwise on screen. */
private fun angle(d: BoardPoint): Double = Math.toDegrees(atan2(d.y, d.x))

/** Corner handle reach (resize), the rotate reach beyond a corner, and the handle dot, in dp. */
private const val HANDLE_DP = 18f
private const val ROTATE_DP = 48f
private const val HANDLE_DOT_DP = 6f

// GLOSSARY
// - BoardCanvas(board, revision, selected, cropping, onSelect, onMenu, onChanged, modifier): the reference board's view and its gestures.
// - Mode: what a one-finger drag does (Pan, Move, Resize, Rotate, Crop).
// - imageHandle(...): the selected image's handle under a point. cropHandle(...): the crop corner under a point.
// - dragCrop(board, id, corner, d): move a crop corner. fullPlace(place, c): where an image's whole source sits.
// - hitTest(...): the topmost image under a point. onScreen(...): whether an image may be in view.
// - drawPicture(...), drawPlaceholder(...): draw an image or its placeholder through the camera.
// - offset(), angle(d): helpers. HANDLE_DP, ROTATE_DP, HANDLE_DOT_DP: handle sizes in dp.
