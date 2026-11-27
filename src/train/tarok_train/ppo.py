"""The PPO update of spec 007.

Pieces, each tested on its own:
- `gae`: generalised advantage estimation per seat-deal, the reward only at the end.
- `realised_return`: what the deal really paid, at every decision.
- `clipped_policy_loss`, `policy_entropy`, `value_error`.
- `partner_loss`, `card_location_loss`: the auxiliary losses.
- `compute_losses`: all parts and their weighted total.
- `update`: epochs of minibatches of seat-deals.
- `EntropyController`: one entropy weight per phase, steered toward a target entropy (spec 008).
- `rollout_stats`, `deal_stats`, `round_stats`, `possible_places`, `possible_partners`: the monitors.

No hyperparameter has a default here. All come from the config's [ppo] table.
"""

from __future__ import annotations

import numpy as np
import torch
import torch.nn.functional as F

from . import contract as C
from .model import Outputs
from .rollout import Minibatch, SeatDeals  # noqa: F401  (Minibatch is part of this module's interface)

# --------------------------------------------------------------------------
# Returns and advantages
# --------------------------------------------------------------------------


def gae(value, valid, reward, gamma, lam):
    """Generalised advantage estimation for padded seat-deals.

    value (G, S, P): the critic's value at each decision, per score part.
    valid (G, S): False on padding. reward (G, P): paid once, after the last decision.
    Returns (advantage, lambda_return), both (G, S, P), zero on padding.

    A seat-deal is one row. Nothing is read across rows or past a row's last
    decision, so nothing leaks from one deal into another.
    """
    g, s, p = value.shape
    lengths = valid.sum(dim=1)
    advantage = torch.zeros_like(value)
    next_value = value.new_zeros(g, p)
    next_advantage = value.new_zeros(g, p)
    for k in reversed(range(s)):
        last = (lengths - 1 == k)[:, None]
        live = valid[:, k][:, None]
        delta = torch.where(last, reward, gamma * next_value) - value[:, k]
        here = delta + gamma * lam * torch.where(last, torch.zeros_like(next_advantage), next_advantage)
        here = torch.where(live, here, torch.zeros_like(here))
        advantage[:, k] = here
        next_value = value[:, k]
        next_advantage = here
    lambda_return = torch.where(valid[:, :, None], advantage + value, torch.zeros_like(value))
    return advantage, lambda_return


def realised_return(valid, reward, gamma):
    """The discounted reward of the deal at every decision of the seat-deal: (G, S, P)."""
    s = valid.shape[1]
    lengths = valid.sum(dim=1)
    steps_to_end = (lengths[:, None] - 1 - torch.arange(s, device=valid.device)[None, :]).clamp(min=0)
    discount = torch.as_tensor(gamma, dtype=reward.dtype, device=reward.device) ** steps_to_end
    return torch.where(valid[:, :, None], discount[:, :, None] * reward[:, None, :], torch.zeros(()).to(reward))


def prepare(deals: SeatDeals, cfg_ppo: dict) -> None:
    """Compute advantages and return targets of a batch of finished seat-deals, once per update."""
    value = torch.from_numpy(deals.value)
    valid = torch.from_numpy(deals.valid)
    parts = torch.from_numpy(deals.parts)
    advantage, lambda_return = gae(value, valid, parts, cfg_ppo["gamma"], cfg_ppo["gae_lambda"])
    deals.advantage = advantage.sum(dim=-1).numpy()  # the policy uses the sum over the score parts
    deals.lambda_return = lambda_return.numpy()
    deals.realised = realised_return(valid, parts, cfg_ppo["gamma"]).numpy()


# --------------------------------------------------------------------------
# Loss parts
# --------------------------------------------------------------------------


def clipped_policy_loss(logp_new, logp_old, advantage, clip):
    """PPO's clipped surrogate, as a loss (to minimise), averaged over the decisions."""
    ratio = torch.exp(logp_new - logp_old)
    surrogate = torch.minimum(ratio * advantage, ratio.clamp(1.0 - clip, 1.0 + clip) * advantage)
    return -surrogate.mean()


def policy_entropy(log_probs, mask):
    """Entropy of each masked policy row."""
    return -(log_probs.exp() * log_probs.masked_fill(~mask, 0.0)).sum(dim=-1)


def value_error(prediction, target, form, delta):
    """Error of a value prediction, element by element.

    form "huber": quadratic up to `delta`, linear beyond, so one very large
    reward cannot dominate the gradient. form "mse": the squared error.
    """
    if form == "huber":
        return F.huber_loss(prediction, target, reduction="none", delta=delta)
    if form == "mse":
        return (prediction - target) ** 2
    raise ValueError(f"unknown value_loss '{form}': use 'huber' or 'mse'")


def partner_loss(logits, target):
    """Cross-entropy of the partner-seat head, skipped where the target is "none".

    logits (n, 4), target (n,). Returns (loss, number of decisions counted).
    With nothing to count the loss is 0.
    """
    keep = target != C.PARTNER_NONE
    count = keep.sum()
    # No branch on the count: reading it would make the GPU wait at every minibatch.
    each = F.cross_entropy(logits, torch.where(keep, target, torch.ones_like(target)), reduction="none")
    return (each * keep).sum() / count.clamp(min=1), count


def card_location_loss(logits, target, unseen):
    """Cross-entropy of the card-location head over the unseen cards only (state block F4).

    logits (n, 54, 4), target (n, 54), unseen (n, 54) bool. Returns (loss, number of cards counted).
    With nothing to count the loss is 0.
    """
    count = unseen.sum()
    each = F.cross_entropy(logits.flatten(0, -2), target.flatten(), reduction="none").view(target.shape)
    return (each * unseen).sum() / count.clamp(min=1), count


def evaluate(net, critic, batch: Minibatch):
    """The actor's outputs and the critic's value for a minibatch, both (B, S, ...).

    The actor gets tokens, state and mask. Only the critic gets the hidden features.
    """
    out = net.forward_deals(batch.tokens, batch.points, batch.state, batch.mask)
    value = critic(torch.cat([batch.state, batch.critic_extra], dim=-1))
    return out, value


def _advantages(batch: Minibatch, cfg_ppo: dict):
    advantage = batch.advantage[batch.valid]
    if cfg_ppo["normalise_advantage"]:
        advantage = (advantage - advantage.mean()) / (advantage.std(unbiased=False) + 1e-8)
    return advantage


# The loss terms and the sign and config weight with which each enters the total.
TERMS = {
    "policy": (1.0, None),
    "critic": (1.0, "critic_weight"),
    "value_head": (1.0, "value_head_weight"),
    "q": (1.0, "q_weight"),
    "partner": (1.0, "partner_weight"),
    "cards": (1.0, "card_location_weight"),
    "entropy": (-1.0, "entropy_weight"),
}


def phase_of(state):
    """The index into C.PHASES of each state row (block B1)."""
    start, width = C.PHASE_BLOCK
    return state[..., start : start + width].argmax(dim=-1)


def compute_losses(out: Outputs, value, batch: Minibatch, cfg_ppo: dict, entropy_weights=None) -> dict:
    """Every loss term and the weighted total, over the valid decisions of the minibatch.

    out.logits are the masked logits of the heads: illegal actions hold -inf, so
    they have probability 0 here as they had when the action was taken.
    entropy_weights: None, or one entropy weight per phase (the entropy controller).
    Each decision's entropy then counts with the weight of its phase instead of
    `entropy_weight`. Nothing else changes.
    Returns the terms by name, "total", "weighted" (each term as it enters the
    total) and "logp" (the log-probability of the actions taken, under these weights).
    """
    v = batch.valid
    action = batch.action[v][:, None]
    log_probs = F.log_softmax(out.logits[v], dim=-1)
    logp = log_probs.gather(1, action).squeeze(1)
    realised = batch.realised[v]
    form, delta = cfg_ppo["value_loss"], cfg_ppo["huber_delta"]
    entropy = policy_entropy(log_probs, batch.mask[v])

    terms = {
        "policy": clipped_policy_loss(logp, batch.logp[v], _advantages(batch, cfg_ppo), cfg_ppo["clip_range"]),
        "entropy": entropy.mean(),
        "critic": value_error(value[v], batch.lambda_return[v], form, delta).sum(dim=-1).mean(),
        "value_head": value_error(out.value[v], realised, form, delta).sum(dim=-1).mean(),
        # The Q head learns the realised return of the action taken (D119 C).
        "q": value_error(out.q[v].gather(1, action).squeeze(1), realised.sum(dim=-1), form, delta).mean(),
        "partner": partner_loss(out.partner[v], batch.partner_target[v])[0],
        "cards": card_location_loss(out.cards[v], batch.card_target[v], batch.unseen[v])[0],
    }
    weighted = {
        name: sign * (1.0 if key is None else cfg_ppo[key]) * terms[name] for name, (sign, key) in TERMS.items()
    }
    if entropy_weights is not None:
        per_decision = entropy_weights.to(entropy)[phase_of(batch.state[v])]
        weighted["entropy"] = -(per_decision * entropy).mean()
    return {**terms, "total": sum(weighted.values()), "weighted": weighted, "logp": logp}


# --------------------------------------------------------------------------
# The update
# --------------------------------------------------------------------------


def minibatches(order: np.ndarray, size: int) -> list:
    """Cut `order` into minibatches of `size`. A remainder under half a batch joins the one before.

    So no optimiser step is taken on a handful of seat-deals with normalised advantages.
    """
    chunks = [order[start : start + size] for start in range(0, len(order), size)]
    if len(chunks) > 1 and len(chunks[-1]) < size // 2:
        chunks[-2:] = [np.concatenate(chunks[-2:])]
    return chunks


def trunk_parameters(net) -> list:
    """The weights all heads share: everything of the actor but the heads."""
    return [weight for name, weight in net.named_parameters() if not name.startswith("heads.")]


def update(
    net, critic, optimizer, deals: SeatDeals, cfg_ppo: dict, device, rng: np.random.Generator, entropy_weights=None
) -> dict:
    """`epochs` passes over the seat-deals in minibatches of `update_batch`. Returns the mean losses.

    entropy_weights: None, or the entropy weight of each phase (see `compute_losses`).

    Actor and critic are clipped apart, each to `gradient_clip`, so a large
    critic gradient cannot shrink the actor's step. (The value, Q and auxiliary
    heads share the actor's trunk and its clip with the policy; "grad/trunk_..."
    shows how much each term pulls on the trunk.) The statistics stay on the
    device until the end: reading a number back makes the GPU wait.

    Measured on the first minibatch, before any step of this update:
    - "ppo/behaviour_mismatch": the largest difference between the stored
      log-probability of an action and the one these weights give. It is 0 when
      the update sees what the actor saw. Anything else is a pipeline bug.
    - "grad/trunk_<term>": the gradient norm of each loss term on the trunk.
    """
    net.train()
    critic.train()
    names = ("total", *TERMS)
    sums = torch.zeros(len(names) + 2, device=device)  # the loss parts, then KL and clipped decisions
    # Per phase: entropy sum and decisions; then the same over the decisions with two or more legal slots.
    phases = torch.zeros(4, len(C.PHASES), device=device)
    partner = torch.zeros(4, device=device)  # hits and decisions: all with a partner, then partner not public
    guess = torch.zeros(2, 3, device=device)  # partner head, card head: hits, hits of a uniform guess, cases
    if entropy_weights is not None:
        entropy_weights = entropy_weights.to(device)
    weight = 0
    limit = float(np.log1p(cfg_ppo["clip_range"]))
    first = {}
    for epoch in range(cfg_ppo["epochs"]):
        for index in minibatches(rng.permutation(len(deals)), cfg_ppo["update_batch"]):
            index = np.sort(index)
            batch = deals.to_torch(device, index)
            out, value = evaluate(net, critic, batch)
            losses = compute_losses(out, value, batch, cfg_ppo, entropy_weights)
            difference = batch.logp[batch.valid] - losses["logp"].detach()
            if not first:
                first["ppo/behaviour_mismatch"] = difference.abs().max()
                trunk = trunk_parameters(net)
                for name in TERMS:
                    if name == "critic":  # the critic is its own network: nothing reaches the actor
                        continue
                    grads = torch.autograd.grad(losses["weighted"][name], trunk, retain_graph=True, allow_unused=True)
                    first[f"grad/trunk_{name}"] = torch.sqrt(sum((g**2).sum() for g in grads if g is not None))
            optimizer.zero_grad(set_to_none=True)
            losses["total"].backward()
            actor_norm = torch.nn.utils.clip_grad_norm_(net.parameters(), cfg_ppo["gradient_clip"])
            critic_norm = torch.nn.utils.clip_grad_norm_(critic.parameters(), cfg_ppo["gradient_clip"])
            optimizer.step()
            first.setdefault("grad/actor", actor_norm)  # before clipping
            first.setdefault("grad/critic", critic_norm)

            n = int(deals.valid[index].sum())
            weight += n
            with torch.no_grad():
                sums += torch.stack(
                    [losses[name].detach() * n for name in names]
                    + [difference.sum(), (difference.abs() > limit).sum()]
                )
                if epoch == 0:
                    _phase_and_partner_stats(out, batch, phases, partner, guess)

    sums = (sums / weight).tolist()
    phases = phases.tolist()
    partner = partner.tolist()
    guess = guess.tolist()
    stats = {f"loss/{name}": value for name, value in zip(names, sums) if name != "entropy"}
    entropy = sums[names.index("entropy")]
    if entropy_weights is None:
        stats["loss/entropy"] = -cfg_ppo["entropy_weight"] * entropy  # the entropy term of the total
    else:  # the term as it entered the total, with the weight of each decision's phase
        stats["loss/entropy"] = stats["loss/total"] - sum(
            sign * (1.0 if key is None else cfg_ppo[key]) * stats[f"loss/{name}"]
            for name, (sign, key) in TERMS.items()
            if name != "entropy"
        )
    stats["entropy/mean"] = entropy
    for i, name in enumerate(C.PHASES):
        if phases[1][i]:
            stats[f"entropy/{name}"] = phases[0][i] / phases[1][i]
            # A decision with one legal slot has entropy 0 whatever the policy is. The mean above
            # moves with the share of such decisions; the mean below does not.
            stats[f"forced_share/{name}"] = 1.0 - phases[3][i] / phases[1][i]
            if phases[3][i]:
                stats[f"entropy_choice/{name}"] = phases[2][i] / phases[3][i]
    stats["aux/partner_accuracy"] = partner[0] / max(partner[1], 1)
    stats["aux/partner_accuracy_hidden"] = partner[2] / max(partner[3], 1)
    # Spec 008, criterion 14: next to each accuracy, what a uniform guess over the places still possible gets.
    stats["aux/partner_guess"] = guess[0][1] / max(guess[0][2], 1)
    stats["aux/card_accuracy"] = guess[1][0] / max(guess[1][2], 1)
    stats["aux/card_guess"] = guess[1][1] / max(guess[1][2], 1)
    stats["ppo/approx_kl"] = sums[-2]
    stats["ppo/clip_fraction"] = sums[-1]
    stats.update({name: float(value) for name, value in first.items()})
    return stats


def possible_places(state):
    """Where each card can still be, as far as the state features say: (n, 54, 4) bool.

    Entry 0 is "out of play" (the talon, the declarer's hidden discards), entries 1 to 3
    the other seats, as in the card-location target. Meant for unseen cards (block F4).
    A seat is possible unless block F2 says it cannot hold the card. "Out of play" is
    possible before the talon is turned up (bidding, contract choice, king call), and
    during the announcements and the play for a seat that is not the declarer if the
    card is a suit card below the king: only those can be hidden discards (a discarded
    tarok is shown, kings are never discarded). The declarer has seen the talon and its
    own discards. Holds for the contracts with a talon exchange (deal flow A).
    """
    n = state.shape[0]
    start, width = C.CANNOT_HOLD_BLOCK
    seats = state[:, start : start + width].reshape(n, 3, C.N_CARDS) < 0.5
    phase = phase_of(state)
    talon_hidden = phase < C.PHASES.index("talon pick")
    after_exchange = phase > C.PHASES.index("discard")
    declarer = state[:, C.ROLE_BLOCK[0] + C.ROLES.index("declarer")] > 0.5
    may_be_discarded = torch.ones(C.N_CARDS, dtype=torch.bool, device=state.device)
    may_be_discarded[: C.N_TAROKS] = False
    may_be_discarded[list(C.KINGS)] = False
    out = talon_hidden[:, None] | ((after_exchange & ~declarer)[:, None] & may_be_discarded[None, :])
    return torch.cat([out[:, :, None], seats.permute(0, 2, 1)], dim=2)


def possible_partners(state):
    """Which relative seat can still be the acting seat's one team-mate: (n, 4) bool, entry 0 never.

    The called partner knows the declarer. An opponent knows it is not the declarer.
    Once the declarer's partner is public (block B6), the declarer has that seat and an
    opponent the one seat left. Before the teams are fixed all three stay possible (the
    partner target is "none" there and such decisions are not counted).
    """

    def block(name):
        start, width = C.STATE_BLOCKS[name]
        return state[:, start : start + width] > 0.5

    declarer_seat, role, public = block("B3"), block("B4"), block("B6")
    is_declarer, is_partner, is_opponent = (
        role[:, C.ROLES.index(name)] for name in ("declarer", "called partner", "opponent")
    )
    out = torch.ones_like(declarer_seat)
    out[:, 0] = False
    known = public.any(dim=1)
    out = torch.where((is_declarer & known)[:, None], public, out)
    out = torch.where(is_partner[:, None], declarer_seat, out)
    out = torch.where(is_opponent[:, None], out & ~declarer_seat & ~public, out)
    out[:, 0] = False
    return out


def accuracy_and_guess(hit, counted, n_places):
    """Sums for an accuracy next to its guessing level: hits, expected hits of a uniform guess, cases.

    hit: the head was right. counted: the case counts. n_places: how many places were
    still possible there. A uniform guess over them is right with probability 1 / n_places.
    """
    counted = counted.float()
    return torch.stack(
        [(hit.float() * counted).sum(), (counted / n_places.float().clamp(min=1)).sum(), counted.sum()]
    )


def _phase_and_partner_stats(out, batch, phases, partner, guess) -> None:
    """Add this minibatch to the running sums (tensors on the device)."""
    v = batch.valid
    state = batch.state[v]
    entropy = policy_entropy(F.log_softmax(out.logits[v], dim=-1), batch.mask[v])
    phase = phase_of(state)
    choice = (batch.mask[v].sum(dim=1) > 1).to(entropy)
    phases[0].index_add_(0, phase, entropy)
    phases[1].index_add_(0, phase, torch.ones_like(entropy))
    phases[2].index_add_(0, phase, entropy * choice)
    phases[3].index_add_(0, phase, choice)
    target = batch.partner_target[v]
    has_partner = target != C.PARTNER_NONE
    # "None" is never a prediction: the guess is the best of the three other seats.
    best = out.partner[v][:, 1:].argmax(dim=1) + 1
    hit = (best == target) & has_partner
    start, width = C.REVEALED_PARTNER_BLOCK
    hidden = has_partner & (state[:, start : start + width].sum(dim=1) == 0)
    partner += torch.stack([hit.sum(), has_partner.sum(), (hit & hidden).sum(), hidden.sum()])
    guess += aux_sums(out.partner[v], out.cards[v], state, target, batch.card_target[v], batch.unseen[v])


def aux_sums(partner_logits, card_logits, state, partner_target, card_target, unseen):
    """For the partner head (row 0) and the card-location head (row 1): hits, hits of a uniform guess, cases.

    n decisions: partner_logits (n, 4), card_logits (n, 54, 4), state (n, 1382), partner_target (n,),
    card_target (n, 54), unseen (n, 54) bool. A decision without a partner and a card that
    is not unseen do not count. The uniform guess is over the places the state leaves
    possible (`possible_partners`, `possible_places`).
    """
    has_partner = partner_target != C.PARTNER_NONE
    best = partner_logits[:, 1:].argmax(dim=1) + 1
    partner = accuracy_and_guess((best == partner_target) & has_partner, has_partner, possible_partners(state).sum(dim=1))
    cards = accuracy_and_guess(card_logits.argmax(dim=-1) == card_target, unseen, possible_places(state).sum(dim=2))
    return torch.stack([partner, cards])


def rollout_stats(deals: SeatDeals, updates_done: int) -> dict:
    """Numbers about the collected data, before the update. `updates_done`: updates before this one."""
    v = deals.valid
    age = updates_done - deals.version[v]
    stats = {
        # In the real game the rewards of a deal add up to zero, so their mean says nothing.
        "reward/mean_abs": float(np.abs(deals.reward).mean()),
        "reward/std": float(deals.reward.std()),
        "rollout/seat_deals": float(len(deals)),
        "rollout/decisions": float(deals.decisions),
        # Updates between taking an action and training on it. 0: all data is from the current weights.
        "rollout/decision_age_max": float(age.max()),
        "rollout/decision_age_mean": float(age.mean()),
    }
    # Spec 008, criterion 12: the length of a seat-deal in decisions and in tokens (at its last decision).
    lengths = v.sum(axis=1)
    tokens = deals.points.max(axis=1)
    stats["rollout/decisions_per_seat_deal_mean"] = float(lengths.mean())
    stats["rollout/decisions_per_seat_deal_max"] = float(lengths.max())
    stats["rollout/tokens_per_seat_deal_mean"] = float(tokens.mean())
    stats["rollout/tokens_per_seat_deal_max"] = float(tokens.max())
    # Explained variance of the critic: its value when acting against the realised return.
    predicted = deals.value.sum(axis=-1)[v]
    realised = deals.realised.sum(axis=-1)[v]
    variance = realised.var()
    stats["critic/explained_variance"] = float(1.0 - (realised - predicted).var() / variance) if variance > 0 else 0.0
    actions = deals.action[v]
    for name, (first, count) in C.ACTION_GROUPS.items():
        share = float(((actions >= first) & (actions < first + count)).mean())
        if share > 0:
            stats[f"actions/{name}"] = share
    return stats


def _deal_keys(deals: SeatDeals) -> np.ndarray:
    """One number per seat-deal that is equal for the seats of one deal of one table."""
    return deals.table.astype(np.int64) * (int(deals.deal.max()) + 1) + deals.deal


def deal_stats(deals: SeatDeals) -> dict:
    """Monitors of the deals in a batch of real seat-deals (spec 008, criterion 13; D107).

    Read from the state features of each seat-deal's last decision, its first tokens and
    its score parts, once per deal (a deal has up to four stored seat-deals):
    - "deals/contract_<name>": the share of each contract played. The six normal contracts
      are always listed; another contract only when it occurs.
    - "deals/forehand_forced": the three other seats passed before forehand spoke, so
      forehand had to name a contract. "deals/forehand_forced_tri": it named tri (tri can
      only be played that way).
    - "reward/deal_sum_abs_max": the largest sum of a deal's four rewards (criterion 4: zero).
    - "deals/declarer_wins": the declarer's side won the game: its game part is positive,
      or its valat part where a valat replaces the game.
    - the monitors of the round of announcements and the reward size (`round_stats`, spec 018).

    Not meaningful on the mock, whose state features are noise.
    """
    _, first = np.unique(_deal_keys(deals), return_index=True)
    last = deals.valid[first].sum(axis=1) - 1
    state = deals.state[first, last]

    def entry(block):
        start, width = block
        return state[:, start : start + width]

    n = len(first)
    stats = {"deals/count": float(n)}
    contract = entry(C.CONTRACT_BLOCK).argmax(axis=1)
    counts = np.bincount(contract, minlength=len(C.BIDS))
    for index, name in enumerate(C.BIDS):
        if name in C.NORMAL_CONTRACTS or counts[index]:
            stats[f"deals/contract_{name.replace(' ', '_')}"] = float(counts[index] / n)
    # Tokens: the start token, then the calls of the auction. A pass is the first bid event.
    calls = deals.tokens[first, 1:4, 0]
    stats["deals/forehand_forced"] = float((calls == C.EVENT_GROUPS["BID"][0]).all(axis=1).mean())
    stats["deals/forehand_forced_tri"] = float((contract == C.BIDS.index("tri")).mean())
    # The true parts: under the planted fault of a control run `deals.parts` belong to another seat.
    won = deals.true_parts[first][:, [C.SCORE_PARTS.index("game"), C.SCORE_PARTS.index("valat")]].sum(axis=1)
    opponent = entry(C.ROLE_BLOCK)[:, C.ROLES.index("opponent")] > 0.5
    stats["deals/declarer_wins"] = float((np.where(opponent, -won, won) > 0).mean())
    # Criterion 4: the largest sum of a deal's four rewards, over the deals stored for all four seats.
    _, inverse, counts = np.unique(_deal_keys(deals), return_inverse=True, return_counts=True)
    sums = np.zeros(len(counts))
    np.add.at(sums, inverse.reshape(-1), deals.true_parts.sum(axis=1, dtype=np.float64))
    stats["reward/deal_sum_abs_max"] = float(np.abs(sums[counts == 4]).max(initial=0.0))
    stats.update(round_stats(deals))
    return stats


# The round of announcements (deal flow B): the token ids it writes (docs/encoding.md, sections 3 and 4b).
ANNOUNCED = ("trula", "kralji", "kralj_ultimo", "pagat_ultimo", "valat")  # the order of the ANNOUNCE ids
GAME_LEVELS = ("none", "kontra", "re", "sub", "mort")  # the level of a target after 0 to 4 kontra tokens


def round_stats(deals: SeatDeals) -> dict:
    """Monitors of the round of announcements and of the size of the rewards (spec 018, criteria 1 to 5).

    Read from the tokens 146 to 156 of each seat-deal, up to its last visible token. The round
    is public, so every seat of a deal has the same such tokens; a deal is counted once
    (criterion 5), from its first stored seat-deal.
    - "round/announced_<bonus>": the share of deals in which the bonus was announced.
    - "round/game_<level>": the share of deals with the game at that level at the end of the
      round. Each KONTRA token on the game raises the level by one, so the number of such
      tokens is the level the last one left. The five shares add up to 1.
    - "round/bonus_kontra": the share of deals with a kontra on any of the five bonuses.
    Over the trained seat-deals (each one counts, not once per deal):
    - "reward/abs_mean", "reward/abs_max", "reward/share_above_10": the size of the rewards.
    - "round/decision_share": the share of the trained decisions in the phase "announcements".
    - "round/decisions_per_seat_deal": such decisions per trained seat-deal.
    """
    _, first = np.unique(_deal_keys(deals), return_index=True)
    events = deals.tokens[first, :, 0].astype(np.int64)
    seen = deals.points[first].max(axis=1)
    events = np.where(np.arange(events.shape[1])[None, :] < seen[:, None], events, C.EVENT_GROUPS["PAD"][0])
    n = len(first)
    stats = {}
    announce, _ = C.EVENT_GROUPS["ANNOUNCE"]
    for index, name in enumerate(ANNOUNCED):
        stats[f"round/announced_{name}"] = float((events == announce + index).any(axis=1).mean())
    kontra, count = C.EVENT_GROUPS["KONTRA"]
    level = (events == kontra).sum(axis=1)
    if level.max(initial=0) >= len(GAME_LEVELS):
        raise ValueError(f"a deal holds {int(level.max())} kontra tokens on the game; mort, the highest level, is 4")
    for index, name in enumerate(GAME_LEVELS):
        stats[f"round/game_{name}"] = float((level == index).sum() / n)
    stats["round/bonus_kontra"] = float(((events > kontra) & (events < kontra + count)).any(axis=1).mean())

    size = np.abs(deals.reward.astype(np.float64))
    stats["reward/abs_mean"] = float(size.mean())
    stats["reward/abs_max"] = float(size.max())
    stats["reward/share_above_10"] = float((size > 10).mean())
    phase = deals.state[..., C.PHASE_BLOCK[0] : C.PHASE_BLOCK[0] + C.PHASE_BLOCK[1]].argmax(axis=-1)
    in_round = ((phase == C.PHASES.index("announcements")) & deals.valid).sum()
    stats["round/decision_share"] = float(in_round / deals.valid.sum())
    stats["round/decisions_per_seat_deal"] = float(in_round / len(deals))
    return stats


class EntropyController:
    """One entropy weight per phase, steered so that the phase's policy entropy stays near a target.

    After every update, for each phase that had decisions in it:

        error    = (target - entropy) / target, cut to [-1, 1]
        integral <- integral * exp(gain * error),         kept inside [min_weight, max_weight]
        weight    = integral * exp(proportional * error), kept inside [min_weight, max_weight]

    An entropy under the target raises the weight, one above it lowers the weight. The
    integral moves slowly and remembers the level the phase needs; the proportional part
    answers at once and fades as the entropy comes back, which keeps the loop from swinging.
    The controller changes nothing but these weights (spec 008, criterion 17). All of its
    settings come from the config's [entropy_control] table; every weight and integral
    starts at [ppo] entropy_weight. With `enabled = false` the weights never move and the
    update uses [ppo] entropy_weight as before.
    """

    def __init__(self, cfg_control: dict, start_weight: float):
        self.cfg = cfg_control
        self.enabled = cfg_control["enabled"]
        self.integral = np.full(len(C.PHASES), float(start_weight))
        self.weights = np.full(len(C.PHASES), float(start_weight))

    @staticmethod
    def next_weights(
        integral: float, entropy: float, target: float, gain: float, proportional: float, min_weight: float, max_weight: float
    ) -> tuple:
        """(the new integral, the new weight) of one phase."""
        error = min(max((target - entropy) / target, -1.0), 1.0)
        integral = min(max(integral * float(np.exp(gain * error)), min_weight), max_weight)
        weight = min(max(integral * float(np.exp(proportional * error)), min_weight), max_weight)
        return integral, weight

    def step(self, entropies: dict) -> None:
        """entropies: phase name -> mean policy entropy of the phase's decisions in the last update."""
        if not self.enabled:
            return
        cfg = self.cfg
        for name, entropy in entropies.items():
            index = C.PHASES.index(name)
            self.integral[index], self.weights[index] = self.next_weights(
                self.integral[index],
                entropy,
                cfg["targets"][name],
                cfg["gain"],
                cfg["proportional"],
                cfg["min_weight"],
                cfg["max_weight"],
            )

    def in_band(self, entropies: dict) -> dict:
        """phase name -> the entropy lies within target * (1 - band) and target * (1 + band)."""
        band = self.cfg["band"]
        return {
            name: bool(self.cfg["targets"][name] * (1 - band) <= entropy <= self.cfg["targets"][name] * (1 + band))
            for name, entropy in entropies.items()
        }

    def tensor(self, device):
        """The weights for `compute_losses`, or None when the controller is off."""
        if not self.enabled:
            return None
        return torch.tensor(self.weights, dtype=torch.float32, device=device)

    def state(self) -> dict:
        """What a checkpoint keeps."""
        return {"integral": self.integral.tolist(), "weights": self.weights.tolist()}

    def load_state(self, state: dict) -> None:
        self.integral = np.array(state["integral"], dtype=np.float64)
        self.weights = np.array(state["weights"], dtype=np.float64)
