package com.extemporecreare.app.ui.color

import android.graphics.Bitmap
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.dp
import com.extemporecreare.app.components.color.ColorMath
import com.extemporecreare.app.ui.theme.EtcColors
import com.extemporecreare.app.ui.tools.SettingButton
import com.extemporecreare.app.ui.tools.SettingChoice
import kotlin.math.hypot

/**
 * The colour workspace's mixing pad (`uistandards.md` §8.1): paint on it with the primary or the
 * alternate colour, which mixes with what is there ([ColorMath.mixPigment]); in Sample mode a tap
 * takes the colour under it as the primary ([onSample]); Reset clears it. Its paint lasts while
 * the app runs.
 */
@Composable
fun MixingPad(primary: String?, alternate: String?, onSample: (String) -> Unit) {
    val pad = remember { Bitmap.createBitmap(PAD_W, PAD_H, Bitmap.Config.ARGB_8888) }
    var version by remember { mutableIntStateOf(0) }
    var mode by remember { mutableStateOf("primary") }
    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        SettingChoice("Mixing pad", listOf("primary" to "Primary", "alternate" to "Alternate", "sample" to "Sample"), mode) { mode = it }
        Canvas(
            Modifier.fillMaxWidth().height(120.dp).background(EtcColors.Text, RoundedCornerShape(8.dp)).border(1.dp, EtcColors.Line, RoundedCornerShape(8.dp))
                .pointerInput(mode, primary, alternate) {
                    val toPad = { x: Float, y: Float -> (x / size.width * PAD_W).toInt() to (y / size.height * PAD_H).toInt() }
                    if (mode == "sample") {
                        detectTapGestures { p ->
                            val (x, y) = toPad(p.x, p.y)
                            val c = pad.getPixel(x.coerceIn(0, PAD_W - 1), y.coerceIn(0, PAD_H - 1))
                            if (android.graphics.Color.alpha(c) > 0) {
                                onSample(ColorMath.hex(android.graphics.Color.red(c), android.graphics.Color.green(c), android.graphics.Color.blue(c)))
                            }
                        }
                    } else {
                        val paint = ColorMath.parse(if (mode == "primary") primary else alternate) ?: return@pointerInput
                        detectDragGestures { change, _ ->
                            val (x, y) = toPad(change.position.x, change.position.y)
                            dab(pad, x, y, paint)
                            version++
                        }
                    }
                },
        ) {
            version.let { _ ->
                drawImage(pad.asImageBitmap(), dstSize = IntSize(size.width.toInt(), size.height.toInt()), dstOffset = IntOffset.Zero, filterQuality = FilterQuality.Low)
            }
        }
        Row { SettingButton("Reset pad", { pad.eraseColor(0); version++ }) }
    }
}

/** Paint a soft dab of [rgb] at ([cx], [cy]) on [pad], mixing pigment-like with what is there. */
private fun dab(pad: Bitmap, cx: Int, cy: Int, rgb: IntArray) {
    for (y in (cy - RADIUS).coerceAtLeast(0)..(cy + RADIUS).coerceAtMost(PAD_H - 1)) {
        for (x in (cx - RADIUS).coerceAtLeast(0)..(cx + RADIUS).coerceAtMost(PAD_W - 1)) {
            val d = hypot((x - cx).toDouble(), (y - cy).toDouble()) / RADIUS
            if (d > 1) continue
            val strength = (1 - d) * STRENGTH
            val old = pad.getPixel(x, y)
            val mixed = if (android.graphics.Color.alpha(old) == 0) {
                rgb
            } else {
                ColorMath.mixPigment(rgb, intArrayOf(android.graphics.Color.red(old), android.graphics.Color.green(old), android.graphics.Color.blue(old)), strength)
            }
            pad.setPixel(x, y, android.graphics.Color.rgb(mixed[0], mixed[1], mixed[2]))
        }
    }
}

/** The pad's pixels, a dab's radius in them, and how much a dab's centre takes on its colour. */
private const val PAD_W = 240
private const val PAD_H = 96
private const val RADIUS = 7
private const val STRENGTH = 0.35

// GLOSSARY
// - MixingPad(primary, alternate, onSample): the mixing pad (paint primary or alternate, sample, reset).
// - dab(pad, cx, cy, rgb): one soft pigment-like dab.
// - PAD_W, PAD_H: 240 × 96 px. RADIUS: 7 px. STRENGTH: 0.35.
