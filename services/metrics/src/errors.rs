use std::path::PathBuf;

use thiserror::Error;

pub(crate) type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to get stat from psutil because: {0}")]
    Psutil(#[from] psutil::Error),
    #[error("SemVer failed")]
    SemVer(#[from] semver::Error),
    #[error("Provided path is invalid: {0}")]
    InvalidPath(std::convert::Infallible),
    #[error("Failed to open file: {0}")]
    FailedToOpenFile(std::io::Error),
    #[error("Failed to get file metadata: {0}")]
    FailedToGetFileMetadata(std::io::Error),
    #[error("Failed to write to file: {0}")]
    FailedToWriteFile(std::io::Error),
    #[error("Failed to set file cursor: {0}")]
    FailedToSetFileCursor(std::io::Error),
    #[error("Failed to remove file: {0}")]
    FailedToRemoveFile(std::io::Error),
    #[error("Failed to convert from MessagePack")]
    FromMessagePack(#[from] rmp_serde::decode::Error),
    #[error("Failed to convert to MessagePack")]
    ToMessagePack(#[from] rmp_serde::encode::Error),
    #[error("Timeout while trying to lock {0:?}")]
    LockFileTimeout(PathBuf),
}
