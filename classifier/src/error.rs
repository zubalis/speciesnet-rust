use tensorflow::Status;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // Geofence errors
    #[error("{0}")]
    GeofenceInvalidValue(String),

    // Taxonomy errors.
    #[error("Expected lable made of 7 parts, but found only {0}: {1}.")]
    InvalidLabel(String, String),
    #[error(
        "Expected these taxonomy levels: `species`, `genus`, `family`, `order`, `class`, `kingdom`, but found: {0}."
    )]
    InvalidTaxonomyLevel(String),

    // Tensorflow error.
    #[error("TF error: {0}")]
    TFError(#[from] Status),

    // Miscellaneous
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),
    #[error("Image error: {0}")]
    ImageError(#[from] fast_image_resize::ImageBufferError),
    #[error("Resize error: {0}")]
    ResizeError(#[from] fast_image_resize::ResizeError),
    #[error("Decode error: {0}")]
    ImageDecodeError(#[from] image::ImageError),
}
