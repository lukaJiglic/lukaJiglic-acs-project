# 007 — Model and PPO learner

Status: **draft** (2026-11-10). Not approved. Needs spec 003 (committed) and an approved encoding contract (spec 004). It is written against draft 1 of `docs/encoding.md`; sizes follow that file.

Build-plan row 007. Tool and model: Claude Code 2.1.306, `claude-fable-5-2`.

## Goal

The training code, complete and tested, before the real environment exists: the transformer in layout L2 (D116), the MLP control, the full-information critic, the PPO update, rollout storage, checkpoints and logging. It runs against a mock environment that emits data of the contract's shapes. Spec 006 then swaps in the Rust engine.

Decisions behind it: D14 (PPO, full-information critic, 20% past checkpoints), D15 (heads), D18 and D30 (transformer and state token), D19 (one network, one masked action vector), D107 (every phase from the start), D111 (inference in Python on the GPU), D116 (layout L2). Sources for the starting values: `docs/planning/research/05` sections C, e and g. Hyperparameter values not found there are Claude's proposals and are marked.

## In scope

Python package `tarok_train` in `src/train`:

- `contract.py`: the sizes and index ranges of `docs/encoding.md` as constants, with the encoding version.
- `model.py`:
  - **Transformer (primary).** Token embedding as the sum of five learned embeddings (event, seat, trick, position in trick, index). The state token is a linear layer from the state features to width 256 plus a learned type embedding. 4 pre-norm layers, width 256, 4 heads, feed-forward 1024. Layout L2: play and bid tokens attend causally to each other only; each state token attends to the tokens before its decision and to itself. Acting uses cached keys and values, one cache per table and seat.
  - **MLP control.** State features → 512 → 512 → 256, same heads.
  - **Heads**, read at the state token: policy (160, masked); Q per action (160, from the observation only, D15); decomposed value (one output per score part: game, difference, trula, kralji, kralj ultimo, pagat ultimo, valat, mondfang); partner seat (4); card location (54 × 4).
  - **Critic (training only).** An MLP over the critic features (state plus hidden information) that outputs the same decomposed value. The actor never receives these features.
- `ppo.py`: generalised advantage estimation with the reward only at the end of the deal; clipped policy loss with masked logits; critic loss; Q-head loss toward the realised return of the action taken; auxiliary losses; an entropy term; advantage normalisation; gradient clipping.
- `rollout.py`: storage by table, seat and deal, so one seat's deal is one sequence for the L2 pass. Deals never mix.
- `opponents.py`: a share of the deals (default 20%, D14) has one or more seats played by a frozen past checkpoint. Those seats' decisions are not trained on.
- `mock_env.py`: N tables stepping in parallel with the contract's arrays. Two modes: random legal data, and a small learnable task (the reward is higher the more often a seat picks its lowest legal slot), to prove that learning happens.
- `train.py`: the loop, a config file per run in `src/train/configs/` (D79), checkpoints with the config and the encoding version, TensorBoard logs.
- A config `configs/mock.toml` and a run on the mock environment.

## Out of scope

The Rust environment and its bindings (spec 006). Evaluation against bots and the tournament (specs 011, 012). The Deep Monte Carlo control run. The Rust forward pass and weight export (after training has started, D111). Explanation methods.

## Acceptance criteria

The user approves these before tests are written (D73).

**Contract**
1. `contract.py` holds the sizes of `docs/encoding.md` (288 events, 1,382 state features, 160 actions, sequence limit 128, critic extra 274). A test parses the tables of `docs/encoding.md` and fails if any differs.

**Model**
2. A forward pass of each network on random contract-shaped data gives outputs of the stated shapes and no NaN.
3. Illegal actions have probability exactly 0. A batch row with a single legal action has probability 1 on it.
4. L2 mask: no state token sees another state token or a later play token; no play token sees a state token or a later play token. Tested by changing a token and checking which outputs move.
5. One L2 pass over a seat's deal gives the same outputs, within 1e-4, as computing each decision alone from its prefix.
6. Cached acting, with tables at different decision points in one batch, gives the same outputs as the full pass, within 1e-4.
7. The actor's outputs do not change when the critic's hidden features change. (The actor has no path from them.)
8. Parameter counts are printed and logged. The transformer has between 3 and 4.5 million parameters, the MLP under 1.5 million.

**PPO**
9. GAE on a hand-computed example of two deals of different length in one buffer matches to 1e-6, and nothing leaks across the end of a deal.
10. The reward of a deal reaches every decision of that seat in that deal and no decision of another seat or deal.
11. The clipped policy loss equals a hand-computed value on a three-step example.
12. Masked logits: the gradient with respect to an illegal action's logit is zero.
13. Every head and the critic receive a non-zero gradient from one update.
14. The card-location loss counts only cards marked unseen (feature F4). The partner loss is skipped where the target is "none".
15. Decisions of seats played by a frozen checkpoint are not in the update batch. With the share set to 0.2 over 1,000 mock deals, the share of deals with a frozen seat is between 0.15 and 0.25.
16. After one update all weights and the loss are finite.

**Learning on the mock task**
17. On the learnable mock task the mean reward per deal rises from the random level by at least half of the distance to the best possible, within a fixed number of updates, with the transformer and with the MLP. The numbers of updates are set when the test is written and reported; the test runs on the CPU in under two minutes.
18. On random-reward mock data the run finishes 20 updates without error and the entropy stays above a floor (no collapse on noise).

**Run mechanics**
19. A checkpoint saved and loaded gives identical outputs on a fixed batch. It contains the config, the encoding version, the step count and the optimiser state. Loading one with another encoding version fails with a clear message.
20. With a fixed seed, two CPU runs of 5 updates give identical losses.
21. TensorBoard files are written with at least: total loss and each part, entropy per phase, explained variance of the critic, partner-prediction accuracy, share of each action group chosen, steps per second.
22. `python -m tarok_train.train --config src/train/configs/mock.toml` runs on the GPU of this PC for 200 updates without error. Steps per second for the transformer and the MLP go into the Results. No threshold; the number is compared with the benchmark of spec 003.

**Quality gates**
23. `pytest` passes locally; the `train` workflow is green (GPU tests skipped there).
24. Every hyperparameter is in the config file, none in the code. The Results list the values used and where each comes from (a research file, or "Claude's proposal").

## Starting hyperparameters (to be confirmed, open point A)

| Name | Value | From |
|------|-------|------|
| Clip range | 0.2 | research/05 G1 (0.1–0.2) |
| PPO epochs | 2 | proposal; research/05 says 4–10, but the update is 87% of the step time (spec 003) |
| Discount, GAE λ | 1.0, 0.95 | proposal; one reward per deal, at most about 30 decisions per seat |
| Learning rate | 3e-4, Adam | proposal (common default) |
| Tables in parallel | 1,024 | proposal; fits the benchmark's fastest batches |
| Update batch | 256 seat-deals | spec 003, fastest L2 batch |
| Entropy weight | 0.01 at the start | proposal; research/05 e7 warns about collapse and about too much |
| Critic loss weight | 0.5 | proposal |
| Q-head loss weight | 0.5 | proposal |
| Auxiliary loss weights | 0.1 each | research/05 C2 (0.1 or lower) |
| Gradient clip | 0.5 | proposal |
| Past-checkpoint share | 0.2 | D14 |
| Precision | fp32 | spec 003 (half precision is slower on this GPU) |

## Files expected to change

New in `src/train/tarok_train/`: `contract.py`, `model.py`, `ppo.py`, `rollout.py`, `opponents.py`, `mock_env.py`, `train.py`; `src/train/configs/mock.toml`; tests in `src/train/tests/`.
Changed: `src/train/requirements.txt` only if a package is added (then pinned), `src/train/README.md`, this spec (status, Results). The benchmark's stand-in model stays as it is; shared code may move into `model.py`.

## Commits

(a) contract constants and model, tests 1–8; (b) PPO, rollout, opponents, tests 9–16; (c) mock environment, training loop, checkpoints, logging, tests 17–22. Each with `Spec: docs/specs/007-model-and-learner.md`.

## Open points for the user

- **A. The starting hyperparameters above.** Accept as a starting point, or change any. PPO epochs is the one that costs most speed.
- **B. The critic.** (a) an MLP over the critic features, recommended: cheap, and it sees the played cards per seat in the state; (b) a second transformer over the tokens plus hidden information: sees the order of play, roughly doubles the update cost.
- **C. The Q head's target.** (a) the realised return of the action taken, as in Deep Monte Carlo (D14's control method), recommended; (b) the critic's value of the next state.
- **D. Score parts for the decomposed value:** game, difference, trula, kralji, kralj ultimo, pagat ultimo, valat, mondfang. Kontra and radelc multipliers apply to the parts later. Add or remove any?
- **E. Dropout:** none proposed. Self-play data is unlimited, so there is little to regularise against.
