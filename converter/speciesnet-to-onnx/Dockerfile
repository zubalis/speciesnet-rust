FROM python:3.9-slim-bookworm

WORKDIR /home/workspace

COPY convert_onnx.py convert_onnx.py

# Install dependencies
RUN pip3 install --no-cache-dir tf2onnx==1.16.1 tensorflow==2.15.0 kagglehub==0.3.11

CMD ["python3", "convert_onnx.py"]
