package com.extemporecreare.app.components.tools

import com.extemporecreare.app.components.engine.EngineBridge
import org.json.JSONArray
import org.json.JSONObject

/**
 * A dial tool built on an engine tool ([engineToolId]): activating it chooses that tool in the
 * engine and gives it the app's settings ([pushSettings]); pointer events go to it. Settings of
 * type [S] are read from and saved to [store] under [name].
 */
abstract class EngineTool<S : ToolSettings>(
    final override val name: String,
    private val store: ToolSettingsStore,
    read: (JSONObject) -> S,
) : CanvasTool {

    /** The tool's settings. */
    var settings: S = read(store.load(name))
        private set

    /** Whether the tool is the active one. */
    var isActive: Boolean = false
        private set

    /** The engine tool the current [settings] use. */
    protected abstract fun engineToolId(): String

    /** Give the active engine tool the app's settings → an error message, or `null`. */
    protected open fun pushSettings(engine: EngineBridge): String? = null

    override fun activate(engine: EngineBridge, viewJson: String): ToolResult {
        val chosen = EngineAnswer(engine.selectTool(engineToolId(), viewJson))
        if (!chosen.isOk) return ToolResult.failed(chosen.error ?: "can't choose ${engineToolId()}")
        isActive = true
        return ToolResult.failedOrNone(pushSettings(engine))
    }

    override fun pointer(engine: EngineBridge, event: ToolPointer, viewJson: String): ToolResult =
        ToolResult.fromPointer(engine.pointer(event.toJson(), viewJson))

    override fun deactivate(engine: EngineBridge): ToolResult {
        isActive = false
        return ToolResult.NONE
    }

    /** Change the settings to [changed]: saved, and given to the engine while the tool is active. */
    fun update(engine: EngineBridge, changed: S, viewJson: String = "{}"): ToolResult {
        val before = engineToolId()
        settings = changed
        store.save(name, changed.toJson())
        if (!isActive) return ToolResult.NONE
        return if (engineToolId() != before) activate(engine, viewJson) else ToolResult.failedOrNone(pushSettings(engine))
    }

    /** Set option [key] of the active engine tool to [value] → an error message, or `null`. */
    protected fun setOption(engine: EngineBridge, key: String, value: Any): String? {
        val json = JSONArray().put(value).toString().removePrefix("[").removeSuffix("]")
        return EngineAnswer(engine.setToolOption(key, json)).error
    }
}

// GLOSSARY
// - EngineTool: a dial tool built on an engine tool, with saved settings.
// - settings: the tool's settings.
// - isActive: whether the tool is active.
// - engineToolId(): the engine tool the settings use.
// - pushSettings(engine): give the engine tool the app's settings.
// - activate, pointer, deactivate: CanvasTool, forwarding to the engine tool.
// - update(engine, changed, viewJson): change, save and apply the settings.
// - setOption(engine, key, value): set one engine tool option.
