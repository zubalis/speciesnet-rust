use std::path::{Path, PathBuf};

use image::{
    DynamicImage, GenericImageView, Rgb, RgbImage,
    imageops::{FilterType, replace},
};
use ndarray::Array4;
use speciesnet_core::{image_reader::load_image, shape::Shape};
use tracing::{debug, info};

use crate::error::Error;

#[derive(Debug, Clone)]
pub struct PreprocessedImage {
    inner: PreprocessedImageInner,
    path: PathBuf,
}

impl PreprocessedImage {
    pub fn new<P>(image: PreprocessedImageInner, path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            path: path.as_ref().to_path_buf(),
            inner: image,
        }
    }

    pub fn image(&self) -> &RgbImage {
        &self.inner.image
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn path_owned(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn original_size(&self) -> (u32, u32) {
        self.inner.original_size
    }

    pub fn resized_size(&self) -> (u32, u32) {
        self.inner.resized_size
    }

    fn fill_array_with_image_content(array: &mut Array4<f32>, image: &RgbImage) {
        for pixel in image.enumerate_pixels() {
            let x = pixel.0 as _;
            let y = pixel.1 as _;
            let [r, g, b] = pixel.2.0;
            array[[0, 0, y, x]] = (r as f32) / 255.;
            array[[0, 1, y, x]] = (g as f32) / 255.;
            array[[0, 2, y, x]] = (b as f32) / 255.;
        }
    }

    pub fn into_tensor(self) -> Array4<f32> {
        let inner_image = self.inner.image;

        let mut tensor: Array4<f32> = Array4::zeros([
            1usize,
            3usize,
            inner_image.height() as usize,
            inner_image.width() as usize,
        ]);

        Self::fill_array_with_image_content(&mut tensor, &inner_image);
        tensor
    }
}

/// Struct to store preprocessed image (resized and bordered) along with their original width and
/// height for rescaling the borders.
#[derive(Debug, Clone)]
pub struct PreprocessedImageInner {
    image: RgbImage,
    original_size: (u32, u32),
    resized_size: (u32, u32),
}

impl PreprocessedImageInner {
    pub fn new(image: RgbImage, original_size: (u32, u32), resized_size: (u32, u32)) -> Self {
        Self {
            image,
            original_size,
            resized_size,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LetterboxOptionsBuilder {
    shape: Shape,
    scale_up: bool,
    auto: bool,
    stride: u32,
    scale_fill: bool,
    color: Rgb<u8>,
}

impl Default for LetterboxOptionsBuilder {
    fn default() -> Self {
        Self {
            shape: Shape::Square(640),
            scale_up: true,
            auto: true,
            stride: 64,
            scale_fill: false,
            color: Rgb([114, 114, 114]),
        }
    }
}

impl LetterboxOptionsBuilder {
    pub fn shape(&mut self, shape: Shape) -> &mut Self {
        self.shape = shape;
        self
    }

    pub fn scale_up(&mut self, scale_up: bool) -> &mut Self {
        self.scale_up = scale_up;
        self
    }

    pub fn auto(&mut self, auto: bool) -> &mut Self {
        self.auto = auto;
        self
    }

    pub fn stride(&mut self, stride: u32) -> &mut Self {
        self.stride = stride;
        self
    }

    pub fn scale_fill(&mut self, scale_fill: bool) -> &mut Self {
        self.scale_fill = scale_fill;
        self
    }

    pub fn color(&mut self, color: Rgb<u8>) -> &mut Self {
        self.color = color;
        self
    }

    pub fn build(&self) -> LetterboxOptions {
        LetterboxOptions {
            shape: self.shape,
            scale_up: self.scale_up,
            auto: self.auto,
            stride: self.stride,
            scale_fill: self.scale_fill,
            color: self.color,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LetterboxOptions {
    shape: Shape,
    scale_up: bool,
    auto: bool,
    stride: u32,
    scale_fill: bool,
    color: Rgb<u8>,
}

impl LetterboxOptions {
    pub fn builder() -> LetterboxOptionsBuilder {
        LetterboxOptionsBuilder::default()
    }

    pub fn shape(&self) -> Shape {
        self.shape
    }

    pub fn scale_up(&self) -> bool {
        self.scale_up
    }

    pub fn auto(&self) -> bool {
        self.auto
    }

    pub fn stride(&self) -> u32 {
        self.stride
    }

    pub fn scale_fill(&self) -> bool {
        self.scale_fill
    }

    pub fn color(&self) -> Rgb<u8> {
        self.color
    }
}

/// Loads the image and runs the image through a preprocessor where the image gets resized and
/// compensated for their missing strides.
pub fn preprocess<P>(image_path: P) -> Result<PreprocessedImage, Error>
where
    P: AsRef<Path>,
{
    info!("Loading and decoding {}.", image_path.as_ref().display());
    let loaded_image = load_image(&image_path)?;
    let options = LetterboxOptions::builder()
        .shape(speciesnet_core::shape::Shape::Square(
            speciesnet_core::constants::detector::IMAGE_HEIGHT,
        ))
        .build();

    info!("Resizing and letterboxing the image.");
    let preprocessed_image = letterbox(loaded_image.into(), options)?;

    Ok(PreprocessedImage::new(preprocessed_image, image_path))
}

/// Resize an image while meeting stride-multiple constraints.
pub fn letterbox(
    input_image: DynamicImage, // TODO: Change to RgbImage
    options: LetterboxOptions,
) -> Result<PreprocessedImageInner, Error> {
    let input_image_dimensions = input_image.dimensions();
    let mut input_image = input_image;

    debug!("im.shape shape is {:?}", input_image.dimensions());

    let new_shape = (options.shape.width(), options.shape.height());

    debug!("new shape is {:?}", new_shape);
    let mut r: f32 = f32::min(
        new_shape.0 as f32 / input_image_dimensions.0 as f32,
        new_shape.1 as f32 / input_image_dimensions.1 as f32,
    );
    debug!("Scale ratio is {}", r);

    if !options.scale_up {
        r = f32::min(r, 1.0f32);
    }

    // Compute padding
    let mut ratio: (f32, f32) = (r, r);
    let mut new_unpad: (f32, f32) = (
        (input_image.dimensions().0 as f32 * r).round(),
        (input_image.dimensions().1 as f32 * r).round(),
    );

    // dw, dh
    let mut padded = (
        new_shape.1 as f32 - new_unpad.0,
        new_shape.0 as f32 - new_unpad.1,
    );

    if options.auto {
        debug!("Auto padding is true");

        // rem_euclid is same as doing a modulo.
        padded.0 = padded.0.rem_euclid(options.stride as f32);
        padded.1 = padded.1.rem_euclid(options.stride as f32);
    } else if options.scale_fill {
        ratio.0 = 0.0;
        ratio.1 = 0.0;
        new_unpad = (new_shape.1 as f32, new_shape.0 as f32);
        ratio = (
            new_shape.1 as f32 / input_image_dimensions.1 as f32,
            new_shape.0 as f32 / input_image_dimensions.0 as f32,
        );
    }

    debug!("New unpad is {:?}", new_unpad);

    padded.0 /= 2.0;
    padded.1 /= 2.0;

    if input_image.dimensions().0 != new_unpad.0.round() as u32
        || input_image.dimensions().1 != new_unpad.1.round() as u32
    {
        debug!("The image needs a resize.");

        input_image =
            input_image.resize_exact(new_unpad.0 as u32, new_unpad.1 as u32, FilterType::Triangle);
    }

    debug!("Calculating the border to patch the input image.");
    let (top, left, bottom, right): (u32, u32, u32, u32) = (
        (padded.1 - 0.1).round() as u32,
        (padded.0 - 0.1).round() as u32,
        (padded.1 + 0.1).round() as u32,
        (padded.0 + 0.1).round() as u32,
    );

    if top == 0 && left == 0 && bottom == 0 && right == 0 {
        debug!("The image does not need resizing anymore, returning the image.");
        let preprocessed_image = PreprocessedImageInner::new(
            input_image.into(),
            input_image_dimensions,
            (new_unpad.0 as u32, new_unpad.1 as u32),
        );
        return Ok(preprocessed_image);
    }

    debug!(
        "Needs to extend the image by top {}, left {}, bottom {}, right {}",
        top, left, bottom, right
    );

    // Creates a bigger image with the unpadded width and height.
    let mut blank_image = RgbImage::new(
        new_unpad.0.round() as u32 + left + right,
        new_unpad.1.round() as u32 + top + bottom,
    );

    // Same as `cv2.copyMakeBorder()`
    let color = Rgb([114, 114, 114]);

    for p in blank_image.pixels_mut() {
        *p = color;
    }

    // Replace pixels of the smaller image on top.
    {
        replace(
            &mut blank_image,
            &input_image.to_rgb8(),
            left.into(),
            top.into(),
        );
    }

    debug!("Final preprocessed image {:?} with original size as", ratio);
    let blank_image_dimensions = blank_image.dimensions();
    let preprocessed_image =
        PreprocessedImageInner::new(blank_image, input_image_dimensions, blank_image_dimensions);

    Ok(preprocessed_image)
}
