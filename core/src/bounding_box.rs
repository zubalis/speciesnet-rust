use std::fmt::Display;

use serde::{Deserialize, Serialize, de, ser::SerializeSeq};
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

impl<'de> Deserialize<'de> for BoundingBox {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let variant = Vec::<f32>::deserialize(deserializer)?;

        // The length of the given array must be 4 as the bounding box is saved in the json file as
        // `(min_x, min_y, width, height)`.
        if variant.len() != BoundingBox::EXPECTED_TENSOR_SIZE as usize {
            return Err(de::Error::invalid_length(
                variant.len(),
                &"an array with length of 4.",
            ));
        }

        // SAFETY: These unwraps are safe to do so becase we've verified the length of the array to
        // be (and only be) 4.
        let min_x = variant.first().unwrap();
        let min_y = variant.get(1).unwrap();
        let width = variant.get(2).unwrap();
        let height = variant.get(3).unwrap();

        Ok(BoundingBox::from_megadetector_coordinates(
            *min_x as f64,
            *min_y as f64,
            *width as f64,
            *height as f64,
        ))
    }
}

impl Serialize for BoundingBox {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(BoundingBox::EXPECTED_TENSOR_SIZE as usize))?;
        let (min_x, min_y, width, height) = self.as_megadetector_bounding_box();

        seq.serialize_element(&min_x)?;
        seq.serialize_element(&min_y)?;
        seq.serialize_element(&width)?;
        seq.serialize_element(&height)?;

        seq.end()
    }
}

impl BoundingBox {
    /// Expected tensor size for converting values to this struct from [`Tensor`].
    const EXPECTED_TENSOR_SIZE: i64 = 4;

    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// Construct the [`BoundingBox`] struct from `center_x`, `center_y`, `width`, and `height` values.
    pub fn from_xywh_coordinates(center_x: f64, center_y: f64, width: f64, height: f64) -> Self {
        let x1 = center_x - (width / 2.0);
        let y1 = center_y - (height / 2.0);
        let x2 = center_x + (width / 2.0);
        let y2 = center_y + (height / 2.0);

        Self { x1, y1, x2, y2 }
    }

    /// Constructs the [`BoundingBox`] struct from given `min_x`, `min_y`, `width` and `height`
    /// values.
    pub fn from_megadetector_coordinates(min_x: f64, min_y: f64, width: f64, height: f64) -> Self {
        Self {
            x1: min_x,
            y1: min_y,
            x2: min_x + width,
            y2: min_y + height,
        }
    }

    /// Tries to convert a [`Tensor`] in format of `(x1, y1, x2, y2)` to the bounding box struct.
    /// Tensor must be 1 dimension in the format of `Tensor[4, Float]`.
    pub fn from_xyxy_tensor(tensor: &Tensor) -> Result<Self, Error> {
        let tensor_size = tensor.size1()?;

        if tensor_size < Self::EXPECTED_TENSOR_SIZE {
            return Err(Error::InvalidTensorSize(tensor_size));
        }

        let x1 = tensor.f_i(0)?.f_double_value(&[])?;
        let y1 = tensor.f_i(1)?.f_double_value(&[])?;
        let x2 = tensor.f_i(2)?.f_double_value(&[])?;
        let y2 = tensor.f_i(3)?.f_double_value(&[])?;

        Ok(Self { x1, y1, x2, y2 })
    }

    /// Tries to convert a [Tensor][tensor] in format of `(center_y, center_y, width, height)` to the bounding box struct.
    ///
    /// # Panics
    ///
    /// This function could panic if the [Tensor][tensor]'s shape is not `Tensor[4, Float]`.
    ///
    /// [tensor]: tch::Tensor
    pub fn from_xywh_tensor(tensor: &Tensor) -> Result<Self, Error> {
        let tensor_size = tensor.size1()?;

        if tensor_size < Self::EXPECTED_TENSOR_SIZE {
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

    /// Returns the values of the coordinates in a form on `(min_x, min_y, width, height)` tuple
    /// format.
    pub fn as_megadetector_bounding_box(&self) -> (f64, f64, f64, f64) {
        let min_x = self.x1;
        let min_y = self.y1;
        let width = self.x2 - self.x1;
        let height = self.y2 - self.y1;

        (min_x, min_y, width, height)
    }

    /// Normalize the values to be under `0..1` by the given width and height.
    ///
    /// This is implemented to be chained with the scale function to cap the numbers between `0` and
    /// `1`.
    pub fn normalize(mut self, width: u32, height: u32) -> Self {
        self.x1 /= width as f64;
        self.y1 /= height as f64;
        self.x2 /= width as f64;
        self.y2 /= height as f64;

        self
    }

    /// Scale the bounding box coordinates to a given width and height. This function is a
    /// combination of [YOLOv5's scale_boxes] and [YOLOv5's clip_boxes] function.
    ///
    /// [YOLOv5's scale_boxes]: https://github.com/ultralytics/yolov5/blob/8cc449636da76757a71385a2b57dc977db58b81e/utils/general.py#L953-L966
    /// [YOLOv5's clip_boxes]: https://github.com/ultralytics/yolov5/blob/8cc449636da76757a71385a2b57dc977db58b81e/utils/general.py#L988-L997
    pub fn scale_to(
        mut self,
        resized_width: u32,
        resized_height: u32,
        width: u32,
        height: u32,
    ) -> Self {
        let gain = f32::min(
            resized_width as f32 / width as f32,
            resized_height as f32 / height as f32,
        );
        let pad = (
            (resized_height as f32 - (height as f32 * gain)) / 2.0,
            (resized_width as f32 - (width as f32 * gain)) / 2.0,
        );

        // The clamp part is the clip_boxes function.
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
