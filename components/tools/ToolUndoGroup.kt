package com.extemporecreare.app.components.tools

import com.extemporecreare.app.components.engine.EngineBridge
import org.json.JSONObject

/**
 * One undo step around several engine edits (`edit.undoGroup.begin` / `edit.undoGroup.end`), so a
 * single gesture undoes in one step (`script_structure.md`, Undo rule 1).
 */
class ToolUndoGroup {

    /** Whether a group is open. */
    var isOpen: Boolean = false
        private set

    /** Open a group named [label] → an error message, or `null`. */
    fun begin(engine: EngineBridge, label: String): String? {
        if (isOpen) return null
        val answer = EngineAnswer(engine.execute("edit.undoGroup.begin", JSONObject().put("label", label).toString()))
        isOpen = answer.isOk
        return answer.error
    }

    /** Close the open group; with [cancel] its edits are undone → an error message, or `null`. */
    fun end(engine: EngineBridge, cancel: Boolean = false): String? {
        if (!isOpen) return null
        isOpen = false
        return EngineAnswer(engine.execute("edit.undoGroup.end", JSONObject().put("cancel", cancel).toString())).error
    }
}

// GLOSSARY
// - ToolUndoGroup: one undo step around several engine edits.
// - isOpen: whether a group is open.
// - begin(engine, label): open a group.
// - end(engine, cancel): close the group, undoing it with cancel.
