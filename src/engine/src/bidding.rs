//! The auction.
//!
//! Rules source: spec 005 criteria 1-11, written from pagat.com's
//! four-player rules, and research/01 B3.
//!
//! - Forehand is the seat after the dealer. The seat after forehand speaks
//!   first; forehand speaks fourth. After that the turn goes round the
//!   seats that have not passed.
//! - Priority runs from forehand (highest) round to the dealer (lowest).
//! - A seat below the highest bidder must bid a higher contract or pass.
//!   A seat above the highest bidder may also bid the same contract.
//! - A seat that passed is out.
//! - The auction ends when three seats have passed and the fourth has bid.
//!   If the three others pass before forehand has spoken, forehand must
//!   name a contract.
//!
//! This slice (spec 005) knows the six normal contracts only. Tri is open
//! to forehand alone, and only after three passes. Klop, berač and the
//! higher contracts are never legal; they come with flow C.

use crate::contract::Contract;
use crate::deal::Seat;

/// One call in the auction.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BidEvent {
    pub seat: Seat,
    /// The contract bid, or `None` for a pass.
    pub bid: Option<Contract>,
}

/// Room for the calls of an auction. Every bid is a higher contract than
/// the last one, or the same contract by a seat of higher priority, so
/// 12 x 4 bids and 3 passes is a safe upper limit for any ladder.
///
/// The real maximum is far lower. With the six normal contracts of this
/// slice the longest auction has 13 calls; `tests/bidding.rs` walks every
/// auction and checks that.
const MAX_CALLS: usize = 12 * 4 + 3;

/// The state of the auction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bidding {
    forehand: Seat,
    /// The calls so far. Only the first `made` entries count.
    calls: [BidEvent; MAX_CALLS],
    made: usize,
    passed: [bool; 4],
    highest: Option<(Seat, Contract)>,
    /// The seat to speak. Not meaningful once the auction is over.
    turn: Seat,
}

impl Bidding {
    /// A new auction. The seat after forehand speaks first.
    pub fn new(forehand: Seat) -> Bidding {
        let unused = BidEvent {
            seat: forehand,
            bid: None,
        };
        Bidding {
            forehand,
            calls: [unused; MAX_CALLS],
            made: 0,
            passed: [false; 4],
            highest: None,
            turn: forehand.next(),
        }
    }

    pub fn forehand(&self) -> Seat {
        self.forehand
    }

    /// The calls so far, in order.
    pub fn calls(&self) -> &[BidEvent] {
        &self.calls[..self.made]
    }

    pub fn has_passed(&self, seat: Seat) -> bool {
        self.passed[seat.index()]
    }

    /// The highest bid so far and the seat that made it.
    pub fn highest(&self) -> Option<(Seat, Contract)> {
        self.highest
    }

    /// Steps from forehand: 0 for forehand (highest priority), 3 for the
    /// dealer (lowest).
    pub fn rank(&self, seat: Seat) -> usize {
        (seat.index() + 4 - self.forehand.index()) % 4
    }

    fn passes(&self) -> usize {
        self.passed.iter().filter(|&&p| p).count()
    }

    /// True once three seats have passed and the fourth has bid.
    pub fn is_over(&self) -> bool {
        self.passes() == 3 && self.highest.is_some()
    }

    /// The seat to speak, or `None` when the auction is over.
    pub fn to_speak(&self) -> Option<Seat> {
        if self.is_over() {
            None
        } else {
            Some(self.turn)
        }
    }

    /// The winner and the winning bid, once the auction is over.
    pub fn winner(&self) -> Option<(Seat, Contract)> {
        if self.is_over() { self.highest } else { None }
    }

    /// True when the three others passed before forehand spoke: forehand
    /// must name a contract.
    fn forehand_must_name(&self) -> bool {
        self.highest.is_none() && self.passes() == 3
    }

    /// May the seat to speak pass?
    pub fn may_pass(&self) -> bool {
        !self.is_over() && !self.forehand_must_name()
    }

    /// May the seat to speak bid this contract?
    pub fn may_bid(&self, contract: Contract) -> bool {
        if self.is_over() || !contract.is_normal() {
            return false;
        }
        match self.highest {
            // Nobody has bid. Tri is for forehand after three passes.
            None => contract != Contract::Tri || self.forehand_must_name(),
            // Higher, or the same from a seat of higher priority. Tri is
            // below every bid that can stand here.
            Some((holder, held)) => {
                contract > held || (contract == held && self.rank(self.turn) < self.rank(holder))
            }
        }
    }

    /// The seat to speak passes (`None`) or bids. The caller checks
    /// `may_pass` or `may_bid` first.
    pub(crate) fn call(&mut self, bid: Option<Contract>) {
        debug_assert!(match bid {
            None => self.may_pass(),
            Some(contract) => self.may_bid(contract),
        });
        self.calls[self.made] = BidEvent {
            seat: self.turn,
            bid,
        };
        self.made += 1;
        match bid {
            None => self.passed[self.turn.index()] = true,
            Some(contract) => self.highest = Some((self.turn, contract)),
        }
        if !self.is_over() {
            // The next seat that has not passed. One always exists: at
            // most three have passed. The search is three steps at most.
            let others = [1, 2, 3].map(|steps| self.turn.plus(steps));
            if let Some(next) = others.into_iter().find(|seat| !self.passed[seat.index()]) {
                self.turn = next;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longest_auction_fits() {
        // Bid whenever possible, the lowest legal contract; pass otherwise.
        // This greedy auction stays under the longest one (13 calls).
        for forehand in Seat::ALL {
            let mut bidding = Bidding::new(forehand);
            while !bidding.is_over() {
                let lowest = Contract::NORMAL.into_iter().find(|&c| bidding.may_bid(c));
                assert!(lowest.is_some() || bidding.may_pass());
                bidding.call(lowest);
            }
            assert!(bidding.calls().len() <= 13, "{}", bidding.calls().len());
            assert!(bidding.calls().len() <= MAX_CALLS);
        }
    }

    #[test]
    fn rank_counts_from_forehand() {
        let seat = |i| Seat::new(i).unwrap();
        let bidding = Bidding::new(seat(2));
        assert_eq!([0, 1, 2, 3].map(|i| bidding.rank(seat(i))), [2, 3, 0, 1]);
    }
}
