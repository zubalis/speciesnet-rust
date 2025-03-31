use core::f32;

use ndarray::{Array2, ArrayD, ArrayView2, Axis, concatenate, s, stack};
use tracing::{debug, info};

use crate::{error::Error, torchvision::nms};

const DEFAULT_CONF_THRESHOLD: f32 = 0.25;

const IOU_THRESHOLD: f32 = 0.45;
const AGNOSTIC: bool = false;
const MULTI_LABEL: bool = true;
const MAX_DETECTIONS: i32 = 300;
const NUMBER_OF_MASKS: i32 = 0;

const MAX_BOUNDING_BOX_HEIGHT: i32 = 7680;
const MAX_NMS_BOXES: i32 = 30000;
const REQUIRES_REDUNDANT_DETECTION: bool = true;

const DEFAULT_XYWHN_WIDTH_HEIGHT: i64 = 640;

pub fn xywh_to_xyxy(tensor: ArrayView2<f32>) -> Result<Array2<f32>, Error> {
    let x1 = &tensor.slice(s![.., 0]) - (&tensor.slice(s![.., 2]) / 2.0f32);
    let y1 = &tensor.slice(s![.., 1]) - (&tensor.slice(s![.., 3]) / 2.0f32);
    let x2 = &tensor.slice(s![.., 0]) + (&tensor.slice(s![.., 2]) / 2.0f32);
    let y2 = &tensor.slice(s![.., 1]) + (&tensor.slice(s![.., 3]) / 2.0f32);

    Ok(stack(
        Axis(1),
        &[x1.view(), y1.view(), x2.view(), y2.view()],
    )?)
}

pub fn non_max_suppression(
    predictions: ArrayD<f32>,
    conf_threshold: Option<f32>,
) -> Result<Array2<f32>, Error> {
    let conf_threshold = conf_threshold.map_or(DEFAULT_CONF_THRESHOLD, |v| {
        if (0.0..1.0).contains(&v) {
            v
        } else {
            DEFAULT_CONF_THRESHOLD
        }
    });

    // checks
    let shapes = predictions.shape();
    let batch_size = shapes.first().unwrap();
    let number_of_classes = *shapes.get(2).unwrap() - (NUMBER_OF_MASKS as usize) - 5;
    info!("shapes {:?}", shapes);
    info!("batch size {}", batch_size,);
    info!("number of classes {}", number_of_classes);
    let candidates = predictions
        .slice(s![.., .., 4])
        .mapv(|e| e > conf_threshold);

    info!("candidates {:?}", candidates);

    let mask_start_index = 5 + number_of_classes;

    let view = predictions.index_axis(Axis(0), 0);
    let indices = candidates
        .clone()
        .remove_axis(Axis(0))
        .indexed_iter()
        .filter_map(|(i, val)| if *val { Some(i) } else { None })
        .collect::<Vec<usize>>();

    let tensor = view.select(Axis(0), &indices);

    debug!("after index select, tensor size is {:?}", tensor.shape());
    if *tensor.shape().first().unwrap_or(&0usize) == 0 {
        info!("This image does not have any results.");
    }

    debug!("tensor index selected size {:?}", tensor);

    debug!("Reconstructing the tensor.");
    // last 3 columns
    let object_conf = tensor.slice(s![.., 5..]);
    let class_conf = tensor.slice(s![.., 4..5]);

    let object_conf = &object_conf * &class_conf;
    // Reconstructing the whole 8 columns from
    // (first original 5 columns, last 3 columns that gets multiplied).
    let tensor = concatenate(Axis(1), &[tensor.slice(s![.., ..5]), object_conf.view()])?;

    debug!("Converting the tensor bounding box into xyxy bounding box for non-max suppression.");

    // xywh to xyxy
    let bbox = xywh_to_xyxy(tensor.view())?;

    debug!("bbox information {:?}", bbox);
    // mask calculation we need to take in the index sizes first before indexing.
    //let mask = tensor.slice(s![.., mask_start_index..]);

    debug!("retrieving max confidence.");
    let conf_flat = tensor.slice(s![.., 5..8]).map_axis(Axis(1), |m| {
        *m.iter().max_by(|a, b| a.total_cmp(b)).unwrap()
    });

    let conf = conf_flat.clone().insert_axis(Axis(1));

    debug!("retireving max confidence index.");
    let j = tensor
        .slice(s![.., 5..8])
        .map_axis(Axis(1), |m| {
            m.iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.total_cmp(b))
                .map(|(idx, _)| idx as f32)
                .unwrap()
        })
        .insert_axis(Axis(1));

    debug!("Conf {:?}, J: {:?}", conf, j);

    let tensor = concatenate(Axis(1), &[bbox.view(), conf.view(), j.view()])?;
    debug!("Tensor before filtering {:?}", tensor);
    let bool_mask = conf_flat
        .mapv(|f| f > conf_threshold)
        .indexed_iter()
        .filter_map(|(i, val)| if *val { Some(i) } else { None })
        .collect::<Vec<usize>>();

    let tensor = tensor.select(Axis(0), &bool_mask);

    debug!("checking the shape of the tensor before running non-max suppression.");

    if *tensor.shape().first().unwrap_or(&0usize) == 0usize {
        info!("This image does not have any results after bool mask filtering.");
    }

    // Filter by class
    debug!("Removing excess boxes by confidence.");
    let confidence = tensor.slice(s![.., 4]);
    let mut confidence_argsort: Vec<usize> = (0..confidence.len()).collect();
    confidence_argsort.sort_unstable_by(|&a, &b| confidence[b].total_cmp(&confidence[a]));
    confidence_argsort.truncate(MAX_NMS_BOXES as usize);
    let tensor = tensor.select(Axis(0), &confidence_argsort);

    debug!("tensor is {:?}", tensor);

    // Batched NMS
    let class = tensor
        .slice(s![.., 5..6])
        .mapv(|e| e * MAX_BOUNDING_BOX_HEIGHT as f32);

    let boxes = &tensor.slice(s![.., ..4]) + &class;
    let scores = tensor.column(4);

    debug!("scores {:?}", scores);
    debug!("Running LibTorch's Non-max suppression.");

    let mut nms_indexes = nms(boxes.view(), scores, 0.45);
    nms_indexes.truncate(MAX_NMS_BOXES as usize);

    debug!("nms indexes are {:?}", nms_indexes);

    let filtered_results = tensor.select(Axis(0), &nms_indexes);

    debug!("final results is {:?}", filtered_results);
    Ok(filtered_results)
}
