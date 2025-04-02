use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use serde::Deserialize;
use speciesnet_core::classification::ClassificationBundle;
use speciesnet_core::{Detection, Instances};
use speciesnet_core::prediction::Predictions;
use crate::error::Error;
use crate::error::Error::{EmptyOutputClassifier, EmptyOutputDetector, MismatchDetectionsClassifications};

#[derive(Debug, Clone, Deserialize)]
pub struct EnsembleInput {
    file_path: PathBuf,
    country: Option<String>,
    admin1_region: Option<String>,
    classifications: Option<ClassificationBundle>,
    detections: Option<Vec<Detection>>,
}
impl EnsembleInput {
    pub fn from<P: AsRef<Path>>(instances_path: P, detector_output_path: P, classifier_output_path: P) -> Result<Vec<Self>, Error> {
        let instance_file = BufReader::new(File::open(instances_path)?);
        let instance_outputs: Instances = serde_json::from_reader(instance_file)?;

        let detector_file = BufReader::new(File::open(detector_output_path)?);
        let detector_outputs: Predictions = serde_json::from_reader(detector_file)?;

        let classifier_file = BufReader::new(File::open(classifier_output_path)?);
        let classifier_outputs: Predictions = serde_json::from_reader(classifier_file)?;

        if detector_outputs.predictions().is_empty() {
            return Err(EmptyOutputDetector);
        }

        if classifier_outputs.predictions().is_empty() {
            return Err(EmptyOutputClassifier);
        }

        if detector_outputs.predictions().len() != classifier_outputs.predictions().len() {
            return Err(MismatchDetectionsClassifications);
        }

        let mut path_map: HashMap<PathBuf, (Option<String>, Option<String>, &Option<Vec<Detection>>, &Option<ClassificationBundle>)>= HashMap::new();

        for prediction in detector_outputs.predictions() {
            let path_value = path_map
                .entry(prediction.file_path().to_path_buf())
                .or_insert((None, None, &None, &None));
            path_value.2 = prediction.detections()
        }

        for prediction in classifier_outputs.predictions() {
            let path_value = path_map
                .entry(prediction.file_path().to_path_buf())
                .or_insert((None, None, &None, &None));
            path_value.3 = prediction.classifications()
        }

        for instance in instance_outputs.instances {
            let path_value = path_map
                .entry(instance.filepath)
                .or_insert((None, None, &None, &None));
            path_value.0 = instance.country;
            path_value.1 = instance.admin1_region;
        }

        let ensemble_inputs = path_map.into_iter()
            .map(|(file_path, (country, admin1_region, detections, classifications))| {
                let detections = detections.clone();
                let classifications = classifications.clone();
                EnsembleInput { file_path, country, admin1_region, detections, classifications }
            }).collect::<Vec<_>>();
            
        Ok(ensemble_inputs)
    }
    
    pub fn file_path(&self) -> &PathBuf {
        &self.file_path
    }
    
    pub fn country(&self) -> &Option<String> {
        &self.country
    }
    
    pub fn admin1_region(&self) -> &Option<String> {
        &self.admin1_region
    }

    pub fn detections(&self) -> &Option<Vec<Detection>> {
        &self.detections
    }

    pub fn classifications(&self) -> &Option<ClassificationBundle> {
        &self.classifications
    }
}
