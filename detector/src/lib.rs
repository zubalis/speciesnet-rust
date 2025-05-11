use std::{path::Path, sync::Arc};

use image::DynamicImage;
use ndarray::Ix3;
use ort::{
    session::{Session, builder::GraphOptimizationLevel},
    value::Tensor,
};
use preprocess::{LetterboxOptions, PreprocessedImage, PreprocessedImageInner, letterbox};
use speciesnet_core::{
    detector::{BoundingBox, Category, Detection},
    io::Prediction,
};
use tracing::info;
use yolo::non_max_suppression;

use crate::error::Error;

pub mod error;
pub mod preprocess;
pub mod torchvision;
pub mod yolo;

#[derive(Debug, Clone)]
pub struct SpeciesNetDetector {
    model: Arc<Session>,
}

impl SpeciesNetDetector {
    pub fn new<P>(model_path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let cpus = num_cpus::get();

        let model = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(cpus)?
            .commit_from_file(model_path)?;

        Ok(Self {
            model: Arc::new(model),
        })
    }

    pub fn preprocess(
        &self,
        image: DynamicImage,
        options: LetterboxOptions,
    ) -> Result<PreprocessedImageInner, Error> {
        let preprocessed_image = letterbox(image, options)?;
        Ok(preprocessed_image)
    }

    pub fn predict(
        &self,
        preprocessed_image: PreprocessedImage,
    ) -> Result<Option<Prediction>, Error> {
        let (original_width, original_height) = preprocessed_image.original_size();
        let (resized_width, resized_height) = preprocessed_image.resized_size();
        let path = preprocessed_image.path_owned();
        let tensor = preprocessed_image.into_tensor();

        info!("Running predictions on image {}.", path.display());
        let outputs = self
            .model
            .run(ort::inputs!["images" => Tensor::from_array(tensor)?]?)?;

        let output = outputs
            .get("output")
            .unwrap()
            .try_extract_tensor::<f32>()?
            .into_dimensionality::<Ix3>()?
            .into_owned();

        info!("Running non-max suppression on image {}.", path.display());
        let nms_results = non_max_suppression(output, Some(0.01))?;

        if nms_results.is_empty() {
            return Ok(None);
        }

        let mut detections: Vec<Detection> = Vec::new();

        for raw_detection in nms_results.rows() {
            let x1: f64 = f64::from(raw_detection[0]);
            let y1: f64 = f64::from(raw_detection[1]);
            let x2: f64 = f64::from(raw_detection[2]);
            let y2: f64 = f64::from(raw_detection[3]);

            let confidence = raw_detection[4];
            let category = Category::try_from(raw_detection[5].trunc() as i32 + 1).unwrap();

            let bbox = BoundingBox::new(x1, y1, x2, y2)
                .scale_to(
                    resized_width,
                    resized_height,
                    original_width,
                    original_height,
                )
                .normalize(original_width, original_height);

            detections.push(Detection::new(category, confidence.into(), bbox));
        }

        Ok(Some(Prediction::from_detections(path, detections)))
    }
}
