"""Throughput benchmark of the planned networks on synthetic data (spec 003, criteria 5-8).

Run:  python -m tarok_train.bench_model --label "as used daily"

The networks here are stand-ins with the planned sizes (D18): a transformer with
4 layers, width 256, 4 heads, feed-forward 1024, and an MLP control 512-512-256.
Data is random but shaped like a deal: 70 play and bid tokens, 15 decisions per
seat, 700 state features per decision, 140 actions with a legality mask.

Two sequence layouts are timed for the transformer (B10-4):

  L1  One sequence per decision: the play tokens so far, then one state token.
      The whole sequence is recomputed for every decision.
  L2  One sequence per seat and deal: 70 play tokens and 15 state tokens.
      Play tokens attend causally to play tokens. A state token attends to the
      play tokens before its decision point and to itself. Training runs all 15
      decisions in one pass. Acting keeps the play tokens' keys and values in a
      cache and computes only the new tokens.

With the causal mask both layouts give the same outputs for the same decision
(tests/test_bench.py checks this), so the comparison is about cost only.

Acting with the cache is the same computation for a causal L1 and for L2. The
layouts differ in the update: L1 recomputes the prefix for every decision, L2
shares it between the 15 decisions of a deal. Two cached acting rows are timed:
"act" (idealised: every deal in the batch is at the same decision, the action
stays on the GPU) and "act ragged" (deals at different decisions, one mask per
deal, the action copied to the host at every step).

Every row is timed in several repeats; the median, the minimum and the maximum
are reported. The sizes (state width, token counts, actions) can be set on the
command line, for a run on the sizes of docs/encoding.md.

Not measured here: the engine, moving data from the engine to the GPU, GAE, and
the full-information critic. Eager mode, no torch.compile.
"""

from __future__ import annotations

import argparse
import datetime
import gc
import json
import re
import statistics
import subprocess
import sys
import time
from pathlib import Path

import torch
import torch.nn as nn
import torch.nn.functional as F

# The transformer layer and the L2 mask are shared with the real model (spec 007).
from .model import Block, l2_mask  # noqa: F401  (l2_mask is also used by tests/test_bench.py)

VOCAB = 256
MAX_PLAY = 70  # play and bid tokens in one deal, upper end
N_STATE = 15  # decisions of one seat in one deal
STATE_FEATURES = 700
N_ACTIONS = 140
D_MODEL = 256
N_HEADS = 4
N_LAYERS = 4
D_FF = 1024
MLP_SIZES = (512, 512, 256)
L1_BUCKETS = 4
MIB = 1024 * 1024


def _decision_points(n_play: int, n_state: int) -> tuple:
    return tuple(round((k + 1) * n_play / n_state) for k in range(n_state))


# Decision k of a seat sees the first DECISION_POINTS[k] play tokens: 5, 9, 14, ... 70.
DECISION_POINTS = _decision_points(MAX_PLAY, N_STATE)
MIN_PREFIX = DECISION_POINTS[0]  # shortest L1 prefix


def set_shape(state_features=None, play_tokens=None, decisions=None, actions=None, vocab=None) -> dict:
    """Change the synthetic sizes, module-wide. Arguments left at None keep their value."""
    global STATE_FEATURES, MAX_PLAY, N_STATE, N_ACTIONS, VOCAB, DECISION_POINTS, MIN_PREFIX
    new_play = play_tokens or MAX_PLAY
    new_state = decisions or N_STATE
    if new_state > new_play:
        raise ValueError("more decisions than play tokens")
    STATE_FEATURES = state_features or STATE_FEATURES
    MAX_PLAY = new_play
    N_STATE = new_state
    N_ACTIONS = actions or N_ACTIONS
    VOCAB = vocab or VOCAB
    DECISION_POINTS = _decision_points(MAX_PLAY, N_STATE)
    MIN_PREFIX = DECISION_POINTS[0]
    return shape()


def shape() -> dict:
    return {
        "state_features": STATE_FEATURES,
        "play_tokens": MAX_PLAY,
        "decisions": N_STATE,
        "actions": N_ACTIONS,
        "vocab": VOCAB,
    }


# --------------------------------------------------------------------------
# Networks
# --------------------------------------------------------------------------


class Heads(nn.Module):
    """Policy over the actions with a legality mask, Q over the actions, scalar value."""

    def __init__(self, width: int):
        super().__init__()
        self.policy = nn.Linear(width, N_ACTIONS)
        self.q = nn.Linear(width, N_ACTIONS)
        self.value = nn.Linear(width, 1)

    def forward(self, h, legal):
        logits = self.policy(h).masked_fill(~legal, float("-inf"))
        return logits, self.q(h), self.value(h).squeeze(-1)


class KVCache:
    """Keys and values of the play tokens of a batch of deals, per layer (cached acting)."""

    def __init__(self, batch: int, device, dtype):
        # Slots 0..MAX_PLAY-1: play tokens. One more for the state token of the current
        # step. The last slot is a dump for padding rows (ragged acting only).
        size = (batch, N_HEADS, MAX_PLAY + 2, D_MODEL // N_HEADS)
        self.k = [torch.zeros(size, device=device, dtype=dtype) for _ in range(N_LAYERS)]
        self.v = [torch.zeros(size, device=device, dtype=dtype) for _ in range(N_LAYERS)]
        self.filled = 0

    def reset(self):
        self.filled = 0


class TransformerNet(nn.Module):
    def __init__(self):
        super().__init__()
        self.tok = nn.Embedding(VOCAB, D_MODEL)
        self.pos = nn.Embedding(MAX_PLAY + 1, D_MODEL)
        self.state_proj = nn.Linear(STATE_FEATURES, D_MODEL)
        self.blocks = nn.ModuleList(Block(D_MODEL, N_HEADS, D_FF) for _ in range(N_LAYERS))
        self.norm = nn.LayerNorm(D_MODEL)
        self.heads = Heads(D_MODEL)

    # ---- L1: whole sequence per decision ----

    def forward_l1(self, tokens, lengths, state, legal):
        """tokens (B, >=max length), lengths (B,), state (B, F), legal (B, A).

        The state token sits at index lengths[b]. Tokens after it are padding; the
        causal mask keeps the state token from seeing them.
        """
        b = tokens.shape[0]
        tp = int(lengths.max())
        x = self.tok(tokens[:, :tp]) + self.pos.weight[:tp]
        x = torch.cat([x, x.new_zeros(b, 1, D_MODEL)], dim=1)
        where = lengths.view(b, 1, 1).expand(b, 1, D_MODEL)
        state_token = self.state_proj(state) + self.pos(lengths)
        x = x.scatter(1, where, state_token.unsqueeze(1).to(x.dtype))
        for block in self.blocks:
            x = block(x, causal=True)
        h = self.norm(x.gather(1, where).squeeze(1))
        return self.heads(h, legal)

    def forward_l1_bucketed(self, tokens, lengths, state, legal, n_buckets: int = L1_BUCKETS):
        """Same as forward_l1 for a batch sorted by length: n_buckets slices, each padded to its own max."""
        b = tokens.shape[0]
        edges = [round(i * b / n_buckets) for i in range(n_buckets + 1)]
        outs = [
            self.forward_l1(tokens[lo:hi], lengths[lo:hi], state[lo:hi], legal[lo:hi])
            for lo, hi in zip(edges[:-1], edges[1:])
            if hi > lo
        ]
        return tuple(torch.cat(part, dim=0) for part in zip(*outs))

    # ---- L2: one sequence per seat and deal ----

    def forward_l2(self, tokens, points, state, legal):
        """tokens (B, P), points (B, S), state (B, S, F), legal (B, S, A). All S decisions in one pass."""
        p = tokens.shape[1]
        x = torch.cat([self.tok(tokens) + self.pos.weight[:p], self.state_proj(state) + self.pos(points)], dim=1)
        mask = l2_mask(points, p)
        for block in self.blocks:
            x = block(x, mask=mask)
        return self.heads(self.norm(x[:, p:]), legal)

    # ---- Cached acting (the same computation for a causal L1 and for L2) ----

    def act_l2_cached(self, cache: KVCache, new_tokens, state, legal):
        """One acting decision per deal. new_tokens (B, n): play tokens since the last decision.

        Only the n new play tokens and the state token are computed. Earlier play
        tokens are read from the cache. All deals in the batch are at the same point.
        """
        b, n = new_tokens.shape
        t0 = cache.filled
        t1 = t0 + n
        dev = new_tokens.device
        play = self.tok(new_tokens) + self.pos.weight[t0:t1]
        state_token = self.state_proj(state) + self.pos.weight[t1]
        x = torch.cat([play, state_token.unsqueeze(1).to(play.dtype)], dim=1)  # (B, n+1, D)
        # Rows: n new play tokens, then the state token. Columns: play tokens 0..t1-1, then the state token.
        cols = torch.arange(t1 + 1, device=dev)
        mask = cols[None, :] <= (t0 + torch.arange(n + 1, device=dev))[:, None]
        for i, block in enumerate(self.blocks):
            q, k, v = block.qkv_of(x)
            # Slot t1 holds the state token for this step; the next play token overwrites it.
            cache.k[i][:, :, t0 : t1 + 1] = k
            cache.v[i][:, :, t0 : t1 + 1] = v
            attn = F.scaled_dot_product_attention(
                q, cache.k[i][:, :, : t1 + 1], cache.v[i][:, :, : t1 + 1], attn_mask=mask
            )
            x = block.finish(x, attn)
        cache.filled = t1
        return self.heads(self.norm(x[:, n]), legal)

    def act_l2_ragged(self, cache: KVCache, tokens, t0, n, state, legal, n_max: int):
        """One acting decision per deal, deals at different points.

        tokens (B, P): all play tokens of each deal. t0 (B,): play tokens already in
        the cache. n (B,): new play tokens of this step, at most n_max. The step
        computes n_max + 1 rows per deal: the new play tokens, padding, the state token.
        """
        b, p = tokens.shape
        dev = tokens.device
        offset = torch.arange(n_max, device=dev)
        position = t0[:, None] + offset[None, :]  # (B, n_max): place of each play row in the deal
        is_new = offset[None, :] < n[:, None]
        t1 = t0 + n
        source = position.clamp(max=p - 1)
        play = self.tok(tokens.gather(1, source)) + self.pos(source)  # rows past n are padding
        state_token = self.state_proj(state) + self.pos(t1)
        x = torch.cat([play, state_token.unsqueeze(1).to(play.dtype)], dim=1)  # (B, n_max+1, D)
        # Where each row's key and value go: its own slot, the state slot t1, or the dump slot.
        slot = torch.cat([torch.where(is_new, position, p + 1), t1[:, None]], dim=1)
        index = slot[:, None, :, None].expand(b, N_HEADS, n_max + 1, D_MODEL // N_HEADS)
        # Row r may attend to slots 0..reach[r]: a play token to itself and earlier ones,
        # the state token to every play token so far and to itself.
        reach = torch.cat([position, t1[:, None]], dim=1)
        mask = (torch.arange(p + 1, device=dev)[None, None, :] <= reach[:, :, None])[:, None]
        for i, block in enumerate(self.blocks):
            q, k, v = block.qkv_of(x)
            cache.k[i].scatter_(2, index, k)
            cache.v[i].scatter_(2, index, v)
            attn = F.scaled_dot_product_attention(
                q, cache.k[i][:, :, : p + 1], cache.v[i][:, :, : p + 1], attn_mask=mask
            )
            x = block.finish(x, attn)
        return self.heads(self.norm(x[:, n_max]), legal)


class MLPNet(nn.Module):
    """Control: state -> 512 -> 512 -> 256, same heads. Sees the state features only."""

    def __init__(self):
        super().__init__()
        layers = []
        width = STATE_FEATURES
        for size in MLP_SIZES:
            layers += [nn.Linear(width, size), nn.ReLU()]
            width = size
        self.trunk = nn.Sequential(*layers)
        self.heads = Heads(width)

    def forward(self, state, legal):
        return self.heads(self.trunk(state), legal)


def count_parameters(module: nn.Module) -> int:
    return sum(p.numel() for p in module.parameters())


# --------------------------------------------------------------------------
# Synthetic data and the loss
# --------------------------------------------------------------------------


def make_targets(shape, device, gen):
    """Random PPO targets for decisions of the given leading shape."""
    legal = torch.rand(*shape, N_ACTIONS, device=device, generator=gen) < 0.08
    action = torch.randint(0, N_ACTIONS, shape, device=device, generator=gen)
    legal.scatter_(-1, action.unsqueeze(-1), True)  # the action taken is legal
    return {
        "legal": legal,
        "action": action,
        "old_logp": -2.0 * torch.rand(*shape, device=device, generator=gen),
        "adv": torch.randn(*shape, device=device, generator=gen),
        "ret": torch.randn(*shape, device=device, generator=gen),
    }


def make_mlp_batch(batch, device, gen):
    data = make_targets((batch,), device, gen)
    data["state"] = torch.rand(batch, STATE_FEATURES, device=device, generator=gen)
    return data


def make_l1_batch(batch, device, gen, sort: bool):
    """Decisions with prefix lengths spread evenly over MIN_PREFIX..MAX_PLAY."""
    data = make_mlp_batch(batch, device, gen)
    lengths = torch.randint(MIN_PREFIX, MAX_PLAY + 1, (batch,), device=device, generator=gen)
    if sort:
        lengths = lengths.sort().values
    data["lengths"] = lengths
    data["tokens"] = torch.randint(0, VOCAB, (batch, MAX_PLAY), device=device, generator=gen)
    return data


def make_l2_batch(batch, device, gen):
    """Deals of one seat: MAX_PLAY play tokens and N_STATE decisions at DECISION_POINTS."""
    data = make_targets((batch, N_STATE), device, gen)
    data["tokens"] = torch.randint(0, VOCAB, (batch, MAX_PLAY), device=device, generator=gen)
    data["state"] = torch.rand(batch, N_STATE, STATE_FEATURES, device=device, generator=gen)
    data["points"] = torch.tensor(DECISION_POINTS, device=device).expand(batch, N_STATE).contiguous()
    return data


def ppo_loss(logits, q, value, data):
    """PPO-like loss: clipped surrogate, value and Q regression, entropy bonus. Computed in fp32."""
    logp_all = F.log_softmax(logits.float(), dim=-1)
    action = data["action"].unsqueeze(-1)
    logp = logp_all.gather(-1, action).squeeze(-1)
    ratio = torch.exp(logp - data["old_logp"])
    adv = data["adv"]
    surrogate = torch.minimum(ratio * adv, ratio.clamp(0.8, 1.2) * adv)
    entropy = -(logp_all.exp() * logp_all.masked_fill(~data["legal"], 0.0)).sum(-1)
    value_loss = (value.float() - data["ret"]) ** 2
    q_loss = (q.float().gather(-1, action).squeeze(-1) - data["ret"]) ** 2
    return (-surrogate + 0.5 * value_loss + 0.5 * q_loss - 0.01 * entropy).mean()


def sample_action(logits):
    return torch.multinomial(F.softmax(logits.float(), dim=-1), 1)


# --------------------------------------------------------------------------
# Timed steps. Each builder returns (step, decisions_per_step).
# --------------------------------------------------------------------------


def _autocast(device, amp):
    return torch.autocast(device_type=device.type, dtype=torch.float16, enabled=amp)


def _forward(net, layout, data):
    if layout == "mlp":
        return net(data["state"], data["legal"])
    if layout == "L1 padded":
        return net.forward_l1(data["tokens"], data["lengths"], data["state"], data["legal"])
    if layout == "L1 bucketed":
        return net.forward_l1_bucketed(data["tokens"], data["lengths"], data["state"], data["legal"])
    if layout == "L2":
        return net.forward_l2(data["tokens"], data["points"], data["state"], data["legal"])
    raise ValueError(layout)


def _make_batch(layout, batch, device, gen):
    if layout == "mlp":
        return make_mlp_batch(batch, device, gen), batch
    if layout in ("L1 padded", "L1 bucketed"):
        return make_l1_batch(batch, device, gen, sort=layout == "L1 bucketed"), batch
    if layout == "L2":
        return make_l2_batch(batch, device, gen), batch * N_STATE
    raise ValueError(layout)


def build_act_step(net, layout, batch, device, amp, gen):
    """Acting: forward without gradients and one sampled action per decision.

    MLP and L1: the actions are copied to the host, as an actor must hand them to
    the engine. L2: the idealised cached row; build_ragged_act_step is the other one.
    """
    data, decisions = _make_batch(layout, batch, device, gen)
    net.eval()

    if layout != "L2":

        def step():
            with torch.inference_mode(), _autocast(device, amp):
                logits, _, _ = _forward(net, layout, data)
                sample_action(logits).cpu()

        return step, decisions

    # Idealised cached acting: one step plays a whole deal for the batch, every deal
    # at the same decision, the actions left on the GPU.
    cache = KVCache(batch, device, torch.float16 if amp else torch.float32)

    def deal():
        cache.reset()
        with torch.inference_mode(), _autocast(device, amp):
            for k, point in enumerate(DECISION_POINTS):
                new_tokens = data["tokens"][:, cache.filled : point]
                logits, _, _ = net.act_l2_cached(cache, new_tokens, data["state"][:, k], data["legal"][:, k])
                sample_action(logits)

    return deal, decisions


def build_ragged_act_step(net, layout, batch, device, amp, gen):
    """Cached acting with deals at different decisions: one decision per deal and step.

    Deal b starts at decision b mod N_STATE. A finished deal starts again at once.
    Per step: gather each deal's inputs, compute, sample, copy the actions to the
    host, advance the per-deal counters.
    """
    if layout != "L2":
        raise ValueError("ragged acting is the cached transformer row only")
    data, _ = _make_batch("L2", batch, device, gen)
    net.eval()
    cache = KVCache(batch, device, torch.float16 if amp else torch.float32)
    points = data["points"]
    n_state = points.shape[1]
    n_max = int(torch.diff(points[0], prepend=points.new_zeros(1)).max())
    k = torch.arange(batch, device=device) % n_state  # decision index of each deal
    t0 = torch.where(k > 0, points[0][(k - 1).clamp(min=0)], 0)  # play tokens already cached
    state_shape = (batch, 1, data["state"].shape[-1])
    legal_shape = (batch, 1, data["legal"].shape[-1])

    def step():
        with torch.inference_mode(), _autocast(device, amp):
            point = points.gather(1, k[:, None]).squeeze(1)
            state = data["state"].gather(1, k[:, None, None].expand(state_shape)).squeeze(1)
            legal = data["legal"].gather(1, k[:, None, None].expand(legal_shape)).squeeze(1)
            logits, _, _ = net.act_l2_ragged(cache, data["tokens"], t0, point - t0, state, legal, n_max)
            action = sample_action(logits).cpu()
            done = k + 1 >= n_state
            t0.copy_(torch.where(done, 0, point))
            k.copy_(torch.where(done, 0, k + 1))
        return action

    return step, batch


def build_update_step(net, layout, batch, device, amp, gen):
    """Update: forward, loss, backward, gradient clipping, optimizer step."""
    data, decisions = _make_batch(layout, batch, device, gen)
    net.train()
    opt = torch.optim.Adam(net.parameters(), lr=1e-4)
    scaler = torch.amp.GradScaler(device.type, enabled=amp)
    last = {}

    def step():
        opt.zero_grad(set_to_none=True)
        with _autocast(device, amp):
            logits, q, value = _forward(net, layout, data)
        loss = ppo_loss(logits, q, value, data)
        scaler.scale(loss).backward()
        scaler.unscale_(opt)
        nn.utils.clip_grad_norm_(net.parameters(), 0.5)
        scaler.step(opt)
        scaler.update()
        last["loss"] = loss.detach()

    step.last = last
    return step, decisions


BUILDERS = {"act": build_act_step, "act ragged": build_ragged_act_step, "update": build_update_step}


def time_step(step, device, seconds: float, repeats: int = 3, warmup: int = 2, min_iters: int = 3) -> list:
    """Seconds per call of step(), one value per repeat.

    Warm-up first. Each repeat runs for at least `seconds` and at least `min_iters`
    calls. CUDA is synchronised around every call.
    """
    cuda = device.type == "cuda"
    for _ in range(warmup):
        step()
    if cuda:
        torch.cuda.synchronize()
    per_call = []
    for _ in range(repeats):
        iters = 0
        start = time.perf_counter()
        while True:
            step()
            if cuda:
                torch.cuda.synchronize()
            iters += 1
            elapsed = time.perf_counter() - start
            if iters >= min_iters and elapsed >= seconds:
                break
        per_call.append(elapsed / iters)
    return per_call


# --------------------------------------------------------------------------
# The benchmark
# --------------------------------------------------------------------------

NETWORKS = {
    "mlp": ("MLP 512-512-256", MLPNet),
    "L1 padded": ("transformer", TransformerNet),
    "L1 bucketed": ("transformer", TransformerNet),
    "L2": ("transformer", TransformerNet),
}

# Batch sizes tried: (fp32 sizes, fp16 sizes). For L2 "act" and "update" the unit is
# deals of one seat (N_STATE decisions each); for "act ragged" it is deals in flight.
# Sizes that do not fit in the VRAM budget are skipped.
GPU_BATCHES = {
    ("mlp", "act"): ((4096, 16384, 65536), (4096, 65536)),
    ("mlp", "update"): ((1024, 4096, 16384, 65536), (4096, 16384)),
    ("L1 padded", "act"): ((128, 512, 2048), (128, 512)),
    ("L1 padded", "update"): ((64, 128, 256, 512), (32, 128)),
    ("L1 bucketed", "act"): ((128, 512, 2048), (128, 512)),
    ("L1 bucketed", "update"): ((64, 128, 256, 512), (32, 128)),
    ("L2", "act"): ((64, 256, 1024), (64, 1024)),
    ("L2", "act ragged"): ((256, 1024, 4096), (256, 4096)),
    ("L2", "update"): ((16, 64, 256, 512), (16, 64)),
}
QUICK_BATCHES = {key: ((4,), (4,)) for key in GPU_BATCHES}


def gpu_snapshot() -> dict:
    """GPU load and memory right now, from nvidia-smi. Empty if nvidia-smi is missing.

    Numbers only. Process names are not recorded: the result files are committed.
    """
    try:
        line = subprocess.run(
            [
                "nvidia-smi",
                "--query-gpu=utilization.gpu,memory.used,memory.total,temperature.gpu",
                "--format=csv,noheader,nounits",
            ],
            capture_output=True,
            text=True,
            timeout=20,
        ).stdout.strip()
        util, used, total, temp = (int(part) for part in line.split(","))
        return {
            "utilization_percent": util,
            "memory_used_mib": used,
            "memory_total_mib": total,
            "temperature_c": temp,
        }
    except (OSError, ValueError, subprocess.SubprocessError):
        return {}


def end_to_end(act_per_s: float, update_per_s: float, epochs: int) -> float:
    """Steps per second when one GPU acts once and learns `epochs` times on every step.

    Time per step = 1/act_per_s + epochs/update_per_s. The engine is not counted.
    """
    return 1.0 / (1.0 / act_per_s + epochs / update_per_s)


def _measure_gpu(layout, mode, precision, batches, device, seconds, repeats, seed):
    rows = []
    amp = precision == "fp16 autocast"
    label, cls = NETWORKS[layout]
    for batch in batches:
        row = {"network": label, "layout": layout, "precision": precision, "mode": mode, "batch": batch}
        net = step = None
        try:
            gen = torch.Generator(device=device).manual_seed(seed)
            torch.manual_seed(seed)
            net = cls().to(device)
            step, decisions = BUILDERS[mode](net, layout, batch, device, amp, gen)
            if device.type == "cuda":
                torch.cuda.reset_peak_memory_stats()
            per_call = time_step(step, device, seconds, repeats)
            rates = [decisions / t for t in per_call]
            row["decisions_per_call"] = decisions
            row["per_s"] = statistics.median(rates)
            row["per_s_min"] = min(rates)
            row["per_s_max"] = max(rates)
            if device.type == "cuda":
                row["peak_mib"] = torch.cuda.max_memory_allocated() / MIB
            if mode == "update":
                row["loss_finite"] = bool(torch.isfinite(step.last["loss"]).item())
        except torch.OutOfMemoryError:
            row["skipped"] = "out of memory within the VRAM budget"
        del net, step
        gc.collect()
        if device.type == "cuda":
            torch.cuda.empty_cache()
        rows.append(row)
        print(_progress_line(row), flush=True)
    return rows


def _progress_line(row) -> str:
    head = f"  {row['network']:<16} {row['layout']:<12} {row['precision']:<14} {row['mode']:<10} batch {row['batch']:>6}"
    if "skipped" in row:
        return f"{head}  skipped: {row['skipped']}"
    if row["mode"] == "cpu":
        return (
            f"{head}  {row['ms_per_decision']:.3f} ms per decision "
            f"({row['ms_min']:.3f} to {row['ms_max']:.3f})"
        )
    tail = f"  {row['per_s']:>11,.0f} /s  ({row['per_s_min']:,.0f} to {row['per_s_max']:,.0f})"
    if "peak_mib" in row:
        tail += f"  peak {row['peak_mib']:,.0f} MiB"
    if row.get("loss_finite") is False:
        tail += "  LOSS NOT FINITE"
    return head + tail


def _measure_cpu(seconds, repeats, seed):
    """Single-thread CPU forward time per decision, batch 1, fp32."""
    device = torch.device("cpu")
    threads_before = torch.get_num_threads()
    torch.set_num_threads(1)
    rows = []
    try:
        torch.manual_seed(seed)
        gen = torch.Generator().manual_seed(seed)

        def add(label, layout, step, decisions, note):
            ms = [1000.0 * t / decisions for t in time_step(step, device, seconds, repeats)]
            row = {
                "network": label,
                "layout": layout,
                "precision": "fp32",
                "mode": "cpu",
                "batch": 1,
                "ms_per_decision": statistics.median(ms),
                "ms_min": min(ms),
                "ms_max": max(ms),
                "note": note,
            }
            rows.append(row)
            print(_progress_line(row), flush=True)

        mlp = MLPNet().eval()
        mlp_data = make_mlp_batch(1, device, gen)

        def mlp_step():
            with torch.inference_mode():
                mlp(mlp_data["state"], mlp_data["legal"])

        add("MLP 512-512-256", "mlp", mlp_step, 1, "one decision")

        net = TransformerNet().eval()
        data = make_l1_batch(1, device, gen, sort=False)

        def l1_at(lengths):
            def step():
                with torch.inference_mode():
                    for length in lengths:
                        net.forward_l1(data["tokens"], torch.tensor([length]), data["state"], data["legal"])

            return step

        add("transformer", "L1", l1_at(DECISION_POINTS), N_STATE, "mean over the decisions of a deal, no padding")
        add("transformer", "L1 longest", l1_at((MAX_PLAY,)), 1, "the longest prefix")

        l2_step, decisions = build_act_step(net, "L2", 1, device, False, gen)
        add("transformer", "L2", l2_step, decisions, "mean over the decisions of a deal, cached keys and values")
    finally:
        torch.set_num_threads(threads_before)
    return rows


def run(
    label: str = "unlabelled",
    seconds: float = 1.0,
    repeats: int = 3,
    quick: bool = False,
    device: str | None = None,
    precisions: tuple = ("fp32", "fp16 autocast"),
) -> dict:
    """Run the benchmark and return {meta, rows, best, estimates}."""
    dev = torch.device(device or ("cuda" if torch.cuda.is_available() else "cpu"))
    meta = {
        "label": label,
        "date": datetime.datetime.now().isoformat(timespec="seconds"),
        "torch": torch.__version__,
        "device": torch.cuda.get_device_name(0) if dev.type == "cuda" else "cpu",
        "seconds_per_repeat": seconds,
        "repeats": repeats,
        "quick": quick,
        "shape": shape(),
        "decision_points": list(DECISION_POINTS),
        "parameters": {
            "transformer": count_parameters(TransformerNet()),
            "MLP 512-512-256": count_parameters(MLPNet()),
        },
    }
    if dev.type != "cuda":
        precisions = ("fp32",)
    print(f"benchmark '{label}' on {meta['device']}, torch {meta['torch']}")
    print(f"shape: {meta['shape']}")
    print(f"parameters: {meta['parameters']}")
    print(f"timing: {repeats} repeats of at least {seconds} s and 3 calls; median (min to max)")

    batches = QUICK_BATCHES if quick else GPU_BATCHES
    rows = []
    try:
        if dev.type == "cuda":
            meta["gpu_before"] = gpu_snapshot()
            torch.ones(8, 8, device=dev) @ torch.ones(8, 8, device=dev)  # load the CUDA libraries first
            torch.cuda.synchronize()
            free, total = torch.cuda.mem_get_info()
            # Windows lets a process spill into shared memory instead of failing, which is very
            # slow. A hard cap below the free VRAM turns that into a clean out-of-memory skip.
            budget = max(free - 200 * MIB, 64 * MIB)
            torch.cuda.set_per_process_memory_fraction(min(1.0, budget / total))
            meta["vram_free_mib"] = free // MIB
            meta["vram_total_mib"] = total // MIB
            meta["vram_budget_mib"] = budget // MIB
            snap = meta["gpu_before"]
            if snap:
                print(
                    f"GPU before the run: {snap['utilization_percent']}% busy, "
                    f"{snap['memory_used_mib']} of {snap['memory_total_mib']} MiB used; "
                    f"budget for this run {meta['vram_budget_mib']} MiB"
                )
        for (layout, mode), (sizes_fp32, sizes_fp16) in batches.items():
            for precision in precisions:
                sizes = sizes_fp32 if precision == "fp32" else sizes_fp16
                rows += _measure_gpu(layout, mode, precision, sizes, dev, seconds, repeats, seed=3)
        rows += _measure_cpu(seconds, repeats, seed=3)
        if dev.type == "cuda":
            meta["gpu_after"] = gpu_snapshot()
    finally:
        if dev.type == "cuda":
            torch.cuda.set_per_process_memory_fraction(1.0)  # lift the cap again

    best = summarise(rows)
    return {"meta": meta, "rows": rows, "best": best, "estimates": estimate(best)}


def summarise(rows) -> list[dict]:
    """The fastest batch size (by median) per layout, precision and mode."""
    out = []
    seen = []
    for r in rows:
        key = (r["layout"], r["precision"], r["mode"])
        if r["mode"] != "cpu" and key not in seen:
            seen.append(key)
    for key in seen:
        ok = [r for r in rows if (r["layout"], r["precision"], r["mode"]) == key and "per_s" in r]
        if ok:
            out.append(dict(max(ok, key=lambda r: r["per_s"])))
    return out


# Which acting row and which update row make one end-to-end estimate.
ESTIMATES = (
    ("MLP 512-512-256", ("mlp", "act"), ("mlp", "update")),
    ("L1 padded, acting uncached", ("L1 padded", "act"), ("L1 padded", "update")),
    ("L1 bucketed, acting uncached", ("L1 bucketed", "act"), ("L1 bucketed", "update")),
    ("L1 bucketed update, acting cached (ragged)", ("L2", "act ragged"), ("L1 bucketed", "update")),
    ("L2, acting cached (idealised)", ("L2", "act"), ("L2", "update")),
    ("L2, acting cached (ragged)", ("L2", "act ragged"), ("L2", "update")),
)


def estimate(best) -> list[dict]:
    """End-to-end steps/s for 4 and 2 epochs, with the range from the repeats' min and max."""
    out = []
    by_key = {(b["layout"], b["mode"], b["precision"]): b for b in best}
    for precision in ("fp32", "fp16 autocast"):
        for name, act_key, update_key in ESTIMATES:
            act = by_key.get((*act_key, precision))
            update = by_key.get((*update_key, precision))
            if act is None or update is None:
                continue
            line = {"case": name, "precision": precision}
            for epochs in (4, 2):
                line[f"steps_per_s_{epochs}_epochs"] = end_to_end(act["per_s"], update["per_s"], epochs)
                line[f"steps_per_s_{epochs}_epochs_min"] = end_to_end(act["per_s_min"], update["per_s_min"], epochs)
                line[f"steps_per_s_{epochs}_epochs_max"] = end_to_end(act["per_s_max"], update["per_s_max"], epochs)
            out.append(line)
    return out


MODE_NAMES = {
    "act": "acting",
    "act ragged": "acting, ragged, host copy",
    "update": "update",
}


def markdown(result) -> str:
    """The best rows, the CPU rows and the estimate as Markdown tables."""
    lines = [
        "| Network | Layout | Measured | Precision | Batch | Per second, median (min to max) | Peak VRAM (MiB) |",
        "|---|---|---|---|---|---|---|",
    ]
    for b in result["best"]:
        what = MODE_NAMES[b["mode"]]
        if b["layout"] == "L2" and b["mode"] == "act":
            what = "acting, cached, idealised"
        unit = " deals" if b["layout"] == "L2" else ""
        peak = f"{b['peak_mib']:,.0f}" if "peak_mib" in b else "n/a"
        lines.append(
            f"| {b['network']} | {b['layout']} | {what} | {b['precision']} | {b['batch']:,}{unit} "
            f"| {b['per_s']:,.0f} ({b['per_s_min']:,.0f} to {b['per_s_max']:,.0f}) | {peak} |"
        )
    lines += ["", "| Network | Layout | CPU ms per decision, median (min to max) | Note |", "|---|---|---|---|"]
    for r in result["rows"]:
        if r["mode"] == "cpu":
            lines.append(
                f"| {r['network']} | {r['layout']} | {r['ms_per_decision']:.3f} "
                f"({r['ms_min']:.3f} to {r['ms_max']:.3f}) | {r['note']} |"
            )
    lines += [
        "",
        "| Case | Precision | Steps/s, 4 epochs (derived) | Steps/s, 2 epochs (derived) |",
        "|---|---|---|---|",
    ]
    for e in result["estimates"]:
        cells = [
            f"{e[f'steps_per_s_{n}_epochs']:,.0f} "
            f"({e[f'steps_per_s_{n}_epochs_min']:,.0f} to {e[f'steps_per_s_{n}_epochs_max']:,.0f})"
            for n in (4, 2)
        ]
        lines.append(f"| {e['case']} | {e['precision']} | {cells[0]} | {cells[1]} |")
    return "\n".join(lines)


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--label", default="unlabelled", help='what else used the GPU, e.g. "as used daily"')
    parser.add_argument("--seconds", type=float, default=1.0, help="least timed seconds per repeat")
    parser.add_argument("--repeats", type=int, default=3, help="repeats per row; median, min and max are kept")
    parser.add_argument("--quick", action="store_true", help="tiny batches, for a smoke test")
    parser.add_argument("--device", default=None, help="cuda or cpu; default: cuda when available")
    parser.add_argument("--precisions", default="fp32,fp16", help="fp32, fp16 or both (default)")
    parser.add_argument("--state-features", type=int, default=None, help=f"default {STATE_FEATURES}")
    parser.add_argument("--play-tokens", type=int, default=None, help=f"play and bid tokens per deal, default {MAX_PLAY}")
    parser.add_argument("--decisions", type=int, default=None, help=f"decisions per seat and deal, default {N_STATE}")
    parser.add_argument("--actions", type=int, default=None, help=f"default {N_ACTIONS}")
    parser.add_argument("--vocab", type=int, default=None, help=f"token ids, default {VOCAB}")
    parser.add_argument("--out", default=None, help="JSON file for the raw rows")
    args = parser.parse_args(argv)

    set_shape(args.state_features, args.play_tokens, args.decisions, args.actions, args.vocab)
    names = {"fp32": "fp32", "fp16": "fp16 autocast"}
    precisions = tuple(names[p.strip()] for p in args.precisions.split(","))

    started = time.perf_counter()
    result = run(args.label, args.seconds, args.repeats, args.quick, args.device, precisions)
    result["meta"]["wall_seconds"] = round(time.perf_counter() - started, 1)

    if args.out:
        out = Path(args.out)
    else:
        slug = re.sub(r"[^a-z0-9]+", "-", args.label.lower()).strip("-") or "run"
        stamp = datetime.datetime.now().strftime("%Y%m%d-%H%M%S")
        out = Path(__file__).resolve().parents[1] / "bench_results" / f"{stamp}-{slug}.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(result, indent=2), encoding="utf-8")

    print()
    print(markdown(result))
    print()
    print("Formula: steps/s = 1 / (1/acting + epochs/update). The engine is not counted.")
    print(f"Raw rows: {out.name} in {out.parent.name}")
    print(f"Wall time: {result['meta']['wall_seconds']} s")
    return 0


if __name__ == "__main__":
    sys.exit(main())
