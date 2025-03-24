use std::path::PathBuf;

use crate::Detection;

#[derive(Debug, Clone)]
pub struct Predictions(Vec<Prediction>);

#[derive(Debug, Clone)]
pub struct Prediction {
    file_path: PathBuf,
    country: Option<String>,
    admin1_region: Option<String>,
    detections: Vec<Detection>,
    prediction: Option<String>,
    prediction_score: Option<f64>,
    model_version: Option<String>,
}
