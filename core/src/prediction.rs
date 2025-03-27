use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::Detection;
use crate::classification::ClassificationBundle;

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
    #[serde(rename = "filepath")]
    file_path: PathBuf,
    country: Option<String>,
    admin1_region: Option<String>,
    detections: Option<Vec<Detection>>,
    classifications: Option<ClassificationBundle>,
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
            classifications: None,
            prediction: None,
            prediction_score: None,
            model_version: None,
        }
    }
    
    pub fn from_classifications(file_path: PathBuf, classifications: ClassificationBundle) -> Self {
        Self {
            file_path,
            country: None,
            admin1_region: None,
            detections: None,
            classifications: Some(classifications),
            prediction: None,
            prediction_score: None,
            model_version: None,
        }
    }

    pub fn detections(&self) -> &Option<Vec<Detection>> {
        &self.detections
    }

    pub fn classifications(&self) -> &Option<ClassificationBundle> {
        &self.classifications
    }
}
