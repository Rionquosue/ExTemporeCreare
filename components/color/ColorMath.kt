package com.extemporecreare.app.components.color

import kotlin.math.abs
import kotlin.math.max
import kotlin.math.min
import kotlin.math.pow
import kotlin.math.roundToInt

/**
 * Colour conversions for the colour workspace: `#rrggbb` text, RGB 0..255, HSV (hue 0..360,
 * saturation and value 0..1), and the mixing pad's pigment-like mix.
 */
object ColorMath {

    /** `#rrggbb` as [r, g, b] 0..255, or null when it isn't one. */
    fun parse(hex: String?): IntArray? {
        val h = hex?.trim()?.removePrefix("#") ?: return null
        if (h.length != 6 || !h.all { it.isDigit() || it.lowercaseChar() in 'a'..'f' }) return null
        return intArrayOf(h.substring(0, 2).toInt(16), h.substring(2, 4).toInt(16), h.substring(4, 6).toInt(16))
    }

    /** [r], [g], [b] 0..255 as `#rrggbb`. */
    fun hex(r: Int, g: Int, b: Int): String = "#%02x%02x%02x".format(r.coerceIn(0, 255), g.coerceIn(0, 255), b.coerceIn(0, 255))

    /** RGB 0..255 as HSV: hue 0..360, saturation and value 0..1. */
    fun toHsv(rgb: IntArray): FloatArray {
        val (r, g, b) = rgb.map { it / 255f }
        val mx = max(r, max(g, b))
        val mn = min(r, min(g, b))
        val d = mx - mn
        val h = when {
            d == 0f -> 0f
            mx == r -> 60f * (((g - b) / d) % 6f)
            mx == g -> 60f * (((b - r) / d) + 2f)
            else -> 60f * (((r - g) / d) + 4f)
        }.let { if (it < 0) it + 360f else it }
        return floatArrayOf(h, if (mx == 0f) 0f else d / mx, mx)
    }

    /** HSV (hue 0..360, saturation and value 0..1) as `#rrggbb`. */
    fun fromHsv(h: Float, s: Float, v: Float): String {
        val c = v * s
        val x = c * (1 - abs((h / 60f) % 2f - 1))
        val m = v - c
        val (r, g, b) = when ((h.coerceIn(0f, 359.999f) / 60f).toInt()) {
            0 -> Triple(c, x, 0f)
            1 -> Triple(x, c, 0f)
            2 -> Triple(0f, c, x)
            3 -> Triple(0f, x, c)
            4 -> Triple(x, 0f, c)
            else -> Triple(c, 0f, x)
        }
        return hex(((r + m) * 255).roundToInt(), ((g + m) * 255).roundToInt(), ((b + m) * 255).roundToInt())
    }

    private fun toLinear(c: Int): Double = (c / 255.0).let { if (it <= 0.04045) it / 12.92 else ((it + 0.055) / 1.055).pow(2.4) }
    private fun toSrgb(l: Double): Int = (if (l <= 0.0031308) l * 12.92 else 1.055 * l.pow(1 / 2.4) - 0.055).times(255).roundToInt().coerceIn(0, 255)

    /**
     * The mixing pad's mix of [a] into [b] by [t] (0: all [b], 1: all [a]), pigment-like rather than
     * an average: a weighted geometric mean of the colours' linear-light reflectances, so blue and
     * yellow make green and mixes darken as paint does. A provisional model (`uistandards.md` §8.1
     * leaves the production pigment model open).
     */
    fun mixPigment(a: IntArray, b: IntArray, t: Double): IntArray {
        val w = t.coerceIn(0.0, 1.0)
        return IntArray(3) { i ->
            val la = toLinear(a[i]).coerceAtLeast(1e-4)
            val lb = toLinear(b[i]).coerceAtLeast(1e-4)
            toSrgb(la.pow(w) * lb.pow(1 - w))
        }
    }
}

// GLOSSARY
// - ColorMath: colour conversions and the pad's mix.
// - parse(hex), hex(r, g, b): #rrggbb text and RGB.
// - toHsv(rgb), fromHsv(h, s, v): HSV.
// - toLinear, toSrgb: sRGB transfer.
// - mixPigment(a, b, t): the provisional pigment-like mix.
