use std::path::Path;
use std::sync::Arc;

use ::image::DynamicImage;
use ndarray::{Array1, Array4, Ix2};
use ort::session::Session;
use ort::session::builder::GraphOptimizationLevel;
use ort::value::Tensor;

use speciesnet_core::BoundingBox;

pub mod classifier;
pub mod error;
pub mod image;
pub mod input;

use crate::{error::Error, image::preprocess_impl};

#[derive(Debug, Clone)]
pub struct SpeciesNetClassifier {
    model: Arc<Session>,
}

impl SpeciesNetClassifier {
    /// Create classifier from given config
    pub fn new<P>(model_path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let cpus = num_cpus::get();
        let session = Session::builder()?
            .with_optimization_level(GraphOptimizationLevel::Level3)?
            .with_intra_threads(cpus)?
            .commit_from_file(model_path)?;

        Ok(Self {
            model: Arc::new(session),
        })
    }

    /// run a classification from given input
    pub fn classify(&self, input_tensor: Array4<f32>) -> Result<Array1<f32>, Error> {
        let outputs = self
            .model
            .run(ort::inputs!["input" => Tensor::from_array(input_tensor)?]?)?;
        let output: Array1<f32> = outputs
            .get("dense")
            .unwrap()
            .try_extract_tensor::<f32>()?
            .into_dimensionality::<Ix2>()?
            .row(0) // to get the scores
            .into_owned();
        Ok(output)
    }

    /// Preprocess a given image to be classifier compatible format.
    pub fn preprocess(
        &self,
        image: DynamicImage,
        bboxes: &[BoundingBox],
    ) -> Result<Array4<f32>, Error> {
        let processed_image = preprocess_impl(image, bboxes.first().cloned())?;

        // The tensor's structure for classifier is batch, width, height, and channel.
        let mut tensor = Array4::zeros([1usize, 480usize, 480usize, 3usize]);

        for pixel in processed_image.enumerate_pixels() {
            let x = pixel.0 as _;
            let y = pixel.1 as _;
            let [r, g, b] = pixel.2.0;
            tensor[[0, x, y, 0]] = (r as f32) / 255.;
            tensor[[0, x, y, 1]] = (g as f32) / 255.;
            tensor[[0, x, y, 2]] = (b as f32) / 255.;
        }

        Ok(tensor)
    }
}
