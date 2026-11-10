"""Environment check for the training PC (spec 003, criterion 2).

Run:  python -m tarok_train.envcheck
Prints versions, the CUDA device, its compute capability and the free VRAM.
Exit code 0 when CUDA is usable, 1 when it is not.
"""

import platform
import sys

import numpy
import torch

MIB = 1024 * 1024


def main() -> int:
    print(f"python: {platform.python_version()} ({sys.executable})")
    print(f"platform: {platform.platform()}")
    print(f"torch: {torch.__version__}")
    print(f"torch cuda build: {torch.version.cuda}")
    print(f"numpy: {numpy.__version__}")

    available = torch.cuda.is_available()
    print(f"cuda available: {available}")
    if not available:
        print("FAIL: CUDA is not usable. Check the driver and that torch is the +cu130 build.")
        return 1

    index = torch.cuda.current_device()
    print(f"device: {torch.cuda.get_device_name(index)}")
    print(f"capability: {torch.cuda.get_device_capability(index)}")
    print(f"cudnn: {torch.backends.cudnn.version()}")
    free, total = torch.cuda.mem_get_info(index)
    print(f"vram total: {total // MIB} MiB")
    print(f"vram free: {free // MIB} MiB")

    # "Available" is not enough: run one real kernel on the device.
    x = torch.arange(6, dtype=torch.float32).reshape(2, 3)
    want = (x @ x.T).sum().item()  # 83.0, computed on the CPU
    try:
        on_gpu = x.cuda()
        got = (on_gpu @ on_gpu.T).sum().item()
        torch.cuda.synchronize()
    except RuntimeError as err:
        print(f"FAIL: a tensor operation on the GPU raised: {err}")
        return 1
    if got != want:
        print(f"FAIL: GPU matmul gave {got}, the CPU gave {want}")
        return 1
    print("gpu matmul: ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
