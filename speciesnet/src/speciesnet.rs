use std::path::{Path, PathBuf};

use rayon::prelude::*;
use speciesnet_classifier::SpeciesNetClassifier;
use speciesnet_classifier::classifier::{read_labels_from_file, transform};
use speciesnet_classifier::image::preprocess as classifier_preprocess;
use speciesnet_classifier::input::ClassifierInput;
use speciesnet_core::prediction::Prediction;
use speciesnet_detector::{SpeciesNetDetectorOrt, preprocess::preprocess};
use tracing::{debug, error, info};

use crate::error::Error;

#[derive(Debug, Clone)]
pub struct SpeciesNet {
    detector_ort: SpeciesNetDetectorOrt,
    classifier: SpeciesNetClassifier,
}

impl SpeciesNet {
    /// Initialize the detector and the classifier by loading them into memory.
    pub fn new<P>(detector_model_path: P, classifier_model_dir_path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let classifier = SpeciesNetClassifier::new(classifier_model_dir_path)?;
        info!("Classifier initialized.");

        let detector_ort = SpeciesNetDetectorOrt::new(detector_model_path)?;
        info!("Detector ort initialized.");

        Ok(Self {
            classifier,
            detector_ort,
        })
    }

    /// Performs the detection by MegaDetector Model from given file or folder. Returns a list of
    /// detections.
    pub fn detect(&self, list_of_files: &[PathBuf]) -> Result<Vec<Prediction>, Error> {
        info!("Starting the detector ort step.");

        let detections = list_of_files
            .iter()
            .map(|fp| {
                let preprocessed_image = match preprocess(fp) {
                    Ok(pi) => pi,
                    Err(e) => {
                        error!("{}", e);
                        return None;
                    }
                };

                match self.detector_ort.predict(preprocessed_image) {
                    Ok(d) => d,
                    Err(e) => {
                        error!("{}", e);
                        None
                    }
                }
            })
            .collect::<Vec<Option<Prediction>>>();

        Ok(detections
            .into_iter()
            .flatten()
            .collect::<Vec<Prediction>>())
    }

    /// Performs the classification from detector output by the cameratrap model.
    pub fn classify(
        &self,
        detector_output_path: &PathBuf,
        label_path: PathBuf,
    ) -> Result<Vec<Prediction>, Error> {
        let classifier_inputs = ClassifierInput::from_detector_output(detector_output_path)?;
        // Load labels
        let labels: Vec<String> = read_labels_from_file(&label_path)?;
        let predictions = classifier_inputs
            .par_iter()
            .map(|fp| {
                let image = classifier_preprocess(fp)?;
                let tensor = image.image_tensor;
                let image_path = image.path;
                let outputs = self.classifier.classify(&tensor)?;
                // Transform outputs into usable format (softmax, mapping labels, pick top 5)
                let prediction = transform(image_path, &outputs, &labels);
                Ok(prediction)
            })
            .collect::<Result<Vec<Prediction>, Error>>()?;

        debug!("Finished classification");
        Ok(predictions)
    }

    /// Performs both detection by MegaDetector and classify by the cameratrap model.
    pub fn predict(&self) {
        todo!()
    }
}
