# 007 — Model and PPO learner

Status: **approved** by the user on 2026-11-10 (D119), **executed 2026-11-10, review fixes worked in on 2026-11-11, committed in `2cd15e3` (D122), CI green on `819220d`. Done 2026-11-11.** Built on `docs/encoding.md` version 1 (approved, D117) and spec 003.

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

## Answers to the open points (D119)

- A. The starting hyperparameters are accepted as the starting point, with 2 PPO epochs.
- B. The critic is an MLP over the critic features.
- C. The Q head is trained toward the realised return of the action taken.
- D. Score parts as listed, plus "klop" and "radelc": ten parts (D121, 2026-11-11). The order is in `docs/encoding.md` section 8.
- E. No dropout.

## Results

Executed on 2026-11-10 by a parallel session (D109). Rewritten on 2026-11-11 after the review pass (D72 step 4): the review found gaps in the learning test and several risks for the real game; all thirteen review items are worked in. Committed on 2026-11-11 (D122).

Criteria 1 to 22 and 24 are met locally. Criterion 23 is half met: `pytest` passes locally; the `train` workflow has not run on this code, because nothing is pushed.

Paths in this section are relative to the repository root. GPU numbers say whether the GPU was free.

### Criteria and their tests

All tests are in `src/train/tests/test_spec007.py`. A test's name starts with its criterion number. `[t, m]`: the test runs once for the transformer and once for the MLP.

| # | Result | Tests |
|---|--------|-------|
| 1 | met | `test_c01_contract_sizes_match_the_tables_of_encoding_md`; `test_c01_the_number_of_score_parts_is_one_constant` (runs an update with 12 parts); `test_c01_the_mock_emits_ten_parts_that_add_up_to_the_reward`. The first also reads the "Score parts" table of section 8: ten parts, in the contract's order (D121) |
| 2 | met | `test_c02_forward_pass_gives_the_stated_shapes_and_no_nan` [t, m] |
| 3 | met | `test_c03_illegal_actions_have_probability_zero_and_a_single_legal_action_one` [t, m] |
| 4 | met | `test_c04_l2_mask_which_outputs_move_when_a_token_changes` |
| 5 | met | `test_c05_one_l2_pass_equals_each_decision_alone_from_its_prefix` |
| 6 | met | `test_c06_cached_acting_equals_the_full_pass_also_across_a_change_of_weights`; `test_c06_acting_and_update_see_byte_identical_actor_inputs` [t, m] |
| 7 | met | `test_c07_the_actor_does_not_depend_on_the_critics_hidden_features` [t, m]: through the training path; outputs and actor gradients are bit-equal |
| 8 | met | `test_c08_parameter_counts_are_in_range_printed_and_logged` |
| 9 | met | `test_c09_gae_matches_a_hand_computed_example_of_two_deals`: the function, and the same deals through the storage and `prepare` |
| 10 | met | `test_c10_the_reward_reaches_every_decision_of_that_seat_and_deal_only`; `test_c10_score_parts_that_do_not_add_up_to_the_reward_are_refused` |
| 11 | met | `test_c11_clipped_policy_loss_equals_a_hand_computed_value` (0.85) |
| 12 | met | `test_c12_no_gradient_reaches_an_illegal_actions_logit_in_the_update` [t, m]: on the update's own path |
| 13 | met | `test_c13_every_head_and_the_critic_get_a_gradient_from_one_update` [t, m]; `test_c13_the_critic_reads_the_environments_critic_array_of_the_same_decision`; `test_c13_the_value_losses_are_huber_or_squared_as_the_config_says` |
| 14 | met | `test_c14_card_loss_counts_unseen_cards_only_and_partner_loss_skips_none`; `test_c14_in_the_update_the_unseen_cards_come_from_state_block_f4` (a batch built by hand) |
| 15 | met | `test_c15_frozen_seats_are_played_by_the_past_checkpoint_and_are_not_trained` [t, m]; `test_c15_the_share_of_deals_with_a_frozen_seat` |
| 16 | met | `test_c16_after_one_update_all_weights_and_the_loss_are_finite` [t, m]; `test_c16_a_fresh_rollout_has_probability_ratio_one_on_every_decision` [t, m]; `test_c16_a_small_remainder_joins_the_previous_minibatch` |
| 17 | met | `test_c17_the_mean_reward_rises_by_half_the_distance_to_the_best` [t, m]; two control tests (below) |
| 18 | met | `test_c18_random_rewards_run_20_updates_and_the_entropy_does_not_collapse` [t, m] |
| 19 | met | `test_c19_checkpoint_round_trip_and_encoding_version_check` [t, m]; `test_c19_a_resumed_run_equals_an_unbroken_run` [t, m] |
| 20 | met | `test_c20_two_cpu_runs_with_one_seed_give_identical_losses` [t, m] |
| 21 | met | `test_c21_tensorboard_files_hold_the_required_scalars` |
| 22 | met | The two 200-update runs below. `test_c22_the_train_command_runs_on_the_gpu_and_resumes` [t, m] (3 updates, then `--resume`); `test_c22_the_train_command_runs_on_the_cpu_and_resumes` |
| 23 | met locally; CI pending | 75 tests: 70 pass with the GPU hidden and the 5 GPU tests pass on their own (output below). The workflow has not run on this code. |
| 24 | met | `test_c24_every_hyperparameter_is_in_the_config_file`; the list is below. |

### Parameter counts

Printed at the start of every run and written to TensorBoard (`parameters/...`).

| Network | Parameters | Of which |
|---------|-----------|----------|
| Transformer | 3,767,590 | 4 layers 3,159,040; state projection 354,048; token embeddings 112,384; heads 141,350; the rest 768 |
| MLP control | 1,243,430 | trunk 1,102,080; heads 141,350 |
| Critic | 1,244,938 | an MLP 1,656 → 512 → 512 → 256 → 10 parts; training only |

The counts are for the ten score parts of D121 (set on 2026-11-11; with eight parts each network had 514 parameters fewer). The number of parts is the constant `N_SCORE_PARTS` in `contract.py`; a test runs both networks with 12. The 200-update runs below were made with eight parts and were not repeated: two more outputs on the value heads do not change their speed or what they show.

### Learning on the mock task (criterion 17)

**The task was changed after the review.** In the first version the rewarded slot was the lowest legal one. A fixed preference solved it, without reading the observation, so many planted bugs still passed. Now the legal slots of a decision are the first four of its phase's action group, one of them is rewarded, and which one can only be read from the observation:

- **Mode `learnable` (the MLP's test):** a hint in the state features names the rewarded slot.
- **Mode `tokens` (the transformer's test):** at three decisions out of four a flag in the state says "read the cue", and the deal's cue token (the second token of the sequence) names the rewarded slot. At the other decisions the state hint counts.

The reward of a seat is the share of its decisions in the deal that took the rewarded slot. Random play gives 0.25, the best play 1.0. Half of the distance is 0.625.

The test uses the hyperparameters of `configs/mock.toml`. Only the size is smaller, for the CPU. Seed 17. Sizes and numbers of updates were fixed when the test was written.

| Network | Mode | Tables | Seat-deals per update | Update batch | Updates | Reward per deal: first update → mean of the last three | Time on the CPU |
|---------|------|--------|-----------------------|--------------|---------|--------------------------------------------------------|-----------------|
| Transformer | tokens | 32 | 128 | 32 seat-deals | 30 | 0.276 → 0.800 | 49 s |
| MLP | learnable | 64 | 256 | 32 seat-deals | 30 | 0.245 → 0.935 | 12 s |

Mean reward per deal after each update (measured, `pytest -s`):

```
transformer  0.28 0.23 0.26 0.28 0.25 0.24 0.25 0.29 0.32 0.36 0.48 0.51 0.57 0.60 0.66 0.66 0.73 0.73 0.73 0.72 0.76 0.71 0.79 0.79 0.78 0.81 0.79 0.78 0.82 0.80
mlp          0.25 0.26 0.24 0.26 0.25 0.25 0.30 0.29 0.34 0.40 0.42 0.44 0.49 0.53 0.54 0.61 0.61 0.72 0.76 0.79 0.83 0.84 0.88 0.88 0.90 0.90 0.92 0.94 0.93 0.94
```

The test also asserts, at every update: the update saw what the actor saw (`ppo/behaviour_mismatch` under 1e-4; in the GPU runs it was at most 4e-6); no decision was older than the current weights (`rollout/decision_age_max` is 0); `ppo/approx_kl` under 0.2 and the clip fraction under 0.8 (measured at most 0.10 and 0.56). The last two limits are loose on purpose: a mask left out of the update gives 5 and 1.0.

Two control tests:

- **Reward to the wrong seat.** Every seat gets its neighbour's reward. The MLP then stays at the random level.
- **The token task without tokens.** The MLP reads no tokens. On mode `tokens` it stays under 0.52, the level the state alone allows (0.44) plus a margin. So the transformer's 0.80 on that task shows that the tokens reach the policy, through the decision points in the update and through the cache when acting.

**Finding 1: the first scale of the policy head.** My first config set it to 0.01. On the new task neither network learned in 30 updates (0.25). Measured on the MLP, first update: gradient norm on the trunk from the policy loss 0.000, from the value head 0.028, from the Q head 0.025, from the partner head 0.019. With the PyTorch default scale (1.0) the policy loss gives 0.048 and the MLP learns. The config now has 1.0. With 0.01 and the same number of optimiser steps the MLP reaches 0.60 in 40 updates, with 1.0 it reaches 0.98.

The scale is a trade-off, measured on the test sizes (score after 30 updates; entropy of the transformer on random rewards over 20 updates, where 1.39 is the uniform policy):

| First scale of the policy head | MLP, mode `learnable` | Transformer, mode `tokens` | Transformer's entropy on random rewards |
|---|---|---|---|
| 0.01 | 0.25 (batch 128) | 0.30 (batch 64) | not measured |
| 0.1 | 0.57 | 0.63 | 1.38 → 1.30, flat |
| 0.3 | 0.79 | 0.70 | 1.37 → 1.04, sliding down |
| 1.0 (in the config) | 0.94 | 0.80 | 1.09 → 0.97, between 0.78 and 1.09, no slide |

With 1.0 the first policy is not uniform (entropy 1.09 to 1.29 instead of 1.39), and at the transformer's first update on random rewards the policy moved by an `approx_kl` of 0.18 although the rewards carried nothing: a side effect of the value head's large first gradient on the shared trunk (see the gradient table below). The test for criterion 18 was written for a uniform first policy (floor 0.7 × 1.39); it now asks that the entropy never falls under half of the uniform policy's and that the mean of the last five updates is over 0.8 of the mean of the first five. Question 5.

**Finding 2: the transformer stops at 0.81 on the token task.** In the 200-update run its score is 0.81 from update 20 to 200, with a policy entropy of 0.001 to 0.03. Measured on the final checkpoint, 61,440 decisions: 1.000 right where the cue token names the slot (46,147 decisions), 0.248 right where the state hint names it (15,293 decisions). It always follows the token. 0.75 × 1 + 0.25 × 0.25 = 0.8125. What is checked: the transformer does learn the hint when the hint is the only signal (mode `learnable`: 0.78 after 40 updates and rising); an entropy weight of 0.05 instead of 0.01 did not lift the plateau in 60 updates. Why it stays there is not established; an early collapse of the policy's entropy is the likely cause (research/05 e7), and that is a guess. The criterion is met either way; the plateau is a behaviour of the learner worth knowing before the real run.

### The 200-update runs on the GPU (criterion 22)

Commands, from `src\train`:

```bat
python -m tarok_train.train --config configs\mock.toml --network mlp
python -m tarok_train.train --config configs\mock.toml --mode tokens
```

Both ran 200 updates without error on 2026-11-11. **Label: GPU free.** nvidia-smi before each run: 0% busy, 234 of 6,144 MiB used. "Steps" are decisions played: one per table and environment step, trained on or not.

| | MLP, mode `learnable` | Transformer, mode `tokens` |
|---|-----|-------------|
| Tables | 1,024 | 1,024 |
| Decisions played | 5,238,784 | 5,210,112 |
| Share of them trained on (median) | 39% | 39% |
| Decisions per update (trained on) | 7,300 to 12,400 | 7,300 to 12,300 |
| Wall time | 271 s | 931 s |
| Steps per second, median over the updates | 19,604 | 5,621 |
| Steps per second, whole run | 19,326 | 5,594 |
| Time per update: acting / update (median) | 40% / 60% | 37% / 63% |
| Peak GPU memory held by PyTorch | 133 MiB | 3,621 MiB |
| Task score, first → last update | 0.247 → 1.000 | 0.248 → 0.814 |
| Largest `ppo/behaviour_mismatch` | 3.1e-6 | 3.8e-6 |
| Oldest decision in an update | age 0 | age 0 |
| Largest `ppo/approx_kl`, largest clip fraction | 0.145, 0.33 | 0.104, 0.51 |

Task score at updates 1, 10, 20, 50, 100, 200: MLP 0.247, 0.656, 0.985, 0.994, 0.995, 1.000; transformer 0.248, 0.670, 0.819, 0.811, 0.808, 0.814.

Against the GPU-free benchmark of spec 003 (2026-11-11, derived for 2 epochs, the engine not counted):

- **Transformer:** the benchmark gives 7,732 steps/s for L2 with ragged cached acting, if every decision played is trained on. The loop plays 5,621 decisions per second and trains on 39% of them: about 2,200 trained decisions per second. Where the difference comes from: 61% of the decisions are played for nothing (see "Whole deals" below); the loop also runs the mock environment, the storage, the critic and the auxiliary heads; the state is 1,382 wide; the mock's seat-deals are short, so an update batch of 256 seat-deals holds about 1,150 decisions.
- **MLP:** the benchmark's derived number is 265,177 steps/s. The loop plays 19,604. The network is not the limit here: the mock environment alone makes 38,000 decisions per second on one core (measured on 2026-11-10).

Earlier runs, on 2026-11-10 with a game on the GPU and the first version of the collection (stale decisions were trained on): MLP 7,389 and transformer 1,967 steps/s (median). They are not comparable with the table above: the GPU load, the collection and the task all changed.

### Whole deals (review item 2): what was chosen and what it costs

Chosen: **an update trains only on deals played from their first decision under the current weights.** After an update every table that is inside a deal is "stale": its deal is played to the end but not stored. A table becomes "fresh" when its next deal starts. The collection runs until `seat_deals_per_update` stored seat-deals have finished. Rows of deals still running at that moment are dropped. The environment is not reset, so the tables' runs of 24 deals with their radelci (D117 C) stay whole.

Measured: `rollout/decision_age_max` is 0 at every update of every run. Before the change, 60 to 70% of an update's decisions were one to three updates old (the review's measurement).

The cost: each table loses about one deal per update, the one that straddles it. In the GPU runs 39% of the decisions played were trained on (34 to 48% over the updates), with `seat_deals_per_update = 2048` and 1,024 tables. A longer collection wastes less: if a table plays T steps per update and a deal has L steps, about (T − L) / T is trained on (arithmetic). The value for the real game is a decision for spec 008.

### What the logs show about the loss terms (review items 5 and 11)

The value, critic and Q losses are Huber losses now (`value_loss = "huber"`, `huber_delta = 1.0` in the config; `"mse"` is the other choice). Once per update, on the first minibatch and before any step, the gradient norm of each loss term on the trunk is logged (`grad/trunk_...`), each term as it enters the total.

Measured in the GPU runs (GPU free), gradient norm on the trunk:

| Update | Network | policy | value head | Q head | partner | cards | entropy | whole actor, before the clip of 0.5 |
|--------|---------|--------|------------|--------|---------|-------|---------|-----|
| 1 | Transformer | 1.33 | 19.9 | 2.21 | 1.25 | 0.089 | 0.026 | 22.2 |
| 5 | Transformer | 0.28 | 0.13 | 0.069 | 0.069 | 0.009 | 0.007 | 0.70 |
| 200 | Transformer | 0.0003 | 0.010 | 0.017 | 0.021 | 0.004 | 0.0001 | 0.12 |
| 1 | MLP | 0.037 | 0.049 | 0.019 | 0.019 | 0.001 | 0.00003 | 0.12 |
| 10 | MLP | 0.58 | 0.13 | 0.072 | 0.026 | 0.009 | 0.005 | 0.87 |

- **At the transformer's first update the value head's gradient on the trunk is 15 times the policy's**, and the whole gradient is 44 times the clip. The review's concern (item 5) is visible even with rewards between 0 and 1 and Huber losses. In that update the clip fraction was 0.43. It settles within a few updates. With real rewards of up to about ±10 this needs watching; the weights are the user's decision (question 1).
- The actor and the critic are clipped apart. That keeps the critic's gradient away from the actor. It does **not** separate the value-head, Q and auxiliary gradients from the policy gradient: they share the trunk and the actor's clip.

**The critic's explained variance, measured after the change of the collection:**

| Update | 1 | 10 | 20 | 50 | 100 | 200 |
|--------|---|----|----|----|-----|-----|
| Transformer run | −0.01 | 0.07 | 0.33 | 0.43 | 0.45 | 0.46 |
| MLP run | −0.01 | 0.05 | −0.46 | −0.17 | −0.01 | −0.02 |

Measured: in the transformer run the policy is stable from update 20 on and a quarter of its decisions are still decided by chance, so the returns vary; there the critic explains 33 to 49% of their variance. In the MLP run the score goes from 0.66 to 0.99 between updates 10 and 20; there the explained variance is negative (down to −1.26), and near zero afterwards, when almost every deal pays 1.0. Not shown, a guess: that the negative values come from the critic lagging behind a fast-moving policy. A critic that is never trained scores about −0.02 (the review's measurement), so a value near zero alone does not show that the critic works; the 0.33 to 0.49 of the transformer run does.

Partner-prediction accuracy in the mock: 0.66 to 0.69 over all decisions with a partner, 0.33 where the partner is not public (chance: the mock's partner is random). The head reads the public partner from the state and cannot know the hidden one.

### Hyperparameters used

All from `src/train/configs/mock.toml`. The code has no defaults: `validate_config` refuses a config with a missing or an unknown key, and the test for criterion 24 removes each key in turn.

| Name | Value | From |
|------|-------|------|
| Clip range | 0.2 | research/05 G1 (0.1–0.2) |
| PPO epochs | 2 | D119 A |
| Discount, GAE λ | 1.0, 0.95 | spec table (proposal) |
| Learning rate | 3e-4, Adam | spec table (proposal) |
| Adam β1, β2, ε | 0.9, 0.999, 1e-8 | PyTorch defaults, written in the config |
| Tables in parallel | 1,024 | spec table (proposal) |
| Update batch | 256 seat-deals | spec 003 |
| Entropy weight | 0.01 | spec table (proposal) |
| Critic loss weight | 0.5 | spec table (proposal) |
| Q-head loss weight | 0.5 | spec table (proposal) |
| Partner and card-location loss weights | 0.1 each | research/05 C2 |
| Gradient clip | 0.5 | spec table (proposal); actor and critic are clipped apart |
| Past-checkpoint share | 0.2 | D14 |
| Precision | fp32 | spec 003 |
| Advantage normalisation | on | spec, In scope |
| **Value-head loss weight** | 0.5 | **not approved**: Claude's proposal, a config value (question 1) |
| **Form of the value, critic and Q losses; Huber delta** | Huber; 1.0 | review item 5; the delta is Claude's proposal (question 1) |
| **Seat-deals per update** | 2,048 | Claude's proposal: 8 update batches (question 3) |
| **First scale of the policy head** | 1.0 | PyTorch default; changed from 0.01 after the measurement in "Finding 1" |
| Frozen seats in a deal with a past checkpoint | 1 to 3 | Claude's proposal |
| Snapshot pool size, updates between snapshots | 10, 20 | Claude's proposal (research/05 C5: "the last N snapshots") |
| Width, layers, heads, feed-forward | 256, 4, 4, 1024 | D18 |
| MLP control, hidden layers | 512, 512, 256 | D18 |
| Critic, hidden layers | 512, 512, 256 | Claude's proposal (D119 B fixes the type, not the size) |
| Acting cache: first size, second-pass limit | 32 tokens, 8 new tokens | speed only; no effect on results |
| Seed | 3 | Claude's proposal |
| Mock only: decisions per seat and deal, legal slots, chance of an extra token, share of random state bits | 3 to 6, 4, 0.5, 0.1 | Claude's proposal |

Numbers that stay in the code, because they are not hyperparameters of the learner: the constants of the mock's made-up data (named at the top of `mock_env.py`), the split of the mock reward over the score parts, the 1e-8 that guards the division in the advantage normalisation, the first spread (0.02) of the state-type embedding, the 1e-5 by which score parts may miss the reward, and the rule that a remainder under half an update batch joins the previous minibatch.

### pytest output

`python -m pytest -v` in `src\train` with the GPU hidden (`CUDA_VISIBLE_DEVICES=-1`), probe built, on 2026-11-11 after the change to ten score parts. This is what the GitHub runner runs, plus the three probe tests:

```
platform win32
rootdir: src\train
configfile: pyproject.toml
testpaths: tests
collected 75 items

tests/test_bench.py::test_l1_l2_and_cached_acting_give_the_same_outputs PASSED
tests/test_bench.py::test_ragged_cached_acting_equals_l2_for_deals_at_different_points PASSED
tests/test_bench.py::test_ragged_step_wraps_around_and_returns_host_actions PASSED
tests/test_bench.py::test_l1_bucketed_equals_l1_padded PASSED
tests/test_bench.py::test_l2_state_tokens_do_not_see_each_other_or_later_play_tokens PASSED
tests/test_bench.py::test_l2_mask_shape_and_rows PASSED
tests/test_bench.py::test_illegal_actions_get_zero_probability PASSED
tests/test_bench.py::test_update_step_gives_finite_loss_and_changes_weights[mlp] PASSED
tests/test_bench.py::test_update_step_gives_finite_loss_and_changes_weights[L1 padded] PASSED
tests/test_bench.py::test_update_step_gives_finite_loss_and_changes_weights[L1 bucketed] PASSED
tests/test_bench.py::test_update_step_gives_finite_loss_and_changes_weights[L2] PASSED
tests/test_bench.py::test_sizes_match_the_plan PASSED
tests/test_bench.py::test_end_to_end_formula PASSED
tests/test_bench.py::test_time_step_gives_one_value_per_repeat PASSED
tests/test_bench.py::test_quick_benchmark_runs_on_cpu PASSED
tests/test_bench.py::test_sizes_can_be_set_to_the_encoding_contract PASSED
tests/test_bench.py::test_half_precision_update_is_finite_on_the_gpu SKIPPED
tests/test_env.py::test_envcheck_reports_this_gpu_and_exits_zero SKIPPED
tests/test_env.py::test_envcheck_exits_nonzero_without_cuda PASSED
tests/test_env.py::test_matmul_on_gpu_matches_cpu_within_1e_5 SKIPPED
tests/test_env.py::test_pyo3_probe_values_and_twelve_threads PASSED
tests/test_env.py::test_pyo3_probe_threads_run_in_parallel PASSED
tests/test_env.py::test_pyo3_probe_releases_the_gil PASSED
tests/test_spec007.py::test_c01_contract_sizes_match_the_tables_of_encoding_md PASSED
tests/test_spec007.py::test_c01_the_number_of_score_parts_is_one_constant PASSED
tests/test_spec007.py::test_c01_the_mock_emits_ten_parts_that_add_up_to_the_reward PASSED
tests/test_spec007.py::test_c02_forward_pass_gives_the_stated_shapes_and_no_nan[transformer] PASSED
tests/test_spec007.py::test_c02_forward_pass_gives_the_stated_shapes_and_no_nan[mlp] PASSED
tests/test_spec007.py::test_c03_illegal_actions_have_probability_zero_and_a_single_legal_action_one[transformer] PASSED
tests/test_spec007.py::test_c03_illegal_actions_have_probability_zero_and_a_single_legal_action_one[mlp] PASSED
tests/test_spec007.py::test_c04_l2_mask_which_outputs_move_when_a_token_changes PASSED
tests/test_spec007.py::test_c05_one_l2_pass_equals_each_decision_alone_from_its_prefix PASSED
tests/test_spec007.py::test_c06_cached_acting_equals_the_full_pass_also_across_a_change_of_weights PASSED
tests/test_spec007.py::test_c06_acting_and_update_see_byte_identical_actor_inputs[transformer] PASSED
tests/test_spec007.py::test_c06_acting_and_update_see_byte_identical_actor_inputs[mlp] PASSED
tests/test_spec007.py::test_c07_the_actor_does_not_depend_on_the_critics_hidden_features[transformer] PASSED
tests/test_spec007.py::test_c07_the_actor_does_not_depend_on_the_critics_hidden_features[mlp] PASSED
tests/test_spec007.py::test_c08_parameter_counts_are_in_range_printed_and_logged PASSED
tests/test_spec007.py::test_c09_gae_matches_a_hand_computed_example_of_two_deals PASSED
tests/test_spec007.py::test_c10_the_reward_reaches_every_decision_of_that_seat_and_deal_only PASSED
tests/test_spec007.py::test_c10_score_parts_that_do_not_add_up_to_the_reward_are_refused PASSED
tests/test_spec007.py::test_c11_clipped_policy_loss_equals_a_hand_computed_value PASSED
tests/test_spec007.py::test_c12_no_gradient_reaches_an_illegal_actions_logit_in_the_update[transformer] PASSED
tests/test_spec007.py::test_c12_no_gradient_reaches_an_illegal_actions_logit_in_the_update[mlp] PASSED
tests/test_spec007.py::test_c13_every_head_and_the_critic_get_a_gradient_from_one_update[transformer] PASSED
tests/test_spec007.py::test_c13_every_head_and_the_critic_get_a_gradient_from_one_update[mlp] PASSED
tests/test_spec007.py::test_c13_the_critic_reads_the_environments_critic_array_of_the_same_decision PASSED
tests/test_spec007.py::test_c13_the_value_losses_are_huber_or_squared_as_the_config_says PASSED
tests/test_spec007.py::test_c14_card_loss_counts_unseen_cards_only_and_partner_loss_skips_none PASSED
tests/test_spec007.py::test_c14_in_the_update_the_unseen_cards_come_from_state_block_f4 PASSED
tests/test_spec007.py::test_c15_frozen_seats_are_played_by_the_past_checkpoint_and_are_not_trained[transformer] PASSED
tests/test_spec007.py::test_c15_frozen_seats_are_played_by_the_past_checkpoint_and_are_not_trained[mlp] PASSED
tests/test_spec007.py::test_c15_the_share_of_deals_with_a_frozen_seat PASSED
tests/test_spec007.py::test_c16_after_one_update_all_weights_and_the_loss_are_finite[transformer] PASSED
tests/test_spec007.py::test_c16_after_one_update_all_weights_and_the_loss_are_finite[mlp] PASSED
tests/test_spec007.py::test_c16_a_fresh_rollout_has_probability_ratio_one_on_every_decision[transformer] PASSED
tests/test_spec007.py::test_c16_a_fresh_rollout_has_probability_ratio_one_on_every_decision[mlp] PASSED
tests/test_spec007.py::test_c16_a_small_remainder_joins_the_previous_minibatch PASSED
tests/test_spec007.py::test_c17_the_mean_reward_rises_by_half_the_distance_to_the_best[transformer] PASSED
tests/test_spec007.py::test_c17_the_mean_reward_rises_by_half_the_distance_to_the_best[mlp] PASSED
tests/test_spec007.py::test_c17_control_a_reward_given_to_the_wrong_seat_stops_the_learning PASSED
tests/test_spec007.py::test_c17_control_the_token_task_cannot_be_learned_without_the_tokens PASSED
tests/test_spec007.py::test_c18_random_rewards_run_20_updates_and_the_entropy_does_not_collapse[transformer] PASSED
tests/test_spec007.py::test_c18_random_rewards_run_20_updates_and_the_entropy_does_not_collapse[mlp] PASSED
tests/test_spec007.py::test_c19_checkpoint_round_trip_and_encoding_version_check[transformer] PASSED
tests/test_spec007.py::test_c19_checkpoint_round_trip_and_encoding_version_check[mlp] PASSED
tests/test_spec007.py::test_c19_a_resumed_run_equals_an_unbroken_run[transformer] PASSED
tests/test_spec007.py::test_c19_a_resumed_run_equals_an_unbroken_run[mlp] PASSED
tests/test_spec007.py::test_c20_two_cpu_runs_with_one_seed_give_identical_losses[transformer] PASSED
tests/test_spec007.py::test_c20_two_cpu_runs_with_one_seed_give_identical_losses[mlp] PASSED
tests/test_spec007.py::test_c21_tensorboard_files_hold_the_required_scalars PASSED
tests/test_spec007.py::test_c22_the_train_command_runs_on_the_gpu_and_resumes[transformer] SKIPPED
tests/test_spec007.py::test_c22_the_train_command_runs_on_the_gpu_and_resumes[mlp] SKIPPED
tests/test_spec007.py::test_c22_the_train_command_runs_on_the_cpu_and_resumes PASSED
tests/test_spec007.py::test_c24_every_hyperparameter_is_in_the_config_file PASSED

================== 70 passed, 5 skipped in 642.85s (0:10:42) ==================
```

The 5 skipped tests are the GPU tests. They were run on their own afterwards (`python -m pytest -m gpu`): 5 passed in 10 s (nvidia-smi then: 0% busy, 3.9 GB in use by another program).

**Run time and what the `train` workflow runs.** The suite's time depends on what else the PC does. Measured on this PC, all on 2026-11-11:

| Run | Tests | Time | Load on the PC |
|-----|-------|------|----------------|
| `python -m pytest`, GPU used | 74 passed (before D121) | 134 s | nothing else running |
| `python -m pytest`, GPU hidden | 70 passed, 5 skipped | 643 s | another program used the CPU and the GPU (nvidia-smi: 54% busy, 3.2 GB) |
| `python -m pytest -m "not slow"`, GPU hidden | 65 passed, 5 skipped, 5 left out | 327 s | the same load |

Two transformer tests make most of the time: criterion 17 (30 updates; 49 s on the idle PC, 289 s under load) and the resume test of criterion 19 (197 s under load).

Five tests carry the marker `slow`: the two control tests of criterion 17 and the transformer runs of criteria 18, 19 (resume) and 20. Each of them repeats, for the second network or as a control, something another test already shows. A local `python -m pytest` runs them. Without them every criterion from 1 to 21 and 24 still has a test, the PPO tests of criteria 9 to 16 are all in, and criterion 17 runs for both networks.

Recommended command for the workflow: `python -m pytest`, the whole suite, as now. It is under 11 minutes here even with the PC loaded. If the runner proves too slow, `python -m pytest -m "not slow"` is the shorter set described above. On the runner the GPU tests and the three probe tests skip in both cases.

### Decisions taken while building

Places where the spec or the contract left a choice. The user may change any.

1. **Number codes of the auxiliary targets.** Partner: the partner's relative seat, 1 to 3; 0 means "none" and is skipped. Card location: 0 = out of play, 1 to 3 = that other seat. Constants in `contract.py`; the bindings (spec 006) must emit the same codes.
2. **Reward by score part.** The learner expects, at the end of a deal, 4 seats × `N_SCORE_PARTS` parts that add up to each seat's reward, and refuses them if they do not (to 1e-5). Since D121 the contract has this array (`reward_parts`, section 8) with ten parts in a fixed order; `contract.py` holds the same names in the same order and a test compares them with the document. The mock emits ten parts; klop and radelc are zero, as from the engine until its flow C.
3. **Two value outputs.** The critic (full information) gives the advantages. The actor's own value head (observation only) is trained toward the realised return, for later use in the app. Its loss weight of 0.5 is **not approved**; it is a config value and question 1.
4. **Critic target and advantage.** The critic learns the GAE(λ) return of each part. The policy's advantage is the sum over the parts.
5. **Only whole deals under the current weights are trained on** (see "Whole deals"). This replaces the first version, which kept running deals for the next update.
6. **The acting cache is emptied after every update.** The next decision of each seat recomputes its prefix in a second pass.
7. **One past checkpoint at a time.** It is drawn evenly from the pool after every update. Frozen seats are drawn when a deal starts and stay for the deal. Before the first snapshot exists no seat is frozen.
8. **Gradient clipping** is done for the actor and for the critic apart, each to 0.5. This does not separate the policy from the value-head, Q and auxiliary gradients (see above).
9. **The mock's reward is not zero-sum** in the learnable modes. Otherwise the mean reward could not rise when all four seats improve together. For the real, zero-sum game the logs carry `reward/mean_abs` and `reward/std`; `task/score` exists only where the environment says its mean reward is a score.
10. **A remainder minibatch** of under half an update batch joins the previous minibatch.
11. **Shared code.** The transformer layer and the L2 mask are in `model.py`; the benchmark imports them. Its tests pass unchanged.
12. **Files beyond the spec's list:** `src/train/requirements-ci.txt` gets `tensorboard==2.21.0`; `src/train/.gitignore` gets `runs/`.

### Open issues

1. **CI has not run on this code** (criterion 23).
2. **Memory of the acting cache.** One table and seat needs 8 KB per token: 1,024 tables × 4 seats × 130 positions is 4.4 GB (arithmetic). The mock stays under 50 tokens; the transformer run peaked at 3.6 GB. With real deals of up to 128 tokens, 1,024 tables do not fit on this 6 GB card next to the update (question 4).
3. **Decisions played but not trained on:** 61% in the mock runs (see "Whole deals"). The share depends on `seat_deals_per_update` and on the deal length.
4. **The transformer's plateau at 0.81** on the token task (Finding 2). Not explained.
5. **The value head's gradient at the first updates** (15 times the policy's on the trunk at update 1). Watch `grad/trunk_...` in the first real run.
6. **Rollout memory.** A stored decision takes 6,896 bytes (state 5,528, hidden critic features 1,096, mask 160, the rest 112); before the review it took 8,554, with the token array in every row. The tokens are now stored once per seat-deal (1,280 bytes). Computed from the array sizes: 31 MB per 1,000 seat-deals in the mock (4.5 decisions each), 133 MB per 1,000 seat-deals of 20 decisions. `take` builds one padded copy of the finished seat-deals per update.
7. **Checkpoints hold the pool of past checkpoints as weights**, so they are large: with a full pool of 10 snapshots 88 MB for the MLP and 229 MB for the transformer (measured at update 200).
8. **A resumed run equals an unbroken one on the CPU** (tested). On the GPU runs are not bit-reproducible at all (criterion 20 asks for the CPU).
9. **Minibatches are padded to their longest seat-deal.** Seat-deals are not sorted by length; with real deals that wastes work.
10. **Entropy per phase and partner accuracy are measured during the first epoch of an update**, so later minibatches of that epoch see slightly changed weights. Explained variance, action shares, rewards and decision age are exact.
11. **Run folders** under `src/train/runs/` are on this PC and not tracked. Their TensorBoard file names contain the PC's host name; they must not be committed as they are.
12. **Still to do before a commit (D72):** the user's read of the diff, CI after the push, the copy of the results into `docs/progress.md` (D109).

### Review fixes

From the review pass of the main session, items 1 to 13.

1. **The learning test.** (a) `test_c16_a_fresh_rollout_has_probability_ratio_one_on_every_decision` [t, m]: after two updates, with a past checkpoint in play, the stored log-probabilities and those the update computes agree to 1e-5; the update measures the same at every update (`ppo/behaviour_mismatch`). (b) The mock's rewarded slot depends on the observation: on a state hint for the MLP, on the deal's cue token for the transformer. (c) Criterion 17 asserts the mismatch, the decision age, `approx_kl` and the clip fraction at every update. The ten planted bugs were put back one at a time; the table below names the tests that fail for each.
2. **Stale data.** Only whole deals under the current weights are trained on; the decision age is logged and is 0. Cost: 61% of the decisions played are not trained on in the mock runs. See "Whole deals".
3. **Parts and reward.** `Rollout.finish` refuses score parts that do not add up to the seat's reward (to 1e-5); tested.
4. **Buffers and one source.** The loop copies what it needs from the observation before `env.step`. The actor is fed from `obs["state"]`, the same array that is stored. `test_c06_acting_and_update_see_byte_identical_actor_inputs` compares the actor's inputs when acting with the update's, byte for byte.
5. **Loss scale.** Huber losses for the critic, the value head and the Q head, form and delta in the config. Gradient norm of each loss term on the trunk logged once per update. The weights are unchanged.
6. **Resume.** A checkpoint holds the pool, the frozen network, the random generators and the environment's state. `--resume` goes on from one. `test_c19_a_resumed_run_equals_an_unbroken_run` [t, m]: 3 updates, checkpoint, load, 3 updates give the same losses as 6 updates in one go, with a past checkpoint playing.
7. **Criterion 12** is tested on the path the update takes (a hook on the raw policy logits through `evaluate` and `compute_losses`). The unused function `masked_log_probs` is removed.
8. **Remainder minibatch.** Under half an update batch it joins the previous one; tested.
9. **Rollout memory.** Tokens are stored once per seat-deal; target arrays are stored as small integers. Sizes in open issue 6.
10. **Test gaps.** c15: the past checkpoint gets weights that give it away (it always takes its highest legal slot); the test checks that frozen seats take that slot and trained seats do not. c06: the weights change in the middle and the cache is reset; every decision must equal the decision computed alone with the weights of its time. c07: the critic-only features of a collected batch are changed; the actor's outputs and gradients must not move by a bit. c14: a batch built by hand, with three cards marked in F4.
11. **The critic check.** Rewritten above with what is measured and what is a guess, and looked at again after item 2.
12. **Nits.** The logs carry `reward/mean_abs`, `reward/std` and, for the mock, `task/score`. Results of deals with no stored rows no longer stay in the storage.
13. **Score parts.** `N_SCORE_PARTS` is used everywhere; no literal number of parts is left in the package. It is 10 since D121. `test_c01_the_number_of_score_parts_is_one_constant` runs an update of both networks with 12 parts.

Also corrected: the weight of the actor's own value head is marked as not approved, here and in the config; decision 8 no longer claims to separate the value and Q gradients from the policy gradient.

**The ten planted bugs.** Each was put into the code alone (as a patch applied when the tests start), and the whole of `tests/test_spec007.py` was run on the CPU. The last column lists every test that failed.

One correction to these runs: when they were made, `test_c18_...[transformer]` failed on the unchanged code too (its entropy floor did not fit the new first scale of the policy head, see Finding 1), so it failed in all ten. That test was then corrected and run again alone under each planted bug; the table has its results from that second run.

| # | Planted bug | Tests that fail |
|---|-------------|-----------------|
| 1 | The mask is not applied in the update | 8 fail: `test_c12_no_gradient_reaches_an_illegal_actions_logit_in_the_update` [m, t]; `test_c16_a_fresh_rollout_has_probability_ratio_one_on_every_decision` [m, t]; `test_c17_the_mean_reward_rises_by_half_the_distance_to_the_best` [m, t]; `test_c18_random_rewards_run_20_updates_and_the_entropy_does_not_collapse` [m, t] |
| 2 | The stored log-probability is zeroed | 6 fail: `test_c16_a_fresh_rollout_has_probability_ratio_one_on_every_decision` [m, t]; `test_c17_the_mean_reward_rises_by_half_the_distance_to_the_best` [m, t]; `test_c18_random_rewards_run_20_updates_and_the_entropy_does_not_collapse` [m, t] |
| 3 | The critic is never trained (its loss is left out of the total) | 4 fail: `test_c07_the_actor_does_not_depend_on_the_critics_hidden_features` [m, t]; `test_c13_every_head_and_the_critic_get_a_gradient_from_one_update` [m, t] |
| 4 | The advantages are reversed in time | 1 fail: `test_c09_gae_matches_a_hand_computed_example_of_two_deals` |
| 5 | No bootstrapping in GAE (the value of the next decision is dropped) | 1 fail: `test_c09_gae_matches_a_hand_computed_example_of_two_deals` |
| 6 | The reward is averaged over a seat's deals | 1 fail: `test_c10_the_reward_reaches_every_decision_of_that_seat_and_deal_only` |
| 7 | The reward is swapped with the seat's other finished deal | 4 fail: `test_c10_the_reward_reaches_every_decision_of_that_seat_and_deal_only`; `test_c17_the_mean_reward_rises_by_half_the_distance_to_the_best` [m, t]; `test_c18_random_rewards_run_20_updates_and_the_entropy_does_not_collapse` [t] |
| 8 | The state is rolled to another seat-deal in the update | 7 fail: `test_c06_acting_and_update_see_byte_identical_actor_inputs` [m, t]; `test_c13_the_critic_reads_the_environments_critic_array_of_the_same_decision`; `test_c16_a_fresh_rollout_has_probability_ratio_one_on_every_decision` [m, t]; `test_c17_the_mean_reward_rises_by_half_the_distance_to_the_best` [m, t] |
| 9 | Transformer: the decision points are zeroed in the update | 6 fail: `test_c06_acting_and_update_see_byte_identical_actor_inputs` [m, t]; `test_c13_every_head_and_the_critic_get_a_gradient_from_one_update` [t]; `test_c16_a_fresh_rollout_has_probability_ratio_one_on_every_decision` [t]; `test_c17_the_mean_reward_rises_by_half_the_distance_to_the_best` [t]; `test_c18_random_rewards_run_20_updates_and_the_entropy_does_not_collapse` [t] |
| 10 | Transformer: the acting cache is never reset | 2 fail: `test_c06_cached_acting_equals_the_full_pass_also_across_a_change_of_weights`; `test_c19_a_resumed_run_equals_an_unbroken_run` [t] |

### Questions for the user

1. **Loss weights and the Huber delta.** The actor's own value head has weight 0.5 (not approved) and the Huber delta is 1.0 (my proposal). At the transformer's first update the value head's gradient on the trunk was 15 times the policy's. Keep 0.5 and 1.0, lower the value-head weight, or leave this head out of the loss until the app needs it?
2. **Reward by score part (decision 2).** Answered by the user on 2026-11-11 (D121): yes, ten parts, in `docs/encoding.md` section 8. Done in the code.
3. **Seat-deals per update.** 2,048 in the mock; 61% of the decisions played are then not trained on. Recommended: decide it in spec 008 from the measured deal length; a longer collection wastes less.
4. **Acting cache memory (open issue 2).** Fewer tables, a half-precision cache, or a shorter sequence limit? Recommended: start the first real run with 512 tables and measure.
5. **First scale of the policy head.** Changed from 0.01 to 1.0 after the measurement in Finding 1. 1.0 learns fastest but starts from a non-uniform policy; 0.1 starts uniform and learns more slowly. Keep 1.0, or take a value between?
6. **The transformer's plateau (Finding 2).** It follows the cue token and ignores the state hint. Is a target-entropy controller (research/05 e7) wanted before the first real run, or only if the monitors show a collapse?
