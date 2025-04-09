use std::path::PathBuf;

use fast_image_resize::images::Image;
use fast_image_resize::{PixelType, Resizer};
use image::{DynamicImage, ImageReader, RgbImage};
use speciesnet_core::BoundingBox;
use tensorflow::Tensor;

use crate::error::Error;
use crate::input::ClassifierInput;

#[derive(Debug)]
pub struct ProceededImage {
    pub path: PathBuf,
    pub image_tensor: Tensor<f32>,
}

pub fn preprocess(classifier_input: &ClassifierInput) -> Result<ProceededImage, Error> {
    let reader = ImageReader::open(&classifier_input.file_path)?;
    let decoded_img = reader.decode()?;

    let proceeded_image = preprocess_impl(decoded_img, classifier_input.bbox)?;

    let pixels = proceeded_image
        .into_vec()
        .iter()
        .map(|p| *p as f32 / 255.0)
        .collect::<Vec<f32>>();
    let tensor = Tensor::new(&[1, 480, 480, 3]).with_values(&pixels)?;

    Ok(ProceededImage {
        path: classifier_input.file_path.clone(),
        image_tensor: tensor,
    })
}

pub fn preprocess_impl(
    decoded_image: DynamicImage,
    bbox: Option<BoundingBox>,
) -> Result<RgbImage, Error> {
    // Performs cropping with given bounding box if there is a bounding box, otherwise just return.
    let cropped_image = match bbox {
        Some(bbox) => {
            let min_x = (bbox.x1() * decoded_image.width() as f64) as u32;
            let min_y = (bbox.y1() * decoded_image.height() as f64) as u32;
            let max_x = (bbox.x2() * decoded_image.width() as f64) as u32;
            let max_y = (bbox.y2() * decoded_image.height() as f64) as u32;
            let cropped_img = decoded_image.crop_imm(min_x, min_y, max_x - min_x, max_y - min_y);

            cropped_img.to_rgb8()
        }
        None => decoded_image.to_rgb8(),
    };

    // Resize the image to 480 by 480 (classifier's accept input size).
    let mut resizer = Resizer::new();

    let src_image = Image::from_vec_u8(
        cropped_image.width(),
        cropped_image.height(),
        cropped_image.into_raw(),
        PixelType::U8x3,
    )?;
    let mut dest_image = Image::new(480, 480, PixelType::U8x3);

    resizer.resize(&src_image, &mut dest_image, None)?;

    // Creates the image back.
    let image = RgbImage::from_raw(480, 480, dest_image.into_vec()).unwrap();

    // Returns the image back.
    Ok(image)
}
