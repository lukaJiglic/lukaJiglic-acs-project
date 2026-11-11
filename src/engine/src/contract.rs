//! The contract ladder.
//!
//! All twelve contracts of the Default preset are listed, in bidding order,
//! because the action slots of `docs/encoding.md` are fixed for all of
//! them. This slice (spec 005) plays the six normal contracts: tri, dve,
//! ena, solo tri, solo dve, solo ena. The others are never legal yet.

use std::fmt;

/// A contract. The order is the bidding order, low to high.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Contract {
    Klop,
    Tri,
    Dve,
    Ena,
    SoloTri,
    SoloDve,
    SoloEna,
    Berac,
    SoloBrez,
    OdprtiBerac,
    BarvniValat,
    Valat,
}

impl Contract {
    /// All twelve, in bidding order. This is also the order of the bid
    /// slots 1-12 of the encoding.
    pub const ALL: [Contract; 12] = [
        Contract::Klop,
        Contract::Tri,
        Contract::Dve,
        Contract::Ena,
        Contract::SoloTri,
        Contract::SoloDve,
        Contract::SoloEna,
        Contract::Berac,
        Contract::SoloBrez,
        Contract::OdprtiBerac,
        Contract::BarvniValat,
        Contract::Valat,
    ];

    /// The six normal contracts, low to high.
    pub const NORMAL: [Contract; 6] = [
        Contract::Tri,
        Contract::Dve,
        Contract::Ena,
        Contract::SoloTri,
        Contract::SoloDve,
        Contract::SoloEna,
    ];

    /// Position in the ladder: 0 for klop, 11 for valat.
    pub const fn index(self) -> usize {
        self as usize
    }

    /// The contract at a ladder position, or `None` above 11.
    pub const fn from_index(index: usize) -> Option<Contract> {
        if index < Contract::ALL.len() {
            Some(Contract::ALL[index])
        } else {
            None
        }
    }

    /// Tri, dve, ena and their solo forms.
    pub const fn is_normal(self) -> bool {
        self.packet_size().is_some()
    }

    /// Tri, dve and ena: the declarer calls a king and may have a partner.
    pub const fn calls_king(self) -> bool {
        matches!(self, Contract::Tri | Contract::Dve | Contract::Ena)
    }

    /// How many talon cards the declarer takes, which is also the size of a
    /// talon packet: 3, 2 or 1. `None` for a contract that is not normal.
    pub const fn packet_size(self) -> Option<usize> {
        match self {
            Contract::Tri | Contract::SoloTri => Some(3),
            Contract::Dve | Contract::SoloDve => Some(2),
            Contract::Ena | Contract::SoloEna => Some(1),
            _ => None,
        }
    }

    /// The game value of a normal contract: tri 10, dve 20, ena 30, solo
    /// tri 40, solo dve 50, solo ena 60. `None` for the others, which are
    /// scored in a later spec.
    pub const fn game_value(self) -> Option<i32> {
        match self {
            Contract::Tri => Some(10),
            Contract::Dve => Some(20),
            Contract::Ena => Some(30),
            Contract::SoloTri => Some(40),
            Contract::SoloDve => Some(50),
            Contract::SoloEna => Some(60),
            _ => None,
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Contract::Klop => "klop",
            Contract::Tri => "tri",
            Contract::Dve => "dve",
            Contract::Ena => "ena",
            Contract::SoloTri => "solo tri",
            Contract::SoloDve => "solo dve",
            Contract::SoloEna => "solo ena",
            Contract::Berac => "berac",
            Contract::SoloBrez => "solo brez",
            Contract::OdprtiBerac => "odprti berac",
            Contract::BarvniValat => "barvni valat",
            Contract::Valat => "valat",
        }
    }
}

impl fmt::Display for Contract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ladder_positions() {
        for (index, contract) in Contract::ALL.into_iter().enumerate() {
            assert_eq!(contract.index(), index);
            assert_eq!(Contract::from_index(index), Some(contract));
        }
        assert_eq!(Contract::from_index(12), None);
        assert!(Contract::Tri < Contract::Dve && Contract::SoloEna < Contract::Berac);
        let normal: Vec<Contract> = Contract::ALL
            .into_iter()
            .filter(|c| c.is_normal())
            .collect();
        assert_eq!(normal, Contract::NORMAL);
    }
}
