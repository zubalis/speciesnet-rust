use tch::{IndexOp, Tensor};

/// A function to perform Non-max suppression on a detection tensor and scores tensor.
///
/// This function is mimicked from [LibTorch's Non-max suppression](https://github.com/pytorch/vision/blob/124dfa404f395db90280e6dd84a51c50c742d5fd/torchvision/csrc/ops/cpu/nms_kernel.cpp)
///
/// This function's implementation is taken from [this issue in tch](https://github.com/laurentmazare/tch-rs/issues/833#issuecomment-1905027185)
///
/// # Panics
///
/// The function can panic since the tensor operations are C++ FFI.
pub fn nms(detections: &Tensor, scores: &Tensor, iou_threshold: f64) -> Tensor {
    let mut sorting: Vec<i64> = scores.argsort(0, false).try_into().unwrap();
    let mut keep: Vec<i64> = Vec::new();
    while let Some(idx) = sorting.pop() {
        keep.push(idx);
        for i in (0..sorting.len()).rev() {
            if iou(&detections.i(idx), &detections.i(sorting[i])).double_value(&[]) > iou_threshold
            {
                _ = sorting.remove(i);
            }
        }
    }

    Tensor::try_from(keep).unwrap().to_device(tch::Device::Cpu)
}

/// Performs the calculation of Intersection Over Union value that's being used in Non-max
/// suppression function.
///
/// The function's implementation is taken from [this issue in tch](https://github.com/laurentmazare/tch-rs/issues/833#issuecomment-1905027185)
///
/// # Panics
///
/// The function can panic since the tensor operations are C++ FFI.
fn iou(a: &Tensor, b: &Tensor) -> Tensor {
    let zero = Tensor::zeros_like(&a.i(0));
    let a_area = (a.i(2) - a.i(0) + 1) * (a.i(3) - a.i(1) + 1);
    let b_area = (b.i(2) - b.i(0) + 1) * (b.i(3) - b.i(1) + 1);
    let i_xmin = a.i(0).max_other(&b.i(0));
    let i_xmax = a.i(2).min_other(&b.i(2));
    let i_ymin = a.i(1).max_other(&b.i(1));
    let i_ymax = a.i(3).min_other(&b.i(3));
    let i_area = (i_xmax - i_xmin + 1).max_other(&zero) * (i_ymax - i_ymin + 1).max_other(&zero);
    &i_area / (a_area + b_area - &i_area)
}
