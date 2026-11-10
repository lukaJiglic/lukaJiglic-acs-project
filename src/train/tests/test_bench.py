"""Tests for the benchmark stand-in networks (spec 003, criteria 5-8).

They check that the two sequence layouts compute the same thing, so the
benchmark compares cost and nothing else. All but the last run on the CPU.
"""

import json

import pytest
import torch

from tarok_train import bench_model as bm

TOL = 1e-4


def _deal(batch=2, seed=5):
    gen = torch.Generator().manual_seed(seed)
    torch.manual_seed(seed)
    net = bm.TransformerNet().eval()
    data = bm.make_l2_batch(batch, torch.device("cpu"), gen)
    return net, data


def _close(a, b):
    """Equal within TOL; -inf (masked actions) must sit at the same places."""
    finite = torch.isfinite(a)
    assert torch.equal(finite, torch.isfinite(b))
    return torch.max(torch.abs(a[finite] - b[finite])).item() < TOL


def test_l1_l2_and_cached_acting_give_the_same_outputs():
    net, data = _deal()
    with torch.inference_mode():
        l2 = net.forward_l2(data["tokens"], data["points"], data["state"], data["legal"])
        cache = bm.KVCache(2, torch.device("cpu"), torch.float32)
        for k, point in enumerate(bm.DECISION_POINTS):
            lengths = torch.full((2,), point)
            l1 = net.forward_l1(data["tokens"], lengths, data["state"][:, k], data["legal"][:, k])
            new_tokens = data["tokens"][:, cache.filled : point]
            cached = net.act_l2_cached(cache, new_tokens, data["state"][:, k], data["legal"][:, k])
            for name, a, b, c in zip(("policy", "q", "value"), l1, cached, l2):
                assert _close(a, c[:, k]), f"L1 and L2 differ in {name} at decision {k}"
                assert _close(b, c[:, k]), f"cached acting and L2 differ in {name} at decision {k}"


def test_ragged_cached_acting_equals_l2_for_deals_at_different_points():
    gen = torch.Generator().manual_seed(11)
    net, data = _deal(batch=3, seed=11)
    # Every deal gets its own decision points; equal neighbours give steps with no new token.
    points = torch.randint(0, bm.MAX_PLAY + 1, (3, bm.N_STATE), generator=gen).sort(dim=1).values
    n_max = int(torch.diff(points, dim=1, prepend=torch.zeros(3, 1, dtype=torch.long)).max())
    with torch.inference_mode():
        l2 = net.forward_l2(data["tokens"], points, data["state"], data["legal"])
        cache = bm.KVCache(3, torch.device("cpu"), torch.float32)
        t0 = torch.zeros(3, dtype=torch.long)
        for k in range(bm.N_STATE):
            n = points[:, k] - t0
            got = net.act_l2_ragged(cache, data["tokens"], t0, n, data["state"][:, k], data["legal"][:, k], n_max)
            for name, a, b in zip(("policy", "q", "value"), got, l2):
                assert _close(a, b[:, k]), f"ragged acting and L2 differ in {name} at decision {k}"
            t0 = points[:, k]


def test_ragged_step_wraps_around_and_returns_host_actions():
    gen = torch.Generator().manual_seed(12)
    torch.manual_seed(12)
    net = bm.TransformerNet()
    step, decisions = bm.build_ragged_act_step(net, "L2", 5, torch.device("cpu"), False, gen)
    assert decisions == 5
    for _ in range(2 * bm.N_STATE + 3):  # every deal finishes at least twice
        action = step()
        assert action.device.type == "cpu" and action.shape == (5, 1)
        assert int(action.min()) >= 0 and int(action.max()) < bm.N_ACTIONS


def test_l1_bucketed_equals_l1_padded():
    gen = torch.Generator().manual_seed(7)
    torch.manual_seed(7)
    net = bm.TransformerNet().eval()
    data = bm.make_l1_batch(13, torch.device("cpu"), gen, sort=True)
    args = (data["tokens"], data["lengths"], data["state"], data["legal"])
    with torch.inference_mode():
        padded = net.forward_l1(*args)
        bucketed = net.forward_l1_bucketed(*args)
    for a, b in zip(padded, bucketed):
        assert a.shape == b.shape
        assert _close(a, b)


def test_l2_state_tokens_do_not_see_each_other_or_later_play_tokens():
    net, data = _deal(batch=1)
    k = 6
    point = bm.DECISION_POINTS[k]
    with torch.inference_mode():
        base = net.forward_l2(data["tokens"], data["points"], data["state"], data["legal"])

        other_state = data["state"].clone()
        other_state[:, k + 1] += 1.0  # change another decision's state token
        other_state[:, k - 1] -= 1.0
        moved = net.forward_l2(data["tokens"], data["points"], other_state, data["legal"])
        assert _close(base[1][:, k], moved[1][:, k])
        assert not _close_or_false(base[1][:, k + 1], moved[1][:, k + 1])

        later = data["tokens"].clone()
        later[:, point:] = (later[:, point:] + 1) % bm.VOCAB  # change play tokens after the decision
        moved = net.forward_l2(later, data["points"], data["state"], data["legal"])
        assert _close(base[1][:, k], moved[1][:, k])

        earlier = data["tokens"].clone()
        earlier[:, point - 1] = (earlier[:, point - 1] + 1) % bm.VOCAB  # the last visible play token
        moved = net.forward_l2(earlier, data["points"], data["state"], data["legal"])
        assert not _close_or_false(base[1][:, k], moved[1][:, k])


def _close_or_false(a, b):
    return torch.max(torch.abs(a - b)).item() < TOL


def test_l2_mask_shape_and_rows():
    points = torch.tensor([[0, 3, 5]])
    mask = bm.l2_mask(points, 5)[0, 0]
    assert mask.shape == (8, 8)
    assert mask[:5, :5].equal(torch.tril(torch.ones(5, 5, dtype=torch.bool)))
    assert not mask[:5, 5:].any()  # play tokens never see state tokens
    assert mask[5].tolist() == [False] * 5 + [True, False, False]  # decision point 0: itself only
    assert mask[6].tolist() == [True] * 3 + [False] * 2 + [False, True, False]
    assert mask[7].tolist() == [True] * 5 + [False, False, True]


def test_illegal_actions_get_zero_probability():
    gen = torch.Generator().manual_seed(1)
    net = bm.MLPNet().eval()
    data = bm.make_mlp_batch(8, torch.device("cpu"), gen)
    with torch.inference_mode():
        logits, q, value = net(data["state"], data["legal"])
    probs = torch.softmax(logits, dim=-1)
    assert logits.shape == (8, bm.N_ACTIONS) and q.shape == (8, bm.N_ACTIONS) and value.shape == (8,)
    assert torch.all(probs[~data["legal"]] == 0)
    assert torch.allclose(probs.sum(-1), torch.ones(8), atol=1e-5)
    assert data["legal"].gather(-1, data["action"].unsqueeze(-1)).all()


@pytest.mark.parametrize("layout", ["mlp", "L1 padded", "L1 bucketed", "L2"])
def test_update_step_gives_finite_loss_and_changes_weights(layout):
    device = torch.device("cpu")
    gen = torch.Generator().manual_seed(2)
    torch.manual_seed(2)
    net = bm.NETWORKS[layout][1]()
    before = net.heads.policy.weight.detach().clone()
    step, decisions = bm.build_update_step(net, layout, 4, device, False, gen)
    step()
    assert decisions == (4 * bm.N_STATE if layout == "L2" else 4)
    assert torch.isfinite(step.last["loss"])
    assert not torch.equal(before, net.heads.policy.weight)
    for name, weight in net.named_parameters():
        assert torch.isfinite(weight).all(), f"{name} is not finite after the step"
        assert weight.grad is None or torch.isfinite(weight.grad).all(), f"gradient of {name} is not finite"


def test_sizes_match_the_plan():
    assert bm.DECISION_POINTS[0] == 5 and bm.DECISION_POINTS[-1] == bm.MAX_PLAY
    assert len(bm.DECISION_POINTS) == bm.N_STATE
    n = bm.count_parameters(bm.TransformerNet())
    assert 3_000_000 < n < 3_700_000  # build-plan section 6: about 3.1 M in the layers


def test_end_to_end_formula():
    # 1000 acting/s and 1000 update samples/s: 1 ms + 4 * 1 ms per step with 4 epochs.
    assert bm.end_to_end(1000.0, 1000.0, 4) == pytest.approx(200.0)
    assert bm.end_to_end(1000.0, 1000.0, 2) == pytest.approx(1000.0 / 3)


def test_time_step_gives_one_value_per_repeat():
    calls = []
    per_call = bm.time_step(lambda: calls.append(1), torch.device("cpu"), seconds=0.0, repeats=3, warmup=2)
    assert len(per_call) == 3 and all(t >= 0 for t in per_call)
    assert len(calls) == 2 + 3 * 3  # warm-up, then at least 3 calls per repeat


def test_quick_benchmark_runs_on_cpu():
    result = bm.run(label="test", seconds=0.01, repeats=2, quick=True, device="cpu")
    modes = {(r["layout"], r["mode"]) for r in result["rows"]}
    for layout in ("mlp", "L1 padded", "L1 bucketed", "L2"):
        assert (layout, "act") in modes and (layout, "update") in modes
    assert ("L2", "act ragged") in modes
    assert {r["layout"] for r in result["rows"] if r["mode"] == "cpu"} == {"mlp", "L1", "L1 longest", "L2"}
    for r in result["rows"]:
        if r["mode"] != "cpu":
            assert 0 < r["per_s_min"] <= r["per_s"] <= r["per_s_max"]
    assert len(result["estimates"]) == len(bm.ESTIMATES)
    for e in result["estimates"]:
        assert e["steps_per_s_4_epochs"] < e["steps_per_s_2_epochs"]
    text = bm.markdown(result)
    assert "acting, ragged, host copy" in text and "Steps/s, 4 epochs (derived)" in text
    # The result is committed: no process names and no machine paths in it.
    dumped = json.dumps(result)
    assert ".exe" not in dumped and "Users" not in dumped and "processes" not in dumped


def test_sizes_can_be_set_to_the_encoding_contract():
    before = bm.shape()
    try:
        bm.set_shape(state_features=1382, play_tokens=128, decisions=20, actions=160, vocab=288)
        assert bm.DECISION_POINTS[-1] == 128 and len(bm.DECISION_POINTS) == 20
        gen = torch.Generator().manual_seed(9)
        net = bm.TransformerNet().eval()
        data = bm.make_l2_batch(2, torch.device("cpu"), gen)
        assert data["state"].shape == (2, 20, 1382) and data["tokens"].shape == (2, 128)
        with torch.inference_mode():
            logits, q, value = net.forward_l2(data["tokens"], data["points"], data["state"], data["legal"])
        assert logits.shape == (2, 20, 160) and q.shape == (2, 20, 160) and value.shape == (2, 20)
        step, decisions = bm.build_ragged_act_step(net, "L2", 3, torch.device("cpu"), False, gen)
        assert step().shape == (3, 1) and decisions == 3
    finally:
        bm.set_shape(**{
            "state_features": before["state_features"],
            "play_tokens": before["play_tokens"],
            "decisions": before["decisions"],
            "actions": before["actions"],
            "vocab": before["vocab"],
        })
    assert bm.shape() == before and bm.DECISION_POINTS[-1] == 70


@pytest.mark.gpu
def test_half_precision_update_is_finite_on_the_gpu():
    """The GTX 16 series has a history of NaNs in half precision; check each layout once."""
    device = torch.device("cuda")
    for layout in ("mlp", "L1 padded", "L2"):
        gen = torch.Generator(device=device).manual_seed(4)
        net = bm.NETWORKS[layout][1]().to(device)
        step, _ = bm.build_update_step(net, layout, 8, device, True, gen)
        for _ in range(3):
            step()
        assert torch.isfinite(step.last["loss"]).item(), layout
    # Cached acting in half precision: the cache holds fp16 keys and values.
    gen = torch.Generator(device=device).manual_seed(4)
    net = bm.TransformerNet().to(device)
    for builder in (bm.build_act_step, bm.build_ragged_act_step):
        step, _ = builder(net, "L2", 8, device, True, gen)
        for _ in range(bm.N_STATE + 2):
            step()
