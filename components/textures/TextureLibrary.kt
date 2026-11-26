package com.extemporecreare.app.components.textures

import android.content.Context
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import java.io.File

/**
 * The Texture library's presets (`uistandards.md` §7.4): the images in the repository folder
 * `tools/Libraries/textures`, packaged as the `textures` assets. A texture's name is its file
 * name without the extension.
 */
class TextureLibrary(private val context: Context) {

    /** One texture: its name and asset path. */
    data class Texture(val name: String, val asset: String)

    /** The textures, by name. */
    fun list(): List<Texture> = context.assets.list(FOLDER).orEmpty()
        .filter { it.substringAfterLast('.', "").lowercase() in EXTENSIONS }
        .sorted()
        .map { Texture(it.substringBeforeLast('.'), "$FOLDER/$it") }

    /** [texture]'s preview, scaled down to at most [PREVIEW] pixels a side, or null. Reads the asset: call off the main thread. */
    fun preview(texture: Texture): Bitmap? = runCatching {
        val bounds = BitmapFactory.Options().apply { inJustDecodeBounds = true }
        context.assets.open(texture.asset).use { BitmapFactory.decodeStream(it, null, bounds) }
        var sample = 1
        while (maxOf(bounds.outWidth, bounds.outHeight) / sample > PREVIEW * 2) sample *= 2
        context.assets.open(texture.asset).use { BitmapFactory.decodeStream(it, null, BitmapFactory.Options().apply { inSampleSize = sample }) }
    }.getOrNull()

    /** [texture]'s image copied to a cache file, for the engine to read. */
    fun file(texture: Texture): File {
        val dir = File(context.cacheDir, "textures").apply { mkdirs() }
        val out = File(dir, texture.asset.substringAfterLast('/'))
        context.assets.open(texture.asset).use { src -> out.outputStream().use { src.copyTo(it) } }
        return out
    }

    companion object {
        /** The assets folder the textures are packaged in. */
        const val FOLDER = "textures"

        /** The image file extensions read as textures. */
        val EXTENSIONS = setOf("png", "jpg", "jpeg", "webp", "gif", "bmp")

        /** A preview tile's size, in pixels. */
        const val PREVIEW = 96
    }
}

// GLOSSARY
// - TextureLibrary(context): the Texture library's presets from the textures assets.
// - Texture(name, asset): one texture. list(): the textures. preview(texture): its preview. file(texture): its image as a cache file.
// - FOLDER, EXTENSIONS, PREVIEW: the assets folder, the image extensions read, the preview size.
