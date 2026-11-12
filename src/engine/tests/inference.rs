//! Spec 006, criteria 7 and 8: the exact inference bits, block F.
//!
//! F1 other seat is void (3 seats x srce, karo, pik, križ, taroks).
//! F2 other seat cannot hold the card (3 x 54).
//! F3 other seat certainly holds the card (3 x 54).
//! F4 card is unseen (54).
//! F5 own card is the highest unseen of its suit (54).
//! F6 derived counts (16).
//!
//! "Other seat" blocks are in relative order: the next seat, the seat
//! across, the previous seat.

mod common;

use common::encoding::{card_bits, features, relative, scripted_position};
use common::flow::{A, B, C, D, act, deal_with, declare, exchange, game, play_random};
use common::{court, kralj, pip, run_cases, seat, tarok};
use proptest::prelude::*;
use tarok_engine::encode::{Observation, observe};
use tarok_engine::{Action, Card, CardSet, Contract, Game, Rank, Seat, Suit};

/// The scripted position played on to the end of the third trick.
///
/// Trick 2: A XII, B XIII, C pik 7, D križ 7. C and D play no tarok on a
/// tarok lead: both are void in taroks. B wins.
/// Trick 3: B srce 1, C srce kaval, D križ 8, A III. D plays neither srce
/// nor tarok: void in srce (and in taroks). A trumps: void in srce. A wins.
///
/// Hands after it:
/// - A: I, IV-XI (nine taroks).
/// - B: XIV-XX, škis, srce fant.
/// - C: srce dama, karo fant, kaval, dama, kralj, pik 8, 9, 10, fant.
/// - D: pik kaval, dama, kralj, križ 9, 10, fant, kaval, dama, kralj.
///
/// Out of play: karo 4 (hidden discard), II (shown discard), and the talon
/// cards A did not take: karo 3, XXI, karo 2, karo 1. They are A's, because
/// A won a trick with the called king out of the packet it took.
fn after_three_tricks() -> Game {
    let mut game = scripted_position();
    for (seat, card) in [
        (C, pip(Suit::Pik, 7)),
        (D, pip(Suit::Kriz, 7)),
        (B, pip(Suit::Srce, 1)),
        (C, court(Suit::Srce, Rank::Kaval)),
        (D, pip(Suit::Kriz, 8)),
        (A, tarok(3)),
    ] {
        act(&mut game, seat, Action::Play(card));
    }
    assert_eq!(game.public().tricks().len(), 3);
    assert_eq!(game.to_act(), Some(A));
    game
}

const SRCE: usize = 0;
const TAROKS: usize = 4;

/// F1 of `observer` for `other`: five bits, srce, karo, pik, križ, taroks.
fn void_bits(observation: &Observation, observer: Seat, other: Seat) -> [f32; 5] {
    let index = relative(observer, other) - 1;
    features(observation, "F1")[index * 5..index * 5 + 5]
        .try_into()
        .unwrap()
}

/// The 54 bits of block `name` (F2 or F3) of `observer` for `other`.
fn about(observation: &Observation, name: &str, observer: Seat, other: Seat) -> CardSet {
    let index = relative(observer, other) - 1;
    let bits = &features(observation, name)[index * 54..index * 54 + 54];
    as_set(bits)
}

fn as_set(bits: &[f32]) -> CardSet {
    assert_eq!(bits.len(), 54);
    Card::all()
        .filter(|card| bits[usize::from(card.index())] == 1.0)
        .collect()
}

fn taroks(numbers: impl IntoIterator<Item = u8>) -> CardSet {
    numbers.into_iter().map(tarok).collect()
}

#[test]
fn c07_f1_a_void_is_shown_by_not_following() {
    // Before anyone fails to follow: no void is known.
    let before = scripted_position();
    for observer in Seat::ALL {
        assert!(
            features(&observe(&before, observer), "F1")
                .iter()
                .all(|&v| v == 0.0)
        );
    }

    let game = after_three_tricks();
    // The voids are public: every observer sees the same for a given seat.
    for observer in Seat::ALL {
        let observation = observe(&game, observer);
        let expected = |other: Seat| -> [f32; 5] {
            let mut bits = [0.0; 5];
            // C and D played a suit card on a tarok lead.
            if other == C || other == D {
                bits[TAROKS] = 1.0;
            }
            // D and A did not follow the srce lead.
            if other == D || other == A {
                bits[SRCE] = 1.0;
            }
            bits
        };
        for other in Seat::ALL {
            if other != observer {
                assert_eq!(
                    void_bits(&observation, observer, other),
                    expected(other),
                    "{} about {}",
                    observer.index(),
                    other.index()
                );
            }
        }
    }
    // B followed every lead: nothing about B.
    assert_eq!(void_bits(&observe(&game, A), A, B), [0.0; 5]);

    // Review item A5: a void is known as soon as the card lies on the
    // table, before the trick is over. In the scripted position A led the
    // XII and B followed. C plays the pik 7 on the tarok lead: D, who acts
    // next, already sees C void in taroks.
    let mut running = scripted_position();
    act(&mut running, C, Action::Play(pip(Suit::Pik, 7)));
    assert_eq!(running.public().current_trick().len(), 3);
    assert_eq!(running.to_act(), Some(D));
    for observer in [D, A, B] {
        let mut bits = [0.0; 5];
        bits[TAROKS] = 1.0;
        assert_eq!(
            void_bits(&observe(&running, observer), observer, C),
            bits,
            "seen by {}",
            observer.index()
        );
    }
    // Nothing yet about the others.
    assert_eq!(void_bits(&observe(&running, C), C, D), [0.0; 5]);
    assert_eq!(void_bits(&observe(&running, D), D, B), [0.0; 5]);
    // And F2 follows at once: C cannot hold a tarok.
    let cannot = about(&observe(&running, D), "F2", D, C);
    assert!(CardSet::TAROKS.is_subset(cannot));
}

#[test]
fn c07_f2_cannot_hold_from_a_seen_card_and_from_a_void() {
    let game = after_three_tricks();
    let from_b = observe(&game, B);

    // Seen cards that are out of every hand: played cards, B's own cards,
    // the shown discard, the talon cards nobody took.
    let gone = [
        kralj(Suit::Srce),  // played
        tarok(13),          // played by B itself
        Card::SKIS,         // in B's own hand
        tarok(2),           // the shown discard
        Card::MOND,         // talon, not taken
        pip(Suit::Karo, 3), // talon, not taken
    ];
    for other in [C, D, A] {
        let cannot = about(&from_b, "F2", B, other);
        for card in gone {
            assert!(cannot.contains(card), "{card} for seat {}", other.index());
        }
    }
    // The karo 4 was in the packet A took. C and D cannot hold it. A may:
    // B does not know that A put it down.
    let karo4 = pip(Suit::Karo, 4);
    assert!(about(&from_b, "F2", B, C).contains(karo4));
    assert!(about(&from_b, "F2", B, D).contains(karo4));
    assert!(!about(&from_b, "F2", B, A).contains(karo4));

    // From a void: D and A are void in srce, C and D in taroks.
    let srce_dama = court(Suit::Srce, Rank::Dama);
    assert!(about(&from_b, "F2", B, D).contains(srce_dama));
    assert!(about(&from_b, "F2", B, A).contains(srce_dama));
    assert!(!about(&from_b, "F2", B, C).contains(srce_dama));
    for number in [1u8, 4, 5, 11] {
        assert!(about(&from_b, "F2", B, C).contains(tarok(number)));
        assert!(about(&from_b, "F2", B, D).contains(tarok(number)));
        assert!(!about(&from_b, "F2", B, A).contains(tarok(number)));
    }
    // An unseen card of a suit in which nobody is void: nothing is known.
    let pik_kralj = kralj(Suit::Pik);
    for other in [C, D, A] {
        assert!(!about(&from_b, "F2", B, other).contains(pik_kralj));
    }

    // The whole block for D, as B sees it: everything B has seen (D took
    // nothing from the talon), all srce and all taroks.
    let seen_by_b = as_set(features(&from_b, "F4")).complement();
    let expected = seen_by_b | CardSet::suit(Suit::Srce) | CardSet::TAROKS;
    assert_eq!(about(&from_b, "F2", B, D), expected);
}

#[test]
fn c07_f3_certainly_holds_the_taken_king_and_by_elimination() {
    // The called king out of the packet the declarer took: it cannot be
    // put down, so the declarer holds it until it is played. Position:
    // after the exchange of the scripted deal, before the first card.
    let mut early = game(&common::flow::king_in_talon_deal(false));
    let dve = Action::Bid(Contract::Dve);
    for (seat, action) in [
        (B, dve),
        (C, Action::Pass),
        (D, Action::Pass),
        (A, dve),
        (B, Action::Pass),
    ] {
        act(&mut early, seat, action);
    }
    act(&mut early, A, dve);
    act(&mut early, A, Action::CallKing(Suit::Srce));
    exchange(&mut early, 0, &[pip(Suit::Karo, 4), tarok(2)]);
    for observer in [B, C, D] {
        let observation = observe(&early, observer);
        assert_eq!(
            about(&observation, "F3", observer, A),
            [kralj(Suit::Srce)].into_iter().collect(),
            "observer {}",
            observer.index()
        );
        // The two others then cannot hold it.
        for other in [B, C, D] {
            if other != observer {
                assert!(about(&observation, "F3", observer, other).is_empty());
                assert!(about(&observation, "F2", observer, other).contains(kralj(Suit::Srce)));
            }
        }
    }
    // Once it is played nobody holds it.
    let later = scripted_position();
    assert!(about(&observe(&later, B), "F3", B, A).is_empty());

    // A tarok out of the taken packet that was not shown as a discard is
    // in the declarer's hand once the discards are made. A plays solo tri,
    // takes X, karo 4, XI and puts down the karo 4 and two srce, hidden.
    let talon = [
        tarok(10),
        pip(Suit::Karo, 4),
        tarok(11),
        tarok(12),
        tarok(13),
        tarok(14),
    ];
    let deal = deal_with(
        [&[pip(Suit::Srce, 4), pip(Suit::Srce, 3)], &[], &[], &[]],
        &talon,
    );
    let mut solo = game(&deal);
    declare(&mut solo, A, Contract::SoloTri);
    act(&mut solo, A, Action::TakePacket(common::flow::packet(0)));
    // While the discards go on, nothing is certain yet.
    assert!(about(&observe(&solo, B), "F3", B, A).is_empty());
    for card in [pip(Suit::Karo, 4), pip(Suit::Srce, 4), pip(Suit::Srce, 3)] {
        act(&mut solo, A, Action::Discard(card));
    }
    let from_b = observe(&solo, B);
    // X and XI: certain. The karo 4: not (it may be among the hidden cards).
    assert_eq!(about(&from_b, "F3", B, A), taroks([10, 11]));
    assert!(!about(&from_b, "F2", B, A).contains(pip(Suit::Karo, 4)));

    // By elimination. After three tricks C and D are void in taroks, so
    // every tarok B has not seen is with A: I and IV-XI.
    let game = after_three_tricks();
    let from_b = observe(&game, B);
    let with_a = taroks([1, 4, 5, 6, 7, 8, 9, 10, 11]);
    assert_eq!(about(&from_b, "F3", B, A), with_a);
    assert!(about(&from_b, "F3", B, C).is_empty());
    assert!(about(&from_b, "F3", B, D).is_empty());
    // The srce dama: D and A are void in srce, but B cannot tell it from
    // the one hidden discard. Not certain.
    assert!(!about(&from_b, "F3", B, C).contains(court(Suit::Srce, Rank::Dama)));

    // The declarer knows its own discards. For A every unseen tarok is
    // with B (XIV-XX and the škis), and the srce dama is not certain:
    // only D is void in srce besides A itself.
    let from_a = observe(&game, A);
    let mut with_b = taroks(14..=20);
    with_b.insert(Card::SKIS);
    assert_eq!(about(&from_a, "F3", A, B), with_b);
    assert!(about(&from_a, "F3", A, C).is_empty());
    // For D, who is no declarer, the taroks it has not seen are split
    // between A and B: nothing certain. But the kings cannot be discards:
    // none is certain either, since no two seats are void in karo or pik.
    let from_d = observe(&game, D);
    for other in [A, B, C] {
        assert!(about(&from_d, "F3", D, other).is_empty());
    }
}

#[test]
fn c07_f3_a_king_by_elimination_for_a_seat_that_is_no_declarer() {
    // A king cannot be a hidden discard, so for a seat that is not the
    // declarer a king is certain where a plain suit card is not.
    //
    // D plays solo ena. A holds six pik with the kralj, B holds the pik 9
    // and 10, C and D hold none. D takes the srce 4 and puts it down again:
    // one hidden discard. A leads the pik 7: B follows, C trumps, D throws
    // a križ.
    let pik = |n| pip(Suit::Pik, n);
    let a = [
        kralj(Suit::Pik),
        pik(7),
        pik(8),
        court(Suit::Pik, Rank::Fant),
        court(Suit::Pik, Rank::Kaval),
        court(Suit::Pik, Rank::Dama),
    ];
    let srce = |n| pip(Suit::Srce, n);
    let talon = [
        srce(4),
        srce(3),
        srce(2),
        srce(1),
        pip(Suit::Karo, 4),
        pip(Suit::Karo, 3),
    ];
    let deal = deal_with([&a, &[pik(9), pik(10)], &[], &[]], &talon);
    let mut game = game(&deal);
    declare(&mut game, D, Contract::SoloEna);
    exchange(&mut game, 0, &[srce(4)]);
    for (seat, card) in [
        (A, pik(7)),
        (B, pik(9)),
        (C, tarok(17)),
        (D, pip(Suit::Kriz, 7)),
    ] {
        act(&mut game, seat, Action::Play(card));
    }

    // B sees C and D fail to follow pik. The unseen pik are the 8, fant,
    // kaval, dama and kralj. Only the kralj is certainly with A: any of the
    // others may be D's hidden discard.
    let from_b = observe(&game, B);
    assert_eq!(
        about(&from_b, "F3", B, A),
        [kralj(Suit::Pik)].into_iter().collect()
    );
    for other in [C, D] {
        assert!(about(&from_b, "F2", B, other).contains(kralj(Suit::Pik)));
        assert!(about(&from_b, "F2", B, other).contains(pik(8)));
    }
    assert!(!about(&from_b, "F2", B, A).contains(pik(8)));

    // C sees one void only (D): the kralj may be with A or with B.
    let from_c = observe(&game, C);
    for other in [A, B, D] {
        assert!(!about(&from_c, "F3", C, other).contains(kralj(Suit::Pik)));
    }
    // A holds it and knows that.
    assert!(!as_set(features(&observe(&game, A), "F4")).contains(kralj(Suit::Pik)));
}

#[test]
fn c07_f4_unseen_cards() {
    let game = after_three_tricks();
    // B has seen: its hand, the twelve played cards, the six talon cards,
    // the shown discard II.
    let hand_of_b: CardSet = (14..=20)
        .map(tarok)
        .chain([Card::SKIS, court(Suit::Srce, Rank::Fant)])
        .collect();
    let played: CardSet = game
        .public()
        .tricks()
        .iter()
        .flat_map(|trick| trick.cards)
        .collect();
    assert_eq!(played.len(), 12);
    let talon: CardSet = game.public().talon().unwrap().into_iter().collect();
    let seen = hand_of_b | played | talon | [tarok(2)].into_iter().collect();
    // 9 + 12 + 6 + 1, less the srce kralj counted twice.
    assert_eq!(seen.len(), 27);
    let from_b = observe(&game, B);
    assert_eq!(as_set(features(&from_b, "F4")), seen.complement());
    assert_eq!(features(&from_b, "F4"), card_bits(seen.complement().iter()));

    // Written out: I, IV-XI; srce dama; karo fant, kaval, dama, kralj;
    // pik 8, 9, 10, fant, kaval, dama, kralj; križ 9, 10, fant, kaval,
    // dama, kralj.
    let mut unseen = taroks([1, 4, 5, 6, 7, 8, 9, 10, 11]);
    unseen.insert(court(Suit::Srce, Rank::Dama));
    for rank in Rank::ALL {
        unseen.insert(court(Suit::Karo, rank));
        unseen.insert(court(Suit::Pik, rank));
        unseen.insert(court(Suit::Kriz, rank));
    }
    for number in [8, 9, 10] {
        unseen.insert(pip(Suit::Pik, number));
    }
    for number in [9, 10] {
        unseen.insert(pip(Suit::Kriz, number));
    }
    assert_eq!(unseen.len(), 27);
    assert_eq!(as_set(features(&from_b, "F4")), unseen);

    // The declarer has also seen its own hidden discard, the karo 4; it was
    // in the shown talon anyway. A hidden discard out of the hand: the
    // declarer has seen it, the others have not.
    let talon = [
        tarok(10),
        pip(Suit::Karo, 4),
        tarok(11),
        tarok(12),
        tarok(13),
        tarok(14),
    ];
    let deal = deal_with(
        [&[pip(Suit::Srce, 4), pip(Suit::Srce, 3)], &[], &[], &[]],
        &talon,
    );
    let mut solo = common::flow::game(&deal);
    declare(&mut solo, A, Contract::SoloTri);
    exchange(
        &mut solo,
        0,
        &[pip(Suit::Karo, 4), pip(Suit::Srce, 4), pip(Suit::Srce, 3)],
    );
    assert!(!as_set(features(&observe(&solo, A), "F4")).contains(pip(Suit::Srce, 4)));
    assert!(as_set(features(&observe(&solo, B), "F4")).contains(pip(Suit::Srce, 4)));

    // Before the talon is turned up its cards are unseen.
    let fresh = common::flow::game(&deal);
    let unseen = as_set(features(&observe(&fresh, B), "F4"));
    assert_eq!(unseen, fresh.private(B).hand().complement());
    assert_eq!(unseen.len(), 42);
}

#[test]
fn c07_f5_own_card_is_the_highest_unseen_of_its_suit() {
    let game = after_three_tricks();
    // B: XIV-XX, škis, srce fant. The taroks B has not seen are I and
    // IV-XI, all lower: every tarok of B is set. The srce dama is unseen
    // and beats the srce fant: not set.
    let from_b = observe(&game, B);
    let mut expected = taroks(14..=20);
    expected.insert(Card::SKIS);
    assert_eq!(as_set(features(&from_b, "F5")), expected);

    // C: srce dama; karo fant, kaval, dama, kralj; pik 8, 9, 10, fant.
    // No karo is unseen for C (the four pips lay in the talon): all four
    // karo are set. The only unseen srce is B's fant: the dama is set. The
    // pik kaval, dama, kralj are unseen: no pik of C is set.
    let from_c = observe(&game, C);
    let mut expected: CardSet = Rank::ALL
        .into_iter()
        .map(|rank| court(Suit::Karo, rank))
        .collect();
    expected.insert(court(Suit::Srce, Rank::Dama));
    assert_eq!(as_set(features(&from_c, "F5")), expected);

    // A: I, IV-XI against the unseen XIV-XX and škis: none.
    assert!(as_set(features(&observe(&game, A), "F5")).is_empty());
    // Only own cards can be set.
    for observer in Seat::ALL {
        let observation = observe(&game, observer);
        assert!(as_set(features(&observation, "F5")).is_subset(game.private(observer).hand()));
    }
}

#[test]
fn c07_f6_derived_counts() {
    let game = after_three_tricks();
    let from_b = observe(&game, B);
    let f6 = features(&from_b, "F6");
    assert_eq!(f6.len(), 16);
    // Taroks unseen / 22: I and IV-XI, nine.
    assert_eq!(f6[0], 9.0 / 22.0);
    // Unseen per suit / 8: srce 1 (dama); karo 4 (the courts); pik 7 (all
    // but the 7); križ 6 (all but the 7 and the 8).
    assert_eq!(f6[1..5], [1.0 / 8.0, 4.0 / 8.0, 7.0 / 8.0, 6.0 / 8.0]);
    // King unseen: srce no (played); karo, pik, križ yes.
    assert_eq!(f6[5..9], [0.0, 1.0, 1.0, 1.0]);
    // Škis (own hand), mond (shown talon), pagat (unseen).
    assert_eq!(f6[9..12], [0.0, 0.0, 1.0]);
    // Card points unseen / 70. The 27 unseen cards as one pile: taroks
    // 5 + 8 = 13; srce dama 4; karo courts 14; pik 3 + 14 = 17; križ
    // 2 + 14 = 16. Values 64, less 2 per three cards: 64 - 18 = 46.
    assert_eq!(f6[12], 46.0 / 70.0);
    // Most taroks each other seat can still hold / 12: C and D are void;
    // A holds nine cards and nine taroks are possible.
    assert_eq!(f6[13..16], [0.0, 0.0, 9.0 / 12.0]);

    // At the start of a deal: 22 less the own taroks are unseen, and a
    // seat can hold at most twelve.
    let fresh = common::flow::game(&common::flow::king_in_talon_deal(false));
    let from_a = observe(&fresh, A);
    let f6 = features(&from_a, "F6");
    // A holds I-XII: ten taroks are unseen.
    assert_eq!(f6[0], 10.0 / 22.0);
    assert_eq!(f6[1..5], [1.0; 4]);
    assert_eq!(f6[5..9], [1.0; 4]);
    assert_eq!(f6[9..12], [1.0, 1.0, 0.0]);
    assert_eq!(f6[13..16], [10.0 / 12.0; 3]);
}

fn case() -> impl Strategy<Value = (u64, u8, Vec<u8>)> {
    (
        any::<u64>(),
        0..4u8,
        proptest::collection::vec(any::<u8>(), 96),
    )
}

#[test]
fn c08_inference_is_sound_at_every_decision_of_10_000_random_deals() {
    let voids = std::cell::Cell::new(0u64);
    let cannot = std::cell::Cell::new(0u64);
    let certain = std::cell::Cell::new(0u64);
    let taken_king_or_trula = std::cell::Cell::new(0u64);
    let taken_tarok = std::cell::Cell::new(0u64);
    let taken_other = std::cell::Cell::new(0u64);
    let eliminated = std::cell::Cell::new(0u64);
    run_cases(10_000, case(), |(seed, dealer, choices)| {
        play_random(seed, seat(dealer), &choices, |game| {
            for observer in Seat::ALL {
                let observation = observe(game, observer);
                for other in Seat::ALL {
                    if other == observer {
                        continue;
                    }
                    let hand = game.full().hand(other);
                    // F1: a void is a true void.
                    let bits = void_bits(&observation, observer, other);
                    for (index, suit) in common::SUITS.into_iter().enumerate() {
                        if bits[index] == 1.0 {
                            assert!(hand.is_disjoint(CardSet::suit(suit)), "void in {suit:?}");
                            voids.set(voids.get() + 1);
                        }
                    }
                    if bits[TAROKS] == 1.0 {
                        assert!(hand.is_disjoint(CardSet::TAROKS), "void in taroks");
                        voids.set(voids.get() + 1);
                    }
                    // F2: "cannot hold" is true.
                    let cannot_hold = about(&observation, "F2", observer, other);
                    assert!(
                        cannot_hold.is_disjoint(hand),
                        "F2 names {:?}, which seat {} holds",
                        cannot_hold & hand,
                        other.index()
                    );
                    cannot.set(cannot.get() + u64::from(cannot_hold.len()));
                    // F3: "certainly holds" is true.
                    let holds = about(&observation, "F3", observer, other);
                    assert!(
                        holds.is_subset(hand),
                        "F3 names {:?}, which seat {} does not hold",
                        holds - hand,
                        other.index()
                    );
                    certain.set(certain.get() + u64::from(holds.len()));
                    // Which source each bit has, told from the truth: a
                    // card of the packet in the declarer's hand (a king or
                    // trula card, a tarok, any other card), or a card
                    // found by elimination.
                    let from_packet = if game.public().declarer() == Some(other) {
                        holds & game.public().taken_cards()
                    } else {
                        CardSet::EMPTY
                    };
                    for card in from_packet {
                        let counter = if card.value() == 5 {
                            &taken_king_or_trula
                        } else if card.is_tarok() {
                            &taken_tarok
                        } else {
                            &taken_other
                        };
                        counter.set(counter.get() + 1);
                    }
                    eliminated.set(eliminated.get() + u64::from((holds - from_packet).len()));
                    // A card cannot be both.
                    assert!(holds.is_disjoint(cannot_hold));
                }
                // F4: unseen cards are not in the own hand, and every card
                // another seat holds that was never shown is unseen.
                let unseen = as_set(features(&observation, "F4"));
                assert!(unseen.is_disjoint(game.private(observer).hand()));
            }
        });
        Ok(())
    });
    // The bits are in use: the test did check something.
    assert!(voids.get() > 100_000, "{} void bits", voids.get());
    assert!(certain.get() > 100_000, "{} certain bits", certain.get());
    assert!(cannot.get() > certain.get());
    // Every source of "certainly holds" was checked, not one of them only.
    assert!(taken_king_or_trula.get() > 20_000);
    assert!(taken_tarok.get() > 20_000);
    assert!(taken_other.get() > 20_000);
    assert!(eliminated.get() > 20_000);
    println!(
        "certainly-holds bits checked: {} (taken king or trula {}, taken tarok {}, other taken \
         card {}, by elimination {}); void bits {}; cannot-hold bits {}",
        certain.get(),
        taken_king_or_trula.get(),
        taken_tarok.get(),
        taken_other.get(),
        eliminated.get(),
        voids.get(),
        cannot.get()
    );
}
