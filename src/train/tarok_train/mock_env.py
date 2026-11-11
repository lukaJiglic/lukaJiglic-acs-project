"""A mock environment with the arrays of docs/encoding.md (spec 007).

N tables step in parallel. Every step is one decision per table, by the seat
whose turn it is. The arrays have the contract's shapes; their content is made
up. Spec 006 replaces this with the Rust engine behind the same calls:

    obs = env.observe()                  # the acting seat of every table
    done, reward, parts = env.step(a)    # a finished table starts its next deal at once
    state = env.get_state()              # for a checkpoint; env.set_state(state) restores it

A mock deal: `rounds` rounds; in each round the four seats decide once, in seat
order from a random first seat. Round 0 is "bidding", round 1 one of king call,
talon pick, discard or announcements, the rest is "play". The legal slots of a
decision are the first `legal_per_decision` slots of its phase's action group.
Every action adds a public token; sometimes one more public event follows.

Modes. In the two learnable modes one of the legal slots is the rewarded one,
and the reward of a seat is the share of its decisions in the deal that took it.
Random play gives 1 / legal_per_decision, the best play gives 1. Which slot is
rewarded can only be read from the observation; no fixed preference finds it.

- "learnable": the rewarded slot is named by a hint in the state features
  (block B5). The MLP and the transformer can both learn it.
- "tokens": at three decisions out of four a flag in the state (first bit of B7)
  says "read the cue": the rewarded slot is then named by the deal's cue token,
  the second token of the sequence. Only a network that reads the tokens through
  correct decision points, and at later decisions through its cache, can learn it.
  At the other decisions the hint in the state counts, as in "learnable".
- "random": the reward is noise, independent of the actions.
"""

from __future__ import annotations

import numpy as np

from . import contract as C

# Constants of the made-up data. They shape the mock only; the learner does not read them.
UNSEEN_SHARE = 0.3  # share of cards marked unseen (state block F4)
NO_PARTNER_SHARE = 0.25  # share of deals without teams
PARTNER_PUBLIC_SHARE = 0.5  # share of decisions at which the partner is already public
TOKEN_CUE_SHARE = 0.75  # mode "tokens": share of decisions whose rewarded slot is named by the cue token

HINT_BLOCK = C.STATE_BLOCKS["B5"]  # one-hot: which legal slot (in slot order) is the rewarded one
CUE_FLAG = C.STATE_BLOCKS["B7"][0]  # 1: the cue token names the rewarded slot, not the hint
PROGRESS = C.STATE_BLOCKS["E4"][0]  # share of the deal played; the next feature: own score so far
CUE_EVENT = C.EVENT_GROUPS["KING"][0]  # the cue token's event is CUE_EVENT + cue


def part_weights() -> np.ndarray:
    """How a seat's reward is split over the score parts in the mock. They add up to 1.

    Game 0.4, difference 0.3, the rest shared evenly. Klop and radelc get nothing,
    as from the real engine until its flow C.
    """
    weights = np.zeros(C.N_SCORE_PARTS, dtype=np.float32)
    shared = [i for i, name in enumerate(C.SCORE_PARTS) if name not in ("game", "difference", "klop", "radelc")]
    weights[C.SCORE_PARTS.index("game")] = 0.4
    weights[C.SCORE_PARTS.index("difference")] = 0.3
    weights[shared] = 0.3 / len(shared)
    return weights


PHASE_ID = {name: i for i, name in enumerate(C.PHASES)}
# (phase, first slot of its action group) of the middle round, by deal number modulo 4.
MIDDLE_ROUNDS = (
    (PHASE_ID["king call"], C.ACTION_GROUPS["KING"][0]),
    (PHASE_ID["talon pick"], C.ACTION_GROUPS["TALON"][0]),
    (PHASE_ID["discard"], C.ACTION_GROUPS["DISCARD"][0]),
    (PHASE_ID["announcements"], C.ACTION_GROUPS["ANNOUNCE"][0]),
)


def _action_events() -> np.ndarray:
    """The token event that an action slot produces (the public trace of the action)."""
    pairs = {
        "BID": "BID",
        "KING": "KING",
        "TALON": "TALON_TAKEN",
        "DISCARD": "DISCARD_SHOWN",
        "ANNOUNCE": "ANNOUNCE",
        "KONTRA": "KONTRA",
        "ANNOUNCE_DONE": "ANNOUNCE_DONE",
        "ANNOUNCE_BARVNI_VALAT": "ANNOUNCE_BARVNI_VALAT",
        "PLAY": "PLAY",
        "RESERVED": "RESERVED",
    }
    events = np.zeros(C.N_ACTIONS, dtype=np.int16)
    for action_group, event_group in pairs.items():
        first, count = C.ACTION_GROUPS[action_group]
        events[first : first + count] = C.EVENT_GROUPS[event_group][0] + np.arange(count)
    return events


ACTION_EVENT = _action_events()

# The arrays that make up the environment's state (for checkpoints).
STATE_ARRAYS = ("deal", "rounds", "t", "first", "tok", "n_tok", "score", "cue", "location", "unseen", "partner")


class MockEnv:
    def __init__(
        self,
        n_tables: int,
        mode: str,
        seed: int,
        min_rounds: int,
        max_rounds: int,
        legal_per_decision: int,
        extra_token_probability: float,
        feature_density: float,
    ):
        if mode not in ("learnable", "tokens", "random"):
            raise ValueError(f"unknown mock mode '{mode}': use 'learnable', 'tokens' or 'random'")
        if not 2 <= legal_per_decision <= 4:
            raise ValueError("legal_per_decision must be 2 to 4 (the king call has four slots)")
        if not 2 <= min_rounds <= max_rounds or 2 + 8 * max_rounds > C.MAX_TOKENS:
            raise ValueError("rounds must satisfy 2 <= min_rounds <= max_rounds <= 15")
        self.n = n_tables
        self.mode = mode
        self.rng = np.random.default_rng(seed)
        self.min_rounds = min_rounds
        self.max_rounds = max_rounds
        self.legal = legal_per_decision
        self.extra = extra_token_probability
        self.density = feature_density
        self.random_reward = 1.0 / legal_per_decision  # learnable modes: the level of random play
        self.best_reward = 1.0  # learnable modes: every decision takes the rewarded slot
        self.reward_is_score = mode != "random"  # the mean reward is the task's score, worth logging
        self.illegal_actions = 0
        self.part_weights = part_weights()

        n = n_tables
        self.deal = np.zeros(n, dtype=np.int64)
        self.rounds = np.zeros(n, dtype=np.int64)
        self.t = np.zeros(n, dtype=np.int64)
        self.first = np.zeros(n, dtype=np.int64)
        self.tok = np.zeros((n, C.MAX_TOKENS, 4), dtype=np.int16)  # event, absolute seat, trick, position
        self.n_tok = np.zeros(n, dtype=np.int64)
        self.score = np.zeros((n, 4), dtype=np.int64)  # decisions that took the rewarded slot
        self.cue = np.zeros(n, dtype=np.int64)  # the deal's cue, 0 to legal - 1
        self.location = np.zeros((n, 4, C.N_CARDS), dtype=np.int64)
        self.unseen = np.zeros((n, 4, C.N_CARDS), dtype=bool)
        self.partner = np.zeros((n, 4), dtype=np.int64)
        self._new_deal(np.arange(n))
        self._prepare()

    # ---- deals ----

    def _new_deal(self, idx: np.ndarray) -> None:
        m = len(idx)
        rng = self.rng
        self.rounds[idx] = rng.integers(self.min_rounds, self.max_rounds + 1, m)
        self.t[idx] = 0
        self.first[idx] = rng.integers(0, 4, m)
        self.cue[idx] = rng.integers(0, self.legal, m)
        self.tok[idx] = 0
        self.tok[idx, 0] = (C.EVENT_GROUPS["BOS"][0], C.NO_SEAT, 0, 0)
        self.tok[idx, 1, 0] = CUE_EVENT + self.cue[idx]
        self.tok[idx, 1, 1] = C.NO_SEAT
        self.n_tok[idx] = 2
        self.score[idx] = 0
        self.location[idx] = rng.integers(0, C.CARD_LOCATION_CLASSES, (m, 4, C.N_CARDS))
        self.unseen[idx] = rng.random((m, 4, C.N_CARDS)) < UNSEEN_SHARE
        # Teams: a declarer and a partner, the other two together; some deals have none.
        declarer = rng.integers(0, 4, m)
        partner = (declarer + rng.integers(1, 4, m)) % 4
        seats = np.arange(4)[None, :]
        on_declarer_side = (seats == declarer[:, None]) | (seats == partner[:, None])
        mate = np.empty((m, 4), dtype=np.int64)
        for row in range(m):
            for side in (True, False):
                a, b = np.flatnonzero(on_declarer_side[row] == side)
                mate[row, a], mate[row, b] = b, a
        relative = (mate - seats) % 4  # 1 to 3
        relative[rng.random(m) < NO_PARTNER_SHARE] = C.PARTNER_NONE
        self.partner[idx] = relative

    def _prepare(self) -> None:
        """Build the observation of the acting seat of every table."""
        self._rng_before_observation = self.rng.bit_generator.state
        n, rng = self.n, self.rng
        rows = np.arange(n)
        seat = (self.first + self.t) % 4
        round_index = self.t // 4

        phase = np.full(n, PHASE_ID["play"])
        first = np.full(n, C.ACTION_GROUPS["PLAY"][0])
        bidding = round_index == 0
        phase[bidding] = PHASE_ID["bidding"]
        first[bidding] = C.ACTION_GROUPS["BID"][0]
        middle = round_index == 1
        for which, (phase_id, group_first) in enumerate(MIDDLE_ROUNDS):
            pick = middle & (self.deal % 4 == which)
            phase[pick], first[pick] = phase_id, group_first

        # Legal slots: the first `legal` slots of the phase's group. One of them is rewarded.
        mask = np.zeros((n, C.N_ACTIONS), dtype=bool)
        mask[rows[:, None], first[:, None] + np.arange(self.legal)[None, :]] = True
        hint = rng.integers(0, self.legal, n)
        use_cue = (rng.random(n) < TOKEN_CUE_SHARE) if self.mode == "tokens" else np.zeros(n, dtype=bool)
        self._mask = mask
        self._rewarded = first + np.where(use_cue, self.cue, hint)
        self._seat = seat
        self._phase = phase

        state = (rng.random((n, C.STATE_FEATURES)) < self.density).astype(np.float32)
        partner = self.partner[rows, seat]
        for start, width in (
            C.PHASE_BLOCK,
            C.REVEALED_PARTNER_BLOCK,
            HINT_BLOCK,
            C.STATE_BLOCKS["B7"],
            C.STATE_BLOCKS["E4"],
            C.UNSEEN_BLOCK,
        ):
            state[:, start : start + width] = 0.0
        state[rows, C.PHASE_BLOCK[0] + phase] = 1.0
        state[rows, HINT_BLOCK[0] + hint] = 1.0
        state[:, CUE_FLAG] = use_cue
        revealed = (partner != C.PARTNER_NONE) & (rng.random(n) < PARTNER_PUBLIC_SHARE)
        state[rows[revealed], C.REVEALED_PARTNER_BLOCK[0] + partner[revealed]] = 1.0
        state[:, PROGRESS] = self.t / (4 * self.rounds)
        state[:, PROGRESS + 1] = self.score[rows, seat] / self.rounds
        unseen = self.unseen[rows, seat]
        state[:, C.UNSEEN_BLOCK[0] : C.UNSEEN_BLOCK[0] + C.N_CARDS] = unseen

        # The critic's hidden features: where the unseen cards are, and the true partner.
        location = self.location[rows, seat]
        hands = [(location == other) & unseen for other in (1, 2, 3)]
        true_partner = np.zeros((n, 4), dtype=np.float32)
        has_partner = partner != C.PARTNER_NONE
        true_partner[rows[has_partner], partner[has_partner]] = 1.0
        hidden = np.concatenate(
            [*hands, (location == 0) & unseen, np.zeros((n, C.N_CARDS), dtype=bool), true_partner], axis=1
        ).astype(np.float32)

        # Tokens as the acting seat sees them: the seat field is relative to it.
        tokens = np.zeros((n, C.MAX_TOKENS, len(C.TOKEN_FIELDS)), dtype=np.int16)
        absolute = self.tok[:, :, 1]
        tokens[:, :, 0] = self.tok[:, :, 0]
        tokens[:, :, 1] = np.where(absolute == C.NO_SEAT, C.NO_SEAT, (absolute - seat[:, None]) % 4)
        tokens[:, :, 2] = self.tok[:, :, 2]
        tokens[:, :, 3] = self.tok[:, :, 3]
        tokens[:, :, 4] = np.arange(C.MAX_TOKENS)[None, :]
        tokens[np.arange(C.MAX_TOKENS)[None, :] >= self.n_tok[:, None]] = 0  # padding

        self._obs = {
            "seat": seat.copy(),
            "deal": self.deal.copy(),
            "tokens": tokens,
            "n_tokens": self.n_tok.copy(),
            "state": state,
            "mask": mask,
            "critic": np.concatenate([state, hidden], axis=1),
            "partner_target": partner.copy(),
            "card_target": location.copy(),
        }

    def observe(self) -> dict:
        """The arrays of the acting seat of every table (section 2 of the contract).

        seat (N,) absolute seat; deal (N,) deal counter of the table; tokens (N, 128, 5);
        n_tokens (N,); state (N, 1382); mask (N, 160); critic (N, 1656);
        partner_target (N,); card_target (N, 54).
        """
        return self._obs

    def _append(self, idx, event, seat, trick, position) -> None:
        at = self.n_tok[idx]
        self.tok[idx, at, 0] = event
        self.tok[idx, at, 1] = seat
        self.tok[idx, at, 2] = trick
        self.tok[idx, at, 3] = position
        self.n_tok[idx] += 1

    def step(self, actions: np.ndarray):
        """Apply one action per table.

        Returns done (N,) bool, reward (N, 4) and parts (N, 4, parts) by absolute seat.
        The last two are zero where the deal goes on; the parts of a seat add up to
        its reward. An illegal action is an error.
        """
        n, rng = self.n, self.rng
        rows = np.arange(n)
        actions = np.asarray(actions)
        legal = self._mask[rows, actions]
        if not legal.all():
            self.illegal_actions += int((~legal).sum())
            raise ValueError(f"{int((~legal).sum())} illegal actions reached the environment")

        seat = self._seat
        self.score[rows, seat] += actions == self._rewarded
        in_play = self._phase == PHASE_ID["play"]
        trick = np.where(in_play, np.minimum(self.t // 4, 12), 0)
        position = np.where(in_play, self.t % 4 + 1, 0)
        self._append(rows, ACTION_EVENT[actions], seat, trick, position)
        more = np.flatnonzero(rng.random(n) < self.extra)
        shown_first, shown_count = C.EVENT_GROUPS["TALON_SHOWN"]
        self._append(more, shown_first + rng.integers(0, shown_count, len(more)), C.NO_SEAT, 0, 0)

        self.t += 1
        done = self.t == 4 * self.rounds
        reward = np.zeros((n, 4), dtype=np.float32)
        if done.any():
            idx = np.flatnonzero(done)
            if self.mode == "random":
                reward[idx] = rng.normal(size=(len(idx), 4))
            else:
                reward[idx] = self.score[idx] / self.rounds[idx, None]
            self.deal[idx] += 1
            self._new_deal(idx)
        # Split by score part; the largest part takes the rounding rest, so the parts add up.
        parts = reward[:, :, None] * self.part_weights[None, None, :]
        parts[:, :, 0] += reward - parts.sum(axis=2)
        self._prepare()
        return done, reward, parts

    # ---- checkpoints ----

    def get_state(self) -> dict:
        """Everything needed to go on from here: the arrays and the random generator.

        The pending observation was drawn from the generator already. The state
        holds the generator as it was before that draw; set_state draws it again.
        """
        state = {name: getattr(self, name).copy() for name in STATE_ARRAYS}
        state["rng"] = self._rng_before_observation
        state["illegal_actions"] = self.illegal_actions
        return state

    def set_state(self, state: dict) -> None:
        """Restore a state from get_state and rebuild the pending observation."""
        for name in STATE_ARRAYS:
            getattr(self, name)[...] = state[name]
        self.illegal_actions = state["illegal_actions"]
        self.rng.bit_generator.state = state["rng"]
        self._prepare()


def make(cfg_env: dict, seed: int) -> MockEnv:
    """The environment a config asks for. Only the mock exists until spec 006."""
    if cfg_env["kind"] != "mock":
        raise ValueError(f"unknown environment '{cfg_env['kind']}': only 'mock' exists until spec 006")
    return MockEnv(
        cfg_env["tables"],
        cfg_env["mode"],
        seed,
        cfg_env["min_rounds"],
        cfg_env["max_rounds"],
        cfg_env["legal_per_decision"],
        cfg_env["extra_token_probability"],
        cfg_env["feature_density"],
    )
