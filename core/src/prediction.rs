use std::path::PathBuf;

use serde::Deserialize;

use crate::Detection;

/// The output type of `predictions.json` file.
#[derive(Debug, Clone)]
pub struct Predictions {
    predictions: Vec<Prediction>,
}

/// The possible output of each predictions found during the run.
#[derive(Debug, Clone, Deserialize)]
pub struct Prediction {
    file_path: PathBuf,
    country: Option<String>,
    admin1_region: Option<String>,
    detections: Vec<Detection>,
    prediction: Option<String>,
    prediction_score: Option<f64>,
    model_version: Option<String>,
}
