package com.extemporecreare.app.components.engine

import android.content.Context
import android.net.Uri
import android.provider.DocumentsContract
import android.provider.OpenableColumns
import java.io.File
import java.io.FileInputStream
import java.io.FileOutputStream
import java.io.IOException

/**
 * Putting a finished file in place of a content URI's document, as safely as its storage provider
 * allows (`filestandard.md` §11): where the app may create and rename documents in the folder (a
 * folder the user granted), the file is written as a sibling and renamed over the original, so the
 * original stays whole until the new one is complete; elsewhere it is copied into the URI and
 * flushed to storage. Used by [EngineFiles] (`.etc`) and [EngineBoards] (`.etcr`).
 */
class UriReplace(private val context: Context) {

    /** Put [file] in place of the content at [target] → the URI it is at afterwards. */
    fun replace(file: File, target: Uri): Uri = replaceByRename(file, target) ?: target.also { copyInto(file, it) }

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
        val mime = resolver.getType(target) ?: "application/octet-stream"
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
    fun supports(uri: Uri, flags: Int): Boolean {
        context.contentResolver.query(uri, arrayOf(DocumentsContract.Document.COLUMN_FLAGS), null, null, null)?.use { c ->
            if (c.moveToFirst()) return c.getInt(0) and flags == flags
        }
        return false
    }


    /** Copy [file] over the content at [target] ("wt": truncate and write) and flush it to storage. */
    fun copyInto(file: File, target: Uri) {
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
    fun displayName(uri: Uri): String {
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
// - UriReplace: puts finished files in place of content URIs.
// - replace(file, target): put a file in place of a URI's content, by rename where possible → its URI afterwards.
// - replaceByRename(file, target): write a sibling document and rename it over the target; null where the provider can't.
// - supports(uri, flags): whether a document has the given DocumentsContract flags.
// - copyInto(file, target): copy a file over a URI's content and flush it to storage.
// - displayName(uri): the file name of a URI.
