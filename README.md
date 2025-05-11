## SpeciesNet Rust

This is the home of the `speciesnet` and `speciesnet-cli` crate for running an ensemble similar to [google/cameratrapai](https://github.com/google/cameratrapai) pipeline Rust language.

### Required tools

- [docker](https://docker.com) for running virtual environments to convert models to other formats.
- [rustup](https://rustup.rs) for running rust program.

### Setting up the environment

#### Downloading the cameratrapai models

Head over to [https://www.kaggle.com/models/google/speciesnet](https://www.kaggle.com/models/google/speciesnet), download the `v4.0.0a` model and extract the files to the `assets/model` folder (you should have `always_crop_[...].keras` and other files inside the `assets/model` folder).

Go to [https://github.com/agentmorris/MegaDetector/releases/tag/v5.0](https://github.com/agentmorris/MegaDetector/releases/tag/v5.0), download the `md_v5b.0.0.pt` weights for MDv5 and place it in the `assets/model` folder.

#### Converting the models

After you have downloaded the models, You need to convert the MegaDetector and Speciesnet model into [ONNX](https://onnx.ai) format by running the model conversion script inside the [converter](./converter/) directory. See [this README file](./converter/README.md) on how to convert the model. You will get the converted onnx model inside the `assets/model` folder. Now you're ready to load that model into Rust and start using it with Rust.

#### Setup environment variables

- Use `SPECIESNET_LOG` environment variable to control how logs are shown while running. As a starter you can set this to `debug` to show all logs. Please refer to [docs.rs/env_logger](https://docs.rs/env_logger) for more options and per-module log levels. For example

  ```
  export SPECIESNET_LOG=debug,ort=info
  ```

  This means to set the speciesnet CLI's log level to `debug`, and set the logs coming from [ort](https://github.com/pykeio/ort), our model running library, to `info`.

### Running the examples

There is an example program inside `examples/detect` folder that will run the detector on the example image. Go ahead and run it by running

```
cd examples/detect
cargo run
```

A window will be opened and you can use scroll wheel to inspect the image and its bounding boxes.

### Running the CLI

Run the help

```
cargo run -- --help
```

Run the whole pipeline

```
cargo run --bin speciesnet -- --instances-json assets/images/input.json --predictions-json assets/images/output_ensemble_test.json
```

Running only the detector

```
cargo run --bin speciesnet -- --instances-json assets/images/input.json --predictions-json assets/images/output_detector_test.json --detector-only
```

Running only the classifier

`cd` to directory of `instances-json` file 

```
cd assets/images

cargo run --release --bin speciesnet-cli -- --instances-json input.json --predictions-json output_classifier_test.json --detections-json output_detector_test.json --classifier-model ../model/model.onnx --detector-model ../model/md_v5a.0.0_traced.pt --classifier-only
```

Running only the ensemble

`filepath` in `instances-json`, `output_detector_test.json` and `output_classifier_test.json` must be the same so you can run `ensemble`

```
cargo run --bin speciesnet-cli -- --instances-json assets/images/input.json --predictions-json assets/images/output_ensemble_test.json --detections-json assets/images/output_detector_test.json --classifications-json assets/images/output_classifier_test.json --classifier-model assets/model/model.onnx --detector-model assets/model/md_v5a.0.0_traced.pt --ensemble-only 
```

### Testing CLI output

The CLI should output json that is similar to the original Python version of Speciesnet. The 3 `output_xyz.json` files in `assets/images` were generated from the original version and therefore are useful for comparison. Note: floating-point numbers won't be exactly the same because the model conversion and CPU differences.

To compare a JSON output from the CLI with the Python version, use:

```
cd assets/images
python compare.py output_detector_test.json output_detector.json
```

You can adjust the precision of the comparison using the constants (e.g. `DETECTION_CONF_DP = 3`) found in `compare.py`.
