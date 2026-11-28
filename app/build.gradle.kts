// The Android app module. Its source sets are the project's own folders (docs/filestructure.md):
// Kotlin from interface/ (UI), components/ (shared components, the engine bridge) and tools/
// (tools); the built-in presets in tools/Libraries/presets ship as assets; the engine is the Rust
// library libetcbridge.so (engine/crates/bridge), built for each ABI by `buildEngine`.

plugins {
    id("com.android.application")
    id("org.jetbrains.kotlin.plugin.compose")
}

android {
    namespace = "com.extemporecreare.app"
    // 37.2: Compose 1.13 (BOM 2026.11.01) requires compiling against 37.1 or later.
    compileSdk {
        version = release(37) {
            minorApiLevel = 2
        }
    }
    buildToolsVersion = "37.0.0"
    // The NDK buildEngine builds the engine with (cargo-ndk, through ANDROID_NDK_HOME).
    ndkVersion = "30.0.16248370"

    defaultConfig {
        applicationId = "com.extemporecreare.app"
        // 26: DocumentsContract.findDocumentPath, which replacing a saved file by rename uses.
        minSdk = 26
        targetSdk = 37
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

    buildFeatures {
        compose = true
    }

    sourceSets {
        getByName("main") {
            kotlin.directories += listOf("../interface", "../components", "../tools")
            java.directories += listOf("../interface", "../components", "../tools")
            // tools/Libraries/presets → assets/presets
            assets.directories += "../tools/Libraries"
            jniLibs.directories += layout.buildDirectory.dir("engine/jniLibs").get().asFile.path
        }
    }
}

dependencies {
    val composeBom = platform("androidx.compose:compose-bom:2026.11.01")
    implementation(composeBom)
    implementation("androidx.core:core-ktx:1.19.1")
    implementation("androidx.activity:activity-compose:1.13.0")
    implementation("androidx.compose.ui:ui")
    implementation("androidx.compose.foundation:foundation")
    implementation("androidx.compose.material3:material3")
    // The tool, panel and top bar icons.
    implementation("androidx.compose.material:material-icons-extended")
}

/**
 * Build libetcbridge.so for each ABI with cargo-ndk (`cargo install cargo-ndk`; needs the Android
 * NDK and the Rust targets aarch64-linux-android and x86_64-linux-android) into
 * build/engine/jniLibs, which the app packages. It uses the NDK named by android.ndkVersion,
 * passed to cargo-ndk as ANDROID_NDK_HOME, so no ANDROID_NDK_HOME needs setting by hand.
 */
val buildEngine = tasks.register<Exec>("buildEngine") {
    description = "Builds the engine bridge (libetcbridge.so) for every ABI with cargo-ndk."
    workingDir = rootProject.file("engine")
    val out = layout.buildDirectory.dir("engine/jniLibs").get().asFile
    // Cargo's build files stay under app/build, out of the repository's tracked folders.
    environment("CARGO_TARGET_DIR", layout.buildDirectory.dir("engine/target").get().asFile.absolutePath)
    val ndk = androidComponents.sdkComponents.ndkDirectory
    doFirst {
        environment("ANDROID_NDK_HOME", ndk.get().asFile.absolutePath)
    }
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
// - minSdk 26, targetSdk 37, compileSdk 37.2, buildToolsVersion 37.0.0, ndkVersion 30.0.16248370.
// - abiFilters: arm64-v8a (tablets and phones) and x86_64 (emulators).
// - sourceSets.main: Kotlin from interface/, components/, tools/; assets from tools/Libraries (presets/); jniLibs from build/engine/jniLibs.
// - dependencies: Compose (BOM 2026.11.01), core-ktx 1.19.1, activity-compose 1.13.0, material3.
// - buildEngine: builds libetcbridge.so with cargo-ndk before every build (preBuild), with the NDK of ndkVersion as ANDROID_NDK_HOME.
