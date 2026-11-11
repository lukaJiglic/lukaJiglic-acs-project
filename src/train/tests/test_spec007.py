"""Tests for spec 007 (model and PPO learner), one or more per acceptance criterion.

The name of each test starts with its criterion number: test_c04_... is criterion 4.
Written from the criteria before the code; made stricter after the review pass
(the Results of the spec list which planted bug each test catches).
"""

import copy
import inspect
import math
import re
import time
from pathlib import Path

import numpy as np
import pytest
import torch

from tarok_train import contract as C
from tarok_train import mock_env, model, opponents, ppo, rollout, train

REPO = Path(__file__).resolve().parents[3]
ENCODING_MD = REPO / "docs" / "encoding.md"
MOCK_TOML = Path(__file__).resolve().parents[1] / "configs" / "mock.toml"
TOL = 1e-4
CPU = torch.device("cpu")


# --------------------------------------------------------------------------
# Helpers
# --------------------------------------------------------------------------


def cfg_with(**overrides):
    """The mock config with 'section.key' overrides, e.g. cfg_with(**{"env.tables": 8})."""
    cfg = train.load_config(MOCK_TOML)
    for dotted, value in overrides.items():
        section, key = dotted.split(".")
        assert key in cfg[section], dotted
        cfg[section][key] = value
    return cfg


SMALL = {
    "env.tables": 8,
    "ppo.seat_deals_per_update": 24,
    "ppo.update_batch": 16,
    "opponents.snapshot_every": 2,
    "run.checkpoint_every": 1000,
}


def small(**overrides):
    return cfg_with(**{**SMALL, **overrides})


def build(kind, seed=0):
    torch.manual_seed(seed)
    cfg = cfg_with()
    return model.build_network(cfg["model"], kind), model.build_critic(cfg["model"])


def random_deals(batch, n_state, n_play, seed=0):
    """Random data of the contract's shapes for `batch` seat-deals."""
    gen = torch.Generator().manual_seed(seed)
    tokens = torch.stack(
        [torch.randint(0, size, (batch, n_play), generator=gen) for size in C.TOKEN_FIELD_SIZES], dim=-1
    )
    tokens[..., 4] = torch.arange(n_play)
    points = torch.randint(0, n_play + 1, (batch, n_state), generator=gen).sort(dim=1).values
    state = torch.rand(batch, n_state, C.STATE_FEATURES, generator=gen)
    mask = torch.rand(batch, n_state, C.N_ACTIONS, generator=gen) < 0.1
    mask[..., 0] = True
    return tokens, points, state, mask


def close(a, b, tol=TOL):
    finite = torch.isfinite(a)
    return torch.equal(finite, torch.isfinite(b)) and torch.max(torch.abs(a[finite] - b[finite])).item() < tol


def collected(trainer):
    """One collection of the trainer, as a prepared batch of finished seat-deals (no update)."""
    trainer.collect()
    deals = trainer.rollout.take()
    ppo.prepare(deals, trainer.cfg["ppo"])
    return deals


def tables_of(text):
    """All Markdown tables of a document as lists of rows of cells."""
    tables, rows = [], []
    for line in text.splitlines():
        if line.startswith("|"):
            cells = [cell.strip() for cell in line.strip().strip("|").split("|")]
            if not all(set(cell) <= set("-: ") for cell in cells):
                rows.append(cells)
        elif rows:
            tables.append(rows)
            rows = []
    if rows:
        tables.append(rows)
    return tables


def table_with(tables, *header):
    for table in tables:
        if [cell.lower() for cell in table[0][: len(header)]] == [h.lower() for h in header]:
            return table[1:]
    raise AssertionError(f"no table with the header {header} in docs/encoding.md")


def span(text):
    """'2–14' -> (2, 13); '88' -> (88, 1)."""
    numbers = [int(n) for n in re.findall(r"\d+", text)]
    return (numbers[0], numbers[-1] - numbers[0] + 1)


def add_rows(store, tables, seat, deal, value=None, n_tokens=None, action=None):
    n = len(tables)
    zeros = [0.0] * C.N_SCORE_PARTS
    store.add(
        tokens=np.zeros((n, C.MAX_TOKENS, 5), dtype=np.int16),
        tables=np.array(tables),
        seat=np.array(seat),
        deal=np.array(deal),
        n_tokens=np.array(n_tokens if n_tokens is not None else [1] * n),
        state=np.zeros((n, C.STATE_FEATURES), dtype=np.float32),
        mask=np.ones((n, C.N_ACTIONS), dtype=bool),
        critic_extra=np.zeros((n, C.CRITIC_EXTRA), dtype=np.float32),
        partner_target=np.zeros(n, dtype=np.int64),
        card_target=np.zeros((n, C.N_CARDS), dtype=np.int64),
        action=np.array(action if action is not None else [0] * n),
        logp=np.zeros(n, dtype=np.float32),
        value=np.array(value if value is not None else [zeros] * n, dtype=np.float32),
    )


def first_part(number):
    """A value or reward that sits in the first score part only."""
    return [number] + [0.0] * (C.N_SCORE_PARTS - 1)


def reward_for(seat, rewards):
    """(n, 4) rewards: the given number for `seat`, a different one for every other seat."""
    out = np.full((len(rewards), 4), 1000.0, dtype=np.float32)
    out += np.arange(4, dtype=np.float32)[None, :]
    out[:, seat] = rewards
    return out


def parts_for(seat, rewards):
    out = np.zeros((len(rewards), 4, C.N_SCORE_PARTS), dtype=np.float32)
    out[:, :, 0] = reward_for(seat, rewards)
    return out


def scalar_tags(log_dir):
    from tensorboard.backend.event_processing.event_accumulator import EventAccumulator

    accumulator = EventAccumulator(str(log_dir))
    accumulator.Reload()
    return set(accumulator.Tags()["scalars"])


# --------------------------------------------------------------------------
# Contract
# --------------------------------------------------------------------------


def test_c01_contract_sizes_match_the_tables_of_encoding_md():
    text = ENCODING_MD.read_text(encoding="utf-8")
    tables = tables_of(text)

    version = re.search(r"Encoding version: \*\*(\d+)\*\*", text)
    assert version and int(version.group(1)) == C.ENCODING_VERSION

    # Section 2: the arrays of one decision.
    arrays = {row[0].strip("`"): row[1] for row in table_with(tables, "Array", "Shape")}
    assert [int(n) for n in re.findall(r"\d+", arrays["tokens"])] == [C.MAX_TOKENS, len(C.TOKEN_FIELDS)]
    assert int(arrays["state"]) == C.STATE_FEATURES
    assert int(arrays["mask"]) == C.N_ACTIONS
    assert [int(n) for n in re.findall(r"\d+", arrays["critic"])] == [C.STATE_FEATURES, C.CRITIC_EXTRA]

    # Section 3: token fields and the event vocabulary.
    fields = table_with(tables, "Field", "Values")
    assert tuple(row[0] for row in fields) == C.TOKEN_FIELDS
    sizes = tuple(max(int(n) for n in re.findall(r"\d+", row[1])) + 1 for row in fields)
    assert sizes == C.TOKEN_FIELD_SIZES
    events = {}
    for ident, count, name, _ in table_with(tables, "Id", "Count", "Name"):
        key = re.sub(r"_(x|suit|card|group|target)$", "", name.strip("`*").strip("`"))
        events[key if key != "reserved" else "RESERVED"] = (span(ident)[0], int(count))
        assert span(ident)[1] == int(count), name
    assert events == C.EVENT_GROUPS
    assert sum(count for _, count in C.EVENT_GROUPS.values()) == C.N_EVENTS == 288

    # Section 4: every state block with an offset, and the total.
    blocks = {}
    total = None
    for name, width, offset, _ in table_with(tables, "Block", "Width", "Offset"):
        head = re.match(r"\**([A-G]\d+)\b", name)
        if head:
            blocks[head.group(1)] = (span(offset)[0], int(width))
            assert span(offset)[1] == int(width), name
        elif "Total" in name:
            total = int(width.strip("*"))
    assert blocks == C.STATE_BLOCKS
    assert total == C.STATE_FEATURES == 1382
    ordered = sorted(C.STATE_BLOCKS.values())
    assert ordered[0][0] == 0 and ordered[-1][0] + ordered[-1][1] == C.STATE_FEATURES
    assert all(a[0] + a[1] == b[0] for a, b in zip(ordered, ordered[1:]))  # no gap, no overlap

    # Section 5: action slots.
    actions = {}
    for slot, count, name, _ in table_with(tables, "Slot", "Count", "Name"):
        key = re.sub(r"_(x|suit|card|group|target)$", "", name.strip("`*").strip("`"))
        actions[key if key != "reserved" else "RESERVED"] = (span(slot)[0], int(count))
        assert span(slot)[1] == int(count), name
    assert actions == C.ACTION_GROUPS
    assert sum(count for _, count in C.ACTION_GROUPS.values()) == C.N_ACTIONS == 160

    # Section 6: the critic's extra features.
    extra = [int(row[0]) for row in table_with(tables, "Width", "Content")]
    assert extra == list(C.CRITIC_EXTRA_BLOCKS.values())
    assert sum(extra) == C.CRITIC_EXTRA == 274
    assert C.CRITIC_FEATURES == 1382 + 274
    assert re.search(r"/ 50\.", text) and C.REWARD_SCALE == 50

    # Section 8: the score parts, their number and their order (D121).
    parts = table_with(tables, "Index", "Part")
    assert [int(row[0]) for row in parts] == list(range(len(parts)))
    assert tuple(row[1] for row in parts) == C.SCORE_PARTS
    assert C.N_SCORE_PARTS == len(parts) == 10


def test_c01_the_number_of_score_parts_is_one_constant(monkeypatch):
    """The number of score parts changed once (8 to 10, D121). Nothing may hold it as a number: run with 12."""
    monkeypatch.setattr(C, "SCORE_PARTS", C.SCORE_PARTS + ("made up 1", "made up 2"))
    monkeypatch.setattr(C, "N_SCORE_PARTS", 12)
    for kind in ("transformer", "mlp"):
        trainer = train.Trainer(small(**{"run.network": kind}), CPU)
        (stats,) = trainer.run(1)
        assert math.isfinite(stats["loss/total"])
        assert trainer.net.heads.value.out_features == 12 and trainer.critic.net[-1].out_features == 12
    assert trainer.env.part_weights.shape == (12,) and abs(float(trainer.env.part_weights.sum()) - 1.0) < 1e-6


def test_c01_the_mock_emits_ten_parts_that_add_up_to_the_reward():
    env = mock_env.make(cfg_with(**{"env.tables": 6})["env"], seed=1)
    rng = np.random.default_rng(1)
    seen = 0
    for _ in range(60):
        obs = env.observe()
        done, reward, parts = env.step(np.array([rng.choice(np.flatnonzero(row)) for row in obs["mask"]]))
        assert parts.shape == (6, 4, C.N_SCORE_PARTS) and C.N_SCORE_PARTS == 10
        assert np.abs(parts.sum(axis=-1) - reward).max() < 1e-6
        assert np.all(parts[..., C.SCORE_PARTS.index("klop")] == 0) and np.all(parts[..., C.SCORE_PARTS.index("radelc")] == 0)
        seen += int(done.sum())
    assert seen >= 6 and float(np.abs(parts).sum()) >= 0


# --------------------------------------------------------------------------
# Model
# --------------------------------------------------------------------------


@pytest.mark.parametrize("kind", ["transformer", "mlp"])
def test_c02_forward_pass_gives_the_stated_shapes_and_no_nan(kind):
    net, critic = build(kind)
    tokens, points, state, mask = random_deals(3, 5, 40)
    with torch.inference_mode():
        out = net.forward_deals(tokens, points, state, mask)
        critic_features = torch.rand(3, 5, C.CRITIC_FEATURES)
        value = critic(critic_features)
    assert out.logits.shape == (3, 5, C.N_ACTIONS)
    assert out.q.shape == (3, 5, C.N_ACTIONS)
    assert out.value.shape == (3, 5, C.N_SCORE_PARTS)
    assert out.partner.shape == (3, 5, C.PARTNER_CLASSES)
    assert out.cards.shape == (3, 5, C.N_CARDS, C.CARD_LOCATION_CLASSES)
    assert value.shape == (3, 5, C.N_SCORE_PARTS)
    assert not torch.isnan(out.logits).any() and torch.isfinite(out.logits[mask]).all()
    for part in (out.q, out.value, out.partner, out.cards, value):
        assert torch.isfinite(part).all()


@pytest.mark.parametrize("kind", ["transformer", "mlp"])
def test_c03_illegal_actions_have_probability_zero_and_a_single_legal_action_one(kind):
    net, _ = build(kind)
    tokens, points, state, mask = random_deals(2, 4, 30)
    mask[0, 1] = False
    mask[0, 1, 77] = True  # one legal action only
    with torch.inference_mode():
        out = net.forward_deals(tokens, points, state, mask)
    probs = torch.softmax(out.logits, dim=-1)
    assert torch.all(out.logits[~mask] == -math.inf)
    assert torch.all(probs[~mask] == 0.0)
    assert torch.allclose(probs.sum(-1), torch.ones(2, 4), atol=1e-6)
    assert probs[0, 1, 77].item() == 1.0


def test_c04_l2_mask_which_outputs_move_when_a_token_changes():
    net, _ = build("transformer")
    tokens, _, state, mask = random_deals(1, 4, 20, seed=4)
    points = torch.tensor([[3, 8, 8, 15]])
    n_play, k = 20, 1

    def hidden(tok, st):
        with torch.inference_mode():
            return net.trunk_deals(tok, points, st)[0]  # (n_play + n_state, width)

    def moved(a, b):
        return (a - b).abs().amax(dim=-1) > 1e-6

    base = hidden(tokens, state)

    # A state token changes: no other state token and no play token moves.
    other = state.clone()
    other[0, k] = 1.0 - other[0, k]
    delta = moved(base, hidden(tokens, other))
    assert delta[n_play + k] and delta.sum() == 1

    # A play token changes at index j: earlier play tokens stay, later ones move;
    # state tokens move exactly when their decision point lies after j.
    for j in (5, 8, 14):
        changed = tokens.clone()
        changed[0, j, 0] = (changed[0, j, 0] + 1) % C.N_EVENTS
        delta = moved(base, hidden(changed, state))
        assert not delta[:j].any(), "a play token saw a later play token"
        assert delta[j:n_play].all()
        expected = points[0] > j
        assert torch.equal(delta[n_play:], expected), f"state tokens moved wrongly for a change at {j}"

    # The same through the heads: a later play token does not move decision k's outputs.
    later = tokens.clone()
    later[0, points[0, k] :, 0] = (later[0, points[0, k] :, 0] + 7) % C.N_EVENTS
    with torch.inference_mode():
        a = net.forward_deals(tokens, points, state, mask)
        b = net.forward_deals(later, points, state, mask)
    assert torch.equal(a.q[0, k], b.q[0, k]) and not torch.equal(a.q[0, 3], b.q[0, 3])


def test_c05_one_l2_pass_equals_each_decision_alone_from_its_prefix():
    net, _ = build("transformer")
    tokens, points, state, mask = random_deals(3, 6, 50, seed=5)
    with torch.inference_mode():
        whole = net.forward_deals(tokens, points, state, mask)
        for k in range(points.shape[1]):
            alone = net.forward_prefix(tokens, points[:, k], state[:, k], mask[:, k])
            for name in whole._fields:
                assert close(getattr(alone, name), getattr(whole, name)[:, k]), f"{name} differs at decision {k}"


def test_c06_cached_acting_equals_the_full_pass_also_across_a_change_of_weights():
    """Tables at different points in one batch; in the middle the weights change and the cache is reset.

    Every decision must equal the same decision computed alone from its prefix with
    the weights of its time. Without the reset, deals that were running at the
    change would be answered from keys and values of the old weights.
    """
    cfg = cfg_with(**{"env.tables": 6, "model.cache_initial_tokens": 4, "model.act_bucket_tokens": 2})
    net, _ = build("transformer", seed=6)
    env = mock_env.make(cfg["env"], seed=6)
    actor = model.Actor(net, None, 6, CPU, cfg["model"])
    gen = torch.Generator().manual_seed(6)
    records = []
    counts = {}
    before_change = None
    for step in range(80):
        obs = env.observe()
        action, logp, out = actor.act(
            np.arange(6),
            obs["seat"],
            obs["deal"],
            obs["n_tokens"],
            np.zeros(6, dtype=bool),
            torch.from_numpy(obs["tokens"]).long(),
            torch.from_numpy(obs["state"]),
            torch.from_numpy(obs["mask"]),
            gen,
        )
        for t in range(6):
            key = (t, int(obs["seat"][t]), int(obs["deal"][t]))
            counts[key] = counts.get(key, 0) + 1
            records.append(
                {
                    "phase": 0 if before_change is None else 1,
                    "key": key,
                    "tokens": torch.from_numpy(obs["tokens"][t]).long(),
                    "n": int(obs["n_tokens"][t]),
                    "state": torch.from_numpy(obs["state"][t]),
                    "mask": torch.from_numpy(obs["mask"][t]),
                    "out": [getattr(out, name)[t].clone() for name in ("logits", "q", "value")],
                }
            )
        env.step(action.numpy())
        if step == 35:
            before_change = copy.deepcopy(net)
            with torch.no_grad():
                for weight in net.parameters():
                    weight.add_(0.05 * torch.randn(weight.shape, generator=gen))
            actor.reset()

    # 1. Each decision alone from its prefix, with the weights of its time.
    for phase, reference in ((0, before_change), (1, net)):
        rows = [r for r in records if r["phase"] == phase]
        with torch.inference_mode():
            alone = reference.forward_prefix(
                torch.stack([r["tokens"] for r in rows]),
                torch.tensor([r["n"] for r in rows]),
                torch.stack([r["state"] for r in rows]),
                torch.stack([r["mask"] for r in rows]),
            )
        for i, r in enumerate(rows):
            for name, got in zip(("logits", "q", "value"), r["out"]):
                assert close(got, getattr(alone, name)[i]), f"{name} of {r['key']} differs in phase {phase}"
        assert len(rows) > 100

    # 2. One L2 pass per seat-deal, for the seat-deals that lie wholly after the change.
    groups = {}
    for r in records:
        groups.setdefault(r["key"], []).append(r)
    whole = [rows for rows in groups.values() if all(r["phase"] == 1 for r in rows) and len(rows) >= 2]
    longest = max(len(rows) for rows in whole)
    points = torch.zeros(len(whole), longest, dtype=torch.long)
    state = torch.zeros(len(whole), longest, C.STATE_FEATURES)
    mask = torch.ones(len(whole), longest, C.N_ACTIONS, dtype=torch.bool)
    for g, rows in enumerate(whole):
        for k, r in enumerate(rows):
            points[g, k], state[g, k], mask[g, k] = r["n"], r["state"], r["mask"]
    with torch.inference_mode():
        full = net.forward_deals(torch.stack([rows[-1]["tokens"] for rows in whole]), points, state, mask)
    for g, rows in enumerate(whole):
        for k, r in enumerate(rows):
            for name, got in zip(("logits", "q", "value"), r["out"]):
                assert close(got, getattr(full, name)[g, k])
    assert len(whole) >= 8
    assert len({r["n"] for r in records}) > 10, "the tables were not at different points"
    assert actor.cache.capacity > 6, "the cache did not grow"


@pytest.mark.parametrize("kind", ["transformer", "mlp"])
def test_c06_acting_and_update_see_byte_identical_actor_inputs(kind):
    """What the actor was given when it chose an action is what the update gives the network."""
    trainer = train.Trainer(small(**{"run.network": kind}), CPU)
    given = {}
    counts = {}
    real_act = trainer.actor.act

    def recording(tables, seat, deal, n_tokens, frozen_rows, tokens, state, mask, generator):
        for i, table in enumerate(tables):
            key = (int(table), int(seat[i]), int(deal[i]))
            counts[key] = counts.get(key, 0) + 1
            n = int(n_tokens[i])
            given[(*key, counts[key] - 1)] = (n, tokens[i, :n].numpy().copy(), state[i].numpy().copy(), mask[i].numpy().copy())
        return real_act(tables, seat, deal, n_tokens, frozen_rows, tokens, state, mask, generator)

    trainer.actor.act = recording
    deals = collected(trainer)
    batch = deals.to_torch(CPU)
    checked = 0
    for g in range(len(deals)):
        for k in range(int(deals.valid[g].sum())):
            n, tokens, state, mask = given[(int(deals.table[g]), int(deals.seat[g]), int(deals.deal[g]), k)]
            assert int(batch.points[g, k]) == n
            assert np.array_equal(batch.tokens[g, :n].numpy(), tokens)  # the earlier sequence is a prefix
            assert batch.state[g, k].numpy().tobytes() == state.tobytes()
            assert np.array_equal(batch.mask[g, k].numpy(), mask)
            checked += 1
    assert checked >= 50


@pytest.mark.parametrize("kind", ["transformer", "mlp"])
def test_c07_the_actor_does_not_depend_on_the_critics_hidden_features(kind):
    """Through the real training path: change the critic-only features of a collected batch.

    The actor's outputs and the actor's gradients of the total loss must not move
    by a bit. The critic's value and gradients must move.
    """
    trainer = train.Trainer(small(**{"run.network": kind}), CPU)
    deals = collected(trainer)

    def run(batch):
        out, value = ppo.evaluate(trainer.net, trainer.critic, batch)
        total = ppo.compute_losses(out, value, batch, trainer.cfg["ppo"])["total"]
        actor = torch.autograd.grad(total, list(trainer.net.parameters()), retain_graph=True, allow_unused=True)
        critic = torch.autograd.grad(total, list(trainer.critic.parameters()), allow_unused=True)
        return out, value, actor, critic

    out, value, actor_grads, critic_grads = run(deals.to_torch(CPU))
    deals.critic_extra = 1.0 - deals.critic_extra
    out2, value2, actor_grads2, critic_grads2 = run(deals.to_torch(CPU))
    for name in out._fields:
        assert torch.equal(getattr(out, name), getattr(out2, name)), f"actor output {name} moved"
    assert all(a is not None and torch.equal(a, b) for a, b in zip(actor_grads, actor_grads2)), "actor gradient moved"
    assert not torch.equal(value, value2), "the critic ignores its hidden features"
    assert any(not torch.equal(a, b) for a, b in zip(critic_grads, critic_grads2))

    # No path in the graph either.
    batch = deals.to_torch(CPU)
    batch.critic_extra.requires_grad_(True)
    out, value = ppo.evaluate(trainer.net, trainer.critic, batch)
    actor_sum = sum(getattr(out, name)[torch.isfinite(getattr(out, name))].sum() for name in out._fields)
    (grad,) = torch.autograd.grad(actor_sum, batch.critic_extra, allow_unused=True, retain_graph=True)
    assert grad is None
    (grad,) = torch.autograd.grad(value.sum(), batch.critic_extra)
    assert grad.abs().sum() > 0


def test_c08_parameter_counts_are_in_range_printed_and_logged(tmp_path, capsys):
    transformer, critic = build("transformer")
    mlp, _ = build("mlp")
    assert 3_000_000 < model.count_parameters(transformer) < 4_500_000
    assert model.count_parameters(mlp) < 1_500_000
    trainer = train.Trainer(small(**{"run.network": "mlp"}), CPU, log_dir=tmp_path)
    printed = capsys.readouterr().out
    counts = trainer.parameter_counts
    assert counts["mlp"] == model.count_parameters(mlp) and counts["critic"] == model.count_parameters(critic)
    assert f"{counts['mlp']:,}" in printed and "parameters" in printed
    trainer.close()
    assert "parameters/mlp" in scalar_tags(tmp_path)


# --------------------------------------------------------------------------
# PPO
# --------------------------------------------------------------------------


def test_c09_gae_matches_a_hand_computed_example_of_two_deals():
    # Deal A has 3 decisions and reward 1.0, deal B has 2 decisions and reward -2.0.
    # gamma 0.9, lambda 0.8. The slot after deal B's end holds a value that must not be read.
    value = torch.tensor([[[0.5], [0.2], [-0.1]], [[0.3], [0.4], [99.0]]])
    valid = torch.tensor([[True, True, True], [True, True, False]])
    reward = torch.tensor([[1.0], [-2.0]])
    adv, ret = ppo.gae(value, valid, reward, gamma=0.9, lam=0.8)
    # A: d2 = 1 + 0.1 = 1.1; d1 = 0.9*(-0.1) - 0.2 = -0.29; d0 = 0.9*0.2 - 0.5 = -0.32
    #    A2 = 1.1; A1 = -0.29 + 0.72*1.1 = 0.502; A0 = -0.32 + 0.72*0.502 = 0.04144
    # B: d1 = -2 - 0.4 = -2.4; d0 = 0.9*0.4 - 0.3 = 0.06; A1 = -2.4; A0 = 0.06 + 0.72*(-2.4) = -1.668
    want_adv = torch.tensor([[0.04144, 0.502, 1.1], [-1.668, -2.4, 0.0]])
    want_ret = torch.tensor([[0.54144, 0.702, 1.0], [-1.368, -2.0, 0.0]])
    assert torch.allclose(adv[..., 0], want_adv, atol=1e-6)
    assert torch.allclose(ret[..., 0], want_ret, atol=1e-6)

    # Nothing leaks across the end of a deal: deal B's numbers do not depend on deal A.
    changed_value = value.clone()
    changed_value[0] = torch.tensor([[7.0], [-3.0], [2.0]])
    adv2, _ = ppo.gae(changed_value, valid, torch.tensor([[50.0], [-2.0]]), gamma=0.9, lam=0.8)
    assert torch.equal(adv2[1], adv[1])

    # The same two deals through the rollout storage, their steps interleaved in one buffer,
    # and through `prepare`, which the update uses: each advantage lands at its own decision.
    store = rollout.Rollout()
    values = {0: [0.5, 0.2, -0.1], 1: [0.3, 0.4]}
    for k in range(3):
        for table in (0, 1):
            if k < len(values[table]):
                add_rows(store, tables=[table], seat=[2], deal=[5], value=[first_part(values[table][k])])
    store.finish(np.array([0, 1]), np.array([5, 5]), reward_for(2, [1.0, -2.0]), parts_for(2, [1.0, -2.0]))
    deals = store.take()
    order = np.argsort(deals.table)
    ppo.prepare(deals, {"gamma": 0.9, "gae_lambda": 0.8})
    assert np.allclose(deals.advantage[order], want_adv.numpy(), atol=1e-6)
    assert np.allclose(deals.lambda_return[order][..., 0], want_ret.numpy(), atol=1e-6)
    assert np.all(deals.lambda_return[..., 1:] == 0)
    # The realised return: the reward discounted by the steps to the deal's end.
    want_realised = np.array([[0.81, 0.9, 1.0], [-1.8, -2.0, 0.0]])
    assert np.allclose(deals.realised[order][..., 0], want_realised, atol=1e-6)


def test_c10_the_reward_reaches_every_decision_of_that_seat_and_deal_only():
    env = mock_env.make(cfg_with(**{"env.tables": 5})["env"], seed=10)
    store = rollout.Rollout()
    rng = np.random.default_rng(10)
    finished = {}  # (table, deal) -> rewards of the four seats
    decisions = {}  # (table, seat, deal) -> number of decisions
    for _ in range(120):
        obs = env.observe()
        action = np.array([rng.choice(np.flatnonzero(row)) for row in obs["mask"]])
        for t in range(5):
            key = (t, int(obs["seat"][t]), int(obs["deal"][t]))
            decisions[key] = decisions.get(key, 0) + 1
        add_rows(store, np.arange(5), obs["seat"], obs["deal"], n_tokens=obs["n_tokens"], action=action)
        deal = obs["deal"].copy()
        done, reward, parts = env.step(action)
        for t in np.flatnonzero(done):
            finished[(int(t), int(deal[t]))] = reward[t].copy()
        if done.any():
            store.finish(np.flatnonzero(done), deal[done], reward[done], parts[done])
    deals = store.take()
    assert len(finished) >= 10
    assert len(deals.reward) == 4 * len(finished)  # every seat of every finished deal, nothing else
    assert len({float(r) for r in deals.reward}) > 3, "the rewards do not differ: the test would prove nothing"
    returns = ppo.realised_return(torch.from_numpy(deals.valid), torch.from_numpy(deals.parts), gamma=1.0).sum(-1)
    for g in range(len(deals.reward)):
        key = (int(deals.table[g]), int(deals.seat[g]), int(deals.deal[g]))
        own = finished[key[0], key[2]][key[1]]
        n = decisions[key]
        assert int(deals.valid[g].sum()) == n
        assert deals.reward[g] == pytest.approx(own)
        assert torch.allclose(returns[g, :n], torch.full((n,), float(own)), atol=1e-6)  # every decision
        assert torch.all(returns[g, n:] == 0)
        assert np.allclose(deals.parts[g].sum(), own, atol=1e-5)
    # A deal that is not finished stays in the storage and is not in the batch.
    assert store.pending_rows > 0
    unfinished = {(t, d) for (t, s, d) in decisions} - set(finished)
    assert unfinished and not any((int(t), int(d)) in unfinished for t, d in zip(deals.table, deals.deal))
    # `discard` drops them: the training loop never trains on a deal that outlived an update.
    store.discard()
    assert store.pending_rows == 0 and store.take() is None


def test_c10_score_parts_that_do_not_add_up_to_the_reward_are_refused():
    store = rollout.Rollout()
    add_rows(store, tables=[0], seat=[1], deal=[0])
    parts = parts_for(1, [0.5])
    parts[0, 1, 3] += 0.01  # seat 1: the parts now add up to 0.51, the reward is 0.5
    with pytest.raises(ValueError, match="do not add up"):
        store.finish(np.array([0]), np.array([0]), reward_for(1, [0.5]), parts)
    store.finish(np.array([0]), np.array([0]), reward_for(1, [0.5]), parts_for(1, [0.5]))
    assert store.take().reward[0] == pytest.approx(0.5)


def test_c11_clipped_policy_loss_equals_a_hand_computed_value():
    logp_old = torch.log(torch.tensor([0.5, 0.25, 0.2]))
    logp_new = torch.log(torch.tensor([0.75, 0.5, 0.1]))  # ratios 1.5, 2.0, 0.5
    advantage = torch.tensor([1.0, -2.0, 0.5])
    # step 1: min(1.5, clip 1.2) * 1.0 = 1.2
    # step 2: min(2.0 * -2, 1.2 * -2) = -4.0
    # step 3: min(0.5 * 0.5, 0.8 * 0.5) = 0.25
    # loss = -(1.2 - 4.0 + 0.25) / 3 = 0.85
    loss = ppo.clipped_policy_loss(logp_new, logp_old, advantage, clip=0.2)
    assert loss.item() == pytest.approx(0.85, abs=1e-6)


@pytest.mark.parametrize("kind", ["transformer", "mlp"])
def test_c12_no_gradient_reaches_an_illegal_actions_logit_in_the_update(kind):
    """On the path the update takes: evaluate, compute_losses, backward. The raw logits are watched."""
    trainer = train.Trainer(small(**{"run.network": kind}), CPU)
    deals = collected(trainer)
    batch = deals.to_torch(CPU)
    raw = {}

    def keep(module, inputs, output):
        output.retain_grad()
        raw["logits"] = output

    hook = trainer.net.heads.policy.register_forward_hook(keep)
    out, value = ppo.evaluate(trainer.net, trainer.critic, batch)
    hook.remove()
    ppo.compute_losses(out, value, batch, trainer.cfg["ppo"])["total"].backward()
    grad = raw["logits"].grad
    legal = batch.mask & batch.valid[..., None]
    assert torch.isfinite(grad).all()
    assert torch.all(grad[~batch.mask] == 0.0), "an illegal action's logit got a gradient"
    assert grad[legal].abs().sum() > 0
    assert torch.all(out.logits[~batch.mask] == -math.inf)  # and probability zero in the loss


@pytest.mark.parametrize("kind", ["transformer", "mlp"])
def test_c13_every_head_and_the_critic_get_a_gradient_from_one_update(kind):
    trainer = train.Trainer(small(**{"run.network": kind}), CPU)
    batch = collected(trainer).to_torch(CPU)
    out, value = ppo.evaluate(trainer.net, trainer.critic, batch)
    losses = ppo.compute_losses(out, value, batch, trainer.cfg["ppo"])
    assert set(losses) >= {"total", "policy", "critic", "value_head", "q", "partner", "cards", "entropy"}
    losses["total"].backward()
    heads = trainer.net.heads
    for name in ("policy", "q", "value", "partner", "cards"):
        grad = getattr(heads, name).weight.grad
        assert grad is not None and grad.abs().sum() > 0, f"no gradient at the {name} head"
    for name, weight in trainer.critic.named_parameters():
        assert weight.grad is not None and weight.grad.abs().sum() > 0, f"no gradient at the critic's {name}"
    assert all(w.grad is not None and w.grad.abs().sum() > 0 for w in ppo.trunk_parameters(trainer.net))


def test_c13_the_critic_reads_the_environments_critic_array_of_the_same_decision():
    """The critic's input in the update is, row for row, the `critic` array the environment gave."""
    trainer = train.Trainer(small(**{"run.network": "mlp", "opponents.past_share": 0.0}), CPU)
    given = {}
    counts = {}
    for _ in range(40):
        obs = trainer.env.observe()
        for t in range(trainer.n):
            key = (t, int(obs["seat"][t]), int(obs["deal"][t]))
            counts[key] = counts.get(key, 0) + 1
            given[(*key, counts[key] - 1)] = (obs["critic"][t].copy(), int(obs["partner_target"][t]))
        trainer.step()
    deals = trainer.rollout.take()
    batch = deals.to_torch(CPU)
    features = torch.cat([batch.state, batch.critic_extra], dim=-1)
    assert features.shape[-1] == C.CRITIC_FEATURES
    for g in range(len(deals)):
        for k in range(int(deals.valid[g].sum())):
            critic, partner = given[(int(deals.table[g]), int(deals.seat[g]), int(deals.deal[g]), k)]
            assert np.array_equal(features[g, k].numpy(), critic)
            assert int(batch.partner_target[g, k]) == partner


def test_c13_the_value_losses_are_huber_or_squared_as_the_config_says():
    prediction = torch.tensor([0.0, 0.5, 3.0, -4.0])
    target = torch.zeros(4)
    huber = ppo.value_error(prediction, target, form="huber", delta=1.0)
    assert torch.allclose(huber, torch.tensor([0.0, 0.125, 2.5, 3.5]))  # 0.5 d^2 up to 1, then |d| - 0.5
    assert torch.allclose(ppo.value_error(prediction, target, form="huber", delta=2.0)[2:], torch.tensor([4.0, 6.0]))
    assert torch.allclose(ppo.value_error(prediction, target, form="mse", delta=1.0), prediction**2)
    with pytest.raises(ValueError, match="value_loss"):
        ppo.value_error(prediction, target, form="l1", delta=1.0)
    # The config's form reaches the three value terms: with large errors they differ.
    trainer = train.Trainer(small(**{"run.network": "mlp"}), CPU)
    deals = collected(trainer)
    deals.realised = deals.realised * 40.0
    deals.lambda_return = deals.lambda_return * 40.0
    batch = deals.to_torch(CPU)
    out, value = ppo.evaluate(trainer.net, trainer.critic, batch)
    as_huber = ppo.compute_losses(out, value, batch, {**trainer.cfg["ppo"], "value_loss": "huber"})
    as_mse = ppo.compute_losses(out, value, batch, {**trainer.cfg["ppo"], "value_loss": "mse"})
    for name in ("critic", "value_head", "q"):
        assert as_mse[name] > 2 * as_huber[name], name
    assert torch.equal(as_mse["policy"], as_huber["policy"])


def test_c14_card_loss_counts_unseen_cards_only_and_partner_loss_skips_none():
    torch.manual_seed(14)
    logits = torch.randn(2, C.N_CARDS, C.CARD_LOCATION_CLASSES, requires_grad=True)
    target = torch.randint(0, C.CARD_LOCATION_CLASSES, (2, C.N_CARDS))
    unseen = torch.zeros(2, C.N_CARDS, dtype=torch.bool)
    unseen[0, 4] = unseen[1, 30] = unseen[1, 31] = True
    loss, count = ppo.card_location_loss(logits, target, unseen)
    want = torch.nn.functional.cross_entropy(logits[unseen], target[unseen])
    assert count == 3 and loss.item() == pytest.approx(want.item(), abs=1e-6)
    loss.backward()
    assert torch.all(logits.grad[~unseen] == 0) and logits.grad[unseen].abs().sum() > 0
    nothing, count = ppo.card_location_loss(logits.detach(), target, torch.zeros_like(unseen))
    assert count == 0 and nothing.item() == 0.0

    partner_logits = torch.randn(4, C.PARTNER_CLASSES, requires_grad=True)
    partner = torch.tensor([C.PARTNER_NONE, 2, C.PARTNER_NONE, 3])
    loss, count = ppo.partner_loss(partner_logits, partner)
    want = torch.nn.functional.cross_entropy(partner_logits[[1, 3]], partner[[1, 3]])
    assert count == 2 and loss.item() == pytest.approx(want.item(), abs=1e-6)
    loss.backward()
    assert torch.all(partner_logits.grad[[0, 2]] == 0)
    nothing, count = ppo.partner_loss(partner_logits.detach(), torch.full((4,), C.PARTNER_NONE))
    assert count == 0 and nothing.item() == 0.0


def test_c14_in_the_update_the_unseen_cards_come_from_state_block_f4():
    """A batch built by hand: one seat-deal, two decisions, three cards marked unseen in F4."""
    torch.manual_seed(15)
    cfg_ppo = cfg_with()["ppo"]
    parts = C.N_SCORE_PARTS
    state = torch.zeros(1, 2, C.STATE_FEATURES)
    offset = C.STATE_BLOCKS["F4"][0]
    assert offset == 1222
    state[0, 0, offset + 4] = 1.0
    state[0, 1, offset + 30] = state[0, 1, offset + 31] = 1.0
    card_target = torch.randint(0, C.CARD_LOCATION_CLASSES, (1, 2, C.N_CARDS))
    batch = ppo.Minibatch(
        tokens=torch.zeros(1, 1, 5, dtype=torch.long),
        points=torch.zeros(1, 2, dtype=torch.long),
        state=state,
        mask=torch.ones(1, 2, C.N_ACTIONS, dtype=torch.bool),
        critic_extra=torch.zeros(1, 2, C.CRITIC_EXTRA),
        valid=torch.ones(1, 2, dtype=torch.bool),
        partner_target=torch.tensor([[C.PARTNER_NONE, 2]]),
        card_target=card_target,
        action=torch.zeros(1, 2, dtype=torch.long),
        logp=torch.full((1, 2), -math.log(C.N_ACTIONS)),
        advantage=torch.zeros(1, 2),
        lambda_return=torch.zeros(1, 2, parts),
        realised=torch.zeros(1, 2, parts),
    )
    out = model.Outputs(
        logits=torch.zeros(1, 2, C.N_ACTIONS),
        q=torch.zeros(1, 2, C.N_ACTIONS),
        value=torch.zeros(1, 2, parts),
        partner=torch.randn(1, 2, C.PARTNER_CLASSES),
        cards=torch.randn(1, 2, C.N_CARDS, C.CARD_LOCATION_CLASSES),
    )
    losses = ppo.compute_losses(out, torch.zeros(1, 2, parts), batch, cfg_ppo)
    cross_entropy = torch.nn.functional.cross_entropy
    picked = torch.stack([out.cards[0, 0, 4], out.cards[0, 1, 30], out.cards[0, 1, 31]])
    targets = torch.stack([card_target[0, 0, 4], card_target[0, 1, 30], card_target[0, 1, 31]])
    assert losses["cards"].item() == pytest.approx(cross_entropy(picked, targets).item(), abs=1e-6)
    want_partner = cross_entropy(out.partner[0, 1:2], torch.tensor([2]))
    assert losses["partner"].item() == pytest.approx(want_partner.item(), abs=1e-6)
    # A card outside F4 does not count: moving its logits changes nothing.
    out.cards[0, 0, 7] += 5.0
    assert ppo.compute_losses(out, torch.zeros(1, 2, parts), batch, cfg_ppo)["cards"] == losses["cards"]


@pytest.mark.parametrize("kind", ["transformer", "mlp"])
def test_c15_frozen_seats_are_played_by_the_past_checkpoint_and_are_not_trained(kind, monkeypatch):
    # In the loop: seat 2 of every deal is frozen.
    def seat_two(rng, n_tables, share, max_frozen_seats):
        out = np.zeros((n_tables, 4), dtype=bool)
        out[:, 2] = True
        return out

    monkeypatch.setattr(opponents, "assign_frozen_seats", seat_two)
    trainer = train.Trainer(small(**{"run.network": kind}), CPU)
    trainer.pool.add(trainer.net)
    # The past checkpoint gets weights that give it away: it always takes its highest legal slot.
    snapshot = trainer.pool.snapshots[0]
    snapshot["heads.policy.weight"].zero_()
    snapshot["heads.policy.bias"] = 50.0 * torch.arange(C.N_ACTIONS, dtype=torch.float32)
    trainer.use_frozen_seats()

    seen = []
    real_step = trainer.env.step

    def recording(actions):
        obs = trainer.env.observe()
        highest = C.N_ACTIONS - 1 - obs["mask"][:, ::-1].argmax(axis=1)
        seen.append((obs["seat"].copy(), np.asarray(actions) == highest))
        return real_step(actions)

    trainer.env.step = recording
    for _ in range(60):
        trainer.step()
    seat = np.concatenate([s for s, _ in seen])
    took_highest = np.concatenate([h for _, h in seen])
    assert trainer.frozen_decisions == int((seat == 2).sum()) > 0
    assert took_highest[seat == 2].all(), "a frozen seat was not played by the past checkpoint"
    assert took_highest[seat != 2].mean() < 0.6, "the trained seats seem to be played by the past checkpoint"
    # No decision of seat 2 reaches the batch.
    assert set(np.unique(trainer.rollout.take().seat)) == {0, 1, 3}


def test_c15_the_share_of_deals_with_a_frozen_seat():
    rng = np.random.default_rng(15)
    frozen = opponents.assign_frozen_seats(rng, 1000, share=0.2, max_frozen_seats=3)
    assert frozen.shape == (1000, 4) and frozen.dtype == bool
    share = frozen.any(axis=1).mean()
    assert 0.15 <= share <= 0.25
    per_deal = frozen.sum(axis=1)
    assert set(per_deal[per_deal > 0]) == {1, 2, 3}
    assert not opponents.assign_frozen_seats(rng, 1000, share=0.0, max_frozen_seats=3).any()
    # Without a past checkpoint no seat is frozen, whatever the share.
    fresh = train.Trainer(small(**{"run.network": "mlp", "opponents.past_share": 1.0}), CPU)
    for _ in range(30):
        fresh.step()
    assert fresh.frozen_decisions == 0 and set(np.unique(fresh.rollout.take().seat)) == {0, 1, 2, 3}


@pytest.mark.parametrize("kind", ["transformer", "mlp"])
def test_c16_after_one_update_all_weights_and_the_loss_are_finite(kind):
    trainer = train.Trainer(small(**{"run.network": kind}), CPU)
    before = copy.deepcopy(trainer.net.state_dict())
    (stats,) = trainer.run(1)
    assert math.isfinite(stats["loss/total"])
    for module in (trainer.net, trainer.critic):
        for name, weight in module.named_parameters():
            assert torch.isfinite(weight).all(), name
    after = trainer.net.state_dict()
    assert any(not torch.equal(before[k], after[k]) for k in before)
    for name in ("policy", "value_head", "q", "partner", "cards", "entropy"):
        assert math.isfinite(stats[f"grad/trunk_{name}"]) and stats[f"grad/trunk_{name}"] > 0, name


@pytest.mark.parametrize("kind", ["transformer", "mlp"])
def test_c16_a_fresh_rollout_has_probability_ratio_one_on_every_decision(kind):
    """After updates, with frozen seats in play: the update must see what the actor saw.

    The log-probability stored when an action was taken and the one the update
    computes for it from the stored inputs, with unchanged weights, must agree to
    1e-5. A mask left out, a state from another row, wrong decision points, a
    stored value overwritten: each of them breaks this.
    """
    cfg = small(**{"run.network": kind, "opponents.snapshot_every": 1, "opponents.past_share": 0.5, "env.mode": "tokens"})
    trainer = train.Trainer(cfg, CPU)
    history = trainer.run(2)  # the weights have changed twice; a past checkpoint is in play
    assert trainer.frozen_active and len(trainer.pool) == 2
    before = trainer.frozen_decisions
    deals = collected(trainer)
    assert trainer.frozen_decisions > before, "no frozen seat was in play"
    batch = deals.to_torch(CPU)
    out, value = ppo.evaluate(trainer.net, trainer.critic, batch)
    logp = ppo.compute_losses(out, value, batch, cfg["ppo"])["logp"]
    assert batch.valid.sum() >= 80
    assert (logp - batch.logp[batch.valid]).abs().max().item() < 1e-5
    # The update measures the same thing itself, before its first step, at every update.
    assert all(h["ppo/behaviour_mismatch"] < 1e-5 for h in history)
    # And every decision it trains on is from the current weights.
    assert all(h["rollout/decision_age_max"] == 0 for h in history)
    assert np.all(deals.version[deals.valid] == trainer.updates)


def test_c16_a_small_remainder_joins_the_previous_minibatch():
    sizes = lambda n, batch: [len(chunk) for chunk in ppo.minibatches(np.arange(n), batch)]  # noqa: E731
    assert sizes(8, 4) == [4, 4]
    assert sizes(9, 4) == [4, 5]  # one seat-deal alone would get a full optimiser step
    assert sizes(10, 4) == [4, 4, 2]  # half a batch or more stands alone
    assert sizes(3, 4) == [3]
    assert sizes(257, 256) == [257]
    order = np.random.default_rng(0).permutation(21)
    assert sorted(np.concatenate(ppo.minibatches(order, 8)).tolist()) == list(range(21))


# --------------------------------------------------------------------------
# Learning on the mock task
# --------------------------------------------------------------------------

# Criterion 17: sizes and numbers of updates, fixed when the test was written.
# The MLP gets the task whose rewarded slot is named in the state. The transformer
# gets the task whose rewarded slot is named by a token at three decisions out of four.
LEARN = {
    "transformer": {"mode": "tokens", "updates": 30, "tables": 32, "seat_deals": 128, "update_batch": 32},
    "mlp": {"mode": "learnable", "updates": 30, "tables": 64, "seat_deals": 256, "update_batch": 32},
}


def learning_config(kind, mode=None):
    plan = LEARN[kind]
    return cfg_with(
        **{
            "run.network": kind,
            "run.seed": 17,
            "env.mode": mode or plan["mode"],
            "env.tables": plan["tables"],
            "ppo.seat_deals_per_update": plan["seat_deals"],
            "ppo.update_batch": plan["update_batch"],
            "opponents.snapshot_every": 10,
            "run.checkpoint_every": 1000,
        }
    )


def final_score(history):
    return float(np.mean([h["task/score"] for h in history[-3:]]))


@pytest.mark.parametrize("kind", ["transformer", "mlp"])
def test_c17_the_mean_reward_rises_by_half_the_distance_to_the_best(kind):
    cfg = learning_config(kind)
    started = time.perf_counter()
    trainer = train.Trainer(cfg, CPU)
    history = trainer.run(LEARN[kind]["updates"])
    seconds = time.perf_counter() - started
    random_level, best = trainer.env.random_reward, trainer.env.best_reward
    assert random_level == pytest.approx(0.25) and best == 1.0
    first, last = history[0]["task/score"], final_score(history)
    print(f"\n{kind}: reward per deal {first:.3f} -> {last:.3f} in {len(history)} updates, {seconds:.0f} s")
    print("  per update: " + " ".join(f"{h['task/score']:.2f}" for h in history))  # shown with pytest -s
    print(
        f"  largest approx_kl {max(h['ppo/approx_kl'] for h in history):.4f}, "
        f"largest clip fraction {max(h['ppo/clip_fraction'] for h in history):.3f}, "
        f"trained share {np.mean([h['rollout/trained_share'] for h in history]):.2f}"
    )
    # The two-minute limit of the criterion is for the training PC (measured in the spec's Results);
    # it is not asserted here, because the GitHub runner is slower.
    assert abs(first - random_level) < 0.08, "the first update is not at the random level"
    assert last >= random_level + 0.5 * (best - random_level), f"{kind} reached {last:.3f} only"
    assert trainer.env.illegal_actions == 0
    # The update must be a small, on-policy step at every update, not only end up well.
    assert all(h["ppo/behaviour_mismatch"] < 1e-4 for h in history), "the update did not see what the actor saw"
    assert all(h["rollout/decision_age_max"] == 0 for h in history), "an update trained on decisions of old weights"
    # Loose limits: a mask left out of the update gives approx_kl 5 and a clip fraction of 1.
    assert max(abs(h["ppo/approx_kl"]) for h in history) < 0.2, "the policy moved too far in one update"
    assert max(h["ppo/clip_fraction"] for h in history) < 0.8, "most decisions were outside the clip range"


@pytest.mark.slow
def test_c17_control_a_reward_given_to_the_wrong_seat_stops_the_learning():
    """The MLP run with one pipeline bug put in: each seat gets its neighbour's reward."""
    trainer = train.Trainer(learning_config("mlp"), CPU)
    real_step = trainer.env.step

    def wrong_seat(actions):
        done, reward, parts = real_step(actions)
        return done, np.roll(reward, 1, axis=1), np.roll(parts, 1, axis=1)

    trainer.env.step = wrong_seat
    last = final_score(trainer.run(LEARN["mlp"]["updates"]))
    level = trainer.env.random_reward
    assert last < level + 0.25 * (trainer.env.best_reward - level), f"it still learned: {last:.3f}"


@pytest.mark.slow
def test_c17_control_the_token_task_cannot_be_learned_without_the_tokens():
    """The MLP reads no tokens. On the token task it stays under half of the distance.

    So the transformer's result on that task shows that the tokens reach the policy:
    through the decision points in the update and through the cache when acting.
    """
    trainer = train.Trainer(learning_config("mlp", mode="tokens"), CPU)
    last = final_score(trainer.run(LEARN["mlp"]["updates"]))
    level, best = trainer.env.random_reward, trainer.env.best_reward
    ceiling = level + (1 - mock_env.TOKEN_CUE_SHARE) * (best - level)  # the decisions the state can solve
    assert last < ceiling + 0.08 < level + 0.5 * (best - level), f"the MLP reached {last:.3f} on the token task"


@pytest.mark.parametrize("kind", [pytest.param("transformer", marks=pytest.mark.slow), "mlp"])
def test_c18_random_rewards_run_20_updates_and_the_entropy_does_not_collapse(kind):
    cfg = small(**{"run.network": kind, "env.mode": "random"})
    trainer = train.Trainer(cfg, CPU)
    history = trainer.run(20)
    assert len(history) == 20 and all(math.isfinite(h["loss/total"]) for h in history)
    assert "task/score" not in history[0] and history[0]["reward/mean_abs"] > 0
    # No collapse on noise: the entropy keeps its level (no slide from the first updates to the
    # last) and never falls under half of the uniform policy's. The first policy is not uniform:
    # the policy head starts at PyTorch's default scale.
    entropy = [h["entropy/mean"] for h in history]
    uniform = math.log(cfg["env"]["legal_per_decision"])
    print(f"{kind}: entropy on random rewards " + " ".join(f"{e:.2f}" for e in entropy))  # shown with pytest -s
    assert min(entropy) > 0.5 * uniform
    assert np.mean(entropy[-5:]) > 0.8 * np.mean(entropy[:5])


# --------------------------------------------------------------------------
# Run mechanics
# --------------------------------------------------------------------------


@pytest.mark.parametrize("kind", ["transformer", "mlp"])
def test_c19_checkpoint_round_trip_and_encoding_version_check(kind, tmp_path):
    trainer = train.Trainer(small(**{"run.network": kind}), CPU)
    trainer.run(2)
    path = tmp_path / "checkpoint.pt"
    trainer.save(path)

    saved = torch.load(path, weights_only=True)
    assert saved["encoding_version"] == C.ENCODING_VERSION
    assert saved["config"] == trainer.cfg
    assert saved["updates"] == 2 and saved["decisions"] == trainer.decisions > 0
    assert saved["optimizer"]["state"], "the optimiser state is empty"
    assert len(saved["pool"]) == 1 and saved["frozen_active"] is True  # one snapshot after two updates

    loaded = train.Trainer.load(path, CPU)
    assert loaded.updates == 2 and loaded.decisions == trainer.decisions
    tokens, points, state, mask = random_deals(2, 4, 30, seed=19)
    with torch.inference_mode():
        a = trainer.net.forward_deals(tokens, points, state, mask)
        b = loaded.net.forward_deals(tokens, points, state, mask)
        features = torch.rand(3, C.CRITIC_FEATURES)
        assert torch.equal(trainer.critic(features), loaded.critic(features))
    for name in a._fields:
        assert torch.equal(getattr(a, name), getattr(b, name)), name
    old = trainer.optimizer.state_dict()["state"]
    new = loaded.optimizer.state_dict()["state"]
    assert all(torch.equal(old[k]["exp_avg"], new[k]["exp_avg"]) for k in old)

    saved["encoding_version"] = C.ENCODING_VERSION + 1
    other = tmp_path / "other-version.pt"
    torch.save(saved, other)
    with pytest.raises(train.EncodingVersionError, match="encoding version"):
        train.Trainer.load(other, CPU)


@pytest.mark.parametrize("kind", [pytest.param("transformer", marks=pytest.mark.slow), "mlp"])
def test_c19_a_resumed_run_equals_an_unbroken_run(kind, tmp_path):
    """3 updates, checkpoint, load, 3 more updates: the same numbers as 6 updates in one go."""
    cfg = small(**{"run.network": kind, "run.seed": 19, "opponents.past_share": 0.5})
    watched = ("loss/total", "loss/policy", "loss/critic", "entropy/mean", "task/score", "rollout/decisions")

    def numbers(history):
        return [tuple(h[name] for name in watched) for h in history]

    unbroken = numbers(train.Trainer(cfg, CPU).run(6))
    first = train.Trainer(cfg, CPU)
    start = numbers(first.run(3))
    first.save(tmp_path / "three.pt")
    resumed = train.Trainer.load(tmp_path / "three.pt", CPU)
    assert resumed.updates == 3 and len(resumed.pool) == len(first.pool) == 1
    assert start + numbers(resumed.run(3)) == unbroken
    assert resumed.frozen_decisions > 0, "the past checkpoint did not play: the test would not cover the pool"


@pytest.mark.parametrize("kind", [pytest.param("transformer", marks=pytest.mark.slow), "mlp"])
def test_c20_two_cpu_runs_with_one_seed_give_identical_losses(kind):
    def losses():
        trainer = train.Trainer(small(**{"run.network": kind, "run.seed": 20}), CPU)
        return [(h["loss/total"], h["loss/policy"], h["task/score"]) for h in trainer.run(5)]

    first, second = losses(), losses()
    assert first == second
    assert len(set(first)) == 5


def test_c21_tensorboard_files_hold_the_required_scalars(tmp_path):
    trainer = train.Trainer(small(**{"run.network": "mlp"}), CPU, log_dir=tmp_path)
    trainer.run(3)
    trainer.close()
    assert list(tmp_path.glob("events.out.tfevents.*"))
    tags = scalar_tags(tmp_path)
    required = {
        "loss/total",
        "loss/policy",
        "loss/critic",
        "loss/value_head",
        "loss/q",
        "loss/partner",
        "loss/cards",
        "loss/entropy",
        "critic/explained_variance",
        "aux/partner_accuracy",
        "speed/steps_per_second",
    }
    assert required <= tags, required - tags
    assert {"entropy/bidding", "entropy/play"} <= tags  # entropy per phase
    assert {f"actions/{name}" for name in ("BID", "PLAY")} <= tags  # share of each action group chosen
    assert all(tag.split("/")[1] in C.PHASES + ("mean",) for tag in tags if tag.startswith("entropy/"))
    assert all(tag.split("/")[1] in C.ACTION_GROUPS for tag in tags if tag.startswith("actions/"))
    # Added by the review: what each loss term pulls on the trunk, and the checks on the data.
    review = {
        "grad/trunk_policy",
        "grad/trunk_value_head",
        "grad/trunk_q",
        "grad/actor",
        "grad/critic",
        "ppo/behaviour_mismatch",
        "rollout/decision_age_max",
        "rollout/trained_share",
        "reward/mean_abs",
        "task/score",
    }
    assert review <= tags, review - tags


@pytest.mark.gpu
@pytest.mark.parametrize("kind", ["transformer", "mlp"])
def test_c22_the_train_command_runs_on_the_gpu_and_resumes(kind, tmp_path, capsys):
    """Three updates through the command line on the GPU, then one more from the checkpoint.

    The 200-update runs are in the spec's Results.
    """
    common = ["--config", str(MOCK_TOML), "--tables", "64", "--seat-deals", "128"]
    code = train.main([*common, "--network", kind, "--updates", "3", "--out-dir", str(tmp_path)])
    out = capsys.readouterr().out
    assert code == 0 and "steps/s" in out and "cuda" in out
    assert list(tmp_path.rglob("events.out.tfevents.*"))
    (checkpoint,) = tmp_path.rglob("checkpoint-000003.pt")
    code = train.main(["--resume", str(checkpoint), "--updates", "4"])
    out = capsys.readouterr().out
    assert code == 0 and "resumed from checkpoint-000003.pt at update 3" in out
    assert (checkpoint.parent / "checkpoint-000004.pt").exists()


def test_c22_the_train_command_runs_on_the_cpu_and_resumes(tmp_path, capsys):
    common = ["--config", str(MOCK_TOML), "--tables", "8", "--seat-deals", "24", "--device", "cpu"]
    code = train.main([*common, "--network", "mlp", "--updates", "2", "--out-dir", str(tmp_path)])
    assert code == 0 and "steps/s" in capsys.readouterr().out
    (checkpoint,) = tmp_path.rglob("checkpoint-000002.pt")
    code = train.main(["--resume", str(checkpoint), "--updates", "3"])
    assert code == 0 and "resumed from checkpoint-000002.pt at update 2" in capsys.readouterr().out
    assert (checkpoint.parent / "checkpoint-000003.pt").exists()
    assert train.main(["--resume", str(checkpoint), "--updates", "2"]) == 0  # nothing left to do


# --------------------------------------------------------------------------
# Quality gates
# --------------------------------------------------------------------------


def test_c24_every_hyperparameter_is_in_the_config_file():
    cfg = train.load_config(MOCK_TOML)
    # The starting values of the spec's table (D119).
    want = {
        ("ppo", "clip_range"): 0.2,
        ("ppo", "epochs"): 2,
        ("ppo", "gamma"): 1.0,
        ("ppo", "gae_lambda"): 0.95,
        ("ppo", "learning_rate"): 3e-4,
        ("env", "tables"): 1024,
        ("ppo", "update_batch"): 256,
        ("ppo", "entropy_weight"): 0.01,
        ("ppo", "critic_weight"): 0.5,
        ("ppo", "q_weight"): 0.5,
        ("ppo", "partner_weight"): 0.1,
        ("ppo", "card_location_weight"): 0.1,
        ("ppo", "gradient_clip"): 0.5,
        ("opponents", "past_share"): 0.2,
    }
    for (section, key), value in want.items():
        assert cfg[section][key] == value, (section, key)
    # Added after the review; the user decides their values.
    for key in ("value_head_weight", "value_loss", "huber_delta", "seat_deals_per_update"):
        assert key in cfg["ppo"]
    # No defaults in the code: a config without any one key is refused.
    for section in cfg:
        for key in cfg[section]:
            broken = copy.deepcopy(cfg)
            del broken[section][key]
            with pytest.raises(train.ConfigError, match=key):
                train.validate_config(broken)
    extra = copy.deepcopy(cfg)
    extra["ppo"]["surprise"] = 1
    with pytest.raises(train.ConfigError, match="surprise"):
        train.validate_config(extra)
    # The functions that use hyperparameters take them as arguments without defaults.
    for function in (
        ppo.gae,
        ppo.realised_return,
        ppo.clipped_policy_loss,
        ppo.value_error,
        ppo.minibatches,
        opponents.assign_frozen_seats,
    ):
        for parameter in inspect.signature(function).parameters.values():
            assert parameter.default is inspect.Parameter.empty, f"{function.__name__}({parameter.name}=...)"
    for cls in (model.TransformerNet, model.MLPNet, model.Critic, mock_env.MockEnv):
        for parameter in list(inspect.signature(cls.__init__).parameters.values())[1:]:
            assert parameter.default is inspect.Parameter.empty, f"{cls.__name__}({parameter.name}=...)"
    # The weight of the actor's own value head is a config value: changing it changes the total.
    trainer = train.Trainer(small(**{"run.network": "mlp"}), CPU)
    batch = collected(trainer).to_torch(CPU)
    out, value = ppo.evaluate(trainer.net, trainer.critic, batch)
    totals = [
        ppo.compute_losses(out, value, batch, {**trainer.cfg["ppo"], "value_head_weight": weight})["total"].item()
        for weight in (0.0, 0.5, 2.0)
    ]
    assert totals[0] < totals[1] < totals[2]
