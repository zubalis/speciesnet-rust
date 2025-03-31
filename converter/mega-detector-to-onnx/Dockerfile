FROM python:3.9-slim-bookworm

WORKDIR /home/workspace

ARG YOLOV5_SHA=c23a441c9df7ca9b1f275e8c8719c949269160d1

RUN apt-get update -y && apt-get install -y build-essential libprotobuf-dev git libgl1 wget cmake

# ONNX dependencies
RUN pip3 install --no-cache-dir numpy==1.24.0 protobuf==3.20.3 typing-extensions==3.10.0.2
RUN pip3 install --no-cache-dir onnx==1.12.0 onnxruntime==1.13.1 onnxoptimizer==0.3.2 torch==1.10.2 torchvision==0.11.3

# yolo dependencies
RUN pip3 install --no-cache-dir pyyaml==6.0 requests==2.28.1 tqdm==4.64.1 opencv-python-headless==4.6.0.66 pandas==1.5.1 seaborn==0.12.1

# Cloning yolov5 and set to correct commit
RUN git clone https://github.com/ultralytics/yolov5.git /home/workspace/yolov5 && \
  cd /home/workspace/yolov5 && git checkout ${YOLOV5_SHA}

# Download megadetector model
RUN mkdir -p /home/workspace/models
RUN wget https://github.com/agentmorris/MegaDetector/releases/download/v5.0/md_v5a.0.0.pt -O /home/workspace/models/md_v5a.0.0.pt

CMD ["python3", "-m", "yolov5.export"]
