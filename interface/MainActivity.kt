package com.extemporecreare.app.ui

import android.net.Uri
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.extemporecreare.app.components.documents.DocumentSettings
import com.extemporecreare.app.components.documents.RecentFiles
import com.extemporecreare.app.ui.document.NewDocumentDialog
import com.extemporecreare.app.ui.editor.EditorRequests
import com.extemporecreare.app.ui.editor.EditorScreen
import com.extemporecreare.app.ui.editor.EditorState
import com.extemporecreare.app.ui.splash.SettingsDialog
import com.extemporecreare.app.ui.splash.SplashActions
import com.extemporecreare.app.ui.splash.SplashScreen

/**
 * The app's activity (`uistandards.md` §5.1): a cold launch shows the opening splash; returning
 * from another app, sleep or a configuration change keeps the editor as it was (the activity
 * isn't recreated for configuration changes). It holds the system pickers for Save As (a new `.etc`
 * file), Open, Locate (a missing recent file) and the folder Export writes into, and the New
 * Document and Settings dialogs.
 */
class MainActivity : ComponentActivity() {

    /** The editor's state, made once. */
    private lateinit var state: EditorState

    /** The dialog showing over the screen, if any. */
    private enum class DialogShown { NONE, NEW_DOCUMENT, DEFAULTS, SETTINGS }
    private var dialog by mutableStateOf(DialogShown.NONE)

    /** The recent file a Locate replaces. */
    private var locating: RecentFiles.Recent? = null

    private val saveAs = registerForActivityResult(ActivityResultContracts.CreateDocument(ETC_MIME)) { uri -> uri?.let { state.save(it) } }
    private val open = registerForActivityResult(ActivityResultContracts.OpenDocument()) { uri -> uri?.let { state.open(it) } }
    private val locate = registerForActivityResult(ActivityResultContracts.OpenDocument()) { uri -> uri?.let { relocated(it) } }
    private val exportFolder = registerForActivityResult(ActivityResultContracts.OpenDocumentTree()) { uri -> uri?.let { state.exportTo(it) } }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        state = EditorState(applicationContext)
        state.start()
        val requests = EditorRequests(
            newDocument = { dialog = DialogShown.NEW_DOCUMENT },
            saveAs = { saveAs.launch("Untitled.etc") },
            open = { open.launch(arrayOf("*/*")) },
            exportFolder = { exportFolder.launch(null) },
            openBoard = { state.message = "The reference board's view is not built yet" },
        )
        val splash = SplashActions(
            newDocument = { dialog = DialogShown.NEW_DOCUMENT },
            open = { open.launch(arrayOf("*/*")) },
            openRecent = { state.open(it.uri) },
            locate = { locating = it; locate.launch(arrayOf("*/*")) },
            remove = { state.forget(it.uri) },
            settings = { dialog = DialogShown.SETTINGS },
        )
        setContent {
            when (state.screen) {
                EditorState.Screen.SPLASH -> SplashScreen(state.recents, state.recentsRevision, splash)
                EditorState.Screen.EDITOR -> EditorScreen(state, requests)
            }
            when (dialog) {
                DialogShown.NEW_DOCUMENT -> NewDocumentDialog(DocumentSettings.defaults(this), { dialog = DialogShown.NONE; state.newDocument(it) }, { dialog = DialogShown.NONE })
                DialogShown.DEFAULTS -> NewDocumentDialog(DocumentSettings.defaults(this), { DocumentSettings.keepDefaults(this, it); dialog = DialogShown.SETTINGS }, { dialog = DialogShown.SETTINGS })
                DialogShown.SETTINGS -> SettingsDialog(
                    defaultsSummary = summary(DocumentSettings.defaults(this)),
                    onEditDefaults = { dialog = DialogShown.DEFAULTS },
                    onClearRecents = { state.clearRecents() },
                    onResetLayout = { state.layout.reset() },
                    onClose = { dialog = DialogShown.NONE },
                )
                DialogShown.NONE -> {}
            }
        }
    }

    /** A Locate picked [uri] for a missing recent file: it takes the old entry's place and opens. */
    private fun relocated(uri: Uri) {
        locating?.let { state.forget(it.uri) }
        locating = null
        state.open(uri)
    }

    /** The New Document defaults in a line. */
    private fun summary(s: DocumentSettings): String =
        if (s.infinite) "Infinite canvas, ${s.ppi.toInt()} ppi, ${s.colorMode.uppercase()}" else "${s.width} × ${s.height} ${s.units}, ${s.ppi.toInt()} ppi, ${s.colorMode.uppercase()}, ${s.background}"

    /** Going to the background: a recovery copy of unsaved work (§5.1). */
    override fun onPause() {
        super.onPause()
        state.saveRecovery()
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
// - MainActivity: the app's activity: splash or editor, the pickers and the dialogs.
// - state: the editor's state. DialogShown, dialog: the dialog showing. locating: the recent file a Locate replaces.
// - saveAs, open, locate, exportFolder: the pickers' launchers.
// - onPause(): a recovery copy when going to the background.
// - relocated(uri): a located file replaces its missing entry and opens.
// - summary(s): the New Document defaults in a line.
// - ETC_MIME: the .etc MIME type.
