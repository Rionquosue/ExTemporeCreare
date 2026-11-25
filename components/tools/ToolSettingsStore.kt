package com.extemporecreare.app.components.tools

import android.content.Context
import org.json.JSONObject

/**
 * Each tool's settings, kept across tool switches and app runs (`uistandards.md` §6.2, §12) in the
 * app's preferences file [FILE], one JSON value per tool name.
 */
class ToolSettingsStore(context: Context) {

    /** The preferences file. */
    private val prefs = context.getSharedPreferences(FILE, Context.MODE_PRIVATE)

    /** The settings saved for [tool], or an empty object (the tool's defaults). */
    fun load(tool: String): JSONObject =
        prefs.getString(tool, null)?.let { runCatching { JSONObject(it) }.getOrNull() } ?: JSONObject()

    /** Save [settings] for [tool]. */
    fun save(tool: String, settings: JSONObject) {
        prefs.edit().putString(tool, settings.toString()).apply()
    }

    companion object {
        /** The preferences file name. */
        const val FILE = "tool-settings"
    }
}

// GLOSSARY
// - ToolSettingsStore: each tool's settings in the app's preferences.
// - prefs: the preferences file.
// - load(tool): a tool's saved settings, or {}.
// - save(tool, settings): save a tool's settings.
// - FILE: the preferences file name, "tool-settings".
