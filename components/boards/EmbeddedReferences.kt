package com.extemporecreare.app.components.boards

import android.content.Context
import android.net.Uri
import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.EngineAnswer
import org.json.JSONObject
import java.io.File
import java.io.FileOutputStream

/**
 * Reference images stored inside the artwork `.etc` (the board's **Embed**, `filestandard.md`
 * §7.2): [store] puts an image's bytes in the open document ([engine], `reference.embed`), and
 * [extract] reads them back, from the open document when it has them, else from the `.etc` file.
 */
class EmbeddedReferences(private val context: Context, private val engine: EngineBridge) {

    /** The folder for temporary copies. */
    private val tempDir: File = File(context.cacheDir, "embedded-references").apply { mkdirs() }

    /** Store the image at [file] (named [name]) in the open document → `{"assetKey", "sha256"}`, or the error. */
    fun store(file: Uri, name: String): Result<JSONObject> {
        val temp = File.createTempFile("embed-", ".tmp", tempDir)
        return try {
            context.contentResolver.openInputStream(file)?.use { src -> FileOutputStream(temp).use { src.copyTo(it) } } ?: return Result.failure(Exception("can't read the image"))
            val answer = EngineAnswer(engine.execute("reference.embed", JSONObject().put("path", temp.path).put("originalFilename", name).toString()))
            answer.okObject?.let { Result.success(it) } ?: Result.failure(Exception(answer.error ?: "can't embed the image"))
        } catch (e: Exception) {
            Result.failure(e)
        } finally {
            temp.delete()
        }
    }

    /** Write the image stored under [assetKey] to [out]: from the open document, else from the `.etc` at [etc] → whether it was written. Reads files: call off the main thread. */
    fun extract(assetKey: String, etc: Uri?, out: File): Boolean {
        if (EngineAnswer(engine.execute("reference.asset", JSONObject().put("assetKey", assetKey).put("path", out.path).toString())).isOk) return true
        val from = etc ?: return false
        val temp = File.createTempFile("etc-", ".tmp", tempDir)
        return try {
            context.contentResolver.openInputStream(from)?.use { src -> FileOutputStream(temp).use { src.copyTo(it) } } ?: return false
            EngineAnswer(EngineBridge.boardEtcAsset(temp.path, assetKey, out.path)).isOk
        } catch (_: Exception) {
            false
        } finally {
            temp.delete()
        }
    }

    /** A temporary file for an extracted image. */
    fun tempFile(): File = File.createTempFile("picture-", ".tmp", tempDir)
}

// GLOSSARY
// - EmbeddedReferences(context, engine): reference images stored inside the artwork .etc.
// - tempDir: the cache folder for temporary copies.
// - store(file, name): store an image in the open document. extract(assetKey, etc, out): read one back.
// - tempFile(): a temporary file for an extracted image.
