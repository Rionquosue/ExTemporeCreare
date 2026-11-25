package com.extemporecreare.app.components.engine

import android.content.Context
import android.net.Uri
import android.provider.OpenableColumns
import org.json.JSONObject
import java.io.File
import java.io.FileInputStream
import java.io.FileOutputStream
import java.io.IOException

/**
 * Opening and saving documents through Android content URIs (the Storage Access Framework).
 *
 * The engine never reads or writes a URI itself:
 * - Open: the URI's bytes are copied into a temporary file in the app's cache, which the engine
 *   opens with the URI as the document's path.
 * - Save: the engine writes each file into a temporary file (atomically). Each `.etc` is checked,
 *   then copied into its URI and flushed to storage; only when every copy succeeded is the
 *   document marked saved. A failed save leaves the files at the URIs as they were whenever the
 *   failure comes before their copy starts (validation, or the engine's own write).
 */
class EngineFiles(private val context: Context, private val engine: EngineBridge) {

    /** The folder for temporary copies: the app's cache, so the system may clear what is left. */
    private val tempDir: File = File(context.cacheDir, "engine-files").apply { mkdirs() }

    /** Open the document at [uri] → the engine's JSON answer. */
    fun open(uri: Uri): String {
        val temp = File.createTempFile("open-", ".tmp", tempDir)
        try {
            val input = context.contentResolver.openInputStream(uri) ?: throw IOException("can't read $uri")
            input.use { src -> FileOutputStream(temp).use { dst -> src.copyTo(dst) } }
            return engine.openCopied(temp.path, displayName(uri), uri.toString())
        } finally {
            temp.delete()
        }
    }

    /**
     * Save the active document to [uri] with [mode] (`save`, `saveAs`, `copy` or `template`) and
     * [options] (a JSON object of save options; `format` defaults to `etc`) → the engine's JSON
     * answer, or an error answer when a file couldn't be checked or copied.
     */
    fun save(uri: Uri, mode: String = "saveAs", options: JSONObject = JSONObject()): String {
        val params = JSONObject(options.toString()).put("path", uri.toString())
        if (!params.has("format")) params.put("format", "etc")
        val written = JSONObject(engine.saveToTemp(mode, params.toString(), tempDir.path))
        val ok = written.optJSONObject("ok") ?: return written.toString()
        val files = ok.optJSONArray("files")
        val temps = (0 until (files?.length() ?: 0)).map { files!!.getJSONObject(it) }
        try {
            // Every .etc is checked before any target is touched.
            for (f in temps) {
                val temp = f.getString("temp")
                if (f.getString("target") == uri.toString() && params.optString("format") == "etc") {
                    val check = JSONObject(EngineBridge.validate(temp))
                    if (!check.has("ok")) {
                        engine.saveFinish(false)
                        return check.toString()
                    }
                }
            }
            for (f in temps) {
                copyInto(File(f.getString("temp")), Uri.parse(f.getString("target")))
            }
        } catch (e: Exception) {
            engine.saveFinish(false)
            return JSONObject().put("error", JSONObject().put("kind", "io").put("message", e.message ?: e.toString())).toString()
        } finally {
            temps.forEach { File(it.getString("temp")).delete() }
        }
        engine.saveFinish(true)
        return written.toString()
    }

    /** Copy [file] over the content at [target] ("wt": truncate and write) and flush it to storage. */
    private fun copyInto(file: File, target: Uri) {
        val pfd = context.contentResolver.openFileDescriptor(target, "wt") ?: throw IOException("can't write $target")
        pfd.use {
            FileOutputStream(it.fileDescriptor).use { out ->
                FileInputStream(file).use { src -> src.copyTo(out) }
                out.flush()
                out.fd.sync()
            }
        }
    }

    /** The display name of [uri] (its file name), else its last path segment. */
    private fun displayName(uri: Uri): String {
        context.contentResolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)?.use { c ->
            if (c.moveToFirst()) {
                val i = c.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                if (i >= 0) c.getString(i)?.let { return it }
            }
        }
        return uri.lastPathSegment ?: "Untitled.etc"
    }
}

// GLOSSARY
// - EngineFiles: opens and saves documents through content URIs for an EngineBridge.
// - tempDir: the cache folder for temporary copies.
// - open(uri): copy a URI into a temporary file and open it with the URI as the document's path.
// - save(uri, mode, options): write the save into temporary files, check each .etc, copy into the URIs, then mark saved.
// - copyInto(file, target): copy a file over a URI's content and flush it to storage.
// - displayName(uri): the file name of a URI.
