// Ex-Tempore Creare: the Android app build. The app module (app/) compiles the Kotlin in
// interface/, components/ and tools/; the engine (engine/) is Rust, built into libetcbridge.so.

pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "ExTemporeCreare"
include(":app")

// GLOSSARY
// - pluginManagement: where Gradle plugins come from (Google, Maven Central, the Gradle plugin portal).
// - dependencyResolutionManagement: where libraries come from (Google, Maven Central).
// - rootProject.name: ExTemporeCreare.
// - include(":app"): the Android app module in app/.
