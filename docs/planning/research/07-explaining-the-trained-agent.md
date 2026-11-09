# 07 — Explaining the trained agent

Research note for the tarok app. Planning only. Access date for all web sources: 2026-11-09.

Scope. The agent is a self-play-trained network for 4-player Slovenian tarok (called king, bidding, talon, announcements, 12 tricks). Explanations must come from ML explanation methods applied to that network. No separate rule-based or search bot. The agent runs in a phone browser. Explanations run there or are precomputed.

Evidence labels used throughout:

- **[V]** verified. I fetched the paper page or full text on 2026-11-09 and the claim is in it.
- **[S]** seen only in search snippets or secondary summaries. Not checked against the primary text.
- **[M]** from memory. Not checked today. Treat as a lead, not a fact.

---

## (a) Sources

### Explainable RL surveys and framing

1. Milani, Topin, Veloso, Fang. *Explainable Reinforcement Learning: A Survey and Comparative Review.* ACM Computing Surveys 2023 (arXiv 2202.08434). https://arxiv.org/abs/2202.08434 — taxonomy: feature importance / learning process and MDP / policy-level. [S]
2. Miller. *Explanation in Artificial Intelligence: Insights from the Social Sciences.* Artificial Intelligence 2019 (arXiv 1706.07269). https://arxiv.org/abs/1706.07269 — explanations are contrastive ("why A rather than B"), selected, social. [S]

### Intrinsic signals (Q per action, value, advantage)

3. Zha et al. *DouZero: Mastering DouDizhu with Self-Play Deep Reinforcement Learning.* ICML 2021 (arXiv 2106.06135). https://arxiv.org/abs/2106.06135 — Deep Monte Carlo; the network scores each legal (state, action) pair; actions are encoded as input. [S for existence and method name; M for the per-action-input detail]
4. Juozapaitis, Koul, Fern, Erwig, Doshi-Velez. *Explainable Reinforcement Learning via Reward Decomposition.* IJCAI/ECAI 2019 XAI workshop. — Q-values split into per-reward-type components shown as bars. [S]

### Feature attribution

5. Sundararajan, Taly, Yan. *Axiomatic Attribution for Deep Networks.* ICML 2017 (arXiv 1703.01365). https://arxiv.org/abs/1703.01365 — Integrated Gradients; axioms of Sensitivity and Implementation Invariance; needs only gradient calls. [V abstract]
6. Lundberg, Lee. *A Unified Approach to Interpreting Model Predictions.* NeurIPS 2017. — SHAP, DeepLIFT connection. [M]
7. Adebayo, Gilmer, Muelly, Goodfellow, Hardt, Kim. *Sanity Checks for Saliency Maps.* NeurIPS 2018 (arXiv 1810.03292). https://arxiv.org/abs/1810.03292 — model-randomisation and label-randomisation tests; Guided BackProp and Guided GradCAM fail; plain gradients and GradCAM pass; an edge detector produces similar-looking maps. [S, consistent with memory]
8. Atrey, Clary, Jensen. *Exploratory Not Explanatory: Counterfactual Analysis of Saliency Maps for Deep RL.* ICLR 2020 (arXiv 1912.05743). https://arxiv.org/abs/1912.05743 — saliency explanations for RL agents are "often unfalsifiable and can be highly subjective"; saliency is "best viewed as an exploratory tool rather than an explanatory tool". [V abstract]
9. Puri et al. *Explain Your Move: Understanding Agent Actions Using Specific and Relevant Feature Attribution (SARFA).* ICLR 2020 (arXiv 1912.12191). https://arxiv.org/abs/1912.12191 — perturbation saliency for the chosen action; chess, Go, Atari; chess saliency dataset of 100 Lichess puzzles labelled by three 2200+ players; 40-player puzzle study. [V full text via ar5iv]
10. Huber, Limmer, André. *Benchmarking Perturbation-Based Saliency Maps for Explaining Atari Agents.* Frontiers in AI 2022 (arXiv 2101.07312). https://arxiv.org/abs/2101.07312 — computational fidelity tests; a noise-based saliency method showed little dependence on output-layer parameters; results differ for state-values vs action-values. [S]
11. Spinnato. *Towards Piece-by-Piece Explanations for Chess Positions with SHAP.* 2025 (arXiv 2510.25775). https://arxiv.org/abs/2510.25775 — SHAP over pieces for engine evaluation; notes an earlier Integrated-Gradients study on an AlphaZero-style transformer. [S]
12. Jain, Wallace. *Attention is not Explanation.* NAACL 2019. — attention weights often do not match gradient attribution and can be changed without changing the output. [M]

### Faithfulness tests

13. Petsiuk, Das, Saenko. *RISE: Randomized Input Sampling for Explanation of Black-box Models.* BMVC 2018 (arXiv 1806.07421). https://arxiv.org/abs/1806.07421 — deletion and insertion curves (area under the output-probability curve). [S]
14. Hooker, Erhan, Kindermans, Kim. *A Benchmark for Interpretability Methods in Deep Neural Networks (ROAR).* NeurIPS 2019 (arXiv 1806.10758). https://arxiv.org/abs/1806.10758 — remove top-attributed inputs, retrain, measure accuracy drop; controls for off-distribution artefacts. [S]
15. Hewitt, Liang. *Designing and Interpreting Probes with Control Tasks.* EMNLP 2019 (arXiv 1909.03368). https://arxiv.org/abs/1909.03368 — control tasks and selectivity for probes. [S]
16. Belinkov. *Probing Classifiers: Promises, Shortcomings, and Advances.* Computational Linguistics 2022 (arXiv 2102.12452). https://arxiv.org/abs/2102.12452 — probe accuracy can say more about the probe than the model. [S]
17. Jacovi, Goldberg. *Towards Faithfully Interpretable NLP Systems: How Should We Define and Evaluate Faithfulness?* ACL 2020. — faithfulness vs plausibility distinction. [M]

### Concept probes and concept teaching

18. Alain, Bengio. *Understanding Intermediate Layers Using Linear Classifier Probes.* ICLR 2017 workshop (arXiv 1610.01644). https://arxiv.org/abs/1610.01644 — linear probes trained independently of the model. [S]
19. McGrath et al. *Acquisition of Chess Knowledge in AlphaZero.* PNAS 2022 (arXiv 2111.09259). https://arxiv.org/abs/2111.09259 — sparse (L1) linear/logistic probes per layer and checkpoint; concepts from Stockfish 8's public evaluation sub-functions plus 116 custom concepts; many concepts become decodable around 32k training steps; "measure only a proximate correlation", not causation. [V full text]
20. Schut et al. *Bridging the Human-AI Knowledge Gap: Concept Discovery and Transfer in AlphaZero.* PNAS 2025 (arXiv 2310.16410). https://arxiv.org/abs/2310.16410 — concept vectors by convex optimisation with L1 sparsity; teachability filter (student network, removes 97.6% of concepts); novelty filter (removes a further 27.1%); four grandmasters (2600–2800) improved on concept puzzles between phase 1 and phase 3 by +42, +25, +16, +6 points. [V full text]
21. Karvonen. *Emergent World Models and Latent Variable Estimation in Chess-Playing Language Models.* COLM 2024 (arXiv 2403.15498). https://arxiv.org/abs/2403.15498 — linear probes recover board state at 99.6% at the best layer; probe-directed interventions change behaviour. [S]

### Counterfactuals

22. Olson, Khanna, Neal, Li, Wong. *Counterfactual State Explanations for RL Agents via Generative Deep Learning.* Artificial Intelligence 2021 (arXiv 2101.12446). https://arxiv.org/abs/2101.12446 — minimal image change that flips the action; user study: counterfactuals helped non-experts identify a flawed agent better than nearest-neighbour and no explanation. [S]
23. Huber, Demmler, Mertes, Olson, André. *GANterfactual-RL.* AAMAS 2023 (arXiv 2302.12689). https://arxiv.org/abs/2302.12689 — Pacman user study; counterfactuals helped users identify agent strategies. [S]

### Strategy summaries and combined explanations

24. Huber, Weitz, André, Amir. *Local and Global Explanations of Agent Behavior: Integrating Strategy Summaries with Saliency Maps.* Artificial Intelligence 2021 (arXiv 2005.08874). https://arxiv.org/abs/2005.08874 — 134 MTurk participants, Ms. Pacman, 2x2 design; HIGHLIGHTS-DIV summaries significantly helped (retrospection p=0.008, agent comparison p=0.014); "the addition of saliency maps did not significantly improve performance in most cases". [V full text via ar5iv]
25. Anderson et al. *Explaining Reinforcement Learning to Mere Mortals: An Empirical Study.* IJCAI 2019 (arXiv 1903.09708). https://arxiv.org/abs/1903.09708 — 124 participants, four treatments (control, saliency, reward bars, both); only the combination gave a statistically significant mental-model gain. [V abstract]
26. Septon, Huber, André, Amir. *Integrating Policy Summaries with Reward Decomposition for Explaining RL Agents.* 2022 (arXiv 2210.11825). https://arxiv.org/abs/2210.11825 [S, title only]

### Human-AI decision studies (general)

27. Bansal et al. *Does the Whole Exceed its Parts? The Effect of AI Explanations on Complementary Team Performance.* CHI 2021. https://www.microsoft.com/en-us/research/?p=728173 — explanations did not raise team accuracy; they raised acceptance of the AI's recommendation, right or wrong. [S]
28. Chiaburu et al. 2024 (arXiv 2409.17157). https://arxiv.org/abs/2409.17157 — 1,200 participants; users copied model predictions more often when shown explanations, including wrong ones; no lasting learning effect. [S]

### Chess coaching tools and LLM narration

29. McIlroy-Young, Sen, Kleinberg, Anderson. *Aligning Superhuman AI with Human Behavior: Chess as a Model System (Maia).* KDD 2020 (arXiv 2006.01855). https://arxiv.org/abs/2006.01855 ; https://maiachess.com ; https://github.com/CSSLab/maia-chess — human-move prediction per rating band; blunder prediction; Maia-2 at NeurIPS 2024. [S]
30. DecodeChess product pages. https://decodechess.com/about/ — natural-language explanation of Stockfish moves, target users under 2000 Elo. Self-reported, no independent study found. [S]
31. Kim et al. *Concept-guided chess commentary generation (GCC-Eval).* NAACL 2025 (arXiv 2410.20811). https://arxiv.org/abs/2410.20811 — LLM commentary is fluent but prone to hallucination. [S]
32. Hebbar et al. *Hallucinations on the Board: Tool-Augmented Evaluation of LLM Chess Commentary (ACT-Eval).* 2026 (arXiv 2608.04240). https://arxiv.org/abs/2608.04240 — fluent commentary scored near-perfect with a naive LLM judge while being unfaithful; atomic-claim checking against engine tools. [S]
33. Parfenova. *Do Chess Explanations Reflect Model Decisions?* 2026 (arXiv 2609.22245). https://arxiv.org/abs/2609.22245 — fluent LLM explanations give only limited evidence of faithful reasoning. [S]
34. *Communicating Chess Strategies in Natural Language.* 2026 (arXiv 2607.11486). https://arxiv.org/abs/2607.11486 — evaluates verbalised strategies by whether a player can follow the line. [S]

### Card-game agents with belief or explanation components

35. Rebstock, Solinas, Buro, Sturtevant. *Policy Based Inference in Trick-Taking Card Games.* IEEE CoG 2019 (arXiv 1905.10911). https://arxiv.org/abs/1905.10911 — Skat; infer hidden cards from players' past actions using learned policies; feeds determinised search. [S]
36. Hu, Lerer, Brown, Foerster. *Learned Belief Search.* 2021 (arXiv 2106.09086). https://arxiv.org/abs/2106.09086 — Hanabi; approximate auto-regressive belief over hidden cards learned as a supervised task; public-private architecture. [S]
37. Li et al. *Suphx: Mastering Mahjong with Deep RL.* 2020 (arXiv 2003.13590). https://arxiv.org/abs/2003.13590 — oracle guiding with perfect-information features dropped out during training. [S]
38. Li, Lu, Wang, Zheng, Li. *Mxplainer: Explain and Learn Insights by Imitating Mahjong Agents.* 2025 (arXiv 2506.14246; Algorithms 18(12):738). https://arxiv.org/abs/2506.14246 — parameterised search agent expressed as a network, trained to imitate a black-box Mahjong agent; gives global and local explanations; top-3 action agreement >90%. [S]
39. Kantack et al. *Instructive AI for human training, assistance, and explainability.* 2021 (arXiv 2111.01726). https://arxiv.org/abs/2111.01726 — Hanabi; notes self-play can produce conventions "completely incomprehensible to a human". [S]
40. NukkAI Nook bridge challenge, March 2022. https://www.imperial.ac.uk/news/235238/ai-based-imperial-research-beats-world/ — hybrid symbolic/neural; "explainable" claim is marketing, play phase only, no bidding. [S]

### Browser runtime

41. ONNX Runtime Web, WebGPU backend since 1.17 (Feb 2024). https://opensource.microsoft.com/blog/2024/02/29/onnx-runtime-web-unleashes-generative-ai-in-the-browser-using-webgpu/ ; https://www.npmjs.com/package/onnxruntime-web [S]
42. TensorFlow.js `tf.grad`, `tf.grads`, `tf.valueAndGrad`. https://alphaxiv.org/abs/1901.05350 (TF.js paper) — gradients run in the browser. [S]

---

## (b) Methods table

Column notes. "Cost" is per explained decision at inference time. "Browser" means a phone browser with the exported policy network. "Card/board use" cites a published game application.

| # | Method | What it explains | Cost | Browser? | Pitfalls | Card/board use |
|---|---|---|---|---|---|---|
| 1 | **Q per legal action / expected score** | How much each legal card or bid is worth; margin of the chosen move over the next best. | One forward pass per legal action (DouZero style), or one pass if a Q-head over the full action space. ≤ ~54 cards, so cheap. | Yes. | Q-values are the agent's estimate, not truth. Overconfident when out of distribution. Score scale must match tarok points for learners to read it. | DouZero (DouDizhu) [S]. Standard in any value-based agent. |
| 2 | **Policy probabilities** | Which moves the agent considered plausible. | Free from the forward pass. | Yes. | Probabilities reflect training noise and exploration. Low-probability moves are not necessarily bad. | Every policy-gradient agent. Maia uses them to predict human moves [S]. |
| 3 | **Value before/after, advantage** | Change in expected result if the move is played vs the alternative. | One extra forward pass per alternative. | Yes. | Same as 1. Also hides *why*; it only says *how much*. | Lichess-style eval swing is the same idea for chess [M]. |
| 4 | **Reward decomposition** | Which reward component drives the choice (trick points, called-king bonus, pagat ultimo, announcements, penalties). | Needs a decomposed value head trained with per-component returns. Inference free. | Yes. | Components must be fixed at training time. Components can be entangled. | Juozapaitis 2019 [S]; Anderson 2019 user study [V]; Septon 2022 [S]. Not for card games. |
| 5 | **Gradient saliency / Integrated Gradients** | Which input cards, history entries, or announcements the output is sensitive to. | IG: 20–50 gradient passes per decision. Plain gradient: 1 backward pass. | Yes with TF.js (`tf.grad`). ONNX Runtime Web does inference only; gradients need the graph exported with backward ops or a hand-written Jacobian. Precompute for the review mode. | Adebayo 2018: some methods are insensitive to model weights [S]. Atrey 2020: RL saliency is often unfalsifiable [V]. Baseline choice matters for IG. Attributions on one-hot card inputs are only meaningful per card, not per bit. | IG on an AlphaZero-style chess transformer [S via Spinnato]. SHAP over pieces for engine eval [S]. |
| 6 | **Perturbation saliency (SARFA, occlusion, RISE-style)** | Which cards, when removed or swapped, change the chosen action's relative Q. | One forward pass per perturbed card. For tarok: 12 hand cards + ≤ 48 history slots = ≤ 60 passes. | Yes. Small MLP/transformer at 60 passes is well under 1 s on a phone. | Perturbed inputs are off-distribution (a hand with 11 cards does not occur). Use card *swaps* with a plausible card instead of deletion. Huber 2022: results differ for state vs action values [S]. | SARFA on chess and Go [V]. 40-player chess puzzle study: SARFA 72% vs no-map 57% accuracy [V]. |
| 7 | **Attention maps** | Which history tokens the encoder attends to. | Free if the encoder is a transformer. | Yes. | Jain & Wallace 2019: attention is not reliably explanation [M]. Use as a view, not as the claim. | None found for card games. |
| 8 | **Counterfactual input** | Minimal change to the known state that flips the move ("if the partner had already shown, lead the king"). | Search over single-card swaps and announcement flags: tens to hundreds of forward passes. Greedy search is enough. | Yes for single-change search. Precompute for multi-change. | Counterfactuals can be unrealistic. Restrict to legal, rule-consistent states. Many flips exist; pick the one with the smallest edit and the largest Q change. | Olson 2021 Atari, user study positive [S]. GANterfactual-RL Pacman [S]. None for card games. |
| 9 | **Contrastive ("why A not B")** | Difference between the chosen move and the learner's move, in Q, in reward components, and in attributions. | Two forward passes plus whichever attribution is used. | Yes. | Must pick a sensible B. In the adviser mode B is the learner's move, which solves this. | Miller 2019 argues this is the form people expect [S]. SARFA is action-specific by design [V]. |
| 10 | **Concept probes (linear readouts)** | Whether the hidden state encodes human concepts: who is the partner, how many trumps remain with each opponent, is the pagat safe, who holds the called king, expected final score. | A linear layer per concept over the hidden state. Negligible. | Yes. Export the probe weights with the model. | Probes show correlation, not use (McGrath 2022 [V]). Probe accuracy can measure the probe, not the model (Hewitt & Liang 2019, Belinkov 2022 [S]). Needs control tasks. | McGrath 2022 AlphaZero chess [V]. Karvonen 2024 chess LMs [S]. None for card games found. |
| 11 | **Auxiliary concept heads (trained jointly)** | Same concepts as 10, but trained into the network with simulator labels. Becomes a reliable readout. | Negligible at inference. | Yes. | Auxiliary losses change the policy. Must check that playing strength does not drop. Heads can be accurate while the policy ignores them. | Suphx oracle guiding uses hidden info during training [S]. LBS learns a belief head as supervised task [S]. |
| 12 | **Belief visualisation** | Predicted location of unseen cards: per card, probability it is in the talon, with each opponent, with the partner. | One forward pass of the belief head. | Yes. | Belief head may be miscalibrated. Calibrate on held-out deals and show calibration in review. | Rebstock 2019 Skat inference [S]. Hu 2021 Hanabi learned belief [S]. Both are not explanation tools but the same artefact. |
| 13 | **Strategy summaries (HIGHLIGHTS)** | Global behaviour: the situations where the agent's decision matters most, shown as short replays. | Offline. Rank states by Q spread. | Precompute, serve as data. | Picks states by the agent's own importance measure; may miss what a human finds important. | Huber 2021 Pacman: summaries helped significantly [V]. Not for card games. |
| 14 | **"What the agent would have done" replay** | Per-trick divergence between learner and agent, with Q loss. | One forward pass per decision in the game. | Yes. | Hindsight bias when hidden cards are revealed post-game. Compute with the information the player had, not the full deal. | Lichess "learn from your mistakes" and Maia blunder prediction are the chess analogues [M]. |
| 15 | **Imitation by an interpretable surrogate** | A parameterised search or rule model fitted to the network's choices. | Offline fitting. Cheap at inference. | Yes if the surrogate is small. | Surrogate fidelity is bounded (Mxplainer reports >90% top-3 agreement, not top-1 [S]). Explains the surrogate, not the network. The student's constraint ("no separate rule bot") makes this borderline; a surrogate *fitted to the agent* is still an ML explanation method, but it must be reported as such. | Mxplainer Mahjong [S]. |
| 16 | **Template NL rendering** | Turns signals 1–12 into sentences. Deterministic. | Negligible. | Yes. | Templates are rigid. Each template must be bound to a signal with a threshold so it cannot say something the signals do not support. | DecodeChess is template-like over Stockfish features [S]. |
| 17 | **LLM narration over structured signals** | Fluent prose from the same signals. | Local Ollama model: seconds on a laptop, not on a phone. Server call otherwise. | No for phones. Precompute for review mode. | Hallucination is documented for chess commentary even with strong models (Kim 2024, Hebbar 2026, Parfenova 2026 [S]). LLM-as-judge is circular [S]. The course requires every model to be declared. | Chess commentary papers above [S]. |

---

## (c) Evidence on what helps human learners

Verified findings (full text or abstract read):

- **Saliency alone rarely helps.** Anderson 2019: 124 participants, RTS game. Neither saliency nor reward bars alone gave a significant mental-model gain. The combination did. [V abstract]
- **Choosing what to show matters more than heatmaps.** Huber 2021: 134 participants, Ms. Pacman. HIGHLIGHTS summaries of important states significantly improved agent comparison and retrospection. Adding LRP saliency maps "did not significantly improve performance in most cases". [V]
- **Action-specific saliency can help on chess puzzles.** Puri 2020: 40 players rated 1600–2000, 15 puzzles. SARFA maps raised accuracy to 72.4% from 56.7% with no map. Two older saliency methods *lowered* accuracy to 40.8% and 24.6%. [V] Lesson: a bad attribution method is worse than none.
- **Concept teaching can move even grandmasters.** Schut 2025: four grandmasters, concept puzzles from AlphaZero's latent space. All four improved between baseline and final test (+6 to +42 points). Sample is four. The authors call it a proof of concept. [V]
- **Saliency maps for RL agents are often unfalsifiable.** Atrey 2020. [V abstract]

Findings seen only in snippets or secondary summaries:

- Counterfactual states helped non-experts spot a flawed Atari agent better than nearest-neighbour frames or no explanation (Olson 2021). [S]
- Counterfactuals helped users identify Pacman agent strategies (GANterfactual-RL 2023). [S]
- In general human-AI decision tasks, explanations tend to raise acceptance of the AI's answer, including wrong answers, and do not raise team accuracy (Bansal 2021; Chiaburu 2024). Chiaburu also reports no lasting learning effect. [S]
- Maia predicts human moves per rating band and predicts blunders. It is used as a coaching/analysis bot on Lichess. No controlled learning-outcome study found in this search. [S]
- DecodeChess claims a tutoring benefit. All claims are the vendor's. No independent study found. [S]

Inferred for tarok (my reading, not a study result):

- A learner at a real table wants a contrastive answer: "why that card and not mine". The Miller argument and the SARFA result both point the same way. The adviser mode should compute explanations relative to the learner's intended move.
- Belief visualisation is the most tarok-specific aid and has no user study anywhere. Hidden-card reasoning (where is the king, who is the partner, how many trumps are out) is the core skill in tarok. Showing the agent's belief should help, but this is a hypothesis to test.
- The over-reliance effect (Bansal, Chiaburu) is a direct risk for the live adviser. A learner who reads the agent's choice before deciding will copy it. The adviser should ask for the learner's move first and explain the difference.

---

## (d) Faithfulness checks

Faithfulness means the explanation reflects what the model computes, not what looks plausible (Jacovi & Goldberg 2020 [M]). Each check below gives a number the course can list as a goal.

**D1. Sanity checks for attribution (Adebayo 2018 [S]).**
Randomise the network weights. Recompute the attribution for the same states. Measure rank correlation between original and randomised attribution maps. Goal: correlation near zero. If attributions do not change, the method is not reading the model.

**D2. Deletion / insertion curves (Petsiuk 2018 [S]; RL variants in Huber 2022 [S]).**
For each explained decision, swap out the top-k attributed cards one at a time for a neutral card and record the chosen action's Q or probability. Area under the deletion curve should be small for a faithful method and smaller than a random ordering. Report the mean AUC gap vs random over ≥1,000 held-out decisions. In tarok use swaps or masks, not deletions, so the input stays legal.

**D3. Counterfactual consistency.**
For each counterfactual shown ("if X, the agent would play Y"), run the model on the counterfactual state. Goal: the model actually plays Y in ≥ 99% of shown counterfactuals. This is cheap and should be 100% by construction. Failures indicate a rendering bug.

**D4. Probe accuracy with control tasks (Hewitt & Liang 2019 [S]; Belinkov 2022 [S]).**
For each concept probe, report accuracy on held-out deals and accuracy on a control task where labels are random but fixed per input. Selectivity = concept accuracy − control accuracy. Goal: selectivity above a set threshold (e.g. 20 points) and concept accuracy above a majority-class baseline by a set margin. Also report accuracy against the simulator's true label (e.g. the real partner identity), which is available for free.

**D5. Probe use, not just presence (McGrath 2022 caveat [V]; Karvonen 2024 interventions [S]).**
Probes only show correlation. To show the concept is used, perturb the hidden state along the probe direction and measure whether the action changes in the predicted direction. Report the fraction of decisions where the intervention flips the action consistently with the concept. This is optional for the course; mark it as a stretch goal.

**D6. Q-value calibration.**
Compare the Q-head's predicted score to the realised score over held-out self-play deals. Report mean absolute error and a calibration plot. Explanations built on Q are only as faithful as Q is accurate.

**D7. Belief-head calibration.**
For each unseen card, the head predicts P(location). Compute Brier score and expected calibration error against the true deal. Goal: ECE below a set value; Brier better than the uniform-prior baseline by a set margin.

**D8. Template-to-signal binding audit.**
Every sentence template must reference a signal and a threshold. Write a test that samples decisions, renders text, and checks that each clause's condition holds. Goal: 100% of rendered clauses are backed by their signal.

**D9. LLM narration claim check (if used).**
Split generated text into atomic claims. Check each against the structured signals. Report the hallucination rate. ACT-Eval (Hebbar 2026 [S]) does this for chess with engine tools. Goal: hallucination rate under a set value, else do not ship the LLM layer.

**D10. Human check.**
A small study: learners predict the agent's next card with and without explanations. Measure prediction accuracy and time. This is the mental-model measure used by Anderson 2019 and Huber 2021 [V]. Keep it small and label it as pilot evidence.

---

## (e) Architecture implications

What the network and export need so that methods 1–14 work.

1. **Card-aligned input encoding.** Every card is one slot in the input (54 slots for the tarok deck, or 54 × channels for hand / played-by-seat / in-talon / unknown). Attributions then map 1:1 to cards. Avoid hashed or learned embeddings as the only input path; if a transformer is used, make each token a card or a single game event so attention and IG remain readable.
2. **Explicit announcement and bidding flags** as separate inputs. Attributions to "trula announced" or "king called: hearts" must be possible.
3. **Q-per-action output.** Either a Q-head over the 54-card action space with legal-move masking, or DouZero-style (state, action) scoring. The first gives all Q-values in one pass and is simpler in the browser. Bidding and talon exchange need their own action spaces and heads.
4. **Decomposed value head.** Predict per-component returns: trick points, called-king result, pagat ultimo, each announcement, and penalties. Sum equals the total value. This gives method 4 at no inference cost.
5. **Auxiliary concept heads trained on simulator labels.** The simulator knows everything. Cheap labels: partner seat (categorical over 3), location of each unseen card (54 × 4 categorical), number of trumps held by each other player, whether the pagat can still be caught, final score. Weight these losses low enough that playing strength does not drop; measure strength with and without.
6. **Separate linear probes trained post hoc** on the frozen hidden state for the same concepts. Compare with the auxiliary heads. Where a post-hoc probe already reads a concept well, the agent learned it on its own. That is a stronger statement than a trained head.
7. **Expose intermediate tensors in the exported model.** Export the hidden state (the input to the policy head) as a model output. Export probe weights as a small JSON. Then concepts and beliefs are a matrix multiply in the browser.
8. **Two export paths.** ONNX for fast inference (ONNX Runtime Web, WebGPU or WASM [S]). TF.js for gradient-based attribution if IG is wanted live; TF.js supports `tf.grad` in the browser [S]. Perturbation attribution needs only inference and works on either path. Keep the model small (a few million parameters) so 60 perturbation passes stay under a second on a mid-range phone.
9. **Determinism and seeding.** Explanations must be reproducible in review mode. Fix the forward pass to be deterministic; store the model version with each saved game.
10. **Information-set discipline.** The review and adviser modes must run the model on exactly the information the player had at the time. Store per-decision observations in the game record, not just the full deal.
11. **Logging for precomputation.** For the review mode, run attribution, counterfactual search and concept readouts once on a server or at the end of the game, and store the results with the game. The live adviser shows only the cheap signals plus one perturbation pass.

---

## (f) Recommendation (mine)

### Layered display

Default view (always shown, no tap):
- The chosen card, its expected score, and the margin over the next best card. Two numbers and a one-line template sentence.
- A small belief strip: where the agent thinks the called king, the pagat and the remaining trumps are.

One tap:
- Q for every legal card as a bar list.
- Reward components for the chosen card (trick points / king / pagat / announcements).
- The three cards in hand or history with the largest perturbation effect on this decision, with a template sentence each.

Second tap or review mode:
- Contrastive view against the learner's move: Q difference, component difference, attribution difference.
- One counterfactual: the smallest state change that flips the decision.
- Concept readouts: partner estimate, trumps-out estimate, pagat safety, estimated final score.
- Full-game replay with per-trick divergence and Q loss.

Adviser mode at a real table: the learner enters their intended card first. The app then shows the agent's card and the contrastive view. This counters the copy-the-AI effect reported by Bansal 2021 and Chiaburu 2024 [S].

### Methods to implement, in order

1. **Intrinsic signals + reward decomposition + templates.** Q per legal action, decomposed value head, template sentences bound to thresholds. Measured by D6 (Q calibration), D8 (template audit), and playing-strength delta from the decomposed head.
2. **Belief head + concept heads/probes.** Auxiliary heads for card location, partner seat, trumps-out, pagat safety, final score. Post-hoc linear probes for the same. Measured by D4 (probe selectivity), D7 (belief calibration), and D5 as a stretch.
3. **Perturbation attribution (SARFA-style, card swaps) and single-edit counterfactuals.** Measured by D1 (randomisation sanity check), D2 (deletion AUC vs random), D3 (counterfactual consistency). Skip gradient saliency unless perturbation is too slow on phones.
4. **Review replays and strategy summaries.** Per-trick divergence and HIGHLIGHTS-style key moments from self-play. Measured by D10 (small learner study).

LLM narration is optional and last. If used: a declared local model via Ollama, server-side, review mode only, with D9 enforced. Do not ship it live on phones.

### Top risks

1. **Plausible but unfaithful explanations.** Saliency methods look convincing even when they do not read the model (Adebayo, Atrey). Mitigation: D1–D3 before any attribution is shown.
2. **Over-reliance.** Learners copy the agent instead of learning. Mitigation: ask for the learner's move first; make the contrastive view the default in adviser mode; track whether the learner's own Q-loss falls over sessions.
3. **Self-play conventions.** A self-play agent may develop signalling that no human uses (Kantack 2021 [S]). Explanations of such moves will be true but unteachable. Mitigation: concept heads reveal it; optionally fine-tune on human games later.
4. **Auxiliary heads weaken play.** Mitigation: measure win rate with and without heads; lower their loss weight; or train probes post hoc only.
5. **Phone cost.** IG or multi-edit counterfactuals may be too slow live. Mitigation: perturbation over ≤60 cards for live; precompute the rest.
6. **No card-game user study exists.** Everything in (c) is from chess, Atari and decision tasks. Mitigation: label the learning claims as hypotheses; run D10.

---

## (g) Verified vs inferred

**Verified by reading the text today [V]:**
- McGrath 2022 probe method, Stockfish-8 label source, 32k-step finding, correlation-only caveat.
- Schut 2025 method, teachability/novelty filter percentages, four-grandmaster results.
- Puri 2020 (SARFA) chess puzzle study numbers and dataset construction.
- Huber 2021 study design, participant count, summary and saliency results.
- Anderson 2019 participant count and combination-only significance.
- Atrey 2020 main conclusion.
- Sundararajan 2017 axioms.

**Seen only in search snippets or secondary pages [S]:**
- Adebayo 2018 details; Olson 2021 and GANterfactual-RL user study results; Bansal 2021 and Chiaburu 2024 findings; Belinkov and Hewitt & Liang details; Karvonen 2024 numbers; Mxplainer numbers; DouZero method; Rebstock, LBS, Suphx details; NukkAI claims; DecodeChess claims; LLM chess commentary papers; ONNX Runtime Web and TF.js capabilities; Milani survey taxonomy; Miller's contrastive thesis.

**From memory [M]:**
- Lundberg & Lee SHAP; Jain & Wallace attention; Jacovi & Goldberg faithfulness; Lichess "learn from your mistakes" and Maia blunder prediction as chess analogues; DouZero feeding the action as input to score each (state, action) pair.

**Inferred (my judgement, not in any source):**
- Card-swap instead of card-deletion for tarok perturbations.
- Belief visualisation as the most useful tarok-specific aid.
- Ask-first adviser flow as a counter to over-reliance.
- The method ordering and the layered display.
- All concrete numeric goals in (d) are placeholders to be set by the student.

**Gaps found:**
- No published explainability user study for any trick-taking card game.
- No published concept-probe study for a card-game agent.
- No independent evaluation of DecodeChess or of NukkAI's explanation claims.
