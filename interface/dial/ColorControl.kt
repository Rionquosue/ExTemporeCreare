package com.extemporecreare.app.ui.dial

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import com.extemporecreare.app.ui.theme.EtcColors

/**
 * The dial's central colour control (`uistandards.md` §6.3): the primary colour in front, the
 * alternate behind it, both visible. A single tap swaps them ([onSwap]), recognised only once a
 * double tap is ruled out; a double tap opens or closes the colour workspace ([onWorkspace]).
 */
@Composable
fun ColorControl(primary: String?, alternate: String?, onSwap: () -> Unit, onWorkspace: () -> Unit, modifier: Modifier = Modifier) {
    Box(
        modifier
            .size(76.dp)
            .semantics { contentDescription = "Colours: tap to swap, double tap for the colour workspace" }
            .pointerInput(Unit) { detectTapGestures(onTap = { onSwap() }, onDoubleTap = { onWorkspace() }) },
    ) {
        Swatch(alternate, Modifier.offset(28.dp, 28.dp).size(44.dp))
        Swatch(primary, Modifier.offset(2.dp, 2.dp).size(52.dp))
    }
}

/** One colour sample: `#rrggbb`, or a slashed white disc for no colour. */
@Composable
private fun Swatch(hex: String?, modifier: Modifier) {
    val color = hex?.let { runCatching { Color(android.graphics.Color.parseColor(it)) }.getOrNull() }
    Box(modifier.background(color ?: Color.White, CircleShape).border(2.dp, EtcColors.Text, CircleShape)) {
        if (color == null) Box(Modifier.offset(0.dp, 0.dp).size(2.dp).background(Color.Red))
    }
}

// GLOSSARY
// - ColorControl(primary, alternate, onSwap, onWorkspace, modifier): the dial's colour control.
// - Swatch(hex, modifier): one colour sample.
