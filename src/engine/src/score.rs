//! Scoring of the six normal contracts.
//!
//! Rules source: spec 005 criteria 29-44 with the answers of D118, written
//! from pagat.com's four-player rules, and research/01 B3.
//!
//! The score of a deal is one entry per seat. An entry is the sum of eight
//! parts, kept apart because the value head of the network predicts them
//! separately (D15).

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

/// The points the declarer's side needs to win: more than half of 70.
pub const POINTS_TO_WIN: u8 = 36;

/// The parts of one seat's entry. All are 0 for an opponent, except the
/// mondfang.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ScoreParts {
    /// Plus the game value for a win, minus it for a loss.
    pub game: i32,
    /// Card points minus 35, rounded to the nearest 5. Negative in a loss.
    pub difference: i32,
    pub trula: i32,
    pub kralji: i32,
    pub kralj_ultimo: i32,
    pub pagat_ultimo: i32,
    /// +250 or -250 when one side took all twelve tricks. Every other part
    /// is then 0.
    pub valat: i32,
    /// -20 for the player who lost the mond. Each player's own.
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
    /// opponents. Empty if the declarer won them (spec 005 criterion 23).
    pub talon_with_opponents: CardSet,
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
    let valat = if declarer_tricks == outcome.tricks.len() {
        SILENT_VALAT
    } else if declarer_tricks == 0 {
        -SILENT_VALAT
    } else {
        0
    };

    if valat != 0 {
        // All twelve tricks to one side: 250 for or against, and nothing
        // else is scored (criterion 41, answer A of D118). Read here as
        // "no mondfang either"; see the Results of spec 005.
        side.valat = valat;
    } else {
        if let Some((game, difference)) = game_and_difference(outcome.contract, declarer_points) {
            side.game = game;
            side.difference = difference;
        }

        // Trula and kralji: all cards in one side's pile. The talon the
        // declarer did not take is in the opponents' pile (answer B).
        let all_in = |cards: CardSet, bonus: i32| {
            if cards.is_subset(declarer_pile) {
                bonus
            } else if cards.is_subset(opponents_pile) {
                -bonus
            } else {
                0
            }
        };
        side.trula = all_in(TRULA, SILENT_TRULA);
        side.kralji = all_in(KINGS, SILENT_KRALJI);

        if let Some(last) = outcome.tricks.last() {
            // Kralj ultimo: the called king in the last trick. Plus if the
            // declarer's side wins that trick, minus if not. Only when a
            // king was called, so never in a solo (answer C: also when the
            // declarer holds the king).
            if let Some(king) = outcome.called_king
                && last.cards.contains(&king)
            {
                side.kralj_ultimo = if on_declarer_side(last.winner) {
                    SILENT_KRALJ_ULTIMO
                } else {
                    -SILENT_KRALJ_ULTIMO
                };
            }
            // Pagat ultimo: the pagat in the last trick. It "wins" when
            // the pagat itself takes the trick, so when its player is the
            // winner. The declarer's side gains when its own pagat wins or
            // an opponent's pagat loses.
            if let Some(player) = player_of(last, Card::PAGAT) {
                let pagat_wins = last.winner == player;
                side.pagat_ultimo = if on_declarer_side(player) == pagat_wins {
                    SILENT_PAGAT_ULTIMO
                } else {
                    -SILENT_PAGAT_ULTIMO
                };
            }
        }

        // Mondfang: škis and mond in one trick. The mond's player pays,
        // whoever wins the trick, also to the partner's škis and in a
        // trick the pagat wins (S22 a, S30 a).
        for trick in outcome.tricks {
            if trick.cards.contains(&Card::SKIS)
                && let Some(player) = player_of(trick, Card::MOND)
            {
                parts[player.index()].mondfang = MONDFANG;
            }
        }
        // The mond left in the talon and lost with it: the declarer pays.
        if outcome.talon_with_opponents.contains(Card::MOND) {
            parts[declarer.index()].mondfang = MONDFANG;
        }
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
