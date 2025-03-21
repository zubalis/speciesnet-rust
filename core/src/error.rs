use tch::TchError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid tensor size, expected 4, found {0}.")]
    InvalidTensorSize(i64),
    #[error("Negative coordinates, width, and height are not allowed.")]
    NegativeCoordinate,
    #[error("tch error: {0}")]
    TchError(#[from] TchError),
    #[error(
        "Category index out of range, expected passed category to be within (1..=3), received {0}"
    )]
    CategoryIndexOutOfRange(f64),
}
