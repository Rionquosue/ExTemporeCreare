package com.extemporecreare.app.ui.common

import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable

/** A question to confirm before a destructive action: its [question], the action's [label], and what it does. */
class Confirmation(val question: String, val label: String, val action: () -> Unit)

/** Ask [confirmation]'s question; its action runs only when confirmed. [onDone] closes it either way. */
@Composable
fun ConfirmDialog(confirmation: Confirmation, onDone: () -> Unit) {
    AlertDialog(
        onDismissRequest = onDone,
        text = { Text(confirmation.question) },
        confirmButton = { TextButton(onClick = { onDone(); confirmation.action() }) { Text(confirmation.label) } },
        dismissButton = { TextButton(onClick = onDone) { Text("Cancel") } },
    )
}

// GLOSSARY
// - Confirmation: a question, its action's label and the action.
// - ConfirmDialog(confirmation, onDone): ask it; act only when confirmed.
