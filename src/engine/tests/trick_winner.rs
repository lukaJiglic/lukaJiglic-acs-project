//! Spec 002, criteria 11-19: the trick winner in normal contracts. Criterion
//! 17 is the one of spec 013 (criteria 8 and 9): the trula in one trick.
//!
//! `trick::winner` takes the four cards in the order of play and returns the
//! position (0-3) of the winning card. Position 0 is the led card.

mod common;

use common::{
    SUITS, built_hands, court, kralj, orders, pip, reference_winner, run_cases, seat, suit_cards,
    suit_high_to_low, tarok, taroks_low_to_high,
};
use proptest::prelude::*;
use tarok_engine::{Card, Deal, PlayState, Rank, Suit, trick};

/// The card that wins the trick.
fn winning_card(cards: [Card; 4]) -> Card {
    cards[trick::winner(&cards)]
}

/// Every choice of `k` items, as index lists.
fn subsets(n: usize, k: usize) -> Vec<Vec<usize>> {
    fn go(start: usize, n: usize, k: usize, cur: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        if cur.len() == k {
            out.push(cur.clone());
            return;
        }
        for i in start..n {
            cur.push(i);
            go(i + 1, n, k, cur, out);
            cur.pop();
        }
    }
    let mut out = Vec::new();
    go(0, n, k, &mut Vec::new(), &mut out);
    out
}

#[test]
fn c11_suit_led_all_follow_highest_of_led_suit_wins() {
    // Worked examples.
    let srce = Suit::Srce;
    let example = [
        pip(srce, 4),
        court(srce, Rank::Kralj),
        pip(srce, 2),
        court(srce, Rank::Fant),
    ];
    assert_eq!(trick::winner(&example), 1);
    let example = [
        pip(Suit::Pik, 10),
        pip(Suit::Pik, 7),
        pip(Suit::Pik, 9),
        pip(Suit::Pik, 8),
    ];
    assert_eq!(trick::winner(&example), 0);

    // Every four cards of one suit, in every order of play.
    for suit in SUITS {
        let ranked = suit_high_to_low(suit);
        for chosen in subsets(8, 4) {
            // `chosen` is ascending, so its first entry is the strongest card.
            let highest = ranked[chosen[0]];
            let cards = [
                ranked[chosen[0]],
                ranked[chosen[1]],
                ranked[chosen[2]],
                ranked[chosen[3]],
            ];
            for order in orders(cards) {
                assert_eq!(winning_card(order), highest, "trick {order:?}");
            }
        }
    }
}

#[test]
fn c12_court_order_kralj_dama_kaval_fant_then_pips() {
    for suit in SUITS {
        let ranked = suit_high_to_low(suit);
        let (kralj, dama, kaval, fant) = (ranked[0], ranked[1], ranked[2], ranked[3]);
        let pips = &ranked[4..];

        // All four courts in one trick: the kralj wins.
        for order in orders([kralj, dama, kaval, fant]) {
            assert_eq!(winning_card(order), kralj, "trick {order:?}");
        }
        // Each court beats the next one down; two pips fill the trick.
        for (higher, lower) in [(kralj, dama), (dama, kaval), (kaval, fant)] {
            for order in orders([higher, lower, pips[0], pips[3]]) {
                assert_eq!(winning_card(order), higher, "trick {order:?}");
            }
        }
        // The fant beats every pip.
        for left_out in 0..4 {
            let three: Vec<Card> = (0..4).filter(|&i| i != left_out).map(|i| pips[i]).collect();
            for order in orders([fant, three[0], three[1], three[2]]) {
                assert_eq!(winning_card(order), fant, "trick {order:?}");
            }
        }
    }
}

#[test]
fn c13_red_pips_1_high_black_pips_10_high() {
    for (suit, high_to_low) in [
        (Suit::Srce, [1u8, 2, 3, 4]),
        (Suit::Karo, [1, 2, 3, 4]),
        (Suit::Pik, [10, 9, 8, 7]),
        (Suit::Kriz, [10, 9, 8, 7]),
    ] {
        let pips = high_to_low.map(|number| pip(suit, number));

        // All four pips: the first of the order wins.
        for order in orders(pips) {
            assert_eq!(winning_card(order), pips[0], "trick {order:?}");
        }
        // Each neighbouring pair, with the pip led. The other two cards are
        // of another suit and cannot win.
        let other = if suit == Suit::Srce {
            Suit::Pik
        } else {
            Suit::Srce
        };
        let fillers = [kralj(other), court(other, Rank::Dama)];
        for pair in pips.windows(2) {
            let (higher, lower) = (pair[0], pair[1]);
            for order in orders([higher, lower, fillers[0], fillers[1]]) {
                if order[0] == higher || order[0] == lower {
                    assert_eq!(winning_card(order), higher, "trick {order:?}");
                }
            }
        }
    }
}

#[test]
fn c14_card_of_another_suit_never_wins() {
    // The lowest srce is led; three kings of other suits do not beat it.
    let example = [
        pip(Suit::Srce, 4),
        kralj(Suit::Karo),
        kralj(Suit::Pik),
        kralj(Suit::Kriz),
    ];
    assert_eq!(trick::winner(&example), 0);
    // One player follows suit and wins; the kings still do not.
    let example = [
        pip(Suit::Srce, 4),
        kralj(Suit::Pik),
        pip(Suit::Srce, 3),
        kralj(Suit::Karo),
    ];
    assert_eq!(trick::winner(&example), 2);

    // Every led suit card against every three suit cards of other suits.
    let all = suit_cards();
    for &led in &all {
        let others: Vec<Card> = all
            .iter()
            .copied()
            .filter(|c| c.suit() != led.suit())
            .collect();
        assert_eq!(others.len(), 24);
        for chosen in subsets(24, 3) {
            let three = [others[chosen[0]], others[chosen[1]], others[chosen[2]]];
            // The three other cards in each of their six orders.
            for [a, b, c] in [
                [0, 1, 2],
                [0, 2, 1],
                [1, 0, 2],
                [1, 2, 0],
                [2, 0, 1],
                [2, 1, 0],
            ] {
                let cards = [led, three[a], three[b], three[c]];
                assert_eq!(trick::winner(&cards), 0, "trick {cards:?}");
            }
        }
    }
}

#[test]
fn c15_any_tarok_beats_any_suit_card() {
    // Pagat against three kings, the kralj led.
    let example = [
        kralj(Suit::Srce),
        kralj(Suit::Karo),
        Card::PAGAT,
        kralj(Suit::Pik),
    ];
    assert_eq!(trick::winner(&example), 2);

    for tarok in taroks_low_to_high() {
        for suit in SUITS {
            let ranked = suit_high_to_low(suit);
            let other = if suit == Suit::Kriz {
                Suit::Srce
            } else {
                Suit::Kriz
            };
            for (i, &card) in ranked.iter().enumerate() {
                // One tarok, the suit card, one more card of that suit and
                // the kralj of another suit, in every order of play.
                let same_suit = ranked[(i + 1) % 8];
                for order in orders([tarok, card, same_suit, kralj(other)]) {
                    assert_eq!(winning_card(order), tarok, "trick {order:?}");
                }
            }
        }
    }
}

#[test]
fn c16_higher_tarok_beats_lower_and_skis_beats_mond() {
    let fillers = [kralj(Suit::Srce), kralj(Suit::Pik)];

    // Škis beats mond.
    for order in orders([Card::SKIS, Card::MOND, fillers[0], fillers[1]]) {
        assert_eq!(winning_card(order), Card::SKIS, "trick {order:?}");
    }
    for order in orders([Card::SKIS, Card::MOND, tarok(20), tarok(2)]) {
        assert_eq!(winning_card(order), Card::SKIS, "trick {order:?}");
    }

    // Every pair of taroks.
    let taroks = taroks_low_to_high();
    for (i, &lower) in taroks.iter().enumerate() {
        for &higher in &taroks[i + 1..] {
            for order in orders([lower, higher, fillers[0], fillers[1]]) {
                assert_eq!(winning_card(order), higher, "trick {order:?}");
            }
        }
    }

    // Every four taroks, except when škis, mond and pagat are all there
    // (criterion 17).
    for chosen in subsets(22, 4) {
        let cards = [
            taroks[chosen[0]],
            taroks[chosen[1]],
            taroks[chosen[2]],
            taroks[chosen[3]],
        ];
        let trula = [Card::SKIS, Card::MOND, Card::PAGAT]
            .iter()
            .all(|c| cards.contains(c));
        if trula {
            continue;
        }
        let highest = cards[3];
        for order in orders(cards) {
            assert_eq!(winning_card(order), highest, "trick {order:?}");
        }
    }
}

/// Spec 013, criteria 8 and 9 (D131, D132). This replaces criterion 17 of
/// spec 002, where the pagat won in any order of play.
#[test]
fn c17_trula_in_one_trick_the_pagat_wins_only_when_it_falls_last() {
    let (s, m, p) = (Card::SKIS, Card::MOND, Card::PAGAT);

    // The 24 orders of play, written out by hand, with the card that wins.
    // `x` is the fourth card. The pagat wins in the first eight rows: it is
    // played after both the škis and the mond. In rows 4 and 8 a fourth
    // card still follows it (reading C12, decided in D132).
    let table = |x: Card| {
        [
            ([x, s, m, p], p),
            ([s, x, m, p], p),
            ([s, m, x, p], p),
            ([s, m, p, x], p),
            ([x, m, s, p], p),
            ([m, x, s, p], p),
            ([m, s, x, p], p),
            ([m, s, p, x], p),
            // The pagat between the two: the škis wins.
            ([x, s, p, m], s),
            ([s, x, p, m], s),
            ([s, p, x, m], s),
            ([s, p, m, x], s),
            ([x, m, p, s], s),
            ([m, x, p, s], s),
            ([m, p, x, s], s),
            ([m, p, s, x], s),
            // The pagat before both: the škis wins.
            ([x, p, s, m], s),
            ([p, x, s, m], s),
            ([p, s, x, m], s),
            ([p, s, m, x], s),
            ([x, p, m, s], s),
            ([p, x, m, s], s),
            ([p, m, x, s], s),
            ([p, m, s, x], s),
        ]
    };
    // The fourth card a suit card, led or not, and another tarok.
    for x in [kralj(Suit::Srce), pip(Suit::Pik, 7), tarok(20), tarok(2)] {
        let rows = table(x);
        let mut seen: Vec<[Card; 4]> = rows.iter().map(|(order, _)| *order).collect();
        seen.sort();
        seen.dedup();
        assert_eq!(seen.len(), 24, "the table has every order once");
        for (order, wins) in rows {
            assert_eq!(winning_card(order), wins, "trick {order:?}");
        }
    }

    // Every fourth card (all 51 others), every order of play (24). The
    // rule is written here by position, apart from the table above.
    let mut pagat_at = [0u32; 4];
    let mut skis_at = [0u32; 4];
    let mut fourth_was_suit = 0;
    let mut fourth_was_tarok = 0;
    for fourth in Card::all() {
        if [s, m, p].contains(&fourth) {
            continue;
        }
        if fourth.is_tarok() {
            fourth_was_tarok += 1;
        } else {
            fourth_was_suit += 1;
        }
        for order in orders([s, m, p, fourth]) {
            let at = |card: Card| order.iter().position(|&c| c == card).unwrap();
            let pagat_falls_last = at(p) > at(s) && at(p) > at(m);
            let winner = trick::winner(&order);
            if pagat_falls_last {
                assert_eq!(order[winner], p, "trick {order:?}");
                pagat_at[winner] += 1;
            } else {
                assert_eq!(order[winner], s, "trick {order:?}");
                skis_at[winner] += 1;
            }
        }
    }
    assert_eq!((fourth_was_suit, fourth_was_tarok), (32, 19));
    // The pagat wins 8 of the 24 orders: never as the first or second
    // card, twice as the third (a fourth card follows), six times as the
    // fourth. The škis wins the other 16; counted in the table above, it
    // is the first card in three of them, the second in three, the third
    // in four and the fourth in six.
    assert_eq!(pagat_at, [0, 0, 51 * 2, 51 * 6]);
    assert_eq!(skis_at, [51 * 3, 51 * 3, 51 * 4, 51 * 6]);
}

#[test]
fn c18_pagat_with_only_one_of_skis_and_mond_does_not_win() {
    let (skis, mond, pagat) = (Card::SKIS, Card::MOND, Card::PAGAT);

    assert_eq!(
        trick::winner(&[pagat, skis, tarok(20), kralj(Suit::Srce)]),
        1
    );
    assert_eq!(
        trick::winner(&[mond, kralj(Suit::Srce), pagat, tarok(2)]),
        0
    );

    let others: Vec<Card> = Card::all()
        .filter(|c| ![skis, mond, pagat].contains(c))
        .collect();
    assert_eq!(others.len(), 51);
    for big in [skis, mond] {
        for chosen in subsets(51, 2) {
            let cards = [pagat, big, others[chosen[0]], others[chosen[1]]];
            for order in orders(cards) {
                // Škis and mond are the two highest taroks, so the one in
                // the trick wins it.
                assert_eq!(winning_card(order), big, "trick {order:?}");
            }
        }
    }
}

#[test]
fn c19_winner_of_a_trick_leads_the_next() {
    // A hand-built deal: see `common::built_hands`.
    let mut state = PlayState::new(built_hands(), seat(0)).expect("valid hands");
    assert_eq!(state.to_move(), seat(0));
    let k = Suit::Kriz;

    // Trick 1: srce 4, srce fant, pagat, tarok V. Seat 3 wins with the V.
    // Trick 2: seat 3 leads VI; karo 4, pik 7, mond. Seat 2 wins.
    // Trick 3: seat 2 leads križ kralj; seat 3 trumps with VII. Seat 3 wins.
    let script: [([Card; 4], u8, u8); 3] = [
        (
            [
                pip(Suit::Srce, 4),
                court(Suit::Srce, Rank::Fant),
                tarok(1),
                tarok(5),
            ],
            0,
            3,
        ),
        (
            [tarok(6), pip(Suit::Karo, 4), pip(Suit::Pik, 7), tarok(21)],
            3,
            2,
        ),
        (
            [kralj(k), tarok(7), pip(Suit::Karo, 3), pip(Suit::Pik, 8)],
            2,
            3,
        ),
    ];
    for (number, (cards, leader, winner)) in script.into_iter().enumerate() {
        assert_eq!(
            state.leader(),
            seat(leader),
            "leader of trick {}",
            number + 1
        );
        assert_eq!(state.to_move(), seat(leader));
        for card in cards {
            state.play(card).expect("the scripted card is legal");
        }
        let done = state.trick_history()[number];
        assert_eq!(done.leader, seat(leader));
        assert_eq!(done.cards, cards);
        assert_eq!(done.winner, seat(winner), "winner of trick {}", number + 1);
        // The winner is on the lead and may play any card.
        assert_eq!(state.leader(), seat(winner));
        assert_eq!(state.to_move(), seat(winner));
        assert!(state.current_trick().is_empty());
        assert_eq!(state.legal_cards(), state.hand(seat(winner)));
    }

    // Dealt hands: in every trick of 200 deals the winner leads the next one.
    for seed in 0..200u64 {
        let deal = Deal::from_seed(seed);
        let first_leader = seat((seed % 4) as u8);
        let mut state = PlayState::from_deal(&deal, first_leader);
        let mut step = seed;
        while !state.is_over() {
            let legal = state.legal_cards();
            step = step
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let pick = (step >> 33) as u32 % legal.len();
            state.play(legal.nth(pick).unwrap()).unwrap();
        }
        let history = state.trick_history();
        assert_eq!(history.len(), 12);
        assert_eq!(history[0].leader, first_leader);
        // The expected winner comes from the tests' own `reference_winner`
        // and from seat numbers added by hand, not from the engine.
        let mut expected_leader = first_leader;
        for (i, done) in history.iter().enumerate() {
            assert_eq!(done.leader, expected_leader, "seed {seed}, trick {}", i + 1);
            let position = reference_winner(done.cards);
            let expected_winner = seat(((done.leader.index() + position) % 4) as u8);
            assert_eq!(done.winner, expected_winner, "seed {seed}, trick {}", i + 1);
            expected_leader = expected_winner;
        }
        // After the last trick its winner is the seat "to move".
        assert_eq!(state.leader(), expected_leader, "seed {seed}");
    }
}

/// Not a numbered criterion: `trick::winner` against the tests' own
/// `reference_winner` on random tricks of four different cards.
///
/// Four random cards hold the whole trula about once in 6,000 tricks, so a
/// quarter of the cases are built with it: škis, mond, pagat and one more
/// card, in a random order of play. The cases are the same in every run
/// (`run_cases` has a fixed seed), and the test counts what it saw.
#[test]
fn winner_matches_the_reference_on_random_tricks() {
    use std::cell::Cell;

    let trula = [Card::SKIS, Card::MOND, Card::PAGAT];
    let four_cards = proptest::sample::subsequence((0..54u8).collect::<Vec<u8>>(), 4)
        .prop_shuffle()
        .prop_map(|indices| {
            let card = |i: usize| Card::from_index(indices[i]).unwrap();
            [card(0), card(1), card(2), card(3)]
        });
    // `kind` 0 turns the four cards into a trula trick: the first of them
    // that is no trula card stays as the fourth card.
    let tricks = (four_cards, 0..4u8, 0..24usize).prop_map(move |(cards, kind, order)| {
        if kind != 0 {
            return cards;
        }
        let fourth = cards
            .into_iter()
            .find(|card| !trula.contains(card))
            .expect("four cards hold at most three trula cards");
        orders([trula[0], trula[1], trula[2], fourth])[order]
    });

    let (with_trula, pagat_won, skis_won) = (Cell::new(0u32), Cell::new(0u32), Cell::new(0u32));
    let (tarok_won, suit_won) = (Cell::new(0u32), Cell::new(0u32));
    let count = |cell: &Cell<u32>| cell.set(cell.get() + 1);
    run_cases(10_000, tricks, |cards| {
        let winner = trick::winner(&cards);
        prop_assert_eq!(winner, reference_winner(cards), "trick {:?}", cards);
        if trula.iter().all(|card| cards.contains(card)) {
            count(&with_trula);
            if cards[winner] == Card::PAGAT {
                count(&pagat_won);
            } else {
                count(&skis_won);
            }
        } else if cards[winner].is_tarok() {
            count(&tarok_won);
        } else {
            count(&suit_won);
        }
        Ok(())
    });
    println!(
        "random tricks: {} with the trula (pagat won {}, škis won {}); others: {} won by a tarok, \
         {} by a suit card",
        with_trula.get(),
        pagat_won.get(),
        skis_won.get(),
        tarok_won.get(),
        suit_won.get()
    );
    // A quarter of 10,000 cases, and the pagat wins a third of the orders.
    assert!(
        with_trula.get() >= 2_000,
        "{} trula tricks",
        with_trula.get()
    );
    assert!(pagat_won.get() >= 500, "the pagat won {}", pagat_won.get());
    assert!(skis_won.get() >= 1_000, "the škis won {}", skis_won.get());
    assert_eq!(pagat_won.get() + skis_won.get(), with_trula.get());
    assert!(tarok_won.get() >= 2_000 && suit_won.get() >= 500);
}
