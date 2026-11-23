# src/train

The Python training code: environment check, the model, the PPO learner, run configs in `configs/`, evaluation and weight export. It talks to the engine through a PyO3 module.

Built so far: the pinned environment, the GPU check, a PyO3 probe and a throughput benchmark (spec 003); the networks, the PPO learner and the training loop on a mock environment (spec 007); the engine as a Python module and the real environment behind the mock's interface (spec 006). The trainer runs on the mock or on the engine, as the config's `[env]` table says; the monitors, the evaluation against random play, the entropy controller and the stale-module check came with the smoke run (spec 008).

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
| `tarok_train/env.py` | The real environment: the Rust engine behind the mock's interface (`EngineEnv`, `make`), and the environment-only benchmark. |
| `tarok_env/` | Rust crate with the PyO3 bindings of the engine (`src/engine`). Built with maturin into the venv as the module `tarok_env`. |
| `tarok_train/train.py` | The loop on the mock or the engine, config check, checkpoints, TensorBoard logs, `run-info.json` and `summary.json` per run. |
| `tarok_train/evaluate.py` | A checkpoint against random legal play on seeded deals, with a 95% interval; exports game records. |
| `tarok_train/smoke_report.py` | Reads the files of a run folder and prints the numbers of spec 008 as Markdown. |
| `configs/` | One TOML file per run (D79). Every hyperparameter is in the file; the code has no defaults. `mock.toml` (spec 007), `smoke-a-mlp.toml`, `smoke-a-transformer.toml` and the small `smoke-a-ci.toml` for the tests (spec 008). |
| `runs/` | Checkpoints and TensorBoard files of local runs. Not tracked by git. |
| `pyo3_probe/` | Throwaway Rust crate: fills an array on 12 threads with the GIL released, returns NumPy arrays. |
| `bench_results/` | Raw rows of each benchmark run, JSON. Numbers only: no process names, no paths. |
| `tests/` | pytest. Tests marked `gpu` are skipped without CUDA. PyO3 tests are skipped when the probe is not built. `test_spec006.py` is skipped when `tarok_env` is not built (in CI it must be built). |

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

## Build the engine module (cmd)

```bat
.venv\Scripts\activate
cd src\train\tarok_env
maturin develop --release
cd ..\..\..
```

Run maturin from `src\train\tarok_env`, for the same reason as the probe: the folder's `.cargo\config.toml` sets `PYO3_USE_RAW_DYLIB=0`. Build again after every change in `src\engine` or `src\train\tarok_env`. The build takes about 15 seconds.

A module that is out of date is refused (D130). The build writes a fingerprint of the Rust sources into the module (`tarok_env.SOURCE_HASH`, `tarok_env.SOURCE_MANIFEST`: every `.rs` file under `src` of both crates, their `Cargo.toml`, this crate's `Cargo.lock` and `build.rs`). `EngineEnv` compares it with the files on disk when it starts. If they differ it raises `StaleModuleError` and names the files; build again. Where the sources are not on disk nothing is compared.

```python
from tarok_train.env import EngineEnv
env = EngineEnv(n_tables=1024, seed=3, threads=12)
obs = env.observe()                 # dict of NumPy arrays, one row per table
done, reward, parts = env.step(actions)
```

The observation arrays are written in place by the engine; the next `step` overwrites them. The docstring of `tarok_train/env.py` lists every array.

Threads: the engine releases the GIL while it observes, steps or runs random steps, and writes into the arrays during that time. No other thread may read, write or resize the arrays while a call is running. The methods of `EngineEnv` take a lock, so a second thread that calls a method waits. A thread that already holds the arrays from `observe()` is not protected. Use one thread per environment.

The arrays must have exactly the documented shapes and be C-contiguous. `tarok_env.Tables` refuses anything else and names the array and both shapes.

Speed of the environment alone, random policy (venv active, from `src\train`):

```bat
python -m tarok_train.env --tables 1024 --threads 1 12 --seconds 5
```

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

## Train on the engine: the smoke run (spec 008; cmd, venv active, from the repo root)

Build the engine module first (above). Check that nothing else uses the GPU: `nvidia-smi`.

```bat
python -m tarok_train.train --config src\train\configs\smoke-a-mlp.toml
python -m tarok_train.train --config src\train\configs\smoke-a-transformer.toml
python -m tarok_train.train --config src\train\configs\smoke-a-mlp.toml --name control-smoke-a --fault rewards_to_next_seat
```

Each line is one run of 200 updates with 512 tables. The third is the control run: every seat gets the reward of the seat before it. A fault is refused unless the run's name starts with `control-`.

A run writes into its folder under `src\train\runs\`:

| File | Content |
|------|---------|
| `run-info.json` | Written at the start: the commit, the output of `git status --short`, the config as it ran, the GPU load, the fingerprint of the engine's sources. |
| `summary.json` | Written at the end, also when the run fails: updates done, finished or not, what failed, decisions played and trained per second, the time split, peak GPU memory, the numbers of the first and the last update. |
| `checkpoint-NNNNNN.pt` | Every `checkpoint_every` updates and at the end. |
| `events.out.tfevents...` | The TensorBoard scalars of every update. |

A run stops with exit code 3 when it passes `time_limit_minutes` and is reported as not finished. A logged number that is not finite stops the run with an error; `summary.json` then names it and counts only the updates before it.

More training options for these runs: `--name`, `--fault` and `--time-limit-minutes`, each overriding the value of `[run]`.

Evaluate a checkpoint against random legal play, then the same network before its first update:

```bat
python -m tarok_train.evaluate --checkpoint src\train\runs\<run folder>\checkpoint-000200.pt --deals 2000 --seed 2008 --records 100
python -m tarok_train.evaluate --checkpoint src\train\runs\<run folder>\checkpoint-000200.pt --deals 2000 --seed 2008 --untrained
```

Every deal is played four times, with the agent in each seat and random legal play in the other three. The value of a deal is the mean of the agent's four rewards. The output is the mean over the deals and its 95% interval; "beats random" means the whole interval is above zero. The result goes to `evaluation-NNNNNN.json` (or `evaluation-untrained.json`) in the run folder. `--records 100` writes 100 of the deals as game records into `records-NNNNNN\` and replays each one. The deals are played in chunks of `--tables` tables; chunk c uses the engine seed `--seed` + c. No chunk may use the seed of the training run, or its deals would be deals the agent trained on: such a call is refused.

The numbers of a run for the spec's Results, as Markdown:

```bat
python -m tarok_train.smoke_report src\train\runs\<run folder>
```

Monitors on real deals (TensorBoard tags, one value per update):

| Tag | Meaning |
|-----|---------|
| `entropy/<phase>`, `entropy_weight/<phase>` | Policy entropy of the decisions of a phase, and the entropy weight the controller gives that phase for the next update. |
| `entropy_choice/<phase>`, `forced_share/<phase>` | The same entropy over the decisions with two or more legal slots only, and the share of decisions with one legal slot. A forced decision has entropy 0 whatever the policy is, so `entropy/<phase>` = `entropy_choice/<phase>` x (1 - `forced_share/<phase>`). The controller steers on `entropy/<phase>`. |
| `entropy/phases_outside_band` | Phases whose entropy is further from its target than the band allows. |
| `deals/contract_<name>` | Share of the deals played in each contract. |
| `deals/forehand_forced`, `deals/forehand_forced_tri` | Share of the deals in which the three others passed before forehand spoke, and in which forehand then played tri. |
| `deals/declarer_wins` | Share of the deals the declarer's side won. |
| `rollout/decisions_per_seat_deal_mean`, `_max`; `rollout/tokens_per_seat_deal_mean`, `_max` | Length of a seat's deal in decisions and in tokens. |
| `rollout/played`, `rollout/played_own_seats`, `rollout/decisions` | Decisions of the update: played by all seats (also those of a past checkpoint), played by the seats of the current network, and trained on. |
| `rollout/trained_share` | Trained / played: the share of played decisions that were trained on. Played counts every seat. This is the number of spec 008, criterion 9, and the `trained_share` of `summary.json`. |
| `rollout/trained_share_own_seats` | Trained / the decisions of the current network's seats. Higher than the share above once a past checkpoint plays some seats. |
| `speed/steps_per_second`, `speed/trained_per_second` | Decisions played (all seats) and trained per second. |
| `rollout/decision_age_max` | Must be 0. A stored decision carries the number of updates done when its deal began; the age is the number of updates since. A deal that began under older weights would show as 1 or more. |
| `time/environment_share`, `time/acting_share`, `time/bookkeeping_share`, `time/update_share` | Where the wall time of the update went. |
| `aux/partner_accuracy`, `aux/partner_guess`; `aux/card_accuracy`, `aux/card_guess` | Accuracy of the two auxiliary heads, each next to what a uniform guess over the places still possible would get. |
| `reward/deal_sum_abs_max`, `env/illegal_actions` | Both must be 0: the four rewards of a deal add up to zero, and the engine refused no action. |
| `gpu/peak_memory_mib`, `gpu/peak_memory_reserved_mib` | GPU runs only, peak so far. The first is the largest sum of the run's tensors (`torch.cuda.max_memory_allocated`). The second is what PyTorch's allocator held from the card for them (`max_memory_reserved`); it is the larger one and the one that must fit. `summary.json` has both, and under `gpu_before` and `gpu_after` what `nvidia-smi` shows for the whole card, every program on it. |

The entropy controller (D127) is set in `[entropy_control]` of the config. It keeps one entropy weight per phase. After every update, for each phase that had decisions:

```
error    = (target - entropy) / target, cut to [-1, 1]
integral <- integral * exp(gain * error),         kept inside [min_weight, max_weight]
weight    = integral * exp(proportional * error), kept inside [min_weight, max_weight]
```

It changes only these weights. Every weight starts at `[ppo] entropy_weight`. With `enabled = false` the weights never move and the update uses `[ppo] entropy_weight`. `band` is used by the monitor `entropy/phases_outside_band` and by the test of the controller: a phase is in its band when its entropy is within that share of its target.
