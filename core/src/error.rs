#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid tensor size, expected 4, found {0}.")]
    InvalidTensorSize(i32),
    #[error("Negative coordinates, width, and height are not allowed.")]
    NegativeCoordinate,
    #[error(
        "Category index out of range, expected passed category to be `1`, `2`, or `3`, received {0}"
    )]
    CategoryIndexOutOfRange(f64),
    #[error("Failed to parse value {0} to Category.")]
    CategoryParseError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Image error: {0}")]
    ImageError(#[from] image::ImageError),
}
