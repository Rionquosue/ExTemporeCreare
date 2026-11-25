package com.extemporecreare.app.components.engine

import android.content.Context
import java.io.File

/**
 * The built-in preset libraries (`tools/Libraries/presets`, packaged as the `presets` assets) on the
 * device: copied into the app's files folder, then given to the engine as its presets folder.
 * Call [install] once at start, before the first library is opened.
 */
object EnginePresets {

    /** The assets folder the presets are packaged in. */
    private const val ASSETS = "presets"

    /** Copy the bundled presets into the app's files folder (again whenever the app's version
     * changed) and set it as the engine's presets folder → that folder. */
    fun install(context: Context): File {
        val root = File(context.filesDir, ASSETS)
        val stamp = File(root, ".version")
        val version = context.packageManager.getPackageInfo(context.packageName, 0).lastUpdateTime.toString()
        if (!stamp.exists() || stamp.readText() != version) {
            root.deleteRecursively()
            copyAssets(context, ASSETS, root)
            stamp.writeText(version)
        }
        EngineBridge.setPresetsRoot(root.path)
        return root
    }

    /** Copy asset folder [path] into [dest], folders included. */
    private fun copyAssets(context: Context, path: String, dest: File) {
        val children = context.assets.list(path).orEmpty()
        if (children.isEmpty()) {
            dest.parentFile?.mkdirs()
            context.assets.open(path).use { src -> dest.outputStream().use { src.copyTo(it) } }
            return
        }
        dest.mkdirs()
        for (child in children) {
            copyAssets(context, "$path/$child", File(dest, child))
        }
    }
}

// GLOSSARY
// - EnginePresets: puts the bundled preset libraries on the device for the engine.
// - ASSETS: "presets", the assets folder they are packaged in.
// - install(context): copy them into filesDir/presets when the app version changed, and set the engine's presets folder.
// - copyAssets(context, path, dest): copy an assets folder to a folder on the device.
