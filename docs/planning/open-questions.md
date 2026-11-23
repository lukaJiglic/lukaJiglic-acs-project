# Open questions

## Where planning stands

- 2026-11-06, session 1: course requirements extracted, first research pass done (4 files), state files created.
- 2026-11-09, session 2: Batch 1 answered (see `decisions.md` D1–D13, F1–F7). Git started, private repo `lukaJiglic/lukaJiglic-acs-project` pushed. Deep research on training, network/browser inference and explanations started (`research/05–07`). Rule-switch list drafted (`rule-switches.md`).
- 2026-11-09, later: Batch 2 answered (D14–D26), C1–C4 answered (D27–D29). Report source copied to `report/`.
- 2026-11-09, later: Batch 3 answered (D31–D42), C5 answered (D30). A Figma connector is available in the user's setup.
- 2026-11-09, later: Batch 4 answered (D43–D49), C6 answered (D50: next session does the screen spec, the Figma wireframe, the pitch slide text).
- 2026-11-09, later: Batch 5 answered (D51–D64), all recommendations accepted.
- 2026-11-09, later: Batch 6 answered (D65–D69); goals G1–G19 with lowered thresholds are in `plan.md` section 4.
- 2026-11-10: Batch 7 answered (D70–D79), all recommendations accepted; plan.md section 5 written. The commit convention (D74) applies from this point.
- 2026-11-10: Batch 8 answered (D80–D87), C7 confirmed (D88). **All eight batches are answered.** `plan-draft.md` renamed `plan.md`; tiers, compliance mapping (section 6), fixed points and demo (section 7), not-in-scope (section 8) written.
- Two actions sit with the user: confirm the topic with the lecturer on 16 Nov (R2); make the repo public or add the instructor by 23 Nov (R5).
- 2026-11-10, session 3: model switched to Claude Fable 5.2 (D89). `screen-spec.md` draft 1 written (D50, first step).
- 2026-11-10: Batch 9 answered (D90–D102). `screen-spec.md` is at draft 2. The user said work can begin (D103).
- 2026-11-10: Figma wireframe built: 28 greyscale frames and one styled sample, file "Tarok trener — wireframe" in the user's Figma drafts (https://www.figma.com/design/XVrAGNVsQuisOd79HYbRBg). Overview image in `docs/wireframe/wireframe-overview.png`. Pitch slide text drafted in `pitch.md`. Spec `docs/specs/001-repo-skeleton.md` drafted, not approved.
- 2026-11-10, later: C8 answered (D104), mode names deferred (D105), building confirmed with training first (D106). `docs/build-plan.md` written: specs 001–009 to the first training run. Specs 002 (engine core) and 003 (training environment and benchmark) drafted. Read-only check of the PC: CUDA-capable driver present, PyTorch installed is CPU-only, Rust toolchain is windows-gnu, maturin missing.
- 2026-11-10, later: Batch 10 answered (D107–D113). The first run trains every phase from the start (D107). Specs 001–003 approved (D112). The trula rule was checked and kept (D113). `docs/build-plan.md` is at draft 2 with twelve specs.
- 2026-11-10, later: spec 001 executed; its files are in the working tree, **not committed, waiting for the user's read of the diff** (D72). A review subagent found 13 defects; 12 fixed, one deviation left for the user (an extra file, `tools/check_trailers.py`). Specs 002 and 003 are running in two parallel sessions (`engine-core`, `train-env`) on the shared checkout. `docs/encoding.md` draft 1 and spec 004 written.
- 2026-11-10, evening: specs 002 and 003 executed by the parallel sessions, each reviewed by a separate subagent and fixed. Verified by the main session: 47 engine tests, 23 training tests, `envcheck` exit 0. Results are in each spec and in `docs/progress.md`. **Nothing from specs 001–003 is committed** (D72: the user reads the diff first). Incident: the root `rust-toolchain.toml` made rustup auto-install an MSVC toolchain; it was removed and the file is disabled as `rust-toolchain.toml.disabled`.
- 2026-11-10, night: the user read the diffs and approved the commit (D114). Specs 001–003 are committed and pushed. The first `engine` CI run failed on an unpinned compiler; fixed in `56f4caf` (D115). All three workflows are green. Specs 001 and 002 are done; 003 waits for the GPU-free run.
- 2026-11-10, night: the layout is L2 (D116). Specs 005 (deal flow A, 53 criteria) and 007 (model and PPO learner, 24 criteria) are drafted.
- 2026-11-10, late: specs 004, 005 and 007 approved (D117–D119). Specs 005 and 007 are in execution in the two track sessions (`engine-core`, `train-env`). `tools/check_encoding.py` (spec 004, criterion 2) is written and passes; it is uncommitted until the user has read it. Spec 006 (encoding in Rust, vectorised environment, bindings, game record; 28 criteria) is drafted.
- Spec 006 approved (D120).
- 2026-11-11: specs 005 and 007 executed, reviewed, fixed and verified. The score has ten parts (D121). The user gave the go-ahead to commit (D122); five commits pushed, all three workflows green on `819220d`. Specs 003, 004, 005 and 007 are done. Spec 006 is in execution in the `engine-core` session.
- 2026-11-11, evening: spec 006 executed by `engine-core` (174 engine tests, 89 training tests in the CI set, verified by the main session). The review pass found no leak, no unsound bit and no wrong reward, but six planted wrong rules that the tests let through, loose array checks in the binding and three overclaims; the fixes are with the session. Nothing of 006 is committed.
- 2026-11-11, night: all 19 review items of spec 006 are fixed and verified by the main session (180 engine tests, 169 without the thread feature). **Spec 006 waits for the user's read of the diff.** Its two workflows are unverified until the push.
- 2026-11-12: the user answered everything open: spec 006 committed (D123; `8f24dbb`, `8cb2bb0`), spec 008 approved (D124), Batch 11 (D125–D129), Batch 12 (D130), C11 (D131). Three rulings change the engine: a valat does not cancel a lost mond, pagat ultimo follows the side, the pagat wins a trula trick only if it falls last. They are spec 013. `docs/encoding.md` has the readings of Batch 12 in section 4a.
- 2026-11-12, later: C12 answered (D132).
- 2026-11-12, later: spec 013 executed and reviewed (no code defect; test gaps in fixing). The review found a fifth valat case; the user ruled it (D133: in the declarer's valat the untaken talon is the declarer's, a mond in it is not lost).
- 2026-11-12, evening: spec 013 is fixed after the review, D133 is built, and the main session verified it (182 engine tests). **Spec 013 waits for the user's read of the diff.**
- 2026-11-12, evening: the user read the diff of spec 013 and approved the commit (D134; `ce175f6`).
- 2026-11-12, night: spec 008's code and short tests are built by `train-env` (138 tests pass here with the GPU tests). The user agreed with the open proposals (D135, D136). The review pass over spec 008 runs; the three long runs wait for it.
- 2026-11-12, late: spec 008 is executed: review fixes in, the three smoke runs done. Both networks beat random play (+0.60 per deal), the control run does not (−0.21). Verified by the main session (159 tests; the checkpoints evaluated again on other deals). **Spec 008 waits for the user's read of the diff.**
- 2026-11-23: the user read the diff of spec 008 and approved the commit (D137; `124b2e9`), and ordered a longer run on flow A (D138; spec 014, in the `train-env` session). C13 and C16 were asked in the same message and not answered; they stay open.
- 2026-11-24: the GPU was free at 00:04 and the longer run of spec 014 started (MLP first). Spec 009 (deal flow B: announcements and kontra) is drafted from a new read of pagat.com; its seven open points are Batch 13 below.
- **Waiting on the user, to decide:** Batch 13 and the acceptance criteria of spec 009; C13 (the new figure of G3); C16 (which entropy mean the controller steers on); the open points A–D of spec 014.
- **Dates:** the repo must be public or shared with the instructor on Mon 23 Nov (R5); it is private on 2026-11-23. The proposal, wireframe and pitch slide were due Wed 18 Nov; the files hold no record that they were handed in.
- **Waiting on the user:** corrections to the wireframe and `pitch.md`; the proposal format. Later: C13 (the new figure of G3, after the smoke run), C14 (tier of the trula toggle).
- Next: spec 013 in the `engine-core` session and spec 008 in the `train-env` session; verification, review pass, the user's read of each diff, commit.
- The proposal, Figma wireframe and pitch slide are due Wed 18 Nov. The product batch must follow soon after the bots batch.

## Clarifications

Answered 2026-11-09: C1 (by D16), C2 (D27), C3 (D28), C4 (D29), C5 (D30: both certain facts and derived counts), C6 (D50). Answered 2026-11-10: C7 (D88: explicit end-session action settles leftover radelci at −70 each).

Answered 2026-11-10: C8 (D104: yes, the adviser contrasts every phase in T0), C9 (D105: names later), C10 (D106: building confirmed, training first).

Answered 2026-11-12: C11 (D131: the user's table plays "the pagat wins only if it falls last"; a toggle, S40, with that as the Default); C12 (D132: the pagat wins when played after both the škis and the mond; a fourth card may follow); C15 (D135: replay keeps going on a record of another engine version and names both versions on a mismatch; built with spec 009).

Open:

- **C13. The new figure of G3** (D128). Measured in spec 008, end to end on this PC with 512 tables and two epochs: transformer 6,506 decisions played per second and 3,194 trained; MLP 34,743 and 17,050. (a) at least 5,000 decisions played per second with the transformer; (b) at least 6,000 played; (c) at least 3,000 trained. Recommended: (a); it is met with a margin of 30%, which flow B's longer sequences will use up in part. Asked 2026-11-12.
- **C16. Which entropy mean the controller steers on.** Now: the mean over all decisions of a phase, forced ones included. In the smoke runs the share of forced decisions in the bidding went from 28% to 0%, so the mean moved without the policy getting firmer or looser. (a) the mean over decisions with two or more legal actions, with the targets set again as a quarter of random play under that mean; (b) keep the mean over all decisions. Recommended: (a). Asked 2026-11-12.
- **C14. Tier of S40's other option** ("in any order"). Recommended: like S14 and S18 in D98: selectable in T0 with the note "agent ni učen s tem pravilom". Asked when the engine gets its toggles.

## Batch 13 — announcements and kontra (asked 2026-11-24; open)

The open points of `docs/specs/009-deal-flow-b.md`. Source: pagat.com, Slovenian Tarok, read again on 2026-11-24. Each point is a place where the page is silent or where the user's table decides. The first option is the recommendation.

| # | Question | Options | Recommended, and why |
|---|---|---|---|
| B13-1 | May both sides announce the same bonus? | (a) each bonus once in a deal; the other side answers with kontra. (b) each side may announce it | a: the page has one kontra ladder per bonus and so has the encoding; b makes the encoding version 2 |
| B13-2 | Kontra on a bonus when the announcer's side is hidden | (a) only a seat that knows the announcer's side may say it. (b) every announcement shows the announcer's side | a: the page's rule ("knows for certain"); the legal actions then leak nothing |
| B13-3 | The duty to keep an announced ultimo card | (a) the engine enforces it: not a legal play before the last trick while another card is legal. (b) free play | a: the page's rule; an agent cannot break it by mistake |
| B13-4 | Announced pagat ultimo, the pagat's side takes the last trick with another card | (a) won, like the silent one (D126). (b) the pagat itself must take the trick | a: one rule for silent and announced |
| B13-5 | An announced valat that fails | (a) −500 and nothing else is scored. (b) −500 and the game is scored as usual | a: the page says a valat removes all other scores |
| B13-6 | Order of the round | (a) the declarer first, then in order of play; several things per turn; ends after three passes in a row. (b) another order | a: the page's rule with a fixed order added |
| B13-7 | Kralj ultimo by a declarer who holds the called king | (a) may announce it. (b) only a partner may | a: the silent one is already scored in this case (spec 005, answer C) |

Four readings about the machine side are in the spec under "Readings chosen by Claude" ("done" writes no token; an announcement of unknown side; when the partner is shown as public; the size of the reward under kontra). They are not table rules. They stand unless the user objects.

## Batch 12 — readings of the encoding contract (asked 2026-11-11, answered 2026-11-12: a for all fifteen; see D130 and `docs/encoding.md` section 4a)

Context: the `engine-core` session read `docs/encoding.md` against the engine before writing spec 006 and found eight places the contract leaves open (B12-1 to B12-8). The review pass over spec 006 added three (B12-9 to B12-11) and the session asked four more (B12-12 to B12-15). None changes a size or an id. The session builds the recommended reading of each in one marked place, so another ruling is a small edit. Checked by the main session against `docs/encoding.md` sections 3, 4, 6, 7 and the learner's `contract.py` and `ppo.py`. After your answer the contract is amended; it stays version 1, because no model has been trained on it.

| # | Place in the contract | Question | Options | Recommended | Reason |
|---|-----------------------|----------|---------|-------------|--------|
| B12-1 | G1, scoring toggles (19 bits) | The engine has no toggles yet. What do the bits hold now? | a) the one-hot of the Default preset's values; b) all zero | a | The inputs then mean the same when toggles arrive. "One-hot each" already implies one bit per toggle. |
| B12-2 | Critic, "the true partner seat" (4) | Whose partner? | a) the declarer's true partner, relative to the acting seat, like B6; b) the acting seat's own team-mate | a | The contract's wording. The same information for the critic either way. |
| B12-3 | Partner target | What is the target when the acting seat has no single team-mate: the declarer alone, an opponent in a one-against-three game, every seat during the bidding? | a) "none", and the loss skips the decision; b) something else | a | The learner already skips "none". An opponent in a two-against-two game keeps a normal target: the other opponent. |
| B12-4 | F5, "own card is the highest unseen of its suit" | Which own cards get the bit? | a) every own card that no unseen card of its suit beats (king and queen in hand: both); b) only the single highest own card | a | (a) marks every sure winner of the suit. |
| B12-5 | F3, "certainly holds", "includes the revealed called king" | The partner is public only when the king is played, and then nobody holds it. Reading: a king or trula card in the packet the declarer took is certainly in the declarer's hand until played; so is a taken tarok that was not shown as a discard. Plus plain elimination. | a) this reading; b) elimination only | a | All of it follows from public facts: kings and trula cannot be discarded, discarded taroks are shown. The soundness test (criterion 8) covers each source. |
| B12-6 | F6, two counts | "Card points unseen / 70": the unseen cards counted as one pile. "Most taroks a seat can still hold / 12": the smaller of its hand size and the taroks it can still hold, capped at 12. | a) as written here; b) other | a | The cap keeps every feature in 0 to 1; the declarer holds 15 cards during the exchange. |
| B12-7 | Token seat field | Which seat do tokens without an actor carry? | a) the start token and the six shown talon cards: "no seat"; contract, called king, taken packet, shown discard: the declarer; b) other | a | The declarer did those four things. Nobody did the other two. |
| B12-8 | Record and deal seeds | Three details not fixed anywhere: the seed in the JSON record, the mixing function for (base seed, table, run, deal), the first dealer of a run. | a) seed as a decimal string; SplitMix64 mixing; first dealer of run r at table t = (t + r) mod 4; all frozen by a golden test; b) other | a | A 64-bit seed does not survive JavaScript numbers. The rest only has to be fixed and written down. |
| B12-9 | C4, "last bid per seat" | Found by the review of spec 006. The code wrote the seat's last call, so a pass erased its bid; after the auction C4 said the same as C3. | a) the seat's last real bid, kept after a pass, zero if it never bid; b) the last call, a pass included | a | The MLP reads only the state. With (b) it cannot see what a seat bid before it passed. The code now does (a). A seat that passed without ever bidding has all thirteen entries at zero; C3 holds the pass. |
| B12-10 | F4, "card is unseen" | The taken packet and the declarer's own discards count as seen. So the card-location head is never trained on a taken card, although for the other seats a taken suit card may be in the declarer's hand or in the hidden discards. | a) as built; b) a taken card whose place is not certain counts as unseen | a | The contract's words: "not shown". The shown-talon and taken-packet features already carry the cards. Look again if discards are played poorly. |
| B12-11 | B7, E3, E4, small readings | "Called king lies in the shown talon" stays set after the king is taken and played. E3 rounds each seat's pile of card points on its own. E4's trick number is 0 before play and 12/12 at the end. | a) as built; b) name the one to change | a | None loses information. |
| B12-12 | Repeated token sequence | The declarer puts down two or three cards one at a time and a hidden discard is no token. Between those decisions the tokens are equal; only the state token differs. | a) leave it; b) a new token "the declarer put a card down", without the card: one new event id in the contract | a | The model reads the state token at every decision. Look again if the smoke run shows poor discards. |
| B12-13 | Engine tests | The engine's tests now play several hundred thousand deals. Unoptimised, the soundness test alone took 215 seconds. | a) test builds are optimised (`opt-level = 2`, overflow checks stay on); b) the long tests run only with a flag | a | One command runs every test, here and in CI. |
| B12-14 | `EngineEnv.observe()` | It returns the same NumPy arrays on every call and the next step overwrites them. The mock returns new arrays. | a) keep reusing; the caller copies what it keeps; b) new arrays per call, a copy of about 14 MB per step at 1,024 tables | a | The trainer already copies; the review checked every place. The rule is written in the docstring and the README. |
| B12-15 | A stale Python module | After a change in `src/engine` the installed module must be built again, or Python runs the old rules with no sign of it. | a) a check: the module carries a hash of the engine's sources from build time, and `EngineEnv` refuses to start when the files on disk differ; about 40 lines, built with spec 008; b) the README rule only | a | A stale module would train on old rules silently. |

Not a question: the trainer (`train.py`) still builds only the mock environment. Switching it to the real one is in spec 008's scope.

## Batch 11 — rulings and settings found while building (asked 2026-11-11, answered 2026-11-12: 1 b, 2 b, 3 a, 4 b, 5 b, 6 a, 7 b, 8 a, 9 a; see D125–D129)

Context: specs 005 and 007 are done. Their sessions and the two review passes left these points. None blocks spec 006. Items 4 to 6 should be answered before the first real run; they can also be settled inside spec 008. Sources: `docs/specs/005-deal-flow-a.md` ("Readings and deviations", "Questions for the user"), `docs/specs/007-model-and-learner.md` ("Questions for the user"), `docs/progress.md`.

| # | Question | Options | Recommended | Reason |
|---|----------|---------|-------------|--------|
| B11-1 | **Valat and a lost mond.** One side takes all twelve tricks and someone lost the mond. Four cases: (i) declarer's side takes all, an opponent loses the mond to the škis; (ii) declarer's side takes all, one of the two loses the mond to the other's škis; (iii) opponents take all, the declarer loses the mond or leaves it in the talon; (iv) opponents take all, one of them loses the mond to another opponent. | a) the valat cancels the −20 in all four (the code now); b) the −20 is written next to ±250 in all four; c) your table's answer per case | a | Checked: research/01 B3 says a valat cancels all other scores of the hand. Inferred: that this includes the mondfang; no source names the case. Five tests pin (a). |
| B11-2 | **Pagat ultimo when the pagat's own side wins the last trick with another card** (the partner's pagat is in the trick, the declarer takes it with a higher tarok). | a) lost, −25 for that side (the code now); b) won | a | The usual meaning: the pagat itself must take the last trick. |
| B11-3 | **Kralj ultimo when the called king is in the last trick and the declarer's side wins it with another card.** | a) +10, the side decides (the code now, criterion 38); b) the king itself must take the trick | a | The spec's criterion as approved (D118). Your table decides. |
| B11-4 | **Weight of the actor's own value head in the loss.** Measured: at the transformer's first update its gradient on the trunk is 15 times the policy's. | a) 0.5 (now); b) 0.1; c) leave the head out of the loss until the app needs it | b | The critic, not this head, drives PPO. The head is for the app's explanations. 0.1 keeps it learning without steering the trunk. |
| B11-5 | **First scale of the policy head.** | a) 1.0 (now; learns fastest on the mock, first policy not uniform); b) 0.3, then measure on the smoke run; c) 0.1 (uniform start, slower) | b | 0.01 did not learn the stronger mock task; 1.0 may be part of the early entropy collapse. A middle value, checked by measurement. |
| B11-6 | **An entropy controller before the first real run** (target entropy per phase, research/05 e7). Measured: the transformer's policy entropy is near zero from update 20 on the token task, and it stops at 0.81. | a) yes, built and tested before the first real run; b) only if the monitors show a collapse | a | D107 trains every phase from the start; a collapse in bidding is the documented failure there. The mock already shows one. |
| B11-7 | **G3, 10,000 steps per second.** Measured and derived: the transformer learner alone reaches about 7,700 with two epochs; the MLP is far above. | a) G3 stays for the pipeline, measured with the MLP; the transformer's number is reported next to it; b) G3 is lowered to a figure the transformer can reach; c) G3 stays as written and is reported as not met | a | The goal was set to show the pipeline is fast enough. The transformer's limit is the GPU, and the report should say so with numbers. |
| B11-8 | **The local compiler pin** (`rust-toolchain.toml.disabled`, untracked). | a) delete the file; the pin lives in the `engine` workflow only; b) switch the rustup default host to GNU and restore the file | a | CI already pins 1.94.1 (D115). The file caused the MSVC install incident. Option (b) changes a machine-wide setting of yours. |
| B11-9 | **"Fant" or "pob"** for the jack, in code and in the app. | a) fant (the code now); b) pob | a | research/01 and the Tarok.net rules use "fant". Your table's word wins if it differs. |

Moved into spec 008 (the smoke run), not asked here: the number of tables for the first run (512, because the acting cache of 1,024 tables does not fit in 6 GB) and the number of seat-deals per update.

## Batch 10 — build decisions (asked 2026-11-10, answered 2026-11-10; see D107–D113 and `docs/build-plan.md` draft 2)

Context: the user wants training running as soon as possible (D106). `docs/build-plan.md` gives the path. These are the choices in it that are the user's. Sources: research/05 (stages, failure modes, throughput arithmetic), research/06 (training stack), and read-only checks of the PC on 2026-11-10. The throughput numbers are inferred by arithmetic; spec 003 measures them.

**B10-1. What the first training run covers (stage 1).**
- a) The network learns card play. Rule-based code bids, calls the king, picks the talon packet and discards; their tokens are in the sequence (D21). Contracts: tri, dve, ena, solo tri, solo dve, solo ena. If the other three pass, forehand plays tri (legal in the baseline), so no klop yet. No announcements, no kontra. Silent bonuses and mondfang scored. Radelci off.
- b) One fixed contract only (tri with a called king), as research/05 first sketched.
- c) Everything from the start, including klop, berač and announcements.
- Recommendation: **a**. It gives the agent the real mix of partner and solo games. b is quicker by a day at most and teaches less. c puts the passive-bidding failure (research/05 e1) in front of the first result.

**B10-2. Engine order.**
- a) A vertical slice first: the rules stage 1 needs (specs 002 and 005). Klop, berač, solo brez, valat, announcements, kontra, radelci and the other toggles follow while the first runs train. The encoding reserves their slots from the start, so old checkpoints stay loadable.
- b) The whole T0 rule set with all tests before any training.
- Recommendation: **a**. It is what "as soon as possible" needs. G1 (150 tests, every T0 toggle) is still met, later.

**B10-3. Two tracks at once.**
- a) After spec 001, the Rust track (002, 005, 006, 007) and the Python track (003, 008) run in separate sessions at the same time. They touch different folders. Spec 004 (the encoding document) is the contract between them.
- b) One spec at a time.
- Recommendation: **a**. It can halve the waiting time. The cost is yours: you read two streams of diffs (D72 step 3).

**B10-4. How the state token sits in the sequence (D18, D30).** By arithmetic, recomputing the whole sequence for every decision gives about 600 steps/s on this GPU; G3 asks for 10,000. See `docs/build-plan.md` section 5.
- a) Play and bid tokens are computed once per seat and deal. Each decision has its own state token that looks at the tokens before it, through an attention mask. Play tokens never look at state tokens. Training is one pass per seat and deal; acting reuses cached tokens. Estimated about 5,000 steps/s or more.
- b) The state token is appended last and everything is recomputed per decision. Simplest. Estimated about 600 steps/s.
- c) Measure both in spec 003, then decide.
- Recommendation: **a as the working design, with spec 003 measuring both**. It keeps your design (a state token inside the transformer) and changes only how attention is masked.

**B10-5. Where the network runs during the first self-play runs (D22, D54).**
- a) In Python on the GPU, in batches, with the Rust engine stepping many games at once. The hand-written Rust forward pass (D54) is built after training has started, and is then measured as the alternative for actors (D22) and used for the browser.
- b) The Rust forward pass first, actors on CPU threads.
- Recommendation: **a**. It needs no second implementation of the network before the first run. By arithmetic a 3 M-parameter transformer on CPU threads would be far slower than the GPU.

**B10-6. Approve specs 001–003 and their open points.**
- 001 repo skeleton: A force LF line endings (recommended yes); B licence holder "Luka Jelenko Iglic"; C a `tools/` folder for check scripts (recommended yes).
- 002 engine core: A Slovenian card names in code (recommended yes); B "fant" or "pob"; C check the trick and legal-card rules (criteria 11–25) against how you play.
- 003 training environment: A TensorBoard (recommended yes); B benchmark with the GPU free and as used daily (recommended both). It downloads about 3 GB into `.venv`.
- Recommendation: approve all three. 001 runs first; 002 and 003 right after (in parallel under B10-3 a).

## Batch 9 — screen spec (asked 2026-11-10, answered 2026-11-10; see D90–D102 and `screen-spec.md` draft 2)

Context: `screen-spec.md` draft 1 describes the T0 screens. These are the points where Claude had to choose and should not. Sources: decisions D2–D88; `research/07` (f) and (c) for the explanation layers and the over-reliance risk; `research/02` (e) for ideas from other apps; `research/01` for the direction of play. Layout sizes and tap counts are Claude's estimates, not measurements.

**B9-1. Navigation.**
- a) Home is a hub with three large tiles (Learn, Play, Table) and small links (Rules, Levels, Settings). No tab bar. The table is full screen with a back button and a small menu.
- b) Bottom tab bar on every screen.
- c) Side menu.
- Recommendation: **a**. The table needs every pixel of a 360 px screen; a tab bar costs about 56 px of it.

**B9-2. Direction of play on screen.** The sources disagree (`research/01` point 2): anticlockwise in pagat.com and the 4-player texts, clockwise in the TARZS and Tarok.net competition rules. The engine does not care; it is a display choice.
- a) Anticlockwise.
- b) Clockwise.
- c) A setting, default as at your table.
- Recommendation: **the direction your table uses**, as a fixed choice in T0; a setting can come in T1. Which is it?

**B9-3. Hand layout (12 to 15 cards on a 360 px wide screen).**
- a) Two rows: taroks above, suits below, sorted, slight overlap.
- b) One overlapping fan, sorted.
- c) One row with sideways scroll.
- Recommendation: **a**. Every card stays tappable without scrolling; the split matches how players sort a hand.

**B9-4. Coach interrupt in coached play (D34).**
- a) A banner over the explanation strip. The card is held. Buttons: "Vzemi nazaj", "Vseeno igraj", "Zakaj?". Three strictness levels (blago / srednje / strogo); the point values are set after the Q head is calibrated (G10). Card play only in T0; bidding, call, talon and announcements are graded in review, not interrupted.
- b) A full-screen stop.
- c) As a, but also interrupt in bidding, talon and announcements in T0.
- Recommendation: **a**. A banner keeps the table visible. Bidding advice is already T1 (D36). The idea of an adjustable stop comes from GNU Backgammon's tutor mode (`research/02` e7).

**B9-5. Information rule and hints.** Explaining why a bot played a card uses that bot's hidden hand. Shown during the deal, it would leak cards.
- a) During a deal the app explains only the user's own seat ("what would the agent play with my cards"). Other seats' moves are explained only after the deal, in review. In Play mode hints are off by default; a "Namig" button gives one and marks the deal "s pomočjo".
- b) As a, but no hints at all in Play mode; hints exist only in Learn.
- c) Explain every seat live (leaks hidden cards).
- Recommendation: **a**.

**B9-6. Move grades in review.**
- a) Three classes by points lost against the agent's choice: v redu / netočno / napaka.
- b) Five classes as on Chess.com (best, good, inaccuracy, mistake, blunder).
- c) Numbers only, no classes.
- Recommendation: **a**. Two thresholds to calibrate instead of four; the number is still shown.

**B9-7. Adviser: corrections and contradictions.**
- a) "Razveljavi" steps back one entry at a time, any number of steps. An entry that contradicts earlier play (a seat that showed void now follows suit) gives a warning and can be kept, since real tables make mistakes.
- b) Any earlier trick can be edited directly.
- c) Contradicting entries are blocked.
- Recommendation: **a** for T0.

**B9-8. Adviser extras in T0.**
- a) Show the deal score at the end of an entered deal. The engine has every card by then, so it costs nothing. Talon and bidding contrast wait for the T1 bidding adviser.
- b) No score in the adviser until the T1 scoring calculator.
- Recommendation: **a**.

**B9-9. Toggle values the first model is not trained on.** `rule-switches.md` defines T0 as "every option is in the first trained model". D20 fixes the play-changing toggles for the first model. S14 (discarding taroks) and S18 (kontra levels) are T0 and change legal moves, so the two statements disagree for their non-default values. (S6 is pinned off, D44.)
- a) The app lets the user pick them. The bots still play legally through the move mask. A visible note says "agent ni učen s tem pravilom" and the strength figures in Levels apply to the Default preset only.
- b) Lock S14 and S18 to their defaults until the rule-vector model; retier their other values to T1.
- c) Add S14 and S18 as inputs to the first model and randomise them in self-play, like the scoring toggles (changes D20).
- Recommendation: **a**. The engine supports and tests them anyway (G1); the note keeps the claim honest. c is the cleaner end state but changes the training design; it can be revisited at stage 3.

**B9-10. Wireframe fidelity, frame and labels.**
- a) Low-fidelity greyscale frames (boxes and labels), about 26 of them, plus one styled sample frame of the table in your craft palette. Frame 360 × 800. Slovenian UI labels, English frame titles and notes for the lecturer.
- b) Every frame styled.
- c) Greyscale only, no styled sample.
- Recommendation: **a**. The course asks for a wireframe; the styled sample shows the direction without spending the week on it.

**B9-11. Lesson topics for T0 (at least three, G17).** Claude's proposals from general tarok knowledge, not from a source:
1. Štetje tarokov: how many taroks are still out, and who has them.
2. Kje je kralj?: reading who holds the called king before it falls.
3. Varovanje monda: keeping the mond away from the škis.
4. Pagat ultimo: when to try it and how to stop it.
5. Klicana barva: why the opponents lead the called suit.
6. Klop: how to take few points.
- Recommendation: **1, 2, 3** for T0, the rest T1. You know the game better; replace any of them.

**B9-12. Rule cards (at least eight, G17).** Proposed list: klop; mondfang; trula in one trick (the pagat wins); kralj ultimo; pagat ultimo; valat; radelc; kontra and its levels; berač (must overtake, pagat last); called king in the talon; talon cards left to the opponents.
- a) Accept the eleven.
- b) Change the list.
- Recommendation: **a**.

**B9-13. Names.** The pitch slide and the Home screen need them.
- App name: a) "Škis"; b) "Tarok trener"; c) your own.
- Mode names: a) Učenje · Igra · Svetovalec; b) Uči se · Igraj · Za mizo; c) your own.
- Recommendation: yours to choose. Claude's suggestions: "Škis" (short, the highest card) and option a for the modes.

## Batch 8 — tiers and demo (asked 2026-11-10, answered 2026-11-10; see D80–D87 and plan.md sections 1, 5–8)

Context: this is the last planned batch. It fixes what the proposal promises (tiers), the order in which work happens (no calendar, D26), what the 10-minute demo shows (R3), and what the checkpoint deliverable contains (D2 in `course-requirements.md`). Sources: the course text for the deadlines and formats; everything else is Claude's proposal built on D1–D79.

**B8-1. The T0 list (what the proposal promises as "must work for the demo").**
Claude's reading of D31–D64 gives this T0 set:
- Engine: 4-player rules on the Default preset, the 12 T0 toggles, scoring, seeded deals, game records, CLI. Tests G1–G2.
- Bots: random, rule-based, trained transformer through at least stages 1–2 (card play, talon, bidding, announcements, kontra, klop, berač), levels from checkpoints. MLP control run (G7) and search benchmark (G6) are measured and reported, not shown in the demo.
- App: Home, Learn hub, Table in Lesson / Play / Adviser modes, Review, Rules and presets, Levels, Settings. PWA on GitHub Pages, offline.
- Learn: coached play, review, at least 3 lessons and 8 rule cards (G17).
- Adviser: full entry of a real deal (G16).
- Explanations: default layer and tap layer (D16), review contrast with one counterfactual, faithfulness goals G8–G13.
- Process: CI, specs, progress file, AI log, trailers (G19).
- a) Accept this T0 list as final for the proposal. T1–T3 as in plan.md section 1.
- b) Trim: name the items to move to T1.
- c) Extend: name the items to pull into T0.
- Recommendation: **a**. It matches the goals already fixed in Batch 6; moving items would reopen them.

**B8-2. Fallback if the trained agent is weaker than hoped at demo time.**
- a) The app always ships the best measured checkpoint, whatever its strength. The demo shows that model and its real numbers. The report states what was reached (R15 asks for failures too). Nothing is hidden behind the rule bot.
- b) Demo against the rule bot only if the trained agent fails G4.
- Recommendation: **a**. The levels system (D24) already exposes checkpoints; honesty is graded (R4, R15).

**B8-3. Order of work (a sequence, not a calendar).**
- a) (0) Repo skeleton: README with goals, `CLAUDE.md`, `docs/progress.md`, CI, licences. (1) Engine stage 0: rules, Default preset, T0 toggles, tests, CLI, records, random and rule bots, seeded deals, speed harness and gate. (2) Training pipeline: PyO3 actors, learner, stage-1 card-play run started. (3) App shell while training runs: Table component, Play vs rule bot, cards, PWA, Pages. (4) WASM export of engine and forward pass, first checkpoint in the browser, levels. (5) Stage-2 training (talon, bidding, announcements, klop/berač), monitors. (6) Explanations: signals, templates, aux-head readouts, attributions, counterfactuals. (7) Learn: coached play, review, lessons, rule cards. (8) Adviser entry. (9) Evaluation: benchmarks, MLP control, faithfulness checks, phone measurements. (10) Report and demo script. Training runs are long and unattended, so app work fills the GPU time.
- b) Engine → full app → training last.
- c) Engine → training to completion → app.
- Recommendation: **a**. Bots come first (D7) but the GPU should not sit idle, and the course has a checkpoint where something must run.

**B8-4. What the 10-minute demo shows.**
- a) Scripted on a seeded deal, phone in hand, laptop as fallback. (1) 1 min: the problem, the agent, the three uses. (2) 3 min: play a deal against the agent in the app; open the explanation layers on one or two moves. (3) 2 min: coached play: the learner's card vs the agent's card, the contrast, one counterfactual. (4) 2 min: adviser on a real deal with physical cards, or a prepared deal if time is short. (5) 2 min: numbers: goals table with measured values, tests and CI, commit log with trailers, AI log. Script in `docs/demo.md`; the seeded deal is fixed so the demo repeats.
- b) Free play and questions.
- Recommendation: **a**. R3 asks for "demoable in 10 minutes"; a fixed script and seed make it repeatable.

**B8-5. Checkpoint deliverable (sprint-1 milestone, self-submitted Sun 13 Dec; 5-min lightning talk Mon 14 Dec; 10%).**
- a) A written sprint report `docs/sprint-1.md`: goals table G1–G19 with status and measured values so far; what was built; stage-0 measurements (test count, speed gate result, per-deal score spread); what broke and what was dropped; what the next sprint does; the AI-log excerpt for the sprint. The talk shows one live thing (engine CLI tournament, or the app shell playing vs the rule bot) and the goals table. The aim for the sprint-1 content is order-of-work steps 0–3 (engine, pipeline, stage-1 run started, app shell). The aim is a target, not a plan against dates (D26): the report shows whatever exists.
- b) Slides only.
- Recommendation: **a**. It reuses `docs/progress.md` and the goals table; nothing is written twice.

**B8-6. Report writing cadence (ERK report, 11 Jan).**
- a) Skeleton in `report/` from the start of building: section headings, stack table, goals table pulled from README. Method and stack table updated at the end of each session that changes them. Results filled as measurements land in `docs/progress.md`. Final pass at the end.
- b) Written at the end from the files.
- Recommendation: **a**. The AI-use declaration (R17) and stack table (R13) are hard to reconstruct late.

**B8-7. "Not in scope" list.**
- a) README and plan carry a short "Not in scope" section: 3-player tarok (after T0, if time), online multiplayer (T3), LLM narration (never), code from other tarok engines (never), the rule variants in the left-out list of `rule-switches.md`, styles as fine-tuned copies (T2).
- b) No such list.
- Recommendation: **a**. It tells the reader what the goals do not claim.

**B8-8. Plan document for the proposal.**
- a) After Batch 8, `plan.md` is renamed `plan.md` and tidied: decisions referenced by number, open items marked, sections 6 (compliance mapping) and 7 (fixed points only) completed. The proposal text is derived from it.
- b) Keep the draft as is and write the proposal separately.
- Recommendation: **a**. One source for the proposal, README and report.

## Batch 7 — working with agents day to day (asked 2026-11-09, answered 2026-11-10; see D70–D79 and plan.md section 5)

Context: the course grades the process (R4, R9, R14, R17): how work was split, which files carried state, how output was verified before each commit, and which prompt and model produced which part. Primary tool: Claude Code with Claude Fable 5.1 (F6). The user reviews TypeScript/Vue, Python, Rust, C++ (F5). Progress is recorded thoroughly instead of dates (D26). These are Claude's proposals; the course text is the only source.

**B7-1. Unit of work: task specs.**
- a) Every task is a spec file `docs/specs/NNN-short-name.md`: goal, acceptance criteria (the tests or checks that prove it), files expected to change, out of scope, the model and tool that will run it. Claude drafts, the user approves, an agent executes, verification runs, the commit references the spec. Specs are numbered and never deleted; status at the top (draft / approved / done / dropped).
- b) GitHub Issues as the task unit.
- c) Chat only, no task files.
- Recommendation: **a**. Files carry state across sessions (G6) and are the "specs, progress files" the report template asks about. Issues can mirror specs later if useful.

**B7-2. State files.**
- a) `docs/planning/` (decisions, open questions, plan, research, AI log) stays as is. Add `docs/progress.md` (one entry per session: date, what was done, measurements, what is broken), `docs/specs/` (tasks), `docs/architecture.md` (the built architecture, kept current), `CLAUDE.md` at the repo root (standing instructions for every agent session).
- b) One big `STATUS.md`.
- Recommendation: **a**.

**B7-3. Verification before each commit.**
- a) Checklist in `CLAUDE.md`: (1) the spec's acceptance criteria are met and named in the commit; (2) tests pass locally (`cargo test`, app tests, `pytest`, as applicable); (3) the user reads the diff; (4) an agent review pass over the diff for correctness, separate from the writing pass; (5) CI green after push. For model weights: the golden forward-pass test. For rule changes: the scenario tests for that toggle.
- b) Tests and CI only.
- Recommendation: **a**. Step 3 is what makes it your process rather than the agent's.

**B7-4. Acceptance criteria before tests.** a) The user approves the acceptance criteria in the spec before an agent writes the tests, so tests cannot be written to fit the code. b) Agents write tests freely. Recommendation: **a**.

**B7-5. Commit convention.**
- a) `type(scope): subject` (types: feat, fix, test, docs, plan, chore, train, model). Body says what and why. Trailers: `Spec: docs/specs/NNN-name.md` (when a spec applies), `Produced-by: Claude Code <version>, model <tag>` (or `Produced-by: user` for hand-written changes), `Session: <ai-log row>`, `Co-Authored-By` line. One commit per task or logical step. No end-of-day dumps.
- b) Plain messages.
- Recommendation: **a**. It makes the commit log the replication record the template asks for (R14) and gives G19 something to count.

**B7-6. Session logging.**
- a) One `ai-log.md` row per session: date, tool and version, model tag, what the agent did, what the user did, outputs. Prompts summarised in the row; standing instructions live in `CLAUDE.md`; a prompt is stored verbatim in `docs/prompts/` only when it is reused (e.g. the lesson-text prompt). Subagents get a sub-row with their model.
- b) Full transcripts exported per session.
- Recommendation: **a**. Transcripts are optional extras; the row plus commit trailers already give prompt, model and output traceability.

**B7-7. Tools and roles (the agentic-stack table).**
- a) Claude Code with Claude Fable 5.1 for planning, code, tests, research subagents and review passes. Figma connector for the wireframe. GitHub Actions for CI. A local Ollama model only later, with one declared role (e.g. a second-opinion review or Slovenian text checks), if time allows (F6). Every model gets its own row with a pinned tag and the date range of use.
- b) Several models from the start.
- Recommendation: **a**.

**B7-8. Branching.**
- a) Trunk-based: small commits on `main`; a branch only for a long or risky change (e.g. a training-pipeline rewrite), merged with a merge commit so the history stays visible.
- b) A pull request per task.
- Recommendation: **a**. Solo project; the course grades the commit history, not PRs.

**B7-9. Session routine.**
- a) Start: read `docs/planning/README.md`, `docs/progress.md`, the open specs. Work in spec order. End: update `docs/progress.md`, the spec status, `ai-log.md`; commit; push.
- b) Free form.
- Recommendation: **a**. Written into `CLAUDE.md` so every session does it.

**B7-10. Training runs as commits.**
- a) Each training run has a config file under `src/train/configs/`, a run log in `docs/progress.md` (steps, hours, metrics, checkpoint hash), and released weights in `models/` with a `MODEL.md` (config, data, date, metrics). A `model:` commit per released weight file.
- b) Weights outside git.
- Recommendation: **a**, with weights kept small (one f16 file per release). Large intermediate checkpoints stay out of git.

## Batch 6 — measurement: the numbered goals (asked 2026-11-09, answered 2026-11-09; see D65–D69 and plan.md section 4)

Context: the course wants goals someone can check (R7, R12). They go into `README.md` and the report as G1, G2, …. Every goal below names a number, how it is measured, and where the measurement lives. Thresholds are Claude's proposals; the ones most worth adjusting are marked. Sources: research/05 (evaluation protocol), research/07 (faithfulness checks D1–D8, D10).

**B6-1. Goal list.** Accept, or change thresholds (give the goal and the new number).

Rules and engine
- G1. The engine passes a rule-scenario suite of at least **200** tests covering the baseline and every T0 toggle value, plus property tests (card conservation, 70 points per deal, legal-move sets) on 100,000 random deals. Measured by `cargo test` in CI.
- G2. Native, WASM and PyTorch agree: identical legal-move sets and scores on 10,000 random games; forward-pass outputs within 1e-4 (fp32) / 1e-2 (fp16) on 10,000 random states. Measured by the parity tests in CI.

Training pipeline
- G3. End-to-end self-play throughput of at least **20,000** steps/s on the training PC (D22). Measured by the stage-0 benchmark script; the number is logged in `docs/progress`.

Agent strength (8-seating duplicate protocol, D25)
- G4. The trained agent (transformer) beats the rule bot by a positive mean score per deal with a 95% confidence interval that excludes zero, over at least **2,000** deals. [threshold worth adjusting: a minimum margin, e.g. +5 points per deal]
- G5. Zero illegal moves in 100,000 self-play deals; declare rate, pass rate and announcement rate per contract logged and within ranges set in stage 2 (no all-pass collapse).
- G6. Agent vs the PIMC/ISMCTS benchmark at a fixed iteration budget: result reported with a 95% CI over at least 2,000 deals. No pass/fail threshold; the honest number is the result. [option: add "not worse than the benchmark" as an aspirational target, marked as such]
- G7. Transformer vs MLP control, same pipeline, same step budget: score difference with a 95% CI over at least 2,000 deals, reported.

Explanations (faithfulness)
- G8. Partner-prediction accuracy before the called king falls: at least **60%** on held-out deals (chance is about 33%). Measured on 10,000 held-out self-play deals.
- G9. Belief head: Brier score better than the uniform prior by at least **20%** on held-out deals; expected calibration error reported.
- G10. Q-head calibration: mean absolute error between predicted and realised deal score reported on held-out deals. [threshold optional, e.g. ≤ 15 points]
- G11. Attribution sanity: rank correlation between attributions from trained and weight-randomised models below **0.2**; deletion-curve AUC better than random ordering on at least 1,000 decisions.
- G12. Counterfactual consistency: the model plays the stated card in at least **99%** of shown counterfactuals.
- G13. Template audit: **100%** of rendered explanation clauses are backed by their signal and threshold, checked by a test over sampled decisions.

Phone and app
- G14. Agent move latency at most **300 ms** on the reference phone (to be named; see B6-2); app shell plus weights at most **10 MB**; first load at most 3 s on a 4G connection (Lighthouse or manual timing). Measured on the Pages build.
- G15. Coached play, review and adviser each complete a full deal end to end in the T0 app (manual test script, recorded in `docs/`).
- G16. Adviser: a full deal (hand, bids, king, talon if declarer, 12 tricks) can be entered in at most **4 minutes** by the user. Timed.
- G17. At least **5** scripted lessons and at least **10** rule cards in Slovenian, each tied to a seeded deal or rule.
- G18. Human sessions: at least **50** deals played against the agent by at least **3** people, results and feedback recorded, reported with standard error as anecdote. [option: drop if time runs short; keep as T1]

Process
- G19. **100%** of commits on `main` carry the tool and model trailer (P3); `ai-log.md` has a row for every session; `README.md` lists every tool with a pinned model tag.

**B6-2. Reference phone for G14.** a) Your own phone (name the model). b) A mid-range Android of a friend. c) Browser DevTools CPU throttling ×4 on desktop as a proxy. Recommendation: **a**, with c as the CI-able proxy.

**B6-3. Strength margin in G4.** a) CI excludes zero (any positive margin). b) At least +5 points per deal. c) At least +10. Recommendation: **a** for the goal, b as the aspiration; tarok's per-deal spread is unknown, so a fixed margin is a guess until stage 1 measures it.

**B6-4. Human sessions (G18).** a) Keep in T0 goals. b) Move to T1 (nice to have). Recommendation: **b**; it depends on other people's time.

**B6-5. Goal numbering.** a) Freeze G1–G19 now and renumber only by appending. b) Renumber freely until the proposal. Recommendation: **a** after the proposal is submitted; b until then.

## Batch 5 — architecture and technology (asked 2026-11-09, answered 2026-11-09; see D51–D64)

Context: Rust engine with the forward pass in Rust, PyO3 to PyTorch (D22); transformer network (D18); model runs in the phone browser (research/06); Vue is the user's stated preference in their global notes and they review TypeScript/Vue, Python, Rust, C++ (F5); online play is T3 (D38). Sources: `research/06` for the browser and training stack. The rest are Claude's proposals.

**B5-1. Engine.**
- a) One Rust crate `tarok-engine`: rules, legal moves, scoring, rule toggles, deal seeding, game records, state encoding, and the network forward pass. Built three ways: native library for training actors (PyO3), WASM for the browser (wasm-bindgen, SIMD), native CLI for tests and tournaments.
- b) Rust for training plus a separate TypeScript engine for the app, kept equal by cross-tests.
- Recommendation: **a**. One rules implementation means one set of rule tests and no drift between what the agent learned and what the app enforces. b doubles the rule-switch work (D12).

**B5-2. App framework.**
- a) Vue 3 + Pinia + Vite, TypeScript, custom CSS variables (no Tailwind), VueUse for storage. PWA via vite-plugin-pwa.
- b) React.
- c) Svelte.
- Recommendation: **a**. It is the stack you review best and your notes already set its conventions (Pinia store per concern, Teleport for drawers, v-reveal). Agents produce it reliably.

**B5-3. Where the model runs.**
- a) In the browser only, inside the engine's WASM module. Weights as a versioned f16 file fetched once and cached by the service worker. No server.
- b) A small inference server.
- c) Both, browser by default.
- Recommendation: **a**. Nothing in T0–T2 needs a server; the table adviser must work offline at a real table.

**B5-4. Forward pass for the transformer in the browser.**
- a) Hand-written in Rust inside the engine crate (embedding, attention, LayerNorm, GELU, heads), with a golden test against PyTorch on 10,000 random states on every weight export. ONNX export kept in CI only as a second cross-check.
- b) tract-onnx compiled into the WASM module.
- c) ONNX Runtime Web (10–12 MB runtime).
- Recommendation: **a**, with b as the fallback if the hand-written pass is too slow or too much work. A hand-written pass exposes the hidden state and attention weights directly, which the explanation layer needs (D16). The network is small (4+ layers, 256 wide), so the code is a few hundred lines.

**B5-5. Training code.**
- a) Python package `train/`: PyTorch learner, config files, checkpointing, evaluation runner, export script. Rust actors called through PyO3/maturin return batches as NumPy arrays. Native Windows, PyTorch cu130 wheel (Turing supported). No WSL2.
- b) All in Rust (burn/candle).
- Recommendation: **a** (research/06). PyTorch tooling and debugging outweigh the FFI cost. Pin versions in `requirements.txt` and `Cargo.lock`.

**B5-6. Repository layout.**
- a) `src/engine/` (Rust crate), `src/app/` (Vue), `src/train/` (Python), `src/assets/cards/` (SVG generator and output), `docs/` (planning, architecture notes, screenshots), `report/` (ERK source), `models/` (released weights, small), `.github/workflows/`.
- b) Separate repositories per part.
- Recommendation: **a**. The course names `src/` and `docs/`; one repo keeps one commit history, which is graded.

**B5-7. Hosting and demo.**
- a) GitHub Pages from the same repo, deployed by GitHub Actions on push to `main`. Phone demo = open the URL. `README` gives install, run and demo steps.
- b) Cloudflare Pages or Netlify.
- c) Local only.
- Recommendation: **a**. No extra account; the repo goes public on 23 Nov anyway.

**B5-8. Storage.**
- a) Browser storage (IndexedDB for games and sessions, localStorage for settings via VueUse `useStorage`), plus JSON export/import of games, sessions and presets. No backend.
- b) Backend database.
- Recommendation: **a**.

**B5-9. Game record format.**
- a) One JSON schema for every deal, used by self-play logs, app games, adviser sessions and lessons: seed, preset and toggle values, model version, seat names, full deal, and for every decision the acting seat, the observation as that seat saw it (information-set discipline, research/07), the legal actions, the chosen action, and the agent's outputs if an agent acted. Defined in the engine crate; validated in tests.
- b) Separate formats per use.
- Recommendation: **a**. Review, replays, puzzles and the faithfulness checks all read the same records.

**B5-10. Card assets.**
- a) Index-only SVG cards generated by a script from one template (54 files): large Roman numeral for taroks, suit symbol and rank for suit cards, colour-blind-safe suits. Your own work by authorship of the template; licensed CC BY 4.0 in the repo, separate from the code licence. Style per your craft aesthetic (warm palette, serif numerals).
- b) Hand-drawn cards.
- Recommendation: **a**. Consistent, cheap, readable on a phone. A hand-drawn "classic" skin can come later (T2).

**B5-11. Code licence.** a) MIT for code, CC BY 4.0 for cards and texts. b) Apache-2.0. c) GPL. Recommendation: **a**; no copyleft dependency is planned (D9 bans the copyleft tarok engines anyway).

**B5-12. Engine tests.**
- a) Rule scenario tests per toggle (from `rule-switches.md`), property tests (card conservation, 70 points per deal, legal-move sets), golden games replayed from records, parity tests native ↔ WASM ↔ PyTorch forward pass, Python binding tests.
- b) Fewer: scenarios and parity only.
- Recommendation: **a**. The counts feed the measurement batch.

**B5-13. CI.** a) GitHub Actions: `cargo test`, `cargo clippy`, WASM build, app lint/unit tests, Python tests, golden forward-pass test when weights change, Pages deploy. b) No CI. Recommendation: **a**.

**B5-14. Offline.** a) PWA: service worker caches the app and the weights; weights refetched silently if evicted (iOS 7-day cap); never block the UI on cache presence. b) No offline support. Recommendation: **a**.

## Batch 4 — rules (asked 2026-11-09, answered 2026-11-09; see D43–D49)

Source: `rule-switches.md` (39 toggles, presets, incompatibilities, left-out list), derived from `research/01`. Claude read the whole file. Facts in it come from the research subagent's reading of pagat.com, the Tarok.net rulebook, the TARZS rules and others; a few cells are marked unverified. Context: 4-player (D4), first model on one default preset with play-changing toggles fixed (D20).

**B4-1. Default preset.**
- a) Your own table's rules. You answer the 11 open defaults below; the result is saved as the preset "Default" and is the rule set the first model is trained on.
- b) pagat.com's 4-player description.
- c) The "Majority" preset (per toggle, the option most Slovenian sources support).
- Recommendation: **a**. You are the one who checks the agent's play and the rule tests. A rule set you know by heart is the one you can verify. pagat.com and Majority stay as named presets.

**B4-2. The 11 open defaults** (from `rule-switches.md` section 6). Answer each with a letter.
1. S14 discarding taroks: a) allowed, shown face up; b) only when forced.
2. S25 rounding: a) nearest 5; b) exact.
3. S29 mondfang: a) −20; b) −21.
4. S28 klop "full": a) more than 35 points; b) 35 or more.
5. S26 partner's share: a) same as declarer; b) half.
6. S33 radelc cancellation: a) only when the declarer wins; b) after the next game as declarer, won or lost.
7. S34/S35 session: unused radelci a) −100 each, b) none; session end a) škis round, b) fixed deal count (how many), c) open-ended.
8. S6 odprti berač in the first model: a) yes, on by default; b) in the engine but pinned off until T1.
9. S7 barvni valat: a) in the engine, pinned off until T1 (proposed); b) in the first model.
10. S1 compulsory klop at a cumulative score of exactly 0: a) yes (pagat.com); b) none.
11. S18 kontra depth: a) four levels (kontra, re, sub, mort, ×16); b) two (×4).
- Recommendation: answer from your table. Where your table has no habit: 1b, 2b, 3b, 4b, 5a, 6a, 7 none + škis round, 8a, 9a, 10b, 11a. Reason: these are the options with the most Slovenian sources behind them; 8a and 9a keep the first model's action space small.

**B4-3. The toggle list and tiers.** 39 toggles: T0 12 (S6, S14, S15, S18, S22, S25, S26, S28, S29, S31, S32, S33), T1 13, T2 14 (incl. four 3-player-only).
- a) Accept. b) Changes (name the toggle and the new tier or option).
- Recommendation: **a**.

**B4-4. The left-out list** (15 variation points dropped, 12 partly, each with a reason in section 5).
- a) Accept. b) Bring back a point (name it).
- Recommendation: **a**.

**B4-5. Scoring-only T0 toggles in the first model.**
- a) Feed the T0 scoring-only toggles (S15, S22, S25, S26, S28, S29, S31, S32, S33) to the network as inputs and randomise them in self-play, so one model covers all their values.
- b) Fix them to the default preset for the first model; add them to the input later with the rule vector.
- Recommendation: **a**. They change the payoff only, not the legal moves, so they cost no new actions. D20 allows this. Risk: slightly more to learn; mitigated by evaluating per preset.

**B4-6. Incompatibility handling in the UI.**
- a) Dependent toggles are greyed out and forced (e.g. klop off forces radelci off); presets set every toggle; the share code encodes all toggle values; a warning lists what a preset leaves unspecified.
- b) No enforcement; a warning only.
- Recommendation: **a**. The 13 rules in section 3 are mechanical.

**B4-7. All pass with klop off (S4 = off).** No source says what happens. a) Redeal by the next dealer. b) Keep klop always on in T0 and T1 (S4 moves to T2). Recommendation: **b**; it removes a source-less rule from the first releases.

**C6. Figma wireframe timing.** The connector is available (D41). a) Next session: Claude writes the screen spec, then builds the wireframe in Figma, with the pitch slide text after it. b) After Batch 5 (architecture). Recommendation: **a**; the screens depend on Batches 1–3 only, which are decided.

## Batch 3 — product and screens (asked 2026-11-09, answered 2026-11-09; see D31–D42)

Context: Learn → Play → Table (D2), Learn screens built first (D29), live adviser in the core (D3), adviser asks for the learner's card first (D17), explanations layered (D16), own SVG cards (D11), 4-player (D4). Sources: `research/02` (existing apps, borrowable features), `research/07` (explanation display). No new research was run for this batch; the options are Claude's design proposals.

**B3-1. Primary user.**
- a) A player who knows the rules and wants to play better. Beginners get a rules layer on top.
- b) A beginner who has never played tarok.
- c) Both as equals.
- Recommendation: **a**. The explanation signals (expected score, partner belief, attributions) are about strategy. A beginner needs rules first, which is cheap to add as rule cards. Designing for the beginner first would push the agent's output out of the main screens.

**B3-2. Form factor.**
- a) Phone-first web app (portrait), installable as a PWA, usable on desktop.
- b) Desktop-first, phone later.
- c) Both from the start, separate layouts.
- Recommendation: **a**. The table adviser is used on a phone. One responsive layout keeps the Figma work small.

**B3-3. Screen list for Tier 0** (the Figma wireframe).
- a) Home · Learn hub (lessons, progress) · Lesson deal (table with teaching overlay) · Play table (game screen with explanation panel) · Post-game review (trick by trick, divergence, score loss) · Adviser (hand entry → trick-by-trick entry → learner's card → contrast) · Rules and presets (toggles, incompatibility warnings, share code) · Levels (bot ladder with measured strength) · Settings (language, card style).
- b) Same, but Lesson deal, Play table and Adviser are one table component in three modes (one screen in Figma, three states).
- c) A trimmed list: Home, Table (3 modes), Review, Rules. Levels and Settings folded into Home.
- Recommendation: **b**. One table component with modes is less to build and keeps the explanation panel identical everywhere. Levels and Settings stay separate screens because they carry report results (strength ladder) and the rule toggles.

**B3-4. What "teaching as you go" means.**
- a) Scripted lessons: each lesson is a seeded deal chosen to trigger one concept (e.g. partner inference, pagat safety, when to kontra); the explanation layer narrates; the learner plays it.
- b) Coached free play: the learner plays any deal; the app interrupts when the learner's intended card loses more than a threshold against the agent's choice, and shows the contrast.
- c) Both: lessons first, then coached play with an adjustable interrupt threshold.
- Recommendation: **c**. Lessons give structure for a beginner-to-intermediate path; coached play scales without writing content. Lesson deals can be mined from self-play by the key-moment method (research/07, method 13).

**B3-5. Lesson content and who writes it.**
- a) Claude drafts the lesson list and texts from the rules and the agent's signals; the user, who plays, corrects them.
- b) The user writes them.
- Recommendation: **a**. Template sentences are bound to signals (D16), so most "teaching" text is generated from numbers. Strategy framing needs a player's judgement, which is the user's review step.

**B3-6. Learn features and tiers.** Proposed: post-game review (T0); coached play (T0); scripted lessons (T0, small set; grows in T1); bidding adviser from the Q over bids (T1); "what if" replays (T1); puzzles generated from key moments (T1); progress tracking (T1); card-counting trainer (T2); puzzles from own mistakes with spaced repetition (T2).
- a) Accept. b) Move the bidding adviser to T0. c) Other changes.
- Recommendation: **a**. The bidding adviser is cheap once bidding is learned, but bidding is learned in stage 2, after card play.

**B3-7. Adviser input flow (T0).**
- a) Hand entry on a 54-card grid (tap 12 cards). Then bidding entry (what each seat bid), king call, talon shown if the learner is declarer. Then trick by trick: tap the cards others played in order; the grid hides cards already seen. Before each of the learner's plays: tap the intended card → agent's card and contrast.
- b) As a, but play phase only in T0 (bids and talon entered as a summary: contract, declarer, called king).
- c) Voice or camera entry.
- Recommendation: **b** for T0, **a** in T1. The agent needs the contract and the called king to reason; it does not need the full bid sequence to give card-play advice. Entering every bid is friction at a real table. Camera entry is out (high risk, no gain for the core).

**B3-8. Play features and tiers.** Proposed: play vs bots (T0); levels (T0); daily deal with comparison to the bots (T1); share a deal by code (T1); beat-the-bot duplicate mode (T2); online play with friends (out of the course scope: needs a server and adds nothing to the agent core).
- a) Accept. b) Keep online play as T3 (after the course). c) Other.
- Recommendation: **b**. Listing it as T3 records the idea without planning for it.

**B3-9. Table tools and tiers.** Proposed: scoring calculator (T1; the rules engine already computes it); house-rules share code (T1; the preset is a short string); score sheet for a session (T2); rules arbiter (T2).
- a) Accept. b) Other.
- Recommendation: **a**.

**B3-10. Languages.**
- a) i18n keys from the first screen, English text first, Slovenian added as a translation pass. Tarok terms stay Slovenian in both.
- b) Slovenian first.
- c) English only during the course.
- Recommendation: **a**. The demo and report are in English; the users are Slovenian; keys from the start cost little, retrofitting costs a lot.

**B3-11. Figma wireframe.**
- a) Claude writes a screen-by-screen spec (layout, elements, states); the user draws it in Figma.
- b) Claude also builds a clickable HTML wireframe from the spec; the user screenshots it into Figma or redraws it.
- c) Use a Figma connector if one is available in the user's Claude setup.
- Recommendation: **b**. The HTML wireframe becomes the first UI code later; the spec is needed either way.

**B3-12. Tier scheme.** T0 = must work for the demo; T1 = planned; T2 = stretch; T3 = after the course.
- a) Accept. b) Other.
- Recommendation: **a**.

## Batch 2 — bots (asked 2026-11-09, answered 2026-11-09; see D14–D26)

Sources: `research/05` (training method), `research/06` (network, browser), `research/07` (explanations). Claude read the method, failure-mode, evaluation and recommendation sections of all three; the subagents read the papers. Key numbers that are inferred, not measured: pipeline step rates, training hours, tarok's per-deal score spread. Nothing in the literature trains a rule-conditioned card-game policy, and no user study of explanations exists for any card game.

**B2-1. Training method.**
- a) PPO actor-critic with a full-information critic (critic sees all hands, talon and true partner during training; actor sees only its own view), self-play with 20% past checkpoints. Plus a Deep Monte Carlo (DouZero-style) run on the same pipeline as a control and fallback.
- b) Deep Monte Carlo only.
- c) PPO with a behaviour-cloning warm start from a search bot from day one.
- d) AlphaZero-style with determinisation.
- Recommendation: **a**. PerfectDou's ablation (read by the subagent): vanilla PPO against DouZero reached win rate 0.509; the same PPO with a full-information critic reached 0.738. The hidden partner is exactly what the critic can be told. DMC needs 10^9–10^10 steps in Dou Dizhu. AlphaZero-style lost at student scale without supervised pretraining (Jass, Klaverjas). c is kept as a fallback if the agent loses badly to the benchmark.

**B2-2. Heads on the network (what explanations are built from).**
- a) Actor trunk with: policy head (masked logits), an observation-only Q-per-action head regressed on returns (usable in the browser, gives "expected score per card"), a decomposed value head (trick points / king / pagat / announcements / penalties), auxiliary heads for partner seat and unseen-card location at low loss weight. Full-information critic is a separate network, training only.
- b) Policy and value only. Explanations post hoc (probes, perturbation).
- c) Q-per-action only (DMC style).
- Recommendation: **a**. The PPO critic needs hidden information, so it cannot produce expected scores on the phone; an observation-only Q head fills that gap. Research 07's methods 1, 4, 11 and 12 need these heads. Cost at inference is negligible. Risk: auxiliary losses can weaken play; mitigation is to measure strength with and without and keep weights ≤ 0.1.

**B2-3. Explanation methods and order** (this also answers C1).
- a) In order: (1) intrinsic signals (Q per card, margin, reward components) rendered through templates bound to thresholds; (2) belief and concept readouts (partner estimate, trumps out, pagat safety, final score) via auxiliary heads and post-hoc probes; (3) perturbation attribution by card swaps and single-edit counterfactuals; (4) review replays and key-moment summaries. LLM narration optional and last, local Ollama model, review mode only, with a hallucination check.
- b) Only (1) and (2).
- c) LLM narration as a core feature.
- Recommendation: **a**. Each method has a numbered faithfulness check (research 07, D1–D10), which feeds the course's checkable goals. Evidence from chess: a good action-specific attribution raised puzzle accuracy from 57% to 72%; two older saliency methods lowered it. LLM commentary has documented hallucination even with strong models and does not run on phones.
- Sub-decision: in adviser mode the learner enters their intended card first, then sees the agent's card and the contrast. Reason: studies on human-AI decisions show people copy the AI otherwise. Recommendation: yes.

**B2-4. Network and encoding.**
- a) Flat encoding of about 620 features (own hand, cards played per seat, current trick, talon seen, discards, contract, called king, partner seat if revealed, announcements, counts, phase, rule vector), seat-relative. MLP 620→512→512→256, about 0.8 M parameters (1.5 MB fp16). GRU over the play history as a measured ablation later.
- b) GRU over history from the start.
- c) Transformer over card tokens.
- Recommendation: **a**. Every credible trick-taking net is a shallow wide MLP of 1–7 M parameters; the Skat imitation net at 2.4 M ran at 2.5 ms per move on one CPU thread. A card-aligned flat encoding keeps attributions readable per card. Start simple, measure, add order information only if the ablation says so.

**B2-5. Phases and action space.**
- a) One network for all phases. Phase one-hot in the input. One masked action vector of about 140 logits (bids, king call, talon choice, single-card discards in sequence, announcements and kontra one at a time, 54 cards).
- b) Separate networks per phase.
- Recommendation: **a**. Masking is a valid policy gradient (Huang and Ontañón). Separate heads are an optimisation to try only if bidding underperforms.

**B2-6. Rule switches inside the agent.**
- a) One network with a rule vector, trained on a mixture of presets from the start.
- b) One model per preset.
- c) First model on one default preset with play-changing toggles fixed; rule vector and preset mixture added in a later stage (research 05 stage 3). Scoring-only toggles can be inputs from the start.
- Recommendation: **c**. Rule-conditioned card-game policies have no published precedent, so start without that risk and add it once the basic loop is proven. The rule list and the default preset come in Batch 4.

**B2-7. Curriculum.**
- a) Phase by phase: card play with a fixed contract and a rule-called king → talon exchange → bidding → announcements and kontra → klop and berač last. Each phase has monitors (declare rate, pass rate, announcement rate, partner-prediction accuracy, legal-move violations).
- b) Whole game from the start.
- Recommendation: **a**. Bidding collapse ("never bid") is documented three times in this game family; kontra must exist before bidding is trained or the agent over-bids.

**B2-8. Training pipeline.**
- a) Rust engine with the forward pass in Rust; self-play actors on 12 CPU threads; PyO3 hands batches to a PyTorch learner on the GPU. Gate: at least 20,000 steps/s end to end, measured in week 1.
- b) Rust engine, Python actors with batched GPU inference.
- c) Python environment.
- Recommendation: **a**, decided by measurement. Research 05 and 06 differ on where inference runs during self-play (GPU batches vs CPU in Rust); both agree the engine must be Rust. A 10^9-step run is estimated at 3–9 h with a fast pipeline and 2–6 days in Python. These rates are inferred; no Rust trick-taking engine publishes a benchmark. PyTorch still ships CUDA wheels for this GPU (Turing). torch.compile is not usable on it under Windows; eager mode is fine. 6 GB VRAM is not a constraint for a 0.8 M-parameter net.

**B2-9. Other bots.**
- a) Random bot and a rule-based bot (per-contract heuristics) in stage 0; PIMC/ISMCTS search bot in stage 3 as the benchmark and, if needed, a behaviour-cloning teacher; a cheating full-information version of the agent as an upper bound; past checkpoints as yardsticks.
- b) Search bot early (stage 0–1).
- Recommendation: **a**. The search bot is only needed when there is something to benchmark (D10). Build it early only if bidding collapse needs seeding with non-passive games.

**B2-10. Difficulty levels and styles.**
- a) Levels = random bot, rule bot, and checkpoints of the trained agent at measured strengths; styles (e.g. aggressive bidder, cautious) = fine-tuned copies with small reward changes, later tier.
- b) Levels by sampling temperature only.
- c) Separate training runs per level.
- Recommendation: **a**. Checkpoints give measured strength for free from the evaluation harness. Temperature alone produces random blunders rather than weaker play.

**B2-11. Evaluation protocol.**
- a) Fixed seeded deal files. Each deal played in 8 seatings (A in each seat with B elsewhere, and the reverse). Mean, standard error, paired t-test. 2,000 deals for routine checks, 10,000 for release claims, recomputed once the per-deal spread is measured. Behaviour monitors from day one. Human sessions of 50–200 deals reported as anecdote with standard error.
- b) Simpler: 4-seat rotation, 1,000 deals.
- Recommendation: **a**. Sample-size formula n = (1.96·σ/d)². With an assumed σ of 50 points per deal, ±5 points needs about 400 deals, ±2 points about 2,400. σ for tarok is unknown and must be measured.

**B2-12. Stages and budget** (research 05, G2; weeks counted from the start of building).
- Stage 0 (wk 1–2): engine, rules tests, harness, random and rule bots, pipeline speed gate.
- Stage 1 (wk 2–3): card play only, 10^7 then 10^8 steps. Gate: beats random widely by 10^7; beats the rule bot at card play by 10^8.
- Stage 2 (wk 3–5): talon, bidding, announcements, kontra, then klop and berač.
- Stage 3 (wk 5–6): search benchmark, tournament, rule vector on a preset mixture; BC warm start if the agent loses badly.
- Stage 4 (wk 6–8): one or two 10^9 runs overnight, export to WASM, phone test, human sessions.
- Stage 5 (wk 8–9): evaluation tables, ablations, writing.
- Options: a) accept as the draft schedule; b) compress stages 1–2; c) other.
- Recommendation: **a** as a draft; the sprint plan in Batch 8 will align it with 23 Nov, 13 Dec and 11 Jan. Note: 66 days from 2026-11-06 is about 9.4 weeks including planning, so the stages overlap the proposal and checkpoint dates.

## Batch 1 — foundations (asked 2026-11-06, answered 2026-11-09; kept for the record)

### Facts only the user knows

- **F1. Topic approval.** Is a tarok app on the Moodle topic list (e.g. as "browser game"), already approved, or still to be approved at consultation? Own topics need approval before 18 Nov.
- **F2. Your tarok.** Which form do you play (3 players, 4 players with a called king, both)? How well do you play? Which house rules does your table use?
- **F3. Time.** Hours per week until 11 Jan. Dates you cannot work. (The exam is 4 Jan.)
- **F4. Compute.** Is this PC (Ryzen 5 5600, 32 GB, GTX 1660 SUPER 6 GB) the training machine? Can it run overnight or for several days? Any other compute: the course's dingo server, a cloud budget?
- **F5. Review skills.** Which languages can you read well enough to catch an agent's mistakes: TypeScript/Vue, Python, Rust, C++?
- **F6. Agent tools.** Which tools and plans do you have (Claude plan and its limits, the course's pi + dingo, local Ollama)? Does the course expect local or dingo models to be used?
- **F7. Tarok Counter.** Research found "Tarok Counter" (GitHub user gajbrus; web + Android, closed source, updated 2026-11-04): score sheet with a rule engine, heuristic and Monte Carlo bots, online play, replays. Do you know it? Is it a classmate's or a friend's?

### Decisions

**D1. Centrepiece — what must be excellent, with the rest thinner.**
- a) The explaining bot. One rules engine, one bot that scores every legal move, and an explanation built from those scores. Play, learn and table use are layers on it.
- b) Self-play-trained bots and their measured strength.
- c) The teaching experience (tutorial, review, puzzles).
- d) The real-life table tools.
- Recommendation: **a**. All three uses need it. It gives checkable numbers for the report. Research found only one tarock app with "ask the bot why" (KubiTarock, for Königrufen) and none for Slovenian tarok.

**D2. Order of the three uses if time runs short.**
- a) Play → Learn → Table.
- b) Learn → Play → Table.
- c) Table first.
- Recommendation: **a**. Play is the test bench for the engine and bots. Learn is Play plus explanations. Table needs its own input screens. Note: play-against-bots already exists (TAROK.net, Tarock Palčka), so Play alone is not the pitch. The pitch is the explanation.

**D3. The real-life use.**
- a) Live adviser in the core, as in the brief.
- b) Adviser in Tier 1, on top of the play core. Input designed to work during a hand (with the table's consent) and after it.
- c) Score sheet and scoring calculator first, adviser later.
- d) Leave all real-life features for after the course.
- Recommendation: **b**. Research found no adviser for real tables, so it is the new part. Score sheets already exist. The adviser is only as good as the bot behind it, so it comes after the bot. Live advice is cheating unless the table agrees; after-the-hand entry avoids that.

**D4. Which form of the game is the baseline.**
- a) 4 players with a called king. No official ruleset exists; reference would be pagat.com plus the Tarok.net rulebook.
- b) 3 players. Official TARZS tournament rules exist (2010, amended to 2022). No hidden partner, so bots and explanations are simpler. Earlier academic work (Silicon Tarokist 2003) is 3-player, which gives a comparison.
- c) Both from the start.
- Recommendation: **the form you play most**, since you have to catch rule errors. If you play both equally: **b** for the core and 4 players as the first expansion. The engine should be designed for both either way.

**D5. What "played differently everywhere" covers.**
- a) Slovenian tarok and its house rules only.
- b) Also the neighbouring games (Königrufen, Hungarian tarokk, Czech taroky).
- Recommendation: **a**. The neighbouring games differ in deck, partner call or scoring unit. They are separate games, not settings. Research found 55 variation points inside Slovenian tarok alone; which become switches is a later batch.

**D6. Repository and planning files.**
- a) Start git now. Commit the planning files. Create `lukaJiglic-acs-project` on GitHub as private now, public by 23 Nov.
- b) Local git now, GitHub later.
- c) No git until building starts.
- Sub-question: planning files stay in `planning/` at the root, or move to `docs/planning/`?
- Recommendation: **a**, with `docs/planning/`. The commit history is graded as the log of how you worked, and planning is part of that. The course names `docs/` as the place for notes.

**D7. Order of the planning itself.**
- a) Product core and screens → architecture and agent workflow → bots → measurement → tiers and timeline. A deeper bot-research pass runs before the bots batch.
- b) Bots first, since they carry the most risk.
- c) Workflow first, since the repo goes live on 23 Nov.
- Recommendation: **a**. The proposal and Figma wireframe are due in 12 days and need the product and screens.

## Backlog — later batches (not asked yet)

None. Batch 8 was the last planned batch. New questions go to "Loose questions" below or become a clarification (C-numbers).

## Loose questions noticed along the way

- ~~The `project/` folder: public repo or only the report source?~~ Answered: only the report source (D28), `project/` gitignored.
- ~~Who draws the Figma wireframe?~~ Answered: Claude through the Figma connector, from an approved screen spec (D50).
- ~~OpenSpiel's tarok engine worth a closer look before Batch 4.~~ Closed: no code from other engines (D9); rules were derived from research/01 instead.
- Tarok's per-deal score variance is unknown. It sets how many deals a strength comparison needs. Measured early in stage 0 (D25). Still open; needs an engine.
- Proposal format for 18 Nov: the course text gives no length or template for the proposal itself. Ask the user, or check Moodle. **Still open; blocks the proposal text.**
- The Figma file sits in the user's drafts (starter plan, three pages per file at most, so the wireframe is on one page in rows A–E). The lecturer needs a share link; sharing is the user's action.
- In the wireframe the suit glyphs come from a fallback font and ♦ shows in red. Cosmetic. The real cards are the script-generated SVGs (D60).
- Card index letters in the wireframe (K, D, C, P for kralj, dama, kaval, pob) are drafts. Decide with the card SVG spec.
- Weight file size for the 4-layer, 256-wide transformer is not estimated; G14's 15 MB budget assumes it fits. Measure at the first export.
- Git warns LF→CRLF on every commit (`.gitattributes` has `* text=auto`). Harmless; decide at step 0 whether to force LF.
