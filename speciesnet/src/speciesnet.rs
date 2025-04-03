use std::path::{Path, PathBuf};

use rayon::prelude::*;
use speciesnet_classifier::SpeciesNetClassifier;
use speciesnet_classifier::classifier::{read_labels_from_file, transform};
use speciesnet_classifier::image::preprocess as classifier_preprocess;
use speciesnet_classifier::input::ClassifierInput;
use speciesnet_core::prediction::Prediction;
use speciesnet_detector::{SpeciesNetDetector, preprocess::preprocess};
use speciesnet_ensemble::SpeciesNetEnsemble;
use speciesnet_ensemble::error::Error::NoneDetectionOrClassification;
use speciesnet_ensemble::input::EnsembleInput;
use tracing::{debug, error, info};

use crate::error::Error;

#[derive(Debug, Clone)]
pub struct SpeciesNet {
    detector: SpeciesNetDetector,
    classifier: SpeciesNetClassifier,
    ensemble: SpeciesNetEnsemble,
}

impl SpeciesNet {
    /// Initialize the detector and the classifier by loading them into memory.
    pub fn new<P>(
        detector_model_path: P,
        classifier_model_dir_path: P,
        geofence_base_path: P,
        geofence_fix_path: P,
        taxonomy_path: P,
    ) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let classifier = SpeciesNetClassifier::new(classifier_model_dir_path)?;
        info!("Classifier initialized.");

        let detector = SpeciesNetDetector::new(detector_model_path)?;
        info!("Detector ort initialized.");

        let ensemble =
            SpeciesNetEnsemble::new(geofence_base_path, geofence_fix_path, taxonomy_path)?;
        info!("Ensemble initialized.");

        Ok(Self {
            classifier,
            detector,
            ensemble,
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

                match self.detector.predict(preprocessed_image) {
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
        label_path: &PathBuf,
    ) -> Result<Vec<Prediction>, Error> {
        info!("Starting classification");
        let classifier_inputs = ClassifierInput::from_detector_output(detector_output_path)?;
        // Load labels
        let labels: Vec<String> = read_labels_from_file(label_path)?;
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

    /// Performs the ensemble
    pub fn ensemble(
        &self,
        instances_path: &PathBuf,
        detector_output_path: &PathBuf,
        classifier_output_path: &PathBuf,
    ) -> Result<Vec<Prediction>, Error> {
        info!("Starting ensemble");
        let ensemble_inputs =
            EnsembleInput::from(instances_path, detector_output_path, classifier_output_path)?;
        let predictions = ensemble_inputs
            .par_iter()
            .map(|input| {
                if let (Some(detections), Some(classification)) =
                    (input.detections(), input.classifications())
                {
                    let geofence_result = &self.ensemble.ensemble(
                        detections,
                        classification,
                        input.country().clone(),
                        input.admin1_region().clone(),
                    )?;
                    Ok(Prediction::ensemble(
                        input.file_path().clone(),
                        input.country().clone(),
                        input.admin1_region().clone(),
                        geofence_result.clone(),
                        detections.clone(),
                        classification.clone(),
                    ))
                } else {
                    Err(NoneDetectionOrClassification.into())
                }
            })
            .collect::<Result<Vec<Prediction>, Error>>()?;

        Ok(predictions)
    }

    /// Performs both detection by MegaDetector and classify by the cameratrap model.
    pub fn predict(&self) {
        todo!()
    }
}
