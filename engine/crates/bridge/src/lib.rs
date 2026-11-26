//! The interface layer between the engine and the Android UI. The Kotlin class
//! `com.extemporecreare.app.components.engine.EngineBridge` (`components/engine/EngineBridge.kt`)
//! loads this library (`libetcbridge.so`) and calls the JNI functions in [`jni_exports`], which
//! hand off to [`calls`].

pub mod boards;
pub mod calls;
pub mod colors;
pub mod documents;
mod convert;
pub mod files;
mod jni_exports;
pub mod paint;
pub mod panels;
mod sessions;

pub use sessions::{create, destroy};

// GLOSSARY
// - `boards`: reference boards (`.etcr`) and their linked images, as JSON.
// - `calls`: the operations the UI asks of the engine, answered as JSON.
// - `colors`: the paint defaults the tools paint with (primary: stroke, alternate: fill).
// - `documents`: new documents (finite or infinite) and the artwork camera they keep.
// - `convert`: JSON conversions of view state and tool requests.
// - `files`: opening and saving through temporary files for content URIs.
// - `jni_exports`: the JNI functions of `EngineBridge`.
// - `paint`: the Brush tool's live raster strokes into splotches.
// - `panels`: the Layers panel's rows.
// - `sessions`: open engine sessions by handle.
// - `create`, `destroy`: open and close a session (re-exported from `sessions`).
