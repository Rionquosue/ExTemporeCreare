package com.extemporecreare.app.tools.selection

import com.extemporecreare.app.components.tools.EngineTool
import com.extemporecreare.app.components.tools.NoToolSettings
import com.extemporecreare.app.components.tools.ToolSettingsStore

/**
 * The Selection tool, built on the engine's Selection tool: tap or drag a marquee to select
 * objects; dragging a selected object or its bounding box handles moves, resizes or rotates it.
 */
class SelectionTool(store: ToolSettingsStore) : EngineTool<NoToolSettings>(NAME, store, { NoToolSettings }) {

    override fun engineToolId(): String = "selection"

    companion object {
        /** The tool's dial name and settings key. */
        const val NAME = "Selection"
    }
}

// GLOSSARY
// - SelectionTool: the Selection tool (engine Selection).
// - engineToolId(): "selection".
// - NAME: "Selection".
