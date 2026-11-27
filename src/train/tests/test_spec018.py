"""Tests for spec 018 (first runs on deal flow B), written from its criteria before the code.

The name of each test starts with its criterion number. Criteria 1 to 5: the monitors of
the round of announcements, on built token arrays with known contents and on deals of the
engine. Criteria 6 and 7: the opponent `silent-random` in the evaluation. Criterion 10: the
two configs of the runs against the smoke configs of flow A, key by key. The runs and the
measurement of criterion 8 are not tests. D155 (Batch 22, after the first transformer run ran
out of memory): the acting cache is freed before the update step, and no number changes.
"""

import json
import tomllib
from pathlib import Path

import numpy as np
import pytest

from tarok_train import contract as C
from tarok_train import ppo, train
from tarok_train.model import KVCache
from tarok_train.rollout import SeatDeals

from test_spec007 import CPU
from test_spec008 import ci_cfg, engine
from test_spec015 import result_of, run_evaluation, trained_checkpoint, versus

CONFIGS = Path(__file__).resolve().parents[1] / "configs"

ANNOUNCE = C.EVENT_GROUPS["ANNOUNCE"][0]  # 146: trula, kralji, kralj ultimo, pagat ultimo, valat
KONTRA = C.EVENT_GROUPS["KONTRA"][0]  # 151: on the game, then on the five bonuses
BONUSES = ("trula", "kralji", "kralj_ultimo", "pagat_ultimo", "valat")
LEVELS = ("none", "kontra", "re", "sub", "mort")
ROUND = C.PHASES.index("announcements")
PLAY = C.PHASES.index("play")
DONE_SLOT = C.ACTION_GROUPS["ANNOUNCE_DONE"][0]  # 88
PLAY_EVENT = C.EVENT_GROUPS["PLAY"][0]


# --------------------------------------------------------------------------
# Built seat-deals
# --------------------------------------------------------------------------


def seat_deal(table, seat, deal, events, phases=(PLAY,), reward=0.0, padding=()):
    """One seat-deal: its token events (the event field), the phase of each decision, its reward.

    padding: events written after the seat's last visible token (they must not be read).
    """
    return {"table": table, "seat": seat, "deal": deal, "events": list(events), "phases": list(phases), "reward": reward, "padding": list(padding)}


def built(rows) -> SeatDeals:
    """SeatDeals of the given rows, padded as `Rollout.take` pads them."""
    g = len(rows)
    s = max(len(row["phases"]) for row in rows)
    p = max(len(row["events"]) + len(row["padding"]) for row in rows) + 1
    valid = np.zeros((g, s), dtype=bool)
    points = np.zeros((g, s), dtype=np.int64)
    tokens = np.zeros((g, p, len(C.TOKEN_FIELDS)), dtype=np.int16)
    state = np.zeros((g, s, C.STATE_FEATURES), dtype=np.float32)
    for i, row in enumerate(rows):
        n = len(row["phases"])
        valid[i, :n] = True
        points[i, :n] = len(row["events"])  # every token is visible at the seat's decisions
        events = [*row["events"], *row["padding"]]
        tokens[i, : len(events), 0] = events
        tokens[i, : len(events), 1] = row["seat"]
        for k, phase in enumerate(row["phases"]):
            state[i, k, C.PHASE_BLOCK[0] + phase] = 1.0
    reward = np.array([row["reward"] for row in rows], dtype=np.float32)
    parts = np.zeros((g, C.N_SCORE_PARTS), dtype=np.float32)
    parts[:, 0] = reward
    return SeatDeals(
        table=np.array([row["table"] for row in rows], dtype=np.int64),
        seat=np.array([row["seat"] for row in rows], dtype=np.int64),
        deal=np.array([row["deal"] for row in rows], dtype=np.int64),
        reward=reward,
        parts=parts,
        true_parts=parts.copy(),
        valid=valid,
        points=points,
        tokens=tokens,
        state=state,
        mask=np.ones((g, s, C.N_ACTIONS), dtype=bool),
        critic_extra=np.zeros((g, s, C.CRITIC_EXTRA), dtype=np.float32),
        partner_target=np.zeros((g, s), dtype=np.int8),
        card_target=np.zeros((g, s, C.N_CARDS), dtype=np.int8),
        action=np.zeros((g, s), dtype=np.int16),
        logp=np.zeros((g, s), dtype=np.float32),
        value=np.zeros((g, s, C.N_SCORE_PARTS), dtype=np.float32),
        version=np.zeros((g, s), dtype=np.int32),
    )


def head(*events):
    """A deal's tokens up to the round: start, a bid, a contract; then `events`; then a play."""
    return [C.EVENT_GROUPS["BOS"][0], C.EVENT_GROUPS["BID"][0] + 4, C.EVENT_GROUPS["CONTRACT"][0] + 4, *events, PLAY_EVENT + 3]


# --------------------------------------------------------------------------
# Criteria 1 and 5: the announced bonuses, one deal counted once
# --------------------------------------------------------------------------


def test_c01_each_bonus_share_counts_the_deals_with_its_announcement_token():
    rows = [
        seat_deal(0, 0, 0, head(ANNOUNCE + 0, ANNOUNCE + 4)),  # trula and valat
        seat_deal(1, 0, 0, head(ANNOUNCE + 4)),  # valat
        seat_deal(2, 0, 0, head(ANNOUNCE + 1, ANNOUNCE + 2, ANNOUNCE + 3)),  # kralji, kralj ultimo, pagat ultimo
        seat_deal(3, 0, 0, head()),  # nothing said
    ]
    stats = ppo.round_stats(built(rows))
    assert stats["round/announced_trula"] == pytest.approx(1 / 4)
    assert stats["round/announced_kralji"] == pytest.approx(1 / 4)
    assert stats["round/announced_kralj_ultimo"] == pytest.approx(1 / 4)
    assert stats["round/announced_pagat_ultimo"] == pytest.approx(1 / 4)
    assert stats["round/announced_valat"] == pytest.approx(2 / 4)


@pytest.mark.parametrize("index", range(5))
def test_c01_one_announcement_sets_its_own_bonus_and_no_other(index):
    stats = ppo.round_stats(built([seat_deal(0, 0, 0, head(ANNOUNCE + index))]))
    for other, name in enumerate(BONUSES):
        assert stats[f"round/announced_{name}"] == (1.0 if other == index else 0.0), name


def test_c01_neighbouring_event_ids_are_not_announcements():
    # 145 is the last discard shown, 151 the kontra on the game, 157 "done" (never written), 158 reserved.
    events = head(ANNOUNCE - 1, KONTRA, C.EVENT_GROUPS["ANNOUNCE_DONE"][0], C.EVENT_GROUPS["ANNOUNCE_BARVNI_VALAT"][0])
    stats = ppo.round_stats(built([seat_deal(0, 0, 0, events)]))
    assert all(stats[f"round/announced_{name}"] == 0.0 for name in BONUSES)


def test_c01_tokens_after_the_seats_last_visible_token_are_not_read():
    stats = ppo.round_stats(built([seat_deal(0, 0, 0, head(), padding=[ANNOUNCE + 4, KONTRA])]))
    assert stats["round/announced_valat"] == 0.0 and stats["round/game_none"] == 1.0


def test_c05_a_deal_is_counted_once_whatever_number_of_its_seats_is_trained():
    valat = head(ANNOUNCE + 4)
    rows = [
        # Table 0, deal 5: four trained seats, each with the deal's tokens.
        *(seat_deal(0, seat, 5, valat) for seat in range(4)),
        # Table 0, deal 6 and table 1, deal 5: one trained seat each, nothing said.
        seat_deal(0, 2, 6, head()),
        seat_deal(1, 3, 5, head()),
    ]
    stats = ppo.round_stats(built(rows))
    assert stats["round/announced_valat"] == pytest.approx(1 / 3)  # 4 / 6 if each seat-deal counted
    assert stats["round/game_none"] == pytest.approx(1.0)


def test_c05_the_same_deal_number_at_two_tables_is_two_deals():
    rows = [seat_deal(0, 0, 7, head(ANNOUNCE + 4)), seat_deal(1, 0, 7, head())]
    assert ppo.round_stats(built(rows))["round/announced_valat"] == pytest.approx(1 / 2)


# --------------------------------------------------------------------------
# Criterion 2: the level of the game at the end of the round, and kontra on a bonus
# --------------------------------------------------------------------------


@pytest.mark.parametrize("level", range(5))
def test_c02_the_level_of_the_game_is_the_number_of_kontra_tokens_on_the_game(level):
    # Each kontra token raises the level by one: the last one written gives the level at the end.
    events = head(ANNOUNCE + 0, *([KONTRA] * level))
    stats = ppo.round_stats(built([seat_deal(0, 0, 0, events)]))
    for index, name in enumerate(LEVELS):
        assert stats[f"round/game_{name}"] == (1.0 if index == level else 0.0), name
    assert stats["round/bonus_kontra"] == 0.0


def test_c02_the_five_level_shares_add_up_to_one():
    rows = [seat_deal(table, 0, 0, head(*([KONTRA] * level))) for table, level in enumerate([0, 0, 1, 2, 2, 2, 3, 4])]
    stats = ppo.round_stats(built(rows))
    assert [stats[f"round/game_{name}"] for name in LEVELS] == pytest.approx([2 / 8, 1 / 8, 3 / 8, 1 / 8, 1 / 8])
    assert sum(stats[f"round/game_{name}"] for name in LEVELS) == pytest.approx(1.0)


def test_c02_a_kontra_on_a_bonus_does_not_raise_the_game_and_counts_as_bonus_kontra():
    rows = [
        seat_deal(0, 0, 0, head(ANNOUNCE + 4, KONTRA + 5, KONTRA + 5)),  # re on the valat
        seat_deal(1, 0, 0, head(ANNOUNCE + 0, KONTRA + 1, KONTRA)),  # kontra on trula, then on the game
        seat_deal(2, 0, 0, head(ANNOUNCE + 0)),
        seat_deal(3, 0, 0, head(KONTRA)),
    ]
    stats = ppo.round_stats(built(rows))
    assert stats["round/bonus_kontra"] == pytest.approx(2 / 4)
    assert stats["round/game_none"] == pytest.approx(2 / 4)
    assert stats["round/game_kontra"] == pytest.approx(2 / 4)


@pytest.mark.parametrize("target", range(1, 6))
def test_c02_every_bonus_kontra_slot_counts(target):
    stats = ppo.round_stats(built([seat_deal(0, 0, 0, head(KONTRA + target))]))
    assert stats["round/bonus_kontra"] == 1.0 and stats["round/game_none"] == 1.0


def test_c02_more_than_four_kontras_on_the_game_is_an_error():
    with pytest.raises(ValueError, match="kontra"):
        ppo.round_stats(built([seat_deal(0, 0, 0, head(*([KONTRA] * 5)))]))


# --------------------------------------------------------------------------
# Criterion 3: the size of the rewards, over the trained seat-deals
# --------------------------------------------------------------------------


def test_c03_reward_size_over_the_trained_seat_deals():
    rewards = [-12.0, 4.0, 10.0, -0.5, 30.0]  # 10 is not above 10
    rows = [seat_deal(table, 0, 0, head(), reward=r) for table, r in enumerate(rewards)]
    stats = ppo.round_stats(built(rows))
    assert stats["reward/abs_mean"] == pytest.approx(np.mean(np.abs(rewards)))
    assert stats["reward/abs_max"] == pytest.approx(30.0)
    assert stats["reward/share_above_10"] == pytest.approx(2 / 5)


def test_c03_every_trained_seat_of_a_deal_counts_for_the_reward_size():
    # Over seat-deals, not deals: the four seats of one deal are four rewards.
    rows = [seat_deal(0, seat, 0, head(), reward=r) for seat, r in enumerate([-30.0, 10.0, 10.0, 10.0])]
    stats = ppo.round_stats(built(rows))
    assert stats["reward/abs_mean"] == pytest.approx(15.0)
    assert stats["reward/share_above_10"] == pytest.approx(1 / 4)


# --------------------------------------------------------------------------
# Criterion 4: the decisions of the round
# --------------------------------------------------------------------------


def test_c04_the_share_of_trained_decisions_in_the_round_and_per_seat_deal():
    rows = [
        seat_deal(0, 0, 0, head(), phases=[0, ROUND, ROUND, PLAY, PLAY]),
        seat_deal(0, 1, 0, head(), phases=[0, ROUND, PLAY]),
        seat_deal(1, 0, 0, head(), phases=[PLAY, PLAY, PLAY]),  # padding of the shorter rows is not a decision
    ]
    stats = ppo.round_stats(built(rows))
    assert stats["round/decision_share"] == pytest.approx(3 / 11)
    assert stats["round/decisions_per_seat_deal"] == pytest.approx(3 / 3)


def test_c04_the_phase_next_to_the_round_is_not_the_round():
    rows = [seat_deal(0, 0, 0, head(), phases=[ROUND - 1, ROUND + 1, ROUND + 1, ROUND])]
    stats = ppo.round_stats(built(rows))
    assert stats["round/decision_share"] == pytest.approx(1 / 4)


def test_c01_c04_the_deal_monitors_hold_the_round_monitors():
    rows = [seat_deal(0, 0, 0, head(ANNOUNCE + 4, KONTRA), phases=[ROUND, PLAY], reward=3.0)]
    deals = built(rows)
    in_deal_stats = ppo.deal_stats(deals)
    for name, value in ppo.round_stats(deals).items():
        assert in_deal_stats[name] == value, name


# --------------------------------------------------------------------------
# Criteria 1 to 5 on deals of the engine: the tokens against the actions taken
# --------------------------------------------------------------------------


def engine_deals(n_tables=64, seed=31):
    """Random legal play on the engine: one seat-deal per deal (its tokens at the deal's last decision)
    and, from the actions taken, what the round said in each deal."""
    engine_env = engine()
    from tarok_train.evaluate import RandomLegal

    env = engine_env.EngineEnv(n_tables, seed, 2)
    pick = RandomLegal(seed)
    rows = np.arange(n_tables)
    said = [[] for _ in range(n_tables)]
    last_tokens = [None] * n_tables
    last_count = np.zeros(n_tables, dtype=np.int64)
    open_tables = np.ones(n_tables, dtype=bool)
    while open_tables.any():
        obs = env.observe()
        action = pick(rows, obs)
        for table in np.flatnonzero(open_tables):
            said[table].append(int(action[table]))
            last_tokens[table] = obs["tokens"][table].copy()
            last_count[table] = obs["n_tokens"][table]
        done, _, _ = env.step(action)
        open_tables &= ~done
    data = [seat_deal(table, 0, 0, []) for table in rows]
    deals = built(data)
    width = int(last_count.max())
    deals.tokens = np.stack([tokens[:width] for tokens in last_tokens]).astype(np.int16)
    deals.points = last_count[:, None]
    return deals, said


def test_c01_c02_on_engine_deals_the_tokens_say_what_the_actions_said():
    deals, said = engine_deals()
    stats = ppo.round_stats(deals)
    announce, kontra = C.ACTION_GROUPS["ANNOUNCE"][0], C.ACTION_GROUPS["KONTRA"][0]
    n = len(said)
    for index, name in enumerate(BONUSES):
        expected = sum(announce + index in slots for slots in said) / n
        assert stats[f"round/announced_{name}"] == pytest.approx(expected), name
    levels = np.array([slots.count(kontra) for slots in said])
    for index, name in enumerate(LEVELS):
        assert stats[f"round/game_{name}"] == pytest.approx(float((levels == index).mean())), name
    bonus = sum(any(kontra + 1 <= slot <= kontra + 5 for slot in slots) for slots in said) / n
    assert stats["round/bonus_kontra"] == pytest.approx(bonus)
    # Random legal play in the round says a lot (spec 009's review): the test is not about empty rounds.
    assert stats["round/announced_valat"] > 0.5 and stats["round/game_none"] < 0.5


def test_c01_c05_the_trainer_logs_the_round_monitors_every_update():
    engine()
    trainer = train.Trainer(ci_cfg(**{"run.network": "mlp"}), CPU)
    try:
        history = trainer.run(2)
    finally:
        trainer.close()
    names = [
        *(f"round/announced_{name}" for name in BONUSES),
        *(f"round/game_{name}" for name in LEVELS),
        "round/bonus_kontra",
        "reward/abs_mean",
        "reward/abs_max",
        "reward/share_above_10",
        "round/decision_share",
        "round/decisions_per_seat_deal",
    ]
    for stats in history:
        for name in names:
            assert name in stats and np.isfinite(stats[name]), name
        assert sum(stats[f"round/game_{name}"] for name in LEVELS) == pytest.approx(1.0)
        assert 0.0 < stats["round/decision_share"] < 1.0
        assert stats["reward/abs_max"] >= stats["reward/abs_mean"] > 0.0


# --------------------------------------------------------------------------
# Criterion 6: the silent opponent
# --------------------------------------------------------------------------


def test_c06_four_silent_seats_over_500_deals_write_no_announcement_or_kontra_and_play_only_legal_actions():
    engine_env = engine()
    from tarok_train import evaluate

    n = 500
    env = engine_env.EngineEnv(n, 18, 2)
    silent = evaluate.SilentRandom(18)
    rows = np.arange(n)
    open_tables = np.ones(n, dtype=bool)
    in_round = said_done = 0
    while open_tables.any():
        obs = env.observe()
        action = silent(rows, obs)
        live = np.flatnonzero(open_tables)
        assert obs["mask"][live, action[live]].all()  # every action is legal
        tokens = obs["tokens"][live, :, 0]
        visible = np.arange(tokens.shape[1])[None, :] < obs["n_tokens"][live][:, None]
        written = tokens[visible]
        assert not ((written >= ANNOUNCE) & (written < KONTRA + 6)).any()  # no token 146 to 156
        round_rows = live[obs["mask"][live, DONE_SLOT]]
        in_round += len(round_rows)
        said_done += int((action[round_rows] == DONE_SLOT).sum())
        done, _, _ = env.step(action)  # an illegal action would raise here
        open_tables &= ~done
    assert env.illegal_actions == 0
    assert in_round >= n and said_done == in_round  # the round was reached, and "done" said at every turn of it


def test_c06_outside_the_round_the_silent_opponent_is_random_legal_play_with_the_same_numbers():
    engine_env = engine()
    from tarok_train import evaluate

    n = 64
    env = engine_env.EngineEnv(n, 5, 2)
    silent, plain = evaluate.SilentRandom(7), evaluate.RandomLegal(7)
    rows = np.arange(n)
    compared = 0
    for _ in range(400):
        obs = env.observe()
        a, b = silent(rows, obs), plain(rows, obs)
        outside = ~obs["mask"][:, DONE_SLOT]
        assert np.array_equal(a[outside], b[outside])
        assert (a[~outside] == DONE_SLOT).all()
        compared += int(outside.sum())
        env.step(b)
    assert compared > 0


def test_c06_the_silent_opponent_draws_its_numbers_whatever_the_phase():
    # The stream of numbers does not depend on how many tables stand in the round.
    from tarok_train import evaluate

    mask = np.zeros((3, C.N_ACTIONS), dtype=bool)
    mask[:, [90, 91, 92]] = True
    in_round = mask.copy()
    in_round[1] = False
    in_round[1, [77, 82, DONE_SLOT]] = True
    a, b = evaluate.SilentRandom(3), evaluate.SilentRandom(3)
    a(np.arange(3), {"mask": in_round})
    b(np.arange(3), {"mask": mask})
    assert np.array_equal(a(np.arange(3), {"mask": mask}), b(np.arange(3), {"mask": mask}))


# --------------------------------------------------------------------------
# Criterion 7: the file and the JSON name the opponent; random and a checkpoint as before
# --------------------------------------------------------------------------


@pytest.fixture(scope="module")
def checkpoint(tmp_path_factory):
    engine()
    return trained_checkpoint(tmp_path_factory.mktemp("spec018"), "a", seed=8)


def test_c07_silent_random_writes_a_file_that_names_it(checkpoint, capsys):
    assert run_evaluation("--checkpoint", checkpoint, "--opponent", "silent-random", "--deals", 24, "--tables", 12, "--seed", 5) == 0
    path = checkpoint.parent / "evaluation-000001-vs-silent-random.json"
    result = result_of(path)
    assert result["opponent_kind"] == "silent-random"
    assert "silent" in result["opponents"] and result["opponent"] is None
    assert "beats_random" not in result
    assert result["above_zero"] == (result["low"] > 0) and result["below_zero"] == (result["high"] < 0)
    assert result["seed"] == 5 and result["deals"] == 24 and result["updates"] == 1
    assert "silent" in capsys.readouterr().out


def test_c07_random_is_the_default_and_its_file_is_the_one_of_before(checkpoint):
    assert run_evaluation("--checkpoint", checkpoint, "--deals", 24, "--tables", 12, "--seed", 5) == 0
    default = result_of(checkpoint.parent / "evaluation-000001.json")
    assert run_evaluation("--checkpoint", checkpoint, "--opponent", "random", "--deals", 24, "--tables", 12, "--seed", 5) == 0
    named = result_of(checkpoint.parent / "evaluation-000001.json")
    assert named["rewards"] == default["rewards"]
    for result in (default, named):
        assert result["opponent_kind"] == "random" and result["opponents"] == "random legal play"
        assert result["opponent"] is None and result["beats_random"] == (result["low"] > 0)


def test_c07_silent_random_and_random_are_different_opponents_on_the_same_deals(checkpoint):
    assert run_evaluation("--checkpoint", checkpoint, "--opponent", "silent-random", "--deals", 24, "--tables", 12, "--seed", 5) == 0
    assert run_evaluation("--checkpoint", checkpoint, "--opponent", "random", "--deals", 24, "--tables", 12, "--seed", 5) == 0
    silent = result_of(checkpoint.parent / "evaluation-000001-vs-silent-random.json")
    plain = result_of(checkpoint.parent / "evaluation-000001.json")
    assert not np.allclose(silent["rewards"], plain["rewards"])
    # Random seats announce valat in most deals: the size of the rewards shows it.
    assert np.abs(plain["rewards"]).mean() > np.abs(silent["rewards"]).mean()


def test_c07_the_untrained_network_against_silent_random_has_its_own_file(checkpoint):
    assert run_evaluation("--checkpoint", checkpoint, "--opponent", "silent-random", "--untrained", "--deals", 24, "--tables", 12, "--seed", 5) == 0
    result = result_of(checkpoint.parent / "evaluation-untrained-vs-silent-random.json")
    assert result["updates"] == 0 and result["opponent_kind"] == "silent-random"


def test_c07_a_checkpoint_as_the_opponent_works_as_before(checkpoint, tmp_path):
    b = trained_checkpoint(tmp_path, "b", seed=9)
    assert run_evaluation("--checkpoint", checkpoint, "--opponent", b, "--deals", 24, "--tables", 12, "--seed", 5, "--overwrite") == 0
    result = result_of(versus(checkpoint, b))
    assert result["opponent_kind"] == "checkpoint"
    assert result["opponent"]["checkpoint"] == b.name and result["opponent"]["folder"] == b.parent.name


# --------------------------------------------------------------------------
# Criterion 10: the configs of the runs
# --------------------------------------------------------------------------


def flat(path: Path) -> dict:
    out = {}

    def walk(table, prefix):
        for key, value in table.items():
            name = f"{prefix}{key}"
            if isinstance(value, dict):
                walk(value, name + ".")
            else:
                out[name] = value

    walk(tomllib.loads(path.read_text(encoding="utf-8")), "")
    return out


@pytest.mark.parametrize("network", ["mlp", "transformer"])
def test_c10_the_flow_b_config_differs_from_the_flow_a_config_only_in_the_name_and_the_target_of_announcements(network):
    a, b = flat(CONFIGS / f"smoke-a-{network}.toml"), flat(CONFIGS / f"smoke-b-{network}.toml")
    assert set(a) == set(b)
    different = {key for key in a if a[key] != b[key]}
    # The memory limit is a setting of the machine, not of the run (D150): the transformer on the
    # 6 GB card needs it (spec 015). It is the one further difference, and only for the transformer.
    # Criterion 10 names only the name and the target; the limit is the one exception, because
    # criterion 13 runs the transformer under spec 015's limit and D155 builds on it (its step c
    # raises it to 5,300). The Results of spec 018 name this exception under criterion 10.
    allowed = {"run.name", "entropy_control.targets.announcements"}
    if network == "transformer":
        allowed.add("run.gpu_memory_limit_mib")
    # The target may come out equal to the placeholder (it does: see the next test); the rest must differ.
    assert different <= allowed and different >= allowed - {"entropy_control.targets.announcements"}
    assert b["run.name"] == "smoke-b"
    assert b["run.gpu_memory_limit_mib"] == (5000 if network == "transformer" else 0)
    assert b["run.seed"] == 8 and b["run.updates"] == 200


def test_c09_the_rule_gives_the_six_targets_of_d136_from_their_measured_values():
    from tarok_train.flowb_report import target_of

    measured = {"bidding": 0.87, "contract choice": 0.11, "king call": 1.39, "talon pick": 1.74, "discard": 2.43, "play": 0.95}
    targets = flat(CONFIGS / "smoke-a-mlp.toml")
    for name, entropy in measured.items():
        assert target_of(entropy) == targets[f"entropy_control.targets.{name}"], name


def test_c09_c10_both_flow_b_configs_have_the_target_of_announcements_that_the_rule_gives():
    from tarok_train.flowb_report import ROUND_RANDOM_ENTROPY, target_of

    for network in ("mlp", "transformer"):
        cfg = flat(CONFIGS / f"smoke-b-{network}.toml")
        assert cfg["entropy_control.targets.announcements"] == target_of(ROUND_RANDOM_ENTROPY), network
    assert 0.0 < ROUND_RANDOM_ENTROPY < float(np.log(12))  # at most 12 slots are legal in the round
    # The number is in the configs' comments (criterion 8).
    for network in ("mlp", "transformer"):
        text = (CONFIGS / f"smoke-b-{network}.toml").read_text(encoding="utf-8")
        assert f"{ROUND_RANDOM_ENTROPY:.2f}" in text, network


@pytest.mark.parametrize("network", ["mlp", "transformer"])
def test_c10_the_flow_b_configs_load(network):
    cfg = train.load_config(CONFIGS / f"smoke-b-{network}.toml")
    assert cfg["run"]["network"] == network and cfg["env"]["kind"] == "engine"


# --------------------------------------------------------------------------
# Criteria 14, 15, 17 and 19: the report script reads the run folders
# --------------------------------------------------------------------------

STAMP = "20260101-000000"


def valat_share(update):
    """Under 50% from update 40 (a bounce above it at 35), under 20% from 60, under 5% from 120."""
    if update < 30:
        return 0.95
    if update < 35:
        return 0.40
    if update < 40:
        return 0.55
    if update < 60:
        return 0.30
    if update < 120:
        return 0.10
    return 0.04


def fake_flow_b_run(folder: Path, updates: int = 200, size=lambda update: 1.5, valat=valat_share, network="mlp") -> Path:
    """A run folder with the tags the report reads, with values a test can check."""
    from torch.utils.tensorboard import SummaryWriter

    cfg = train.load_config(CONFIGS / f"smoke-b-{network}.toml")
    folder.mkdir(parents=True)
    (folder / "run-info.json").write_text(json.dumps({"run": f"smoke-b-{network}", "config": cfg, "engine_sources": "x"}), encoding="utf-8")
    (folder / "summary.json").write_text(
        json.dumps({"updates": updates, "updates_wanted": 200, "finished": updates == 200, "failed": None, "stopped_by_time_limit": False, "illegal_actions": 0}),
        encoding="utf-8",
    )
    writer = SummaryWriter(str(folder))
    for update in range(1, updates + 1):
        writer.add_scalar("loss/total", 0.2, update)
        writer.add_scalar("round/announced_valat", valat(update), update)
        for name in BONUSES[:-1]:
            writer.add_scalar(f"round/announced_{name}", 0.1, update)
        for index, name in enumerate(LEVELS):
            writer.add_scalar(f"round/game_{name}", 0.6 if index == 0 else 0.1, update)
        writer.add_scalar("round/bonus_kontra", 0.2, update)
        writer.add_scalar("reward/abs_mean", size(update), update)
        writer.add_scalar("reward/abs_max", 10 * size(update), update)
        writer.add_scalar("reward/share_above_10", 0.01, update)
        writer.add_scalar("round/decision_share", 0.2, update)
        writer.add_scalar("round/decisions_per_seat_deal", 5.0, update)
        for name in C.PHASES[:-1]:
            writer.add_scalar(f"entropy/{name}", 0.5, update)
            writer.add_scalar(f"entropy_weight/{name}", 0.01, update)
        writer.add_scalar("deals/contract_ena", 0.7, update)
        writer.add_scalar("deals/contract_dve", 0.3, update)
        writer.add_scalar("deals/declarer_wins", 0.55, update)
        writer.add_scalar("time/seconds", 3.0, update)
        writer.add_scalar("env/illegal_actions", 0.0, update)
        writer.add_scalar("reward/deal_sum_abs_max", 2e-6, update)
        for tag in train.MEMORY_TAGS:
            writer.add_scalar(tag, 1000.0, update)
    writer.close()
    return folder


def test_c14_the_report_gives_the_tags_at_the_eight_updates(tmp_path):
    from tarok_train import flowb_report

    run = flowb_report.read(fake_flow_b_run(tmp_path / f"smoke-b-mlp-{STAMP}"))
    assert sorted(run["at"]) == [1, 5, 10, 25, 50, 100, 150, 200]
    row = run["at"][50]
    assert row["round/announced_valat"] == pytest.approx(0.30)
    assert row["reward/abs_mean"] == pytest.approx(1.5) and row["time/seconds"] == pytest.approx(3.0)
    for name in C.PHASES[:-1]:
        assert row[f"entropy/{name}"] == pytest.approx(0.5) and row[f"entropy_weight/{name}"] == pytest.approx(0.01)
    assert row["deals/contract_ena"] == pytest.approx(0.7) and row["deals/declarer_wins"] == pytest.approx(0.55)
    assert all(row[tag] == 1000.0 for tag in train.MEMORY_TAGS)


def test_c15_the_first_update_from_which_the_valat_share_stays_under_each_level(tmp_path):
    from tarok_train import flowb_report

    run = flowb_report.read(fake_flow_b_run(tmp_path / f"smoke-b-mlp-{STAMP}"))
    # Under 50% first at 30, but above it again at 35 to 39: it stays under from 40.
    assert run["valat_stays_under"] == {0.5: 40, 0.2: 60, 0.05: 120}


def test_c15_a_level_never_reached_for_good_is_none(tmp_path):
    from tarok_train import flowb_report

    folder = fake_flow_b_run(tmp_path / f"smoke-b-mlp-{STAMP}", valat=lambda update: 0.04 if update != 200 else 0.06)
    assert flowb_report.read(folder)["valat_stays_under"][0.05] is None


@pytest.mark.parametrize(
    "valat, size, size_mean, holds",
    [
        (valat_share, lambda update: 1.5, 1.5, True),
        (valat_share, lambda update: 2.5 if update > 180 else 1.5, 1.9, True),
        (valat_share, lambda update: 2.5 if update > 175 else 1.5, 2.0, False),  # 2.0 exactly is not under 2
        (valat_share, lambda update: 9.0 if update <= 150 else 1.5, 1.5, True),  # only 151 to 200 count
        (lambda update: 0.06, lambda update: 1.5, 1.5, False),
        (lambda update: 0.04 if update <= 150 else 0.051, lambda update: 1.5, 1.5, False),
        (lambda update: 0.9 if update <= 150 else 0.04, lambda update: 1.5, 1.5, True),  # only 151 to 200 count
    ],
)
def test_c19_the_flood_ends_by_itself_by_the_rule_fixed_before_the_runs(tmp_path, valat, size, size_mean, holds):
    from tarok_train import flowb_report

    run = flowb_report.read(fake_flow_b_run(tmp_path / f"smoke-b-mlp-{STAMP}", size=size, valat=valat))
    assert run["reading"]["size_mean"] == pytest.approx(size_mean)
    assert run["reading"]["valat_mean"] == pytest.approx(np.mean([valat(update) for update in range(151, 201)]))
    assert run["reading"]["holds"] is holds


def test_c19_a_run_that_did_not_reach_update_200_has_no_reading(tmp_path):
    from tarok_train import flowb_report

    run = flowb_report.read(fake_flow_b_run(tmp_path / f"smoke-b-mlp-{STAMP}", updates=160))
    assert run["reading"]["holds"] is None and "200" in run["reading"]["why"]


def test_c12_the_report_gives_the_loop_checks(tmp_path):
    from tarok_train import flowb_report

    run = flowb_report.read(fake_flow_b_run(tmp_path / f"smoke-b-mlp-{STAMP}"))
    assert run["checks"]["illegal_actions"] == 0.0
    assert run["checks"]["deal_sum_abs_max"] == pytest.approx(2e-6)
    # D147: within 1e-6 times the largest reward; the largest reward here is 15.
    assert run["checks"]["deal_sum_within_tolerance"] is True
    assert run["checks"]["finished"] is True


def evaluation_file(folder: Path, name: str, values, seed: int, deals: int = None):
    values = list(values)
    from tarok_train.evaluate import interval

    mean, low, high = interval(values)
    (folder / name).write_text(
        json.dumps({"deals": deals or len(values), "seed": seed, "tables": 500, "values": values, "mean": mean, "low": low, "high": high, "engine_version": "0.3.0", "opponent": {"folder": folder.name}}),
        encoding="utf-8",
    )


def test_c16_c18_the_report_reads_the_evaluations_and_the_deals_needed_for_a_difference_of_0_1(tmp_path):
    from tarok_train import flowb_report

    folder = fake_flow_b_run(tmp_path / f"smoke-b-mlp-{STAMP}")
    rng = np.random.default_rng(0)
    evaluation_file(folder, "evaluation-000200-vs-silent-random.json", rng.normal(0.3, 1.0, 400), 2008)
    evaluation_file(folder, "evaluation-000200.json", rng.normal(0.1, 2.0, 400), 2008)
    evaluation_file(folder, "evaluation-untrained-vs-silent-random.json", rng.normal(-0.2, 1.0, 400), 2008)
    run = flowb_report.read(folder)
    silent = run["evaluations"]["200 against silent-random"]
    values = json.loads((folder / "evaluation-000200-vs-silent-random.json").read_text())["values"]
    assert silent["mean"] == pytest.approx(np.mean(values))
    # 1.96 standard errors equal to 0.1: n = (1.96 * sd / 0.1) ** 2 deals.
    assert silent["deals_for_0.1"] == int(np.ceil((1.96 * np.std(values, ddof=1) / 0.1) ** 2))
    assert set(run["evaluations"]) == {"200 against silent-random", "200 against random", "untrained against silent-random"}


def test_c17_progress_inside_a_run_is_paired_over_the_deals(tmp_path):
    from tarok_train import flowb_report, trial_compare

    folder = fake_flow_b_run(tmp_path / f"smoke-b-mlp-{STAMP}")
    rng = np.random.default_rng(1)
    late = rng.normal(0.2, 1.0, 300)
    early = -0.5 * late + rng.normal(0.0, 0.5, 300)
    evaluation_file(folder, f"evaluation-000200-vs-{folder.name}-000100.json", late, 5151)
    evaluation_file(folder, f"evaluation-000100-vs-{folder.name}-000200.json", early, 5151)
    run = flowb_report.read(folder)
    progress = run["progress"]
    want = trial_compare.paired(
        json.loads((folder / f"evaluation-000200-vs-{folder.name}-000100.json").read_text()),
        json.loads((folder / f"evaluation-000100-vs-{folder.name}-000200.json").read_text()),
    )
    assert progress["half_difference"] == pytest.approx(want["half_difference"])
    assert progress["one"]["mean"] == pytest.approx(late.mean()) and progress["other"]["mean"] == pytest.approx(early.mean())
    # The old rule of D153: both intervals on one side of zero, one above and one below.
    assert progress["side"] == trial_compare.side(progress["one"], progress["other"])


def test_c14_c19_the_markdown_report_names_the_reading(tmp_path):
    from tarok_train import flowb_report

    run = flowb_report.read(fake_flow_b_run(tmp_path / f"smoke-b-mlp-{STAMP}"))
    text = flowb_report.report([run])
    assert "smoke-b-mlp" in text and "the flood ends by itself" in text
    assert "| 200 |" in text


# --------------------------------------------------------------------------
# D155 (Batch 22): the transformer's acting cache is freed before the update step
# --------------------------------------------------------------------------


def test_d155_freeing_the_acting_cache_before_the_update_changes_no_number():
    """Three updates of the transformer on the CPU, with and without freeing the cache: the same numbers.

    On flow B the acting cache grows to its limit and stayed allocated through the update step
    (spec 018, the first transformer run). It is now freed before the update step and grows again
    in the next acting phase. The weights that wrote it change in the update, so no number may change.
    """
    engine()

    def history(free):
        trainer = train.Trainer(ci_cfg(**{"run.network": "transformer"}), CPU)
        actor = trainer.actor
        release = actor.release if free else actor.reset  # reset alone: the cache stays allocated
        before, during = [], []

        def counted():
            before.append(actor.cache.capacity)
            release()

        def spy(*args, **kwargs):
            during.append((actor.cache.capacity, all(k is None for k in actor.cache.k + actor.cache.v)))
            return update(*args, **kwargs)

        update = ppo.update
        actor.release = counted
        train.ppo.update = spy
        try:
            out = trainer.run(3)
        finally:
            train.ppo.update = update
            trainer.close()
        numbers = [{key: value for key, value in stats.items() if not key.startswith(("time/", "speed/"))} for stats in out]
        return numbers, before, during, actor.cache.capacity

    freed, before, during, after = history(free=True)
    kept, before_kept, during_kept, _ = history(free=False)
    assert freed == kept
    assert len(freed) == 3 and len({stats["loss/total"] for stats in freed}) == 3
    initial = ci_cfg()["model"]["cache_initial_tokens"] + 2
    # Every acting phase grew the cache past its first size; every update step ran without it.
    assert len(before) == 3 and all(capacity > initial for capacity in before), before
    assert during == [(0, True)] * 3
    assert after == 0  # the run stops after an update: the cache is left freed
    # Without freeing, the update step ran with the cache at its full size.
    assert [capacity for capacity, _ in during_kept] == before_kept and all(c > initial for c in before_kept)


def test_d155_a_freed_cache_grows_again_from_its_first_size():
    cache = KVCache(8, 2, 2, 4, "cpu", 34)
    cache.ensure(70)
    assert cache.capacity == 70  # twice 34 is less than asked: the size asked
    cache.release()
    assert cache.capacity == 0 and all(k is None for k in cache.k + cache.v)
    cache.ensure(10)
    assert cache.capacity == 34  # the first size, as a new cache
    assert all(k.shape == (8, 2, 34, 4) and not k.any() for k in cache.k + cache.v)
