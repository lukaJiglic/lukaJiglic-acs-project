# src/train

The Python training code: environment check, the model, the PPO learner, run configs in `configs/`, evaluation and weight export. It talks to the engine through a PyO3 module.

Built so far (spec 003): the pinned environment, the GPU check, a PyO3 probe and a throughput benchmark. The model and the learner come with spec 007.

## Files

| Path | What it is |
|------|------------|
| `requirements.txt` | Every package pinned with `==`. PyTorch 2.14.1 for CUDA 13.0 from `https://download.pytorch.org/whl/cu130`. |
| `requirements-ci.txt` | CPU-only set for the GitHub runner (`.github/workflows/train.yml`). |
| `tarok_train/envcheck.py` | Prints versions, the GPU, its capability and the free VRAM. Exit code 1 if CUDA is not usable. |
| `tarok_train/bench_model.py` | Times the stand-in transformer (layouts L1 and L2) and the MLP control on random data. |
| `pyo3_probe/` | Throwaway Rust crate: fills an array on 12 threads with the GIL released, returns NumPy arrays. |
| `bench_results/` | Raw rows of each benchmark run, JSON. Numbers only: no process names, no paths. |
| `tests/` | pytest. Tests marked `gpu` are skipped without CUDA. PyO3 tests are skipped when the probe is not built. |

## Set up (cmd, from the repo root)

```bat
py -3.12 -m venv .venv
.venv\Scripts\python -m pip install -r src\train\requirements.txt
.venv\Scripts\python -m pip install --no-build-isolation --no-deps -e src\train
```

The first command downloads about 2 GB. Nothing is installed outside `.venv`.

## Build the PyO3 probe (cmd)

```bat
.venv\Scripts\activate
cd src\train\pyo3_probe
maturin develop --release
cd ..\..\..
```

Run maturin from `src\train\pyo3_probe`: the folder's `.cargo\config.toml` sets `PYO3_USE_RAW_DYLIB=0`, which the GNU Rust toolchain needs here.

## Run (cmd, venv active)

```bat
python -m tarok_train.envcheck
python -m tarok_train.bench_model --label "gpu free"
cd src\train
python -m pytest
```

The benchmark writes its raw rows to `src\train\bench_results\`. It times every row in 3 repeats and prints the median with the minimum and maximum. A full run takes about 7 minutes.

Benchmark options:

| Option | Meaning |
|--------|---------|
| `--label "..."` | What else used the GPU. Goes into the result file name. |
| `--seconds 1 --repeats 3` | Least time per repeat and the number of repeats. |
| `--precisions fp32` | Only fp32 (default: `fp32,fp16`). |
| `--state-features 1382 --actions 160 --vocab 288` | Sizes of the encoding contract (`docs/encoding.md`). |
| `--play-tokens 128 --decisions 20` | Tokens per deal and decisions per seat and deal. |
| `--quick` | Tiny batches, a smoke test. |

The PyO3 setting `PYO3_USE_RAW_DYLIB=0` is read by cargo from the folder it runs in. A new binding crate needs its own `.cargo\config.toml`, or one at the repo root.
