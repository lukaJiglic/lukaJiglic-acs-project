# Architecture

Status: **planned, not built** (2026-11-10). This file follows the code: each spec that builds a part updates its section and changes "planned" to "built". Decisions: D51–D64, D106–D111.

## Parts

```
                      +---------------------------+
                      |  tarok-engine (Rust)      |
                      |  rules, scoring, seeding, |
                      |  records, encoding        |
                      +-----+-----------+---------+
                            |           |
              native + PyO3 |           | wasm32 + wasm-bindgen
                            v           v
   +-------------------------------+   +--------------------------------+
   | src/train (Python, PyTorch)   |   | src/app (Vue 3, Pinia, PWA)    |
   | vectorised self-play, PPO     |   | engine and model run in the    |
   | learner on the GPU, export    |   | browser; no server             |
   +---------------+---------------+   +----------------+---------------+
                   |  weights (f16 file, versioned)     ^
                   +------------------------------------+
```

## tarok-engine (`src/engine`) — planned

One Rust crate. Rules of four-player Slovenian tarok on the Default preset, toggles, scoring, seeded deals, JSON game records, the observation and action encoding. Built natively for training and tests, and as WASM for the browser. The hand-written transformer forward pass (D54) is added after training has started (D111).

## Training (`src/train`) — planned

A Python package. The Rust engine steps many games at once through a PyO3 module; the network runs in PyTorch on the GPU in batches (D111). PPO with a full-information critic (D14). A transformer over play and bid tokens with a state token (D18, D30), and an MLP control. Native Windows, pinned CUDA wheel (D55).

## App (`src/app`) — planned

Vue 3, Pinia, Vite, TypeScript, custom CSS variables, PWA (D52). The engine and the model run in the browser inside the WASM module. Weights are a versioned f16 file cached by the service worker (D53, D64). Hosted on GitHub Pages (D57). Storage in IndexedDB and localStorage (D58).

## Shared formats — planned

- One JSON game-record schema for self-play logs, app games, adviser sessions and lessons (D59).
- The encoding contract `docs/encoding.md` (spec 004): card indices, tokens, state features, action vector.
