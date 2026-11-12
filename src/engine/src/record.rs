//! The JSON game record, first version (D59, narrowed by D120).
//!
//! A record holds what is needed to play a deal again: the seed of the
//! cards, the dealer, the rules in force, and per decision the acting
//! seat, the legal slots and the chosen slot. Then the entries of the four
//! seats and their ten parts. It stores no observation: replaying the
//! record rebuilds every observation.
//!
//! ```json
//! {
//!   "record_version": 1,
//!   "encoding_version": 1,
//!   "engine_version": "0.2.0",
//!   "seed": "42",
//!   "dealer": 3,
//!   "preset": "default",
//!   "toggles": {"S15": "a", "S22": "a"},
//!   "decisions": [{"seat": 1, "legal": [0, 3, 4, 5, 6, 7], "chosen": 0}],
//!   "entries": [0, 0, 0, 0],
//!   "parts": [[0, 0, 0, 0, 0, 0, 0, 0, 0, 0]]
//! }
//! ```
//!
//! The seed is a decimal string, because a 64-bit number does not survive
//! JavaScript. Slots are those of `docs/encoding.md` section 5. The parts
//! are in the order of section 8: game, difference, trula, kralji, kralj
//! ultimo, pagat ultimo, valat, mondfang, klop, radelc. A field `agent` may
//! be added by whoever has a model (values, beliefs); the engine keeps it
//! and does not read it.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::action::Action;
use crate::deal::Seat;
use crate::encode::{self, ENCODING_VERSION, N_SCORE_PARTS, PRESET, SCORING_TOGGLES};
use crate::game::{Game, Phase};

/// The version of this record format.
pub const RECORD_VERSION: u32 = 1;

/// One decision of a deal.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    /// The seat that acted, 0 to 3.
    pub seat: u8,
    /// The slots that were legal, in rising order.
    pub legal: Vec<u16>,
    /// The slot that was taken.
    pub chosen: u16,
}

/// The record of one finished deal. A field that is not listed here is
/// refused when a record is read; only `agent` may hold anything.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub record_version: u32,
    pub encoding_version: u32,
    /// The version of the engine crate that wrote the record.
    pub engine_version: String,
    /// The seed of the cards, as a decimal string: digits only, no sign, no
    /// leading zero, at most 2^64 - 1.
    ///
    /// Reading B12-8 (Batch 12, open): a string, because a 64-bit number
    /// does not survive JavaScript's numbers.
    pub seed: String,
    /// The seat that dealt, 0 to 3.
    pub dealer: u8,
    pub preset: String,
    /// The scoring toggles and their option letters.
    pub toggles: BTreeMap<String, String>,
    pub decisions: Vec<Decision>,
    /// The entry each seat wrote.
    pub entries: [i32; 4],
    /// The ten parts of each entry.
    pub parts: [[i32; N_SCORE_PARTS]; 4],
    /// Optional outputs of an agent. Not read by the engine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<serde_json::Value>,
}

/// Why a record was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RecordError {
    /// The text is not a record.
    Json(String),
    /// The record or encoding version is not the one this engine reads.
    Version {
        field: &'static str,
        found: u32,
        expected: u32,
    },
    /// The seed, the dealer, the preset or a toggle cannot be used.
    Header(String),
    /// The chosen slot of a decision is not a legal action there.
    Illegal { decision: usize, slot: u16 },
    /// A decision does not fit the deal: another seat was to act, or other
    /// slots were legal.
    Mismatch { decision: usize, what: &'static str },
    /// The deal is not over after the last decision.
    NotFinished,
    /// The replayed deal writes other entries than the record says.
    Entries { expected: [i32; 4], found: [i32; 4] },
}

impl fmt::Display for RecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecordError::Json(why) => write!(f, "not a game record: {why}"),
            RecordError::Version {
                field,
                found,
                expected,
            } => {
                write!(f, "{field} is {found}; this engine reads {expected}")
            }
            RecordError::Header(why) => write!(f, "the record's header cannot be used: {why}"),
            RecordError::Illegal { decision, slot } => {
                write!(
                    f,
                    "decision {decision}: slot {slot} is not a legal action there"
                )
            }
            RecordError::Mismatch { decision, what } => {
                write!(f, "decision {decision}: {what}")
            }
            RecordError::NotFinished => write!(f, "the deal is not over after the last decision"),
            RecordError::Entries { expected, found } => write!(
                f,
                "the replayed deal writes {found:?} and not the recorded entries {expected:?}"
            ),
        }
    }
}

impl std::error::Error for RecordError {}

/// The option letters of the scoring toggles in force.
fn toggles() -> BTreeMap<String, String> {
    SCORING_TOGGLES
        .iter()
        .map(|&(name, option, _)| {
            let letter = char::from(b'a' + option as u8);
            (name.to_string(), letter.to_string())
        })
        .collect()
}

impl Record {
    /// The record of a finished deal that was dealt from `seed`. The
    /// dealer and the actions are taken from the game; the deal is played
    /// again from the seed to collect the legal slots, and must come out
    /// the same.
    pub fn of_game(seed: u64, game: &Game) -> Result<Record, RecordError> {
        if game.phase() != Phase::Over {
            return Err(RecordError::NotFinished);
        }
        let dealer = game.public().dealer();
        let mut replay = Game::from_seed(seed, dealer);
        let mut decisions = Vec::new();
        for (index, action) in game.full().actions().enumerate() {
            let Some(seat) = replay.to_act() else {
                return Err(RecordError::Mismatch {
                    decision: index,
                    what: "the deal is already over",
                });
            };
            let legal = replay.legal_actions();
            let slot = action.slot() as u16;
            if replay.apply(action).is_err() {
                return Err(RecordError::Illegal {
                    decision: index,
                    slot,
                });
            }
            decisions.push(Decision {
                seat: seat.index() as u8,
                legal: legal.iter().map(|legal| legal.slot() as u16).collect(),
                chosen: slot,
            });
        }
        if replay != *game {
            return Err(RecordError::Mismatch {
                decision: decisions.len(),
                what: "the deal of this seed is not the game that was played",
            });
        }
        let score = game.score().ok_or(RecordError::NotFinished)?;
        let rewards = encode::rewards(&score);
        Ok(Record {
            record_version: RECORD_VERSION,
            encoding_version: ENCODING_VERSION,
            engine_version: crate::VERSION.to_string(),
            seed: seed.to_string(),
            dealer: dealer.index() as u8,
            preset: PRESET.to_string(),
            toggles: toggles(),
            decisions,
            entries: rewards.entries,
            parts: rewards.entry_parts,
            agent: None,
        })
    }

    /// Plays the record again. Every decision must fit: the seat, the legal
    /// slots, a legal chosen slot. At the end the deal must be over and
    /// write the recorded entries. The error names the first decision that
    /// does not fit, counted from 0.
    pub fn replay(&self) -> Result<Game, RecordError> {
        for (field, found, expected) in [
            ("record_version", self.record_version, RECORD_VERSION),
            ("encoding_version", self.encoding_version, ENCODING_VERSION),
        ] {
            if found != expected {
                return Err(RecordError::Version {
                    field,
                    found,
                    expected,
                });
            }
        }
        // One written form only: what `of_game` writes. "+42" and "042"
        // would parse, and two texts would then name one deal.
        let seed = self
            .seed
            .parse::<u64>()
            .ok()
            .filter(|seed| seed.to_string() == self.seed)
            .ok_or_else(|| {
                RecordError::Header(format!(
                    "the seed \"{}\" is not a number from 0 to 2^64 - 1 in plain decimal digits",
                    self.seed
                ))
            })?;
        let dealer = Seat::new(self.dealer)
            .ok_or_else(|| RecordError::Header(format!("dealer {} is not a seat", self.dealer)))?;
        if self.preset != PRESET {
            return Err(RecordError::Header(format!(
                "preset \"{}\"; this engine plays \"{PRESET}\"",
                self.preset
            )));
        }
        // The engine plays one set of rules. A record made under other
        // toggles would replay to other entries, so it is refused here,
        // with the toggle named.
        let in_force = toggles();
        for (name, option) in &in_force {
            match self.toggles.get(name) {
                Some(found) if found == option => {}
                Some(found) => {
                    return Err(RecordError::Header(format!(
                        "toggle {name} is \"{found}\"; this engine plays \"{option}\""
                    )));
                }
                None => {
                    return Err(RecordError::Header(format!("toggle {name} is missing")));
                }
            }
        }
        if let Some(name) = self
            .toggles
            .keys()
            .find(|name| !in_force.contains_key(*name))
        {
            return Err(RecordError::Header(format!(
                "toggle {name} is not known to this engine"
            )));
        }

        let mut game = Game::from_seed(seed, dealer);
        for (index, decision) in self.decisions.iter().enumerate() {
            let Some(seat) = game.to_act() else {
                return Err(RecordError::Mismatch {
                    decision: index,
                    what: "the deal is already over",
                });
            };
            if usize::from(decision.seat) != seat.index() {
                return Err(RecordError::Mismatch {
                    decision: index,
                    what: "another seat is to act",
                });
            }
            let legal = game.legal_actions();
            let action = Action::from_slot(usize::from(decision.chosen))
                .filter(|&action| legal.contains(action))
                .ok_or(RecordError::Illegal {
                    decision: index,
                    slot: decision.chosen,
                })?;
            let slots: Vec<u16> = legal.iter().map(|legal| legal.slot() as u16).collect();
            if decision.legal != slots {
                return Err(RecordError::Mismatch {
                    decision: index,
                    what: "the legal slots differ",
                });
            }
            if game.apply(action).is_err() {
                return Err(RecordError::Illegal {
                    decision: index,
                    slot: decision.chosen,
                });
            }
        }
        let score = game.score().ok_or(RecordError::NotFinished)?;
        let rewards = encode::rewards(&score);
        if rewards.entries != self.entries || rewards.entry_parts != self.parts {
            return Err(RecordError::Entries {
                expected: self.entries,
                found: rewards.entries,
            });
        }
        Ok(game)
    }

    /// The record as JSON on one line.
    pub fn to_json(&self) -> String {
        compact(self)
    }

    /// The record as JSON laid out for reading: one header field per line
    /// and one decision per line. It ends with a line break.
    pub fn to_json_pretty(&self) -> String {
        let mut text = String::from("{\n");
        text += &format!("  \"record_version\": {},\n", self.record_version);
        text += &format!("  \"encoding_version\": {},\n", self.encoding_version);
        text += &format!("  \"engine_version\": {},\n", compact(&self.engine_version));
        text += &format!("  \"seed\": {},\n", compact(&self.seed));
        text += &format!("  \"dealer\": {},\n", self.dealer);
        text += &format!("  \"preset\": {},\n", compact(&self.preset));
        text += &format!("  \"toggles\": {},\n", compact(&self.toggles));
        text += "  \"decisions\": [\n";
        for (index, decision) in self.decisions.iter().enumerate() {
            let comma = if index + 1 < self.decisions.len() {
                ","
            } else {
                ""
            };
            text += &format!("    {}{comma}\n", compact(decision));
        }
        text += "  ],\n";
        text += &format!("  \"entries\": {},\n", compact(&self.entries));
        text += "  \"parts\": [\n";
        for (index, row) in self.parts.iter().enumerate() {
            let comma = if index + 1 < self.parts.len() {
                ","
            } else {
                ""
            };
            text += &format!("    {}{comma}\n", compact(row));
        }
        text += "  ]";
        if let Some(agent) = &self.agent {
            text += &format!(",\n  \"agent\": {}", compact(agent));
        }
        text += "\n}\n";
        text
    }

    /// Reads a record from JSON, in either layout. It is not replayed; call
    /// [`Record::replay`] to check it.
    pub fn from_json(text: &str) -> Result<Record, RecordError> {
        serde_json::from_str(text).map_err(|error| RecordError::Json(error.to_string()))
    }
}

/// One value as JSON on one line.
fn compact<T: Serialize + ?Sized>(value: &T) -> String {
    // The values of a record are strings, numbers, arrays and maps with
    // string keys: writing them cannot fail.
    serde_json::to_string(value).unwrap_or_default()
}
