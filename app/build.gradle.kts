// The Android app module. Its source sets are the project's own folders (docs/filestructure.md):
// Kotlin from interface/ (UI), components/ (shared components, the engine bridge) and tools/
// (tools); the built-in presets in tools/Libraries/presets ship as assets; the engine is the Rust
// library libetcbridge.so (engine/crates/bridge), built for each ABI by `buildEngine`.

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
}

android {
    namespace = "com.extemporecreare.app"
    compileSdk = 35

    defaultConfig {
        applicationId = "com.extemporecreare.app"
        // 26: DocumentsContract.findDocumentPath, which replacing a saved file by rename uses.
        minSdk = 26
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"
        ndk {
            abiFilters += listOf("arm64-v8a", "x86_64")
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    buildFeatures {
        compose = true
    }

    sourceSets {
        getByName("main") {
            kotlin.srcDirs("../interface", "../components", "../tools")
            java.srcDirs("../interface", "../components", "../tools")
            // tools/Libraries/presets → assets/presets
            assets.srcDirs("../tools/Libraries")
            jniLibs.srcDirs(layout.buildDirectory.dir("engine/jniLibs"))
        }
    }
}

dependencies {
    val composeBom = platform("androidx.compose:compose-bom:2024.12.01")
    implementation(composeBom)
    implementation("androidx.core:core-ktx:1.15.0")
    implementation("androidx.activity:activity-compose:1.9.3")
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.foundation:foundation")
    implementation("androidx.compose.material3:material3")
}

/**
 * Build libetcbridge.so for each ABI with cargo-ndk (`cargo install cargo-ndk`; needs the Android
 * NDK and the Rust targets aarch64-linux-android and x86_64-linux-android) into
 * build/engine/jniLibs, which the app packages.
 */
val buildEngine by tasks.registering(Exec::class) {
    description = "Builds the engine bridge (libetcbridge.so) for every ABI with cargo-ndk."
    workingDir = rootProject.file("engine")
    val out = layout.buildDirectory.dir("engine/jniLibs").get().asFile
    commandLine(
        "cargo", "ndk",
        "-t", "arm64-v8a",
        "-t", "x86_64",
        "-o", out.absolutePath,
        "build", "--release", "-p", "etc-bridge",
    )
}

tasks.named("preBuild") {
    dependsOn(buildEngine)
}

// GLOSSARY
// - android.namespace / applicationId: com.extemporecreare.app.
// - minSdk 26, targetSdk 35, compileSdk 35.
// - abiFilters: arm64-v8a (tablets and phones) and x86_64 (emulators).
// - sourceSets.main: Kotlin from interface/, components/, tools/; assets from tools/Libraries (presets/); jniLibs from build/engine/jniLibs.
// - dependencies: Compose (BOM 2024.12.01), core-ktx, activity-compose, material3.
// - buildEngine: builds libetcbridge.so with cargo-ndk before every build (preBuild).
