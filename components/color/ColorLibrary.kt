package com.extemporecreare.app.components.color

import android.content.Context
import org.json.JSONArray

/**
 * The colour workspace's saved swatches and recent colours (`uistandards.md` §8.1), kept app-wide in
 * the app's preferences (the palette's persistence scope is still open, §13.3). History holds the
 * last [HISTORY] colours used, newest first.
 */
class ColorLibrary(context: Context) {

    private val prefs = context.getSharedPreferences(FILE, Context.MODE_PRIVATE)

    /** The saved swatches, in order. */
    fun swatches(): List<String> = read("swatches")

    /** The recent colours, newest first. */
    fun history(): List<String> = read("history")

    /** Save [hex] as a swatch (once). */
    fun addSwatch(hex: String) = write("swatches", (swatches() + hex).distinct())

    /** Remove swatch [hex]. */
    fun removeSwatch(hex: String) = write("swatches", swatches() - hex)

    /** Note [hex] as used. */
    fun used(hex: String) = write("history", (listOf(hex) + history().filter { it != hex }).take(HISTORY))

    private fun read(key: String): List<String> {
        val arr = runCatching { JSONArray(prefs.getString(key, "[]")) }.getOrDefault(JSONArray())
        return (0 until arr.length()).map { arr.optString(it) }.filter { ColorMath.parse(it) != null }
    }

    private fun write(key: String, items: List<String>) {
        prefs.edit().putString(key, JSONArray(items).toString()).apply()
    }

    companion object {
        /** How many recent colours are kept. */
        const val HISTORY = 12

        /** The preferences file. */
        const val FILE = "colour-library"
    }
}

// GLOSSARY
// - ColorLibrary: saved swatches and recent colours, app-wide.
// - swatches(), history(): read them. addSwatch(hex), removeSwatch(hex), used(hex): change them.
// - read(key), write(key, items): the preferences.
// - HISTORY: 12. FILE: "colour-library".
