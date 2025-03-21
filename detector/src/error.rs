use image::ImageError;
use tch::TchError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("tch error: {0}")]
    TchError(#[from] TchError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Image error: {0}")]
    ImageError(#[from] ImageError),
    #[error("SpeciesNet core error: {0}")]
    CoreError(#[from] speciesnet_core::error::Error),
}
