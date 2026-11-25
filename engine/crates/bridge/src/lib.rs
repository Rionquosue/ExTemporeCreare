//! The interface layer between the engine and the Android UI. The Kotlin class
//! `com.extemporecreare.app.components.engine.EngineBridge` (`components/engine/EngineBridge.kt`)
//! loads this library (`libetcbridge.so`) and calls the JNI functions in [`jni_exports`], which
//! hand off to [`calls`].

pub mod calls;
mod convert;
mod jni_exports;
mod sessions;

pub use sessions::{create, destroy};

// GLOSSARY
// - `calls`: the operations the UI asks of the engine, answered as JSON.
// - `convert`: JSON conversions of view state and tool requests.
// - `jni_exports`: the JNI functions of `EngineBridge`.
// - `sessions`: open engine sessions by handle.
// - `create`, `destroy`: open and close a session (re-exported from `sessions`).
