// Root build: the plugin versions every module uses.

plugins {
    id("com.android.application") version "8.7.3" apply false
    id("org.jetbrains.kotlin.android") version "2.0.21" apply false
    id("org.jetbrains.kotlin.plugin.compose") version "2.0.21" apply false
}

// GLOSSARY
// - com.android.application: the Android Gradle plugin (8.7.3), applied in app/.
// - org.jetbrains.kotlin.android: Kotlin for Android (2.0.21), applied in app/.
// - org.jetbrains.kotlin.plugin.compose: the Compose compiler (2.0.21), applied in app/.
