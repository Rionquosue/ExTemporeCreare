package com.extemporecreare.app.ui.dial

import androidx.compose.animation.core.Animatable
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.layout
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import com.extemporecreare.app.ui.theme.EtcColors
import kotlinx.coroutines.launch
import kotlin.math.atan2
import kotlin.math.cos
import kotlin.math.roundToInt
import kotlin.math.sin

/**
 * The rotary tool dial (`uistandards.md` §6.1): the tools sit round a ring that turns beneath a
 * fixed active sight 45° into the upper-right quadrant. Dragging round the ring turns it; on release
 * it settles with a tool in the sight, which becomes active ([onChoose]). Tapping an inactive tool
 * does nothing; tapping the active one opens its settings ([onActiveTap]). Slot spacing follows the
 * number of tools. The dial is drawn [diameter] across; its placement (cropped into the corner) is
 * the caller's. [center] is drawn in its middle (the colour control).
 */
@Composable
fun RotaryDial(
    names: List<String>,
    active: Int,
    onChoose: (Int) -> Unit,
    onActiveTap: () -> Unit,
    modifier: Modifier = Modifier,
    diameter: Dp = 340.dp,
    center: @Composable () -> Unit,
) {
    val step = 360f / names.size
    // The ring's turn, in degrees: tool i sits at the sight when turn = i × step.
    val turn = remember { Animatable(active * step) }
    val scope = rememberCoroutineScope()
    LaunchedEffect(active) { if ((turn.value / step).roundToInt().mod(names.size) != active) turn.animateTo(active * step) }
    val radiusPx = with(LocalDensity.current) { (diameter / 2).toPx() }
    val ringPx = radiusPx * RING
    Box(
        modifier
            .size(diameter)
            .background(EtcColors.Panel, CircleShape)
            .border(1.dp, EtcColors.Line, CircleShape)
            .pointerInput(names.size) {
                var last = 0f
                detectDragGestures(
                    onDragStart = { p -> last = angleOf(p, radiusPx) },
                    onDragEnd = {
                        val index = (turn.value / step).roundToInt().mod(names.size)
                        scope.launch { turn.animateTo((turn.value / step).roundToInt() * step) }
                        onChoose(index)
                    },
                ) { change, _ ->
                    val now = angleOf(change.position, radiusPx)
                    var delta = now - last
                    if (delta > 180f) delta -= 360f
                    if (delta < -180f) delta += 360f
                    last = now
                    // Turning the ring clockwise brings the next tool round into the sight.
                    scope.launch { turn.snapTo(turn.value + delta) }
                }
            }
            .pointerInput(active, names.size) {
                detectTapGestures { p ->
                    val (sx, sy) = slot(active, turn.value, step, ringPx, radiusPx)
                    if ((p - Offset(sx, sy)).getDistance() < radiusPx * 0.16f) onActiveTap()
                }
            },
    ) {
        Canvas(Modifier.size(diameter)) {
            drawCircle(EtcColors.LineSoft, radius = ringPx, style = Stroke(1.dp.toPx()))
            // The active sight.
            val (sx, sy) = polar(SIGHT, ringPx, radiusPx)
            drawCircle(EtcColors.Gold, radius = radiusPx * 0.15f, center = Offset(sx, sy), style = Stroke(2.dp.toPx()))
        }
        names.forEachIndexed { i, name ->
            val (x, y) = slot(i, turn.value, step, ringPx, radiusPx)
            val inSight = i == (turn.value / step).roundToInt().mod(names.size)
            val size = if (inSight) 50.dp else 40.dp
            Box(
                Modifier
                    .layout { m, c ->
                        val p = m.measure(c)
                        layout(p.width, p.height) { p.place((x - p.width / 2f).roundToInt(), (y - p.height / 2f).roundToInt()) }
                    }
                    .size(size)
                    .background(if (inSight) EtcColors.Gold else EtcColors.Panel2, CircleShape)
                    .border(1.dp, if (inSight) EtcColors.Gold else EtcColors.Line, CircleShape)
                    .semantics { contentDescription = name },
                contentAlignment = Alignment.Center,
            ) {
                Icon(iconOf(name), null, tint = if (inSight) EtcColors.Bg else EtcColors.Muted, modifier = Modifier.size(if (inSight) 24.dp else 18.dp))
            }
        }
        Box(Modifier.align(Alignment.Center)) { center() }
    }
}

/** Where the sight is: 45° into the upper-right quadrant (degrees, counter-clockwise from right). */
private const val SIGHT = 45f

/** The ring's radius as a share of the dial's. */
private const val RING = 0.74f

/** The angle (degrees, counter-clockwise from right) of point [p] round the dial's centre. */
private fun angleOf(p: Offset, r: Float): Float = Math.toDegrees(atan2((r - p.y).toDouble(), (p.x - r).toDouble())).toFloat()

/** A point at [deg] degrees on a circle of radius [ring] about the centre of a dial of radius [r]. */
private fun polar(deg: Float, ring: Float, r: Float): Pair<Float, Float> {
    val a = Math.toRadians(deg.toDouble())
    return (r + ring * cos(a)).toFloat() to (r - ring * sin(a)).toFloat()
}

/** Where tool [i] sits on the ring at [turn]: at the sight when turn = i × step, the others following anticlockwise. */
private fun slot(i: Int, turn: Float, step: Float, ring: Float, r: Float): Pair<Float, Float> = polar(SIGHT + (turn - i * step), ring, r)

// GLOSSARY
// - RotaryDial(names, active, onChoose, onActiveTap, modifier, diameter, center): the rotary tool dial.
// - SIGHT: the active sight's angle, 45°. RING: the ring's radius share, 0.74.
// - angleOf(p, r): a point's angle round the centre.
// - polar(deg, ring, r): a point on the ring.
// - slot(i, turn, step, ring, r): where a tool sits for the ring's turn.
