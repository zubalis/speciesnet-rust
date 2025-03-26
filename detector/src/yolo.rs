use log::{debug, info};
use tch::{IndexOp, TchError, Tensor};

use crate::{error::Error, torchvision::nms};

const DEFAULT_CONF_THRESHOLD: f64 = 0.25;
const MAX_BOUNDING_BOX_HEIGHT: i32 = 7680;
const MAX_NMS_BOXES: i32 = 30000;
const REQUIRES_REDUNDANT_DETECTION: bool = true;

const DEFAULT_XYWHN_WIDTH_HEIGHT: i64 = 640;

/// Convert [Tensor] output in shape of `Tensor[4]` which is in the format of `(center_x, center_y,
/// width, height)` to `(x1, y1, x2, y2)`.
///
/// The function could panic if the dimension of the [Tensor](tch::Tensor)
///
/// # Panics
///
/// The function could still panic if the tensor dimension is not `Tensor[4, Float]`.
///
/// [Tensor]: tch::Tensor
pub fn xywh_to_xyxy(tensor: &Tensor) -> Result<Tensor, Error> {
    let tensor_size = tensor.size2()?;

    if tensor_size.1 != 4 {
        return Err(Error::TchError(TchError::Shape(
            "Invalid tensor shape.".to_string(),
        )));
    }

    Ok(Tensor::f_stack(
        &[
            // x1 (top left x)
            tensor.f_i((.., 0))? - (tensor.f_i((.., 2))? / 2),
            // y1 (top left y)
            tensor.f_i((.., 1))? - (tensor.f_i((.., 3))? / 2),
            // x2 (bottom right x)
            tensor.f_i((.., 0))? + (tensor.f_i((.., 2))? / 2),
            // y2 (bottom right y)
            tensor.f_i((.., 1))? + (tensor.f_i((.., 3))? / 2),
        ],
        -1,
    )?)
}

/// Convert [Tensor] output in the shape of `Tensor[4]` which is in the format of `(x1, y1, x2, y2)`
/// to `(center_x, center_y, width, height)`.
///
/// # Panics
///
/// The function could still panic if the tensor dimension is not `Tensor[4, Float]`.
///
/// [Tensor]: tch::Tensor
pub fn xyxy_to_xywh(tensor: &Tensor) -> Result<Tensor, Error> {
    let xywh_tensor = Tensor::f_stack(
        &[
            (tensor.f_i((.., 0))? + tensor.f_i((.., 2))?) / 2,
            (tensor.f_i((.., 1))? + tensor.f_i((.., 3))?) / 2,
            tensor.f_i((.., 2))? - tensor.f_i((.., 0))?,
            tensor.f_i((.., 3))? - tensor.f_i((.., 1))?,
        ],
        -1,
    )?;

    Ok(xywh_tensor)
}

/// Convert an array of coordinates in the format of
pub fn yolo_xywhn_to_mega_detector_xywh(
    yolo_xywhn: Vec<(f64, f64, f64, f64)>,
) -> Result<Vec<(f64, f64, f64, f64)>, Error> {
    let result: Vec<_> = yolo_xywhn
        .iter()
        .map(|coords| {
            let (center_x, center_y, width, height) = coords;
            let min_x = center_x - (width / 2.0f64);
            let min_y = center_y - (height / 2.0f64);

            (min_x, min_y, *width, *height)
        })
        .collect();

    Ok(result)
}

/// Convert [Tensor] input in the shape of `Tensor[4]` which is in the format of `(x1, y1, x2, y2)`
/// to `(x, y, width, height)` with scaling back to original image's size where `x1` and `y1` are top left coordinates, and `x2` and `y2` are
/// bottom right coordinates.
///
/// This function is just a partial implementation of yolo's [xyxy2xywhn] function. The function's
/// call signature is similar to `xyxy2xywhn(x, w, h, clip=False, eps=0.0)`. The function does not
/// support box clipping.
///
/// # Panics
///
/// The function could still panic if the tensor dimension is not `Tensor[4, Float]`.
///
/// [xyxy2xywhn]: https://github.com/ultralytics/yolov5/blob/5cdad8922c83b0ed49a0173cd1a8b0739acbb336/utils/general.py#L903-L912
/// [Tensor]: tch::Tensor
pub fn xyxy_to_xywhn(
    tensor: &Tensor,
    width: Option<i64>,
    height: Option<i64>,
) -> Result<Tensor, Error> {
    let width = width.unwrap_or(DEFAULT_XYWHN_WIDTH_HEIGHT);
    let height = height.unwrap_or(DEFAULT_XYWHN_WIDTH_HEIGHT);

    let tensor_size = tensor.size2()?;

    if tensor_size.1 != 4 {
        return Err(
            Error::TchError(
                TchError::Shape("Invalid tensor size at the 2nd dimension, expected 4. Run the `unsqueeze` function to get 2d tensor.".to_string())
            )
        );
    }

    let xywhn_tensor = Tensor::f_stack(
        &[
            ((tensor.f_i((.., 0))? + tensor.f_i((.., 2))?) / 2) / width,
            ((tensor.f_i((.., 1))? + tensor.f_i((.., 3))?) / 2) / height,
            (tensor.f_i((.., 2))? - tensor.f_i((.., 0))?) / width,
            (tensor.f_i((.., 3))? - tensor.f_i((.., 1))?) / height,
        ],
        -1,
    )?;

    Ok(xywhn_tensor)
}

pub fn non_max_suppression(
    predictions: &Tensor,
    conf_threshold: Option<f64>,
) -> Result<Vec<Tensor>, Error> {
    let conf_threshold: f64 = conf_threshold.map_or(DEFAULT_CONF_THRESHOLD, |v| {
        if (0.0..1.0).contains(&v) {
            v
        } else {
            DEFAULT_CONF_THRESHOLD
        }
    });

    let iou_threshold = 0.45f64;
    let _classes = Vec::<String>::new();
    let agnostic = false;
    let multi_label = true;
    let _labels = Vec::<i32>::new();
    let max_detections = 300;
    let number_of_masks = 0i32;

    // checks

    debug!("Getting variables out of the predictions tensor");
    let (batch_size, _, number_of_classes) = predictions.size3()?;
    let number_of_classes: i64 = number_of_classes - (number_of_masks as i64) - 5i64;
    let candidates = predictions.i((.., .., 4)).gt(conf_threshold);

    let max_wh = 7680;
    let max_nms = 30000;
    let _redundant = true;
    let _multi_label = multi_label & (number_of_classes > 1);
    let _merge = false;

    let mask_start_index = 5 + number_of_classes;

    let mut output: Vec<Tensor> = Vec::new();

    debug!("batch size {}", batch_size);
    debug!("number of classes {}", number_of_classes);
    debug!("candidates {:?}", candidates);

    for i in 0..batch_size {
        debug!("Filtering the tensor with candidates tensor by minimum confidence threshold.");
        let tensor = predictions.i(i);
        let tensor = tensor.index_select(0, &candidates.i(i).nonzero().squeeze());

        // TODO: Implement labels support.

        if tensor.size().first().copied().unwrap_or_default() == 0i64 {
            info!("The image image does not have any results.");
            return Ok(Vec::new());
        }

        // Compute conf

        // last 3 columns
        debug!("Reconstructing the tensor.");
        let mut object_conf = tensor.i((.., 5..));

        // only the 4th column in tensor format.
        let class_conf = tensor.i((.., 4..5));
        object_conf *= class_conf;

        // reconstructing the whole 8 columns from
        // (first original 5 columns, last 3 columns that gets timed).
        let tensor = Tensor::cat(&[tensor.i((.., ..5)), object_conf], 1);

        debug!(
            "Converting the tensor bounding box into xyxy bounding box for non-max suppression."
        );
        let bbox = xywh_to_xyxy(&tensor.i((.., ..4)))?;
        let mask = tensor.i((.., mask_start_index..));

        // Supposed to be the multi_label if else. We're only implementing the else route.
        let (conf, j) = tensor.i((.., 5..mask_start_index)).max_dim(1, true);
        let mut conf_copy = Tensor::zeros_like(&conf);
        conf_copy.copy_(&conf);

        let tensor = Tensor::cat(&[bbox, conf_copy, j.to_kind(tch::Kind::Float), mask], 1);
        let conf_flat = conf.view(-1);
        let bool_mask = conf_flat.gt(conf_threshold);
        let indices = bool_mask.nonzero().squeeze();
        let tensor = tensor.index_select(0, &indices);

        // Filter by class
        // being skipped because the variable is not provided.

        // Shape checking
        debug!("Checking the shape of the tensor before running non-max suppression.");
        if tensor.size().first().copied().unwrap_or_default() == 0i64 {
            info!("The image does not have any results after filtering.");
            return Ok(Vec::new());
        }

        // Sort by confidence and remove excess boxes.
        debug!("Removing excess boxes by confidence.");
        let confidence_argsorted = tensor.i((.., 4)).argsort(-1, true);
        let confidence = if confidence_argsorted.size1()? > max_nms {
            confidence_argsorted.i(..max_nms)
        } else {
            confidence_argsorted
        };

        let tensor = tensor.index_select(0, &confidence);

        // Batched NMS
        let class = if agnostic {
            Tensor::zeros_like(&tensor.i((.., 5..6)))
        } else {
            tensor.i((.., 5..6)) * max_wh
        };

        // boxes (offset by class)
        let boxes = tensor.i((.., ..4)) + class;
        // socres
        let scores = tensor.i((.., 4));

        debug!("Running the LibTorch's Non-max suppression.");
        let nms_raw_results = nms(&boxes, &scores, iou_threshold);

        // Limit max detections
        debug!("Limiting the number of returned results from LibTorch's NMS.");
        let nms_indexes = if nms_raw_results.size1()? > max_detections {
            nms_raw_results.i(..max_nms)
        } else {
            nms_raw_results
        };

        // INFO: Implement merge branch.

        debug!("Pushing the result tensor back to the image.");
        output.push(tensor.index_select(0, &nms_indexes));
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tch::Tensor;

    #[test]
    fn xywh_to_xyxy_conversion() {
        let initial_tensor = Tensor::from_slice2(&[&[9f64, 11f64, 8f64, 6f64]]);
        let xyxy_tensor: Vec<Vec<f64>> = xywh_to_xyxy(&initial_tensor).unwrap().try_into().unwrap();

        assert_eq!(xyxy_tensor, [[5f64, 8f64, 13f64, 14f64]]);
    }

    #[test]
    #[should_panic]
    fn xywh_tensor_invalid_length() {
        let initial_tensor = Tensor::from_slice2(&[&[9f64, 11f64, 8f64]]);
        let xyxy_tensor: Result<Tensor, Error> = xywh_to_xyxy(&initial_tensor);

        assert!(xyxy_tensor.is_err());
    }

    #[test]
    fn xyxy_to_xywh_conversion() {
        let initial_tensor = Tensor::from_slice2(&[&[5f64, 8f64, 13f64, 14f64]]);
        let xywh_tensor: Vec<Vec<f64>> = xyxy_to_xywh(&initial_tensor).unwrap().try_into().unwrap();
        assert_eq!(xywh_tensor, [[9f64, 11f64, 8f64, 6f64]]);
    }

    #[test]
    #[should_panic]
    fn xyxy_to_xywh_invalid_length() {
        let initial_tensor = Tensor::from_slice2(&[&[5f64, 8f64, 13f64]]);
        let xywh_tensor: Result<Tensor, Error> = xyxy_to_xywh(&initial_tensor);
        assert!(xywh_tensor.is_err());
    }

    #[test]
    fn xyxy_to_xywhn_conversion() {
        let initial_tensor = Tensor::from_slice2(&[&[50f64, 50f64, 150f64, 150f64]]);
        let width = Some(200i64);
        let height = Some(200i64);

        let xywhn_result: Vec<Vec<f64>> = xyxy_to_xywhn(&initial_tensor, width, height)
            .unwrap()
            .try_into()
            .unwrap();

        assert_eq!(xywhn_result, [[0.5f64, 0.5f64, 0.5f64, 0.5f64]]);
    }
}
