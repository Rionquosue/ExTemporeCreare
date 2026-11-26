package com.extemporecreare.app.ui.editor

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.gestures.detectDragGesturesAfterLongPress
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraintsScope
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.dp
import com.extemporecreare.app.ui.theme.EtcColors
import kotlin.math.roundToInt

/**
 * A floating panel that can be moved (`uistandards.md` §10): holding its header ([content]'s
 * header modifier) and dragging moves it with visible feedback; near a dock target (the workspace's
 * left top, right top or right bottom) a placement preview shows, and releasing there docks it;
 * releasing elsewhere leaves it floating where it is. Its place is [layout]'s for [key], kept within
 * the workspace whatever its size. The long press keeps the header's tap (collapse) from firing
 * after a drag, and other controls in the panel don't start a move.
 */
@Composable
fun BoxWithConstraintsScope.MovablePanel(key: String, layout: PanelLayout, content: @Composable (Modifier) -> Unit) {
    val density = LocalDensity.current
    val area = with(density) { IntSize(maxWidth.roundToPx(), maxHeight.roundToPx()) }
    val margin = with(density) { 14.dp.toPx() }
    var size by remember { mutableStateOf(IntSize.Zero) }
    var drag by remember { mutableStateOf<Offset?>(null) }
    val place = layout.places[key] ?: PanelLayout.Place(PanelLayout.DEFAULT_DOCKS[key] ?: "topStart")
    val home = positionOf(place, size, area, margin)
    val now = drag?.let { home + it } ?: home
    val target = drag?.let { dockNear(now, size, area, margin * 5) }
    if (target != null) {
        val p = positionOf(PanelLayout.Place(target), size, area, margin)
        Box(
            Modifier.offset { IntOffset(p.x.roundToInt(), p.y.roundToInt()) }
                .size(with(density) { size.width.toDp() }, with(density) { size.height.toDp() })
                .background(EtcColors.Gold.copy(alpha = 0.18f), RoundedCornerShape(16.dp))
                .border(2.dp, EtcColors.Gold, RoundedCornerShape(16.dp)),
        )
    }
    val header = Modifier.pointerInput(key, area, size) {
        detectDragGesturesAfterLongPress(
            onDragStart = { drag = Offset.Zero },
            onDragEnd = {
                val end = home + (drag ?: Offset.Zero)
                val dock = dockNear(end, size, area, margin * 5)
                val clamped = clamp(end, size, area)
                layout.place(key, if (dock != null) PanelLayout.Place(dock) else PanelLayout.Place("float", clamped.x, clamped.y))
                drag = null
            },
            onDragCancel = { drag = null },
        ) { change, delta ->
            change.consume()
            drag = (drag ?: Offset.Zero) + delta
        }
    }
    Box(
        Modifier.offset { IntOffset(now.x.roundToInt(), now.y.roundToInt()) }
            .onSizeChanged { size = it }
            .alpha(if (drag != null) 0.85f else 1f),
    ) { content(header) }
}

/** The top-left corner of a panel of [size] at [place] in a workspace of [area]. */
private fun positionOf(place: PanelLayout.Place, size: IntSize, area: IntSize, margin: Float): Offset {
    val p = when (place.dock) {
        "topStart" -> Offset(margin, margin)
        "topEnd" -> Offset(area.width - size.width - margin, margin)
        "bottomEnd" -> Offset(area.width - size.width - margin, area.height - size.height - margin)
        else -> Offset(place.x, place.y)
    }
    return clamp(p, size, area)
}

/** [p] kept so a panel of [size] stays in the workspace of [area]. */
private fun clamp(p: Offset, size: IntSize, area: IntSize): Offset = Offset(
    p.x.coerceIn(0f, (area.width - size.width).coerceAtLeast(0).toFloat()),
    p.y.coerceIn(0f, (area.height - size.height).coerceAtLeast(0).toFloat()),
)

/** The dock target a panel of [size] at [p] is within [reach] of, or null. */
private fun dockNear(p: Offset, size: IntSize, area: IntSize, reach: Float): String? {
    val nearLeft = p.x < reach
    val nearRight = area.width - (p.x + size.width) < reach
    val nearTop = p.y < reach
    val nearBottom = area.height - (p.y + size.height) < reach
    return when {
        nearLeft && nearTop -> "topStart"
        nearRight && nearTop -> "topEnd"
        nearRight && nearBottom -> "bottomEnd"
        else -> null
    }
}

// GLOSSARY
// - MovablePanel(key, layout, content): a panel moved by holding and dragging its header, docking at edge targets.
// - positionOf(place, size, area, margin): where a panel's place puts it.
// - clamp(p, size, area): kept inside the workspace.
// - dockNear(p, size, area, reach): the dock target near a position.
