## SpeciesNet Rust

This is the home of the `speciesnet` crate for running [google/cameratrapai](https://github.com/google/cameratrapai) pipeline in Rust language.

### Required tools

- [uv](https://github.com/astral-sh/uv) for running the model conversion code.
- [rustup](https://rustup.rs) for running rust program.
- [tensorflow](https://tensorflow.org) for running the cameratrap model.


### Setting up the environment

#### Downloading the cameratrapai models

Head over to [https://www.kaggle.com/models/google/speciesnet](https://www.kaggle.com/models/google/speciesnet), download the `v4.0.0a` model and extract it to the `assets/model` folder. Then, run the megadetector model conversion script in python by

```
cd python
uv run mega-detector-trace-model.py
```

You will get the traced model inside the `assets/model` folder. Now you're ready to load that model into Rust and start using it with Rust.

#### Setting up the environment variables and C++ modules

To run the project. Please head over to [https://pytorch.org](https://pytorch.org) to download the LibTorch C++ module from the front page of PyTorch website. Make sure that you are grabbing the LibTorch v2.6.0 version, because other versions will just *not work*.

Then you can start extracting the model out somewhere (preferrably inside `assets` folder because it is git-ignored automatically) and set a couple environment variables to be able to run the program.

- `LIBTORCH` which is the absolute path to where your extracted your downloaded libtorch.
- `DYLD_LIBRARY_PATH` if you're on MacOS, `LD_LIBRARY_PATH` if you're on Linux, which is the value of `LIBTORCH` plus `/lib`. If your `LIBTORCH` is `/opt/homebrew/libtorch`, then your `DYLD_LIBRARY_PATH` is `/opt/homebrew/libtorch/lib`.
- `RUST_LOG` which is to make the program show logs while running. As a starter you can set this to `debug` to show all logs. Please refer to [docs.rs/env_logger](https://docs.rs/env_logger) for more options and per-module log levels.

  Please make sure that you're not setting `RUST_LOG` permanently as it could interfere with other Rust projects in the future.

### Running the examples

There is an example program inside `examples/detect` folder that will run the detector on the example image. Go ahead and run it by running

```
cd examples/detect
cargo run
```

One image will be saved out of it with the bounding boxes and confidence drawn on it.
