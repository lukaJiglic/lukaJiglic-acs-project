# 004 — Encoding contract

Status: **approved** by the user on 2026-11-10 (D117). The check script of criterion 2 was committed in `b3927e0` on 2026-11-11 (D122). **Done 2026-11-11.**

Build-plan row 004. Tool and model: Claude Code 2.1.306, `claude-fable-5-2`.

## Goal

One approved document, `docs/encoding.md`, that fixes what the network sees and what it can do, for every phase (D107). It is the interface between the Rust track (spec 006, bindings) and the Python track (spec 007, model and learner).

## In scope

`docs/encoding.md` (draft 1 is written): card and seat conventions, the token fields and the event vocabulary (288 ids), the state-token features (1382, of which 463 exact inference bits), the action vector (160 slots), the critic input, the auxiliary targets, the reward, versioning.

## Out of scope

Code. The engine constants, the feature computation and its tests belong to spec 006. The network belongs to spec 007. Legal-action rules belong to the engine specs.

## Acceptance criteria

1. The user has read `docs/encoding.md` and answered its open points A–E.
2. The document's tables are consistent: id ranges are contiguous and the totals match (288, 1382, 160). `python tools/check_encoding.py` verifies this from the tables.
3. Every decision type of the Default preset has an action slot: bid, final contract, king call, talon packet, discard, announcement, kontra, card play. Every public event has a token.
4. The status line of `docs/encoding.md` says "approved" with the date, and the answers are recorded as decisions.

## Files expected to change

`docs/encoding.md`, `tools/check_encoding.py`, `docs/planning/decisions.md`, this spec.

## Open points for the user

The five points A–E at the end of `docs/encoding.md`.
