package com.extemporecreare.app.ui

import android.net.Uri
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.SystemBarStyle
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.displayCutout
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.union
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.ui.Modifier
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import com.extemporecreare.app.components.documents.DocumentSettings
import com.extemporecreare.app.components.documents.RecentFiles
import com.extemporecreare.app.ui.document.NewDocumentDialog
import com.extemporecreare.app.ui.editor.EditorRequests
import com.extemporecreare.app.ui.panels.BoardActions
import com.extemporecreare.app.ui.editor.EditorScreen
import com.extemporecreare.app.ui.editor.EditorState
import com.extemporecreare.app.ui.splash.SettingsDialog
import com.extemporecreare.app.ui.splash.SplashActions
import com.extemporecreare.app.ui.splash.SplashScreen
import com.extemporecreare.app.ui.theme.EtcColors

/**
 * The app's activity (`uistandards.md` §5.1): a cold launch shows the opening splash; returning
 * from another app, sleep or a configuration change keeps the editor as it was (the activity
 * isn't recreated for configuration changes). It holds the system pickers for Save As (a new `.etc`
 * file), Open, Locate (a missing recent file), the folder Export writes into and the reference
 * board (add an image, open, save, relink), and the New Document and Settings dialogs.
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

    /** The reference board's pickers: add an image, open or save a `.etcr`, relink a missing image. */
    private val boardAdd = registerForActivityResult(ActivityResultContracts.OpenDocument()) { uri -> uri?.let { state.boardDone(state.board.addImage(it)) } }
    private val boardOpen = registerForActivityResult(ActivityResultContracts.OpenDocument()) { uri -> uri?.let { state.boardSelected = null; state.boardDone(state.board.open(it)) } }
    private val boardSave = registerForActivityResult(ActivityResultContracts.CreateDocument(ETCR_MIME)) { uri -> uri?.let { state.boardDone(state.board.save(it)) } }
    private val boardRelink = registerForActivityResult(ActivityResultContracts.OpenDocument()) { uri -> uri?.let { file -> relinking?.let { state.boardDone(state.board.relink(it, file)) } }; relinking = null }

    /** The board image a Relink replaces. */
    private var relinking: String? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // The system bars stay the system's: light icons over the app's dark shell, the app laid out between them.
        enableEdgeToEdge(
            statusBarStyle = SystemBarStyle.dark(android.graphics.Color.TRANSPARENT),
            navigationBarStyle = SystemBarStyle.dark(android.graphics.Color.TRANSPARENT),
        )
        state = EditorState(applicationContext)
        state.start()
        val requests = EditorRequests(
            newDocument = { dialog = DialogShown.NEW_DOCUMENT },
            saveAs = { saveAs.launch("Untitled.etc") },
            open = { open.launch(arrayOf("*/*")) },
            exportFolder = { exportFolder.launch(null) },
        )
        val board = BoardActions(
            add = { boardAdd.launch(arrayOf("image/*")) },
            newBoard = { state.boardSelected = null; state.board.newBoard(); state.boardChanged() },
            open = { boardOpen.launch(arrayOf("*/*")) },
            save = { state.board.uri?.let { state.boardDone(state.board.save(it)) } ?: boardSave.launch("References.etcr") },
            relink = { relinking = it; boardRelink.launch(arrayOf("image/*")) },
            embed = { state.boardEmbed(it) },
            select = { state.boardSelected = it },
            changed = { state.boardChanged() },
            picture = { state.boardPicture(it) },
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
            // Laid out inside the status bar, navigation bar and any display cutout, never under them.
            Box(Modifier.fillMaxSize().background(EtcColors.Shell).windowInsetsPadding(WindowInsets.systemBars.union(WindowInsets.displayCutout))) {
                when (state.screen) {
                    EditorState.Screen.SPLASH -> SplashScreen(state.recents, state.recentsRevision, splash)
                    EditorState.Screen.EDITOR -> EditorScreen(state, requests, board)
                }
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
        state.backgroundBoard()
    }

    override fun onDestroy() {
        super.onDestroy()
        if (isFinishing) state.engine.close()
    }

    companion object {
        /** The `.etc` MIME type (`filestandard.md`). */
        const val ETC_MIME = "application/x-ex-tempore-creare"

        /** The `.etcr` MIME type (`filestandard.md` §7.1). */
        const val ETCR_MIME = "application/x-ex-tempore-creare-reference"
    }
}

// GLOSSARY
// - MainActivity: the app's activity: splash or editor laid out between the system bars, the pickers and the dialogs.
// - state: the editor's state. DialogShown, dialog: the dialog showing. locating: the recent file a Locate replaces.
// - saveAs, open, locate, exportFolder: the pickers' launchers.
// - boardAdd, boardOpen, boardSave, boardRelink: the reference board's pickers. relinking: the image a Relink replaces.
// - onPause(): a recovery copy, and the reference board kept or saved, when going to the background.
// - relocated(uri): a located file replaces its missing entry and opens.
// - summary(s): the New Document defaults in a line.
// - ETC_MIME, ETCR_MIME: the .etc and .etcr MIME types.
