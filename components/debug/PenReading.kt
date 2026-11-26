package com.extemporecreare.app.components.debug

import android.view.MotionEvent
import java.util.Locale

/**
 * What one pointer reported in a touch or hover event, as Android gives it: the action, the
 * pointer's tool type, its screen and document position, pressure, tilt from upright and
 * orientation (degrees), hover distance, buttons, and the tilt the app sends the tools
 * ([tiltX], [tiltY], degrees in document axes; null for a pointer that isn't a stylus). Android
 * has no standard barrel rotation axis, so none is read.
 */
data class PenReading(
    val action: String,
    val toolType: String,
    val x: Float,
    val y: Float,
    val docX: Double,
    val docY: Double,
    val pressure: Float,
    val tilt: Double,
    val orientation: Double,
    val distance: Float,
    val buttons: Int,
    val tiltX: Float?,
    val tiltY: Float?,
) {
    /** One line for logcat and the readout. */
    fun line(): String = String.format(
        Locale.ROOT,
        "%s %s screen(%.1f, %.1f) doc(%.2f, %.2f) pressure %.3f tilt %.1f° orientation %.1f° distance %.2f buttons %d → tools tilt(%s, %s)",
        action, toolType, x, y, docX, docY, pressure, tilt, orientation, distance, buttons,
        tiltX?.let { String.format(Locale.ROOT, "%.1f", it) } ?: "-",
        tiltY?.let { String.format(Locale.ROOT, "%.1f", it) } ?: "-",
    )

    companion object {
        /** The reading of pointer [i] of [e], at document point ([docX], [docY]), with the tilt sent to the tools. */
        fun of(e: MotionEvent, i: Int, docX: Double, docY: Double, tiltX: Float?, tiltY: Float?): PenReading = PenReading(
            action = MotionEvent.actionToString(e.actionMasked),
            toolType = when (e.getToolType(i)) {
                MotionEvent.TOOL_TYPE_STYLUS -> "stylus"
                MotionEvent.TOOL_TYPE_ERASER -> "eraser"
                MotionEvent.TOOL_TYPE_FINGER -> "finger"
                MotionEvent.TOOL_TYPE_MOUSE -> "mouse"
                else -> "unknown"
            },
            x = e.getX(i),
            y = e.getY(i),
            docX = docX,
            docY = docY,
            pressure = e.getPressure(i),
            tilt = Math.toDegrees(e.getAxisValue(MotionEvent.AXIS_TILT, i).toDouble()),
            orientation = Math.toDegrees(e.getAxisValue(MotionEvent.AXIS_ORIENTATION, i).toDouble()),
            distance = e.getAxisValue(MotionEvent.AXIS_DISTANCE, i),
            buttons = e.buttonState,
            tiltX = tiltX,
            tiltY = tiltY,
        )
    }
}

// GLOSSARY
// - PenReading: what one pointer reported (action, tool type, positions, pressure, tilt, orientation, distance, buttons, tools' tilt).
// - line(): the reading as one line.
// - of(e, i, docX, docY, tiltX, tiltY): the reading of a MotionEvent pointer.
