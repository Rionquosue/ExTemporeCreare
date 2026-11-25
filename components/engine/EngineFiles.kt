package com.extemporecreare.app.components.engine

import android.content.Context
import android.net.Uri
import org.json.JSONObject
import java.io.File
import java.io.FileOutputStream
import java.io.IOException

/**
 * Opening and saving documents through Android content URIs (the Storage Access Framework).
 *
 * The engine never reads or writes a URI itself:
 * - Open: the URI's bytes are copied into a temporary file in the app's cache, which the engine
 *   opens with the URI as the document's path.
 * - Save: the engine writes each file into a temporary file (atomically). Each `.etc` is checked,
 *   then put in place of its URI ([UriReplace]: renamed over the original where the provider
 *   allows, else copied in and flushed). Only when every file is in place is the document marked
 *   saved.
 */
class EngineFiles(private val context: Context, private val engine: EngineBridge) {

    /** The folder for temporary copies: the app's cache, so the system may clear what is left. */
    private val tempDir: File = File(context.cacheDir, "engine-files").apply { mkdirs() }

    /** Puts saved files in place of their URIs. */
    private val uris = UriReplace(context)

    /** Open the document at [uri] → the engine's JSON answer. */
    fun open(uri: Uri): String {
        val temp = File.createTempFile("open-", ".tmp", tempDir)
        try {
            val input = context.contentResolver.openInputStream(uri) ?: throw IOException("can't read $uri")
            input.use { src -> FileOutputStream(temp).use { dst -> src.copyTo(dst) } }
            return engine.openCopied(temp.path, uris.displayName(uri), uri.toString())
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
        var finalUri = ""
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
                val target = Uri.parse(f.getString("target"))
                val placed = uris.replace(File(f.getString("temp")), target)
                if (target == uri && placed != target) finalUri = placed.toString()
            }
        } catch (e: Exception) {
            engine.saveFinish(false)
            return JSONObject().put("error", JSONObject().put("kind", "io").put("message", e.message ?: e.toString())).toString()
        } finally {
            temps.forEach { File(it.getString("temp")).delete() }
        }
        engine.saveFinish(true, finalUri)
        return written.toString()
    }
}

// GLOSSARY
// - EngineFiles: opens and saves documents through content URIs for an EngineBridge.
// - tempDir: the cache folder for temporary copies.
// - uris: the UriReplace that puts saved files in place.
// - open(uri): copy a URI into a temporary file and open it with the URI as the document's path.
// - save(uri, mode, options): write the save into temporary files, check each .etc, put them in place of the URIs, then mark saved.
