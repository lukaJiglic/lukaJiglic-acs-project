"""Past checkpoints as opponents (D14, spec 007).

A share of the deals has one or more seats played by a frozen past checkpoint.
The decisions of those seats are not trained on. The pool keeps the last
snapshots of the network; one of them, drawn evenly, plays the frozen seats.
"""

from __future__ import annotations

import copy

import numpy as np
import torch.nn as nn


def assign_frozen_seats(rng: np.random.Generator, n_tables: int, share: float, max_frozen_seats: int) -> np.ndarray:
    """Which seats a past checkpoint plays in the next deal of each table: (n_tables, 4) bool.

    A deal has frozen seats with probability `share`. Such a deal has 1 to
    `max_frozen_seats` of them, each count equally likely, at random seats.
    """
    frozen = np.zeros((n_tables, 4), dtype=bool)
    chosen = np.flatnonzero(rng.random(n_tables) < share)
    how_many = rng.integers(1, max_frozen_seats + 1, len(chosen))
    order = rng.random((len(chosen), 4)).argsort(axis=1)  # a random order of the seats
    frozen[chosen[:, None], order] = np.arange(4)[None, :] < how_many[:, None]
    return frozen


class OpponentPool:
    """The last `size` snapshots of the network's weights, kept on the CPU."""

    def __init__(self, size: int):
        self.size = size
        self.snapshots = []

    def __len__(self) -> int:
        return len(self.snapshots)

    def add(self, net: nn.Module) -> None:
        self.snapshots.append({k: v.detach().cpu().clone() for k, v in net.state_dict().items()})
        del self.snapshots[: -self.size]

    def load_one(self, net: nn.Module, rng: np.random.Generator) -> int:
        """Load a snapshot, drawn evenly, into `net`. Returns its index in the pool."""
        index = int(rng.integers(0, len(self.snapshots)))
        net.load_state_dict(copy.deepcopy(self.snapshots[index]))
        return index
