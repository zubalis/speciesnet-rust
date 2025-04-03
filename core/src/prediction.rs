use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::Detection;
use crate::classification::ClassificationBundle;
use crate::geofence::GeofenceResult;

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

    pub fn predictions(&self) -> &[Prediction] {
        &self.predictions
    }
}

/// The possible output of each predictions found during the run.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Prediction {
    #[serde(rename = "filepath")]
    file_path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    admin1_region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    detections: Option<Vec<Detection>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    classifications: Option<ClassificationBundle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prediction: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prediction_score: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prediction_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
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
            prediction_source: None,
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
            prediction_source: None,
            model_version: None,
        }
    }

    pub fn ensemble(
        file_path: PathBuf,
        country: Option<String>,
        admin1_region: Option<String>,
        geofence_result: GeofenceResult,
        detections: Vec<Detection>,
        classifications: ClassificationBundle,
    ) -> Self {
        Self {
            file_path,
            country,
            admin1_region,
            detections: Some(detections),
            classifications: Some(classifications),
            prediction: Some(geofence_result.label().to_string()),
            prediction_score: Some(geofence_result.score()),
            prediction_source: Some(geofence_result.source().to_string()),
            model_version: None,
        }
    }

    pub fn detections(&self) -> &Option<Vec<Detection>> {
        &self.detections
    }

    pub fn classifications(&self) -> &Option<ClassificationBundle> {
        &self.classifications
    }

    pub fn file_path(&self) -> &PathBuf {
        &self.file_path
    }
}
