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
     * Feed a pointer event to the active tool → the JSON answer, whose `ok` is `{"requests": [the
     * tool's requests to the UI], "created": [ids], "changed"}`; on a release (`up`) the tool's edit is
     * applied first, `created` lists the outermost objects the gesture added since its press and
     * `changed` tells whether it changed the document. [eventJson] is `{"kind": "down"|"drag"|"up"|"move", "pos": {"x", "y"},
     * "pressure"?, "mods"?}` in document points; [viewJson] is the view state (`zoom`, `outline`,
     * `smartGuides`, `snapToGrid`…; missing fields keep the engine's defaults).
     */
    fun pointer(eventJson: String, viewJson: String = "{}"): String =
        nativePointer(live(), eventJson, viewJson)

    /** Make tool [id] the active tool → the JSON answer. */
    fun selectTool(id: String, viewJson: String = "{}"): String =
        nativeSelectTool(live(), id, viewJson)

    /** The active tool's options → `{"ok": {"tool", "options"}}`. */
    fun toolOptions(): String = nativeToolOptions(live())

    /**
     * Set option [key] of the active tool to [valueJson] (JSON text) → `{"ok": {"tool", "options"}}`.
     * The engine keeps each tool's options when another tool is chosen.
     */
    fun setToolOption(key: String, valueJson: String): String = nativeSetToolOption(live(), key, valueJson)

    /**
     * What the app's tools read → `{"ok": {"tool", "document", "canvas": "finite"|"infinite",
     * "artboards": [{"index", "x", "y", "width", "height"}], "selection": [ids], "images": [ids],
     * "editing"}}`, in document points.
     */
    fun state(): String = nativeState(live())

    /** What the active tool draws over the canvas → `{"ok": [{"type", …}]}`, in document coordinates. */
    fun overlays(viewJson: String = "{}"): String = nativeOverlays(live(), viewJson)

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
     * A new document from the New Document dialog's [settingsJson] (`{"name", "infinite", "width",
     * "height" (pt), "units"?, "colorMode"?, "ppi"?, "background"?: "transparent"|"white"|"#rrggbb"}`)
     * → `{"ok": {"infinite", "artboards"}}`. An infinite canvas has no artboards.
     */
    fun newDocument(settingsJson: String): String = nativeNewDocument(live(), settingsJson)

    /** The artwork camera the active document keeps → `{"ok": {"x", "y", "zoom"} | null}`. */
    fun view(): String = nativeView(live())

    /** Keep [cameraJson] (`{"x", "y", "zoom"}`) as the document's camera, saved with it. */
    fun setView(cameraJson: String): String = nativeSetView(live(), cameraJson)

    /**
     * The Layers panel's rows, topmost first → `{"ok": {"rows": [{"id", "name", "kind":
     * "folder"|"raster"|"vector"|"guide", "visible", "locked", "opacity" (0–100), "blend", "current",
     * "children"?}]}}`.
     */
    fun layers(): String = nativeLayers(live())

    /** The paint defaults the tools paint with → `{"ok": {"stroke": "#rrggbb"|null, "fill": "#rrggbb"|null}}` (primary: stroke, alternate: fill). */
    fun paintDefaults(): String = nativePaintDefaults(live())

    /** Set the paint defaults in [valuesJson] (`{"stroke"?, "fill"?}`, `"#rrggbb"` or null) without touching the selection → the defaults. */
    fun setPaintDefaults(valuesJson: String): String = nativeSetPaintDefaults(live(), valuesJson)

    /**
     * The Brush tool's pen went down: get a live raster stroke and its splotch ready. [paramsJson] is
     * as `paint.stroke` takes it without `points` (`view`, `zoom`, `locked`?, `size`, `hardness`,
     * `spacing`, `flow`, `opacity`, `pressureSize`, `color`?) → `{"ok": {"x", "y", "width", "height",
     * "pixelsWide", "pixelsHigh", "splotch"}}`, the splotch's document area and pixel size.
     */
    fun paintBegin(paramsJson: String): String = nativePaintBegin(live(), paramsJson)

    /** Paint the live stroke on to [pointsJson] (`[[x, y, pressure]…]`) → `{"ok": {"painted": [x, y, w, h] | null}}`, splotch pixels. */
    fun paintAdd(pointsJson: String): String = nativePaintAdd(live(), pointsJson)

    /**
     * The live stroke's own layer over splotch pixels ([x], [y], [width] × [height]) into [out] as
     * premultiplied RGBA (what an `ARGB_8888` bitmap's `copyPixelsFromBuffer` reads), for the canvas
     * to draw over the document while the stroke lasts → whether [out] was filled.
     */
    fun paintPixels(x: Int, y: Int, width: Int, height: Int, out: ByteBuffer): Boolean {
        require(out.isDirect) { "the paint buffer must be direct" }
        return nativePaintPixels(live(), x, y, width, height, out)
    }

    /** The pen was released: keep the live stroke in its splotch, one undo step → `{"ok": {"id", "created", "width", "height"}}`. */
    fun paintEnd(): String = nativePaintEnd(live())

    /** Drop the live stroke, changing nothing → the JSON answer. */
    fun paintCancel(): String = nativePaintCancel(live())

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

    /**
     * End the pending save: [ok] when every file reached its URI → the JSON answer. [finalUri], when
     * not empty, is the URI the document's file ended up at (a replace by rename can change it).
     */
    fun saveFinish(ok: Boolean, finalUri: String = ""): String = nativeSaveFinish(live(), ok, finalUri)

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

        /** A new empty reference board titled [title] → `{"ok": board}` (`board.json` as JSON). */
        fun boardNew(title: String): String = nativeBoardNew(title)

        /** Read the `.etcr` file at [path], checking its links from [boardDir] → `{"ok": {"board", "links"}}`. */
        fun boardRead(path: String, boardDir: String, embeddedDir: String = ""): String = nativeBoardRead(path, boardDir, embeddedDir)

        /** Write [boardJson] as an `.etcr` file at [path] (atomically), with the PNG at [thumbnail] if not empty. */
        fun boardWrite(path: String, boardJson: String, thumbnail: String = "", embeddedDir: String = ""): String = nativeBoardWrite(path, boardJson, thumbnail, embeddedDir)

        /** The link status of each image of [boardJson], from [boardDir] → `{"ok": [{"id", "status", …}]}`. */
        fun boardLinks(boardJson: String, boardDir: String): String = nativeBoardLinks(boardJson, boardDir)

        /** [boardJson] with image [id] linked to the file at [file] → `{"ok": board}`. */
        fun boardRelink(boardJson: String, id: String, file: String, boardDir: String): String =
            nativeBoardRelink(boardJson, id, file, boardDir)

        /** Files in [foldersJson] (a JSON array of paths) that are the board's missing images → `{"ok": [{"id", "path"}]}`. */
        fun boardFindMissing(boardJson: String, boardDir: String, foldersJson: String): String =
            nativeBoardFindMissing(boardJson, boardDir, foldersJson)

        /** Set the folder the built-in preset libraries are read from (once, at start). */
        fun setPresetsRoot(path: String) = nativeSetPresetsRoot(path)

        @JvmStatic private external fun nativeCreate(): Long
        @JvmStatic private external fun nativeDestroy(handle: Long)
        @JvmStatic private external fun nativeSetPresetsRoot(path: String)
        @JvmStatic private external fun nativeExecute(handle: Long, command: String, params: String): String
        @JvmStatic private external fun nativePointer(handle: Long, event: String, view: String): String
        @JvmStatic private external fun nativeSelectTool(handle: Long, id: String, view: String): String
        @JvmStatic private external fun nativeToolOptions(handle: Long): String
        @JvmStatic private external fun nativeSetToolOption(handle: Long, key: String, value: String): String
        @JvmStatic private external fun nativeState(handle: Long): String
        @JvmStatic private external fun nativeOverlays(handle: Long, view: String): String
        @JvmStatic private external fun nativeNewDocument(handle: Long, settings: String): String
        @JvmStatic private external fun nativeView(handle: Long): String
        @JvmStatic private external fun nativeSetView(handle: Long, camera: String): String
        @JvmStatic private external fun nativeLayers(handle: Long): String
        @JvmStatic private external fun nativePaintDefaults(handle: Long): String
        @JvmStatic private external fun nativeSetPaintDefaults(handle: Long, values: String): String
        @JvmStatic private external fun nativePaintBegin(handle: Long, params: String): String
        @JvmStatic private external fun nativePaintAdd(handle: Long, points: String): String
        @JvmStatic private external fun nativePaintPixels(handle: Long, x: Int, y: Int, width: Int, height: Int, out: ByteBuffer): Boolean
        @JvmStatic private external fun nativePaintEnd(handle: Long): String
        @JvmStatic private external fun nativePaintCancel(handle: Long): String
        @JvmStatic private external fun nativeOpen(handle: Long, temp: String, name: String, uri: String): String
        @JvmStatic private external fun nativeSave(handle: Long, mode: String, params: String, tempDir: String): String
        @JvmStatic private external fun nativeSaveFinish(handle: Long, ok: Boolean, finalUri: String): String
        @JvmStatic private external fun nativeValidate(path: String): String
        @JvmStatic private external fun nativeBoardNew(title: String): String
        @JvmStatic private external fun nativeBoardRead(path: String, boardDir: String, embeddedDir: String): String
        @JvmStatic private external fun nativeBoardWrite(path: String, board: String, thumbnail: String, embeddedDir: String): String
        @JvmStatic private external fun nativeBoardLinks(board: String, boardDir: String): String
        @JvmStatic private external fun nativeBoardRelink(board: String, id: String, file: String, boardDir: String): String
        @JvmStatic private external fun nativeBoardFindMissing(board: String, boardDir: String, folders: String): String
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
// - pointer(eventJson, viewJson): feed a pointer event to the active tool → JSON answer (with the objects a gesture added).
// - selectTool(id, viewJson): make a tool active → JSON answer.
// - toolOptions(): the active tool's options → JSON answer.
// - setToolOption(key, valueJson): set one option of the active tool → JSON answer.
// - state(): canvas mode, artboards, selection and selected images → JSON answer.
// - overlays(viewJson): what the active tool draws over the canvas → JSON answer.
// - render(x, y, width, height, scale, out): draw part of the active document into a direct buffer.
// - newDocument(settingsJson): a finite or infinite document → JSON answer.
// - view(), setView(cameraJson): the artwork camera kept with the document.
// - layers(): the Layers panel's rows → JSON answer.
// - paintDefaults(), setPaintDefaults(valuesJson): the primary (stroke) and alternate (fill) colours → JSON answer.
// - paintBegin, paintAdd, paintPixels, paintEnd, paintCancel: the Brush tool's live raster stroke into a splotch.
// - openCopied(tempPath, name, uri): open a file copied to a temporary file, with its URI as path.
// - saveToTemp(mode, paramsJson, tempDir): write a save into temporary files and keep it pending.
// - saveFinish(ok, finalUri): complete or drop the pending save; finalUri is where the file ended up.
// - validate(path): whether a file is a valid .etc project.
// - close(): close the engine session.
// - renderBuffer(width, height): a direct buffer sized for render().
// - setPresetsRoot(path): set the built-in presets folder.
// - boardNew, boardRead, boardWrite, boardLinks, boardRelink, boardFindMissing: reference boards (.etcr) on file paths;
//   read and write take the folder embedded images are kept in.
// - nativeCreate, nativeDestroy, nativeSetPresetsRoot, nativeExecute, nativePointer, nativeSelectTool, nativeToolOptions, nativeSetToolOption, nativeState, nativeOverlays, nativeNewDocument, nativeView, nativeSetView, nativeLayers, nativePaintDefaults, nativeSetPaintDefaults, nativePaint*, nativeOpen, nativeSave, nativeSaveFinish, nativeValidate, nativeBoard*, nativeRender: the JNI functions in engine/crates/bridge/src/jni_exports.rs.
