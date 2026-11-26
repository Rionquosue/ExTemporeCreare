package com.extemporecreare.app.components.documents

import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.net.Uri
import org.json.JSONArray
import org.json.JSONObject
import java.util.zip.ZipInputStream

/**
 * The recently saved or opened documents (`uistandards.md` §5.2): at most [MAX], newest first,
 * kept in the app's preferences with each file's name and when it was last used. Adding a file
 * keeps the app's permission to read and write it across restarts. A `.etc` file's own thumbnail
 * (`Thumbnails/thumbnail.png`, `filestandard.md`) is read from it for the splash.
 */
class RecentFiles(private val context: Context) {

    /** One recent file: its URI, display name and when it was last used (ms since the epoch). */
    data class Recent(val uri: Uri, val name: String, val usedAt: Long)

    private val prefs = context.getSharedPreferences(FILE, Context.MODE_PRIVATE)

    /** The recent files, newest first. */
    fun list(): List<Recent> {
        val arr = runCatching { JSONArray(prefs.getString(KEY, "[]")) }.getOrDefault(JSONArray())
        return (0 until arr.length()).mapNotNull { i ->
            val o = arr.optJSONObject(i) ?: return@mapNotNull null
            Recent(Uri.parse(o.optString("uri")), o.optString("name"), o.optLong("usedAt"))
        }
    }

    /** Put [uri] (named [name]) first, keeping the app's access to it; the oldest beyond [MAX] go. */
    fun add(uri: Uri, name: String) {
        runCatching {
            context.contentResolver.takePersistableUriPermission(uri, Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_GRANT_WRITE_URI_PERMISSION)
        }
        save(listOf(Recent(uri, name, System.currentTimeMillis())) + list().filter { it.uri != uri })
    }

    /** Take [uri] off the list. */
    fun remove(uri: Uri) = save(list().filter { it.uri != uri })

    /** Empty the list. */
    fun clear() = save(emptyList())

    /** Whether [uri] can still be read. */
    fun exists(uri: Uri): Boolean = runCatching { context.contentResolver.openInputStream(uri)?.use { true } ?: false }.getOrDefault(false)

    /** The thumbnail a `.etc` file at [uri] carries, or null. Reads the file; call off the main thread. */
    fun thumbnail(uri: Uri): Bitmap? = runCatching {
        context.contentResolver.openInputStream(uri)?.use { input ->
            ZipInputStream(input).use { zip ->
                generateSequence { zip.nextEntry }.firstOrNull { it.name == THUMBNAIL }?.let { BitmapFactory.decodeStream(zip) }
            }
        }
    }.getOrNull()

    private fun save(items: List<Recent>) {
        val arr = JSONArray()
        for (r in items.take(MAX)) arr.put(JSONObject().put("uri", r.uri.toString()).put("name", r.name).put("usedAt", r.usedAt))
        prefs.edit().putString(KEY, arr.toString()).apply()
    }

    companion object {
        /** The most recent files kept and shown. */
        const val MAX = 10

        /** The preferences file and key. */
        const val FILE = "recent-files"
        const val KEY = "recent"

        /** A `.etc` file's thumbnail entry. */
        const val THUMBNAIL = "Thumbnails/thumbnail.png"
    }
}

// GLOSSARY
// - RecentFiles: the recently used documents, newest first, with persisted access.
// - Recent: one recent file (uri, name, usedAt).
// - list(), add(uri, name), remove(uri), clear(): read and change the list.
// - exists(uri): whether a file can still be read.
// - thumbnail(uri): a .etc file's own thumbnail.
// - MAX: 10. FILE, KEY: "recent-files", "recent". THUMBNAIL: "Thumbnails/thumbnail.png".
