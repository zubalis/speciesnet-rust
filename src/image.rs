use std::error::Error;
use tensorflow::{Tensor};
use image::{io::Reader};

#[derive(Debug)]
pub struct ProceededImages {
    pub success_images: SuccessImages,
    pub failed_images: Vec<FailedImage>,
}

#[derive(Debug)]
pub struct SuccessImages {
    pub paths: Vec<String>,
    pub image_tensor: Tensor<f32>,
}

#[derive(Debug)]
struct FailedImage {
    path: String,
    error_message: String,
}

pub fn load_and_preprocess_images(image_paths: Vec<String>) -> Result<ProceededImages, Box<dyn Error>> {
    let mut success_images: Vec<Vec<f32>> = Vec::new();
    let mut success_images_paths: Vec<String> = Vec::new();
    let mut failed_images: Vec<FailedImage> = Vec::new();

    for image_path in image_paths.iter() {
        match Reader::open(image_path) {
            Ok(img) => match img.decode() {
                Ok(decoded_img) => {
                    let resized_img = decoded_img.resize_exact(480, 480, image::imageops::FilterType::Triangle);
                    let img_rgb = resized_img.to_rgb8();
                    let pixels: Vec<f32> = img_rgb.pixels()
                        .flat_map(|p| p.0)
                        .map(|v| v as f32 / 255.0)
                        .collect();

                    success_images.push(pixels);
                    success_images_paths.push(image_path.to_string());
                }
                Err(e) => failed_images.push(FailedImage {
                    path: image_path.clone(),
                    error_message: e.to_string(),
                }),
            },
            Err(e) => failed_images.push(FailedImage {
                path: image_path.clone(),
                error_message: e.to_string(),
            }),
        }
    }

    let success_counts = success_images.len();
    let flatted_success_images: Vec<f32> = success_images.into_iter().flatten().collect();
    let tensor = Tensor::new(&[success_counts as u64, 480, 480, 3]).with_values(&flatted_success_images)?;


    Ok(ProceededImages {
        success_images: SuccessImages {
            paths: success_images_paths,
            image_tensor: tensor,
        },
        failed_images
    })
}