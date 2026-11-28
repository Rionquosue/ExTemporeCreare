// Root build: the plugin versions every module uses.

plugins {
    id("com.android.application") version "9.5.3" apply false
    id("org.jetbrains.kotlin.plugin.compose") version "2.4.21" apply false
}

// GLOSSARY
// - com.android.application: the Android Gradle plugin (9.5.3), applied in app/. It compiles Kotlin itself (built-in Kotlin).
// - org.jetbrains.kotlin.plugin.compose: the Compose compiler (2.4.21), applied in app/; its version sets the Kotlin version.
