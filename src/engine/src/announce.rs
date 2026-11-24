//! The round of announcements and kontra (spec 009, deal flow B).
//!
//! Rules source: pagat.com, Slovenian Tarok, read on 2026-11-24, with the
//! answers of the user's table (D141). The round lies between the talon
//! exchange and the first lead of the six normal contracts.
//!
//! - The declarer speaks first, then the seats in order of play.
//! - A turn is any number of announcements and kontras, one per action,
//!   and then "done". A turn of "done" alone is a pass.
//! - The round ends when every seat has had a turn and the last three
//!   turns were passes.
//! - Five bonuses can be announced, each once in a deal: trula, kralji,
//!   kralj ultimo, pagat ultimo, valat.
//! - Six things can be raised by kontra: the game and each announced
//!   bonus. Four levels: kontra, re, sub, mort.
//!
//! This module holds what was said and whose turn it is. Who may say what
//! depends on the hands and the sides; that is in `game.rs`.

use crate::deal::Seat;

/// A bonus that can be announced.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Bonus {
    Trula,
    Kralji,
    KraljUltimo,
    PagatUltimo,
    Valat,
}

impl Bonus {
    /// In the order of the slots 77-81 and the tokens 146-150.
    pub const ALL: [Bonus; 5] = [
        Bonus::Trula,
        Bonus::Kralji,
        Bonus::KraljUltimo,
        Bonus::PagatUltimo,
        Bonus::Valat,
    ];

    /// 0 to 4, in the order of [`Bonus::ALL`].
    pub const fn index(self) -> usize {
        self as usize
    }
}

/// What a kontra is said on: the game or an announced bonus.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Target {
    Game,
    Trula,
    Kralji,
    KraljUltimo,
    PagatUltimo,
    Valat,
}

impl Target {
    /// In the order of the slots 82-87 and the tokens 151-156.
    pub const ALL: [Target; 6] = [
        Target::Game,
        Target::Trula,
        Target::Kralji,
        Target::KraljUltimo,
        Target::PagatUltimo,
        Target::Valat,
    ];

    /// 0 to 5, in the order of [`Target::ALL`].
    pub const fn index(self) -> usize {
        self as usize
    }

    /// The bonus of this target. `None` for the game.
    pub const fn bonus(self) -> Option<Bonus> {
        match self {
            Target::Game => None,
            Target::Trula => Some(Bonus::Trula),
            Target::Kralji => Some(Bonus::Kralji),
            Target::KraljUltimo => Some(Bonus::KraljUltimo),
            Target::PagatUltimo => Some(Bonus::PagatUltimo),
            Target::Valat => Some(Bonus::Valat),
        }
    }

    /// The target of a bonus.
    pub const fn of(bonus: Bonus) -> Target {
        match bonus {
            Bonus::Trula => Target::Trula,
            Bonus::Kralji => Target::Kralji,
            Bonus::KraljUltimo => Target::KraljUltimo,
            Bonus::PagatUltimo => Target::PagatUltimo,
            Bonus::Valat => Target::Valat,
        }
    }
}

/// The highest kontra level: 1 kontra, 2 re, 3 sub, 4 mort (S18 a).
pub const MORT: u8 = 4;

/// The most announcements and kontras a deal can have: five bonuses, and
/// four levels on each of six targets.
pub const MAX_SAID: usize = Bonus::ALL.len() + Target::ALL.len() * MORT as usize;

/// What a kontra level multiplies by: 1, 2, 4, 8, 16.
pub const fn multiplier(level: u8) -> i32 {
    1 << level
}

/// True if the next level of a target is said by the side that owns it:
/// re (2) and mort (4). Kontra (1) and sub (3) are said by the other side.
pub const fn next_is_for_the_owner(level: u8) -> bool {
    level % 2 == 1
}

/// True for the two levels the owning side says: re (2) and mort (4).
pub const fn is_re_or_mort(level: u8) -> bool {
    level == 2 || level == MORT
}

/// What has been said in the round, and whose turn it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Round {
    /// Who announced each bonus.
    announced: [Option<Seat>; 5],
    /// The kontra level of each target, 0 to 4.
    levels: [u8; 6],
    /// The seat whose turn it is.
    speaker: Seat,
    /// Turns that are finished.
    turns: u16,
    /// How many of the last finished turns were passes, in a row.
    passes: u16,
    /// The speaker has said something in this turn.
    spoke: bool,
}

impl Round {
    /// A round in which nothing is said yet. `first` has the first turn:
    /// the declarer.
    pub const fn new(first: Seat) -> Round {
        Round {
            announced: [None; 5],
            levels: [0; 6],
            speaker: first,
            turns: 0,
            passes: 0,
            spoke: false,
        }
    }

    /// The seat whose turn it is.
    pub const fn speaker(&self) -> Seat {
        self.speaker
    }

    /// The seat that announced a bonus.
    pub const fn announced(&self, bonus: Bonus) -> Option<Seat> {
        self.announced[bonus.index()]
    }

    /// The kontra level of a target: 0 none, 1 kontra, 2 re, 3 sub, 4 mort.
    pub const fn level(&self, target: Target) -> u8 {
        self.levels[target.index()]
    }

    /// The speaker announces a bonus. The caller has checked that it may.
    pub(crate) const fn announce(&mut self, bonus: Bonus) {
        self.announced[bonus.index()] = Some(self.speaker);
        self.spoke = true;
    }

    /// The speaker raises a target by one level and gets the new level.
    /// The caller has checked that it may; mort is never passed.
    pub(crate) const fn raise(&mut self, target: Target) -> u8 {
        if self.levels[target.index()] < MORT {
            self.levels[target.index()] += 1;
        }
        self.spoke = true;
        self.levels[target.index()]
    }

    /// The speaker says "done": the turn goes to the next seat. Returns
    /// true if the round is over: every seat has had a turn and the last
    /// three turns were passes.
    pub(crate) const fn done(&mut self) -> bool {
        self.turns += 1;
        self.passes = if self.spoke { 0 } else { self.passes + 1 };
        self.spoke = false;
        self.speaker = self.speaker.next();
        self.turns >= 4 && self.passes >= 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bonuses_and_targets_are_in_slot_order() {
        for (index, bonus) in Bonus::ALL.into_iter().enumerate() {
            assert_eq!(bonus.index(), index);
            assert_eq!(Target::of(bonus).bonus(), Some(bonus));
            assert_eq!(Target::of(bonus).index(), index + 1);
        }
        for (index, target) in Target::ALL.into_iter().enumerate() {
            assert_eq!(target.index(), index);
        }
        assert_eq!(Target::Game.bonus(), None);
        assert_eq!(MAX_SAID, 29);
        assert_eq!([0, 1, 2, 3, 4].map(multiplier), [1, 2, 4, 8, 16]);
    }

    #[test]
    fn a_long_round_cannot_overflow_its_counters() {
        // Far more turns than a deal can have.
        let mut round = Round::new(Seat::ALL[0]);
        for _ in 0..1_000 {
            round.announce(Bonus::Trula);
            assert!(!round.done());
        }
        assert!(!round.done());
        assert!(!round.done());
        assert!(round.done());
    }
}
