use ab_glyph::FontArc;
use image::{ImageReader, Pixel, Rgb};
use imageproc::{
    drawing::{draw_hollow_rect_mut, draw_text_mut},
    rect::Rect,
};
use speciesnet::speciesnet::SpeciesNet;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let font = FontArc::try_from_slice(include_bytes!("../Exo2-Light.ttf"))?;

    let image_path = "../../assets/images/african_elephants.jpg";
    let speciesnet = SpeciesNet::new("../../assets/model/md_v5a.0.0_traced.pt")?;

    let results = speciesnet.detect(&[image_path.into()])?;
    let mut loaded_image = ImageReader::open(image_path)?.decode()?.to_rgb8();
    let image_width = loaded_image.width();
    let image_height = loaded_image.height();

    println!("results {:?}", results);
    println!("width {}", image_width);
    println!("height {}", image_height);

    let mut rectangles: Vec<Rect> = Vec::new();
    for result in &results {
        if let Some(detections) = result.detections() {
            for detection in detections {
                let (x1, y1, x2, y2) = detection.bounding_box().as_xyxy_bounding_box();
                let scaled_x1: i32 = (x1 * loaded_image.width() as f64) as i32;
                let scaled_y1: i32 = (y1 * loaded_image.height() as f64) as i32;
                let scaled_x2: i32 = (x2 * loaded_image.width() as f64) as i32;
                let scaled_y2: i32 = (y2 * loaded_image.height() as f64) as i32;

                rectangles.push(Rect::at(scaled_x1, scaled_y1).of_size(
                    (scaled_x2 - scaled_x1) as u32,
                    (scaled_y2 - scaled_y1) as u32,
                ));
            }
        }
    }

    let detections = results.first().unwrap().detections().as_ref().unwrap();
    let color = Rgb::from_slice(&[255, 0, 125]);
    for (rectangle, result) in std::iter::zip(rectangles, detections) {
        draw_hollow_rect_mut(&mut loaded_image, rectangle, *color);
        draw_text_mut(
            &mut loaded_image,
            *color,
            rectangle.left(),
            rectangle.top(),
            16.0,
            &font,
            format!("{:?}", result.confidence()).as_str(),
        )
    }

    // Please be reminded that the image will not be overwritten if the original image exists.
    loaded_image.save("./bboxed_african_elephants.jpg")?;

    Ok(())
}
