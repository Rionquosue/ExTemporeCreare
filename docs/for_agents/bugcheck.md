# Bugcheck procedures

## Before every prep to push into a mainline release perform the following bugchecks:

    Undo bugs (Z)

    1. Everything applied by one user gesture, button press or confirmation (a brush stroke, a filter apply, a layer move, a transform commit) should be undone in a single Undo step.
    2. Actions should not grab, merge into or overwrite another action's Undo history. If an action cannot avoid breaking the Undo history, do not perform it without a warning popup.
    3. Changes that live outside the document's Undo history, such as files written to storage, exports or `.etcr` reference-board edits, should be listed in the log so users can find and remove them if desired.
    4. Document geometry and export-slice changes must participate in Undo/Redo, as required by `applicationstandard.md`.

    UI Bugs (U)

    1. UI elements that collide or draw over each other, such as an input field drawing over the control below it, or a floating panel covering the rotary.
    2. Fixed-size panels or dialogs that do not reflow on rotation, split-screen, or different tablet and phone screen sizes.
    3. Long text, labels or handles that crowd neighbouring controls or get clipped.
    4. Developer notes that have made their way into the UI, such as "No X is applied until Y process is done". They cause clutter and do not clarify anything for the user.
    5. Touch targets too small or too close together for finger or stylus use.
    6. UI overlays (dot grid, handles, guides, selection highlights, export controls) that appear in exported artwork.
    7. Anything that contradicts `uistandards.md`, such as a moved or replaced approved control, or a re-added redundant readout.

    Destructive Bugs (D)

    1. Anything that can overwrite or delete something that is not part of the purposeful action of the tool, such as a filter writing into a layer other than the selected one.
    2. Edits that leak into shared data, such as a texture, linked asset or `.etcr` reference board that other documents use.
    3. Destructive edits where the standards require non-destructive ones (masks, effects, filters, export slices and reference transforms must stay editable).
    4. Wrong keys, such as identifying a layer, asset or document by display name instead of its stable identifier.
    5. Save paths that can corrupt or replace the last good `.etc` file before the new archive has been validated.
    6. Session or recovery-checkpoint data lost when the app is backgrounded, suspended or terminated by the OS.
    7. Data loss on apply, or anything irreversible done without warning.

    Log issues (L)

    1. Any action that does not generate a log entry.
    2. Any error that does not generate a log entry that can be traced back to the exact script that errored.
    3. Excessive log entries from a single action, such as autosave or recovery checkpointing writing an entry on every tick.


## Perform the following resolution standards:

    (Z)
        Group everything the action does into a single Undo transaction, or consolidate the history once the action completes, so one press undoes the whole action instead of needing several presses.

    (U)
        Follow standard flexible layout and tab-based UI conventions and the placement rules in `uistandards.md`. Give every UI element some spacing, use collapsible sections where applicable, and make sure every element stays reachable on every supported screen size and orientation. If a tool needs more space than its panel allows, move it into a popup or expandable panel rather than shrinking it.

    (D)
        Fix every destructive bug. Never edit shared assets in place; apply changes to the document's own copy or as a non-destructive layer, mask or filter instead. Saves must write to a temporary file, validate it, then replace the original. If a destructive edit is unavoidable because of what the tool does, show a warning before it runs, prompt the user to save first, and state that Undo will not reverse it.

    (L)
        Each tool run should generate one log entry when it starts and one when it completes, with a summary of what was changed. If an error can happen without a log entry, treat it as a critical bug and fix it immediately so it can be traced. Repeating background jobs such as autosave or recovery checkpoints should only log when they are turned on, turned off, or hit an exception. This keeps the log free of non-contextual noise.

## If none of the above can be applied

Append to the end of this file below the break line with what the bug is, and recommended fixes.


---

1. (U1, U2, U3) Overlapping elements, panels that don't reflow, and crowded long text can only be confirmed by viewing the running app; reading the code cannot find them for certain. Recommended fix: open each panel, popup and dialog at the smallest supported screen size in both orientations, and list what overlaps.
