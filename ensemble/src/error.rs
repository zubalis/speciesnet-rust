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

    // Ensemble errors
    #[error("`detections` cannot be empty")]
    EmptyDetections,
    #[error("`classifications` cannot be empty")]
    EmptyClassifications,
    #[error("`output_detector` cannot be empty")]
    EmptyOutputDetector,
    #[error("`output_classifier` cannot be empty")]
    EmptyOutputClassifier,
    #[error("missing `detections` or `classifications` in `output_detector.json`")]
    NoneDetectionOrClassification,
    #[error("`detections` and `classifications` size are not equal")]
    MismatchDetectionsClassifications,

    // Miscellaneous
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),
    #[error("Deserialize error: {0}")]
    DeserializeError(#[from] serde_json::error::Error),
}
