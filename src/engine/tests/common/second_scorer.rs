//! A second scorer, for the tests only.
//!
//! It is written from spec 005 criteria 29-44 with the answers of D118 and
//! from spec 013 (D125: a valat keeps the mondfang; D126: pagat ultimo
//! follows the side; D133: in a valat the declarer wins the packets not
//! taken) and from spec 009 criteria 18-29 (announced bonuses, the
//! announced valat, kontra), seat by seat. It does not call the engine's
//! `score` module or the pile accessors of the full view. It reads the
//! record of a finished deal: the contract, the declarer, the called king,
//! the talon, the taken packet, the tricks and what was said in the round
//! of announcements from the public view; the partner and the discards
//! from the full view. Card points come from `points::count` (spec 002)
//! and the winner of the last trick from the tests' own
//! `reference_winner`.
//!
//! The tests compare it with `Game::score` on random deals.

use tarok_engine::{Bonus, Card, CardSet, Contract, Game, Seat, Suit, Target, points};

use super::{kralj, reference_winner};

/// The eight parts of one seat's entry, in the order of the spec: game,
/// difference, trula, kralji, kralj ultimo, pagat ultimo, valat, mondfang.
pub type Parts = [i32; 8];

/// What the second scorer finds for a finished deal.
pub struct Second {
    /// The cards that count for the declarer's side.
    pub declarer_pile: CardSet,
    /// The cards that count for the opponents, collected on their own (not
    /// as "the rest").
    pub opponents_pile: CardSet,
    pub declarer_points: u8,
    pub opponents_points: u8,
    /// The parts by seat index.
    pub parts: [Parts; 4],
}

impl Second {
    pub fn entries(&self) -> [i32; 4] {
        self.parts.map(|parts| parts.iter().sum())
    }
}

fn game_value(contract: Contract) -> i32 {
    match contract {
        Contract::Tri => 10,
        Contract::Dve => 20,
        Contract::Ena => 30,
        Contract::SoloTri => 40,
        Contract::SoloDve => 50,
        Contract::SoloEna => 60,
        other => panic!("{other} is not played in this slice"),
    }
}

/// The multiple of 5 nearest to `points - 35`, found by trying them all.
fn nearest_five(points: u8) -> i32 {
    let exact = i32::from(points) - 35;
    (-7..=7)
        .map(|k| 5 * k)
        .min_by_key(|multiple| (multiple - exact).abs())
        .unwrap()
}

/// The seat that played the card at `position` of a trick led by `leader`.
fn seat_at(leader: Seat, position: usize) -> Seat {
    Seat::new(((leader.index() + position) % 4) as u8).unwrap()
}

pub fn second_score(game: &Game) -> Second {
    let public = game.public();
    let full = game.full();
    let contract = public.contract().expect("a contract");
    let declarer = public.declarer().expect("a declarer");
    let partner = full.partner();
    let tricks = public.tricks();
    assert_eq!(tricks.len(), 12, "the deal is over");
    let with_declarer = |seat: Seat| seat == declarer || Some(seat) == partner;

    // --- The piles (criteria 21, 23-25, 29) -----------------------------
    let talon: Vec<Card> = public.talon().expect("the talon was shown").to_vec();
    let taken: Vec<Card> = talon
        .iter()
        .copied()
        .filter(|&card| public.taken_cards().contains(card))
        .collect();
    let untaken: Vec<Card> = talon
        .iter()
        .copied()
        .filter(|card| !taken.contains(card))
        .collect();
    let called_king = if matches!(contract, Contract::Tri | Contract::Dve | Contract::Ena) {
        Some(kralj(public.called_suit().expect("a called suit")))
    } else {
        None
    };
    // The rest of the talon is the declarer's when the called king lay in
    // the packet taken and the declarer won the trick it was played to.
    let rest_won = called_king.is_some_and(|king| {
        taken.contains(&king)
            && tricks
                .iter()
                .any(|trick| trick.cards.contains(&king) && trick.winner == declarer)
    });

    let mut ours: Vec<Card> = full.discards().iter().collect();
    let mut theirs: Vec<Card> = Vec::new();
    let mut our_tricks = 0;
    for trick in tricks {
        if with_declarer(trick.winner) {
            ours.extend(trick.cards);
            our_tricks += 1;
        } else {
            theirs.extend(trick.cards);
        }
    }
    // In a valat the declarer gets the packets not taken (spec 013,
    // criterion 16, D133): the opponents have no trick to put them with.
    // Not in a valat against.
    let rest_won = rest_won || theirs.is_empty();
    if rest_won {
        ours.extend(&untaken);
    } else {
        theirs.extend(&untaken);
    }
    assert_eq!(ours.len() + theirs.len(), 54, "every card is in a pile");
    let declarer_pile: CardSet = ours.iter().copied().collect();
    let opponents_pile: CardSet = theirs.iter().copied().collect();
    assert_eq!(declarer_pile.len() as usize, ours.len(), "no card twice");
    assert_eq!(opponents_pile.len() as usize, theirs.len(), "no card twice");
    let declarer_points = points::count(declarer_pile);
    let opponents_points = points::count(opponents_pile);

    // --- Mondfang (criteria 42-44) --------------------------------------
    // The seat that loses the mond, if any: its player in a trick with the
    // škis, or the declarer when the mond stays in the talon with the
    // opponents.
    let mut mond_loser: Option<Seat> = None;
    for trick in tricks {
        if trick.cards.contains(&Card::SKIS)
            && let Some(position) = trick.cards.iter().position(|&c| c == Card::MOND)
        {
            mond_loser = Some(seat_at(trick.leader, position));
        }
    }
    if untaken.contains(&Card::MOND) && !rest_won {
        mond_loser = Some(declarer);
    }

    // --- What the declarer's side writes ---------------------------------
    // Flow A: spec 005 criteria 30-41. Flow B: spec 009 criteria 18-29,
    // written here a second time, from the criteria.
    //
    // What was said in the round. `owner(bonus)` is +1 if the declarer's
    // side announced the bonus, -1 if an opponent did, and nothing if it
    // was not announced. `times(target)` is 1, 2, 4, 8 or 16.
    let owner = |bonus: Bonus| -> Option<i32> {
        let announcer = public.announced(bonus)?;
        Some(if with_declarer(announcer) { 1 } else { -1 })
    };
    let times =
        |target: Target| -> i32 { [1, 2, 4, 8, 16][usize::from(public.kontra_level(target))] };
    // An announced bonus: `amount` for its owner if `declarers_have_it`
    // says so for the owner's side, else against the owner; times its own
    // kontra level. From the declarer's side: plus if the owner is that
    // side and wins, or the owner is the other side and fails.
    let announced = |sign: i32, owner_wins: bool, amount: i32, target: Target| -> i32 {
        let for_the_owner = if owner_wins { amount } else { -amount };
        sign * for_the_owner * times(target)
    };

    let mut side: Parts = [0; 8];
    if let Some(sign) = owner(Bonus::Valat) {
        // Criterion 23: 500 for the owner with all twelve tricks, else 500
        // against it; nothing else is scored. Criterion 29: only the
        // kontra on the valat itself counts.
        let owner_tricks = if sign == 1 {
            our_tricks
        } else {
            12 - our_tricks
        };
        side[6] = announced(sign, owner_tricks == 12, 500, Target::Valat);
    } else if our_tricks == 12 || our_tricks == 0 {
        // Spec 005 criterion 41 and answer A: 250 for or against, and none
        // of the parts of the side, the announced ones neither (spec 009,
        // criterion 24). Never multiplied (criteria 28, 29). The mondfang
        // is not one of the parts: it is written below in every deal
        // (spec 013, D125).
        side[6] = if our_tricks == 12 { 250 } else { -250 };
    } else {
        // Game and difference: 36 wins, 35 loses. A kontra on the game
        // multiplies both and nothing else (criterion 26).
        let value = game_value(contract);
        side[0] = (if declarer_points >= 36 { value } else { -value }) * times(Target::Game);
        side[1] = nearest_five(declarer_points) * times(Target::Game);

        // Trula: all three with one side. Kralji: all four with one side.
        let trula = [Card::SKIS, Card::MOND, Card::PAGAT];
        let ours_of_trula = trula.iter().filter(|c| ours.contains(c)).count();
        side[2] = match owner(Bonus::Trula) {
            // Announced (criteria 18, 20): 20 for the owner if its side
            // has all three, else 20 against it. Every card is in one of
            // the two piles, so "none with us" is "all with them".
            Some(sign) => {
                let owners = if sign == 1 {
                    ours_of_trula
                } else {
                    3 - ours_of_trula
                };
                announced(sign, owners == 3, 20, Target::Trula)
            }
            None => match ours_of_trula {
                3 => 10,
                0 => -10,
                _ => 0,
            },
        };
        let kings = [Suit::Srce, Suit::Karo, Suit::Pik, Suit::Kriz].map(kralj);
        let ours_of_kings = kings.iter().filter(|c| ours.contains(c)).count();
        side[3] = match owner(Bonus::Kralji) {
            Some(sign) => {
                let owners = if sign == 1 {
                    ours_of_kings
                } else {
                    4 - ours_of_kings
                };
                announced(sign, owners == 4, 20, Target::Kralji)
            }
            None => match ours_of_kings {
                4 => 10,
                0 => -10,
                _ => 0,
            },
        };

        let last = &tricks[11];
        let taker = seat_at(last.leader, reference_winner(last.cards));
        // Kralj ultimo: the called king in the last trick; the side that
        // wins the trick decides the sign (spec 005 criterion 38).
        let king_in_last = called_king.is_some_and(|king| last.cards.contains(&king));
        side[4] = match owner(Bonus::KraljUltimo) {
            // Announced (criterion 21): +20 when the king is played to the
            // last trick and the declarer's side takes it, -20 in every
            // other case. Only the holder of the called king announces it,
            // so the owner is the declarer's side.
            Some(sign) => {
                assert_eq!(sign, 1, "kralj ultimo is the declarer's side's");
                announced(
                    sign,
                    king_in_last && with_declarer(taker),
                    20,
                    Target::KraljUltimo,
                )
            }
            None if king_in_last => {
                if with_declarer(last.winner) {
                    10
                } else {
                    -10
                }
            }
            None => 0,
        };
        // Pagat ultimo: the pagat in the last trick. The side of the
        // pagat's player wins the bonus when one of its seats takes the
        // trick, with the pagat or with another card (spec 013, D126). The
        // seat that takes the trick is found here from the cards.
        let pagat_at = last.cards.iter().position(|&c| c == Card::PAGAT);
        side[5] = match (owner(Bonus::PagatUltimo), pagat_at) {
            // Announced (criterion 22): 50 for the owner when the pagat is
            // played to the last trick and the owner's side takes that
            // trick; 50 against the owner in every other case.
            (Some(sign), Some(_)) => {
                let owners_take_it = with_declarer(taker) == (sign == 1);
                announced(sign, owners_take_it, 50, Target::PagatUltimo)
            }
            (Some(sign), None) => announced(sign, false, 50, Target::PagatUltimo),
            (None, Some(position)) => {
                let own_pagat = with_declarer(seat_at(last.leader, position));
                match (own_pagat, with_declarer(taker)) {
                    // The declarer's side played it and took the trick.
                    (true, true) => 25,
                    // The declarer's side played it and lost the trick.
                    (true, false) => -25,
                    // An opponent played it and the declarer's side took it.
                    (false, true) => 25,
                    // An opponent played it and the opponents took the trick.
                    (false, false) => -25,
                }
            }
            (None, None) => 0,
        };
    }

    // --- One entry per seat (criteria 35, 42, 43) ------------------------
    let mut parts: [Parts; 4] = [[0; 8]; 4];
    for seat in Seat::ALL {
        if with_declarer(seat) {
            parts[seat.index()] = side;
        }
        if mond_loser == Some(seat) {
            parts[seat.index()][7] = -20;
        }
    }
    Second {
        declarer_pile,
        opponents_pile,
        declarer_points,
        opponents_points,
        parts,
    }
}

/// The engine's parts of a seat in the same order as [`Parts`].
pub fn engine_parts(game: &Game, seat: Seat) -> Parts {
    let score = game.score().expect("the deal is over");
    let p = score.parts(seat);
    [
        p.game,
        p.difference,
        p.trula,
        p.kralji,
        p.kralj_ultimo,
        p.pagat_ultimo,
        p.valat,
        p.mondfang,
    ]
}

/// Compares `Game::score` with the second scorer: both piles, both counts
/// and every part of every seat. Returns the second scorer's result.
pub fn assert_scores_agree(game: &Game) -> Second {
    let second = second_score(game);
    let score = game.score().expect("the deal is over");
    assert_eq!(game.full().declarer_pile(), second.declarer_pile);
    assert_eq!(game.full().opponents_pile(), second.opponents_pile);
    assert_eq!(score.declarer_points(), second.declarer_points);
    assert_eq!(score.opponents_points(), second.opponents_points);
    assert_eq!(second.declarer_points + second.opponents_points, 70);
    for seat in Seat::ALL {
        assert_eq!(
            engine_parts(game, seat),
            second.parts[seat.index()],
            "parts of seat {} (game, difference, trula, kralji, kralj ultimo, pagat ultimo, \
             valat, mondfang)",
            seat.index()
        );
    }
    assert_eq!(score.entries(), second.entries());
    second
}
