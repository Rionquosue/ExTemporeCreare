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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalConfiguration
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
 * colour control, readout and settings) away from its corner, within the screen; ring rotation and
 * colour taps don't move it.
 */
@Composable
fun DialGrip(layout: PanelLayout, modifier: Modifier = Modifier) {
    val density = LocalDensity.current
    val config = LocalConfiguration.current
    Box(
        modifier
            .size(44.dp)
            .semantics { contentDescription = "Dial grip: hold and drag to move the dial" }
            .pointerInput(Unit) {
                detectDragGesturesAfterLongPress { change, delta ->
                    change.consume()
                    with(density) {
                        layout.moveDial(delta.x.toDp().value, delta.y.toDp().value, config.screenWidthDp * 0.6f, config.screenHeightDp * 0.6f)
                    }
                }
            },
        contentAlignment = Alignment.Center,
    ) {
        Box(Modifier.size(28.dp).background(EtcColors.Panel2, CircleShape).border(1.dp, EtcColors.Line, CircleShape), contentAlignment = Alignment.Center) {
            Icon(Icons.Outlined.DragIndicator, null, tint = EtcColors.Muted, modifier = Modifier.size(16.dp))
        }
    }
}

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
// - DialGrip(layout, modifier): hold and drag to move the dial.
// - DialStepper(onBack, onNext, modifier): step the dial by one tool.
