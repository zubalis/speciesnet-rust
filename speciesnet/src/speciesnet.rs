use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use image::ImageReader;
use rayon::prelude::*;
use speciesnet_classifier::{
    SpeciesNetClassifier,
    classifier::{read_labels_from_file, transform},
    image::preprocess as classifier_preprocess,
    input::ClassifierInput,
};
use speciesnet_core::{BoundingBox, Instance, prediction::Prediction, shape::Shape};
use speciesnet_detector::{
    SpeciesNetDetector,
    preprocess::{LetterboxOptions, PreprocessedImage},
};
use speciesnet_ensemble::{
    SpeciesNetEnsemble, error::Error::NoneDetectionOrClassification, input::EnsembleInput,
};
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
    pub fn detect(&self, instances: &[Instance]) -> Result<Vec<Prediction>, Error> {
        info!("Starting the detector ort step.");

        let image_format_options: Arc<LetterboxOptions> = Arc::new(
            LetterboxOptions::builder()
                .shape(Shape::Square(1280))
                .build(),
        );

        let detections = instances
            .par_iter()
            .map(|fp| {
                let loaded_image = ImageReader::open(&fp.filepath)?.decode()?;
                let preprocessed_image = self
                    .detector
                    .preprocess(loaded_image, *image_format_options)?;
                let preprocessed_image =
                    PreprocessedImage::new(preprocessed_image, fp.filepath.clone());

                let predictions = self.detector.predict(preprocessed_image)?;

                Ok(predictions)
            })
            .collect::<Result<Vec<Option<Prediction>>, Error>>()?;

        Ok(detections.into_iter().flatten().collect())
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

                    Ok(Prediction::from_ensemble(
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

    /// Performs the whole pipeline (Detection, Classification, Ensemble) from given list of
    /// instances.
    pub fn predict(
        &self,
        instances: &[Instance],
        labels_path: &Path,
    ) -> Result<Vec<Prediction>, Error> {
        info!("Starting the predictions on the whole pipeline.");

        // loads the image, this will gets converted to both detector input and classifier so they
        // need to stay.

        let letterbox_options: Arc<LetterboxOptions> = Arc::new(
            LetterboxOptions::builder()
                .shape(Shape::Square(1280))
                .build(),
        );

        let labels = read_labels_from_file(labels_path)?;

        let predictions = instances
            .par_iter()
            .map(|fp| {
                let mut prediction = Prediction::new(fp.filepath.clone());

                // Loading the image
                let loaded_image = match ImageReader::open(&fp.filepath)?.decode() {
                    Ok(image) => image,
                    Err(e) => {
                        error!("image failed to load {}", e);
                        return Ok(prediction);
                    }
                };

                // Running the detector
                let detector_image = self
                    .detector
                    .preprocess(loaded_image.clone(), *letterbox_options)?;
                let detector_image = PreprocessedImage::new(detector_image, fp.filepath.clone());

                let detector_results = self.detector.predict(detector_image)?;

                if let Some(ref res) = detector_results {
                    prediction.merge(res.clone());
                }

                let bounding_boxes = match detector_results {
                    Some(detections) => match detections.detections() {
                        Some(det) => {
                            let binding = det
                                .iter()
                                .map(|d| *d.bounding_box())
                                .collect::<Vec<BoundingBox>>();

                            binding
                        }
                        None => vec![],
                    },
                    None => vec![],
                };

                // Running the classifier
                let classifier_tensor =
                    self.classifier.preprocess(loaded_image, &bounding_boxes)?;

                let classifier_results = self.classifier.classify(&classifier_tensor)?;
                let classifier_results = transform(&fp.filepath, &classifier_results, &labels);

                prediction.merge(classifier_results);

                // Running the emsembler
                if let (Some(detections), Some(classifications)) =
                    (prediction.detections(), prediction.classifications())
                {
                    let ensemble_results = self.ensemble.ensemble(
                        detections,
                        classifications,
                        fp.country.clone(),
                        fp.admin1_region.clone(),
                    )?;

                    let ensemble_prediction = Prediction::from_ensemble(
                        fp.filepath.clone(),
                        fp.country.clone(),
                        fp.admin1_region.clone(),
                        ensemble_results.clone(),
                        detections.clone(),
                        classifications.clone(),
                    );

                    prediction.merge(ensemble_prediction);
                }

                Ok(prediction)
            })
            .collect::<Result<Vec<Prediction>, Error>>()?;

        info!("Finished running the whole flow.");
        Ok(predictions)
    }
}
