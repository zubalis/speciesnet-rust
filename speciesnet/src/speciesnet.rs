use std::path::{Path, PathBuf};

use log::debug;
use rayon::prelude::*;
use speciesnet_core::prediction::Prediction;
use speciesnet_detector::{SpeciesNetDetector, preprocess::preprocess};

use crate::error::Error;

#[derive(Debug, Clone)]
pub struct SpeciesNet {
    detector: SpeciesNetDetector,
}

impl SpeciesNet {
    /// Initialize the detector and the classifier by loading them into memory.
    pub fn new<P>(detector_model_path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let detector = SpeciesNetDetector::new(detector_model_path)?;
        debug!("Detector initialized.");

        Ok(Self { detector })
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

    /// Performs the classification by the cameratrap model.
    pub fn classify(&self) {
        todo!()
    }

    /// Performs both detection by MegaDetector and classify by the cameratrap model.
    pub fn predict(&self) {
        todo!()
    }
}
