"""The PPO update of spec 007.

Pieces, each tested on its own:
- `gae`: generalised advantage estimation per seat-deal, the reward only at the end.
- `realised_return`: what the deal really paid, at every decision.
- `clipped_policy_loss`, `policy_entropy`, `value_error`.
- `partner_loss`, `card_location_loss`: the auxiliary losses.
- `compute_losses`: all parts and their weighted total.
- `update`: epochs of minibatches of seat-deals.

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


def compute_losses(out: Outputs, value, batch: Minibatch, cfg_ppo: dict) -> dict:
    """Every loss term and the weighted total, over the valid decisions of the minibatch.

    out.logits are the masked logits of the heads: illegal actions hold -inf, so
    they have probability 0 here as they had when the action was taken.
    Returns the terms by name, "total", "weighted" (each term as it enters the
    total) and "logp" (the log-probability of the actions taken, under these weights).
    """
    v = batch.valid
    action = batch.action[v][:, None]
    log_probs = F.log_softmax(out.logits[v], dim=-1)
    logp = log_probs.gather(1, action).squeeze(1)
    realised = batch.realised[v]
    form, delta = cfg_ppo["value_loss"], cfg_ppo["huber_delta"]

    terms = {
        "policy": clipped_policy_loss(logp, batch.logp[v], _advantages(batch, cfg_ppo), cfg_ppo["clip_range"]),
        "entropy": policy_entropy(log_probs, batch.mask[v]).mean(),
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


def update(net, critic, optimizer, deals: SeatDeals, cfg_ppo: dict, device, rng: np.random.Generator) -> dict:
    """`epochs` passes over the seat-deals in minibatches of `update_batch`. Returns the mean losses.

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
    phases = torch.zeros(2, len(C.PHASES), device=device)  # entropy sum and decisions, per phase
    partner = torch.zeros(4, device=device)  # hits and decisions: all with a partner, then partner not public
    weight = 0
    limit = float(np.log1p(cfg_ppo["clip_range"]))
    first = {}
    for epoch in range(cfg_ppo["epochs"]):
        for index in minibatches(rng.permutation(len(deals)), cfg_ppo["update_batch"]):
            index = np.sort(index)
            batch = deals.to_torch(device, index)
            out, value = evaluate(net, critic, batch)
            losses = compute_losses(out, value, batch, cfg_ppo)
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
                    _phase_and_partner_stats(out, batch, phases, partner)

    sums = (sums / weight).tolist()
    phases = phases.tolist()
    partner = partner.tolist()
    stats = {f"loss/{name}": value for name, value in zip(names, sums) if name != "entropy"}
    entropy = sums[names.index("entropy")]
    stats["loss/entropy"] = -cfg_ppo["entropy_weight"] * entropy  # the entropy term of the total
    stats["entropy/mean"] = entropy
    for i, name in enumerate(C.PHASES):
        if phases[1][i]:
            stats[f"entropy/{name}"] = phases[0][i] / phases[1][i]
    stats["aux/partner_accuracy"] = partner[0] / max(partner[1], 1)
    stats["aux/partner_accuracy_hidden"] = partner[2] / max(partner[3], 1)
    stats["ppo/approx_kl"] = sums[-2]
    stats["ppo/clip_fraction"] = sums[-1]
    stats.update({name: float(value) for name, value in first.items()})
    return stats


def _phase_and_partner_stats(out, batch, phases, partner) -> None:
    """Add this minibatch to the running sums (tensors on the device)."""
    v = batch.valid
    state = batch.state[v]
    entropy = policy_entropy(F.log_softmax(out.logits[v], dim=-1), batch.mask[v])
    start, width = C.PHASE_BLOCK
    phase = state[:, start : start + width].argmax(dim=1)
    phases[0].index_add_(0, phase, entropy)
    phases[1].index_add_(0, phase, torch.ones_like(entropy))
    target = batch.partner_target[v]
    has_partner = target != C.PARTNER_NONE
    # "None" is never a prediction: the guess is the best of the three other seats.
    guess = out.partner[v][:, 1:].argmax(dim=1) + 1
    hit = (guess == target) & has_partner
    start, width = C.REVEALED_PARTNER_BLOCK
    hidden = has_partner & (state[:, start : start + width].sum(dim=1) == 0)
    partner += torch.stack([hit.sum(), has_partner.sum(), (hit & hidden).sum(), hidden.sum()])


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
