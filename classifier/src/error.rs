use speciesnet_core::error::Error as SpeciesNetCoreError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // ORT error.
    #[error("ORT error: {0}")]
    ORTError(#[from] ort::error::Error),

    // Miscellaneous
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),
    #[error("Decode error: {0}")]
    ImageDecodeError(#[from] image::ImageError),
    #[error("Deserialize error: {0}")]
    DeserializeError(#[from] serde_json::error::Error),
    #[error("NDArray error: {0}")]
    NDArray(#[from] ndarray::ShapeError),
    #[error("SpeciesNet core error: {0}")]
    SpeciesNetCoreError(#[from] SpeciesNetCoreError),
}
