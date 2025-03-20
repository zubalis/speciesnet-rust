import yolov5
import torch

assert yolov5

def main():
    print(f"MegaDetector V5 tracer to TorchScript CPU model.")

    checkpoint = torch.load('./md_v5a.0.0.pt', weights_only=False, map_location="cpu")
    model = checkpoint["model"].float()
    model.eval()

    for m in model.modules():
        if isinstance(m, torch.nn.Upsample) and not hasattr(
            m, "recompute_scale_factor"
        ):
            m.recompute_scale_factor = None

    # Example input in Channel, Height, Width format.
    example_input = torch.rand(1, 3, 960, 1280).to(device="cpu", dtype=torch.float32)

    # Test run the modle once.
    __results = model(example_input)

    # Start jit tracing the model to convert to TorchScript
    traced_model = torch.jit.trace(model, example_input)
    traced_model.save("./md_v5a.0.0_traced.pt")

if __name__ == '__main__':
    main()
