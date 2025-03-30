use image::{DynamicImage, GenericImageView, ImageReader};
use ndarray::{Array4, ArrayBase, Dim, OwnedRepr};
use ort::{
    session::{Session, SessionOutputs, builder::GraphOptimizationLevel},
    value::Tensor,
};
use speciesnet_core::{BoundingBox, Category, Detection};
use tracing::info;
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};
use yolo::non_max_suppression;

mod torchvision;
mod yolo;

pub fn fill_array_with_image_content(array: &mut Array4<f32>, image: &DynamicImage) {
    for pixel in image.pixels() {
        let x = pixel.0 as _;
        let y = pixel.1 as _;
        let [r, g, b, _] = pixel.2.0;
        array[[0, 0, y, x]] = (r as f32) / 255.;
        array[[0, 1, y, x]] = (g as f32) / 255.;
        array[[0, 2, y, x]] = (b as f32) / 255.;
    }
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug,ort=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_ansi(true))
        .init();

    info!("Program started.");

    let model = Session::builder()?
        .with_optimization_level(GraphOptimizationLevel::Level3)?
        .with_intra_threads(4)?
        .commit_from_file("../assets/model/md_v5a.0.0.onnx")?;

    info!("Model is loaded.");

    info!("Loading image.");

    let image1: DynamicImage = ImageReader::open("./processed_african_elephants.jpg")?.decode()?;

    info!("Images loaded.");

    let mut image1_input: Array4<f32> = Array4::zeros([
        1usize,
        3usize,
        image1.height() as usize,
        image1.width() as usize,
    ]);

    fill_array_with_image_content(&mut image1_input, &image1);
    info!("Image1 loaded.");
    //fill_array_with_image_content(&mut image2_input, &image2);

    info!("Image loaded into ndarray.");

    info!("Inferencing images.");

    let outputs: SessionOutputs =
        model.run(ort::inputs!["images" => Tensor::from_array(image1_input)?]?)?;
    let output = outputs
        .get("output")
        .unwrap()
        .try_extract_tensor::<f32>()?
        .view()
        .into_owned();

    //info!("We got some results!!! {:?}", output);
    //info!("We got more outputs!!! {:?}", output2);

    let results = non_max_suppression(output, Some(0.01))?;

    let mut detections: Vec<Detection> = Vec::new();
    for raw_detection in results.rows() {
        // The get with unwrap is safe to to so as the dimension of model is of length 6.
        let x1: f64 = f64::from(*raw_detection.get(0).unwrap());
        let y1: f64 = f64::from(*raw_detection.get(1).unwrap());
        let x2: f64 = f64::from(*raw_detection.get(2).unwrap());
        let y2: f64 = f64::from(*raw_detection.get(3).unwrap());

        let confidence = *raw_detection.get(4).unwrap();
        let category = Category::try_from(*raw_detection.get(5).unwrap() + 1.0).unwrap();

        let bbox = BoundingBox::new(x1, y1, x2, y2)
            .scale_to(1280, 960, 2048, 1536)
            .normalize(2048, 1536);

        detections.push(Detection::new(category, confidence.into(), bbox));
    }

    for det in detections {
        info!("{}", det);
    }

    info!("Program finished.");

    Ok(())
}
