package com.extemporecreare.app.components.tools

/**
 * A stroke being drawn, shown on the canvas as it is made (`uistandards.md` §2, live feedback):
 * the pointer's points in document points, drawn [width] points wide with round ends, in the
 * primary colour or, for an eraser, as a translucent grey ([erases]).
 */
class LivePath(val width: Double, val erases: Boolean = false) {

    /** The points so far: x and y in document points, one pair after another. */
    var points = DoubleArray(64)
        private set

    /** How many points there are. */
    var count = 0
        private set

    /** Add a point at ([x], [y]) in document points. */
    fun add(x: Double, y: Double) {
        if (count * 2 + 2 > points.size) points = points.copyOf(points.size * 2)
        points[count * 2] = x
        points[count * 2 + 1] = y
        count++
    }
}

/** A tool whose strokes show as they are drawn: its stroke in progress, or `null`. */
interface DrawsLive {
    val livePath: LivePath?
}

/**
 * Keeps a tool's [LivePath] up to date from its pointer events: a press starts one [width] points
 * wide, drags add to it, and a release ends it.
 */
class LivePathRecorder {

    /** The stroke in progress, or `null`. */
    var path: LivePath? = null
        private set

    /** Follow [event]; [width] and [erases] describe a stroke a press starts. */
    fun follow(event: ToolPointer, width: Double, erases: Boolean = false) {
        when (event.kind) {
            ToolPointer.Kind.DOWN -> path = LivePath(width, erases).also { it.add(event.x, event.y) }
            ToolPointer.Kind.DRAG -> path?.add(event.x, event.y)
            ToolPointer.Kind.UP -> path = null
            ToolPointer.Kind.MOVE -> Unit
        }
    }

    /** Drop the stroke in progress. */
    fun clear() {
        path = null
    }
}

// GLOSSARY
// - LivePath(width, erases): a stroke being drawn: its points (document points), width (pt) and whether it erases.
// - points, count: the points as x, y pairs, and how many. add(x, y): add a point.
// - DrawsLive: a tool whose stroke in progress (livePath) shows as it is drawn.
// - LivePathRecorder: builds a tool's LivePath from its pointer events. path: the stroke in progress. follow(event, width, erases): press starts, drag adds, release ends. clear(): drop it.
