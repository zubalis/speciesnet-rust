use std::{fs::read_dir, path::Path};

use log::debug;
use rayon::prelude::*;
use speciesnet_core::Detection;
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
    pub fn detect<P>(&self, file_or_folder: P) -> Result<Vec<Detection>, Error>
    where
        P: AsRef<Path>,
    {
        // TODO: File filtration for image only.
        let file_paths = if file_or_folder.as_ref().is_dir() {
            debug!("Gathering files inside folder");
            read_dir(&file_or_folder)?
                .map(|e| {
                    let entry = e?;
                    let path = entry.path();

                    Ok(path)
                })
                .collect::<Result<Vec<_>, Error>>()
        } else {
            Ok(vec![file_or_folder.as_ref().to_path_buf()])
        };

        debug!("Starting the rayon multithread for files.");
        let file_paths = file_paths?;

        let detections = file_paths
            .par_iter()
            .map(|fp| {
                let preprocessed_image = preprocess(fp)?;
                let detections = self.detector.predict(preprocessed_image)?;

                Ok(detections)
            })
            .collect::<Result<Vec<Vec<_>>, Error>>()?;

        Ok(detections.into_iter().flatten().collect::<Vec<Detection>>())
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
