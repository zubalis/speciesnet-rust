use std::path::{Path, PathBuf};
use fast_image_resize::images::Image;
use fast_image_resize::{PixelType, Resizer};

use image::io::Reader;
use tensorflow::Tensor;
use crate::error::Error;

#[derive(Debug)]
pub struct ProceededImage {
    pub path: PathBuf,
    pub image_tensor: Tensor<f32>,
}

pub fn load_and_preprocess_images<P: AsRef<Path>>(
    image_path: P,
) -> Result<ProceededImage, Error> {
    let reader = Reader::open(image_path.as_ref())?;
    let decoded_img = reader.decode()?;
    let img_rgb = decoded_img.to_rgb8();
    let (w, h) = img_rgb.dimensions();
    let src_image = Image::from_vec_u8(
        w, h,
        img_rgb.into_raw(),
        PixelType::U8x3,
    )?;
    let mut dst_image = Image::new(480, 480, PixelType::U8x3);
    let mut resizer = Resizer::new();

    resizer.resize(&src_image, &mut dst_image, None)?;

    let pixels: Vec<f32> = dst_image
        .buffer()
        .to_vec()
        .into_iter()
        .map(|v| v as f32 / 255.0)
        .collect();

    let tensor =
        Tensor::new(&[1, 480, 480, 3]).with_values(&pixels)?;
    Ok(ProceededImage {
        path: image_path.as_ref().to_path_buf(),
        image_tensor: tensor
    })
}
