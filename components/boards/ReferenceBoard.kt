package com.extemporecreare.app.components.boards

import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.net.Uri
import com.extemporecreare.app.components.engine.EngineBoards
import com.extemporecreare.app.components.engine.UriReplace
import com.extemporecreare.app.components.tools.EngineAnswer
import org.json.JSONArray
import org.json.JSONObject
import java.security.MessageDigest

/**
 * The open reference board (`uistandards.md` §8.2, `filestandard.md` §7): its `board.json` (images
 * linked to their own files, never copied in; their placement; the board's own camera), the `.etcr`
 * file it came from, and which images can't be found. It lives apart from the artwork document:
 * collapsing the panel or switching documents keeps it. Changes are kept in memory until [save];
 * [checkpoint] keeps them, and the camera keyed by the board's id, in app-local recovery state
 * (`filestandard.md` §7.3), which [restore] and [open] bring back.
 */
class ReferenceBoard(private val context: Context) {

    private val files = EngineBoards(context)

    /** App-local recovery state: the current board, and each board's last camera not yet saved. */
    private val prefs = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)

    /** The board as JSON (`board.json`). */
    var board: JSONObject = newBoardJson()
        private set

    /** The `.etcr` file the board was opened from or saved to, or null. */
    var uri: Uri? = null
        private set

    /** The board file's name, or null when it has none. */
    var fileName: String? = null
        private set

    /** Whether the board has changes not saved. */
    var modified = false
        private set

    /** Each image's id → its file's URI when found, else absent (missing). */
    private val found = mutableMapOf<String, Uri>()

    /** Decoded pictures by id (see [decode]), null when the file couldn't be read. Main thread only. */
    val pictures = mutableMapOf<String, Bitmap?>()

    /** The images, bottom first. */
    fun images(): List<JSONObject> = board.optJSONArray("images")?.let { a -> (0 until a.length()).mapNotNull { a.optJSONObject(it) } } ?: emptyList()

    /** Whether image [id]'s file is missing. */
    fun isMissing(id: String): Boolean = id !in found

    /** The board's camera: centre and zoom. */
    fun camera(): Triple<Double, Double, Double> {
        val v = board.optJSONObject("viewport")
        val c = v?.optJSONArray("center")
        return Triple(c?.optDouble(0) ?: 0.0, c?.optDouble(1) ?: 0.0, v?.optDouble("zoom", 1.0) ?: 1.0)
    }

    /** Set the board's camera (saved with the board, apart from the artwork's). */
    fun setCamera(x: Double, y: Double, zoom: Double) {
        board.put("viewport", JSONObject().put("center", JSONArray().put(x).put(y)).put("zoom", zoom.coerceIn(0.01, 64.0)))
        modified = true
    }

    /** A new empty board in place of this one. */
    fun newBoard() {
        board = newBoardJson()
        uri = null
        fileName = null
        modified = false
        found.clear()
        pictures.clear()
        checkpoint()
    }

    /** Keep the board as it is (its file, unsaved changes, camera) in app-local recovery state. */
    fun checkpoint() {
        val e = prefs.edit().putString(CURRENT, JSONObject().put("uri", uri?.toString() ?: "").put("modified", modified).put("board", board).toString())
        val camera = board.optJSONObject("viewport")
        if (modified && camera != null) e.putString(CAMERA + board.optString("boardId"), camera.toString())
        e.apply()
    }

    /** Bring back the board kept by [checkpoint], its links checked again; false when none was kept. */
    fun restore(): Boolean {
        val kept = prefs.getString(CURRENT, null)?.let { runCatching { JSONObject(it) }.getOrNull() } ?: return false
        board = kept.optJSONObject("board") ?: return false
        uri = kept.optString("uri").takeIf { it.isNotEmpty() }?.let { Uri.parse(it) }
        fileName = uri?.let { nameOf(it) }
        modified = kept.optBoolean("modified")
        pictures.clear()
        readLinks(files.links(uri ?: Uri.EMPTY, board))
        return true
    }

    /** Saving while going to the background (§7.3): a board with a file and unsaved changes is saved to it → an error message, or null. */
    fun saveIfChanged(): String? {
        checkpoint()
        val file = uri ?: return null
        return if (modified) save(file) else null
    }

    /** Put back a camera checkpointed for this board and not saved since. */
    private fun recoverCamera() {
        val camera = prefs.getString(CAMERA + board.optString("boardId"), null)?.let { runCatching { JSONObject(it) }.getOrNull() } ?: return
        board.put("viewport", camera)
        modified = true
    }

    /** Open the `.etcr` at [file] → an error message, or null. */
    fun open(file: Uri): String? {
        val answer = EngineAnswer(files.open(file))
        val ok = answer.okObject ?: return answer.error ?: "can't open the board"
        board = ok.getJSONObject("board")
        uri = file
        fileName = nameOf(file)
        modified = false
        pictures.clear()
        readLinks(ok.optJSONArray("links"))
        recoverCamera()
        checkpoint()
        return null
    }

    /** Save the board to [file] (its own, or a new one) → an error message, or null. */
    fun save(file: Uri): String? {
        val answer = EngineAnswer(files.save(file, board))
        val ok = answer.okObject ?: return answer.error ?: "can't save the board"
        uri = Uri.parse(ok.optString("uri", file.toString()))
        fileName = nameOf(uri!!)
        modified = false
        prefs.edit().remove(CAMERA + board.optString("boardId")).apply()
        checkpoint()
        return null
    }

    /** Add the image at [file], linked, centred on the camera → an error message, or null. */
    fun addImage(file: Uri): String? {
        runCatching { context.contentResolver.takePersistableUriPermission(file, Intent.FLAG_GRANT_READ_URI_PERMISSION) }
        val bounds = BitmapFactory.Options().apply { inJustDecodeBounds = true }
        context.contentResolver.openInputStream(file)?.use { BitmapFactory.decodeStream(it, null, bounds) } ?: return "can't read the image"
        if (bounds.outWidth <= 0 || bounds.outHeight <= 0) return "that file isn't an image the board can show"
        val (cx, cy, zoom) = camera()
        val scale = PLACED_SIZE / zoom / maxOf(bounds.outWidth, bounds.outHeight)
        val w = bounds.outWidth * scale
        val h = bounds.outHeight * scale
        val id = "ref-" + java.util.UUID.randomUUID().toString().take(8)
        val source = JSONObject().put("kind", "linked-file").put("uri", file.toString()).put("originalFilename", nameOf(file) ?: file.lastPathSegment.orEmpty())
            .put("width", bounds.outWidth).put("height", bounds.outHeight)
        sha256(file)?.let { source.put("sha256", it) }
        val image = JSONObject().put("id", id).put("source", source).put("visible", true)
            .put("transform", JSONObject().put("x", cx - w / 2).put("y", cy - h / 2).put("width", w).put("height", h).put("rotationDegrees", 0.0).put("pivot", JSONArray().put(0.5).put(0.5)))
        board.optJSONArray("images")?.put(image) ?: board.put("images", JSONArray().put(image))
        found[id] = file
        modified = true
        return null
    }

    /** Move image [id] by ([dx], [dy]) board units. */
    fun move(id: String, dx: Double, dy: Double) {
        val t = image(id)?.optJSONObject("transform") ?: return
        t.put("x", t.optDouble("x") + dx).put("y", t.optDouble("y") + dy)
        modified = true
    }

    /** Scale image [id] by [factor] about its centre. */
    fun scale(id: String, factor: Double) {
        val t = image(id)?.optJSONObject("transform") ?: return
        val (w, h) = t.optDouble("width") to t.optDouble("height")
        val (nw, nh) = (w * factor).coerceAtLeast(1.0) to (h * factor).coerceAtLeast(1.0)
        t.put("x", t.optDouble("x") - (nw - w) / 2).put("y", t.optDouble("y") - (nh - h) / 2).put("width", nw).put("height", nh)
        modified = true
    }

    /** Bring image [id] to the front. */
    fun toFront(id: String) {
        val all = images()
        val it = all.firstOrNull { i -> i.optString("id") == id } ?: return
        board.put("images", JSONArray(all.filter { i -> i.optString("id") != id } + it))
        modified = true
    }

    /** Take image [id] off the board (its file stays as it is). */
    fun remove(id: String) {
        board.put("images", JSONArray(images().filter { it.optString("id") != id }))
        found.remove(id)
        pictures.remove(id)
        modified = true
    }

    /** Link image [id] to [file], keeping its placement → an error message, or null. */
    fun relink(id: String, file: Uri): String? {
        runCatching { context.contentResolver.takePersistableUriPermission(file, Intent.FLAG_GRANT_READ_URI_PERMISSION) }
        val updated = files.relink(uri ?: file, board, id, file) ?: return "can't read that file"
        board = updated
        found[id] = file
        pictures.remove(id)
        modified = true
        return null
    }

    /** Image [id]'s file when found, else null (missing). */
    fun fileOf(id: String): Uri? = found[id]

    /** [file]'s display name, or null when it can't be read. */
    private fun nameOf(file: Uri): String? = runCatching { UriReplace(context).displayName(file) }.getOrNull()

    private fun image(id: String): JSONObject? = images().firstOrNull { it.optString("id") == id }

    /** Which images were found, from a link check `[{"id", "status", "uri"?}]`. */
    private fun readLinks(links: JSONArray?) {
        found.clear()
        for (i in 0 until (links?.length() ?: 0)) {
            val l = links?.optJSONObject(i) ?: continue
            if (l.optString("status") == "found") l.optString("uri").takeIf { it.isNotEmpty() }?.let { found[l.optString("id")] = Uri.parse(it) }
        }
    }

    private fun newBoardJson(): JSONObject = EngineAnswer(files.newBoard("Reference board")).okObject ?: JSONObject().put("images", JSONArray())

    private fun sha256(file: Uri): String? = runCatching {
        val digest = MessageDigest.getInstance("SHA-256")
        context.contentResolver.openInputStream(file)?.use { input ->
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
        /** The recovery state's preferences, and its keys: the current board, and a board's camera by id. */
        const val PREFS = "reference-board"
        const val CURRENT = "current"
        const val CAMERA = "camera:"

        /** The picture at [file], scaled down to at most [MAX_SIDE] pixels a side, or null. Reads the file: call off the main thread. */
        fun decode(context: Context, file: Uri): Bitmap? = runCatching {
            val bounds = BitmapFactory.Options().apply { inJustDecodeBounds = true }
            context.contentResolver.openInputStream(file)?.use { BitmapFactory.decodeStream(it, null, bounds) }
            var sample = 1
            while (maxOf(bounds.outWidth, bounds.outHeight) / sample > MAX_SIDE) sample *= 2
            context.contentResolver.openInputStream(file)?.use { BitmapFactory.decodeStream(it, null, BitmapFactory.Options().apply { inSampleSize = sample }) }
        }.getOrNull()

        /** The largest side a board picture is decoded at, in pixels. */
        const val MAX_SIDE = 1024

        /** How large a newly added image is placed, in screen pixels at the board's zoom. */
        const val PLACED_SIZE = 280.0
    }
}

// GLOSSARY
// - ReferenceBoard: the open .etcr board: its JSON, file, found images and decoded pictures.
// - board, uri, fileName, modified: the board JSON, its file and the file's name, unsaved changes.
// - found, pictures: images found (id → URI) and decoded pictures.
// - images(), isMissing(id), camera(), setCamera(x, y, zoom): read it.
// - prefs, PREFS, CURRENT, CAMERA: app-local recovery state and its keys.
// - newBoard(), open(file), save(file), saveIfChanged(): the board's file.
// - checkpoint(), restore(), recoverCamera(): recovery of unsaved board state and camera.
// - addImage(file), move(id, dx, dy), scale(id, factor), toFront(id), remove(id), relink(id, file): edit it.
// - fileOf(id): an image's file when found. decode(context, file): a scaled-down picture. image(id): an image's JSON. readLinks(links): which were found.
// - nameOf(file), newBoardJson(), sha256(file): helpers. MAX_SIDE: 1024 px. PLACED_SIZE: 280 px.
