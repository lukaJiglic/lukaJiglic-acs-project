//! Scoring of the six normal contracts.
//!
//! Rules source: spec 005 criteria 29-44 with the answers of D118, written
//! from pagat.com's four-player rules, and research/01 B3. Two points are
//! the rulings of the user's table (spec 013): a valat does not cancel a
//! lost mond (D125), and pagat ultimo follows the side that takes the last
//! trick (D126).
//!
//! Announced bonuses, the announced valat and kontra (spec 009, criteria
//! 18-29, with the answers of D141):
//!
//! - An announced bonus replaces the silent one of its kind. It is won or
//!   lost for the side of the seat that announced it; the declarer's side
//!   writes the amount with its sign, the opponents write 0 (S20 a).
//! - Trula and kralji 20, kralj ultimo 20, pagat ultimo 50, valat 500.
//! - A valat, announced or silent, removes every other score of the deal
//!   but a lost mond.
//! - A kontra multiplies its own target by 2, 4, 8 or 16: game and
//!   difference together, or one announced bonus. Silent bonuses, the
//!   silent valat and the mondfang are never multiplied (S21 a).
//!
//! The score of a deal is one entry per seat. An entry is the sum of eight
//! parts, kept apart because the value head of the network predicts them
//! separately (D15). A multiplied part holds the multiplied amount.

use crate::announce::{self, Bonus, Round, Target};
use crate::card::Card;
use crate::cardset::CardSet;
use crate::contract::Contract;
use crate::deal::Seat;
use crate::play::Trick;
use crate::points;
use crate::talon::{KINGS, TRULA};

/// Silent trula and silent kralji.
pub const SILENT_TRULA: i32 = 10;
pub const SILENT_KRALJI: i32 = 10;
/// Silent kralj ultimo.
pub const SILENT_KRALJ_ULTIMO: i32 = 10;
/// Silent pagat ultimo.
pub const SILENT_PAGAT_ULTIMO: i32 = 25;
/// Silent valat: all twelve tricks.
pub const SILENT_VALAT: i32 = 250;
/// The penalty for losing the mond to the škis (S29 a).
pub const MONDFANG: i32 = -20;
/// Announced trula and announced kralji.
pub const ANNOUNCED_TRULA: i32 = 20;
pub const ANNOUNCED_KRALJI: i32 = 20;
/// Announced kralj ultimo.
pub const ANNOUNCED_KRALJ_ULTIMO: i32 = 20;
/// Announced pagat ultimo.
pub const ANNOUNCED_PAGAT_ULTIMO: i32 = 50;
/// Announced valat.
pub const ANNOUNCED_VALAT: i32 = 500;

/// The points the declarer's side needs to win: more than half of 70.
pub const POINTS_TO_WIN: u8 = 36;

/// The parts of one seat's entry. All are 0 for an opponent, except the
/// mondfang.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ScoreParts {
    /// Plus the game value for a win, minus it for a loss. Times 2, 4, 8
    /// or 16 under a kontra on the game.
    pub game: i32,
    /// Card points minus 35, rounded to the nearest 5. Negative in a loss.
    /// Multiplied with the game.
    pub difference: i32,
    /// The four bonuses: silent, or announced and then times its own
    /// kontra.
    pub trula: i32,
    pub kralji: i32,
    pub kralj_ultimo: i32,
    pub pagat_ultimo: i32,
    /// +250 or -250 when one side took all twelve tricks and no valat was
    /// announced. +500 or -500 for an announced valat, times its own
    /// kontra. Every other part is then 0, except the mondfang.
    pub valat: i32,
    /// -20 for the player who lost the mond. Each player's own. Written in
    /// every deal, also next to a valat (D125). A mond in a packet not
    /// taken is not lost when the declarer's side takes all twelve tricks
    /// (D133).
    pub mondfang: i32,
}

impl ScoreParts {
    /// The entry: the sum of the parts.
    pub const fn entry(&self) -> i32 {
        self.game
            + self.difference
            + self.trula
            + self.kralji
            + self.kralj_ultimo
            + self.pagat_ultimo
            + self.valat
            + self.mondfang
    }
}

/// The score of a finished deal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Score {
    parts: [ScoreParts; 4],
    declarer_points: u8,
    opponents_points: u8,
}

impl Score {
    /// The parts of a seat's entry.
    pub fn parts(&self, seat: Seat) -> &ScoreParts {
        &self.parts[seat.index()]
    }

    /// The entry a seat writes.
    pub fn entry(&self, seat: Seat) -> i32 {
        self.parts[seat.index()].entry()
    }

    /// The four entries by seat index.
    pub fn entries(&self) -> [i32; 4] {
        self.parts.map(|parts| parts.entry())
    }

    /// Card points of the declarer's side, counted as one pile.
    pub fn declarer_points(&self) -> u8 {
        self.declarer_points
    }

    /// Card points of the opponents, counted as one pile.
    pub fn opponents_points(&self) -> u8 {
        self.opponents_points
    }

    /// True if the declarer's side has 36 points or more.
    pub fn declarer_won(&self) -> bool {
        self.declarer_points >= POINTS_TO_WIN
    }
}

/// Card points minus 35, rounded to the nearest 5 (S25 a). No count lies
/// half way between two multiples of 5, so there are no ties.
pub const fn rounded_difference(points: u8) -> i32 {
    let exact = points as i32 - 35;
    // Add 2 and round down to a multiple of 5: for -4 that is -5, for 3
    // it is 5, for 2 it is 0.
    (exact + 2).div_euclid(5) * 5
}

/// The game part and the difference part for a normal contract and the
/// card points of the declarer's side.
///
/// - 36 or more is a win: plus the game value, plus the rounded difference.
/// - 35 or fewer is a loss: minus the game value, and the rounded
///   difference, which is then 0 or negative.
///
/// `None` for a contract that is not normal.
pub const fn game_and_difference(contract: Contract, points: u8) -> Option<(i32, i32)> {
    let Some(value) = contract.game_value() else {
        return None;
    };
    let game = if points >= POINTS_TO_WIN {
        value
    } else {
        -value
    };
    Some((game, rounded_difference(points)))
}

/// What the scoring needs to know about a finished deal.
pub(crate) struct Outcome<'a> {
    pub contract: Contract,
    pub declarer: Seat,
    /// The partner, if the declarer has one.
    pub partner: Option<Seat>,
    /// The called king, if a king was called.
    pub called_king: Option<Card>,
    pub tricks: &'a [Trick],
    /// All cards that count for the declarer's side.
    pub declarer_pile: CardSet,
    /// The talon cards the declarer did not take, if they stayed with the
    /// opponents. Empty if the declarer won them: with the called king
    /// (spec 005 criterion 23) or by taking all twelve tricks (D133).
    pub talon_with_opponents: CardSet,
    /// What was said in the round of announcements.
    pub round: &'a Round,
}

/// The seat that played `card` in a trick it is in.
fn player_of(trick: &Trick, card: Card) -> Option<Seat> {
    let position = trick.cards.iter().position(|&c| c == card)?;
    Some(trick.leader.plus(position))
}

pub(crate) fn score(outcome: &Outcome<'_>) -> Score {
    let declarer = outcome.declarer;
    let on_declarer_side = |seat: Seat| seat == declarer || Some(seat) == outcome.partner;
    let declarer_pile = outcome.declarer_pile;
    let opponents_pile = declarer_pile.complement();
    let declarer_points = points::count(declarer_pile);
    let opponents_points = points::count(opponents_pile);

    // What the declarer's side writes. The partner writes the same (S26 a).
    let mut side = ScoreParts::default();
    let mut parts = [ScoreParts::default(); 4];

    let declarer_tricks = outcome
        .tricks
        .iter()
        .filter(|trick| on_declarer_side(trick.winner))
        .count();
    let all_tricks = declarer_tricks == outcome.tricks.len();
    let no_trick = declarer_tricks == 0;

    // The round of announcements. `owner(bonus)` is true if the declarer's
    // side announced the bonus, false if an opponent did, and nothing if
    // nobody did.
    let round = outcome.round;
    let owner = |bonus: Bonus| round.announced(bonus).map(on_declarer_side);
    // What the declarer's side writes for an announced bonus: `amount` for
    // the owner if the owner made it, `amount` against the owner if not,
    // times the kontra level of the bonus (criterion 27).
    let announced = |declarers_own_it: bool, made: bool, amount: i32, target: Target| {
        let for_the_owner = if made { amount } else { -amount };
        let for_the_declarers = if declarers_own_it {
            for_the_owner
        } else {
            -for_the_owner
        };
        for_the_declarers * announce::multiplier(round.level(target))
    };

    if let Some(declarers_own_it) = owner(Bonus::Valat) {
        // An announced valat (criterion 23; D141, B13-5 a): 500 for the
        // owner with all twelve tricks, 500 against it with fewer. In both
        // cases no game, no difference and no other bonus, announced or
        // silent. Only the kontra on the valat itself counts (criterion
        // 29). The mondfang is not cancelled: see below.
        let made = if declarers_own_it {
            all_tricks
        } else {
            no_trick
        };
        side.valat = announced(declarers_own_it, made, ANNOUNCED_VALAT, Target::Valat);
    } else if all_tricks || no_trick {
        // All twelve tricks to one side, not announced: 250 for or
        // against, and no game, no difference and no other bonus (spec 005
        // criterion 41, answer A of D118), the announced ones neither
        // (spec 009, criterion 24). Never multiplied (S21 a).
        side.valat = if all_tricks {
            SILENT_VALAT
        } else {
            -SILENT_VALAT
        };
    } else {
        // A kontra on the game multiplies the game value and the rounded
        // difference, and nothing else (criterion 26).
        if let Some((game, difference)) = game_and_difference(outcome.contract, declarer_points) {
            let times = announce::multiplier(round.level(Target::Game));
            side.game = game * times;
            side.difference = difference * times;
        }

        // Trula and kralji: all cards in one side's pile. The talon the
        // declarer did not take is in the opponents' pile (answer B).
        // Silent: 10 for the side that has them. Announced (criteria 18 to
        // 20): 20 for the owner if its side has them, else 20 against the
        // owner; the silent bonus of that kind is not scored next to it.
        let in_piles = |cards: CardSet, bonus: Bonus, silent: i32, loud: i32| match owner(bonus) {
            Some(declarers_own_it) => {
                let pile = if declarers_own_it {
                    declarer_pile
                } else {
                    opponents_pile
                };
                announced(
                    declarers_own_it,
                    cards.is_subset(pile),
                    loud,
                    Target::of(bonus),
                )
            }
            None if cards.is_subset(declarer_pile) => silent,
            None if cards.is_subset(opponents_pile) => -silent,
            None => 0,
        };
        side.trula = in_piles(TRULA, Bonus::Trula, SILENT_TRULA, ANNOUNCED_TRULA);
        side.kralji = in_piles(KINGS, Bonus::Kralji, SILENT_KRALJI, ANNOUNCED_KRALJI);

        if let Some(last) = outcome.tricks.last() {
            let declarers_take_it = on_declarer_side(last.winner);
            // Kralj ultimo: the called king in the last trick. Plus if the
            // declarer's side wins that trick, minus if not. Only when a
            // king was called, so never in a solo (answer C: also when the
            // declarer holds the king).
            let king_in_it = outcome
                .called_king
                .is_some_and(|king| last.cards.contains(&king));
            side.kralj_ultimo = if owner(Bonus::KraljUltimo).is_some() {
                // Announced (criterion 21): +20 when the king is played to
                // the last trick and the declarer's side takes that trick,
                // -20 in every other case, also when the king was played
                // earlier (no duty to keep it: D141, B13-3 b). Only the
                // holder of the called king may announce it, so the bonus
                // is the declarer's side's.
                announced(
                    true,
                    king_in_it && declarers_take_it,
                    ANNOUNCED_KRALJ_ULTIMO,
                    Target::KraljUltimo,
                )
            } else if !king_in_it {
                0
            } else if declarers_take_it {
                SILENT_KRALJ_ULTIMO
            } else {
                -SILENT_KRALJ_ULTIMO
            };
            // Pagat ultimo: the pagat in the last trick. The bonus follows
            // the side, as kralj ultimo does (D126): the declarer's side
            // writes plus when it takes the trick, with the pagat or with
            // another card, and minus when the opponents take it. Who
            // played the pagat does not change the sign: the side of the
            // pagat's player wins the bonus exactly when it takes the
            // trick, and the bonus of one side is the loss of the other.
            let pagat_in_it = last.cards.contains(&Card::PAGAT);
            side.pagat_ultimo = if let Some(declarers_own_it) = owner(Bonus::PagatUltimo) {
                // Announced (criterion 22; D141, B13-4 a): 50 for the
                // owner when the pagat is played to the last trick and the
                // owner's side takes that trick, also with another card.
                // 50 against the owner in every other case.
                announced(
                    declarers_own_it,
                    pagat_in_it && declarers_take_it == declarers_own_it,
                    ANNOUNCED_PAGAT_ULTIMO,
                    Target::PagatUltimo,
                )
            } else if !pagat_in_it {
                0
            } else if declarers_take_it {
                SILENT_PAGAT_ULTIMO
            } else {
                -SILENT_PAGAT_ULTIMO
            };
        }
    }

    // Mondfang: škis and mond in one trick. The mond's player pays, whoever
    // wins the trick, also to the partner's škis and in a trick the pagat
    // wins (S22 a, S30 a). A valat does not cancel it (D125), and no kontra
    // multiplies it (spec 009, criterion 28).
    for trick in outcome.tricks {
        if trick.cards.contains(&Card::SKIS)
            && let Some(player) = player_of(trick, Card::MOND)
        {
            parts[player.index()].mondfang = MONDFANG;
        }
    }
    // The mond left in the talon and lost with it: the declarer pays. Not
    // in the declarer's valat, where the packets not taken are won
    // (D133); the set is then empty.
    if outcome.talon_with_opponents.contains(Card::MOND) {
        parts[declarer.index()].mondfang = MONDFANG;
    }

    for seat in Seat::ALL {
        if on_declarer_side(seat) {
            let mondfang = parts[seat.index()].mondfang;
            parts[seat.index()] = ScoreParts { mondfang, ..side };
        }
    }
    Score {
        parts,
        declarer_points,
        opponents_points,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounding_examples() {
        assert_eq!(rounded_difference(43), 10);
        assert_eq!(rounded_difference(37), 0);
        assert_eq!(rounded_difference(21), -15);
        assert_eq!(rounded_difference(35), 0);
        assert_eq!(rounded_difference(38), 5);
        assert_eq!(rounded_difference(32), -5);
    }
}
