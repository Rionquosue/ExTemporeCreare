package com.extemporecreare.app.components.tools

import kotlin.math.abs
import kotlin.math.max
import kotlin.math.min
import kotlin.math.roundToLong

/**
 * A rectangle on the canvas, in document points, that a drag edits: its corners and edges resize
 * it, its inside moves it, and it is never smaller than [MIN] points. The Crop tool's canvas box and
 * the Export tool's slices use it.
 */
class DragBox(left: Double, top: Double, right: Double, bottom: Double) {

    /** The left edge. */
    var left = left
        private set

    /** The top edge. */
    var top = top
        private set

    /** The right edge. */
    var right = right
        private set

    /** The bottom edge. */
    var bottom = bottom
        private set

    /** The width. */
    val width: Double get() = right - left

    /** The height. */
    val height: Double get() = bottom - top

    /** What a drag holds: the edges it moves (all four: the inside, which moves the box). */
    data class Grip(val left: Boolean, val top: Boolean, val right: Boolean, val bottom: Boolean) {
        /** Whether the grip moves the whole box. */
        val isInside: Boolean get() = left && top && right && bottom

        companion object {
            /** The inside. */
            val INSIDE = Grip(left = true, top = true, right = true, bottom = true)

            /** The bottom-right corner: what a drag that draws a new box holds. */
            val BOTTOM_RIGHT = Grip(left = false, top = false, right = true, bottom = true)
        }
    }

    /** The grip at ([x], [y]) within [reach] points of an edge or corner, else the inside, else `null`. */
    fun gripAt(x: Double, y: Double, reach: Double): Grip? {
        val alongY = y in top - reach..bottom + reach
        val alongX = x in left - reach..right + reach
        val l = alongY && abs(x - left) <= reach
        val r = alongY && abs(x - right) <= reach && !l
        val t = alongX && abs(y - top) <= reach
        val b = alongX && abs(y - bottom) <= reach && !t
        if (l || r || t || b) return Grip(l, t, r, b)
        return if (contains(x, y)) Grip.INSIDE else null
    }

    /** Move the edges [grip] holds by ([dx], [dy]), keeping the box at least [MIN] points wide and high. */
    fun drag(grip: Grip, dx: Double, dy: Double) {
        if (grip.isInside) {
            left += dx; right += dx; top += dy; bottom += dy
            return
        }
        if (grip.left) left = min(left + dx, right - MIN)
        if (grip.right) right = max(right + dx, left + MIN)
        if (grip.top) top = min(top + dy, bottom - MIN)
        if (grip.bottom) bottom = max(bottom + dy, top + MIN)
    }

    /**
     * Put the edges [grip] holds on the nearest multiple of [spacing] (document coordinates, so the
     * snap is the same at any pan or zoom); moving the inside snaps the top-left corner and keeps the size.
     */
    fun snap(grip: Grip, spacing: Double) {
        if (spacing <= 0) return
        fun near(v: Double) = (v / spacing).roundToLong() * spacing
        if (grip.isInside) {
            val dx = near(left) - left
            val dy = near(top) - top
            drag(grip, dx, dy)
            return
        }
        if (grip.left) left = min(near(left), right - MIN)
        if (grip.right) right = max(near(right), left + MIN)
        if (grip.top) top = min(near(top), bottom - MIN)
        if (grip.bottom) bottom = max(near(bottom), top + MIN)
    }

    /** Give the box [width] × [height] points (each at least [MIN]) about its centre. */
    fun resizeAboutCentre(width: Double, height: Double) {
        val cx = (left + right) / 2
        val cy = (top + bottom) / 2
        val w = max(width, MIN)
        val h = max(height, MIN)
        left = cx - w / 2; right = cx + w / 2; top = cy - h / 2; bottom = cy + h / 2
    }

    /** Whether ([x], [y]) is inside the box. */
    fun contains(x: Double, y: Double): Boolean = x in left..right && y in top..bottom

    /** A copy of the box. */
    fun copy(): DragBox = DragBox(left, top, right, bottom)

    /** Whether [other] has the same edges. */
    fun sameAs(other: DragBox): Boolean = left == other.left && top == other.top && right == other.right && bottom == other.bottom

    companion object {
        /** The smallest the box gets, in points. */
        const val MIN = 1.0

        /** A box from ([x], [y]) [width] × [height] points. */
        fun of(x: Double, y: Double, width: Double, height: Double) = DragBox(x, y, x + width, y + height)

        /** The reach in points of a grip [px] screen pixels wide at the view state [viewJson]'s zoom. */
        fun reach(px: Double, viewJson: String): Double {
            val zoom = runCatching { org.json.JSONObject(viewJson).optDouble("zoom", 1.0) }.getOrDefault(1.0)
            return px / (if (zoom.isFinite() && zoom > 0) zoom else 1.0)
        }
    }
}

// GLOSSARY
// - DragBox: a rectangle a drag edits (edges, corners, inside), in document points.
// - left, top, right, bottom, width, height: the box's edges and size.
// - Grip: the edges a drag moves; isInside; INSIDE; BOTTOM_RIGHT (drawing a new box).
// - gripAt(x, y, reach): the edge, corner or inside under a point.
// - drag(grip, dx, dy): move the held edges, keeping the minimum size.
// - snap(grip, spacing): put the held edges on the grid.
// - resizeAboutCentre(width, height): resize keeping the centre.
// - contains(x, y): whether a point is inside.
// - copy(), sameAs(other): copy and compare.
// - MIN: the smallest size, 1 pt.
// - of(x, y, width, height): a box from position and size.
// - reach(px, viewJson): screen pixels as points at the view's zoom.
