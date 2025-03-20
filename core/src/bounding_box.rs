use std::{cmp::min, fmt::Display, ops::RangeFrom};

use tch::{IndexOp, Tensor};

use crate::error::Error;

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    /// Top left `x` point of the image.
    x1: f64,
    /// Top left `y` point of the image.
    y1: f64,
    /// Bottom right `x` point of the image.
    x2: f64,
    /// Bottom right `y` point of the image.
    y2: f64,
}

impl Display for BoundingBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(x1 = {}, y1 = {}, x2 = {}, y2 = {})",
            self.x1, self.y1, self.x2, self.y2
        )
    }
}

impl BoundingBox {
    /// Bounding box range to limit the bounding box coordinates into the positive space only.
    const BOUNDING_BOX_RANGE: RangeFrom<f64> = (0.0..);
    /// Expected tensor size for converting values to this struct from [`Tensor`].
    const EXPECTED_TENSOR_SIZE: i64 = 4;

    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// Construct the [`BoundingBox`] struct from `center_x`, `center_y`, `width`, and `height` values.
    pub fn from_xywh_coordinates(
        center_x: f64,
        center_y: f64,
        width: f64,
        height: f64,
    ) -> Result<Self, Error> {
        let x1 = center_x - (width / 2.0);
        let y1 = center_y - (height / 2.0);
        let x2 = center_x + (width / 2.0);
        let y2 = center_y + (height / 2.0);

        Ok(Self { x1, y1, x2, y2 })
    }

    /// Returns the values of the coordinates in a form of `(x1, y1, x2, y2)` tuple format.
    pub fn as_xyxy_bounding_box(&self) -> (f64, f64, f64, f64) {
        (self.x1, self.y1, self.x2, self.y2)
    }

    /// Returns the values of the coordinates in a form of `(center_x, center_y, width, height)`
    /// tuple format.
    pub fn as_xywh_bounding_box(&self) -> (f64, f64, f64, f64) {
        let center_x = self.x1 + (self.x2 / 2.0);
        let center_y = self.y1 + (self.y2 / 2.0);
        let width = self.x2 - self.x1;
        let height = self.y2 - self.y1;

        (center_x, center_y, width, height)
    }

    /// Tries to convert a [`Tensor`] in format of `(x1, y1, x2, y2)` to the bounding box struct.
    /// Tensor must be 1 dimension in the format of `Tensor[4, Float]`.
    pub fn from_xyxy_tensor(tensor: &Tensor) -> Result<Self, Error> {
        let tensor_size = tensor.size1()?;

        if tensor_size < 4 {
            return Err(Error::InvalidTensorSize(tensor_size));
        }

        let x1 = tensor.f_i(0)?.f_double_value(&[])?;
        let y1 = tensor.f_i(1)?.f_double_value(&[])?;
        let x2 = tensor.f_i(2)?.f_double_value(&[])?;
        let y2 = tensor.f_i(3)?.f_double_value(&[])?;

        Ok(Self { x1, y1, x2, y2 })
    }

    /// Tries to convert a [Tensor] in format of `(center_y, center_y, width, height)` to the bounding box struct.
    ///
    /// # Panics
    ///
    /// This function could panic if the [Tensor]'s shape is not `Tensor[4, Float]`.
    ///
    /// [Tensor]: tch::Tensor
    pub fn from_xywh_tensor(tensor: &Tensor) -> Result<Self, Error> {
        let tensor_size = tensor.size1()?;

        if tensor_size < 4 {
            return Err(Error::InvalidTensorSize(tensor_size));
        }

        let center_x = tensor.f_i(0)?.f_double_value(&[])?;
        let center_y = tensor.f_i(1)?.f_double_value(&[])?;
        let width = tensor.f_i(2)?.f_double_value(&[])?;
        let height = tensor.f_i(3)?.f_double_value(&[])?;

        let x1 = center_x - (width / 2.0);
        let y1 = center_y - (height / 2.0);
        let x2 = center_x + (width / 2.0);
        let y2 = center_y + (height / 2.0);

        Ok(Self { x1, y1, x2, y2 })
    }

    //def scale_boxes(img1_shape, boxes, img0_shape, ratio_pad=None):
    //    # Rescale boxes (xyxy) from img1_shape to img0_shape
    //    if ratio_pad is None:  # calculate from img0_shape
    //        gain = min(img1_shape[0] / img0_shape[0], img1_shape[1] / img0_shape[1])  # gain  = old / new
    //        pad = (img1_shape[1] - img0_shape[1] * gain) / 2, (img1_shape[0] - img0_shape[0] * gain) / 2  # wh padding
    //    else:
    //        gain = ratio_pad[0][0]
    //        pad = ratio_pad[1]
    //
    //    boxes[..., [0, 2]] -= pad[0]  # x padding
    //    boxes[..., [1, 3]] -= pad[1]  # y padding
    //    boxes[..., :4] /= gain
    //    clip_boxes(boxes, img0_shape)
    //    return boxes
    //boxes[..., 0].clamp_(0, shape[1])  # x1
    //boxes[..., 1].clamp_(0, shape[0])  # y1
    //boxes[..., 2].clamp_(0, shape[1])  # x2
    //boxes[..., 3].clamp_(0, shape[0])  # y2

    /// Scale the bounding box coordinates to a given width and height.
    pub fn scale_to(
        mut self,
        scaled_down_width: u32,
        scaled_down_height: u32,
        width: u32,
        height: u32,
    ) -> Self {
        let gain = f32::min(
            scaled_down_width as f32 / width as f32,
            scaled_down_height as f32 / height as f32,
        );
        let pad = (
            (scaled_down_height as f32 - (height as f32 * gain)) / 2.0,
            (scaled_down_width as f32 - (width as f32 * gain)) / 2.0,
        );

        let x1 = ((self.x1 - pad.0 as f64) / gain as f64).clamp(0.0, width as f64);
        let x2 = ((self.x2 - pad.0 as f64) / gain as f64).clamp(0.0, width as f64);

        let y1 = ((self.y1 - pad.1 as f64) / gain as f64).clamp(0.0, height as f64);
        let y2 = ((self.y2 - pad.1 as f64) / gain as f64).clamp(0.0, height as f64);

        self.x1 = x1;
        self.y1 = y1;
        self.x2 = x2;
        self.y2 = y2;

        self
    }
}
