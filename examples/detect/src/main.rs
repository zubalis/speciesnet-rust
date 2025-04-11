use std::{fs::File, path::PathBuf};

use font_kit::loader::Loader;
use image::ImageReader;
use raqote::{DrawOptions, DrawTarget, LineJoin, PathBuilder, Point, Source, StrokeStyle};
use show_image::{
    AsImageView, WindowOptions,
    event::{VirtualKeyCode, WindowEvent},
};
use speciesnet::speciesnet::SpeciesNet;
use speciesnet_core::Instance;
use tracing::info;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

#[show_image::main]
fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_ansi(true))
        .init();

    let image_path = PathBuf::from("../../assets/images/african_elephants.jpg");
    let speciesnet = SpeciesNet::new(
        "../../assets/model/md_v5a.0.0.onnx",
        "../../assets/model/",
        "../../assets/geofence_base.json",
        "../../assets/geofence_fixes.csv",
        "../../assets/taxonomy_release.txt",
    )?;

    info!(
        "Running detector on the example image {}.",
        image_path.display()
    );
    let results = speciesnet.detect(&[Instance::from_path_buf(image_path.clone())])?;

    info!("Loading the original image {}.", image_path.display());
    let loaded_image = ImageReader::open(&image_path)?.decode()?.to_rgb8();
    let image_width = loaded_image.width();
    let image_height = loaded_image.height();

    let bbox_colour = raqote::Color::new(0xff, 0x64, 0x53, 0x94);
    let font = Loader::from_file(&mut File::open("./Roboto.ttf")?, 0)?;

    info!("Drawing the image with result bounding boxes.");
    let mut draw_target = DrawTarget::new(image_width as i32, image_height as i32);

    for detection in results.first().unwrap().detections().clone().unwrap() {
        let mut path_builder = PathBuilder::new();

        let (x1, y1, x2, y2) = detection.bounding_box().as_xyxy_bounding_box();
        let scaled_x1: f32 = (x1 * loaded_image.width() as f64) as f32;
        let scaled_y1: f32 = (y1 * loaded_image.height() as f64) as f32;
        let scaled_x2: f32 = (x2 * loaded_image.width() as f64) as f32;
        let scaled_y2: f32 = (y2 * loaded_image.height() as f64) as f32;

        info!(
            "Adding bounding box at ({}, {}), ({}, {}) on the image.",
            scaled_x1, scaled_y1, scaled_x2, scaled_y2
        );

        path_builder.rect(
            scaled_x1,
            scaled_y1,
            scaled_x2 - scaled_x1,
            scaled_y2 - scaled_y1,
        );

        let path = path_builder.finish();

        draw_target.stroke(
            &path,
            &bbox_colour.into(),
            &raqote::StrokeStyle {
                join: LineJoin::Round,
                width: 4.,
                ..StrokeStyle::default()
            },
            &DrawOptions::new(),
        );

        draw_target.draw_text(
            &font,
            30.,
            &format!("{:.5}", detection.confidence()),
            Point::new(scaled_x1, scaled_y1 - 10.),
            &Source::Solid(bbox_colour.into()),
            &DrawOptions::new(),
        )
    }

    info!("Initializing the overlay for opening the image.");
    let overlay: show_image::Image = draw_target.into();

    info!("Creating the window proxy.");
    let window = show_image::context()
        .run_function_wait(move |context| -> Result<_, String> {
            let mut wd = context
                .create_window(
                    format!("SpeciesNet Detector {}", image_path.display()),
                    WindowOptions {
                        size: Some([image_width, image_height]),
                        default_controls: true,
                        ..WindowOptions::default()
                    },
                )
                .map_err(|e| e.to_string())?;

            wd.set_image(
                "animal",
                &loaded_image.as_image_view().map_err(|e| e.to_string())?,
            );
            wd.set_overlay(
                "yolo",
                &overlay.as_image_view().map_err(|e| e.to_string())?,
                true,
            );

            Ok(wd.proxy())
        })
        .unwrap();

    info!("Registering the event channel for escape events or closing events.");
    for event in window.event_channel()? {
        if let WindowEvent::KeyboardInput(event) = event {
            if event.input.key_code == Some(VirtualKeyCode::Escape)
                && event.input.state.is_pressed()
            {
                break;
            }
        }
    }

    Ok(())
}
