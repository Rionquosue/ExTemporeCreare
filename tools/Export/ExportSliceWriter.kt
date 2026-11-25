package com.extemporecreare.app.tools.export

import android.content.Context
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Color
import android.net.Uri
import android.os.Build
import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.DragBox
import kotlin.math.roundToInt

/**
 * Writes export regions as image files: the engine renders the whole document's composition into
 * the region (so effects from outside it are kept, §7A), then it is encoded as PNG, JPEG (on white,
 * which has no transparency) or WebP and written to a content URI. Without user slices the region
 * is the automatic content slice (§9.4); with none, there is nothing to export.
 */
class ExportSliceWriter(private val context: Context) {

    /** One region to write: its name, box, format and scale. */
    data class Region(val name: String, val box: DragBox, val format: String, val scale: Double)

    /** The regions an export writes: the visible slices, else the automatic content slice, else none ("Nothing to export"). */
    fun regions(list: ExportSliceList): List<Region> =
        if (list.slices.isNotEmpty()) {
            list.slices.filter { it.visible }.map { Region(it.name, it.box(), it.format, it.scale) }
        } else {
            listOfNotNull(list.contentBounds?.let { Region("Content", it, "png", 1.0) })
        }

    /** Render [region] and write it to [uri] → an error message, or `null`. */
    fun write(engine: EngineBridge, region: Region, uri: Uri): String? {
        val w = (region.box.width * region.scale).roundToInt()
        val h = (region.box.height * region.scale).roundToInt()
        if (w < 1 || h < 1) return "${region.name} is smaller than one pixel at this scale"
        if (w.toLong() * h > MAX_PIXELS) return "${region.name} is too large to export at this scale"
        val buffer = EngineBridge.renderBuffer(w, h)
        if (!engine.render(region.box.left, region.box.top, w, h, region.scale, buffer)) return "${region.name} couldn't be rendered"
        var bitmap = Bitmap.createBitmap(w, h, Bitmap.Config.ARGB_8888)
        buffer.rewind()
        bitmap.copyPixelsFromBuffer(buffer)
        val format = when (region.format) {
            "jpg" -> Bitmap.CompressFormat.JPEG
            "webp" -> if (Build.VERSION.SDK_INT >= 30) Bitmap.CompressFormat.WEBP_LOSSLESS else @Suppress("DEPRECATION") Bitmap.CompressFormat.WEBP
            else -> Bitmap.CompressFormat.PNG
        }
        if (format == Bitmap.CompressFormat.JPEG) bitmap = onWhite(bitmap)
        return try {
            val out = context.contentResolver.openOutputStream(uri, "wt") ?: return "can't write $uri"
            val ok = out.use { bitmap.compress(format, QUALITY, it) }
            if (ok) null else "${region.name} couldn't be encoded"
        } catch (e: Exception) {
            e.message ?: e.toString()
        } finally {
            bitmap.recycle()
        }
    }

    /** [bitmap] drawn over white, for a format without transparency. */
    private fun onWhite(bitmap: Bitmap): Bitmap {
        val flat = Bitmap.createBitmap(bitmap.width, bitmap.height, Bitmap.Config.ARGB_8888)
        Canvas(flat).apply {
            drawColor(Color.WHITE)
            drawBitmap(bitmap, 0f, 0f, null)
        }
        bitmap.recycle()
        return flat
    }

    companion object {
        /** The most pixels one exported image may have. */
        const val MAX_PIXELS = 100_000_000L

        /** JPEG and lossy WebP quality. */
        const val QUALITY = 95
    }
}

// GLOSSARY
// - ExportSliceWriter: renders export regions and writes them as PNG, JPEG or WebP to content URIs.
// - Region: a region to write (name, box, format, scale).
// - regions(list): the visible slices, else the automatic content slice, else none.
// - write(engine, region, uri): render, encode and write one region.
// - onWhite(bitmap): flatten onto white for JPEG.
// - MAX_PIXELS: 100 000 000 pixels per image.
// - QUALITY: 95.
