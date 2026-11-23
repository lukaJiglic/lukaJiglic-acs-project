"""Rollout storage by table, seat and deal (spec 007).

The loop adds one row per decision. When a table's deal ends, `finish` records
the reward of each seat. `take` returns the finished deals as one sequence per
table, seat and deal (a "seat-deal"): what one L2 pass of the transformer reads.
Deals never mix: a row belongs to exactly one (table, seat, deal).

The token sequence is kept once per seat-deal, not per decision: a seat's
sequence at a later decision starts with its sequence at every earlier one, so
the last one is enough.

The training loop calls `discard` after `take`, so rows of deals that are still
running at an update are never trained on (they would be stale after it).
"""

from __future__ import annotations

from dataclasses import dataclass

import numpy as np
import torch

from . import contract as C

# The fields of one decision and the type they are stored in.
ROW_FIELDS = {
    "tables": np.int32,
    "seat": np.int8,
    "deal": np.int64,
    "n_tokens": np.int16,
    "state": np.float32,
    "mask": np.bool_,
    "critic_extra": np.float32,
    "partner_target": np.int8,
    "card_target": np.int8,
    "action": np.int16,
    "logp": np.float32,
    "value": np.float32,
    "version": np.int32,  # the number of updates done when the action was taken
}


@dataclass
class Minibatch:
    """Tensors of some seat-deals, padded to the longest. Shapes: B seat-deals, S decisions, P tokens."""

    tokens: torch.Tensor  # (B, P, 5)
    points: torch.Tensor  # (B, S) tokens visible at each decision
    state: torch.Tensor  # (B, S, 1382)
    mask: torch.Tensor  # (B, S, 160) legal actions; all True on padding
    critic_extra: torch.Tensor  # (B, S, 274) the critic's hidden features
    valid: torch.Tensor  # (B, S) False on padding
    partner_target: torch.Tensor = None  # (B, S)
    card_target: torch.Tensor = None  # (B, S, 54)
    action: torch.Tensor = None  # (B, S)
    logp: torch.Tensor = None  # (B, S) log-probability of the action when it was taken
    advantage: torch.Tensor = None  # (B, S)
    lambda_return: torch.Tensor = None  # (B, S, parts) the critic's target
    realised: torch.Tensor = None  # (B, S, parts) the realised return, per score part

    @property
    def unseen(self):
        """(B, S, 54): the cards the card-location loss counts (state block F4)."""
        start, width = C.UNSEEN_BLOCK
        return self.state[..., start : start + width] > 0.5


@dataclass
class SeatDeals:
    """Finished seat-deals as padded numpy arrays. G seat-deals, S decisions, P tokens."""

    table: np.ndarray  # (G,)
    seat: np.ndarray  # (G,)
    deal: np.ndarray  # (G,)
    reward: np.ndarray  # (G,) the seat's reward for the deal
    parts: np.ndarray  # (G, parts) the same reward by score part
    true_parts: np.ndarray  # (G, parts) what the environment paid the seat; differs from `parts` only under a planted fault
    valid: np.ndarray  # (G, S)
    points: np.ndarray  # (G, S)
    tokens: np.ndarray  # (G, P, 5) the seat's token sequence at its last decision
    state: np.ndarray
    mask: np.ndarray
    critic_extra: np.ndarray
    partner_target: np.ndarray
    card_target: np.ndarray
    action: np.ndarray
    logp: np.ndarray
    value: np.ndarray  # (G, S, parts) the critic's value when the action was taken
    version: np.ndarray  # (G, S) updates done when the action was taken
    advantage: np.ndarray = None  # set by ppo.prepare
    lambda_return: np.ndarray = None
    realised: np.ndarray = None

    def __len__(self) -> int:
        return len(self.reward)

    @property
    def decisions(self) -> int:
        return int(self.valid.sum())

    def to_torch(self, device, index=None) -> Minibatch:
        """The seat-deals `index` (all if None) as tensors, cut to their longest deal."""
        pick = slice(None) if index is None else index
        valid = self.valid[pick]
        s = int(valid.sum(axis=1).max())
        points = self.points[pick][:, :s]
        p = max(int(points.max()), 1)

        def tensor(array, cut=s, kind=None):
            if array is None:
                return None
            out = torch.from_numpy(np.ascontiguousarray(array[pick][:, :cut])).to(device)
            return out if kind is None else out.to(kind)

        return Minibatch(
            tokens=tensor(self.tokens, p, torch.long),
            points=tensor(self.points, kind=torch.long),
            state=tensor(self.state),
            mask=tensor(self.mask),
            critic_extra=tensor(self.critic_extra),
            valid=tensor(self.valid),
            partner_target=tensor(self.partner_target, kind=torch.long),
            card_target=tensor(self.card_target, kind=torch.long),
            action=tensor(self.action, kind=torch.long),
            logp=tensor(self.logp),
            advantage=tensor(self.advantage),
            lambda_return=tensor(self.lambda_return),
            realised=tensor(self.realised),
        )


class Rollout:
    def __init__(self, zero_sum: bool = False):
        """zero_sum: the four rewards of a deal must add up to zero (the real game, not the mock)."""
        self.zero_sum = zero_sum
        self._chunks = {name: [] for name in ROW_FIELDS}
        self._order = []  # the order in which rows arrived
        self._next = 0
        self._results = {}  # (table, deal) -> (reward (4,), parts (4, parts), tokens (4, 128, 5), true parts)
        self._live = np.zeros((0, 4, C.MAX_TOKENS, len(C.TOKEN_FIELDS)), dtype=np.int16)
        self.pending_rows = 0

    def add(self, tokens, **rows) -> None:
        """Add one row per decision.

        tokens (n, 128, 5): each seat's token array at this decision. The other
        arguments are the ROW_FIELDS, each with n rows.
        """
        rows.setdefault("version", np.zeros(len(rows["tables"]), dtype=np.int32))
        if set(rows) != set(ROW_FIELDS):
            raise ValueError(f"rows must have exactly the fields {tuple(ROW_FIELDS)} and tokens")
        tables = np.asarray(rows["tables"])
        n = len(tables)
        if n == 0:
            return
        for name in ROW_FIELDS:
            if len(rows[name]) != n:
                raise ValueError(f"field {name} has {len(rows[name])} rows, expected {n}")
        # Spec 008, criterion 2: the chosen slot of every stored decision is legal in its mask.
        legal = np.asarray(rows["mask"])[np.arange(n), np.asarray(rows["action"])]
        if not legal.all():
            raise ValueError(f"{int((~legal).sum())} stored decisions took an illegal action (a slot the mask forbids)")
        for name, kind in ROW_FIELDS.items():
            self._chunks[name].append(np.asarray(rows[name]).astype(kind))  # a copy: the caller may reuse its buffers
        self._order.append(np.arange(self._next, self._next + n))
        self._next += n
        self.pending_rows += n
        # The latest token array of each (table, seat) replaces the one before.
        need = int(tables.max()) + 1
        if need > len(self._live):
            grown = np.zeros((need, *self._live.shape[1:]), dtype=np.int16)
            grown[: len(self._live)] = self._live
            self._live = grown
        self._live[tables, np.asarray(rows["seat"])] = tokens

    def finish(self, tables, deals, reward, parts, true_parts=None) -> None:
        """The deals `deals` of the tables `tables` ended. reward (m, 4), parts (m, 4, parts), by seat.

        true_parts: what the environment paid, where `reward` and `parts` carry a planted fault
        (the control run of spec 008). The monitors read it; the learner never does.

        The parts of a seat must add up to its reward: the advantages are computed
        from the parts, the logs from the reward. With `zero_sum` the four rewards of
        a deal must add up to zero as well (spec 008, criterion 4).
        """
        reward = np.asarray(reward, dtype=np.float32)
        parts = np.asarray(parts, dtype=np.float32)
        if parts.shape[1:] != (4, C.N_SCORE_PARTS):
            raise ValueError(f"parts must have the shape (m, 4, {C.N_SCORE_PARTS}), got {parts.shape}")
        gap = np.abs(parts.sum(axis=-1) - reward).max(initial=0.0)
        if gap > 1e-5:
            raise ValueError(f"the score parts of a seat do not add up to its reward (largest gap {gap:.2e})")
        if self.zero_sum:
            gap = np.abs(reward.sum(axis=1, dtype=np.float64)).max(initial=0.0)
            if gap > 1e-5:
                raise ValueError(f"the four rewards of a deal do not add up to zero (largest sum {gap:.2e})")
        true_parts = parts if true_parts is None else np.asarray(true_parts, dtype=np.float32)
        for i, (table, deal) in enumerate(zip(tables, deals)):
            table = int(table)
            tokens = self._live[table].copy() if table < len(self._live) else None
            self._results[(table, int(deal))] = (reward[i].copy(), parts[i].copy(), tokens, true_parts[i].copy())

    def discard(self) -> None:
        """Drop every stored row and result: the rows of running deals, and results nobody took."""
        self._chunks = {name: [] for name in ROW_FIELDS}
        self._order = []
        self._results.clear()
        self.pending_rows = 0

    def take(self) -> SeatDeals | None:
        """The finished deals, removed from the storage. None if no deal with stored rows has finished."""
        if self.pending_rows == 0:
            self._results.clear()
            return None
        data = {name: np.concatenate(chunks) for name, chunks in self._chunks.items()}
        order = np.concatenate(self._order)
        table, seat, deal = data["tables"], data["seat"], data["deal"]

        # Which rows belong to a finished deal.
        pairs, inverse = np.unique(np.stack([table.astype(np.int64), deal], axis=1), axis=0, return_inverse=True)
        inverse = inverse.reshape(-1)
        finished = np.array([(int(t), int(d)) in self._results for t, d in pairs])
        done = finished[inverse]

        keep = ~done
        self._chunks = {name: ([data[name][keep]] if keep.any() else []) for name in ROW_FIELDS}
        self._order = [order[keep]] if keep.any() else []
        self.pending_rows = int(keep.sum())
        if not done.any():
            return None

        rows = np.flatnonzero(done)
        rows = rows[np.lexsort((order[rows], deal[rows], seat[rows], table[rows]))]
        t, s, d = table[rows], seat[rows], deal[rows]
        new = np.ones(len(rows), dtype=bool)
        new[1:] = (t[1:] != t[:-1]) | (s[1:] != s[:-1]) | (d[1:] != d[:-1])
        group = np.cumsum(new) - 1
        starts = np.flatnonzero(new)
        position = np.arange(len(rows)) - starts[group]
        g, longest = len(starts), int(position.max()) + 1

        def padded(name, fill=0, kind=None):
            source = data[name][rows]
            out = np.full((g, longest, *source.shape[1:]), fill, dtype=kind or source.dtype)
            out[group, position] = source
            return out

        valid = np.zeros((g, longest), dtype=bool)
        valid[group, position] = True
        points = padded("n_tokens", kind=np.int64)
        width = max(int(points.max()), 1)
        reward = np.empty(g, dtype=np.float32)
        parts = np.empty((g, C.N_SCORE_PARTS), dtype=np.float32)
        true_parts = np.empty((g, C.N_SCORE_PARTS), dtype=np.float32)
        tokens = np.zeros((g, width, len(C.TOKEN_FIELDS)), dtype=np.int16)
        for i, at in enumerate(starts):
            seat_reward, seat_parts, seat_tokens, seat_true_parts = self._results[(int(t[at]), int(d[at]))]
            reward[i] = seat_reward[s[at]]
            parts[i] = seat_parts[s[at]]
            true_parts[i] = seat_true_parts[s[at]]
            tokens[i] = seat_tokens[s[at], :width]
        # Every result of a finished deal is used up, also of deals with no stored rows.
        for key in [key for key in self._results if key not in {(int(a), int(b)) for a, b in pairs[~finished]}]:
            del self._results[key]

        return SeatDeals(
            table=t[starts].astype(np.int64),
            seat=s[starts].astype(np.int64),
            deal=d[starts],
            reward=reward,
            parts=parts,
            true_parts=true_parts,
            valid=valid,
            points=points,
            tokens=tokens,
            state=padded("state"),
            mask=padded("mask", fill=True),
            critic_extra=padded("critic_extra"),
            partner_target=padded("partner_target"),
            card_target=padded("card_target"),
            action=padded("action"),
            logp=padded("logp"),
            value=padded("value"),
            version=padded("version", fill=-1),
        )
