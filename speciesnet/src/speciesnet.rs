use std::path::{Path, PathBuf};
use log::{debug, info};
use rayon::prelude::*;
use speciesnet_classifier::classifier::{read_labels_from_file, transform};
use speciesnet_classifier::image::preprocess as classifier_preprocess;
use speciesnet_classifier::input::ClassifierInput;
use speciesnet_classifier::SpeciesNetClassifier;
use speciesnet_core::prediction::Prediction;
use speciesnet_detector::{SpeciesNetDetector, preprocess::preprocess};

use crate::error::Error;

#[derive(Debug, Clone)]
pub struct SpeciesNet {
    detector: SpeciesNetDetector,
    classifier: SpeciesNetClassifier,
}

impl SpeciesNet {
    /// Initialize the detector and the classifier by loading them into memory.
    pub fn new<P>(detector_model_path: P, classifier_model_dir_path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let detector = SpeciesNetDetector::new(detector_model_path)?;
        info!("Detector initialized.");
        let classifier = SpeciesNetClassifier::new(classifier_model_dir_path)?;
        info!("Classifier initialized.");

        Ok(Self {
            detector,
            classifier,
        })
    }

    /// Performs the detection by MegaDetector Model from given file or folder. Returns a list of
    /// detections.
    pub fn detect(&self, list_of_files: &[PathBuf]) -> Result<Vec<Prediction>, Error> {
        debug!("Starting the rayon multithread for files.");

        let detections = list_of_files
            .par_iter()
            .map(|fp| {
                let preprocessed_image = preprocess(fp)?;
                let detections = self.detector.predict(preprocessed_image)?;

                Ok(detections)
            })
            .collect::<Result<Vec<Option<Prediction>>, Error>>()?;

        Ok(detections
            .into_iter()
            .flatten()
            .collect::<Vec<Prediction>>())
    }

    /// Performs the classification from detector output by the cameratrap model.
    pub fn classify(&self, detector_output_path: &PathBuf, label_path: PathBuf) -> Result<Vec<Prediction>, Error> {
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
            }).collect::<Result<Vec<Prediction>, Error>>();
        debug!("Finished classification");
        Ok(predictions?)
    }

    /// Performs both detection by MegaDetector and classify by the cameratrap model.
    pub fn predict(&self) {
        todo!()
    }
}
