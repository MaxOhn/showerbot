#[derive(Debug, thiserror::Error)]
pub enum PpError {
    #[error("io error")]
    IoError(#[from] tokio::io::Error),
    #[error("failed to prepare beatmap file")]
    MapFile(#[from] crate::error::MapFileError),
}
