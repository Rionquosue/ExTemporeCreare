package com.extemporecreare.app.components.engine

import android.content.Context
import android.net.Uri
import android.provider.DocumentsContract
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
 *   then put in place of its URI ([replace]): where the storage provider lets the app create a
 *   sibling document and rename documents (a folder the user granted), the new file is written
 *   beside the old one and renamed over it, so the old file stays whole until the new one is
 *   complete; elsewhere it is copied into the URI and flushed to storage. Only when every file is
 *   in place is the document marked saved.
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
                val placed = replace(File(f.getString("temp")), target)
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

    /** Put [file] in place of the content at [target] → the URI it is at afterwards. */
    private fun replace(file: File, target: Uri): Uri = replaceByRename(file, target) ?: target.also { copyInto(file, it) }

    /**
     * Write [file] into a new sibling of [target] and rename it over [target]: the old document is
     * renamed aside, the new one takes its name, then the old one is deleted. If renaming the new
     * one fails, the old one gets its name back. → the new document's URI, or `null` where the
     * provider can't do this (no granted folder, no rename or create), and nothing was changed.
     */
    private fun replaceByRename(file: File, target: Uri): Uri? {
        val resolver = context.contentResolver
        if (!DocumentsContract.isDocumentUri(context, target) || !DocumentsContract.isTreeUri(target)) return null
        if (!supports(target, DocumentsContract.Document.FLAG_SUPPORTS_RENAME or DocumentsContract.Document.FLAG_SUPPORTS_DELETE)) return null
        val path = runCatching { DocumentsContract.findDocumentPath(resolver, target) }.getOrNull() ?: return null
        val ids = path.path
        if (ids.size < 2) return null
        val parent = DocumentsContract.buildDocumentUriUsingTree(target, ids[ids.size - 2])
        if (!supports(parent, DocumentsContract.Document.FLAG_DIR_SUPPORTS_CREATE)) return null
        val name = displayName(target)
        val mime = resolver.getType(target) ?: ETC_MIME
        val fresh = runCatching { DocumentsContract.createDocument(resolver, parent, mime, "$name.saving") }.getOrNull() ?: return null
        try {
            copyInto(file, fresh)
        } catch (e: Exception) {
            runCatching { DocumentsContract.deleteDocument(resolver, fresh) }
            throw e
        }
        val aside = DocumentsContract.renameDocument(resolver, target, "$name.old") ?: target
        val placed = try {
            DocumentsContract.renameDocument(resolver, fresh, name) ?: fresh
        } catch (e: Exception) {
            runCatching { DocumentsContract.renameDocument(resolver, aside, name) }
            runCatching { DocumentsContract.deleteDocument(resolver, fresh) }
            throw e
        }
        runCatching { DocumentsContract.deleteDocument(resolver, aside) }
        return placed
    }

    /** Whether document [uri] has every flag in [flags] (`Document.FLAG_*`). */
    private fun supports(uri: Uri, flags: Int): Boolean {
        context.contentResolver.query(uri, arrayOf(DocumentsContract.Document.COLUMN_FLAGS), null, null, null)?.use { c ->
            if (c.moveToFirst()) return c.getInt(0) and flags == flags
        }
        return false
    }

    companion object {
        /** The provisional MIME type of `.etc` files (`filestandard.md` §3). */
        const val ETC_MIME = "application/x-ex-tempore-creare"
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
// - replace(file, target): put a file in place of a URI's content, by rename where possible → its URI afterwards.
// - replaceByRename(file, target): write a sibling document and rename it over the target; null where the provider can't.
// - supports(uri, flags): whether a document has the given DocumentsContract flags.
// - ETC_MIME: the provisional MIME type of .etc files.
// - copyInto(file, target): copy a file over a URI's content and flush it to storage.
// - displayName(uri): the file name of a URI.
