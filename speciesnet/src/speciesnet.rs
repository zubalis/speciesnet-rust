use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use ort::execution_providers::{
    CPUExecutionProvider, CUDAExecutionProvider, CoreMLExecutionProvider, ExecutionProvider,
    ExecutionProviderDispatch,
};
use rayon::prelude::*;
use speciesnet_classifier::{
    SpeciesNetClassifier,
    classifier::{read_labels_from_file, transform},
    image::preprocess as classifier_preprocess,
    input::ClassifierInput,
};
use speciesnet_core::{
    detector::BoundingBox,
    io::{Instance, Prediction},
    load_image,
    shape::Shape,
};
use speciesnet_detector::{
    SpeciesNetDetector,
    preprocess::{LetterboxOptions, PreprocessedImage},
};
use speciesnet_ensemble::{
    SpeciesNetEnsemble, error::Error::NoneDetectionOrClassification, input::EnsembleInput,
};
use tracing::{debug, info, warn};

use crate::{error::Error, execution_info::ExecutionInfo, model_info::ModelInfo};

#[derive(Debug, Default, Clone)]
pub enum ModelInfoBuildMethod {
    Folder(PathBuf),
    #[default]
    DownloadedModel,
}

#[derive(Debug, Clone)]
pub struct SpeciesNetBuilder {
    build_method: ModelInfoBuildMethod,
    coreml_ep_config: Option<CoreMLExecutionProvider>,
    cuda_ep_config: Option<CUDAExecutionProvider>,
}

impl SpeciesNetBuilder {
    fn new(method: ModelInfoBuildMethod) -> Self {
        Self {
            build_method: method,
            coreml_ep_config: None,
            cuda_ep_config: None,
        }
    }

    pub fn with_coreml(mut self, provider: CoreMLExecutionProvider) -> Self {
        self.coreml_ep_config = Some(provider);
        self
    }

    pub fn with_cuda(mut self, provider: CUDAExecutionProvider) -> Self {
        self.cuda_ep_config = Some(provider);
        self
    }

    pub fn build(self) -> Result<SpeciesNet, Error> {
        let mut execution_providers: Vec<ExecutionProviderDispatch> =
            vec![CPUExecutionProvider::default().into()];
        let mut coreml_enabled = false;
        let mut cuda_enabled = false;

        if let Some(coreml_ep_config) = &self.coreml_ep_config {
            if coreml_ep_config.is_available().unwrap_or(false) {
                info!("CoreML execution provider is configured and available.");
                execution_providers.insert(0, coreml_ep_config.clone().into());
                coreml_enabled = true;
            } else {
                warn!("CoreML execution provider is configured but not available.");
            }
        }

        if let Some(cuda_ep_config) = &self.cuda_ep_config {
            if cuda_ep_config.is_available().unwrap_or(false) {
                info!("CUDA execution provider is configured and available.");
                execution_providers.insert(0, cuda_ep_config.clone().into());
                cuda_enabled = true;
            } else {
                warn!("CUDA execution provider is configured but not available.");
            }
        }

        let execution_info = ExecutionInfo::new(coreml_enabled, cuda_enabled);

        let model_info = match &self.build_method {
            #[cfg(feature = "download-model")]
            ModelInfoBuildMethod::DownloadedModel => ModelInfo::from_default_url()?,

            ModelInfoBuildMethod::Folder(f) => ModelInfo::from_path(f)?,
        };

        SpeciesNet::from_model_info_with_execution_providers(
            model_info,
            execution_info,
            execution_providers,
        )
    }
}

#[derive(Debug, Clone)]
pub struct SpeciesNet {
    execution_info: ExecutionInfo,
    model_info: ModelInfo,
    detector: SpeciesNetDetector,
    classifier: SpeciesNetClassifier,
    classifier_labels: Vec<String>,
    ensemble: SpeciesNetEnsemble,
}

impl SpeciesNet {
    #[cfg(feature = "download-model")]
    pub fn from_downloaded_model() -> SpeciesNetBuilder {
        SpeciesNetBuilder::new(ModelInfoBuildMethod::DownloadedModel)
    }

    pub fn from_model_folder<P: AsRef<Path>>(model_folder: P) -> SpeciesNetBuilder {
        SpeciesNetBuilder::new(ModelInfoBuildMethod::Folder(
            model_folder.as_ref().to_path_buf(),
        ))
    }

    pub fn from_model_info_with_execution_providers(
        model_info: ModelInfo,
        execution_info: ExecutionInfo,
        execution_providers: Vec<ExecutionProviderDispatch>,
    ) -> Result<Self, Error> {
        let classifier = SpeciesNetClassifier::with_execution_providers(
            model_info.classifier(),
            execution_providers.clone(),
        )?;
        info!("Classifier initialized.");

        let classifier_labels: Vec<String> = read_labels_from_file(model_info.classifier_labels())?;
        info!("Classifier labels loaded.");

        let detector = SpeciesNetDetector::with_execution_providers(
            model_info.detector(),
            execution_providers,
        )?;
        info!("Detector initialized.");

        let ensemble = SpeciesNetEnsemble::new(model_info.geofence(), model_info.taxonomy(), None)?;
        info!("Ensemble initialized.");

        Ok(Self {
            execution_info,
            model_info,
            classifier,
            classifier_labels,
            detector,
            ensemble,
        })
    }

    // Get information on how the model is executed (e.g. whether gpu will be used)
    pub fn execution_info(&self) -> &ExecutionInfo {
        &self.execution_info
    }

    /// Performs the detection by MegaDetector Model from given file or folder. Returns a list of
    /// detections.
    pub fn detect(&self, instances: &[Instance]) -> Vec<Result<Prediction, Error>> {
        info!("Starting detection");

        let image_format_options: Arc<LetterboxOptions> = Arc::new(
            LetterboxOptions::builder()
                .shape(Shape::Square(1280))
                .build(),
        );

        instances
            .par_iter()
            .map(|fp| {
                let loaded_image = load_image(fp.file_path())?;
                let preprocessed_image = self
                    .detector
                    .preprocess(loaded_image.into(), *image_format_options)?;
                let preprocessed_image = PreprocessedImage::new(preprocessed_image, fp.file_path());

                match self.detector.predict(preprocessed_image) {
                    Ok(prediction) => Ok(prediction),
                    Err(err) => {
                        warn!("Detector failed for {:?}: {}", fp.file_path(), err);
                        Err(Error::DetectorError(err))
                    }
                }
            })
            .collect()
    }

    /// Performs the classification from detector output by the cameratrap model.
    pub fn classify(
        &self,
        detector_output_path: &PathBuf,
    ) -> Result<Vec<Result<Prediction, Error>>, Error> {
        info!("Starting classification");

        let classifier_inputs = ClassifierInput::from_detector_output(detector_output_path)?;

        let predictions = classifier_inputs
            .par_iter()
            .map(|fp| {
                let image = classifier_preprocess(fp)?;
                let tensor = image.image_tensor;
                let image_path = image.path;
                let outputs = self.classifier.classify(tensor)?;

                // Transform outputs into usable format (softmax, mapping labels, pick top 5)
                let prediction = transform(image_path, outputs.view(), &self.classifier_labels);
                Ok::<Prediction, Error>(prediction)
            })
            .collect::<Vec<_>>();

        debug!("Finished classification");
        Ok(predictions)
    }

    /// Performs the ensemble
    pub fn ensemble(
        &self,
        instances_path: &PathBuf,
        detector_output_path: &PathBuf,
        classifier_output_path: &PathBuf,
    ) -> Result<Vec<Result<Prediction, Error>>, Error> {
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
            .collect::<Vec<_>>();

        debug!("Finished ensemble");
        Ok(predictions)
    }

    /// Performs the whole pipeline (Detection, Classification, Ensemble) from given list of
    /// instances.
    pub fn predict(&self, instances: &[Instance]) -> Vec<Result<Prediction, Error>> {
        info!("Starting prediction (detection + classification + ensemble)");

        let letterbox_options: Arc<LetterboxOptions> = Arc::new(
            LetterboxOptions::builder()
                .shape(Shape::Square(1280))
                .build(),
        );

        instances
            .par_iter()
            .map(|fp| {
                let mut prediction = Prediction::new(fp.file_path().to_path_buf());

                // Loading the image
                let loaded_image = match load_image(fp.file_path()) {
                    Ok(image) => image,
                    Err(e) => {
                        warn!("Image {} failed to load {}", fp.file_path().display(), e);
                        return Err(e.into());
                    }
                };

                // Running the detector
                let detector_image = self
                    .detector
                    .preprocess(loaded_image.clone().into(), *letterbox_options)?;
                let detector_image = PreprocessedImage::new(detector_image, fp.file_path());

                let detector_results = self.detector.predict(detector_image)?;
                let bounding_boxes = match detector_results.detections() {
                    Some(det) => det
                        .iter()
                        .map(|d| *d.bounding_box())
                        .collect::<Vec<BoundingBox>>(),
                    None => vec![],
                };

                prediction.merge(detector_results);

                // Running the classifier
                let classifier_tensor = self
                    .classifier
                    .preprocess(loaded_image.into(), &bounding_boxes)?;

                let classifier_results = self.classifier.classify(classifier_tensor)?;
                let classifier_results = transform(
                    fp.file_path(),
                    classifier_results.view(),
                    &self.classifier_labels,
                );

                prediction.merge(classifier_results);

                // Running the ensemble
                if let (Some(detections), Some(classifications)) =
                    (prediction.detections(), prediction.classifications())
                {
                    let ensemble_results = self.ensemble.ensemble(
                        detections,
                        classifications,
                        fp.country().map(str::to_string),
                        fp.admin1_region().map(str::to_string),
                    )?;

                    let ensemble_prediction = Prediction::from_ensemble(
                        fp.file_path().to_path_buf(),
                        fp.country().map(str::to_string),
                        fp.admin1_region().map(str::to_string),
                        ensemble_results.clone(),
                        detections.clone(),
                        classifications.clone(),
                    );

                    prediction.set_model_version(Some(self.model_info.version().to_string()));
                    prediction.merge(ensemble_prediction);
                }

                Ok(prediction)
            })
            .collect()
    }
}
