package com.extemporecreare.app.tools.fill

import com.extemporecreare.app.components.engine.EngineBridge
import com.extemporecreare.app.components.tools.EngineTool
import com.extemporecreare.app.components.tools.ToolSettingsStore

/**
 * The Fill tool, built on the engine's Fill Area tool: a tap fills the area enclosed by paths
 * around it with the current fill. The tolerance is Fill Area's gap detection: gaps up to that
 * length close the area.
 */
class FillTool(store: ToolSettingsStore) : EngineTool<FillToolSettings>(NAME, store, FillToolSettings::fromJson) {

    override fun engineToolId(): String = "fillArea"

    override fun pushSettings(engine: EngineBridge): String? {
        val tolerance = settings.tolerance
        setOption(engine, "gapDetection", tolerance > 0)?.let { return it }
        return if (tolerance > 0) setOption(engine, "gapLength", tolerance) else null
    }

    companion object {
        /** The tool's dial name and settings key. */
        const val NAME = "Fill"
    }
}

// GLOSSARY
// - FillTool: the Fill tool (engine Fill Area).
// - engineToolId(): "fillArea".
// - pushSettings(engine): tolerance as gap detection and gap length.
// - NAME: "Fill".
