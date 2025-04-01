use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use serde::{Deserialize};
use speciesnet_core::{BoundingBox, Detection};
use crate::error::Error;

#[derive(Debug)]
pub struct ClassifierInput {
    pub file_path: PathBuf,
    pub bbox: BoundingBox,
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
        let folder = path.parent().unwrap_or(&Path::new("./")); // the default is root of the project
        let file = BufReader::new(File::open(*path)?);
        let detector_outputs: DetectorOutputs = serde_json::from_reader(file)?;
        
        let classifier_inputs = detector_outputs.predictions.iter().filter(|prediction| !prediction.detections.is_empty()).map(|prediction| {
            let image_path = folder.join(&prediction.file_path);
            let top_detection = prediction.detections.get(0).unwrap(); // this guaranteed to have one since passed filter
            ClassifierInput {
                file_path: image_path,
                bbox: top_detection.bounding_box().clone(),
            }
        }).collect();
        Ok(classifier_inputs)
    }
}