//! Errors reading or writing `.etc` files.

use crate::zipread::ZipError;

/// What went wrong reading or writing an `.etc` file.
#[derive(Debug, thiserror::Error)]
pub enum EtcError {
    /// Not an Ex-Tempore Creare project, or a damaged one.
    #[error("not an Ex-Tempore Creare project: {0}")]
    NotEtc(String),
    /// Written by a newer version of the format than this app reads.
    #[error("the project's format version {0} is newer than this app reads ({ours})", ours = crate::FORMAT_VERSION)]
    TooNew(String),
    /// An asset the manifest lists is missing or can't be read.
    #[error("the project's asset `{0}` is missing or damaged")]
    BadAsset(String),
    /// The file unpacks to more than this app reads.
    #[error("the project unpacks to more than this app reads")]
    TooBig,
    /// The archive itself is unreadable or unsafe.
    #[error(transparent)]
    Zip(#[from] ZipError),
    /// The document couldn't be written.
    #[error("can't write the project: {0}")]
    Encode(String),
}

impl EtcError {
    /// An [`EtcError::Encode`] from any error.
    pub(crate) fn encode(e: impl std::fmt::Display) -> Self {
        EtcError::Encode(e.to_string())
    }
}

// GLOSSARY
// - `EtcError`: `.etc` read and write errors (`NotEtc`, `TooNew`, `BadAsset`, `TooBig`, `Zip`, `Encode`).
// - `EtcError::encode`: wrap any error as `Encode`.
