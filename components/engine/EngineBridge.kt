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

        /** Set the folder the built-in preset libraries are read from (once, at start). */
        fun setPresetsRoot(path: String) = nativeSetPresetsRoot(path)

        @JvmStatic private external fun nativeCreate(): Long
        @JvmStatic private external fun nativeDestroy(handle: Long)
        @JvmStatic private external fun nativeSetPresetsRoot(path: String)
        @JvmStatic private external fun nativeExecute(handle: Long, command: String, params: String): String
        @JvmStatic private external fun nativePointer(handle: Long, event: String, view: String): String
        @JvmStatic private external fun nativeSelectTool(handle: Long, id: String, view: String): String
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
// - close(): close the engine session.
// - renderBuffer(width, height): a direct buffer sized for render().
// - setPresetsRoot(path): set the built-in presets folder.
// - nativeCreate, nativeDestroy, nativeSetPresetsRoot, nativeExecute, nativePointer, nativeSelectTool, nativeRender: the JNI functions in engine/crates/bridge/src/jni_exports.rs.
