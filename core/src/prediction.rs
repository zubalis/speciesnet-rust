use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::Detection;

/// The output type of `predictions.json` file.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Predictions {
    predictions: Vec<Prediction>,
}

impl From<Vec<Prediction>> for Predictions {
    fn from(value: Vec<Prediction>) -> Self {
        Predictions { predictions: value }
    }
}

impl Predictions {
    pub fn new(predictions: Vec<Prediction>) -> Self {
        Self { predictions }
    }
}

/// The possible output of each predictions found during the run.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Prediction {
    file_path: PathBuf,
    country: Option<String>,
    admin1_region: Option<String>,
    detections: Option<Vec<Detection>>,
    prediction: Option<String>,
    prediction_score: Option<f64>,
    model_version: Option<String>,
}

impl Prediction {
    pub fn from_detections(file_path: PathBuf, detections: Vec<Detection>) -> Self {
        Self {
            file_path,
            country: None,
            admin1_region: None,
            detections: Some(detections),
            prediction: None,
            prediction_score: None,
            model_version: None,
        }
    }
}
