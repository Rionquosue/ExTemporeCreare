package com.extemporecreare.app.components.tools

import com.extemporecreare.app.components.engine.EngineBridge

/**
 * A tool of the rotary dial (`uistandards.md` §6): it becomes active when the dial turns it into
 * the active sight, takes the canvas's pointer events, and stops when the dial turns away.
 */
interface CanvasTool {

    /** The tool's name in the dial ("Brush"); also the key its settings are saved under. */
    val name: String

    /** Make the tool active with the view state [viewJson]. */
    fun activate(engine: EngineBridge, viewJson: String): ToolResult

    /** Handle one pointer event on the canvas. */
    fun pointer(engine: EngineBridge, event: ToolPointer, viewJson: String): ToolResult

    /** The dial turned away from the tool. */
    fun deactivate(engine: EngineBridge): ToolResult
}

// GLOSSARY
// - CanvasTool: a rotary dial tool.
// - name: the tool's dial name and settings key.
// - activate(engine, viewJson): make the tool active.
// - pointer(engine, event, viewJson): handle a canvas pointer event.
// - deactivate(engine): the dial turned away.
