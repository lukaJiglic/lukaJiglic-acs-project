//! The reward of a deal (`docs/encoding.md` section 8, D117, D121).
//!
//! Let `e` be the four entries the deal writes on paper. The reward of
//! seat i is (e_i - mean of the other three) / 50. The four rewards sum to
//! zero.
//!
//! An entry is a sum of ten parts: game, difference, trula, kralji, kralj
//! ultimo, pagat ultimo, valat, mondfang, klop, radelc. The same formula is
//! applied to each part on its own, which gives 4 x 10 numbers; the ten of
//! a seat add up to its reward. Klop and radelc are zero until those rules
//! are built.

use crate::score::Score;

use super::{N_SCORE_PARTS, REWARD_SCALE};

/// The entries of a deal and the rewards made from them, by seat index.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rewards {
    /// The entry each seat writes.
    pub entries: [i32; 4],
    /// The ten parts of each entry; they add up to the entry.
    pub entry_parts: [[i32; N_SCORE_PARTS]; 4],
    /// The reward of each seat: the sum of its ten reward parts.
    pub reward: [f32; 4],
    /// The reward by part: (own part - mean of the others' part) / 50.
    pub parts: [[f32; N_SCORE_PARTS]; 4],
}

/// The rewards of a finished deal.
pub fn rewards(score: &Score) -> Rewards {
    let mut entry_parts = [[0i32; N_SCORE_PARTS]; 4];
    for (seat, row) in crate::deal::Seat::ALL.into_iter().zip(&mut entry_parts) {
        let p = score.parts(seat);
        // Klop (8) and radelc (9) stay zero.
        *row = [
            p.game,
            p.difference,
            p.trula,
            p.kralji,
            p.kralj_ultimo,
            p.pagat_ultimo,
            p.valat,
            p.mondfang,
            0,
            0,
        ];
    }

    // (own - (total - own) / 3) / 50, in 64 bits, then rounded once.
    let relative = |own: i32, total: i32| {
        ((f64::from(own) - f64::from(total - own) / 3.0) / f64::from(REWARD_SCALE)) as f32
    };

    let entries = score.entries();
    let total_entry: i32 = entries.iter().sum();
    let reward = entries.map(|entry| relative(entry, total_entry));

    let mut parts = [[0f32; N_SCORE_PARTS]; 4];
    for part in 0..N_SCORE_PARTS {
        let total: i32 = entry_parts.iter().map(|row| row[part]).sum();
        for seat in 0..4 {
            parts[seat][part] = relative(entry_parts[seat][part], total);
        }
    }
    // Each number is rounded to 32 bits on its own, so the ten parts of a
    // seat may miss its reward by a unit or two in the last place (measured:
    // at most 2.4e-7 in 40,000 seat-deals). The contract asks for 1e-6.
    Rewards {
        entries,
        entry_parts,
        reward,
        parts,
    }
}
