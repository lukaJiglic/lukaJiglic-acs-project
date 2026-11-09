# 06 — Network design and browser inference for a tarok agent

Scope: input encoding, network architecture, size, running the trained net in a phone browser, PyTorch→browser export, and the training stack on a Ryzen 5 5600 / GTX 1660 SUPER (6 GB) / 32 GB / Windows 11 PC. The training algorithm itself is covered in another document.

Marking convention used throughout:
- **[V]** verified by reading the source on 2026-11-09.
- **[V-partial]** source read, but the specific number or claim was not in the readable part.
- **[M]** from memory. Not re-verified. Treat as a lead, not a fact.
- **[I]** inferred or computed by me from verified inputs.

---

## (a) Sources

All accessed 2026-11-09.

### Card-game networks

| ID | Source | URL | What it gave |
|---|---|---|---|
| S1 | DouZero paper (Zha et al. 2021), ar5iv HTML | https://ar5iv.labs.arxiv.org/html/2106.06135 | Encoding tables, LSTM over 15 moves, 6×512 MLP, hardware. [V] |
| S2 | Suphx paper (Li et al. 2020), ar5iv HTML | https://ar5iv.labs.arxiv.org/html/2003.13590 | Input 34×838, CNN w/o pooling, oracle dropout, hardware. Figure-level layer details not readable. [V-partial] |
| S3 | Rebstock, Solinas, Buro, "Learning Policies from Human Data for Skat" (2019), ar5iv | https://ar5iv.labs.arxiv.org/html/1905.10907 | Full feature tables, 5-layer MLP, 2.3–2.4 M weights, 2.5 ms/move on CPU. [V] |
| S4 | "Transformer Based Planning in the Observation Space with Applications to Trick Taking Card Games" (2024), arXiv HTML | https://arxiv.org/html/2404.13150 | GPT-2 8×256 over observation tokens; Hearts, Skat, The Crew; ms/turn numbers. [V] |
| S5 | SchafkopfRL (tobiasemrich), GitHub README | https://github.com/tobiasemrich/SchafkopfRL | 55-dim info vector + 16-dim card tokens; PPO; LSTM and linear variants. [V] |
| S6 | AlphaHoldem (Zhao et al. AAAI 2022) | https://ojs.aaai.org/index.php/AAAI/article/view/20394 | Abstract only read. Pseudo-Siamese conv streams. [V-partial] |
| S7 | OpenHoldem (arXiv 2012.06168) | https://arxiv.org/pdf/2012.06168 | Card tensor 6×4×13, action tensor 24 channels. From search snippet only. [V-partial] |
| S8 | Niklaus et al., "Challenging Human Supremacy ... Jass" (AAAI-20 workshop / ICAISC 2020), Zenodo | https://zenodo.org/records/5769551 | Abstract + slide summary: DMCTS ≈ DNN; rule-based trump ≈ DNN. Network layers not retrieved. [V-partial] |
| S9 | Niklaus master thesis (Jass) | https://niklaus.ai/files/theses/Master_Thesis.pdf | PDF fetched but not text-extractable here. [V-partial] |
| S10 | Kita et al. 2024 bridge bidding (arXiv 2406.10306), describing Lockhart et al.'s baseline | https://arxiv.org/html/2406.10306v1 | 480-dim binary input, 4×1024 MLP. From search snippet. [V-partial] |
| S11 | Lockhart et al., "Human-Agent Cooperation in Bridge Bidding" (2020) | https://arxiv.org/abs/2011.14124 | Abstract only. [V-partial] |
| S12 | Edelkamp, "Outer-Learning Framework ... Skat" (arXiv 2512.15435, Dec 2025) | https://arxiv.org/html/2512.15435v1 | Statement that top Bridge/Skat bots are still combinatorial. From snippet. [V-partial] |

### Browser runtimes and platform support

| ID | Source | URL | What it gave |
|---|---|---|---|
| B1 | onnxruntime-web 1.31.1 README (jsDelivr) | https://cdn.jsdelivr.net/npm/onnxruntime-web@1.31.1/README.md | Backend × browser compatibility table. [V] |
| B2 | ORT Web deploy tutorial | https://onnxruntime.ai/docs/tutorials/web/deploy.html | .wasm file names, conditional import, IndexedDB caching advice. [V] |
| B3 | caniuse WebGPU | https://caniuse.com/webgpu | Per-browser support, 88.6 % global. [V] |
| B4 | caniuse WASM SIMD | https://caniuse.com/wasm-simd | 96.8 % global; Safari 16.4+. [V] |
| B5 | caniuse WASM threads | https://caniuse.com/wasm-threads | 97.2 % global; Safari 14.5+. [V] |
| B6 | WebKit blog, WebGPU in Safari 26 beta | https://webkit.org/blog/16993/news-from-wwdc25-webgpu-now-available-for-testing-in-safari-26-beta/ | Safari 26 ships WebGPU on iOS/iPadOS/macOS/visionOS. [V] |
| B7 | Chrome WebGPU overview | https://developer.chrome.com/docs/web-platform/webgpu/overview | Android: Chrome 121+, Android 12+, Qualcomm/ARM GPUs. [V] |
| B8 | "Anatomizing Deep Learning Inference in Web Browsers" (arXiv 2402.05981v2) | https://arxiv.org/html/2402.05981v2 | 20 mobile devices; browser vs native ≈15.8× on CPU; SIMD −37 %, threads −17 %. [V] |
| B9 | Hugging Face mirrors of ort-wasm-simd-threaded.wasm | (search snippets) | ~10–12 MB uncompressed; jsep variant ~19.5 MB. [V-partial] |
| B10 | web.dev "Storage for the web" + WebKit bugs 232302 / 237350 | https://bugs.webkit.org/show_bug.cgi?id=232302 | Safari 7-day script-writable storage cap; reports it hits home-screen PWAs too. [V-partial] |

### Export and Rust inference

| ID | Source | URL | What it gave |
|---|---|---|---|
| E1 | PyTorch 2.15 torch.onnx docs | https://docs.pytorch.org/docs/2.15/onnx.html | `dynamo=True` default since 2.9; deprecations; `dynamic_shapes`. [V] |
| E2 | onnxscript PR #2674 (aten_lstm / aten_gru) | https://github.com/microsoft/onnxscript/pull/2674 | Merged 2025-11-04; LSTM/GRU in dynamo exporter; dynamic-shape tests skipped. [V] |
| E3 | tract GitHub | https://github.com/sonos/tract | Runs in browser via wasm32; ONNX+NNEF load; rustc ≥ 1.91; 11k commits. [V] |
| E4 | tract-onnx on docs.rs | https://docs.rs/crate/tract-onnx/latest | 0.23.4 dated 2026-07-08. From snippet. [V-partial] |
| E5 | Burn releases | https://github.com/tracel-ai/burn/releases | v0.22.1 (Oct 20), v0.23.0-pre.1 (Oct 29); WASM init notes; wgpu device. [V] |
| E6 | burn-tch docs (tch-rs Windows notes) | https://docs.rs/crate/burn-tch/latest | LibTorch 2.9.0 target; Windows CUDA needs manual LIBTORCH. From snippet. [V-partial] |

### PyTorch / training stack

| ID | Source | URL | What it gave |
|---|---|---|---|
| T1 | PyTorch 2.11.0 release notes (newreleases.io mirror) | https://newreleases.io/project/pypi/torch/release/2.11.0 | CUDA 13.0 default on PyPI (Linux); Volta dropped from 12.8/12.9; Turing kept. [V] |
| T2 | dev-discuss: CUDA 12.6 wheels stop at 2.15 (2026-09-04) | https://dev-discuss.pytorch.org/t/notice-cuda-12-6-wheels-will-no-longer-be-published-from-pytorch-2-15-drops-maxwell-pascal-volta/3432 | Turing sm_75 kept in CUDA 13.x builds. [V] |
| T3 | dev-discuss: PyTorch 2.14 final RC | https://dev-discuss.pytorch.org/t/pytorch-2-14-final-rc-available/3429 | Windows wheels: CUDA 12.6, 13.0, 13.2; release 2026-09-02. [V] |
| T4 | dev-discuss: PyTorch 2.15 RC1 | https://dev-discuss.pytorch.org/t/pytorch-2-15-rc1-produced-for-pytorch-torchvision-torchaudio/3464.md | Release day 2026-10-28. From snippet. [V-partial] |
| T5 | PyO3 0.29 changelog / free-threading guide | https://pyo3.rs/main/free-threading | 0.29.0 on 2026-06-11; abi3t; 3.14t+. From snippets. [V-partial] |

---

## (b) Encodings and architectures

### Table

| Project | Game | Input features | History encoding | Body | Heads | Size / speed | Mark |
|---|---|---|---|---|---|---|---|
| DouZero (S1) | Dou Dizhu, 3 players, 54 cards | Per-zone 4×15 card matrices flattened to 54: own hand, union of others' hands, most recent move, each opponent's played cards, cards-left one-hots (17/20), bombs one-hot (15). Action also a 54-vector. Landlord state ≈ 373 dims. | 15 most recent moves, grouped 3 per step → 5×162 matrix → LSTM. Hidden 128 in released code. | 6 fully-connected layers, 512 wide, ReLU. | Single Q(s,a) scalar; one forward pass per legal action. | ≈1.4 M params [I from code shapes]. 45 actors on 3× 1080 Ti, 30 days. | [V]; LSTM hidden size and param count [M]/[I] |
| Suphx (S2) | Riichi mahjong, 4 players | 34 columns (tile types) × D channels. Discard model 34×838. Private tiles as 4 count-channels. Integers/categoricals as all-0/all-1 channels. 100+ look-ahead feature channels. Oracle features (opponents' hands) with scheduled dropout during RL. | Encoded as channels (no recurrent unit). | Deep CNN, 1-D convs over 34 columns, no pooling. Residual blocks. | Discard: 34-way policy. Riichi/Chow/Pong/Kong: 2-way. Win: rule-based. | Residual depth/channels in Fig. 4–5 only; I recall ~50 blocks × 256 ch [M, uncertain]. 44 GPUs per RL run. | [V-partial] |
| Skat imitation (S3) | Skat, 3 players, 32 cards | Cardplay: hand (32), played cards per player (32×3), lead cards per opponent (32×2), sloughed cards (32×2), void suits (5×2), current trick (32), trick value, max bid type per opp (6×2), points, game flags, trump remaining (32), suit (4), skat (32), declarer position (2). Bidding: hand (32), position (3), bids (67+67). | None. Flattened fixed vector. | 5 hidden FC layers, ReLU, dropout 0.6 on middle 3. One net per role×game-type (6 cardplay nets). | 32-way card policy; bid nets 67-way; discard net 496-way (card pairs); value heads MSE. | Largest ≈2.4 M weights. 2.5 ms/move, one CPU thread. 16 h on one 1080 Ti. Test acc 72–86 %. | [V] |
| Transformer planning (S4) | Hearts, Skat, The Crew | Every observation is one token: cards by identity, positions, bids, declarations. Vocab 60 (Hearts), 113 (Skat), 83 (Crew). | Whole game as token sequence. Context 75–145. | GPT-2: 8 layers, d=256, 8 heads, FFN 1024. | Next-token head (acts as policy) + outcome-distribution head (value). | ≈6–7 M params [I from dims]. 71 ms/turn argmax, single thread. | [V]; param count [I] |
| SchafkopfRL (S5) | Schafkopf, 4 players, 32 cards | info vector 55: game type (7), declarer (4), first player (4), scores/120 (4), own cards (32), teams (4). Card tokens 16 = card (12) + player (4), relative to ego. | LSTM over game history and over current trick (LSTM variant); flat vector (linear variant). | Actor-critic MLP. Sizes not stated; author says "too many hidden units". | 43-way action: game type (9) + double (2) + card (32). | Not stated. PPO, 5–15 M steps, 15 days. Beaten by PIMC and imitation. | [V] |
| AlphaHoldem (S6, S7) | HUNL poker | Card tensor 6×4×13 (private, flop, turn, river, all public, all). Action tensor 24 channels × bet options over 4 rounds. | Action tensor over rounds (fixed window). | Pseudo-Siamese: two conv streams (cards, actions), fused by FC. | Policy over discrete bets + value. | Paper reports ~3 ms per decision on CPU [M]. | [V-partial] |
| Bridge bidding, Lockhart-style (S10, S11) | Bridge bidding | 480-dim binary vector (OpenSpiel / Pgx convention): hand 52, vulnerability, bid history as one-hots. | Flattened fixed window. | 4-layer MLP, 1024 wide, ReLU. | 38-way bid policy (+ value). | ≈3.7 M params [I]. | [V-partial] |
| Jass, Niklaus (S8, S9) | Jass, 4 players, 36 cards | Not retrievable here. | — | DNN for card play and trump selection, trained on Swisslos human logs [M]. | 36-way card, 7-way trump. | Rule-based trump "almost as good as DNN"; DMCTS on par with DNN card play. | [V-partial] |
| Doppelkopf | — | No neural self-play project with published architecture found. | — | — | — | — | not found |

### Patterns across the projects

1. Cards are always one-hot by zone. Each zone is a 54-wide (or 32/36/52-wide) binary vector: own hand, played-by-player, current trick, talon, discards. Nobody uses rank/suit embeddings except the transformer paper (S4), which uses token identity. [V]
2. Partner / role is encoded as a one-hot relative to the acting player. DouZero bakes the role into separate networks (landlord vs peasant); Skat uses soloist vs defender nets; SchafkopfRL uses a teams one-hot. [V]
3. History is either flattened (Skat, Bridge, AlphaHoldem) or recurrent (DouZero: LSTM over last 15 moves; SchafkopfRL: LSTM over all plays). The transformer approach (S4) is the only one that treats the whole game as a sequence. [V]
4. Bodies are shallow and wide MLPs: 4–6 layers, 512–1024 wide. Convolutions appear only where the input has a spatial axis worth convolving (mahjong tile types, poker rank×suit). [V]
5. Two head styles: (a) Q per action with the action fed in as a card vector (DouZero), which generalises to rare actions; (b) fixed-size policy logits with a legal-action mask plus a value head (Skat, Bridge, Schafkopf, transformer). [V]
6. Auxiliary hidden-information heads exist as a training trick, not an inference output: Suphx oracle dropout (S2); the Rong et al. bridge work predicts partner's hand [M]; Solinas et al. 2019 predicts card locations to improve PIMC sampling [V-partial, from snippet]. [V-partial]
7. Separate networks per phase are the norm. Skat has bid / continue / pickup / declare / discard / cardplay nets. Suphx has five. SchafkopfRL uses one net with a 43-way action space covering game choice and card. [V]

---

## (c) Size vs strength

### Evidence

| Evidence | Size | Strength | Mark |
|---|---|---|---|
| Skat imitation net (S3) | ≈2.4 M weights, 5×FC | Beat prior bidding systems; card play ~77–83 % human-move accuracy; "reasonably strong" but weaker than the Kermit search player. 2.5 ms/move on CPU. | [V] |
| DouZero (S1) | ≈1.4 M params (6×512 + LSTM-128) [I] | Top of Botzone leaderboard in 2021; beat DeltaDou after 10 days. | [V] / [I] |
| Transformer planner (S4) | 8×256 GPT-2, ≈6–7 M [I] | Hearts: beats xinxin. Skat: argmax policy loses to Kermit by 2×; with MCTS still behind. | [V] |
| Bridge bidding MLP (S10) | 4×1024, ≈3.7 M [I] | "Current benchmark" for bidding in 2024. | [V-partial] |
| SchafkopfRL (S5) | unstated | PPO agent ranked below PIMC and imitation. | [V] |
| AlphaHoldem (S6) | not stated; ~3 ms CPU decision [M] | Beat Slumbot and DeepStack-like baselines. | [M] |
| Edelkamp 2025 (S12) | — | Says best Bridge and Skat bots are still combinatorial search, not neural. | [V-partial] |

### Reading

- Every published trick-taking network that plays at a credible level is in the 1–7 M parameter range. None needs more. [I from table]
- Strength is bounded by the training signal and by search at inference, not by width. The two Skat results (S3 vs S4) show that a 2.4 M MLP with 300 M human examples and a 7 M transformer with self-play both lose to a search engine. [I]
- Tarok has 54 cards, 4 players, 12 tricks, a talon, bidding, and announcements. Its state vector is about the same size as Skat's (32 cards, 10 tricks) times 1.7. There is no evidence that this needs more than a Skat-sized net. [I]

### Sensible range for tarok

- Input: 500–900 binary/scaled features (see recommendation). [I]
- Body: 3–4 FC layers, 256–512 wide, or a 128-wide GRU over trick tokens feeding a 3×256 MLP. [I]
- Parameters: 0.3–1.5 M. fp32 file 1.2–6 MB; fp16 0.6–3 MB; int8 0.3–1.5 MB. [I]
- Compute per decision: 0.5–3 MFLOP. On a phone this is sub-millisecond in any runtime once warmed up. [I] No published phone benchmark exists for a net this small; see (d).
- Go smaller than DouZero, not bigger. A 54-card game with legal-move masks has a much smaller effective action space than Dou Dizhu's 27 472 combinations. [I]

---

## (d) Running the model in a phone browser (as of 2026-11-09)

### Platform availability

| Feature | Android Chrome | iOS Safari | Global | Mark |
|---|---|---|---|---|
| WASM (MVP) | yes | yes | ~100 % | [V] B1 |
| WASM SIMD | yes (Chrome 91+ desktop; Android current 157) | yes, Safari iOS 16.4+ | 96.8 % | [V] B4 |
| WASM threads (SharedArrayBuffer) | yes (74+) | yes, iOS 14.5+ | 97.2 % | [V] B5 |
| Threads need COOP/COEP headers (cross-origin isolation) | yes | yes | — | [M], standard web behaviour; B2 does not mention it |
| WebGPU | Chrome 121+, Android 12+, Qualcomm or ARM GPUs only | Safari 26 (iOS 26, 2025-09) | 88.6 % | [V] B3, B6, B7 |
| WebGPU in Firefox Android | disabled by default | — | — | [V] B3 |

Note the discrepancy: the onnxruntime-web 1.31.1 README (B1) marks WebGPU as not supported on Safari iOS and Chrome iOS. caniuse (B3) and WebKit (B6) say iOS 26 Safari has WebGPU. The README table is stale or reflects ORT's own testing, not the platform. Treat ORT WebGPU on iOS as untested by Microsoft. [V] / [I]

### Options

| Option | Runtime download (uncompressed) | GPU path | Small-model latency on phone | Model size limits | Offline/PWA | Fit with a Rust/WASM engine | Mark |
|---|---|---|---|---|---|---|---|
| **ONNX Runtime Web, wasm EP** | `ort-wasm-simd-threaded.wasm` ≈ 10–12 MB plus ~0.3–1 MB JS. Brotli roughly 3–4 MB [M]. Conditional import `onnxruntime-web/wasm` keeps JS small; the .wasm is the floor. | none (CPU) | No published sub-5 MB-model phone numbers. B8 measured MobileNetV2 (14 MB) at ~48–67 ms on WebGL and ~15.8× native on CPU across 20 phones. A 1 M-param MLP should be <1 ms warm [I]. First `InferenceSession.create` is hundreds of ms [M]. | Practical limit is memory; B8 reports up to 334× model size RAM in bad cases. Not an issue for <10 MB models. | Model via fetch → Cache API or IndexedDB (B2 recommends IndexedDB). | Two WASM modules (yours + ORT), two memory spaces, JS glue between engine and net. Workable but heaviest. | [V] B1, B2, B8, B9 |
| **ONNX Runtime Web, WebGPU EP** | `ort-wasm-simd-threaded.jsep.wasm` ≈ 19.5 MB | WebGPU | Shader compile on first run dominates; B8 saw GPU warmup 15–35× prediction latency (WebGL, not WebGPU). For a 1 M MLP the GPU path is slower than CPU because dispatch overhead > compute [I]. | same | same | Same as above, plus iOS untested by ORT. | [V-partial] |
| **TensorFlow.js, wasm backend** | tfjs-core + converter + `tfjs-backend-wasm` .wasm; order of 1–3 MB total [M] | WebGPU backend exists; README still says tested on Chrome Canary [V-partial] | B8: SIMD+threads cut latency 54 %; absolute numbers only for 14 MB+ models. | Same as ORT. | Standard; tf.js has IndexedDB model store built in [M]. | Model must be re-exported through tf.js converter from a SavedModel/Keras; PyTorch→ONNX→TF→tf.js chain is brittle [M]. Low maintenance activity signals [V-partial]. | [M] mostly |
| **WebLLM / Transformers.js-style WebGPU** | WebLLM ships MLC-compiled kernels, tens of MB; built for LLMs [M]. Transformers.js v4 uses ORT WebGPU underneath [V-partial]. | WebGPU | Irrelevant for a 1 M MLP. | — | — | Wrong tool. Listed for completeness. | [M] |
| **Hand-written forward pass in Rust → WASM** (weights as flat f32/f16 file) | Adds ~10–50 KB to the engine .wasm [I]. Weights file 1–6 MB. | none; CPU with `std::arch::wasm32` SIMD via `-C target-feature=+simd128` [M] | 1 M MAC in plain WASM at ~1 GFLOP effective ≈ 1–2 ms; with simd128 ~0.3–0.6 ms [I]. No external benchmark. Zero warmup. | none | Weights file fetched once, Cache API. | One WASM module, one memory, zero JS glue for inference. Engine calls `net.forward(&features)` directly. Also usable in the training loop's Rust actors. | [I] / [M] |
| **tract (tract-onnx) in Rust → WASM** | Adds a few MB to .wasm [M]; tract-onnx 0.23.4 (2026-07) [V-partial]; browser via wasm32 documented [V] | none | Not benchmarked here. | Loads ONNX directly. LSTM/GRU support exists in tract [M]. | same | Single WASM module. Pulls in ONNX protobuf parsing. Good middle ground if you want to keep ONNX as the exchange format. | [V] E3, E4 |
| **Burn (burn-import ONNX → Rust code, or NdArray/wgpu backend)** | Burn 0.22.1 / 0.23 pre. WASM supported with explicit init; wgpu device available in browser [V]. Bundle size not stated; likely several MB [M]. | wgpu → WebGPU | Not benchmarked here. | burn-import generates Rust from ONNX; LSTM op coverage not confirmed (page 404) [V-partial]. | same | Single module. Heavier dependency and faster-moving API than tract. | [V] E5 |

### Caching and offline

- Model files are ordinary fetches. Cache them with a service worker (Cache API) or IndexedDB. ORT docs recommend IndexedDB for large models (B2). [V]
- Safari caps script-writable storage at 7 days of non-use. web.dev says home-screen PWAs are exempt; WebKit bugs 232302 and 237350 report that installed apps were hit too. A 2026 report says the same. [V-partial] B10
- Blogs quote a ~50 MB Safari cache quota. No Apple primary source found. For a 1–6 MB model plus a 2–12 MB runtime, this is not binding. [V-partial]
- Practical rule: bundle the weights with the app build and version the URL. If storage is evicted, the next online load refetches. Do not rely on persistence for correctness. [I]

### Which option is simplest to maintain alongside a Rust engine

Hand-written forward pass in the same Rust crate as the engine. Reasons:
- One toolchain (`cargo build --target wasm32-unknown-unknown` + wasm-bindgen), one artifact, no runtime download.
- The same Rust forward pass serves self-play actors during training (no Python in the hot loop) and the phone at inference. One implementation, tested once.
- MLP + GRU is under 200 lines of Rust. There is nothing in these architectures that needs an operator library.
- Falls back to nothing: no WebGPU adapter probing, no SIMD/thread capability detection, no COOP/COEP headers.

tract is the fallback if the architecture grows (attention, many ops) and you want to keep ONNX as the exchange format. ORT Web is the fallback if you later need GPU for a much larger net. [I]

---

## (e) Export path

### PyTorch → ONNX (state 2026-11)

- `torch.onnx.export(..., dynamo=True)` is the default since PyTorch 2.9. The TorchScript exporter is still callable with `dynamo=False`. The automatic fallback to it was removed in 2.11. `dynamic_axes` is deprecated in favour of `dynamic_shapes`. [V] E1, T1
- LSTM and GRU in the dynamo exporter: added by onnxscript PR #2674, merged 2025-11-04. Supports multi-layer, bidirectional, bias/no-bias, `batch_first`, inter-layer dropout. Gate reordering handled (LSTM i,f,g,o → i,o,f,g; GRU r,z,n → z,r,n). [V] E2
- Known pitfalls from that PR: dynamic-shape tests failed with `ConstraintViolationError` and were skipped, so hidden size and possibly sequence dims come out static; a GRU numeric bug (`linear_before_reset`) was fixed later in PR #2853; multi-layer RNNs had a ~10 % perf regression fixed in PR #3111. Packed sequences not mentioned. [V] E2
- Action masking: do it outside the graph. Export logits only. Apply `logits[!legal] = -inf` and softmax/argmax in the host (Rust or JS). Exporting `masked_fill` with a boolean input works in both exporters [M], but keeping it outside avoids dtype issues and lets the engine own legality. [I]
- Export with batch size 1 and fixed sequence length (12 tricks, pad with zeros). Fixed shapes sidestep the dynamic-shape gaps above and make the Rust reimplementation trivial. [I]
- Verify with `torch.onnx.export(..., verify=True)` or run onnxruntime in Python on 1 000 random states and compare to the PyTorch output at 1e-5. [I]

### Alternative: flat weight file + Rust forward pass

- Dump `state_dict()` tensors in a fixed order to one little-endian f32 (or f16) file with a small JSON header (names, shapes). Load in Rust with `bytemuck` or manual `from_le_bytes`. [I]
- Reimplement `Linear`, `ReLU`/`GELU`, `LayerNorm`, `GRUCell`. Match PyTorch's GRU equations exactly (PyTorch: `n = tanh(W_in x + b_in + r*(W_hn h + b_hn))`, i.e. `linear_before_reset=1` in ONNX terms). Test against PyTorch on random inputs. [M] for the equation form, [I] for the plan.
- `safetensors` is the alternative container; the `safetensors` crate reads it in Rust with no_std-friendly code. [M]

### Rust inference crates

| Crate | Version / date | ONNX | WASM | GPU | Notes | Mark |
|---|---|---|---|---|---|---|
| tract-onnx | 0.23.4, 2026-07-08 | load | yes (wasm32) | no | "Tiny, self-contained", pure Rust, nightly benches. 11k commits. | [V] E3 / [V-partial] E4 |
| burn + burn-import | 0.22.1 (2026-10-20), 0.23.0-pre.1 | import → Rust code | yes, explicit init required | wgpu (WebGPU) + CUDA | LSTM op coverage page not reachable; unconfirmed. Faster API churn. | [V] E5 |
| candle | — | partial [M] | yes, examples [V-partial] | CUDA/Metal; no stable WebGPU found | HF-maintained; aimed at LLMs. | [M] |
| ort (pyke) | — | full ORT | not for browser | — | Rust bindings to ONNX Runtime native; not relevant for WASM target. | [M] |

---

## (f) Training stack on this PC

### CUDA on a GTX 1660 SUPER (Turing, sm_75)

- Turing is supported in every current PyTorch build line. CUDA 13.0 "only supports Turing (SM 7.5) and newer" (2.11 notes). The 2026-09-04 notice says CUDA 13.x builds "cover Turing (7.5) through Blackwell". Volta/Pascal/Maxwell are the ones being dropped. [V] T1, T2
- PyTorch 2.14 (released 2026-09-02) ships Windows wheels for CUDA 12.6, 13.0, 13.2. 2.15 (RC 2026-10-28) drops 12.6 wheels. [V] T3, T4
- The 1660 SUPER has no tensor cores. fp16 autocast gives memory savings but little speed-up. Train fp32 or bf16-emulated; a 1 M-param net does not need mixed precision anyway. [M] for hardware, [I] for advice.
- 6 GB VRAM is ample for a 1 M-param model with batch 4096. The GPU will be idle most of the time; self-play throughput is CPU-bound. [I]
- Driver: CUDA 13.x wheels require a recent Game Ready/Studio driver (R580+ class) [M]. Check `nvidia-smi` before choosing cu130 vs cu126.

### Windows and multiprocessing actors

- Windows has no `fork`. `torch.multiprocessing` uses `spawn`, which re-imports the module per actor and pickles everything sent. Start-up cost is seconds per process and shared-memory tensors need `torch.multiprocessing` queues. This is a known friction point but not a blocker. [M]
- NCCL is unavailable on Windows, so `DistributedDataParallel` with multiple GPUs is Linux-only (Isaac Lab docs state this) [V-partial]. Irrelevant for one GPU.
- DouZero's TorchBeast-based actor pool was Linux-only in practice [M].
- If the self-play actors are written in Rust (recommended below), Python never forks. Rust uses OS threads via `rayon` or `std::thread`, which work identically on Windows. The learner is one Python process. This removes the Windows multiprocessing question entirely. [I]

### WSL2

- WSL2 supports CUDA with the Windows NVIDIA driver; no separate Linux driver is installed inside WSL [M, NVIDIA docs]. PyTorch Linux wheels then work, including `fork`.
- Advisable if the stack is Python-heavy with many actor processes, or if you want `pip install torch` to give CUDA 13.0 by default (PyPI default is Linux-only; Windows users use the download.pytorch.org index) [V] T1.
- Not needed if actors are in Rust and Python is only the learner. Native Windows PyTorch + CUDA is fine for that. [I]
- WSL2 cost: file I/O across `/mnt/c` is slow; keep the repo inside the Linux filesystem. GPU memory accounting is shared with Windows. [M]

### Rust ↔ Python bridge

- PyO3 0.29.0 released 2026-06-11. Adds PEP 803 `abi3t` stable ABI for free-threaded builds; drops 3.13t and 3.7; supports 3.14t+ and 3.15 beta. Since 0.28 modules are assumed thread-safe by default. [V-partial] T5
- maturin 1.14 supports abi3t wheels [V-partial]. `maturin develop` builds the extension into the active venv on Windows with MSVC toolchain. [M]
- Pattern that works: Rust crate exposes `play_batch(policy_weights: &[f32], n_games) -> (features, actions, rewards)` as a PyO3 function returning NumPy arrays via `numpy` crate; Python learner trains and hands back weights. Zero-copy via `PyArray`. [M] for crates, [I] for pattern.
- Simpler variant with no FFI at all: Rust binary writes replay shards as `.npy`/`safetensors` files to disk, Python learner reads them, writes weights file back. Slower hand-off (seconds) but no build coupling. Fine for a solo project. [I]

### Pure-Rust training alternatives

| Option | Maturity | Windows CUDA | Verdict | Mark |
|---|---|---|---|---|
| Burn (CubeCL CUDA / wgpu backends) | 0.22.1 stable, active releases, LoRA, multiple optimizers | CUDA feature exists; Windows-specific status not stated in notes | Viable, but you give up the PyTorch ecosystem (optimizers, logging, debugging). API still moving. | [V] E5 |
| tch-rs (LibTorch bindings) | tch 0.24 targets LibTorch 2.9.0 | Works but Windows automatic download does not expose `torch_cuda.dll`; set `LIBTORCH` to a manual cu128/cu130 download | Same kernels as PyTorch. Awkward build on Windows. | [V-partial] E6 |
| candle | active, LLM-focused | CUDA yes | Training support exists but is thinner than Burn. | [M] |

Recommendation on this axis is in (g).

---

## (g) Recommendation (mine)

### Encoding (≈620 features, all in [0,1])

| Block | Width | Content |
|---|---|---|
| Own hand | 54 | one-hot by card |
| Played by each player | 4×54 = 216 | one-hot, players indexed relative to self (self, left, across, right) |
| Current trick so far | 3×54 = 162 | slot-ordered (lead, 2nd, 3rd); zero if empty |
| Talon known cards | 54 | what the declarer saw / took, as visible to this player |
| Discards (declarer's, if visible) | 54 | one-hot; zero when hidden from this seat |
| Contract | ~12 | one-hot over bid levels (three, two, one, solo variants, beggar, valat, etc.) |
| Called king / partner info | 4 + 4 + 4 | called king suit one-hot (4); known partner seat one-hot (4, zero if unknown); declarer seat one-hot (4) |
| Announcements | ~16 | trula, kings, pagat ultimo, king ultimo, valat × (announced / kontra level) as small one-hots |
| Counts | 4 + 1 + 1 | cards left per player /12; trick number /12; points so far /70 |
| Phase | 3 | bidding / exchange / play |

History is covered by the "played by player" zones plus the current trick. This loses the order of past tricks. Add a GRU-128 over the 48 past card-play tokens (card 54 + player 4 = 58 dims each) only if ablation shows the flat version is weaker. DouZero's result says order matters for shedding games; for trick-taking, the Skat net (S3) did without it and was competitive. Start flat. [I]

### Architecture

- Body: 620 → 512 → 512 → 256, ReLU (or GELU), LayerNorm after each hidden layer.
- Heads: policy logits over a unified 54 + ~20 action vocabulary (card, bid, announcement, talon choice); value scalar; optional auxiliary 3×54 sigmoid "where is each unseen card" head, trained with BCE, used only during training.
- Mask illegal actions in the host, not in the net.
- Parameters: 620×512 + 512×512 + 512×256 + heads ≈ 0.32 M + 0.26 M + 0.13 M + ~0.06 M ≈ 0.77 M. fp32 ≈ 3.1 MB; fp16 ≈ 1.5 MB. [I]
- One network for all phases, phase one-hot in the input. Separate nets per phase (Skat style) only if the single net underperforms in bidding. [I]
- Keep the whole thing in the 0.3–1.5 M range. Do not add attention until a flat MLP is measured. [I]

### Deployment to the browser

1. Rust game engine and Rust forward pass in one crate, compiled to `wasm32-unknown-unknown` with `+simd128`, exposed through wasm-bindgen.
2. Weights exported from PyTorch as a flat f16 file (or safetensors) with a JSON header. ~1.5 MB. Served next to the app, cached by the service worker, URL versioned by content hash.
3. No ONNX runtime in the browser. Keep an ONNX export in CI only as a cross-check of the Rust forward pass.
4. Fallback plan if the net ever outgrows an MLP/GRU: tract-onnx inside the same WASM module. ORT Web only if GPU becomes necessary.

### Training stack

- PyTorch (current stable, 2.15 line) on native Windows 11 with the cu130 wheel from download.pytorch.org. Turing is supported. Verify with `python -c "import torch; print(torch.cuda.is_available(), torch.cuda.get_device_capability())"` expecting `(7, 5)`.
- Self-play actors in Rust using the same crate as the browser build, running on 12 threads with the Rust forward pass (CPU). Batched inference on GPU via the Python side is not worth the IPC for a 0.8 M net at this scale. [I]
- Bridge: PyO3 0.29 + maturin. Rust function returns a batch of (features, action, legal mask, return) as NumPy arrays. If the build coupling annoys, fall back to file hand-off.
- WSL2: not needed under this design. Switch to it only if you move actors into Python.
- Do not train in pure Rust (Burn/tch-rs) for the first version. The debugging and tooling cost is higher than the FFI cost. Revisit if PyO3 becomes the bottleneck.

### Top risks

1. **Forward-pass mismatch between PyTorch and Rust.** Mitigation: golden test with 10 000 random states, tolerance 1e-4 fp32 / 1e-2 fp16, run in CI on every weight export.
2. **Flat encoding loses trick order and the agent plays worse than hoped.** Mitigation: GRU-128 over play tokens is a 50-line addition; keep the feature layout stable so both variants share code.
3. **Partner identity is hidden until the called king falls.** The net must infer it. The auxiliary "where is each card" head and a "who is partner" head (4-way) help. If not, the real fix is in the training algorithm (search, belief sampling), which is the other researcher's area.
4. **Self-play throughput on 6 cores.** 12 tricks × 4 players × tiny net is fast; the engine's legality and scoring code will dominate. Profile the Rust engine before touching the net.
5. **iOS storage eviction.** The 7-day cap can drop cached weights even for installed PWAs (B10). Mitigation: weights are a 1.5 MB fetch; refetch silently on next online load; never block the UI on cache presence.
6. **Toolchain churn.** PyTorch moved CUDA defaults three times in a year (T1–T4); PyO3 drops Python versions each release. Pin versions in `requirements.txt` and `Cargo.lock`; record the exact wheel URL.
7. **Suphx-style oracle features are tempting.** They need careful dropout scheduling and doubled feature plumbing. Defer until a plain agent plays legally and beats a rule-based bot.

---

## (h) Verified vs inferred

### Verified by reading the source today [V]

- DouZero: 4×15 card matrices → 54-dim vectors; feature list per role; LSTM over 15 moves grouped as 5×162; 6 FC layers × 512; Q per state-action; 45 actors / 4× 1080 Ti / 30 days. (S1)
- Suphx: 34×838 discard input; CNN without pooling; five models; oracle feature dropout schedule; 44 GPUs per RL run; look-ahead features. (S2)
- Skat imitation: full feature tables, 5 FC layers, dropout 0.6, ≈2.3–2.4 M weights, 2.5 ms/move CPU, accuracies, 8 h + 16 h on one 1080 Ti. (S3)
- Transformer planner: token vocabularies, GPT-2 8×256×8 heads FFN 1024, two heads, 41–72 ms/turn, results vs xinxin and Kermit. (S4)
- SchafkopfRL: 55-dim info vector, 16-dim card tokens, 43-way action, PPO settings, ranking vs PIMC. (S5)
- caniuse: WebGPU 88.6 %, WASM SIMD 96.8 %, WASM threads 97.2 %; Safari iOS 26 WebGPU supported; Chrome Android WebGPU supported; Firefox Android WebGPU off. (B3–B5)
- WebKit: WebGPU ships in Safari 26 on iOS/iPadOS/macOS/visionOS. (B6)
- Chrome docs: Android WebGPU needs Chrome 121+, Android 12+, Qualcomm/ARM GPU. (B7)
- ORT Web 1.31.1 README compatibility table, including "WebGPU not supported" on Safari iOS. (B1)
- ORT Web deploy page: three .wasm files, conditional import, IndexedDB caching advice, no numeric sizes. (B2)
- Browser-inference study: 20 phones, ~15.8× CPU gap vs native, SIMD −36–38 %, threads −13–17 %, both −42–54 %; MobileNetV2 48–67 ms WebGL on phone. (B8)
- PyTorch: `dynamo=True` default since 2.9; `fallback` removed in 2.11; `dynamic_shapes` replaces `dynamic_axes`. (E1, T1)
- onnxscript PR #2674 merged 2025-11-04 adding aten_lstm/aten_gru; dynamic-shape tests skipped; follow-up fixes #2853, #3111. (E2)
- tract runs in browser via wasm32; requires rustc ≥ 1.91; loads ONNX and NNEF. (E3)
- Burn 0.22.1 (Oct 20) and 0.23.0-pre.1 (Oct 29); WASM explicit init; wgpu device; CUDA via CubeCL. (E5)
- PyTorch 2.11: CUDA 13.0 default on PyPI for Linux; Volta dropped from 12.8/12.9; "CUDA 13.0 only supports Turing and newer". (T1)
- 2026-09-04 notice: 12.6 wheels stop at 2.15; Turing kept through Blackwell in 13.x. (T2)
- PyTorch 2.14 Windows wheels: CUDA 12.6, 13.0, 13.2; release day 2026-09-02. (T3)

### Partially verified (snippet or abstract only) [V-partial]

- Suphx residual-block count and channel width (in figures only).
- AlphaHoldem tensor shapes (from OpenHoldem paper snippet).
- Bridge 480-dim input, 4×1024 MLP (from Kita et al. snippet).
- Jass: DMCTS ≈ DNN, rule-based trump ≈ DNN (slide summary); network layers not read.
- ort-wasm-simd-threaded.wasm ≈ 10–12 MB, jsep ≈ 19.5 MB (file listings on mirrors).
- Safari 7-day storage cap and home-screen PWA reports (web.dev + WebKit bugs).
- tract-onnx 0.23.4 dated 2026-07-08; PyO3 0.29.0 dated 2026-06-11; maturin 1.14 abi3t; PyTorch 2.15 RC release day 2026-10-28.
- tch-rs targets LibTorch 2.9.0; Windows CUDA needs manual LIBTORCH.
- Edelkamp 2025: best Skat/Bridge bots remain combinatorial.

### From memory [M]

- DouZero LSTM hidden 128 and `Linear(373+128, 512)` input layer (from the released code).
- Suphx ~50 residual blocks × 256 channels.
- AlphaHoldem ~3 ms per decision on CPU.
- Niklaus Jass DNN trained on Swisslos logs.
- Rong et al. bridge partner-hand prediction network.
- Brotli-compressed ORT .wasm ≈ 3–4 MB; ORT session creation hundreds of ms.
- TF.js wasm backend bundle ~1–3 MB; tf.js IndexedDB model store; converter chain brittleness.
- WebLLM bundle sizes and purpose.
- WASM threads need COOP/COEP headers.
- `std::arch::wasm32` SIMD via `+simd128`.
- PyTorch GRU equation form matching ONNX `linear_before_reset=1`.
- safetensors Rust crate; numpy/PyO3 zero-copy; maturin on MSVC.
- Windows `spawn` semantics; NCCL absent on Windows; TorchBeast Linux-only in practice.
- WSL2 CUDA via Windows driver; `/mnt/c` I/O slowness.
- 1660 SUPER has no tensor cores; CUDA 13 driver requirement.
- candle status; pyke `ort` crate; burn-import LSTM coverage.

### Inferred / computed [I]

- Parameter counts for DouZero (≈1.4 M), transformer planner (≈6–7 M), bridge MLP (≈3.7 M), and the proposed tarok net (≈0.77 M).
- Sub-millisecond phone latency for a ~1 M-param MLP in WASM. No external benchmark exists for nets this small; the claim rests on FLOP counts and B8's relative numbers.
- Tarok state size relative to Skat; sufficiency of a Skat-sized net.
- All recommendations in (g) and the risk list.
