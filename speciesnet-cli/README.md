## SpeciesNet CLI

`speciesnet-cli` is a CLI for running a cameratrapai ensemble written in Rust.

### Installation

#### Cargo

```bash
cargo install --git https://github.com/zubalis/speciesnet-rust.git --path speciesnet-cli
```

### Cargo features

- `download-model`, enabled by default, downloads the [ONNX](https://onnx.ai)-converted detector and classifier model automatically to be used, you will have to manually pass the path to the extracted model folder in order to run the program if this feature is turned off.

### Usage

The CLI is designed to be pretty similar to how [google/cameratrapai](https://github.com/google/cameratrapai) is, except there are some differences as we're working towards the python equivalent.

- The input instance file only support `filepath`, `country`, and `admin1_region` keys.
- The classifier output only support the `classes`, and `scores` key.
- Other keys of `predictions` are still not supported on both reading from and writing to files.
- The Rust version does not override or edit an existing `predictions.json` file, if one is found when supplied using `--predictions-json`, CLI will error saying the file already existed.
- The CLI flag `--country` and `--admin1-region` currently does nothing to the input.

below is the examples of running the ensemble using speciesnet compared to cameratrapai.

#### Running only the detector

```bash
# cameratrapai.
python3 -m speciesnet.scripts.run_model --instance_json ./instance.json --predictions_json ./predictions.json --detector_only

# speciesnet-rust.
speciesnet-cli --instance-json ./instance.json --predictions-json ./predictions.json --detector-only
```

#### Running only the classifier

```bash
# cameratrapai.
python3 -m speciesnet.scripts.run_model --instance_json ./instance.json --predictions_json ./predictions.json --classifier_only

# speciesnet-rust.
speciesnet-cli --instance-json ./instance.json --predictions-json ./predictions.json --classifier-only
```

#### Running only the ensemble

```bash
# cameratrapai.
python3 -m speciesnet.scripts.run_model --instance_json ./instance.json --classifications_json ./output_classifier.json --detections_json ./output_detector.json --predictions_json ./predictions.json --ensemble_only

# speciesnet-rust.
speciesnet-cli --instance-json ./instance.json --classifications-json ./output_classifier.json --detections-json ./output_detector.json --predictions-json ./predictions.json --ensemble-only
```

#### Running the whole inference pipeline

```bash
# cameratrapai.
python3 -m speciesnet.scripts.run_model --instance_json ./instance.json --predictions_json ./predictions.json

# speciesnet-rust.
speciesnet-cli --instance-json ./instance.json --predictions-json ./predictions.json
```
