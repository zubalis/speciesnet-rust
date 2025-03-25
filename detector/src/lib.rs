use std::{path::Path, sync::Arc};

use log::debug;
use preprocess::PreprocessedImage;
use speciesnet_core::{BoundingBox, Detection, prediction::Prediction};
use tch::{CModule, Device, IValue, IndexOp};
use yolo::non_max_suppression;

use crate::error::Error;

pub mod error;
pub mod preprocess;
pub mod torchvision;
pub mod yolo;

#[derive(Debug, Clone)]
pub struct SpeciesNetDetector {
    model: Arc<CModule>,
    device: Device,
}

impl SpeciesNetDetector {
    pub fn new<P>(model_file_path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let mut model = CModule::load(model_file_path)?;
        let device = Device::cuda_if_available();
        model.to(device, tch::Kind::Float, false);

        Ok(Self {
            model: Arc::new(model),
            device,
        })
    }

    pub fn predict(
        &self,
        preprocessed_image: PreprocessedImage,
    ) -> Result<Option<Prediction>, Error> {
        // Converting the image to tensor.
        // We supposed to be able to just use the [`TryFrom`] implementation of rust tensor's image
        // feature but for some reason it does not work.
        let (original_width, original_height) = preprocessed_image.original_size();
        let (resized_width, resized_height) = preprocessed_image.resized_size();
        let path = preprocessed_image.path_owned();
        let tensor = preprocessed_image.into_tensor()?.to(self.device);
        let tensor = tensor.unsqueeze(0);

        debug!(
            "Original dimensions ({}, {})",
            original_width, original_height
        );
        let predictions = self.model.forward_is(&[IValue::Tensor(tensor)])?;

        match predictions {
            // The result of the model is in YOLO format where we have a tuple length of 2, and
            // we only care about the first one.
            tch::IValue::Tuple(ivalues) => {
                if ivalues.is_empty() {
                    return Ok(None);
                }

                // A check has been done above that it does have some amount of value.
                let first_result = ivalues.first().unwrap();

                let IValue::Tensor(predictions) = first_result else {
                    return Ok(None);
                };

                let nmsed_results = non_max_suppression(predictions, Some(0.01))?;
                let Some(nms_result) = nmsed_results.first() else {
                    return Ok(None);
                };

                let (detections_count, _detection_size) = nms_result.size2()?;
                let mut detections: Vec<Detection> = Vec::new();

                for result in 0..detections_count {
                    let xyxy_tensor = nms_result.f_i((result, ..4))?;
                    let confidence: f64 = nms_result.f_i((result, 4))?.f_double_value(&[])?;
                    let category: i64 = nms_result.f_i((result, 5))?.f_int64_value(&[])? + 1;

                    let bbox = BoundingBox::from_xyxy_tensor(&xyxy_tensor)?
                        .scale_to(
                            resized_width,
                            resized_height,
                            original_width,
                            original_height,
                        )
                        .normalize(original_width, original_height);

                    detections.push(Detection::new(category.try_into()?, confidence, bbox));
                }

                Ok(Some(Prediction::from_detections(path, detections)))
            }
            _ => {
                debug!(
                    "Filename {} does not return expected result.",
                    path.display()
                );

                Ok(None)
            }
        }
    }
}
