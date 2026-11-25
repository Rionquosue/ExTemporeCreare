package com.extemporecreare.app.components.engine

import java.nio.ByteBuffer
import java.nio.ByteOrder

/**
 * The interface layer to the engine: one engine session, reached through the native library
 * `libetcbridge.so` (`engine/crates/bridge`).
 *
 * Commands, pointer events and tool selection cross as JSON text. Every answer is
 * `{"ok": <result>}` or `{"error": {"kind", "message"}}`; parsing it is the caller's job.
 * Close the bridge when the editor is done with it.
 */
class EngineBridge : AutoCloseable {

    /** The engine session's handle; 0 once closed. */
    private var handle: Long = nativeCreate()

    /** Whether [close] has run. */
    val isClosed: Boolean get() = handle == 0L

    /** The open session's handle, or an error once closed. */
    private fun live(): Long {
        check(handle != 0L) { "EngineBridge is closed" }
        return handle
    }

    /** Run engine command [command] with [paramsJson] (a JSON object) → the JSON answer. */
    fun execute(command: String, paramsJson: String = "{}"): String =
        nativeExecute(live(), command, paramsJson)

    /**
     * Feed a pointer event to the active tool → the JSON answer, whose `ok` lists the tool's
     * requests to the UI. [eventJson] is `{"kind": "down"|"drag"|"up"|"move", "pos": {"x", "y"},
     * "pressure"?, "mods"?}` in document points; [viewJson] is the view state (`zoom`, `outline`,
     * `smartGuides`, `snapToGrid`…; missing fields keep the engine's defaults).
     */
    fun pointer(eventJson: String, viewJson: String = "{}"): String =
        nativePointer(live(), eventJson, viewJson)

    /** Make tool [id] the active tool → the JSON answer. */
    fun selectTool(id: String, viewJson: String = "{}"): String =
        nativeSelectTool(live(), id, viewJson)

    /**
     * Render the active document's region at ([x], [y]) document points, [width] × [height] pixels at
     * [scale] pixels per point, into [out] as premultiplied RGBA (what an `ARGB_8888` bitmap's
     * `copyPixelsFromBuffer` reads) → whether [out] was filled. [out] must be a direct buffer of at
     * least `width × height × 4` bytes ([renderBuffer] makes one).
     */
    fun render(x: Double, y: Double, width: Int, height: Int, scale: Double, out: ByteBuffer): Boolean {
        require(out.isDirect) { "the render buffer must be direct" }
        return nativeRender(live(), x, y, width, height, scale, out)
    }

    /**
     * Open the file copied to [tempPath] (its display name [name] gives the title and type) as a
     * document whose path is [uri] → the JSON answer. [EngineFiles] does the copying.
     */
    fun openCopied(tempPath: String, name: String, uri: String): String =
        nativeOpen(live(), tempPath, name, uri)

    /**
     * Write a save of the active document into temporary files in [tempDir] and keep it pending →
     * the JSON answer, whose `ok.files` lists `{"target", "temp"}` for each file. [mode] is `save`,
     * `saveAs`, `copy` or `template`; [paramsJson] holds `path` (the target URI) and the save options.
     * End it with [saveFinish].
     */
    fun saveToTemp(mode: String, paramsJson: String, tempDir: String): String =
        nativeSave(live(), mode, paramsJson, tempDir)

    /** End the pending save: [ok] when every file reached its URI → the JSON answer. */
    fun saveFinish(ok: Boolean): String = nativeSaveFinish(live(), ok)

    /** Close the engine session. Calling it again does nothing. */
    override fun close() {
        if (handle != 0L) {
            nativeDestroy(handle)
            handle = 0L
        }
    }

    companion object {
        init {
            System.loadLibrary("etcbridge")
        }

        /** A direct buffer for [render] of [width] × [height] pixels. */
        fun renderBuffer(width: Int, height: Int): ByteBuffer =
            ByteBuffer.allocateDirect(width * height * 4).order(ByteOrder.nativeOrder())

        /** Whether the file at [path] is a valid `.etc` project → `{"ok": true}` or the error. */
        fun validate(path: String): String = nativeValidate(path)

        /** Set the folder the built-in preset libraries are read from (once, at start). */
        fun setPresetsRoot(path: String) = nativeSetPresetsRoot(path)

        @JvmStatic private external fun nativeCreate(): Long
        @JvmStatic private external fun nativeDestroy(handle: Long)
        @JvmStatic private external fun nativeSetPresetsRoot(path: String)
        @JvmStatic private external fun nativeExecute(handle: Long, command: String, params: String): String
        @JvmStatic private external fun nativePointer(handle: Long, event: String, view: String): String
        @JvmStatic private external fun nativeSelectTool(handle: Long, id: String, view: String): String
        @JvmStatic private external fun nativeOpen(handle: Long, temp: String, name: String, uri: String): String
        @JvmStatic private external fun nativeSave(handle: Long, mode: String, params: String, tempDir: String): String
        @JvmStatic private external fun nativeSaveFinish(handle: Long, ok: Boolean): String
        @JvmStatic private external fun nativeValidate(path: String): String
        @JvmStatic private external fun nativeRender(
            handle: Long,
            x: Double,
            y: Double,
            width: Int,
            height: Int,
            scale: Double,
            out: ByteBuffer,
        ): Boolean
    }
}

// GLOSSARY
// - EngineBridge: one engine session behind the native library `libetcbridge.so`.
// - handle: the engine session's handle (0 once closed).
// - isClosed: whether close() has run.
// - live(): the open handle, or an error once closed.
// - execute(command, paramsJson): run an engine command → JSON answer.
// - pointer(eventJson, viewJson): feed a pointer event to the active tool → JSON answer.
// - selectTool(id, viewJson): make a tool active → JSON answer.
// - render(x, y, width, height, scale, out): draw part of the active document into a direct buffer.
// - openCopied(tempPath, name, uri): open a file copied to a temporary file, with its URI as path.
// - saveToTemp(mode, paramsJson, tempDir): write a save into temporary files and keep it pending.
// - saveFinish(ok): complete or drop the pending save.
// - validate(path): whether a file is a valid .etc project.
// - close(): close the engine session.
// - renderBuffer(width, height): a direct buffer sized for render().
// - setPresetsRoot(path): set the built-in presets folder.
// - nativeCreate, nativeDestroy, nativeSetPresetsRoot, nativeExecute, nativePointer, nativeSelectTool, nativeOpen, nativeSave, nativeSaveFinish, nativeValidate, nativeRender: the JNI functions in engine/crates/bridge/src/jni_exports.rs.
