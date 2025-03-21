#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Speciesnet core error: {0}")]
    DetectorError(#[from] speciesnet_detector::error::Error),
    #[error("Speciesnet detector error: {0}")]
    CoreError(#[from] speciesnet_core::error::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
