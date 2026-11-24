//! Spec 009, criteria 34-38: the round of announcements in the observation.
//!
//! Slots, tokens, state block D, the sequence limit, and hidden sides.
//! Seats: A is forehand (seat 0), then B, C, D; D deals. The built tests
//! use `common::flow::round_deal`: A declares, B holds the pagat, C holds
//! the srce kralj and is the hidden partner in tri.

mod common;

use common::encoding::{features, one_hot, relative};
use common::flow::{
    A, B, C, D, act, declare, exchange_back_open, game, pass_round, round_deal, round_game, say,
    seeded_game_b, seeded_game_b_with,
};
use tarok_engine::Contract::{SoloTri, Tri};
use tarok_engine::encode::{self, Observation, observe};
use tarok_engine::game::{MAX_ACTIONS, MAX_EVENTS};
use tarok_engine::{
    Action, Bonus, Card, Deal, EventKind, Game, Phase, PublicPartner, Rank, Seat, Suit, Target,
};

fn ann(bonus: Bonus) -> Action {
    Action::Announce(bonus)
}

fn k(target: Target) -> Action {
    Action::Kontra(target)
}

/// A group of `docs/encoding.md` by name: first id or slot, count.
fn group(groups: &[(&str, usize, usize)], name: &str) -> (usize, usize) {
    let (_, first, count) = groups
        .iter()
        .find(|(group, _, _)| *group == name)
        .unwrap_or_else(|| panic!("no group {name}"));
    (*first, *count)
}

#[test]
fn c34_the_round_uses_the_slots_77_to_88() {
    // The order of `docs/encoding.md` section 5. `encoding_contract.rs`
    // compares these groups with the document.
    assert_eq!(group(&encode::ACTION_GROUPS, "ANNOUNCE"), (77, 5));
    assert_eq!(group(&encode::ACTION_GROUPS, "KONTRA"), (82, 6));
    assert_eq!(group(&encode::ACTION_GROUPS, "ANNOUNCE_DONE"), (88, 1));
    let announcements = [
        Bonus::Trula,
        Bonus::Kralji,
        Bonus::KraljUltimo,
        Bonus::PagatUltimo,
        Bonus::Valat,
    ];
    for (index, bonus) in announcements.into_iter().enumerate() {
        assert_eq!(ann(bonus).slot(), 77 + index);
        assert_eq!(Action::from_slot(77 + index), Some(ann(bonus)));
    }
    let targets = [
        Target::Game,
        Target::Trula,
        Target::Kralji,
        Target::KraljUltimo,
        Target::PagatUltimo,
        Target::Valat,
    ];
    for (index, target) in targets.into_iter().enumerate() {
        assert_eq!(k(target).slot(), 82 + index);
        assert_eq!(Action::from_slot(82 + index), Some(k(target)));
    }
    assert_eq!(Action::Done.slot(), 88);
    assert_eq!(Action::from_slot(88), Some(Action::Done));
    // Slot 89 (barvni valat) has no action.
    assert_eq!(Action::from_slot(89), None);

    // Random deals of flow B. In the round every legal action lies in the
    // slots 77 to 88 and the mask shows exactly the legal set. Outside the
    // round none of the slots 77 to 89 is legal. Slot 89 never is.
    let mut in_the_round = 0u32;
    let mut used = [false; 12];
    for seed in 0..1_000u64 {
        seeded_game_b_with(seed, |game| {
            let seat = game.to_act().expect("a seat acts");
            let legal = game.legal_actions();
            let mask = observe(game, seat).mask;
            assert_eq!(mask.len(), encode::N_ACTIONS);
            for (slot, &bit) in mask.iter().enumerate() {
                assert_eq!(bit, legal.contains_slot(slot), "seed {seed}, slot {slot}");
            }
            assert!(!mask[89]);
            let round = game.phase() == Phase::Announcements;
            in_the_round += u32::from(round);
            for action in legal.iter() {
                let slot = action.slot();
                assert_eq!(
                    (77..=88).contains(&slot),
                    round,
                    "seed {seed}: {action:?} in {:?}",
                    game.phase()
                );
                assert_eq!(Action::from_slot(slot), Some(action));
                if round {
                    used[slot - 77] = true;
                    let fits = match action {
                        Action::Announce(_) => (77..82).contains(&slot),
                        Action::Kontra(_) => (82..88).contains(&slot),
                        Action::Done => slot == 88,
                        _ => false,
                    };
                    assert!(fits, "seed {seed}: {action:?} on slot {slot}");
                }
            }
        });
    }
    assert!(
        in_the_round > 10_000,
        "{in_the_round} decisions in the round"
    );
    assert_eq!(
        used, [true; 12],
        "every slot of the round was legal somewhere"
    );
}

/// The token ids of the round, from the document's table.
const ANNOUNCE: i16 = 146;
const KONTRA: i16 = 151;

#[test]
fn c35_every_announcement_and_kontra_writes_one_token_with_the_acting_seat() {
    assert_eq!(group(&encode::EVENT_GROUPS, "ANNOUNCE"), (146, 5));
    assert_eq!(group(&encode::EVENT_GROUPS, "KONTRA"), (151, 6));
    for (index, bonus) in Bonus::ALL.into_iter().enumerate() {
        assert_eq!(encode::event_id(EventKind::Announce(bonus)), 146 + index);
    }
    for (index, target) in Target::ALL.into_iter().enumerate() {
        assert_eq!(encode::event_id(EventKind::Kontra(target)), 151 + index);
    }

    // A round written out: who says what, and the token it must write.
    // `None` is "done".
    let mut game = round_game(C, Tri);
    let before = observe(&game, D).tokens;
    let turns: [(Seat, &[(Action, i16)]); 6] = [
        (
            A,
            &[
                (ann(Bonus::Trula), ANNOUNCE),
                (ann(Bonus::Valat), ANNOUNCE + 4),
            ],
        ),
        (
            B,
            &[
                (k(Target::Game), KONTRA),
                (ann(Bonus::PagatUltimo), ANNOUNCE + 3),
                (k(Target::Valat), KONTRA + 5),
                (k(Target::Trula), KONTRA + 1),
            ],
        ),
        (
            C,
            &[
                (ann(Bonus::KraljUltimo), ANNOUNCE + 2),
                (ann(Bonus::Kralji), ANNOUNCE + 1),
                (k(Target::PagatUltimo), KONTRA + 4),
            ],
        ),
        (
            D,
            &[
                (k(Target::KraljUltimo), KONTRA + 3),
                (k(Target::Kralji), KONTRA + 2),
            ],
        ),
        (A, &[(k(Target::Game), KONTRA)]),
        (B, &[]),
    ];
    // What every observer must see after the start of the round: the id
    // and the seat that acted.
    let mut expected: Vec<(i16, Seat)> = Vec::new();
    for (seat, actions) in turns {
        for &(action, id) in actions {
            let length = game.public().events().len();
            act(&mut game, seat, action);
            // One token per announcement and per kontra.
            assert_eq!(game.public().events().len(), length + 1, "{action:?}");
            expected.push((id, seat));
        }
        let length = game.public().events().len();
        act(&mut game, seat, Action::Done);
        // "Done" writes no token (reading 1 of the spec).
        assert_eq!(game.public().events().len(), length, "done by {seat:?}");
        for observer in Seat::ALL {
            let tokens = observe(&game, observer).tokens;
            assert_eq!(tokens.len(), before.len() + expected.len());
            for (index, &(id, actor)) in expected.iter().enumerate() {
                let row = tokens[before.len() + index];
                let place = (before.len() + index) as i16;
                // Event, the acting seat as the observer sees it, no
                // trick, no position, the index in the sequence.
                assert_eq!(
                    row,
                    [id, relative(observer, actor) as i16, 0, 0, place],
                    "token {index} seen by {observer:?}"
                );
            }
        }
    }
    assert_eq!(expected.len(), 12);
    // The round goes on to its end and the play follows on the tokens of
    // the round.
    pass_round(&mut game);
    let first_card = game.legal_actions().iter().next().unwrap();
    act(&mut game, A, first_card);
    let tokens = observe(&game, A).tokens;
    assert_eq!(tokens.len(), before.len() + 12 + 1);
    let last = tokens[tokens.len() - 1];
    assert!((159..213).contains(&last[0]), "a played card");
    assert_eq!((last[2], last[3]), (1, 1), "trick 1, first card");

    // Random deals of flow B: the tokens of the round are the
    // announcements and kontras of the deal, in order. The ids 157
    // ("done") and 158 (barvni valat) are never written.
    let mut said_in_all = 0usize;
    for seed in 0..1_000u64 {
        let game = seeded_game_b(seed);
        let said: Vec<(i16, Seat)> = {
            // The seat of each action: replay and ask who acts.
            let mut replay = Game::from_seed(seed, game.public().dealer());
            let mut said = Vec::new();
            for action in game.full().actions() {
                let seat = replay.to_act().unwrap();
                match action {
                    Action::Announce(bonus) => said.push((ANNOUNCE + bonus.index() as i16, seat)),
                    Action::Kontra(target) => said.push((KONTRA + target.index() as i16, seat)),
                    _ => {}
                }
                replay.apply(action).unwrap();
            }
            said
        };
        said_in_all += said.len();
        for observer in [A, C] {
            let tokens = observe(&game, observer).tokens;
            let of_the_round: Vec<(i16, i16)> = tokens
                .iter()
                .filter(|row| (146..=158).contains(&row[0]))
                .map(|row| (row[0], row[1]))
                .collect();
            let expected: Vec<(i16, i16)> = said
                .iter()
                .map(|&(id, seat)| (id, relative(observer, seat) as i16))
                .collect();
            assert_eq!(of_the_round, expected, "seed {seed}");
            // They lie between the exchange and the first played card.
            let first_play = tokens.iter().position(|row| row[0] >= 159).unwrap();
            let last_said = tokens.iter().rposition(|row| (146..=158).contains(&row[0]));
            assert!(last_said.is_none_or(|index| index < first_play));
        }
    }
    assert!(said_in_all > 5_000, "{said_in_all} things said");
}

/// What `observer` must see in D1 for an announcement of `announcer`:
/// (declarer's side, opponents), written from criteria 15 and 36 with the
/// public view and the observer's own view only.
fn d1_bits(game: &Game, observer: Seat, announcer: Seat) -> (bool, bool) {
    let public = game.public();
    let declarer = public.declarer().expect("a declarer");
    // The observer holds the called king or held it when the king was
    // called: it is the partner, or the declarer who called its own king.
    let partner = game.private(observer).is_called_partner();
    let own_king = observer == declarer
        && public
            .contract()
            .is_some_and(|contract| contract.calls_king())
        && !public.king_in_talon()
        && public.called_king().is_some_and(|king| {
            // The declarer's own king: in its hand, or played by it.
            game.private(observer).hand().contains(king)
                || public.tricks().iter().any(|trick| {
                    let position = trick.cards.iter().position(|&card| card == king);
                    position.is_some_and(|position| trick.leader.plus(position) == observer)
                })
                || {
                    let on_table = public.current_trick();
                    let position = on_table.iter().position(|&card| card == king);
                    position.is_some_and(|position| {
                        public.leader().expect("a leader").plus(position) == observer
                    })
                }
        });
    let side = if announcer == declarer {
        Some(true)
    } else if announcer == observer {
        Some(observer == declarer || partner)
    } else {
        match public.partner() {
            PublicPartner::With(seat) => Some(announcer == seat),
            PublicPartner::Alone => Some(false),
            PublicPartner::NotKnown if partner || own_king => Some(false),
            PublicPartner::NotKnown => None,
        }
    };
    match side {
        Some(true) => (true, false),
        Some(false) => (false, true),
        // Unknown side: both bits (reading 2 of the spec).
        None => (true, true),
    }
}

/// Block D of an observation, checked against the public state: D1 by
/// `d1_bits`, D2 one level per target, D3 zero.
fn check_block_d(game: &Game, observer: Seat, observation: &Observation) {
    let public = game.public();
    let mut d1 = [0.0f32; 10];
    for bonus in Bonus::ALL {
        if let Some(announcer) = public.announced(bonus) {
            let (declarers, opponents) = d1_bits(game, observer, announcer);
            d1[bonus.index()] = f32::from(u8::from(declarers));
            d1[5 + bonus.index()] = f32::from(u8::from(opponents));
        }
    }
    assert_eq!(features(observation, "D1"), d1, "D1 seen by {observer:?}");
    let mut d2 = [0.0f32; 30];
    for target in Target::ALL {
        d2[5 * target.index() + usize::from(public.kontra_level(target))] = 1.0;
    }
    assert_eq!(features(observation, "D2"), d2, "D2 seen by {observer:?}");
    assert_eq!(features(observation, "D3"), [0.0]);
}

#[test]
fn c36_state_block_d_holds_the_announcements_by_side_and_the_kontra_levels() {
    // D1: declarer's side trula, kralji, kralj ultimo, pagat ultimo, valat;
    // then the opponents' five.
    let d1 = |game: &Game, observer: Seat| features(&observe(game, observer), "D1").to_vec();
    let bits = |set: &[usize]| {
        let mut bits = vec![0.0f32; 10];
        for &index in set {
            bits[index] = 1.0;
        }
        bits
    };
    const TRULA: usize = 0;
    const KRALJI: usize = 1;
    const KRALJ_ULTIMO: usize = 2;
    const VALAT: usize = 4;
    const OPPONENTS: usize = 5;

    let mut game = round_game(C, Tri);
    // Before anything is said: no announcement, every target at "none".
    for observer in Seat::ALL {
        let observation = observe(&game, observer);
        assert_eq!(features(&observation, "D1"), [0.0; 10]);
        let none: Vec<f32> = (0..6).flat_map(|_| one_hot(5, 0)).collect();
        assert_eq!(features(&observation, "D2"), none);
        assert_eq!(features(&observation, "B1"), one_hot(8, 5), "the phase");
    }

    // The declarer announces trula: every seat knows the side.
    say(&mut game, A, &[ann(Bonus::Trula)]);
    for observer in Seat::ALL {
        assert_eq!(d1(&game, observer), bits(&[TRULA]), "{observer:?}");
    }

    // B, an opponent, announces kralji while the partner is hidden. B
    // knows its own side. C holds the called king and knows that B is an
    // opponent. A and D cannot know: both bits.
    say(&mut game, B, &[ann(Bonus::Kralji)]);
    let known = bits(&[TRULA, OPPONENTS + KRALJI]);
    let unknown = bits(&[TRULA, KRALJI, OPPONENTS + KRALJI]);
    assert_eq!(d1(&game, A), unknown);
    assert_eq!(d1(&game, B), known);
    assert_eq!(d1(&game, C), known);
    assert_eq!(d1(&game, D), unknown);

    // C, the hidden partner, announces valat. Only C knows the side.
    act(&mut game, C, ann(Bonus::Valat));
    assert_eq!(
        d1(&game, C),
        bits(&[TRULA, VALAT, OPPONENTS + KRALJI]),
        "C knows every side"
    );
    assert_eq!(
        d1(&game, A),
        bits(&[TRULA, KRALJI, VALAT, OPPONENTS + KRALJI, OPPONENTS + VALAT])
    );
    assert_eq!(
        d1(&game, B),
        bits(&[TRULA, VALAT, OPPONENTS + KRALJI, OPPONENTS + VALAT]),
        "B knows its own side, not C's"
    );
    assert_eq!(
        d1(&game, D),
        bits(&[TRULA, KRALJI, VALAT, OPPONENTS + KRALJI, OPPONENTS + VALAT])
    );
    for observer in Seat::ALL {
        check_block_d(&game, observer, &observe(&game, observer));
    }

    // C announces kralj ultimo: the partner is public and every bit is on
    // one side for every seat.
    act(&mut game, C, ann(Bonus::KraljUltimo));
    assert_eq!(game.public().partner(), PublicPartner::With(C));
    let all_known = bits(&[TRULA, KRALJ_ULTIMO, VALAT, OPPONENTS + KRALJI]);
    for observer in Seat::ALL {
        assert_eq!(d1(&game, observer), all_known, "{observer:?}");
        // B6, the revealed partner seat, shows C.
        let b6 = features(&observe(&game, observer), "B6").to_vec();
        assert_eq!(b6, one_hot(4, relative(observer, C)));
    }
    act(&mut game, C, Action::Done);

    // D2: one level per target, in the order game, trula, kralji, kralj
    // ultimo, pagat ultimo, valat; each none, kontra, re, sub, mort.
    let d2 = |game: &Game, observer: Seat| features(&observe(game, observer), "D2").to_vec();
    let levels = |levels: [usize; 6]| -> Vec<f32> {
        levels
            .into_iter()
            .flat_map(|level| one_hot(5, level))
            .collect()
    };
    assert_eq!(d2(&game, D), levels([0; 6]));
    say(
        &mut game,
        D,
        &[k(Target::Game), k(Target::Trula), k(Target::Valat)],
    );
    for observer in Seat::ALL {
        assert_eq!(
            d2(&game, observer),
            levels([1, 1, 0, 0, 0, 1]),
            "{observer:?}"
        );
    }
    say(&mut game, A, &[k(Target::Game), k(Target::Kralji)]);
    assert_eq!(d2(&game, B), levels([2, 1, 1, 0, 0, 1]));
    say(&mut game, B, &[k(Target::Game), k(Target::Kralji)]);
    say(&mut game, C, &[k(Target::Game)]);
    for observer in Seat::ALL {
        assert_eq!(
            d2(&game, observer),
            levels([4, 1, 2, 0, 0, 1]),
            "{observer:?}"
        );
        check_block_d(&game, observer, &observe(&game, observer));
    }
    // The block stays through the play.
    pass_round(&mut game);
    let card = game.legal_actions().iter().next().unwrap();
    act(&mut game, A, card);
    for observer in Seat::ALL {
        assert_eq!(d1(&game, observer), all_known);
        assert_eq!(d2(&game, observer), levels([4, 1, 2, 0, 0, 1]));
    }

    // Random deals of flow B: at every decision, for every seat, block D
    // is what follows from the public state and the seat's own view.
    let mut with_both_bits = 0u32;
    let mut checked = 0u32;
    for seed in 0..300u64 {
        seeded_game_b_with(seed, |game| {
            for observer in Seat::ALL {
                let observation = observe(game, observer);
                check_block_d(game, observer, &observation);
                checked += 1;
                let d1 = features(&observation, "D1");
                with_both_bits += u32::from((0..5).any(|i| d1[i] == 1.0 && d1[5 + i] == 1.0));
            }
        });
    }
    assert!(checked > 80_000, "{checked} observations checked");
    assert!(
        with_both_bits > 5_000,
        "{with_both_bits} with an unknown side"
    );
}

/// The longest round: all five announcements and all 24 kontras.
fn longest_round() -> Game {
    use Target::{Game as OnGame, KraljUltimo, Kralji, PagatUltimo, Trula, Valat};
    let mut game = round_game(C, Tri);
    let all_four = [k(OnGame), k(Trula), k(Kralji), k(Valat)];
    say(
        &mut game,
        A,
        &[ann(Bonus::Trula), ann(Bonus::Kralji), ann(Bonus::Valat)],
    );
    // B: kontra on the game and on A's three; B holds the pagat.
    let mut b = all_four.to_vec();
    b.push(ann(Bonus::PagatUltimo));
    say(&mut game, B, &b);
    // C: kralj ultimo, which shows C as the partner; re on the four;
    // kontra on B's pagat ultimo.
    let mut c = vec![ann(Bonus::KraljUltimo)];
    c.extend(all_four);
    c.push(k(PagatUltimo));
    say(&mut game, C, &c);
    // D: kontra on C's kralj ultimo; sub on the four; re on the pagat
    // ultimo of its side.
    let mut d = vec![k(KraljUltimo)];
    d.extend(all_four);
    d.push(k(PagatUltimo));
    say(&mut game, D, &d);
    // A: re on the kralj ultimo; mort on the four; sub on the pagat ultimo.
    let mut a = vec![k(KraljUltimo)];
    a.extend(all_four);
    a.push(k(PagatUltimo));
    say(&mut game, A, &a);
    say(&mut game, B, &[k(KraljUltimo), k(PagatUltimo)]);
    say(&mut game, C, &[k(KraljUltimo)]);
    assert_eq!(common::flow::levels(&game), [4; 6]);
    for bonus in Bonus::ALL {
        assert!(game.public().announced(bonus).is_some());
    }
    // Nothing is left to say: only "done" is legal, for every seat.
    for _ in 0..3 {
        assert_eq!(game.phase(), Phase::Announcements);
        assert_eq!(game.legal_actions().len(), 1);
        assert!(game.legal_actions().contains(Action::Done));
        game.apply(Action::Done).unwrap();
    }
    assert_eq!(game.phase(), Phase::Play);
    game
}

#[test]
fn c37_no_legal_deal_has_more_than_128_tokens() {
    assert_eq!(encode::MAX_TOKENS, 128);
    assert_eq!(MAX_EVENTS, encode::MAX_TOKENS);

    // The longest round, built: 5 announcements and 24 kontras write 29
    // tokens.
    let mut game = longest_round();
    let said = game
        .full()
        .actions()
        .filter(|action| matches!(action, Action::Announce(_) | Action::Kontra(_)))
        .count();
    assert_eq!(said, 29);
    // Before the round: the start, four calls, the contract, the king, six
    // talon cards and the packet. Then 29 from the round.
    assert_eq!(game.public().events().len(), 1 + 4 + 1 + 1 + 6 + 1 + 29);
    common::flow::finish(&mut game);
    assert_eq!(game.phase(), Phase::Over);
    let tokens = observe(&game, A).tokens.len();
    assert_eq!(tokens, 14 + 29 + 48);
    assert!(tokens <= encode::MAX_TOKENS);
    // The longest deal there can be: 26 tokens before the round (the
    // start, 13 calls, the contract, the king, six talon cards, the packet
    // and three shown discards; no deal has all of these at once), 29 in
    // the round, 48 cards.
    const { assert!(26 + 29 + 48 <= encode::MAX_TOKENS) };

    // 10,000 random deals of flow B.
    let mut longest = (0usize, 0u64);
    let mut most_actions = 0usize;
    let mut most_said = 0usize;
    for seed in 0..10_000u64 {
        let game = seeded_game_b(seed);
        let events = game.public().events().len();
        assert!(events <= encode::MAX_TOKENS, "seed {seed}: {events} tokens");
        if events > longest.0 {
            longest = (events, seed);
        }
        let actions = game.full().action_slots().len();
        assert!(actions <= MAX_ACTIONS, "seed {seed}: {actions} actions");
        most_actions = most_actions.max(actions);
        let said = game
            .full()
            .actions()
            .filter(|action| matches!(action, Action::Announce(_) | Action::Kontra(_)))
            .count();
        assert!(said <= 29, "seed {seed}");
        most_said = most_said.max(said);
    }
    println!(
        "longest token sequence in 10,000 random deals of flow B: {} (seed {}); most actions \
         {most_actions}; most announcements and kontras {most_said}",
        longest.0, longest.1
    );
    // The deal with the most tokens gives that many rows, and the
    // observation of every seat has room for them.
    let game = seeded_game_b(longest.1);
    for observer in Seat::ALL {
        assert_eq!(observe(&game, observer).tokens.len(), longest.0);
    }
    assert!(longest.0 > 71, "longer than the longest deal of flow A");
}

/// What a seat sees must be equal in two worlds.
fn same_for_the_observer(one: &Observation, two: &Observation, what: &str) {
    assert_eq!(one.tokens, two.tokens, "{what}: the tokens differ");
    for (index, (a, b)) in one.state.iter().zip(&two.state).enumerate() {
        assert_eq!(a, b, "{what}: state feature {index} differs");
    }
    assert_eq!(one.mask, two.mask, "{what}: the mask differs");
}

#[test]
fn c38_hidden_sides_stay_hidden_in_the_observation() {
    // Two deals. In the first C holds the called srce kralj and B the srce
    // kaval; in the second the two cards have changed places. So C is the
    // partner in one world and B in the other. The hands of A and D and
    // everything that is said and played are the same.
    let first = round_deal(C);
    let second = round_deal(B);
    let king = Card::court(Suit::Srce, Rank::Kralj);
    let kaval = Card::court(Suit::Srce, Rank::Kaval);
    let start = |deal: &Deal| {
        let mut game = game(deal);
        declare(&mut game, A, Tri);
        act(&mut game, A, Action::CallKing(Suit::Srce));
        exchange_back_open(&mut game, 0);
        game
    };
    let mut games = [start(&first), start(&second)];
    assert_eq!(games[0].full().partner(), Some(C));
    assert_eq!(games[1].full().partner(), Some(B));
    assert!(games[0].private(C).hand().contains(king));
    assert!(games[1].private(C).hand().contains(kaval));

    // The round: things B and C may say in both worlds. Then cards that
    // B and C hold in both worlds.
    let tarok = |number| Card::tarok(number).unwrap();
    let karo = |number| Card::pip(Suit::Karo, number).unwrap();
    let kriz = |number| Card::pip(Suit::Kriz, number).unwrap();
    let script: Vec<Action> = vec![
        // A.
        ann(Bonus::Trula),
        Action::Done,
        // B: an announcement of a hidden side, and the pagat ultimo.
        ann(Bonus::Kralji),
        ann(Bonus::PagatUltimo),
        Action::Done,
        // C.
        ann(Bonus::Valat),
        Action::Done,
        // D: kontra on the game and on the declarer's trula.
        k(Target::Game),
        k(Target::Trula),
        Action::Done,
        // A: re on both.
        k(Target::Game),
        k(Target::Trula),
        Action::Done,
        // B, C, D pass: the round is over.
        Action::Done,
        Action::Done,
        Action::Done,
        // Trick 1: A leads the II, B plays the XII, C and D have no tarok.
        Action::Play(tarok(2)),
        Action::Play(tarok(12)),
        Action::Play(karo(2)),
        Action::Play(kriz(7)),
        // Trick 2: B leads the XIII.
        Action::Play(tarok(13)),
        Action::Play(karo(1)),
        Action::Play(kriz(8)),
        Action::Play(tarok(3)),
        // Trick 3: B leads the XIV, A takes it with the mond.
        Action::Play(tarok(14)),
        Action::Play(Card::court(Suit::Karo, Rank::Fant)),
        Action::Play(kriz(9)),
        Action::Play(Card::MOND),
    ];
    let mut compared = 0;
    let mut both_bits_seen = false;
    for (step, &action) in script.iter().enumerate() {
        let seat = games[0].to_act().expect("a seat acts");
        assert_eq!(games[1].to_act(), Some(seat), "step {step}");
        // A and D see the same in both worlds, at every moment: when it is
        // their turn (the mask is then the legal set) and when it is not.
        for observer in [A, D] {
            let here = observe(&games[0], observer);
            let there = observe(&games[1], observer);
            same_for_the_observer(&here, &there, &format!("step {step}, {observer:?}"));
            compared += 1;
            let d1 = features(&here, "D1");
            both_bits_seen |= d1[1] == 1.0 && d1[6] == 1.0;
        }
        // The critic's hidden features do tell the worlds apart.
        let hidden = |game: &Game| observe(game, A).critic[encode::STATE_FEATURES..].to_vec();
        assert_ne!(hidden(&games[0]), hidden(&games[1]));
        for game in &mut games {
            game.apply(action)
                .unwrap_or_else(|error| panic!("step {step}: {error}"));
        }
    }
    assert_eq!(compared, 2 * script.len());
    assert!(
        both_bits_seen,
        "an announcement of unknown side was in view"
    );
    assert_eq!(games[0].public().tricks().len(), 3);
    // The partner is still hidden in both worlds.
    for game in &games {
        assert_eq!(game.public().partner(), PublicPartner::NotKnown);
    }
    // And B and C, whose hands differ, do see different things.
    assert_ne!(
        observe(&games[0], C).state,
        observe(&games[1], C).state,
        "C's own hand differs"
    );

    // The same with a solo next to it, where nothing is hidden about the
    // sides: the two worlds differ in two hidden cards only.
    let start = |deal: &Deal| {
        let mut game = game(deal);
        declare(&mut game, A, SoloTri);
        exchange_back_open(&mut game, 0);
        game
    };
    let mut games = [start(&first), start(&second)];
    for (step, &action) in script.iter().enumerate() {
        // In a solo B and C are both opponents: valat by C and the kontras
        // are legal as written; the re of A too.
        for observer in [A, D] {
            let here = observe(&games[0], observer);
            let there = observe(&games[1], observer);
            same_for_the_observer(&here, &there, &format!("solo, step {step}, {observer:?}"));
        }
        for game in &mut games {
            game.apply(action)
                .unwrap_or_else(|error| panic!("solo, step {step}: {error}"));
        }
    }
}
