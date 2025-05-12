#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("image error: {0}")]
    ImageError(#[from] image::error::ImageError),
    #[error("Detector error: {0}")]
    DetectorError(#[from] speciesnet_detector::error::Error),
    #[error("Classifier error: {0}")]
    ClassifierError(#[from] speciesnet_classifier::error::Error),
    #[error("Ensemble error: {0}")]
    EnsembleError(#[from] speciesnet_ensemble::error::Error),
    #[error("Speciesnet core error: {0}")]
    CoreError(#[from] speciesnet_core::error::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[cfg(feature = "download-model")]
    #[error("ureq error: {0}")]
    UreqError(#[from] ureq::Error),
    #[cfg(feature = "download-model")]
    #[error("http error: response returned with status of {0}")]
    RequestFailed(u16),
    #[cfg(feature = "download-model")]
    #[error("base directory initialization failed.")]
    BaseDirInitFailed,
    #[cfg(feature = "download-model")]
    #[error("zip error: {0}")]
    ZipError(#[from] zip::result::ZipError),
    #[cfg(feature = "download-model")]
    #[error("serde_json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
}
