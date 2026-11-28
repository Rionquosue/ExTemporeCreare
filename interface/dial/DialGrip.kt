package com.extemporecreare.app.ui.dial

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.gestures.detectDragGesturesAfterLongPress
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.KeyboardArrowLeft
import androidx.compose.material.icons.automirrored.outlined.KeyboardArrowRight
import androidx.compose.material.icons.outlined.DragIndicator
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.runtime.setValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.layout.boundsInRoot
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import com.extemporecreare.app.ui.editor.PanelLayout
import com.extemporecreare.app.ui.theme.EtcColors

/**
 * The dial's grip (`uistandards.md` §6.1, §6.3): holding it and dragging moves the dial (with its
 * colour control, readout and settings); ring rotation and colour taps don't move it. Dragged to
 * within [DOCK_REACH] of an edge of the workspace ([origin], [area]: its place in the root and its
 * size, in pixels), a preview shows ([PanelLayout.toolbarPreview]) and releasing there docks the
 * dial to that edge as a toolbar; the docked toolbar's grip dragged away from its edge makes it the
 * rotary dial again where it is released. Released elsewhere, the dial stays within its usual reach
 * of its corner.
 */
@Composable
fun DialGrip(layout: PanelLayout, origin: Offset, area: IntSize, modifier: Modifier = Modifier) {
    val density = LocalDensity.current
    var centre by remember { mutableStateOf(Offset.Zero) }
    var at by remember { mutableStateOf<Offset?>(null) }
    Box(
        modifier
            .size(44.dp)
            .onGloballyPositioned { centre = it.boundsInRoot().center - origin }
            .semantics { contentDescription = "Dial grip: hold and drag to move the dial, or to an edge to dock it as a toolbar" }
            .pointerInput(area, origin) {
                val reach = DOCK_REACH.toPx()
                detectDragGesturesAfterLongPress(
                    onDragStart = { at = centre },
                    onDragEnd = {
                        val end = at ?: centre
                        val edge = edgeNear(end, area, reach)
                        layout.toolbarPreview = null
                        at = null
                        with(density) {
                            val (w, h) = area.width.toDp().value to area.height.toDp().value
                            when {
                                edge != null -> layout.dockToolbar(edge)
                                layout.toolbarDock != null -> {
                                    // Undocked: the rotary dial's grip where the finger let go.
                                    layout.dockToolbar(null)
                                    layout.placeDial(end.x.toDp().value - PanelLayout.GRIP_X_DP, end.y.toDp().value - (h - PanelLayout.GRIP_FROM_BOTTOM_DP), w * 0.6f, h * 0.6f)
                                }
                                else -> layout.placeDial(layout.dialX, layout.dialY, w * 0.6f, h * 0.6f)
                            }
                        }
                    },
                    onDragCancel = { at = null; layout.toolbarPreview = null },
                ) { change, delta ->
                    change.consume()
                    val now = (at ?: centre) + delta
                    at = now
                    layout.toolbarPreview = edgeNear(now, area, reach)
                    if (layout.toolbarDock == null) {
                        // The rotary dial follows the grip anywhere in the workspace while it moves.
                        with(density) {
                            val (w, h) = area.width.toDp().value to area.height.toDp().value
                            layout.moveDialFreely(delta.x.toDp().value, delta.y.toDp().value, w, h)
                        }
                    }
                }
            },
        contentAlignment = Alignment.Center,
    ) {
        Icon(Icons.Outlined.DragIndicator, null, tint = EtcColors.Muted, modifier = Modifier.size(24.dp))
    }
}

/** The edge of a workspace of [area] that [p] is within [reach] of (the nearest), or null. */
private fun edgeNear(p: Offset, area: IntSize, reach: Float): String? = listOf(
    "left" to p.x,
    "right" to area.width - p.x,
    "top" to p.y,
    "bottom" to area.height - p.y,
).filter { it.second < reach }.minByOrNull { it.second }?.first

/** How near an edge the grip docks the dial as a toolbar. */
val DOCK_REACH = 48.dp

/** Buttons stepping the dial back and on by one tool: the stepped alternative to turning it (§13.2). */
@Composable
fun DialStepper(onBack: () -> Unit, onNext: () -> Unit, modifier: Modifier = Modifier) {
    Row(modifier, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        for ((icon, label, action) in listOf(
            Triple(Icons.AutoMirrored.Outlined.KeyboardArrowLeft, "Previous tool", onBack),
            Triple(Icons.AutoMirrored.Outlined.KeyboardArrowRight, "Next tool", onNext),
        )) {
            Box(Modifier.size(44.dp).clickable(onClick = action).semantics { contentDescription = label }, contentAlignment = Alignment.Center) {
                Box(Modifier.size(28.dp).background(EtcColors.Panel2, CircleShape).border(1.dp, EtcColors.Line, CircleShape), contentAlignment = Alignment.Center) {
                    Icon(icon, null, tint = EtcColors.Text, modifier = Modifier.size(18.dp))
                }
            }
        }
    }
}

// GLOSSARY
// - DialGrip(layout, origin, area, modifier): hold and drag to move the dial, or to an edge to dock it as a toolbar.
// - edgeNear(p, area, reach): the edge a point is near. DOCK_REACH: 48 dp.
// - DialStepper(onBack, onNext, modifier): step the dial by one tool.
