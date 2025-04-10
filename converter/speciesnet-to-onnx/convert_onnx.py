import tensorflow as tf
import tf2onnx
import kagglehub
import os

# Download latest version
path = kagglehub.model_download("google/speciesnet/keras/v4.0.0a")

# Load the .keras model
print("Loading keras")
model = tf.keras.models.load_model(path + "/always_crop_99710272_22x8_v12_epoch_00148.keras")
print("Loaded keras")

# Define input signature
spec = (tf.TensorSpec((None, *model.input.shape[1:]), tf.float32, name="input"),)

# Convert to ONNX
print("Converting onnx")
try:
    onnx_model, _ = tf2onnx.convert.from_keras(model, input_signature=spec, opset=13)
except Exception as e:
    print(e)
print("Converted onnx")

# Save the ONNX model
if not os.path.exists("models"):
  os.mkdir("models")

print("Writing onnx file")
with open("models/model.onnx", "wb") as f:
    f.write(onnx_model.SerializeToString())

print("Successful converting keras to onnx file")