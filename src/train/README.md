# src/train

The Python training code: environment check, the model, the PPO learner, run configs in `configs/`, evaluation and weight export. It talks to the engine through a PyO3 module.

Built so far: the pinned environment, the GPU check, a PyO3 probe and a throughput benchmark (spec 003); the networks, the PPO learner and the training loop on a mock environment (spec 007). The Rust engine replaces the mock with spec 006.

## Files

| Path | What it is |
|------|------------|
| `requirements.txt` | Every package pinned with `==`. PyTorch 2.14.1 for CUDA 13.0 from `https://download.pytorch.org/whl/cu130`. |
| `requirements-ci.txt` | CPU-only set for the GitHub runner (`.github/workflows/train.yml`). |
| `tarok_train/envcheck.py` | Prints versions, the GPU, its capability and the free VRAM. Exit code 1 if CUDA is not usable. |
| `tarok_train/bench_model.py` | Times the stand-in transformer (layouts L1 and L2) and the MLP control on random data. |
| `tarok_train/contract.py` | The sizes and index ranges of `docs/encoding.md` (encoding version 1). A test compares them with the document. |
| `tarok_train/model.py` | The transformer in layout L2 with cached acting, the MLP control, the heads, the full-information critic. |
| `tarok_train/ppo.py` | GAE, the clipped policy loss, the critic, Q and auxiliary losses, the update. |
| `tarok_train/rollout.py` | Storage by table, seat and deal. One seat's deal is one sequence. |
| `tarok_train/opponents.py` | Past checkpoints as opponents: which seats they play, the pool of snapshots. |
| `tarok_train/mock_env.py` | Mock environment with the contract's arrays: a learnable task and a random one. |
| `tarok_train/train.py` | The loop, config check, checkpoints, TensorBoard logs. |
| `configs/` | One TOML file per run (D79). Every hyperparameter is in the file; the code has no defaults. |
| `runs/` | Checkpoints and TensorBoard files of local runs. Not tracked by git. |
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

## Train on the mock environment (cmd, venv active, from the repo root)

```bat
python -m tarok_train.train --config src\train\configs\mock.toml --mode tokens
python -m tarok_train.train --config src\train\configs\mock.toml --network mlp
tensorboard --logdir src\train\runs
```

The first line trains the transformer for 200 updates on the mock task that needs the tokens, the second the MLP control on the task that needs the state only. Each run writes a folder under `src\train\runs\` with its checkpoints and TensorBoard files.

To go on from a checkpoint, up to 300 updates in all:

```bat
python -m tarok_train.train --resume src\train\runs\<run folder>\checkpoint-000200.pt --updates 300
```

Training options (each overrides a value of the config file; the checkpoint records what ran):

| Option | Meaning |
|--------|---------|
| `--network mlp` | `transformer` or `mlp`. |
| `--updates 50` | Number of updates, in all. |
| `--tables 256` | Tables stepped in parallel. |
| `--seat-deals 512` | Finished seat-deals an update waits for. |
| `--mode tokens` | Mock mode: `learnable` (a state feature names the rewarded slot), `tokens` (a cue token does) or `random`. |
| `--device cpu` | `cuda` or `cpu`. |
| `--out-dir other\folder` | Another folder for checkpoints and logs. |
| `--resume checkpoint.pt` | Go on from a checkpoint, with its config. On the CPU the result equals an unbroken run. |

How an update collects its data: the tables play until enough seat-deals have finished that began under the current weights. Deals that straddle an update are played but not trained on. `rollout/decision_age_max` in the logs must be 0 and `ppo/behaviour_mismatch` about 1e-6; anything else is a bug in the pipeline.

A checkpoint holds the config, the encoding version, the step counts, the weights, the optimiser, the pool of past checkpoints, the random generators and the environment's state. Loading a checkpoint of another encoding version fails with a message.
