# Pitch — one slide, three minutes

Status: **draft 1, 2026-11-10. Waiting for the user's corrections.** Due Wed 18 Nov with the proposal; presented Mon 23 Nov. Source: `plan.md`, `screen-spec.md`, `research/02`.

Language: English, like the report. Say so if the pitch should be in Slovenian.

---

## Slide text

**Title:** Tarok trener

**Subtitle:** A self-play-trained tarok agent that explains its moves, and an app that teaches with it.

**Problem**
- Slovenian tarok has apps to play against bots. None teaches the game.
- No tool helps at a real table.
- No deep-RL tarok agent has been published.

**Product** (phone-first web app, works offline, Slovenian)
- **Učenje.** You play. When the agent would play differently, it stops you and shows why.
- **Igra.** Bots with measured strength, from random to the trained agent.
- **Svetovalec.** Enter a real deal card by card. Pick your card first, then see the agent's.

**How**
- One Rust rules engine. It trains the agent on one PC (PPO self-play, transformer) and runs it in the browser as WASM.
- Explanations come from the model itself: card values, beliefs about hidden cards, attributions, counterfactuals. No LLM text.
- Built with Claude Code agents: a spec per task, verification before each commit, tool and model named in every commit.

**19 numbered goals, for example**
- Beats the rule-based bot: 95% interval above zero over 1,000 deals.
- A move in at most 500 ms on a mid-range phone.
- Every explanation sentence is backed by a model signal: 100% in an audit of 500 decisions.
- Every commit names its tool and model.

**Footer:** Luka Jelenko Iglic · github.com/lukaJiglic/lukaJiglic-acs-project

**Picture on the slide:** the styled table frame from the wireframe, or three greyscale frames (coach banner, explanation sheet, adviser grid).

---

## Speaker notes (about 3 minutes, about 380 words)

Tarok is the card game most Slovenians know. You can play it against bots in several apps. None of them teaches you to play better, and none helps you at a real table. I checked: the closest thing is an Austrian app for a different game.

I am building Tarok trener. The core is an agent that learns four-player tarok by playing against itself on my own PC. Nobody has published such an agent for tarok, so I do not know yet how strong it gets. I will measure it and report what I find.

Around the agent there are three uses. In Učenje you play a normal game. When your card is clearly worse than the agent's, the app stops you, shows the agent's card and says why. In Igra you play against bots whose strength is measured, not just named easy or hard. In Svetovalec you sit at a real table with real cards. You enter what is played. Before each of your turns you pick your card first. Then the app shows what the agent would do. That order matters: studies show people copy an AI if they see its answer first.

The explanations do not come from a language model. They come from the agent's own numbers: how much each card is worth, where it thinks the called king is, which cards changed its mind. Every sentence on screen is a template tied to one of those numbers, and I test that.

Technically there is one rules engine in Rust. The same code trains the agent and runs it in the browser, so the app needs no server and works offline.

The process is the graded part. Planning took 103 recorded decisions, each with who decided. Every task is a spec file I approve before an agent runs it. Before each commit: tests, my own read of the diff, a separate agent review, green CI. Every commit names the tool and the exact model.

I have 19 numbered goals. Four are on the slide. The honest risk is the training: if the agent ends up weaker than hoped, the app ships the best checkpoint and the report says so.

For the demo in January I will play one deal on my phone, show the coach stopping me, and enter a hand from real cards.

---

## Open for the user

- Language of the pitch: English or Slovenian.
- Which picture goes on the slide.
- The "103 decisions" number changes as decisions are added; fix it on the day.
- The claim "no deep-RL tarok agent has been published" rests on `research/02` and `research/05` (searched 2026-11-06 and 2026-11-09). Worded as "none found" if you prefer the safer form.
