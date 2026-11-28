package com.extemporecreare.app.ui.dial

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectVerticalDragGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.KeyboardArrowDown
import androidx.compose.material.icons.outlined.KeyboardArrowUp
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import com.extemporecreare.app.ui.theme.EtcColors

/**
 * The dial's quick size adjustment (`uistandards.md` §6.2): a vertical pill with a gold knob
 * between an up and a down chevron. Dragging the knob up or down changes the active tool's [size]
 * by a share of it per point dragged; the chevrons step it by [STEP]. Shown only for tools with a
 * size ([size] null: the knob and chevrons do nothing and show muted).
 */
@Composable
fun DialSizeSlider(size: Double?, onSize: (Double) -> Unit, modifier: Modifier = Modifier) {
    val shape = RoundedCornerShape(26.dp)
    val tint = if (size != null) EtcColors.Text else EtcColors.Soft
    Column(
        modifier.width(52.dp).shadow(8.dp, shape, ambientColor = EtcColors.Bg, spotColor = EtcColors.Bg)
            .background(EtcColors.Panel2, shape).border(1.dp, EtcColors.Line, shape),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.SpaceBetween,
    ) {
        Box(Modifier.size(52.dp).clickable(enabled = size != null) { size?.let { onSize(it * STEP) } }.semantics { contentDescription = "Larger" }, contentAlignment = Alignment.Center) {
            Icon(Icons.Outlined.KeyboardArrowUp, null, tint = tint, modifier = Modifier.size(24.dp))
        }
        Box(
            Modifier.size(52.dp, 64.dp)
                .pointerInput(size) {
                    detectVerticalDragGestures { _, dy ->
                        val s = size ?: return@detectVerticalDragGestures
                        onSize(s * (1.0 - dy / 200.0))
                    }
                }
                .semantics { contentDescription = "Drag up or down to change the size" },
            contentAlignment = Alignment.Center,
        ) {
            Box(Modifier.size(34.dp).shadow(6.dp, CircleShape, ambientColor = EtcColors.Gold, spotColor = EtcColors.Gold).background(if (size != null) EtcColors.Gold else EtcColors.Line, CircleShape))
        }
        Box(Modifier.size(52.dp).clickable(enabled = size != null) { size?.let { onSize(it / STEP) } }.semantics { contentDescription = "Smaller" }, contentAlignment = Alignment.Center) {
            Icon(Icons.Outlined.KeyboardArrowDown, null, tint = tint, modifier = Modifier.size(24.dp))
        }
    }
}

/** How much a chevron changes the size: ten percent. */
private const val STEP = 1.1

// GLOSSARY
// - DialSizeSlider(size, onSize, modifier): the dial's vertical size slider: drag the knob, or step with the chevrons.
// - STEP: a chevron's size change, ×1.1.
