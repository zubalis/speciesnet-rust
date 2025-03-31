## Dockerfiles for converting models into various output types.

This directory contains scripts for coverting the model into various output types. Currently we have

- `mega-detector-to-onnx` which is a dockerfile for converting the [MegaDetector V5](https://github.com/agentmorris/MegaDetector/releases/tag/v5.0) model into onnx format.

### Running the onnx converter

To run the onnx converter dockerfile, you can do

```
cd converter/mega-detector-to-onnx

docker build --tag "mega-detector-to-onnx" .
docker run --rm -v "$(PWD)/../../assets/model/:/home/workspace/models/" mega-detector-to-onnx python3 -m yolov5.export --dynamic --include onnx --weights ./models/md_v5a.0.0.pt
```
