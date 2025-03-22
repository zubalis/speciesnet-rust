use serde::{Deserialize, Serialize, de};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Deserialize, Debug)]
pub struct InstanceList {
    instances: Vec<Instance>,
}

#[derive(Deserialize, Debug)]
pub struct Instance {
    filepath: String,
    country: String,
    admin1_region: Option<String>,
}

impl InstanceList {
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let instance_list = serde_json::from_reader(reader)?;
        Ok(instance_list)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PredictionList {
    predictions: Vec<Prediction>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Prediction {
    filepath: String,
    country: Option<String>,
    admin1_region: Option<String>,
    detections: Option<Vec<Detection>>,
    classifications: Option<ClassificationList>,
    prediction: Option<String>,
    prediction_score: Option<f32>,
    prediction_source: Option<String>,
    model_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Detection {
    category: String,
    label: String,
    conf: f32,
    bbox: Vec<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClassificationList {
    classes: Vec<String>,
    scores: Vec<f32>,
}

impl PredictionList {
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let prediction_list = serde_json::from_reader(reader)?;
        Ok(prediction_list)
    }
}
