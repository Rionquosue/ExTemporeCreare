package com.extemporecreare.app.ui.canvas

import android.graphics.Matrix
import android.graphics.RectF
import kotlin.math.abs
import kotlin.math.cos
import kotlin.math.sin

/**
 * The artwork camera: the document point at the view's centre ([cx], [cy]), the [zoom] (screen
 * pixels per document point), the view's [rotation] (degrees, clockwise, two-finger twist) and
 * whether the view is mirrored ([flipped], View Flip). Rotation and flip turn the view only, never
 * the artwork (`uistandards.md` §11); both are saved with the document (§12).
 */
data class Camera(val cx: Double = 0.0, val cy: Double = 0.0, val zoom: Double = 1.0, val flipped: Boolean = false, val rotation: Double = 0.0) {

    /** The document point under screen point ([sx], [sy]) of a [w] × [h] view. */
    fun toDoc(sx: Float, sy: Float, w: Int, h: Int): Pair<Double, Double> {
        val x = (if (flipped) w - sx else sx) - w / 2.0
        val y = sy - h / 2.0
        val (rx, ry) = turn(x, y, -rotation)
        return (cx + rx / zoom) to (cy + ry / zoom)
    }

    /** A screen direction ([dx], [dy]) as a document direction (unscaled). */
    fun toDocDirection(dx: Double, dy: Double): Pair<Double, Double> = turn(if (flipped) -dx else dx, dy, -rotation)

    /** The document area a [w] × [h] view shows (the bounding box when rotated): left, top, width, height in points. */
    fun viewport(w: Int, h: Int): DoubleArray {
        val a = Math.toRadians(rotation)
        val bw = (abs(w * cos(a)) + abs(h * sin(a))) / zoom
        val bh = (abs(w * sin(a)) + abs(h * cos(a))) / zoom
        return doubleArrayOf(cx - bw / 2, cy - bh / 2, bw, bh)
    }

    /** The matrix from document points to screen pixels of a [w] × [h] view. */
    fun matrix(w: Int, h: Int): Matrix = Matrix().apply {
        postTranslate((-cx).toFloat(), (-cy).toFloat())
        postScale(zoom.toFloat(), zoom.toFloat())
        postRotate(rotation.toFloat())
        postTranslate(w / 2f, h / 2f)
        if (flipped) postScale(-1f, 1f, w / 2f, h / 2f)
    }

    /** This camera without its rotation: boxes drawn with it are then turned by [screenTurn] about the view's centre. */
    fun upright(): Camera = copy(rotation = 0.0)

    /** The turn, in degrees about the view's centre, from the [upright] camera's screen to this one's. */
    val screenTurn: Float get() = (if (flipped) -rotation else rotation).toFloat()

    /** The screen rectangle of document area ([x], [y], [width], [height]) in a [w] × [h] view, for the [upright] camera. */
    fun toScreen(x: Double, y: Double, width: Double, height: Double, w: Int, h: Int): RectF =
        RectF(x.toFloat(), y.toFloat(), (x + width).toFloat(), (y + height).toFloat()).also { matrix(w, h).mapRect(it) }

    /** Moved by [dx], [dy] screen pixels (the artwork follows the fingers). */
    fun panned(dx: Float, dy: Float): Camera {
        val (rx, ry) = toDocDirection(dx.toDouble(), dy.toDouble())
        return copy(cx = cx - rx / zoom, cy = cy - ry / zoom)
    }

    /** Zoomed by [factor] about screen point ([sx], [sy]) of a [w] × [h] view, within [MIN_ZOOM]–[MAX_ZOOM]. */
    fun zoomedAbout(factor: Double, sx: Float, sy: Float, w: Int, h: Int): Camera = navigated(sx, sy, sx, sy, factor, 0.0, w, h)

    /**
     * The camera after two fingers moved: their centre from ([x0], [y0]) to ([x1], [y1]) on screen,
     * their distance by [factor] and their angle by [turnDegrees] (clockwise on screen). The
     * document point under the first centre ends under the second.
     */
    fun navigated(x0: Float, y0: Float, x1: Float, y1: Float, factor: Double, turnDegrees: Double, w: Int, h: Int): Camera {
        val (px, py) = toDoc(x0, y0, w, h)
        val turned = if (flipped) -turnDegrees else turnDegrees
        val next = copy(zoom = (zoom * factor).coerceIn(MIN_ZOOM, MAX_ZOOM), rotation = normalized(rotation + turned))
        val (nx, ny) = next.toDoc(x1, y1, w, h)
        return next.copy(cx = next.cx + px - nx, cy = next.cy + py - ny)
    }

    companion object {
        /** The furthest out and in the view zooms. */
        const val MIN_ZOOM = 0.01
        const val MAX_ZOOM = 64.0

        /** A camera fitting document area ([x], [y], [width], [height]) in a [w] × [h] view at [rotation], with a margin. */
        fun fit(x: Double, y: Double, width: Double, height: Double, w: Int, h: Int, rotation: Double = 0.0): Camera {
            val a = Math.toRadians(rotation)
            val bw = abs(width * cos(a)) + abs(height * sin(a))
            val bh = abs(width * sin(a)) + abs(height * cos(a))
            val z = (minOf(w / bw, h / bh) * 0.82).coerceIn(MIN_ZOOM, MAX_ZOOM)
            return Camera(x + width / 2, y + height / 2, z, rotation = rotation)
        }

        /** ([x], [y]) turned by [degrees] (clockwise on screen). */
        fun turn(x: Double, y: Double, degrees: Double): Pair<Double, Double> {
            val a = Math.toRadians(degrees)
            return (x * cos(a) - y * sin(a)) to (x * sin(a) + y * cos(a))
        }

        /** [degrees] in (−180, 180]. */
        fun normalized(degrees: Double): Double {
            var d = degrees % 360.0
            if (d > 180) d -= 360
            if (d <= -180) d += 360
            return d
        }
    }
}

// GLOSSARY
// - Camera: the artwork camera (centre, zoom, View Flip, rotation).
// - toDoc(sx, sy, w, h): the document point under a screen point. toDocDirection(dx, dy): a screen direction in the document.
// - viewport(w, h): the document area the view shows (bounding box when rotated).
// - matrix(w, h): document points → screen pixels.
// - upright(), screenTurn: the camera without rotation, and the turn that puts boxes drawn with it in place.
// - toScreen(x, y, width, height, w, h): a document area on screen (for the upright camera).
// - panned(dx, dy), zoomedAbout(factor, sx, sy, w, h), navigated(x0, y0, x1, y1, factor, turnDegrees, w, h): the camera moved, zoomed, turned.
// - MIN_ZOOM, MAX_ZOOM: 0.01 and 64.
// - fit(x, y, width, height, w, h, rotation): a camera fitting an area. turn(x, y, degrees), normalized(degrees): helpers.
