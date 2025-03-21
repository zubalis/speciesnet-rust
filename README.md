## SpeciesNet Rust

This is the home of the `speciesnet` crate for running [google/cameratrapai](https://github.com/google/cameratrapai) pipeline in Rust language.

### Required tools

- [uv](https://github.com/astral-sh/uv) for running the model conversion code.
- [rustup](https://rustup.rs) for running rust program.
- [tensorflow](https://tensorflow.org) for running the cameratrap model.


### Setting up the environment

#### Downloading the cameratrapai models

Head over to [https://www.kaggle.com/models/google/speciesnet](https://www.kaggle.com/models/google/speciesnet), download the `v4.0.0a` model and extract the files to the `assets/model` folder (you should have `always_crop_[...].keras` and other files inside the `assets/model` folder).

Go to [https://github.com/agentmorris/MegaDetector/releases/tag/v5.0](https://github.com/agentmorris/MegaDetector/releases/tag/v5.0), download the `md_v5b.0.0.pt` weights for MDv5 and place it in the `assets/model` folder.

Then, run the megadetector model conversion script in python by

```
cd python
uv run mega-detector-trace-model.py
```

You will get the traced model inside the `assets/model` folder. Now you're ready to load that model into Rust and start using it with Rust.

#### Setting up the LibTorch C++ module

Download the LibTorch C++ module from [https://pytorch.org](https://pytorch.org), on the front page choose "Stable" -> (Your OS) -> "LibTorch" (Package) -> "C++/Java" (Language). Make sure that you are get v2.6.0, because previous versions will *not work*.

Extract LibTorch (preferrably) inside the `assets` folder. Set a couple environment variables to be able to find LibTorch, e.g.

```
cd assets
export LIBTORCH=${PWD}/libtorch
export LD_LIBRARY_PATH=${LIBTORCH}/lib:$LD_LIBRARY_PATH
export DYLD_LIBRARY_PATH=${LIBTORCH}/lib:$DYLD_LIBRARY_PATH
```

Notes
- `LIBTORCH` is the absolute path to where your extracted your downloaded LibTorch
- `DYLD_LIBRARY_PATH` if you're on MacOS, `LD_LIBRARY_PATH` if you're on Linux, which is the value of `LIBTORCH` plus `/lib`. If your `LIBTORCH` is `/opt/homebrew/libtorch`, then your `DYLD_LIBRARY_PATH` is `/opt/homebrew/libtorch/lib`.

Important on macOS: remove the quarantine flag from the LibTorch library with `xattr -r -d com.apple.quarantine $LIBTORCH/lib`. (Otherwise you'll get messages like "Library not loaded: @rpath/libtorch_cpu.dylib".)

#### Other environment variables

- Use `RUST_LOG` to control how logs are shown while running. As a starter you can set this to `debug` to show all logs. Please refer to [docs.rs/env_logger](https://docs.rs/env_logger) for more options and per-module log levels. (Make sure that you're not setting `RUST_LOG` permanently as it could interfere with other Rust projects in the future.)

```
export RUST_LOG=debug
```


### Running the examples

There is an example program inside `examples/detect` folder that will run the detector on the example image. Go ahead and run it by running

```
cd examples/detect
cargo run
```

One image will be saved out of it with the bounding boxes and confidence drawn on it.
