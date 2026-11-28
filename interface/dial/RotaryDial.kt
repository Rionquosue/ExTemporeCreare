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
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.KeyboardArrowRight
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.PathEffect
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
 * does nothing; tapping the active one, or the settings chevron beside it, opens its settings
 * ([onActiveTap]). Slot spacing follows the number of tools.
 *
 * The dial is drawn [diameter] across: a frosted outer ring with a dashed track, the tools on it
 * (the active one larger, gold, with a glow) and an inner disc ([INNER] of the radius) that the
 * caller fills with the size slider, readout and colour control. Its placement (cropped by the
 * workspace's edges) is the caller's.
 */
@Composable
fun RotaryDial(
    names: List<String>,
    active: Int,
    onChoose: (Int) -> Unit,
    onActiveTap: () -> Unit,
    modifier: Modifier = Modifier,
    diameter: Dp = DIAL_DIAMETER,
) {
    val step = 360f / names.size
    // The ring's turn, in degrees: tool i sits at the sight when turn = i × step.
    val turn = remember { Animatable(active * step) }
    val scope = rememberCoroutineScope()
    LaunchedEffect(active) { if ((turn.value / step).roundToInt().mod(names.size) != active) turn.animateTo(active * step) }
    val density = LocalDensity.current
    val radiusPx = with(density) { (diameter / 2).toPx() }
    val ringPx = radiusPx * RING
    val chipPx = ringPx + with(density) { CHIP_OUT.toPx() }
    Box(
        modifier
            .size(diameter)
            .shadow(18.dp, CircleShape, ambientColor = EtcColors.Bg, spotColor = EtcColors.Bg)
            .background(EtcColors.Panel, CircleShape)
            .border(1.dp, EtcColors.Line, CircleShape)
            .pointerInput(names.size) {
                var last = 0f
                var turning = false
                detectDragGestures(
                    onDragStart = { p ->
                        last = angleOf(p, radiusPx)
                        // The inner disc holds the size slider, readout and colours: only the ring turns.
                        turning = (p - Offset(radiusPx, radiusPx)).getDistance() > radiusPx * INNER
                    },
                    onDragEnd = {
                        if (turning) {
                            val index = (turn.value / step).roundToInt().mod(names.size)
                            scope.launch { turn.animateTo((turn.value / step).roundToInt() * step) }
                            onChoose(index)
                        }
                    },
                ) { change, _ ->
                    if (!turning) return@detectDragGestures
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
                    val (cx, cy) = polar(SIGHT - CHIP_ANGLE, chipPx, radiusPx)
                    if ((p - Offset(sx, sy)).getDistance() < ACTIVE_SIZE.toPx() / 2 || (p - Offset(cx, cy)).getDistance() < 22.dp.toPx()) onActiveTap()
                }
            },
    ) {
        Canvas(Modifier.size(diameter)) {
            // The dashed track near the rim and the inner disc.
            drawCircle(EtcColors.Line, radius = radiusPx * TRACK, style = Stroke(1.dp.toPx(), pathEffect = PathEffect.dashPathEffect(floatArrayOf(6.dp.toPx(), 6.dp.toPx()))))
            drawCircle(EtcColors.Shell.copy(alpha = 0.72f), radius = radiusPx * INNER)
            drawCircle(EtcColors.Line, radius = radiusPx * INNER, style = Stroke(1.dp.toPx()))
            // The active sight's glow.
            val (sx, sy) = polar(SIGHT, ringPx, radiusPx)
            drawCircle(Brush.radialGradient(listOf(EtcColors.Gold.copy(alpha = 0.45f), EtcColors.Gold.copy(alpha = 0f)), Offset(sx, sy), ACTIVE_SIZE.toPx()), radius = ACTIVE_SIZE.toPx(), center = Offset(sx, sy))
        }
        names.forEachIndexed { i, name ->
            val (x, y) = slot(i, turn.value, step, ringPx, radiusPx)
            val inSight = i == (turn.value / step).roundToInt().mod(names.size)
            val size = if (inSight) ACTIVE_SIZE else SLOT_SIZE
            Box(
                Modifier
                    .layout { m, c ->
                        val p = m.measure(c)
                        layout(p.width, p.height) { p.place((x - p.width / 2f).roundToInt(), (y - p.height / 2f).roundToInt()) }
                    }
                    .size(size)
                    .background(if (inSight) EtcColors.Gold else EtcColors.Panel2.copy(alpha = 0.7f), CircleShape)
                    .border(if (inSight) 2.dp else 1.dp, if (inSight) EtcColors.Gold2 else EtcColors.LineSoft, CircleShape)
                    .semantics { contentDescription = if (inSight) "$name, active" else name },
                contentAlignment = Alignment.Center,
            ) {
                Icon(iconOf(name), null, tint = if (inSight) EtcColors.Bg else EtcColors.Soft, modifier = Modifier.size(if (inSight) 32.dp else 24.dp))
            }
        }
        // The settings chevron beside the sight (`uistandards.md` §6.2: the settings trigger).
        val (cx, cy) = polar(SIGHT - CHIP_ANGLE, chipPx, radiusPx)
        Box(
            Modifier
                .layout { m, c ->
                    val p = m.measure(c)
                    layout(p.width, p.height) { p.place((cx - p.width / 2f).roundToInt(), (cy - p.height / 2f).roundToInt()) }
                }
                .size(34.dp)
                .background(EtcColors.Panel2, CircleShape)
                .border(1.dp, EtcColors.Line, CircleShape)
                .semantics { contentDescription = "Tool settings" },
            contentAlignment = Alignment.Center,
        ) { Icon(Icons.AutoMirrored.Outlined.KeyboardArrowRight, null, tint = EtcColors.Text, modifier = Modifier.size(20.dp)) }
    }
}

/** The dial's size: the mockup's large corner dial. */
val DIAL_DIAMETER = 640.dp

/** Where the sight is: 45° into the upper-right quadrant (degrees, counter-clockwise from right). */
private const val SIGHT = 45f

/** The ring's, the dashed track's and the inner disc's radii as shares of the dial's. */
private const val RING = 0.80f
private const val TRACK = 0.94f
const val INNER = 0.62f

/** A tool's circle, and the active tool's. */
private val SLOT_SIZE = 56.dp
private val ACTIVE_SIZE = 70.dp

/** The settings chevron: how far beyond the ring, and how many degrees clockwise of the sight. */
private val CHIP_OUT = 48.dp
private const val CHIP_ANGLE = 9f

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
// - RotaryDial(names, active, onChoose, onActiveTap, modifier, diameter): the rotary tool dial with its settings chevron; the inner disc is the caller's.
// - DIAL_DIAMETER: 640 dp. SIGHT: the active sight's angle, 45°. RING, TRACK, INNER: the ring's, dashed track's and inner disc's radius shares.
// - SLOT_SIZE, ACTIVE_SIZE: a tool's circle, 56 dp, and the active tool's, 70 dp. CHIP_OUT, CHIP_ANGLE: where the settings chevron sits.
// - angleOf(p, r): a point's angle round the centre.
// - polar(deg, ring, r): a point on the ring.
// - slot(i, turn, step, ring, r): where a tool sits for the ring's turn.
