package com.extemporecreare.app.components.boards

import org.json.JSONArray
import org.json.JSONObject
import kotlin.math.cos
import kotlin.math.sin

/** A 2D point (board units or view pixels). */
data class BoardPoint(val x: Double, val y: Double) {
    operator fun plus(o: BoardPoint) = BoardPoint(x + o.x, y + o.y)
    operator fun minus(o: BoardPoint) = BoardPoint(x - o.x, y - o.y)
    operator fun times(k: Double) = BoardPoint(x * k, y * k)
    fun length(): Double = kotlin.math.hypot(x, y)

    /** This point turned by [degrees] (clockwise on screen, where y grows downwards). */
    fun rotated(degrees: Double): BoardPoint {
        val a = Math.toRadians(degrees)
        return BoardPoint(x * cos(a) - y * sin(a), x * sin(a) + y * cos(a))
    }
}

/**
 * The board's camera (`filestandard.md` §7.3): the board point at the view's centre, the zoom
 * (view pixels per board unit) and the view's rotation in degrees, clockwise.
 */
data class BoardCamera(val x: Double = 0.0, val y: Double = 0.0, val zoom: Double = 1.0, val rotation: Double = 0.0) {

    /** Board point [p] in a view [w]×[h] pixels. */
    fun toView(p: BoardPoint, w: Double, h: Double): BoardPoint = ((p - BoardPoint(x, y)) * zoom).rotated(rotation) + BoardPoint(w / 2, h / 2)

    /** View point [v] (in a view [w]×[h] pixels) on the board. */
    fun toBoard(v: BoardPoint, w: Double, h: Double): BoardPoint = (v - BoardPoint(w / 2, h / 2)).rotated(-rotation) * (1 / zoom) + BoardPoint(x, y)

    /**
     * The camera after two fingers moved: [before] and [after] are their centroids (view pixels),
     * [scale] the change of their distance and [turn] of their angle (degrees). The board point
     * under the first centroid ends under the second.
     */
    fun navigated(before: BoardPoint, after: BoardPoint, scale: Double, turn: Double, w: Double, h: Double): BoardCamera {
        val anchor = toBoard(before, w, h)
        val next = copy(zoom = (zoom * scale).coerceIn(MIN_ZOOM, MAX_ZOOM), rotation = normalized(rotation + turn))
        val now = next.toBoard(after, w, h)
        return next.copy(x = next.x + anchor.x - now.x, y = next.y + anchor.y - now.y)
    }

    companion object {
        const val MIN_ZOOM = 0.01
        const val MAX_ZOOM = 64.0

        /** [degrees] in (−180, 180]. */
        fun normalized(degrees: Double): Double {
            var d = degrees % 360.0
            if (d > 180) d -= 360
            if (d <= -180) d += 360
            return d
        }
    }
}

/**
 * One image's placement read from its `transform` (§7.3): the unrotated rectangle ([x], [y], [w],
 * [h]) in board units, its rotation about the pivot ([px], [py] as fractions) and its flips.
 */
data class ImagePlace(val x: Double, val y: Double, val w: Double, val h: Double, val rotation: Double, val px: Double, val py: Double, val flipX: Boolean, val flipY: Boolean) {

    /** The pivot on the board (rotation keeps it in place). */
    val pivot: BoardPoint get() = BoardPoint(x + px * w, y + py * h)

    /** Local point ([u], [v]) of the unrotated rectangle (0..w, 0..h) on the board. */
    fun toBoard(u: Double, v: Double): BoardPoint = pivot + (BoardPoint(u, v) - BoardPoint(px * w, py * h)).rotated(rotation)

    /** Board point [p] in the rectangle's local frame (0..w, 0..h inside). */
    fun toLocal(p: BoardPoint): BoardPoint = (p - pivot).rotated(-rotation) + BoardPoint(px * w, py * h)

    /** The corners on the board: top-left, top-right, bottom-right, bottom-left (unrotated sense). */
    fun corners(): List<BoardPoint> = listOf(toBoard(0.0, 0.0), toBoard(w, 0.0), toBoard(w, h), toBoard(0.0, h))

    /** Whether board point [p] is on the image. */
    fun contains(p: BoardPoint): Boolean = toLocal(p).let { it.x in 0.0..w && it.y in 0.0..h }

    /** This placement as a `transform` object. */
    fun toJson(): JSONObject {
        val t = JSONObject().put("x", x).put("y", y).put("width", w).put("height", h).put("rotationDegrees", rotation).put("pivot", JSONArray().put(px).put(py))
        if (flipX) t.put("flipX", true)
        if (flipY) t.put("flipY", true)
        return t
    }

    /** Scaled by [factor] about the pivot (which stays in place). */
    fun scaledAboutPivot(factor: Double): ImagePlace {
        val p = pivot
        val nw = (w * factor).coerceAtLeast(MIN_SIDE)
        val nh = (h * factor).coerceAtLeast(MIN_SIDE)
        return copy(x = p.x - px * nw, y = p.y - py * nh, w = nw, h = nh)
    }

    /**
     * The placement showing local rectangle ([u], [v], [nw], [nh]) of this one (its local frame)
     * as the whole image, at the same place, scale and rotation (a crop change, §7.3).
     */
    fun reframed(u: Double, v: Double, nw: Double, nh: Double): ImagePlace {
        val newPivot = toBoard(u + px * nw, v + py * nh)
        return copy(x = newPivot.x - px * nw, y = newPivot.y - py * nh, w = nw, h = nh)
    }

    companion object {
        /** The smallest side an image is resized to, in board units. */
        const val MIN_SIDE = 1.0

        /** The placement in a `transform` object. */
        fun of(t: JSONObject): ImagePlace {
            val p = t.optJSONArray("pivot")
            return ImagePlace(
                t.optDouble("x", 0.0), t.optDouble("y", 0.0), t.optDouble("width", 1.0), t.optDouble("height", 1.0),
                t.optDouble("rotationDegrees", 0.0), p?.optDouble(0, 0.5) ?: 0.5, p?.optDouble(1, 0.5) ?: 0.5,
                t.optBoolean("flipX", false), t.optBoolean("flipY", false),
            )
        }
    }
}

/** A crop in source-image fractions (§7.3). */
data class BoardCrop(val x: Double = 0.0, val y: Double = 0.0, val w: Double = 1.0, val h: Double = 1.0) {
    fun toJson(): JSONObject = JSONObject().put("x", x).put("y", y).put("width", w).put("height", h)
    val isWhole: Boolean get() = x <= 0.0 && y <= 0.0 && w >= 1.0 && h >= 1.0

    companion object {
        /** The smallest crop side, as a fraction of the source. */
        const val MIN_FRACTION = 0.02

        fun of(c: JSONObject?): BoardCrop = if (c == null) BoardCrop() else BoardCrop(c.optDouble("x", 0.0), c.optDouble("y", 0.0), c.optDouble("width", 1.0), c.optDouble("height", 1.0))
    }
}

// GLOSSARY
// - BoardPoint: a 2D point; plus, minus, times, length, rotated(degrees).
// - BoardCamera: the board's camera (x, y, zoom, rotation); toView, toBoard, navigated; MIN_ZOOM, MAX_ZOOM; normalized(degrees).
// - ImagePlace: an image's placement (rectangle, rotation, pivot, flips); pivot, toBoard, toLocal, corners, contains, toJson, scaledAboutPivot, reframed; MIN_SIDE; of(transform).
// - BoardCrop: a crop in source fractions; toJson, isWhole; MIN_FRACTION; of(crop).
