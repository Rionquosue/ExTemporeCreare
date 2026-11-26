package com.extemporecreare.app.ui

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import com.extemporecreare.app.ui.editor.EditorRequests
import com.extemporecreare.app.ui.editor.EditorScreen
import com.extemporecreare.app.ui.editor.EditorState

/**
 * The app's activity: the editor, with the system pickers for Save As (a new `.etc` file), Open,
 * and the folder Export writes into. Configuration changes (rotation, resizing) keep the activity,
 * so the open document and the editor's state stay as they are.
 */
class MainActivity : ComponentActivity() {

    /** The editor's state, made once. */
    private lateinit var state: EditorState

    private val saveAs = registerForActivityResult(ActivityResultContracts.CreateDocument(ETC_MIME)) { uri -> uri?.let { state.save(it) } }
    private val open = registerForActivityResult(ActivityResultContracts.OpenDocument()) { uri -> uri?.let { state.open(it) } }
    private val exportFolder = registerForActivityResult(ActivityResultContracts.OpenDocumentTree()) { uri -> uri?.let { state.exportTo(it) } }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        state = EditorState(applicationContext)
        state.start()
        val requests = EditorRequests(
            saveAs = { saveAs.launch("Untitled.etc") },
            open = { open.launch(arrayOf("*/*")) },
            exportFolder = { exportFolder.launch(null) },
            openBoard = { state.message = "The reference board's view is not built yet" },
        )
        setContent { EditorScreen(state, requests) }
    }

    override fun onDestroy() {
        super.onDestroy()
        if (isFinishing) state.engine.close()
    }

    companion object {
        /** The `.etc` MIME type (`filestandard.md`). */
        const val ETC_MIME = "application/x-ex-tempore-creare"
    }
}

// GLOSSARY
// - MainActivity: the app's activity hosting the editor and the system pickers.
// - state: the editor's state.
// - saveAs, open, exportFolder: the pickers' launchers.
// - ETC_MIME: the .etc MIME type.
