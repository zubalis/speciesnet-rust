use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Deserialize;
use speciesnet_core::{BoundingBox, Detection};

use crate::error::Error;

#[derive(Debug)]
pub struct ClassifierInput {
    pub file_path: PathBuf,
    pub bbox: Option<BoundingBox>,
}

#[derive(Debug, Clone, Deserialize)]
struct DetectorOutputs {
    pub predictions: Vec<DetectorOutput>,
}

#[derive(Debug, Clone, Deserialize)]
struct DetectorOutput {
    #[serde(rename = "filepath")]
    pub file_path: String,
    pub detections: Vec<Detection>,
}

impl ClassifierInput {
    pub fn from_detector_output<P: AsRef<Path>>(path: P) -> Result<Vec<ClassifierInput>, Error> {
        let path = Arc::new(path.as_ref());
        let file = BufReader::new(File::open(*path)?);
        let detector_outputs: DetectorOutputs = serde_json::from_reader(file)?;
        
        let classifier_inputs = detector_outputs.predictions.iter().map(|prediction| {
            if let Some(detection) = prediction.detections.first() {
                ClassifierInput {
                    file_path: PathBuf::from(&prediction.file_path),
                    bbox: Some(*detection.bounding_box()),
                }
            } else {
                ClassifierInput {
                    file_path: PathBuf::from(&prediction.file_path),
                    bbox: None,
                }
            }
        }).collect();
        Ok(classifier_inputs)
    }
}
