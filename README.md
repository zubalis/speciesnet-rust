## SpeciesNet Rust

A port of [google/cameratrapai](https://github.com/google/cameratrapai) (a.k.a. SpeciesNet, an ensemble of AI models for classifying wildlife in camera trap images) for Rust. Contains the `speciesnet` and `speciesnet-cli` crates for running a full ensemble as well as components individually.

Key differences:
- Uses an Onnx-converted version of the MegaDetector (converted from PyTorch) and the classifier (converted from TF) models
- Only requires the Onnx runtime library (thus reducing overall install footprint)
- CLI does not implement all the features of the Python SpeciesNet cli -- only supports `--input-json` for input, does not support inference resuming

See: [documentation](https://zubalis.github.io/speciesnet-rust/speciesnet)

### Using the library

The [speciesnet](./speciesnet/) library can be installed to other crates for running the ensemble by running

```bash
cargo add --git https://github.com/zubalis/speciesnet-rust.git --path speciesnet
cargo add --git https://github.com/zubalis/speciesnet-rust.git --path core
cargo add ort@=2.0.0-rc.9 -F download-binaries
```

inside your rust program.

Now, you can start running the speciesnet ensemble, the example below shows how you can run the ensemble (detector + classifier + ensemble) on a list of images.

```rust
use std::path::PathBuf;

use speciesnet_core::io::Instance;
use speciesnet::SpeciesNet;

let instances = vec![
    Instance::from_path_buf(PathBuf::from("./img1.jpeg")),
    Instance::from_path_buf(PathBuf::from("./img2.jpeg"))
];

let speciesnet = SpeciesNet::new()?;
let detections = speciesnet.predict(&instances)?;
```

### Using the CLI

The [speciesnet-cli](./speciesnet-cli/) is a CLI utility to run the ensemble through CLI, similar to `python3 -m speciesnet.scripts.run_model` script. You can install the CLI utility by running

```bash
cargo install --git https://github.com/zubalis/speciesnet-rust.git --path speciesnet-cli
```

Now, you can start running the speciesnet CLI, below is the example of how to run the ensemble a folder of input images.

```bash
speciesnet-cli --folders ./august-dataset --predictions-json ./output-august.json
```

### Developer setup

#### Required tools

- [docker](https://docker.com) for running virtual environments to convert models to other formats.
- [rustup](https://rustup.rs) for running rust program.

#### Setting up the environment

#### Setup environment variables

- Use `SPECIESNET_LOG` environment variable to control how logs are shown while running. As a starter you can set this to `debug` to show all logs. Please refer to [docs.rs/env_logger](https://docs.rs/env_logger) for more options and per-module log levels. For example

  ```bash
  export SPECIESNET_LOG=debug,ort=info
  ```

  This means to set the speciesnet CLI's log level to `debug`, and set the logs coming from [ort](https://github.com/pykeio/ort), our model running library, to `info`.

### Running the CLI

Please refer to this [README](./speciesnet-cli/README.md) for how to run the `speciesnet-cli` compared to the python version. Whilst developing, you can run the CLI without reinstalling the program every time by swapping the `speciesnet-cli` part to `cargo run` instead, for example, this is the command used to run the detector.

```bash
speciesnet-cli --instance-json ./instance.json --predictions_json ./predictions.json --detector-only
```

While developing, you can run the command inside `speciesnet-cli` like this

```bash
cd speciesnet-cli
cargo run -- --instance-json ./instance.json --predictions_json ./predictions.json --detector-only
```

### Running the examples

There is an example program inside `examples/detect` folder that will run the detector on the example image. Go ahead and run it by running

```bash
cd examples/detect
cargo run
```

A window will be opened and you can use scroll wheel to inspect the image and its bounding boxes.

Make sure you're inside the `speciesnet-cli`.

### Testing CLI output

The CLI should output json that is similar to the original Python version of SpeciesNet. The 3 `output_xyz.json` files in `assets/images` were generated from the original version and therefore are useful for comparison. 

> [!NOTE]
> floating-point numbers won't be exactly the same because the model conversion and CPU differences.

To compare a JSON output from the CLI with the Python version, use:

```bash
cd assets/images
python compare.py output_detector_test.json output_detector.json
```

You can adjust the precision of the comparison using the constants (e.g. `DETECTION_CONF_MSE_THRESHOLD = 0.01`) found in `compare.py`.
