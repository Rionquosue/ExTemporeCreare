package com.extemporecreare.app.ui.canvas

import android.graphics.Matrix
import android.graphics.RectF

/**
 * The artwork camera: the document point at the view's centre ([cx], [cy]), the [zoom] (screen
 * pixels per document point), and whether the view is mirrored ([flipped], View Flip: the view
 * only, never the artwork, `uistandards.md` §11).
 */
data class Camera(val cx: Double = 0.0, val cy: Double = 0.0, val zoom: Double = 1.0, val flipped: Boolean = false) {

    /** The document point under screen point ([sx], [sy]) of a [w] × [h] view. */
    fun toDoc(sx: Float, sy: Float, w: Int, h: Int): Pair<Double, Double> {
        val x = if (flipped) w - sx else sx
        return (cx + (x - w / 2.0) / zoom) to (cy + (sy - h / 2.0) / zoom)
    }

    /** The document area a [w] × [h] view shows: left, top, width, height in points. */
    fun viewport(w: Int, h: Int): DoubleArray = doubleArrayOf(cx - w / 2.0 / zoom, cy - h / 2.0 / zoom, w / zoom, h / zoom)

    /** The matrix from document points to screen pixels of a [w] × [h] view. */
    fun matrix(w: Int, h: Int): Matrix = Matrix().apply {
        postTranslate((-cx).toFloat(), (-cy).toFloat())
        postScale(zoom.toFloat(), zoom.toFloat())
        postTranslate(w / 2f, h / 2f)
        if (flipped) postScale(-1f, 1f, w / 2f, h / 2f)
    }

    /** The screen rectangle of document area ([x], [y], [width], [height]) in a [w] × [h] view. */
    fun toScreen(x: Double, y: Double, width: Double, height: Double, w: Int, h: Int): RectF =
        RectF(x.toFloat(), y.toFloat(), (x + width).toFloat(), (y + height).toFloat()).also { matrix(w, h).mapRect(it) }

    /** Moved by [dx], [dy] screen pixels (the artwork follows the fingers). */
    fun panned(dx: Float, dy: Float): Camera = copy(cx = cx - (if (flipped) -dx else dx) / zoom, cy = cy - dy / zoom)

    /** Zoomed by [factor] about screen point ([sx], [sy]) of a [w] × [h] view, within [MIN_ZOOM]–[MAX_ZOOM]. */
    fun zoomedAbout(factor: Double, sx: Float, sy: Float, w: Int, h: Int): Camera {
        val z = (zoom * factor).coerceIn(MIN_ZOOM, MAX_ZOOM)
        val (px, py) = toDoc(sx, sy, w, h)
        val x = if (flipped) w - sx else sx
        return copy(zoom = z, cx = px - (x - w / 2.0) / z, cy = py - (sy - h / 2.0) / z)
    }

    companion object {
        /** The furthest out and in the view zooms. */
        const val MIN_ZOOM = 0.01
        const val MAX_ZOOM = 64.0

        /** A camera fitting document area ([x], [y], [width], [height]) in a [w] × [h] view, with a margin. */
        fun fit(x: Double, y: Double, width: Double, height: Double, w: Int, h: Int): Camera {
            val z = (minOf(w / width, h / height) * 0.82).coerceIn(MIN_ZOOM, MAX_ZOOM)
            return Camera(x + width / 2, y + height / 2, z)
        }
    }
}

// GLOSSARY
// - Camera: the artwork camera (centre, zoom, View Flip).
// - toDoc(sx, sy, w, h): the document point under a screen point.
// - viewport(w, h): the document area the view shows.
// - matrix(w, h): document points → screen pixels.
// - toScreen(x, y, width, height, w, h): a document area on screen.
// - panned(dx, dy), zoomedAbout(factor, sx, sy, w, h): the camera moved or zoomed.
// - MIN_ZOOM, MAX_ZOOM: 0.01 and 64.
// - fit(x, y, width, height, w, h): a camera fitting an area.
