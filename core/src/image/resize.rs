use image::{Rgb, RgbImage};

/// Value used for downscaling the image.
///
/// This function is copied from tensorflow's [tf.image.convert_image_dtype] function, specifically
/// [this bit] that contains the casting and implementation on how the [`u8`] gets converted to [`f32`].
///
/// [tf.image.convert_image_dtype]: https://github.com/tensorflow/tensorflow/blob/4ac8ed89935501c0ad38b9563f42e7bd4d63b086/tensorflow/python/ops/image_ops_impl.py#L2382-L2561
/// [this bit]: https://github.com/tensorflow/tensorflow/blob/4ac8ed89935501c0ad38b9563f42e7bd4d63b086/tensorflow/python/ops/image_ops_impl.py#L2549-L2553
const DOWN_SCALER: f32 = 1.0 / u8::MAX as f32;

/// Value that tensorflow used during the upscaler when converting [`f32`] image to [`u8`]. This
/// code is copied to match tensorflow's [tf.image.convert_image_dtype] function, specifally [this
/// bit].
///
/// [tf.image.convert_image_dtype]: https://github.com/tensorflow/tensorflow/blob/4ac8ed89935501c0ad38b9563f42e7bd4d63b086/tensorflow/python/ops/image_ops_impl.py#L2382-L2561
/// [this bit]: https://github.com/tensorflow/tensorflow/blob/4ac8ed89935501c0ad38b9563f42e7bd4d63b086/tensorflow/python/ops/image_ops_impl.py#L2555-L2561
const UP_SCALER: f32 = (u8::MAX as f32) + 0.5;

fn biinear_interpolate_pixel(
    y_linear_interpolate: f32,
    x_linear_interpolate: f32,
    top_left: f32,
    top_right: f32,
    bottom_left: f32,
    bottom_right: f32,
) -> f32 {
    let top = top_left + (top_right - top_left) * x_linear_interpolate;
    let bottom = bottom_left + (bottom_right - bottom_left) * x_linear_interpolate;

    top + (bottom - top) * y_linear_interpolate
}

/// Function to do [bilinear interpolate] on the pixel.
///
/// [bilinear interpolate]: https://en.wikipedia.org/wiki/Bilinear_interpolation
fn bilinear_interpolate(
    y_linear_interpolate: f32,
    x_linear_interpolate: f32,
    top_left: &Rgb<u8>,
    top_right: &Rgb<u8>,
    bottom_left: &Rgb<u8>,
    bottom_right: &Rgb<u8>,
) -> Rgb<u8> {
    let top_left_r: f32 = top_left.0[0] as f32 * DOWN_SCALER;
    let top_right_r: f32 = top_right.0[0] as f32 * DOWN_SCALER;
    let bottom_left_r: f32 = bottom_left.0[0] as f32 * DOWN_SCALER;
    let bottom_right_r: f32 = bottom_right.0[0] as f32 * DOWN_SCALER;

    let r = biinear_interpolate_pixel(
        y_linear_interpolate,
        x_linear_interpolate,
        top_left_r,
        top_right_r,
        bottom_left_r,
        bottom_right_r,
    );

    let top_left_g: f32 = top_left.0[1] as f32 * DOWN_SCALER;
    let top_right_g: f32 = top_right.0[1] as f32 * DOWN_SCALER;
    let bottom_left_g: f32 = bottom_left.0[1] as f32 * DOWN_SCALER;
    let bottom_right_g: f32 = bottom_right.0[1] as f32 * DOWN_SCALER;

    let g = biinear_interpolate_pixel(
        y_linear_interpolate,
        x_linear_interpolate,
        top_left_g,
        top_right_g,
        bottom_left_g,
        bottom_right_g,
    );

    let top_left_b: f32 = top_left.0[2] as f32 * DOWN_SCALER;
    let top_right_b: f32 = top_right.0[2] as f32 * DOWN_SCALER;
    let bottom_left_b: f32 = bottom_left.0[2] as f32 * DOWN_SCALER;
    let bottom_right_b: f32 = bottom_right.0[2] as f32 * DOWN_SCALER;

    let b = biinear_interpolate_pixel(
        y_linear_interpolate,
        x_linear_interpolate,
        top_left_b,
        top_right_b,
        bottom_left_b,
        bottom_right_b,
    );

    Rgb([
        (r * UP_SCALER) as u8,
        (g * UP_SCALER) as u8,
        (b * UP_SCALER) as u8,
    ])
}

/// Calculates the scale of the image resizing.
///
/// This code is adapted from tensorflow's [CalculateResizeScale function].
///
/// [CalculateResizeScale function]: https://github.com/tensorflow/tensorflow/blob/4ac8ed89935501c0ad38b9563f42e7bd4d63b086/tensorflow/core/util/image_resizer_state.h#L40-L46
fn calculate_resize_scale(size: u32, nsize: u32, align_corners: bool) -> f32 {
    if align_corners && size > 1 && nsize > 1 {
        return (size as f32 - 1.0) / (nsize as f32 - 1.0);
    }

    size as f32 / nsize as f32
}

/// Pixel corners finding that assumes the original pixel is at 0.5.
///
/// This function is adapted from Tensorflow's [HalfPixelScaler struct].
///
/// [HalfPixelScaler struct]: https://github.com/tensorflow/tensorflow/blob/4ac8ed89935501c0ad38b9563f42e7bd4d63b086/tensorflow/core/util/image_resizer_state.h#L48-L57
fn half_pixel_scaler(index: u32, scale: f32) -> f32 {
    ((index as f32 + 0.5) * scale) - 0.5
}

/// A custom resize function using [bilinear interpolation], this function is adapted from
/// tensorflow's [tf.image.resize] function. The code inside is taken and traced from the library
/// itself to make this function's output matches with the tensorflow's output.
///
/// [bilinear interpolation]: https://en.wikipedia.org/wiki/Bilinear_interpolation
/// [tf.image.resize]: https://www.tensorflow.org/api_docs/python/tf/image/resize
pub fn bilinear_resize(
    image: RgbImage,
    nwidth: u32,
    nheight: u32,
    align_corners: bool,
) -> RgbImage {
    let (width, height) = image.dimensions();
    let height_scale = calculate_resize_scale(height, nheight, align_corners);
    let width_scale = calculate_resize_scale(width, nwidth, align_corners);

    let mut output_image: RgbImage = RgbImage::new(nwidth, nheight);

    for y in 0..nheight {
        let in_y: f32 = half_pixel_scaler(y, height_scale);
        let top_y_index: f32 = f32::floor(in_y);
        let bottom_y_index: f32 = f32::min(f32::ceil(in_y), (height - 1) as f32);

        let y_linear_interpolate: f32 = in_y - top_y_index;

        for x in 0..nwidth {
            let in_x: f32 = half_pixel_scaler(x, width_scale);
            let left_x_index: f32 = f32::floor(in_x);
            let right_x_index: f32 = f32::min(f32::ceil(in_x), (width - 1) as f32);

            let x_linear_interpolate: f32 = in_x - left_x_index;

            let top_left = image.get_pixel(left_x_index as u32, top_y_index as u32);
            let top_right = image.get_pixel(right_x_index as u32, top_y_index as u32);
            let bottom_left = image.get_pixel(left_x_index as u32, bottom_y_index as u32);
            let bottom_right = image.get_pixel(right_x_index as u32, bottom_y_index as u32);

            let interpolated_pixel = bilinear_interpolate(
                y_linear_interpolate,
                x_linear_interpolate,
                top_left,
                top_right,
                bottom_left,
                bottom_right,
            );

            output_image.put_pixel(x, y, interpolated_pixel);
        }
    }

    output_image
}
