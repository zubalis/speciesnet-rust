use std::fs::read;
use std::path::PathBuf;

use image::{ImageReader, RgbImage};

use serde::Deserialize;

#[derive(Deserialize)]
struct Pixel {
    x: u32,
    y: u32,
    r: u8,
    g: u8,
    b: u8,
}

fn vec_u8_to_rgb_image(vec: Vec<u8>, width: usize, height: usize) -> RgbImage {
    let mut rgb_image = RgbImage::new(width.try_into().unwrap(), height.try_into().unwrap());
    let component_count = vec.len() / (width * height);
    assert_eq!(component_count, 3); // only handles RGB values
    for (i, chunk) in vec.chunks_exact(component_count).enumerate() {
        let x = (i % width) as u32;
        let y = (i / width) as u32;
        rgb_image.put_pixel(x, y, image::Rgb([chunk[0], chunk[1], chunk[2]]));
    }
    rgb_image
}

fn main() {
    println!("Loading image...");
    let image_path = PathBuf::from("S49-C68_IMG_0128.JPG");
    let loaded_image = ImageReader::open(&image_path).unwrap().decode().unwrap();
    let loaded_rgb_image = loaded_image.as_rgb8().unwrap();
    println!(
        "Image is {}x{}",
        loaded_rgb_image.width(),
        loaded_rgb_image.height()
    );
    println!("Image is {} bytes", loaded_rgb_image.len());

    println!("Loading image with zune...");
    let image_data = read(image_path.clone()).unwrap();
    let mut zune_decoder = zune_jpeg::JpegDecoder::new(&image_data);
    let zune_decoded_image = zune_decoder.decode().unwrap();
    let (zune_width, zune_height) = zune_decoder.dimensions().unwrap();
    let zune_rgb_image = vec_u8_to_rgb_image(zune_decoded_image, zune_width, zune_height);

    println!("Loading image with mozjpeg...");
    let moz_image = mozjpeg::Decompress::with_markers(mozjpeg::ALL_MARKERS)
        .from_path(image_path)
        .unwrap();
    let (moz_width, moz_height) = (moz_image.width(), moz_image.height());
    let moz_decoded_image = moz_image.rgb().unwrap().read_scanlines::<u8>().unwrap();
    let moz_rgb_image = vec_u8_to_rgb_image(moz_decoded_image, moz_width, moz_height);

    println!("Loading reference image... (might take some time)");
    let json_path = PathBuf::from("S49-C68_IMG_0128.JPG.json");
    let json_file = std::fs::File::open(&json_path).unwrap();
    let json: Vec<Pixel> = serde_json::from_reader(json_file).unwrap();
    let width = json.iter().fold(0, |acc, pixel| acc.max(pixel.x + 1));
    let height = json.iter().fold(0, |acc, pixel| acc.max(pixel.y + 1));
    let mut reference_image = RgbImage::new(width, height);
    for pixel in json {
        reference_image.put_pixel(pixel.x, pixel.y, image::Rgb([pixel.r, pixel.g, pixel.b]));
    }
    println!(
        "Reference image is {}x{}",
        reference_image.width(),
        reference_image.height()
    );
    println!(
        "Reference image is {} bytes",
        reference_image.as_raw().len()
    );

    println!("Comparing images...");
    assert_eq!(loaded_rgb_image.width(), reference_image.width());
    assert_eq!(loaded_rgb_image.height(), reference_image.height());
    let error_sum = loaded_rgb_image
        .pixels()
        .zip(reference_image.pixels())
        .map(|(pix_1, pix_2)| (0..3).map(move |i| pix_1[i] as i64 - pix_2[i] as i64))
        .flatten()
        .map(|diff| diff * diff)
        .sum::<i64>();
    let error_mean =
        error_sum as f64 / (loaded_rgb_image.width() * loaded_rgb_image.height()) as f64;
    println!("Mean squared error (image vs ref): {:.6}", error_mean);

    assert_eq!(zune_rgb_image.width(), reference_image.width());
    assert_eq!(zune_rgb_image.height(), reference_image.height());
    let error_sum = zune_rgb_image
        .pixels()
        .zip(reference_image.pixels())
        .map(|(pix_1, pix_2)| (0..3).map(move |i| pix_1[i] as i64 - pix_2[i] as i64))
        .flatten()
        .map(|diff| diff * diff)
        .sum::<i64>();
    let error_mean = error_sum as f64 / (zune_rgb_image.width() * zune_rgb_image.height()) as f64;
    println!("Mean squared error (zune vs ref): {:.6}", error_mean);

    assert_eq!(zune_rgb_image.width(), loaded_rgb_image.width());
    assert_eq!(zune_rgb_image.height(), loaded_rgb_image.height());
    let error_sum = zune_rgb_image
        .pixels()
        .zip(loaded_rgb_image.pixels())
        .map(|(pix_1, pix_2)| (0..3).map(move |i| pix_1[i] as i64 - pix_2[i] as i64))
        .flatten()
        .map(|diff| diff * diff)
        .sum::<i64>();
    let error_mean = error_sum as f64 / (zune_rgb_image.width() * zune_rgb_image.height()) as f64;
    println!("Mean squared error (zune vs image): {:.6}", error_mean);

    assert_eq!(moz_rgb_image.width(), reference_image.width());
    assert_eq!(moz_rgb_image.height(), reference_image.height());
    let error_sum = moz_rgb_image
        .pixels()
        .zip(reference_image.pixels())
        .map(|(pix_1, pix_2)| (0..3).map(move |i| pix_1[i] as i64 - pix_2[i] as i64))
        .flatten()
        .map(|diff| diff * diff)
        .sum::<i64>();
    let error_mean = error_sum as f64 / (moz_rgb_image.width() * moz_rgb_image.height()) as f64;
    println!("Mean squared error (moz vs ref): {:.6}", error_mean);
}
