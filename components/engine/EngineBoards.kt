package com.extemporecreare.app.components.engine

import android.content.Context
import android.net.Uri
import android.provider.DocumentsContract
import org.json.JSONArray
import org.json.JSONObject
import java.io.File
import java.io.FileOutputStream
import java.io.IOException
import java.security.MessageDigest

/**
 * Reference boards (`.etcr`, `filestandard.md` §7) through Android content URIs. A board is edited
 * as JSON (`board.json`); its images stay linked to their own files, never copied in (§7.2).
 *
 * - Open: the board's bytes are copied to a temporary file and read by the engine bridge.
 * - Save: the board is written to a temporary file, read back to check it, then put in place of
 *   its URI ([UriReplace]).
 * - Embedded images (`kind: "embedded"`, Embed on the board) are stored in the `.etcr` itself; the
 *   app keeps their bytes in [embeddedDir]`/<boardId>/` while the board is open: Open puts them
 *   there, Save packs them from there.
 * - Links: a source is found by its URI when the app can read it, else by its relative path from
 *   the board's folder when the board is in a folder the user granted; an embedded one in
 *   [embeddedDir]. Relinking records the
 *   picked file's URI, name, SHA-256 and, inside the board's folder, its relative
 *   path; the image's placement and crop are kept.
 */
class EngineBoards(private val context: Context) {

    /** The folder for temporary copies. */
    private val tempDir: File = File(context.cacheDir, "engine-boards").apply { mkdirs() }

    /** Where open boards' embedded images are kept: one folder per board id. */
    val embeddedDir: File = File(context.filesDir, "board-embedded").apply { mkdirs() }

    /** Puts saved boards in place of their URIs. */
    private val uris = UriReplace(context)

    /** A new empty board titled [title] → `{"ok": board}`. */
    fun newBoard(title: String): String = EngineBridge.boardNew(title)

    /** Read the board at [uri] → `{"ok": {"board", "links"}}`, its links checked as [links] does. */
    fun open(uri: Uri): String {
        val temp = File.createTempFile("board-", ".tmp", tempDir)
        try {
            val input = context.contentResolver.openInputStream(uri) ?: throw IOException("can't read $uri")
            input.use { src -> FileOutputStream(temp).use { dst -> src.copyTo(dst) } }
            val read = JSONObject(EngineBridge.boardRead(temp.path, "", embeddedDir.path))
            val ok = read.optJSONObject("ok") ?: return read.toString()
            ok.put("links", links(uri, ok.getJSONObject("board")))
            return read.toString()
        } finally {
            temp.delete()
        }
    }

    /** Write [board] to [uri] (with the PNG [thumbnail] when given) → `{"ok": {"uri"}}` or the error. */
    fun save(uri: Uri, board: JSONObject, thumbnail: File? = null): String {
        val temp = File.createTempFile("board-", ".tmp", tempDir)
        try {
            val written = JSONObject(EngineBridge.boardWrite(temp.path, board.toString(), thumbnail?.path ?: "", embeddedDir.path))
            if (!written.has("ok")) return written.toString()
            val check = JSONObject(EngineBridge.boardRead(temp.path, ""))
            if (!check.has("ok")) return check.toString()
            val placed = uris.replace(temp, uri)
            return JSONObject().put("ok", JSONObject().put("uri", placed.toString())).toString()
        } catch (e: Exception) {
            return JSONObject().put("error", JSONObject().put("kind", "io").put("message", e.message ?: e.toString())).toString()
        } finally {
            temp.delete()
        }
    }

    /** Each image's link status for the board at [boardUri]: `[{"id", "status": "found"|"missing", "uri"?}]`. */
    fun links(boardUri: Uri, board: JSONObject): JSONArray {
        val out = JSONArray()
        val images = board.optJSONArray("images") ?: JSONArray()
        for (i in 0 until images.length()) {
            val image = images.getJSONObject(i)
            val found = embeddedFile(board, image.getJSONObject("source"))?.takeIf { it.isFile }?.let { Uri.fromFile(it) }
                ?: locate(boardUri, image.getJSONObject("source"))
            val row = JSONObject().put("id", image.getString("id")).put("status", if (found != null) "found" else "missing")
            if (found != null) row.put("uri", found.toString())
            out.put(row)
        }
        return out
    }

    /** The folder [board]'s embedded images are kept in (as the bridge names it: the board id's letters, digits and dashes). */
    fun embeddedFolder(board: JSONObject): File {
        val id = board.optString("boardId").filter { it.isLetterOrDigit() && it.code < 128 || it == '-' }
        return File(embeddedDir, id.ifEmpty { "board" })
    }

    /** The local file of an embedded [source] of [board], or null when it isn't embedded. */
    fun embeddedFile(board: JSONObject, source: JSONObject): File? {
        if (source.optString("kind") != EMBEDDED_KIND) return null
        val name = source.optString("assetPath").substringAfterLast('/').takeIf { it.isNotEmpty() } ?: return null
        return File(embeddedFolder(board), name)
    }

    /** Where [source]'s file is: its URI when readable, else its relative path from the board's folder. */
    private fun locate(boardUri: Uri, source: JSONObject): Uri? {
        source.optString("uri").takeIf { it.isNotEmpty() }?.let { Uri.parse(it) }?.takeIf { readable(it) }?.let { return it }
        val rel = source.optString("relativePath").takeIf { it.isNotEmpty() } ?: return null
        return resolveRelative(boardUri, rel)?.takeIf { readable(it) }
    }

    /** Whether the app can read [uri]. */
    private fun readable(uri: Uri): Boolean =
        runCatching { context.contentResolver.openFileDescriptor(uri, "r")?.use { true } ?: false }.getOrDefault(false)

    /** The document at [relative] (`a/b.png`) from the folder of [boardUri], when that is in a granted folder. */
    private fun resolveRelative(boardUri: Uri, relative: String): Uri? {
        if (!DocumentsContract.isTreeUri(boardUri)) return null
        val ids = runCatching { DocumentsContract.findDocumentPath(context.contentResolver, boardUri) }.getOrNull()?.path ?: return null
        if (ids.size < 2) return null
        var dir = ids[ids.size - 2]
        val parts = relative.split('/').filter { it.isNotEmpty() && it != "." }
        for ((n, part) in parts.withIndex()) {
            if (part == "..") return null
            val child = childNamed(boardUri, dir, part) ?: return null
            if (n == parts.size - 1) return DocumentsContract.buildDocumentUriUsingTree(boardUri, child)
            dir = child
        }
        return null
    }

    /** The document id of the child of folder [dirId] named [name], in the tree of [treeUri]. */
    private fun childNamed(treeUri: Uri, dirId: String, name: String): String? {
        val children = DocumentsContract.buildChildDocumentsUriUsingTree(treeUri, dirId)
        val cols = arrayOf(DocumentsContract.Document.COLUMN_DOCUMENT_ID, DocumentsContract.Document.COLUMN_DISPLAY_NAME)
        context.contentResolver.query(children, cols, null, null, null)?.use { c ->
            while (c.moveToNext()) {
                if (c.getString(1) == name) return c.getString(0)
            }
        }
        return null
    }

    /**
     * [board] with image [id] linked to the file at [file]: its URI, file name and SHA-256, and its
     * path relative to the board's folder when both are in the same granted folder. The image's
     * placement and crop are kept (§7.2). → the board, or `null` when there is no such image or the
     * file can't be read.
     */
    fun relink(boardUri: Uri, board: JSONObject, id: String, file: Uri): JSONObject? {
        val images = board.optJSONArray("images") ?: return null
        val image = (0 until images.length()).map { images.getJSONObject(it) }.firstOrNull { it.optString("id") == id } ?: return null
        val hash = sha256(file) ?: return null
        val source = image.optJSONObject("source") ?: JSONObject().also { image.put("source", it) }
        source.put("kind", "linked-file")
        source.remove("assetPath")
        source.put("uri", file.toString())
        source.put("originalFilename", uris.displayName(file))
        source.put("sha256", hash)
        val rel = relativeTo(boardUri, file)
        if (rel != null) source.put("relativePath", rel) else source.remove("relativePath")
        return board
    }

    /** [file]'s path from the folder of [boardUri] when both are in the same granted folder tree. */
    private fun relativeTo(boardUri: Uri, file: Uri): String? {
        if (!DocumentsContract.isTreeUri(boardUri) || !DocumentsContract.isTreeUri(file)) return null
        val resolver = context.contentResolver
        val board = runCatching { DocumentsContract.findDocumentPath(resolver, boardUri) }.getOrNull()?.path ?: return null
        val target = runCatching { DocumentsContract.findDocumentPath(resolver, file) }.getOrNull()?.path ?: return null
        val folder = board.dropLast(1)
        if (target.size <= folder.size || target.subList(0, folder.size) != folder) return null
        val names = mutableListOf<String>()
        for (docId in target.drop(folder.size)) {
            names.add(nameOf(file, docId) ?: return null)
        }
        return names.joinToString("/")
    }

    /** The display name of document [docId] in the tree of [treeUri]. */
    private fun nameOf(treeUri: Uri, docId: String): String? =
        uris.displayName(DocumentsContract.buildDocumentUriUsingTree(treeUri, docId)).takeIf { it.isNotEmpty() }

    /**
     * For each missing image of [board] (at [boardUri]), a document in the granted folders [folders]
     * (and their subfolders, at most 6 deep) with its recorded SHA-256, else its original file name →
     * `[{"id", "uri"}]`. Nothing is changed (see [relink]).
     */
    fun findMissing(boardUri: Uri, board: JSONObject, folders: List<Uri>): JSONArray {
        val out = JSONArray()
        val images = board.optJSONArray("images") ?: return out
        val missing = (0 until images.length()).map { images.getJSONObject(it) }.filter { locate(boardUri, it.getJSONObject("source")) == null }
        if (missing.isEmpty()) return out
        val files = mutableListOf<Pair<String, Uri>>()
        for (f in folders) {
            val rootId = runCatching { DocumentsContract.getTreeDocumentId(f) }.getOrNull() ?: continue
            collect(f, rootId, 0, files)
        }
        for (image in missing) {
            val source = image.getJSONObject("source")
            val wanted = source.optString("sha256")
            val name = source.optString("originalFilename")
            val match = files.firstOrNull { wanted.isNotEmpty() && sha256(it.second).equals(wanted, ignoreCase = true) }
                ?: files.firstOrNull { name.isNotEmpty() && it.first == name }
            if (match != null) out.put(JSONObject().put("id", image.getString("id")).put("uri", match.second.toString()))
        }
        return out
    }

    /** The files under folder [dirId] of tree [treeUri] as (name, URI), at most 6 folders deep and 20 000 files. */
    private fun collect(treeUri: Uri, dirId: String, depth: Int, out: MutableList<Pair<String, Uri>>) {
        if (depth > 6 || out.size >= 20_000) return
        val children = DocumentsContract.buildChildDocumentsUriUsingTree(treeUri, dirId)
        val cols = arrayOf(
            DocumentsContract.Document.COLUMN_DOCUMENT_ID,
            DocumentsContract.Document.COLUMN_DISPLAY_NAME,
            DocumentsContract.Document.COLUMN_MIME_TYPE,
        )
        context.contentResolver.query(children, cols, null, null, null)?.use { c ->
            while (c.moveToNext() && out.size < 20_000) {
                val id = c.getString(0)
                if (c.getString(2) == DocumentsContract.Document.MIME_TYPE_DIR) {
                    collect(treeUri, id, depth + 1, out)
                } else {
                    out.add(c.getString(1) to DocumentsContract.buildDocumentUriUsingTree(treeUri, id))
                }
            }
        }
    }

    /** Lowercase hex SHA-256 of the content at [uri], or `null` when it can't be read. */
    private fun sha256(uri: Uri): String? = runCatching {
        val digest = MessageDigest.getInstance("SHA-256")
        context.contentResolver.openInputStream(uri)?.use { input ->
            val buf = ByteArray(64 * 1024)
            while (true) {
                val n = input.read(buf)
                if (n < 0) break
                digest.update(buf, 0, n)
            }
        } ?: return null
        digest.digest().joinToString("") { "%02x".format(it) }
    }.getOrNull()

    companion object {
        /** The source kind of an image embedded in the `.etcr` itself (`filestandard.md` §7.2). */
        const val EMBEDDED_KIND = "embedded"

        /** The archive folder of embedded images. */
        const val ASSET_FOLDER = "References/"
    }
}

// GLOSSARY
// - EngineBoards: reference boards (.etcr) through content URIs.
// - tempDir: the cache folder for temporary copies. embeddedDir: where open boards' embedded images are kept.
// - embeddedFolder(board), embeddedFile(board, source): a board's embedded images folder and an embedded image's file.
// - EMBEDDED_KIND, ASSET_FOLDER: the embedded source kind and its archive folder.
// - uris: the UriReplace that puts saved boards in place.
// - newBoard(title): a new empty board.
// - open(uri): read a board and check its links.
// - save(uri, board, thumbnail): write a board, check it, put it in place of its URI.
// - links(boardUri, board): each image's link status.
// - locate(boardUri, source): where a source's file is (its URI, else its relative path).
// - readable(uri): whether the app can read a URI.
// - resolveRelative(boardUri, relative): the document at a relative path from the board's folder.
// - childNamed(treeUri, dirId, name): a folder's child by name.
// - relink(boardUri, board, id, file): link an image to another file, keeping its placement.
// - relativeTo(boardUri, file): a file's path from the board's folder, in the same granted tree.
// - nameOf(treeUri, docId): a document's display name.
// - findMissing(boardUri, board, folders): documents in granted folders that are the board's missing images.
// - collect(treeUri, dirId, depth, out): the files under a granted folder, within limits.
// - sha256(uri): lowercase hex SHA-256 of a URI's content.
