package com.extemporecreare.app.tools.crop

import kotlin.math.abs
import kotlin.math.max
import kotlin.math.min

/**
 * The Crop tool's box over a finite canvas's artboard [index], in document points. Its corners
 * and edges resize it, its inside moves it; it is never smaller than [MIN] points.
 */
class CanvasCropBox(val index: Int, x: Double, y: Double, width: Double, height: Double) {

    /** The box's left edge. */
    var left = x
        private set

    /** The box's top edge. */
    var top = y
        private set

    /** The box's right edge. */
    var right = x + width
        private set

    /** The box's bottom edge. */
    var bottom = y + height
        private set

    /** What a drag holds: the edges it moves (inside: all four). */
    data class Grip(val left: Boolean, val top: Boolean, val right: Boolean, val bottom: Boolean)

    /** The grip at ([x], [y]) within [reach] points of an edge or corner, else inside, else `null`. */
    fun gripAt(x: Double, y: Double, reach: Double): Grip? {
        val nearX = y in top - reach..bottom + reach
        val nearY = x in left - reach..right + reach
        val l = nearX && abs(x - left) <= reach
        val r = nearX && abs(x - right) <= reach && !l
        val t = nearY && abs(y - top) <= reach
        val b = nearY && abs(y - bottom) <= reach && !t
        if (l || r || t || b) return Grip(l, t, r, b)
        if (x in left..right && y in top..bottom) return Grip(left = true, top = true, right = true, bottom = true)
        return null
    }

    /** Move the edges [grip] holds by ([dx], [dy]), keeping the box at least [MIN] points wide and high. */
    fun drag(grip: Grip, dx: Double, dy: Double) {
        val all = grip.left && grip.right && grip.top && grip.bottom
        if (all) {
            left += dx; right += dx; top += dy; bottom += dy
            return
        }
        if (grip.left) left = min(left + dx, right - MIN)
        if (grip.right) right = max(right + dx, left + MIN)
        if (grip.top) top = min(top + dy, bottom - MIN)
        if (grip.bottom) bottom = max(bottom + dy, top + MIN)
    }

    /** Whether ([x], [y]) is inside the box. */
    fun contains(x: Double, y: Double): Boolean = x in left..right && y in top..bottom

    companion object {
        /** The smallest the box gets, in points. */
        const val MIN = 1.0
    }
}

// GLOSSARY
// - CanvasCropBox: the crop box over artboard `index` of a finite canvas, in points.
// - left, top, right, bottom: the box's edges.
// - Grip: the edges a drag moves (all four: the inside).
// - gripAt(x, y, reach): the edge, corner or inside under a point.
// - drag(grip, dx, dy): move the held edges, keeping the minimum size.
// - contains(x, y): whether a point is inside the box.
// - MIN: the smallest box size, 1 pt.
