#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("ort error: {0}")]
    OrtError(#[from] ort::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("image error: {0}")]
    ImageDecodeError(#[from] image::error::ImageError),
    #[error("ndarray shape error: {0}")]
    ShapeError(#[from] ndarray::ShapeError),
}
