package com.extemporecreare.app.components.tools

import org.json.JSONObject

/** A tool's settings, kept as JSON by [ToolSettingsStore]. */
interface ToolSettings {
    /** The settings as JSON. */
    fun toJson(): JSONObject
}

/** The settings of a tool that has none (Selection, Move). */
object NoToolSettings : ToolSettings {
    override fun toJson(): JSONObject = JSONObject()
}

// GLOSSARY
// - ToolSettings: a tool's settings as JSON.
// - toJson(): the settings as JSON.
// - NoToolSettings: the settings of a tool that has none.
