use std::path::PathBuf;

use fast_image_resize::images::Image;
use fast_image_resize::{PixelType, Resizer};
use image::RgbImage;
use image::io::Reader;
use tensorflow::Tensor;

use crate::error::Error;
use crate::input::ClassifierInput;

#[derive(Debug)]
pub struct ProceededImage {
    pub path: PathBuf,
    pub image_tensor: Tensor<f32>,
}

pub fn preprocess(classifier_input: &ClassifierInput) -> Result<ProceededImage, Error> {
    let reader = Reader::open(&classifier_input.file_path)?;
    let decoded_img = reader.decode()?;

    // Crop image
    let img_rgb: RgbImage;
    if let Some(bbox) = &classifier_input.bbox {
        let min_x = (bbox.x1() * decoded_img.width() as f64) as u32;
        let min_y = (bbox.y1() * decoded_img.height() as f64) as u32;
        let max_x = (bbox.x2() * decoded_img.width() as f64) as u32;
        let max_y = (bbox.y2() * decoded_img.height() as f64) as u32;
        let cropped_img = decoded_img.crop_imm(min_x, min_y, max_x - min_x, max_y - min_y);
        img_rgb = cropped_img.to_rgb8();
    } else {
        img_rgb = decoded_img.to_rgb8();
    }
    let (w, h) = img_rgb.dimensions();
    let src_image = Image::from_vec_u8(w, h, img_rgb.into_raw(), PixelType::U8x3)?;
    let mut dst_image = Image::new(480, 480, PixelType::U8x3);
    let mut resizer = Resizer::new();

    resizer.resize(&src_image, &mut dst_image, None)?;

    let pixels: Vec<f32> = dst_image
        .buffer()
        .iter()
        .copied()
        .map(|v| v as f32 / 255.0)
        .collect();

    let tensor = Tensor::new(&[1, 480, 480, 3]).with_values(&pixels)?;
    Ok(ProceededImage {
        path: classifier_input.file_path.clone(),
        image_tensor: tensor,
    })
}
