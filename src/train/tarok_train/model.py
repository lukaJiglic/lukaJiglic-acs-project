"""The networks of spec 007: the transformer in layout L2, the MLP control, the critic.

Layout L2 (D116): one sequence per seat and deal. Play and bid tokens attend
causally to each other only. Every decision has a state token that attends to
the tokens before its decision and to itself. Training runs all decisions of a
seat's deal in one pass (`forward_deals`). Acting keeps the tokens' keys and
values in a cache, one per table and seat (`Actor`).

The actor networks take tokens, state and mask. They have no argument for the
critic's hidden features (criterion 7). The critic is a separate module.

All sizes come from `contract.py` (the encoding) and from the config (the network).
"""

from __future__ import annotations

from typing import NamedTuple

import numpy as np
import torch
import torch.nn as nn
import torch.nn.functional as F

from . import contract as C


class Outputs(NamedTuple):
    """What the heads give for a batch of decisions. The leading shape is the batch's."""

    logits: torch.Tensor  # (..., 160) policy logits, -inf at illegal actions
    q: torch.Tensor  # (..., 160) value of each action, from the observation only (D15)
    value: torch.Tensor  # (..., parts) the actor's own value, one output per score part
    partner: torch.Tensor  # (..., 4) partner seat, one logit per relative seat
    cards: torch.Tensor  # (..., 54, 4) card location


def mask_logits(raw, mask):
    """Illegal actions get -inf: probability exactly 0, and no gradient to their logit."""
    return raw.masked_fill(~mask, float("-inf"))


class Heads(nn.Module):
    """All heads, read at the state token (transformer) or at the trunk output (MLP)."""

    def __init__(self, width: int, policy_init_scale: float):
        super().__init__()
        self.policy = nn.Linear(width, C.N_ACTIONS)
        self.q = nn.Linear(width, C.N_ACTIONS)
        self.value = nn.Linear(width, C.N_SCORE_PARTS)
        self.partner = nn.Linear(width, C.PARTNER_CLASSES)
        self.cards = nn.Linear(width, C.N_CARDS * C.CARD_LOCATION_CLASSES)
        with torch.no_grad():
            self.policy.weight.mul_(policy_init_scale)
            self.policy.bias.zero_()

    def forward(self, h, mask) -> Outputs:
        cards = self.cards(h)
        return Outputs(
            logits=mask_logits(self.policy(h), mask),
            q=self.q(h),
            value=self.value(h),
            partner=self.partner(h),
            cards=cards.view(*cards.shape[:-1], C.N_CARDS, C.CARD_LOCATION_CLASSES),
        )


class Block(nn.Module):
    """Pre-norm transformer layer. Attention is split in two steps so a cache can sit between."""

    def __init__(self, width: int, heads: int, d_ff: int):
        super().__init__()
        self.heads = heads
        self.norm1 = nn.LayerNorm(width)
        self.qkv = nn.Linear(width, 3 * width)
        self.proj = nn.Linear(width, width)
        self.norm2 = nn.LayerNorm(width)
        self.ff1 = nn.Linear(width, d_ff)
        self.ff2 = nn.Linear(d_ff, width)

    def qkv_of(self, x):
        b, t, d = x.shape
        qkv = self.qkv(self.norm1(x)).view(b, t, 3, self.heads, d // self.heads)
        q, k, v = qkv.permute(2, 0, 3, 1, 4).unbind(0)  # each (b, heads, t, d/heads)
        return q, k, v

    def finish(self, x, attn):
        b, h, t, dh = attn.shape
        x = x + self.proj(attn.transpose(1, 2).reshape(b, t, h * dh))
        return x + self.ff2(F.gelu(self.ff1(self.norm2(x))))

    def forward(self, x, mask=None, causal=False):
        q, k, v = self.qkv_of(x)
        attn = F.scaled_dot_product_attention(q, k, v, attn_mask=mask, is_causal=causal)
        return self.finish(x, attn)


def l2_mask(points, n_play: int):
    """Attention mask for layout L2. True = may attend. Shape (batch, 1, T, T), T = n_play + n_state.

    Rows and columns: first the n_play play tokens, then one state token per decision.
    points[b, k] is the number of play tokens that decision k of deal b may see.
    """
    b, s = points.shape
    t = n_play + s
    dev = points.device
    mask = torch.zeros(b, t, t, dtype=torch.bool, device=dev)
    mask[:, :n_play, :n_play] = torch.tril(torch.ones(n_play, n_play, dtype=torch.bool, device=dev))
    mask[:, n_play:, :n_play] = torch.arange(n_play, device=dev)[None, None, :] < points[:, :, None]
    mask[:, n_play:, n_play:] = torch.eye(s, dtype=torch.bool, device=dev)
    return mask[:, None]


class KVCache:
    """Keys and values of the tokens seen so far, per layer, one row per table and seat.

    Position p of a row holds token p of that seat's view of its current deal. The
    position after the last token holds the state token of the current decision; the
    next token overwrites it. The last position is a dump for padding rows. The
    cache starts small and grows to the longest sequence met.
    """

    def __init__(self, n_slots: int, layers: int, heads: int, head_dim: int, device, initial_tokens: int):
        self.shape = (n_slots, heads)
        self.head_dim = head_dim
        self.device = device
        self.capacity = 0
        self.k = [None] * layers
        self.v = [None] * layers
        self.ensure(initial_tokens)

    def ensure(self, positions: int) -> None:
        """Make room for `positions` positions (tokens, the state token and the dump)."""
        limit = C.MAX_TOKENS + 2
        if positions > limit:
            raise ValueError(f"a sequence of {positions - 2} tokens is over the limit of {C.MAX_TOKENS}")
        if positions <= self.capacity:
            return
        new = min(limit, max(positions, 2 * self.capacity))
        for store in (self.k, self.v):
            for i, old in enumerate(store):
                grown = torch.zeros(*self.shape, new, self.head_dim, device=self.device)
                if old is not None:
                    grown[:, :, : self.capacity] = old
                store[i] = grown
        self.capacity = new


class TransformerNet(nn.Module):
    """The primary network (D18, D30): tokens and one state token per decision, layout L2."""

    def __init__(self, width: int, layers: int, heads: int, feed_forward: int, policy_init_scale: float):
        super().__init__()
        self.width = width
        self.n_heads = heads
        # One learned embedding per token field; a token is their sum (D30).
        self.fields = nn.ModuleList(nn.Embedding(size, width) for size in C.TOKEN_FIELD_SIZES)
        self.state_proj = nn.Linear(C.STATE_FEATURES, width)
        self.state_type = nn.Parameter(torch.zeros(width))
        nn.init.normal_(self.state_type, std=0.02)
        self.blocks = nn.ModuleList(Block(width, heads, feed_forward) for _ in range(layers))
        self.norm = nn.LayerNorm(width)
        self.heads = Heads(width, policy_init_scale)

    def embed_tokens(self, tokens):
        """tokens (..., 5) integer fields -> (..., width)."""
        return sum(table(tokens[..., i]) for i, table in enumerate(self.fields))

    def embed_state(self, state):
        return self.state_proj(state) + self.state_type

    # ---- training: all decisions of a seat's deal in one pass ----

    def trunk_deals(self, tokens, points, state):
        """Hidden states after the last layer, (B, P + S, width): P play tokens, then S state tokens."""
        p = tokens.shape[1]
        x = torch.cat([self.embed_tokens(tokens), self.embed_state(state)], dim=1)
        mask = l2_mask(points, p)
        for block in self.blocks:
            x = block(x, mask=mask)
        return x

    def forward_deals(self, tokens, points, state, mask) -> Outputs:
        """tokens (B, P, 5), points (B, S), state (B, S, 1382), mask (B, S, 160)."""
        p = tokens.shape[1]
        x = self.trunk_deals(tokens, points, state)
        return self.heads(self.norm(x[:, p:]), mask)

    # ---- reference: one decision alone from its prefix ----

    def forward_prefix(self, tokens, length, state, mask) -> Outputs:
        """One decision per row, computed from the first length[b] tokens and the state token.

        tokens (B, P, 5), length (B,), state (B, 1382), mask (B, 160). The state
        token sits at index length[b]; the causal mask hides everything after it.
        """
        b = tokens.shape[0]
        longest = int(length.max())
        x = self.embed_tokens(tokens[:, :longest])
        x = torch.cat([x, x.new_zeros(b, 1, self.width)], dim=1)
        where = length.view(b, 1, 1).expand(b, 1, self.width)
        x = x.scatter(1, where, self.embed_state(state).unsqueeze(1))
        for block in self.blocks:
            x = block(x, causal=True)
        return self.heads(self.norm(x.gather(1, where).squeeze(1)), mask)

    # ---- acting: cached keys and values, rows at different points ----

    def act_cached(self, cache: KVCache, slots, tokens, t0, n, state, mask, n_max: int, last: int) -> Outputs:
        """One decision per row. Row b is the seat with cache row slots[b].

        tokens (B, 128, 5): the seat's whole token array. t0 (B,): tokens already in
        the cache. n (B,): new tokens of this step. n_max = max(n) and last = max(t0 + n)
        come from the caller, who knows them without asking the GPU. Per row the step
        computes the new tokens, padding up to n_max, and the state token.
        """
        b = slots.shape[0]
        dev = tokens.device
        t1 = t0 + n
        cache.ensure(last + 2)
        offset = torch.arange(n_max, device=dev)
        position = t0[:, None] + offset[None, :]  # (B, n_max): place of each token row in the deal
        is_new = offset[None, :] < n[:, None]
        source = position.clamp(max=tokens.shape[1] - 1)
        new_tokens = tokens.gather(1, source[:, :, None].expand(b, n_max, tokens.shape[2]))
        x = torch.cat([self.embed_tokens(new_tokens), self.embed_state(state).unsqueeze(1)], dim=1)
        # Where each row's key and value go: its own position, the state position t1, or the dump.
        place = torch.cat([torch.where(is_new, position, cache.capacity - 1), t1[:, None]], dim=1)
        # Row r may attend to positions 0..reach[r]: a token to itself and earlier ones,
        # the state token to every token so far and to itself.
        reach = torch.cat([position, t1[:, None]], dim=1)
        attend = (torch.arange(last + 1, device=dev)[None, None, :] <= reach[:, :, None])[:, None]
        rows = slots[:, None]
        for i, block in enumerate(self.blocks):
            q, k, v = block.qkv_of(x)  # (B, heads, n_max + 1, head_dim)
            cache.k[i][rows, :, place] = k.transpose(1, 2)
            cache.v[i][rows, :, place] = v.transpose(1, 2)
            keys = cache.k[i][:, :, : last + 1][slots]
            values = cache.v[i][:, :, : last + 1][slots]
            x = block.finish(x, F.scaled_dot_product_attention(q, keys, values, attn_mask=attend))
        return self.heads(self.norm(x[:, n_max]), mask)


class MLPNet(nn.Module):
    """The control (D18): state features -> hidden layers -> the same heads. It reads no tokens."""

    def __init__(self, hidden: list, policy_init_scale: float):
        super().__init__()
        layers = []
        width = C.STATE_FEATURES
        for size in hidden:
            layers += [nn.Linear(width, size), nn.ReLU()]
            width = size
        self.trunk = nn.Sequential(*layers)
        self.heads = Heads(width, policy_init_scale)

    def forward(self, state, mask) -> Outputs:
        return self.heads(self.trunk(state), mask)

    def forward_deals(self, tokens, points, state, mask) -> Outputs:
        return self.forward(state, mask)


class Critic(nn.Module):
    """Full-information critic (D14, D119 B): an MLP over the critic features. Training only."""

    def __init__(self, hidden: list):
        super().__init__()
        layers = []
        width = C.CRITIC_FEATURES
        for size in hidden:
            layers += [nn.Linear(width, size), nn.ReLU()]
            width = size
        layers.append(nn.Linear(width, C.N_SCORE_PARTS))
        self.net = nn.Sequential(*layers)

    def forward(self, critic_features):
        """(..., 1656) -> (..., parts): the value of the position, one output per score part."""
        return self.net(critic_features)


def build_network(cfg_model: dict, kind: str) -> nn.Module:
    if kind == "transformer":
        return TransformerNet(
            cfg_model["width"],
            cfg_model["layers"],
            cfg_model["heads"],
            cfg_model["feed_forward"],
            cfg_model["policy_init_scale"],
        )
    if kind == "mlp":
        return MLPNet(cfg_model["mlp_hidden"], cfg_model["policy_init_scale"])
    raise ValueError(f"unknown network '{kind}': use 'transformer' or 'mlp'")


def build_critic(cfg_model: dict) -> Critic:
    return Critic(cfg_model["critic_hidden"])


def count_parameters(module: nn.Module) -> int:
    return sum(p.numel() for p in module.parameters())


class Actor:
    """Chooses actions for a batch of tables. Holds the acting cache of the transformer.

    `net` plays the seats that are trained. `frozen_net` plays the seats given to a
    past checkpoint (it may be None). A table's seat is played by one of the two
    for a whole deal, so both share one cache. The bookkeeping (which seat has how
    many tokens in the cache) is kept on the CPU, so no step waits for the GPU to
    answer a question.
    """

    def __init__(self, net, frozen_net, n_tables: int, device, cfg_model: dict):
        self.net = net
        self.frozen_net = frozen_net
        self.device = device
        self.cached = isinstance(net, TransformerNet)
        self.bucket = cfg_model["act_bucket_tokens"]
        self.cache = None
        if self.cached:
            n_slots = n_tables * 4
            self.cache = KVCache(
                n_slots,
                len(net.blocks),
                net.n_heads,
                net.width // net.n_heads,
                device,
                cfg_model["cache_initial_tokens"] + 2,
            )
            self.t0 = np.zeros(n_slots, dtype=np.int64)  # tokens in the cache
            self.deal = np.full(n_slots, -1, dtype=np.int64)  # the deal they belong to

    def reset(self) -> None:
        """Forget the cache. Called after an update: the weights that made it have changed."""
        if self.cached:
            self.t0[:] = 0

    @torch.inference_mode()
    def act(self, tables, seat, deal, n_tokens, frozen_rows, tokens, state, mask, generator):
        """One decision per row.

        numpy, (B,): tables, seat, deal, n_tokens, frozen_rows (bool: rows played by
        the frozen network). Tensors on the device: tokens (B, 128, 5), state (B, 1382),
        mask (B, 160). Returns (action (B,), log-probability (B,), Outputs).
        """
        batch = len(tables)
        groups = [(self.net, ~frozen_rows)]
        if self.frozen_net is not None and frozen_rows.any():
            groups.append((self.frozen_net, frozen_rows))
        if self.cached:
            slots = tables * 4 + seat
            t0 = np.where(self.deal[slots] == deal, self.t0[slots], 0)
            n = n_tokens - t0
            few = n <= self.bucket
            groups = [(net, rows & part) for net, rows in groups for part in (few, ~few)]
            slots_d, t0_d, n_d = (torch.from_numpy(a).to(self.device) for a in (slots, t0, n))

        parts = []
        for net, rows in groups:
            index = np.flatnonzero(rows)
            if len(index) == 0:
                continue
            pick = slice(None) if len(index) == batch else torch.from_numpy(index).to(self.device)
            if self.cached:
                out = net.act_cached(
                    self.cache,
                    slots_d[pick],
                    tokens[pick],
                    t0_d[pick],
                    n_d[pick],
                    state[pick],
                    mask[pick],
                    n_max=int(n[index].max()),
                    last=int(n_tokens[index].max()),
                )
            else:
                out = net(state[pick], mask[pick])
            parts.append((pick, out))
        if self.cached:
            self.t0[slots] = n_tokens
            self.deal[slots] = deal

        out = _merge(parts, batch)
        probs = torch.softmax(out.logits, dim=-1)
        action = torch.multinomial(probs, 1, generator=generator).squeeze(1)
        logp = torch.log(probs.gather(1, action[:, None]).squeeze(1))
        return action, logp, out


def _merge(parts, batch: int) -> Outputs:
    """Put the outputs of the row groups back in the order of the batch."""
    if len(parts) == 1:
        return parts[0][1]
    fields = []
    for name in Outputs._fields:
        first = getattr(parts[0][1], name)
        whole = first.new_empty(batch, *first.shape[1:])
        for index, out in parts:
            whole[index] = getattr(out, name)
        fields.append(whole)
    return Outputs(*fields)
