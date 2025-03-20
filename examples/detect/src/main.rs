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
    let speciesnet = SpeciesNet::new("../../assets/model/mega_detector_v5_cpu.pt")?;

    let results = speciesnet.detect(image_path)?;
    let mut loaded_image = ImageReader::open(image_path)?.decode()?.to_rgb8();
    let image_width = loaded_image.width();
    let image_height = loaded_image.height();

    println!("results {:?}", results);
    println!("width {}", image_width);
    println!("height {}", image_height);

    let rects = results
        .iter()
        .map(|d| {
            let (x1, y1, x2, y2) = d.bounding_box().as_xyxy_bounding_box();
            Rect::at(x1 as i32, y1 as i32).of_size((x2 - x1) as u32, (y2 - y1) as u32)
        })
        .collect::<Vec<_>>();

    let color = Rgb::from_slice(&[255, 0, 125]);
    for (i, rect) in rects.iter().enumerate() {
        draw_hollow_rect_mut(&mut loaded_image, *rect, *color);
        draw_text_mut(
            &mut loaded_image,
            *color,
            rect.left(),
            rect.top(),
            16.0,
            &font,
            format!("{:?}", results.get(i).unwrap().confidence()).as_str(),
        )
    }

    // Please be reminded that the image will not be overwritten if the original image exists.
    loaded_image.save("./bboxed_african_elephants.jpg")?;

    Ok(())
}
