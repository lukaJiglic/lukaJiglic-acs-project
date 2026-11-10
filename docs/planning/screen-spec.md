# Screen spec — T0 screens

Status: **draft 2, 2026-11-10. Batch 9 answered (D90–D102); this draft carries the answers.** The Figma wireframe is built from it. Open: C8, C9 in `open-questions.md`.

Purpose: the input for the Figma wireframe (D41, D50) and, later, for the app specs. It covers the T0 screens of D33 and D80.

Sources: decisions D2, D3, D16, D17, D24, D31–D40, D44, D47–D49, D57–D60, D64, D80, D88; `rule-switches.md`; `research/07` section (f) for the layered explanation display; `research/02` section (e) for ideas borrowed from other apps. Everything not tied to a decision number is Claude's proposal. The points asked in Batch 9 now carry their decision number.

Slovenian labels are drafts. The user corrects them (D35, D40).

---

## 1. Conventions

- **Frame.** Portrait phone, 360 × 800 CSS px as the narrow case (D99). If it fits at 360 it fits wider phones. Desktop uses the same layout centred; above 900 px the explanation panel sits beside the table instead of over it.
- **IDs.** Screens `SC-01…`. Table states `T-1…`. Adviser steps `A-1…`. Flows `F1…`. (Rule toggles keep `S1…S39`.)
- **Seats.** The user sits at the bottom. The other three seats are left, top and right. Play runs clockwise (D91): user, left, top, right.
- **Information rule.** During a live deal the app shows only what can be known from the user's seat. Agent outputs for the user's seat (what the agent would play with these cards, its beliefs) may be shown live. Explanations of the other seats' moves depend on hidden hands, so they are shown only after the deal, in review (D94).
- **Model loading (D64).** The UI never waits for the weights. Until they are loaded, random and rule-bot levels work; agent features show a small "nalaganje modela" line.
- **Text.** Every label has an i18n key from the first screen (D40). Tarok terms stay Slovenian in both languages.
- **No LLM text anywhere (D16).** Explanation sentences are templates bound to a model signal and a threshold (G13).
- **Look.** The wireframe is low-fidelity greyscale with one styled sample frame (D99). The built app follows the user's craft palette (D60): carbon header band, cream body, clay accent, serif and mono type.

## 2. Screen map

| ID | Screen (SL draft) | Tier | Reached from | Decisions |
|----|-------------------|------|--------------|-----------|
| SC-01 | Home (Domov) | T0 | launch | D2, D33 |
| SC-02 | Learn hub (Učenje) | T0 | SC-01 | D34–D36 |
| SC-03 | Table (Miza), one component, three modes | T0 | SC-01, SC-02 | D33, D37, D38 |
| SC-04 | Explanation panel (Razlaga), part of SC-03 and SC-06 | T0 | SC-03, SC-06 | D16, D17 |
| SC-05 | Deal result and session sheet (Rezultat) | T0 | SC-03 | D44, D88 |
| SC-06 | Post-game review (Pregled igre) | T0 | SC-05, SC-02 | D36 |
| SC-07 | Rules and presets (Pravila) | T0 | SC-01, SC-03 setup | D43–D49 |
| SC-08 | Levels (Stopnje) | T0 | SC-01, Play setup | D24 |
| SC-09 | Settings (Nastavitve) | T0 | SC-01 | D40, D58 |
| — | Rule card popover (Pravilo) | T0 | SC-03 when a rule triggers | D31, G17 |

Navigation (D90): Home is a hub with three large tiles. No tab bar. The table is full screen with a back button and a small menu. Order of the tiles follows D2: Learn, Play, Table.

## 3. Screens

### SC-01 Home (Domov)

Purpose: choose one of the three uses; resume unfinished play.

Blocks, top to bottom:
1. Header band: app name "Tarok trener" (D102), small offline/online mark.
2. "Nadaljuj" card. Shown only when a deal or session is unfinished. Says mode, deal number, score.
3. Three large tiles, each with a colour swatch and one line of text:
   - **Učenje** — "Igraj z razlago. Agent pokaže, kje bi igral drugače."
   - **Igra** — "Igraj proti botom. Izberi stopnjo."
   - **Svetovalec** — "Za pravo mizo. Vnesi karte, primerjaj svojo potezo z agentovo."
4. Row of small links: Pravila · Stopnje · Nastavitve.
5. Footer: app version, model version.

States: first run (no "Nadaljuj" card; one intro sentence; dismissible install hint for the PWA); weights still downloading (progress line in the footer).

### SC-02 Learn hub (Učenje)

Purpose: entry to the three T0 learning forms (D36).

Blocks:
1. **Vodena igra** (coached play), primary card. One sentence, the current coach strictness, "Začni". Opens SC-03 in Lesson mode, coached.
2. **Lekcije** (lessons). Six in T0 (D100): Štetje tarokov; Kje je kralj?; Varovanje monda; Pagat ultimo; Klicana barva; Klop. Each row: title, topic tag, done mark. Opens SC-03 in Lesson mode, scripted.
3. **Pregledi** (reviews). The last finished deals with date, contract, score, number of marked mistakes. Opens SC-06.
4. **Pravila na karticah** (rule cards). Link to the rule-card list in SC-07.

Not drawn in T0: bidding adviser, what-if replays, puzzles, progress tracking (T1).

### SC-03 Table (Miza)

One component for Lesson, Play and Adviser (D33). Layout on 360 × 800, top to bottom:

| Zone | Height (approx.) | Content |
|------|------------------|---------|
| Status bar | 48 | Back; contract chip (e.g. "Tri · pikov kralj"); mode badge; score button (opens SC-05 as a sheet); menu (rules in force, settings, leave) |
| Seats and trick area | 380 | Three opponent seats (left, top, right): label or level, cards left, role mark (declarer; partner once revealed), tricks taken, last bid or announcement as a bubble. Centre: the current trick, each card placed toward its seat; lead mark; "zadnji štih" peek button |
| Explanation strip | 56 | Mode dependent. See the modes below and SC-04 |
| Hand | 220 | The user's cards in two rows: taroks above, suits below, sorted (D92). Legal cards full strength, illegal cards dimmed. Tap lifts a card; second tap or "Igraj" plays it (confirm step is a setting) |
| Action bar | 64 | Context actions: bids, king call, announcements, "Igraj", "Razveljavi" |

Table states (all modes):

- **T-1 Deal.** Cards appear. Forehand and dealer are marked.
- **T-2 Bidding (Licitiranje).** The contract ladder as buttons in the action area; only legal bids are enabled; "Naprej" to pass. Each seat's bid appears as a bubble at the seat. If all pass: klop notice.
- **T-3 King call (Klic kralja).** Four buttons, one per suit king. Own kings are marked. Skipped in solo contracts and klop.
- **T-4 Talon.** The six talon cards face up in packets of the contract size (2 × 3, 3 × 2 or 6 × 1). The declarer taps a packet. Then a discard tray: tap cards from the hand until the count is right. Kings and trula cannot be chosen. Discarded taroks are shown to all (S14). "Potrdi".
- **T-5 Announcements and kontra (Napovedi).** A checklist of the announcements legal for the user (trula, kralji, kralj ultimo, pagat ultimo, valat) and kontra buttons against the game and against each announcement (four levels, S18). "Brez napovedi" to pass. Other seats' announcements appear as bubbles.
- **T-6 Trick play.** The normal state.
- **T-7 Trick end.** The winning card is marked; the trick moves to the winner. Pause length follows the speed setting. In slow speed a tap continues.
- **T-8 Deal end.** Opens SC-05.

Contract variants:
- **Klop.** No declarer. One talon card is added to each of the first six tricks and shown beside the trick. Must-overtake rule is handled by the legal-card dimming.
- **Berač.** Declarer leads. Must-overtake rule applies. The partner mark is not used.

**Rule card popover.** The first time a rule triggers (the eleven cards of D101) a small card slides in over the explanation strip: title, two or three sentences, "Več" (opens the card in SC-07), "Ne kaži več". Can be switched off in settings. This is the beginner layer of D31.

#### Mode: Play (Igra)

- Setup sheet before the first deal: level of the three bots (SC-08), rules in force (link to SC-07), "Začni".
- Explanation strip: quiet. It shows the trick number and a "Namig" button (D94). A hint shows SC-04 layer 1 for the user's seat and marks the deal as "s pomočjo".
- After each deal: SC-05. From there "Pregled igre" opens SC-06.
- A session is a run of deals with a running score and radelci (D44). It ends only by "Končaj sejo" (D88).

#### Mode: Lesson, coached play (Vodena igra)

Same as Play, with the coach active (D34):

- The learner chooses an action: a bid, a king, a talon packet, a discard set, an announcement or kontra, or a card. The app compares its Q value with the agent's best action.
- **Above the threshold:** the action is held. A coach banner replaces the explanation strip: "Agent bi igral [karta]. Tvoja karta: −N točk." (or "Agent bi licitiral …", "Agent bi založil …"). Actions: "Vzemi nazaj", "Vseeno igraj", "Zakaj?" (opens SC-04 at the contrast layer) (D93).
- **Below the threshold:** no interruption. The difference is stored for review.
- **Same action:** a small tick on the strip.
- Scope in T0: every phase (D93). The coach interrupts in bidding, king call, talon, announcements and kontra, and card play.
- Strictness has three levels (blago / srednje / strogo) in settings. The point values behind them are set after the Q head is calibrated (G10).

#### Mode: Lesson, scripted (Lekcija)

- A seeded deal with a script (D34). The script is a list of steps. Each step names a point in the deal, a prompt, and the feedback texts.
- Progress dots in the status bar.
- A prompt card sits over the explanation strip: one or two sentences (drafted by Claude, corrected by the user, D35).
- Between steps the deal plays itself forward.
- At a step the learner plays. Feedback shows the agent's card, the contrast from SC-04 and the authored sentence. "Naprej".
- End: a summary card with the lesson's point in two sentences, then back to SC-02.

#### Mode: Adviser (Svetovalec)

For a real table with real cards (D3, D37). The user enters everything that happens. The app plays no cards.

- **A-1 Setup.** Who dealt (tap a seat). Rules in force (link to SC-07). Optional names for the seats.
- **A-2 My hand.** The 54-card grid: taroks in three rows (8, 8, 6), then one row of 8 per suit. Seven rows fit in about 310 px. Tap 12 cards. Counter "7 / 12". "Potrdi".
- **A-3 Bidding.** The app walks the seats in order. For each seat it shows the legal bids; the user taps what was said. On the user's own turn the user taps their own bid. Bidding advice in the adviser is T1 (D36) unless C8 changes it.
- **A-4 King call.** Tap the called suit. Skipped when not needed.
- **A-5 Talon.** The talon is face up at the real table, so the user enters all six cards on the grid, then taps the packet taken. If the user is the declarer, they also enter their discards. Discarded taroks of another declarer are entered when shown (S14).
- **A-6 Announcements and kontra.** Per seat, as in A-3.
- **A-7 Tricks.** For each play in turn order:
  - Another seat: the grid shows only unseen cards. The user taps the card that seat played.
  - The user's turn: "Katero karto bi igral?" The user taps the intended card in their own hand first (D17). Then the app shows the agent's card. Same card: a tick. Different card: the contrast banner as in coached play, with "Zakaj?". Then the user confirms the card they really played, which may be either or a third.
  - "Razveljavi" steps back one entry at a time (D96).
  - An entry that contradicts earlier play (a seat that showed void now follows suit) gives a warning and can be kept (D96).
- **A-8 Deal end.** The deal score from the engine (D97). "Pregled igre" (SC-06, with only the cards that were seen). "Nova igra" keeps the seats and moves the deal on.

Entry cost, estimated: 12 hand + about 8 bids + 1 call + 6 talon + about 4 announcements + 36 other plays + 24 own taps = about 90 taps. At 3–4 seconds each this is 4.5–6 minutes. G16 allows 6. This is an estimate, not a measurement.

### SC-04 Explanation panel (Razlaga)

Three layers (D16, `research/07` f). Always about the user's seat during a live deal (information rule).

- **Layer 1, the strip.** The agent's card; its expected score; the margin over the next best card; one template sentence. A belief strip: where the agent thinks the called king is (one bar per other seat), whether the pagat and mond are still out, taroks still out.
- **Layer 2, bottom sheet, one tap.** Q value for every legal card as a bar list. Reward parts for the chosen card (game, difference, kings, trula, kralj ultimo, pagat ultimo). The three cards, in hand or in the history, whose swap changes the decision most, each with a template sentence.
- **Layer 3, second tap, and the default in review.** Contrast with the learner's card: Q difference and which reward parts differ. One counterfactual: the smallest change of the situation that makes the agent play the learner's card. Concept readouts: partner estimate, taroks out per seat, pagat safety, estimated final score.

Numbers are in game points. Every sentence is a template tied to a signal and a threshold (G13). A layer whose faithfulness check has not passed (G11, G12) is not shown.

### SC-05 Deal result and session sheet (Rezultat)

- **Deal result.** Contract, declarer and partner. Card points per side. Difference (rounded per S25). Bonuses, silent and announced. Kontra multipliers. Radelc doubling. Mondfang lines. Final entry per player.
- **Session table.** One row per deal, four columns, running totals. Radelci as dots under each name.
- Actions: "Naslednja igra", "Pregled igre", "Končaj sejo". Ending asks for confirmation, settles leftover radelci at −70 each (D88), shows the final table and saves the session.

### SC-06 Post-game review (Pregled igre)

- Header: contract, result, number of marked mistakes, total points lost against the agent's choices.
- Timeline: bidding, call, talon, announcements, then twelve tricks, as a scrubber. Each of the user's decisions has a grade dot in one of five classes (D95): najboljša / dobra / netočnost / napaka / groba napaka (draft labels), with the points lost shown beside it.
- Main area: the table at the selected moment. "Odkrij karte" shows all four hands (in adviser reviews only the seen cards exist).
- Below: SC-04 at layer 3 for the selected decision.
- "Ključni trenutki": the three decisions with the largest loss. Tap jumps to them.
- "Zakaj?" on any seat's card is allowed here, since the deal is over.
- "Izvozi" saves the game record as JSON (D58, D59).

Not in T0: what-if replay from a chosen moment (T1).

### SC-07 Rules and presets (Pravila)

- **Preset picker.** Default (the user's table, D43), pagat.com, Majority. Choosing a preset sets every toggle (D48). A changed preset shows as "Po meri".
- **Toggle groups** as in `rule-switches.md`: dealing, bidding, talon, announcements, play, scoring, session. T0 toggles can be changed. Later-tier toggles are listed with their fixed value and a tier tag.
- **Dependencies (D48).** A toggle forced by another is greyed, with the reason in one line.
- **Agent note.** Scoring-only T0 toggles are covered by the trained agent (D47). Non-default values of S14 and S18 can be chosen and carry the note "agent ni učen s tem pravilom" (D98).
- **Rule cards.** The list of all eleven cards (D101), each two or three sentences.
- Rules are fixed for a session. A change applies to the next session.

Not in T0: share code for house rules (T1, D39).

### SC-08 Levels (Stopnje)

- A list: Naključni (random), Pravila (rule bot), then the released checkpoints (D24).
- Each trained level shows its measured strength: mean points per deal against the rule bot, the 95% interval, the number of deals, the model version and date (G4, D25).
- The user picks one level for the three bot seats. The coach in Lesson and Adviser always uses the strongest released model (D81).

Not in T0: mixed tables, styles (T2).

### SC-09 Settings (Nastavitve)

- Language: slovenščina / English.
- Speed: hitro / normalno / počasi.
- Confirm before playing a card: on / off.
- Coach strictness: blago / srednje / strogo.
- Rule cards for beginners: on / off.
- Data: export all as JSON, import, delete.
- About: app version, model version and hash, licences (MIT for code, CC BY 4.0 for cards and texts), repository link, offline status.

## 4. Flows

- **F1 First game.** Launch → SC-01 → Igra → setup sheet → T-1 … T-8 → SC-05 → "Naslednja igra" or "Pregled igre".
- **F2 Coached play.** SC-01 → Učenje → Vodena igra → T-6 → coach banner → "Zakaj?" → SC-04 layer 3 → back → play on.
- **F3 Lesson.** SC-02 → lesson → prompt → play → feedback → … → summary → SC-02.
- **F4 Adviser.** SC-01 → Svetovalec → A-1 … A-7 (intended card → agent card → contrast → real card) → A-8.
- **F5 Review.** SC-05 or SC-02 → SC-06 → key moment → SC-04 layer 3.
- **F6 Rules.** SC-01 → SC-07 → preset → toggle → dependency greys another toggle → back.
- **F7 End of session.** SC-05 → "Končaj sejo" → confirm → radelci settled → final table.

## 5. Wireframe plan (Figma)

Frames, one page per group, with arrows for F1–F7:

1. SC-01 Home; SC-01 first run.
2. SC-02 Learn hub.
3. Table states: T-2 bidding, T-3 king call, T-4 talon, T-5 announcements, T-6 trick play (Play mode), klop variant.
4. Lesson: coach banner; scripted lesson step.
5. Adviser: A-1 setup, A-2 hand grid, A-3 bidding entry, A-5 talon entry, A-7 intended card, A-7 contrast.
6. SC-04 layer 2 sheet; SC-04 layer 3.
7. SC-05 result and session sheet; end-session confirmation.
8. SC-06 review.
9. SC-07 rules with a greyed dependency; rule card popover.
10. SC-08 levels; SC-09 settings.

About 26 greyscale frames and one styled sample of the table (D99). Slovenian labels, English frame titles and notes.

## 6. Open points

Batch 9 is answered (D90–D102). Still open, in `open-questions.md`:

- **C8.** D93 makes the coach interrupt in bidding, talon and announcements in T0. Does the Adviser mode (real table) also show the agent's bid and talon choice in T0, or does that stay T1 (D36, D97)?
- **C9.** Mode names. Draft labels in use: Učenje · Igra · Svetovalec.
- All Slovenian labels are drafts for the user to correct, including the five grade names.

Resolved while writing: `rule-switches.md` defined T0 as "every option is in the first trained model", against D20. D98 settles it: T0 means in the engine and the first release; the first model covers the Default preset plus the scoring toggles.
