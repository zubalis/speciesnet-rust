use std::path::Path;
use std::sync::Arc;
use tensorflow::{Graph, SavedModelBundle, SessionOptions, SessionRunArgs, Tensor};

pub mod classifier;
pub mod constants;
pub mod error;
pub mod geofence;
pub mod image;
pub mod input;

use crate::error::Error;

#[derive(Debug, Clone)]
pub struct SpeciesNetClassifier {
    bundle: Arc<SavedModelBundle>,
    graph: Arc<Graph>,
}

impl SpeciesNetClassifier {
    /// Create classifier from given config
    pub fn new<P>(model_dir_path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let mut graph = Graph::new();
        let bundle = SavedModelBundle::load(
            &SessionOptions::new(),
            ["serve"],
            &mut graph,
            model_dir_path,
        )?;

        Ok(Self {
            bundle: Arc::new(bundle),
            graph: Arc::new(graph),
        })
    }

    /// run a classification from given input
    pub fn classify(&self, input_tensor: &Tensor<f32>) -> Result<Vec<f32>, Error> {
        let session = &self.bundle.session;
        let mut args = SessionRunArgs::new();

        let input_op = self
            .graph
            .operation_by_name_required("serving_default_input_2")?;
        let output_op = self
            .graph
            .operation_by_name_required("StatefulPartitionedCall")?;

        args.add_feed(&input_op, 0, input_tensor);

        let output_token = args.request_fetch(&output_op, 0);
        session.run(&mut args)?;

        let output_tensor: Tensor<f32> = args.fetch(output_token)?;
        let o_vec = output_tensor.to_vec();

        Ok(o_vec)
    }
}
