use ndarray::{ArrayView1, ArrayView2};
use tracing::debug;

/// A function to perform Non-max suppression on a detection tensor and scores tensor.
///
/// This function is mimicked from [LibTorch's Non-max suppression](https://github.com/pytorch/vision/blob/124dfa404f395db90280e6dd84a51c50c742d5fd/torchvision/csrc/ops/cpu/nms_kernel.cpp).
///
/// This function's implementation is take from [this issue comment in tch](https://github.com/laurentmazare/tch-rs/issues/833#issuecomment-1905027185).
pub fn nms(detections: ArrayView2<f32>, scores: ArrayView1<f32>, iou_threshold: f32) -> Vec<usize> {
    let mut sorting: Vec<usize> = (0..scores.len()).collect();
    sorting.sort_unstable_by(|&a, &b| scores[a].total_cmp(&scores[b]));

    let mut keep: Vec<usize> = Vec::new();

    while let Some(idx) = sorting.pop() {
        keep.push(idx);

        for i in (0..sorting.len()).rev() {
            if iou(detections.row(idx), detections.row(sorting[i])) > iou_threshold {
                _ = sorting.remove(i);
            }
        }
    }

    debug!("keep results index are {:?}", keep);

    keep
}

/// Performs the calculation of Intersection Over Union value that's being used in Non-max
/// suppression function.
///
/// This function's implementation is take from [this issue comment in tch](https://github.com/laurentmazare/tch-rs/issues/833#issuecomment-1905027185).
fn iou(a: ArrayView1<f32>, b: ArrayView1<f32>) -> f32 {
    let zero: f32 = 0.0;

    let a_area = (a[2] - a[0] + 1.0) * (a[3] - a[1] + 1.0);
    let b_area = (b[2] - b[0] + 1.0) * (b[3] - b[1] + 1.0);

    let i_x1 = a[0].max(b[0]);
    let i_y1 = a[1].max(b[1]);
    let i_x2 = a[2].min(b[2]);
    let i_y2 = a[3].min(b[3]);

    let i_width = (i_x2 - i_x1 + 1.0).max(zero);
    let i_height = (i_y2 - i_y1 + 1.0).max(zero);
    let i_area = i_width * i_height;

    i_area / (a_area + b_area - i_area)
}
