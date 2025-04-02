#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Detector error: {0}")]
    DetectorError(#[from] speciesnet_detector::error::Error),
    #[error("Classifier error: {0}")]
    ClassifierError(#[from] speciesnet_classifier::error::Error),
    #[error("Ensemble error: {0}")]
    EnsembleError(#[from] speciesnet_ensemble::error::Error),
    #[error("Speciesnet detector error: {0}")]
    CoreError(#[from] speciesnet_core::error::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
