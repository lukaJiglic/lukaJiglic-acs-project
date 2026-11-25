//! Spec 006, criteria 24 to 26: the JSON game record and its replay. Spec
//! 009, criteria 39 and 40: the round of announcements in the record, and a
//! record of another engine version (D135).

mod common;

use std::path::Path;

use common::flow::{D, finish, seeded_game, seeded_game_b};
use tarok_engine::record::{RECORD_VERSION, Record, RecordError};
use tarok_engine::{Action, Bonus, Game, Phase, Seat, Target, encode};

/// The record of a random deal that depends on the seed only (the dealer
/// is seat `seed mod 4`).
fn record_of_seeded(seed: u64) -> (Game, Record) {
    let game = seeded_game(seed);
    let record = Record::of_game(seed, &game).expect("a finished deal");
    (game, record)
}

#[test]
fn c24_a_finished_deal_exports_a_record_and_the_replay_gives_the_same_entries() {
    let (game, record) = record_of_seeded(31);
    let score = game.score().unwrap();
    let text = record.to_json();
    let json: serde_json::Value = serde_json::from_str(&text).unwrap();

    // The header.
    assert_eq!(json["record_version"], RECORD_VERSION);
    assert_eq!(json["encoding_version"], encode::ENCODING_VERSION);
    assert_eq!(json["engine_version"], env!("CARGO_PKG_VERSION"));
    // The seed is a decimal string: 64 bits do not fit a JavaScript number.
    assert_eq!(json["seed"], "31");
    assert_eq!(json["dealer"], 31 % 4);
    assert_eq!(json["preset"], "default");
    let toggles = json["toggles"].as_object().unwrap();
    let expected_toggles = [
        ("S15", "a"),
        ("S22", "a"),
        ("S25", "a"),
        ("S26", "a"),
        ("S28", "b"),
        ("S29", "a"),
        ("S31", "a"),
        ("S32", "a"),
        ("S33", "a"),
    ];
    assert_eq!(toggles.len(), expected_toggles.len());
    for (name, option) in expected_toggles {
        assert_eq!(toggles[name], option, "{name}");
    }

    // Per decision: the acting seat, the legal slots, the chosen slot.
    // Checked against a replay of the deal step by step.
    let decisions = json["decisions"].as_array().unwrap();
    let actions: Vec<Action> = game.full().actions().collect();
    assert_eq!(decisions.len(), actions.len());
    let mut replayed = Game::from_seed(31, game.public().dealer());
    for (decision, &action) in decisions.iter().zip(&actions) {
        assert_eq!(decision["seat"], replayed.to_act().unwrap().index());
        let legal: Vec<u64> = decision["legal"]
            .as_array()
            .unwrap()
            .iter()
            .map(|slot| slot.as_u64().unwrap())
            .collect();
        let expected: Vec<u64> = replayed
            .legal_actions()
            .iter()
            .map(|legal| legal.slot() as u64)
            .collect();
        assert_eq!(legal, expected);
        assert_eq!(decision["chosen"], action.slot());
        assert!(legal.contains(&(action.slot() as u64)));
        replayed.apply(action).unwrap();
    }
    assert_eq!(replayed.phase(), Phase::Over);

    // The entries and their ten parts.
    let entries: Vec<i64> = json["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e.as_i64().unwrap())
        .collect();
    assert_eq!(entries, score.entries().map(i64::from));
    let parts = json["parts"].as_array().unwrap();
    assert_eq!(parts.len(), 4);
    for (seat, row) in Seat::ALL.into_iter().zip(parts) {
        let row: Vec<i64> = row
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p.as_i64().unwrap())
            .collect();
        assert_eq!(row.len(), 10);
        assert_eq!(row.iter().sum::<i64>(), i64::from(score.entry(seat)));
        let own = score.parts(seat);
        assert_eq!(
            row,
            [
                own.game,
                own.difference,
                own.trula,
                own.kralji,
                own.kralj_ultimo,
                own.pagat_ultimo,
                own.valat,
                own.mondfang,
                0,
                0
            ]
            .map(i64::from)
        );
    }
    // No observation is stored (D120, point A), and no agent output unless
    // someone adds one.
    assert!(json.get("agent").is_none());
    assert!(decisions[0].get("state").is_none() && decisions[0].get("tokens").is_none());

    // Replaying the record gives the same game and the same entries.
    let parsed = Record::from_json(&text).unwrap();
    assert_eq!(parsed, record);
    let again = parsed.replay().unwrap();
    assert_eq!(again, game);
    assert_eq!(again.score().unwrap().entries(), score.entries());

    // The readable layout holds the same record.
    let pretty = record.to_json_pretty();
    assert!(pretty.lines().count() > decisions.len());
    assert_eq!(Record::from_json(&pretty).unwrap(), record);

    // Many deals: export, parse, replay, compare.
    for seed in 0..300u64 {
        let (game, record) = record_of_seeded(seed);
        let replayed = Record::from_json(&record.to_json())
            .unwrap()
            .replay()
            .unwrap();
        assert_eq!(replayed.score(), game.score(), "seed {seed}");
        assert_eq!(replayed, game);
    }

    // An unfinished deal has no record.
    let unfinished = Game::from_seed(31, D);
    assert_eq!(
        Record::of_game(31, &unfinished).err(),
        Some(RecordError::NotFinished)
    );
    // A record made with the wrong seed does not fit the game.
    assert!(matches!(
        Record::of_game(32, &game),
        Err(RecordError::Illegal { .. }) | Err(RecordError::Mismatch { .. })
    ));

    // An optional field with agent outputs survives a round trip.
    let mut with_agent = record.clone();
    with_agent.agent = Some(serde_json::json!({"model": "none", "values": [0.5, -0.5]}));
    let text = with_agent.to_json();
    assert!(text.contains("\"agent\""));
    assert_eq!(Record::from_json(&text).unwrap(), with_agent);
    assert!(with_agent.replay().is_ok());
}

#[test]
fn c25_a_record_with_an_illegal_action_is_refused_with_the_decision_index() {
    let (_, record) = record_of_seeded(31);

    // Decision 20 takes a slot that is not legal there.
    let mut illegal = record.clone();
    let index = 20;
    let bad = (0..160u16)
        .find(|slot| !illegal.decisions[index].legal.contains(slot))
        .unwrap();
    illegal.decisions[index].chosen = bad;
    let error = illegal.replay().unwrap_err();
    assert_eq!(
        error,
        RecordError::Illegal {
            decision: index,
            slot: bad
        }
    );
    let message = error.to_string();
    assert!(message.contains("decision 20"), "{message}");
    assert!(message.contains(&bad.to_string()), "{message}");
    // The same through JSON.
    let error = Record::from_json(&illegal.to_json())
        .unwrap()
        .replay()
        .unwrap_err();
    assert!(error.to_string().contains("decision 20"));

    // Every position of the record: an illegal slot there names that index.
    for index in [0, 1, 5, record.decisions.len() - 1] {
        let mut illegal = record.clone();
        illegal.decisions[index].chosen = 159; // a reserved slot
        assert_eq!(
            illegal.replay().unwrap_err(),
            RecordError::Illegal {
                decision: index,
                slot: 159
            }
        );
        illegal.decisions[index].chosen = 60_000; // not a slot at all
        assert_eq!(
            illegal.replay().unwrap_err(),
            RecordError::Illegal {
                decision: index,
                slot: 60_000
            }
        );
    }

    // A legal action that is not the recorded deal: what follows no longer
    // fits. The error names the first decision that does not match.
    let mut wrong_seat = record.clone();
    wrong_seat.decisions[3].seat = (wrong_seat.decisions[3].seat + 1) % 4;
    assert!(matches!(
        wrong_seat.replay().unwrap_err(),
        RecordError::Mismatch { decision: 3, .. }
    ));
    let mut wrong_legal = record.clone();
    wrong_legal.decisions[7].legal.push(159);
    let error = wrong_legal.replay().unwrap_err();
    assert!(matches!(error, RecordError::Mismatch { decision: 7, .. }));
    assert!(error.to_string().contains("decision 7"));

    // Too few decisions, too many, changed entries, another version.
    let mut short = record.clone();
    short.decisions.pop();
    assert_eq!(short.replay().unwrap_err(), RecordError::NotFinished);
    let mut long = record.clone();
    let last = long.decisions[0].clone();
    long.decisions.push(last);
    assert!(matches!(
        long.replay().unwrap_err(),
        RecordError::Illegal { .. } | RecordError::Mismatch { .. }
    ));
    let mut entries = record.clone();
    entries.entries[0] += 5;
    assert!(matches!(
        entries.replay().unwrap_err(),
        RecordError::Entries { .. }
    ));
    let mut parts = record.clone();
    parts.parts[1][7] -= 20;
    assert!(matches!(
        parts.replay().unwrap_err(),
        RecordError::Entries { .. }
    ));
    let mut version = record.clone();
    version.record_version += 1;
    assert!(matches!(
        version.replay().unwrap_err(),
        RecordError::Version { .. }
    ));
    let mut encoding = record.clone();
    encoding.encoding_version += 1;
    assert!(matches!(
        encoding.replay().unwrap_err(),
        RecordError::Version { .. }
    ));
    let mut seed = record.clone();
    seed.seed = "not a number".to_string();
    assert!(matches!(seed.replay().unwrap_err(), RecordError::Header(_)));
    let mut dealer = record.clone();
    dealer.dealer = 4;
    assert!(matches!(
        dealer.replay().unwrap_err(),
        RecordError::Header(_)
    ));

    // Text that is no record.
    assert!(matches!(Record::from_json("{"), Err(RecordError::Json(_))));
    assert!(matches!(
        Record::from_json("{\"seed\": 1}"),
        Err(RecordError::Json(_))
    ));
}

/// Review items A6 and D14: the header is checked, not only read, and a
/// record has no fields the engine does not know.
#[test]
fn c25_a_record_with_another_preset_toggle_seed_form_or_field_is_refused() {
    let (_, record) = record_of_seeded(31);
    assert!(record.replay().is_ok());
    let header = |record: &Record| -> String {
        match record.replay().unwrap_err() {
            RecordError::Header(why) => why,
            other => panic!("not a header error: {other:?}"),
        }
    };

    // A preset the engine does not play.
    let mut preset = record.clone();
    preset.preset = "tournament".to_string();
    let why = header(&preset);
    assert!(
        why.contains("tournament") && why.contains("default"),
        "{why}"
    );

    // A toggle with another option, a missing toggle, an unknown toggle.
    // The message names the toggle.
    let mut other_option = record.clone();
    other_option
        .toggles
        .insert("S28".to_string(), "a".to_string());
    let why = header(&other_option);
    assert!(why.contains("S28") && why.contains("\"a\""), "{why}");
    let mut missing = record.clone();
    missing.toggles.remove("S15");
    let why = header(&missing);
    assert!(why.contains("S15"), "{why}");
    let mut unknown = record.clone();
    unknown.toggles.insert("S99".to_string(), "a".to_string());
    let why = header(&unknown);
    assert!(why.contains("S99"), "{why}");
    // The same through JSON.
    let text = record.to_json().replace("\"S28\":\"b\"", "\"S28\":\"a\"");
    assert_ne!(text, record.to_json());
    assert!(matches!(
        Record::from_json(&text).unwrap().replay(),
        Err(RecordError::Header(_))
    ));

    // The seed has one written form: decimal digits, no sign, no leading
    // zero, no spaces, at most 2^64 - 1.
    for seed in [
        "+31",
        "031",
        " 31",
        "31 ",
        "",
        "-31",
        "3.1e1",
        "0x1f",
        "18446744073709551616",
    ] {
        let mut changed = record.clone();
        changed.seed = seed.to_string();
        let why = header(&changed);
        assert!(why.contains("seed"), "{seed:?}: {why}");
    }
    // Well-formed seeds get past the header; another deal then does not
    // fit the decisions.
    for seed in ["0", "18446744073709551615"] {
        let mut changed = record.clone();
        changed.seed = seed.to_string();
        assert!(!matches!(
            changed.replay().unwrap_err(),
            RecordError::Header(_)
        ));
    }

    // A field the engine does not know, at the top or in a decision, is
    // refused when the text is read. The message names it.
    let text = record.to_json();
    let extra_top = text.replacen('{', "{\"note\":1,", 1);
    let Err(RecordError::Json(why)) = Record::from_json(&extra_top) else {
        panic!("an unknown top-level field was accepted");
    };
    assert!(why.contains("note"), "{why}");
    let extra_decision = text.replacen("{\"seat\":", "{\"value\":0.5,\"seat\":", 1);
    assert_ne!(extra_decision, text);
    let Err(RecordError::Json(why)) = Record::from_json(&extra_decision) else {
        panic!("an unknown field of a decision was accepted");
    };
    assert!(why.contains("value"), "{why}");
    // The optional `agent` field stays free-form.
    let with_agent = text.replacen('{', "{\"agent\":{\"anything\":[1,{\"goes\":true}]},", 1);
    assert!(Record::from_json(&with_agent).unwrap().replay().is_ok());
}

#[test]
fn c26_the_record_of_a_seeded_deal_equals_the_golden_file() {
    // Seed 42, seat 3 deals, every seat takes its lowest legal action.
    let mut game = Game::from_seed(42, D);
    finish(&mut game);
    let record = Record::of_game(42, &game).unwrap();
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/record_seed_42.json");
    let golden = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
        .replace("\r\n", "\n");
    assert_eq!(
        record.to_json_pretty(),
        golden,
        "the record changed: see tests/golden"
    );
    // The file replays to the same entries.
    let parsed = Record::from_json(&golden).unwrap();
    assert_eq!(parsed, record);
    assert_eq!(parsed.replay().unwrap().score(), game.score());
    assert_eq!(parsed.seed, "42");
    assert_eq!(parsed.dealer, 3);
}

/// Spec 009, criterion 39: the decisions of the round of announcements are
/// in the record like every other decision.
#[test]
fn c39_a_record_with_announcements_and_kontras_replays_to_the_same_entries() {
    let mut announcements = 0usize;
    let mut kontras = 0usize;
    let mut multiplied = 0usize;
    for seed in 0..300u64 {
        let game = seeded_game_b(seed);
        let record = Record::of_game(seed, &game).expect("a finished deal");
        assert_eq!(record.engine_version, "0.3.0");
        assert_eq!(record.engine_version, tarok_engine::VERSION);

        // One decision per action, the round included.
        let actions: Vec<Action> = game.full().actions().collect();
        assert_eq!(record.decisions.len(), actions.len());
        for (decision, action) in record.decisions.iter().zip(&actions) {
            assert_eq!(usize::from(decision.chosen), action.slot());
            assert!(decision.legal.contains(&decision.chosen));
            match action {
                Action::Announce(_) => {
                    assert!((77..82).contains(&decision.chosen));
                    announcements += 1;
                }
                Action::Kontra(_) => {
                    assert!((82..88).contains(&decision.chosen));
                    kontras += 1;
                }
                // "Done" is always among the legal slots of the round.
                Action::Done => assert_eq!(decision.chosen, 88),
                _ => assert!(!(77..90).contains(&decision.chosen)),
            }
            if (77..=88).contains(&decision.chosen) {
                assert!(decision.legal.contains(&88));
            }
        }

        // Through JSON, in both layouts, and played again: the same game,
        // the same entries, the same parts.
        for text in [record.to_json(), record.to_json_pretty()] {
            let read = Record::from_json(&text).unwrap();
            assert_eq!(read, record);
            let replayed = read
                .replay()
                .unwrap_or_else(|error| panic!("seed {seed}: {error}"));
            assert_eq!(replayed, game, "seed {seed}");
            let rewards = encode::rewards(&replayed.score().unwrap());
            assert_eq!(rewards.entries, record.entries);
            assert_eq!(rewards.entry_parts, record.parts);
            for bonus in Bonus::ALL {
                assert_eq!(
                    replayed.public().announced(bonus),
                    game.public().announced(bonus)
                );
            }
            for target in Target::ALL {
                assert_eq!(
                    replayed.public().kontra_level(target),
                    game.public().kontra_level(target)
                );
            }
        }
        // The parts of a seat add up to its entry, multiplied parts too.
        for seat in 0..4 {
            assert_eq!(record.parts[seat].iter().sum::<i32>(), record.entries[seat]);
        }
        let game_level = game.public().kontra_level(Target::Game);
        multiplied += usize::from(game_level > 0 && record.parts.iter().any(|row| row[0] != 0));
    }
    assert!(announcements > 500, "{announcements} announcements");
    assert!(kontras > 500, "{kontras} kontras");
    assert!(multiplied > 20, "{multiplied} deals with a multiplied game");

    // A decision of the round that is not legal is refused with its index.
    let game = seeded_game_b(7);
    let record = Record::of_game(7, &game).unwrap();
    let first_of_the_round = record
        .decisions
        .iter()
        .position(|decision| decision.legal.contains(&88))
        .expect("a round");
    // Slot 89 (barvni valat) is never legal.
    let mut bad = record.clone();
    bad.decisions[first_of_the_round].chosen = 89;
    assert_eq!(
        bad.replay(),
        Err(RecordError::Illegal {
            decision: first_of_the_round,
            slot: 89
        })
    );
    // A kontra on a bonus nobody announced: trula, at the first decision
    // of the round.
    let mut bad = record.clone();
    assert!(!bad.decisions[first_of_the_round].legal.contains(&83));
    bad.decisions[first_of_the_round].chosen = 83;
    assert_eq!(
        bad.replay(),
        Err(RecordError::Illegal {
            decision: first_of_the_round,
            slot: 83
        })
    );
}

/// Spec 009, criterion 40 (D135): a record written by another engine
/// version.
#[test]
fn c40_a_record_of_another_engine_version_keeps_going_and_names_both_versions() {
    let seed = 7;
    let game = seeded_game_b(seed);
    let record = Record::of_game(seed, &game).unwrap();
    assert_eq!(record.engine_version, "0.3.0");
    let other_version = |record: &Record| {
        let mut other = record.clone();
        other.engine_version = "0.2.0".to_string();
        other
    };
    let names_both = |error: &RecordError| {
        let text = error.to_string();
        assert!(text.contains("0.2.0"), "{text}");
        assert!(text.contains("0.3.0"), "{text}");
    };

    // 1. Its decisions are still legal and give the same entries: the
    // replay runs to the end.
    let other = other_version(&record);
    assert_eq!(other.replay().as_ref(), Ok(&game));
    // Through JSON too.
    let read = Record::from_json(&other.to_json_pretty()).unwrap();
    assert_eq!(read.engine_version, "0.2.0");
    assert_eq!(read.replay().as_ref(), Ok(&game));
    // Another engine may have had other legal actions at a decision. As
    // long as the chosen one is legal here, the replay keeps going. For a
    // record of this engine's own version the legal slots must fit.
    let mut fewer = record.clone();
    let decision = fewer
        .decisions
        .iter()
        .position(|decision| decision.legal.len() > 1)
        .unwrap();
    let chosen = fewer.decisions[decision].chosen;
    fewer.decisions[decision]
        .legal
        .retain(|&slot| slot == chosen);
    assert_eq!(
        fewer.replay(),
        Err(RecordError::Mismatch {
            decision,
            what: "the legal slots differ"
        })
    );
    assert_eq!(other_version(&fewer).replay().as_ref(), Ok(&game));

    // 2. The entries differ: the error names both versions.
    let mut entries = other_version(&record);
    entries.entries[0] += 5;
    let error = entries.replay().unwrap_err();
    names_both(&error);
    let RecordError::OtherEngine {
        recorded,
        running,
        error: inner,
    } = &error
    else {
        panic!("{error:?}");
    };
    assert_eq!((recorded.as_str(), running.as_str()), ("0.2.0", "0.3.0"));
    assert!(matches!(**inner, RecordError::Entries { .. }), "{inner:?}");
    // The parts differ, the entries do not: also named.
    let mut parts = other_version(&record);
    parts.parts[0][0] += 5;
    parts.parts[0][1] -= 5;
    names_both(&parts.replay().unwrap_err());

    // 3. A decision is not legal here. A record as engine 0.2.0 wrote it
    // has no round of announcements: the decisions of this deal without
    // the round.
    let mut old = other_version(&record);
    old.decisions
        .retain(|decision| !decision.legal.contains(&88));
    assert!(old.decisions.len() < record.decisions.len());
    let error = old.replay().unwrap_err();
    names_both(&error);
    let RecordError::OtherEngine { error: inner, .. } = &error else {
        panic!("{error:?}");
    };
    // The first decision that does not fit is the first card: the round
    // comes before it in this engine.
    let first_card = old
        .decisions
        .iter()
        .position(|decision| decision.chosen >= 90)
        .unwrap();
    match **inner {
        RecordError::Illegal { decision, .. } | RecordError::Mismatch { decision, .. } => {
            assert_eq!(decision, first_card);
        }
        ref other => panic!("{other:?}"),
    }
    // A decision with a slot that is not legal, in the middle.
    let mut illegal = other_version(&record);
    illegal.decisions[first_card].chosen = 89;
    let error = illegal.replay().unwrap_err();
    names_both(&error);
    // A record that stops early.
    let mut short = other_version(&record);
    short.decisions.pop();
    let error = short.replay().unwrap_err();
    names_both(&error);
    assert!(matches!(
        error,
        RecordError::OtherEngine { ref error, .. } if **error == RecordError::NotFinished
    ));

    // 4. The same faults in a record of this engine's version: the plain
    // errors, as before.
    let mut entries = record.clone();
    entries.entries[0] += 5;
    assert!(matches!(entries.replay(), Err(RecordError::Entries { .. })));
    let mut short = record.clone();
    short.decisions.pop();
    assert_eq!(short.replay(), Err(RecordError::NotFinished));

    // 5. The header is checked as before, whatever the version: another
    // preset is not a matter of the engine version.
    let mut preset = other_version(&record);
    preset.preset = "other".to_string();
    assert!(matches!(preset.replay(), Err(RecordError::Header(_))));
    // Any text is taken as a version.
    let mut odd = record.clone();
    odd.engine_version = "not a version".to_string();
    assert_eq!(odd.replay().as_ref(), Ok(&game));
}
