use std::{io, path::Path};

use image::{ImageReader, RgbImage};

use crate::error::Error;

/// Converts a vector of raw RGB pixel into an [`RgbImage`], this is being used as a conversion
/// method from [`mozjpeg`].
///
/// # Panics
///
/// This function assumes the passed in pixels vector has a length of `width * height * 3`,
/// otherwise a panic will happen, this assumption allows us to get better performance.
fn vec_u8_to_rgb_image(pixels: Vec<u8>, width: usize, height: usize) -> RgbImage {
    // SAFETY: The image returned from other image functions are guaranteed to be RGB space. The
    // fucntion is also not exposed outside so it's safe to assume creation from raw vector will
    // not fail, iterating over chunks slows down hugely (we're looking at 2ms iteration vs 3ns raw
    // vector creation).
    RgbImage::from_raw(width as u32, height as u32, pixels).unwrap()
}

fn load_jpeg_image<P>(path: P) -> Result<RgbImage, std::io::Error>
where
    P: AsRef<Path>,
{
    let moz_image = mozjpeg::Decompress::with_markers(mozjpeg::ALL_MARKERS).from_path(path)?;
    let (moz_width, moz_height) = (moz_image.width(), moz_image.height());
    let moz_decoded_image = moz_image.rgb()?.read_scanlines::<u8>()?;
    let moz_rgb_image = vec_u8_to_rgb_image(moz_decoded_image, moz_width, moz_height);

    Ok(moz_rgb_image)
}

fn load_other_image<P>(path: P) -> Result<RgbImage, Error>
where
    P: AsRef<Path>,
{
    let loaded_image = ImageReader::open(path)?.decode()?;
    let loaded_rgb_image = loaded_image.into_rgb8();

    Ok(loaded_rgb_image)
}

/// Loads an image from the given path. It tries to determine the image format based on the file extension.
/// If the extension is "jpg" or "jpeg", it uses the [mozjpeg] decoder. Otherwise, it uses the default image decoder.
///
/// [mozjpeg]: https://crates.io/crates/mozjpeg
pub fn load_image<P>(path: P) -> Result<RgbImage, Error>
where
    P: AsRef<Path> + std::panic::UnwindSafe + std::panic::RefUnwindSafe,
{
    let Some(extension) = path.as_ref().extension() else {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "File extension not found").into());
    };

    match extension.to_string_lossy().to_lowercase().as_str() {
        "jpg" | "jpeg" => Ok(std::panic::catch_unwind(|| load_jpeg_image(path)).map_err(
            |e| {
                if let Some(cause) = e.downcast_ref::<&str>() {
                    Error::MozjpegPanicError(cause.to_string())
                } else if let Some(cause) = e.downcast_ref::<String>() {
                    Error::MozjpegPanicError(cause.clone())
                } else {
                    Error::MozjpegPanicError("Unknown panic type".to_string())
                }
            },
        )??),
        _ => load_other_image(path),
    }
}
