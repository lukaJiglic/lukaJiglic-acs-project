//! Spec 005, criteria 1-11: bidding and the choice of the contract.
//!
//! Seats: A is forehand (seat 0), then B, C, D; D (seat 3) deals.

mod common;

use common::flow::{A, B, C, D, NORMAL, act, bids, game, legal};
use tarok_engine::Contract::{Dve, Ena, SoloDve, SoloEna, SoloTri, Tri};
use tarok_engine::{Action, Action::Pass, Contract, Deal, Game, Phase, Seat};

fn new_game() -> Game {
    game(&Deal::from_seed(1))
}

fn bid(contract: Contract) -> Action {
    Action::Bid(contract)
}

/// Pass, then the listed bids: the usual shape of a legal set in bidding.
fn pass_and(contracts: &[Contract]) -> Vec<Action> {
    let mut actions = vec![Pass];
    actions.extend(bids(contracts));
    actions
}

/// Applies the actions. Each entry names the seat that must be on turn.
fn auction(game: &mut Game, steps: &[(Seat, Action)]) {
    for &(seat, action) in steps {
        assert_eq!(game.phase(), Phase::Bidding);
        act(game, seat, action);
    }
}

#[test]
fn c01_b_speaks_first_then_c_d_a_then_round_the_players_still_in() {
    let mut game = new_game();
    assert_eq!(game.phase(), Phase::Bidding);
    assert_eq!(game.public().dealer(), D);
    assert_eq!(game.public().forehand(), A);

    // First round: B, C, D, A. Then round again: B, C.
    auction(
        &mut game,
        &[
            (B, bid(Dve)),
            (C, bid(Ena)),
            (D, bid(SoloTri)),
            (A, Pass),
            (B, Pass),
            (C, Pass),
        ],
    );
    assert_eq!(game.phase(), Phase::ContractChoice);

    // A seat that passed is skipped in the later rounds: after C's pass the
    // turn goes B, (C skipped), D.
    let mut game = new_game();
    auction(
        &mut game,
        &[
            (B, bid(Dve)),
            (C, Pass),
            (D, bid(Ena)),
            (A, bid(Ena)),
            (B, Pass),
            (D, bid(SoloTri)),
            (A, bid(SoloTri)),
            (D, Pass),
        ],
    );
    assert_eq!(game.phase(), Phase::ContractChoice);
    assert_eq!(game.public().declarer(), Some(A));

    // The same order for another dealer: seat 0 deals, so seat 1 is
    // forehand and seat 2 speaks first.
    let seat = |i| Seat::new(i).unwrap();
    let mut game = Game::new(&Deal::from_seed(1), seat(0));
    assert_eq!(game.public().forehand(), seat(1));
    auction(
        &mut game,
        &[
            (seat(2), bid(Dve)),
            (seat(3), Pass),
            (seat(0), Pass),
            (seat(1), Pass),
        ],
    );
    assert_eq!(game.public().declarer(), Some(seat(2)));
}

#[test]
fn c02_first_speaker_may_pass_or_bid_dve_to_solo_ena() {
    let game = new_game();
    assert_eq!(game.to_act(), Some(B));
    assert_eq!(
        legal(&game),
        pass_and(&[Dve, Ena, SoloTri, SoloDve, SoloEna])
    );
    // Not klop and not tri.
    assert!(!game.legal_actions().contains(bid(Contract::Klop)));
    assert!(!game.legal_actions().contains(bid(Tri)));
}

#[test]
fn c03_lower_priority_must_bid_higher_or_pass() {
    let mut game = new_game();
    auction(&mut game, &[(B, bid(Dve))]);
    // C is below B: not dve again.
    assert_eq!(game.to_act(), Some(C));
    assert_eq!(legal(&game), pass_and(&[Ena, SoloTri, SoloDve, SoloEna]));
    assert_eq!(
        game.apply(bid(Dve)),
        Err(tarok_engine::GameError::NotLegal(bid(Dve)))
    );

    auction(&mut game, &[(C, bid(SoloTri))]);
    // D is below C.
    assert_eq!(legal(&game), pass_and(&[SoloDve, SoloEna]));

    // Under solo ena a lower seat can only pass.
    auction(&mut game, &[(D, bid(SoloEna))]);
    auction(&mut game, &[(A, Pass), (B, Pass)]);
    // C is above D and may still hold solo ena.
    assert_eq!(legal(&game), pass_and(&[SoloEna]));
    auction(&mut game, &[(C, bid(SoloEna))]);
    assert_eq!(game.to_act(), Some(D));
    assert_eq!(legal(&game), vec![Pass]);
}

#[test]
fn c04_higher_priority_may_bid_the_same() {
    // The source example: B bids dve, C passes, D bids ena; A may bid ena.
    let mut game = new_game();
    auction(&mut game, &[(B, bid(Dve)), (C, Pass), (D, bid(Ena))]);
    assert_eq!(game.to_act(), Some(A));
    assert_eq!(legal(&game), pass_and(&[Ena, SoloTri, SoloDve, SoloEna]));
    auction(&mut game, &[(A, bid(Ena))]);
    assert_eq!(game.public().highest_bid(), Some((A, Ena)));

    // B is below A: above ena or pass.
    assert_eq!(game.to_act(), Some(B));
    assert_eq!(legal(&game), pass_and(&[SoloTri, SoloDve, SoloEna]));
    auction(&mut game, &[(B, Pass)]);

    // D must then bid above ena or pass.
    assert_eq!(game.to_act(), Some(D));
    assert_eq!(legal(&game), pass_and(&[SoloTri, SoloDve, SoloEna]));
}

#[test]
fn c05_priority_runs_a_b_c_d() {
    // (the seats that pass before the bidder, the bidder, then every seat
    // that speaks after it, with "may hold the bid")
    let may_hold = |game: &Game| game.legal_actions().contains(bid(Dve));

    // B bids dve: C and D may not hold it, A may.
    let mut game = new_game();
    auction(&mut game, &[(B, bid(Dve))]);
    assert!(!may_hold(&game), "C is below B");
    auction(&mut game, &[(C, Pass)]);
    assert!(!may_hold(&game), "D is below B");
    auction(&mut game, &[(D, Pass)]);
    assert!(may_hold(&game), "A is above B");

    // C bids dve: D may not hold it, A may.
    let mut game = new_game();
    auction(&mut game, &[(B, Pass), (C, bid(Dve))]);
    assert!(!may_hold(&game), "D is below C");
    auction(&mut game, &[(D, Pass)]);
    assert!(may_hold(&game), "A is above C");

    // D bids dve: A may hold it.
    let mut game = new_game();
    auction(&mut game, &[(B, Pass), (C, Pass), (D, bid(Dve))]);
    assert!(may_hold(&game), "A is above D");

    // B above D and C above D: D raises to ena, B and C may hold ena.
    let hold_ena = |game: &Game| game.legal_actions().contains(bid(Ena));
    let mut game = new_game();
    auction(
        &mut game,
        &[(B, bid(Dve)), (C, Pass), (D, bid(Ena)), (A, Pass)],
    );
    assert!(hold_ena(&game), "B is above D");
    let mut game = new_game();
    auction(
        &mut game,
        &[(B, Pass), (C, bid(Dve)), (D, bid(Ena)), (A, Pass)],
    );
    assert!(hold_ena(&game), "C is above D");

    // B above C: C raises to ena, B may hold ena.
    let mut game = new_game();
    auction(
        &mut game,
        &[(B, bid(Dve)), (C, bid(Ena)), (D, Pass), (A, Pass)],
    );
    assert!(hold_ena(&game), "B is above C");

    // Nobody is above A: after A holds dve, B may not hold it back.
    let mut game = new_game();
    auction(
        &mut game,
        &[(B, bid(Dve)), (C, Pass), (D, Pass), (A, bid(Dve))],
    );
    assert_eq!(game.to_act(), Some(B));
    assert!(!may_hold(&game), "B is below A");

    // Priority follows the dealer: with seat 1 dealing, seat 2 is forehand
    // and may hold a bid of seat 3; seat 0 may not.
    let seat = |i| Seat::new(i).unwrap();
    let mut game = Game::new(&Deal::from_seed(1), seat(1));
    auction(&mut game, &[(seat(3), bid(Dve))]);
    assert_eq!(game.to_act(), Some(seat(0)));
    assert!(!may_hold(&game));
    auction(&mut game, &[(seat(0), Pass), (seat(1), Pass)]);
    assert_eq!(game.to_act(), Some(seat(2)));
    assert!(may_hold(&game));
}

#[test]
fn c06_a_player_who_passed_gets_no_further_turn() {
    let mut game = new_game();
    // C passes at once. B, D and A go on for three more rounds.
    let steps = [
        (B, bid(Dve)),
        (C, Pass),
        (D, bid(Ena)),
        (A, bid(Ena)),
        (B, bid(SoloTri)),
        (D, bid(SoloDve)),
        (A, bid(SoloDve)),
        (B, bid(SoloEna)),
        (D, Pass),
        (A, bid(SoloEna)),
        (B, Pass),
    ];
    for (done, &(seat, action)) in steps.iter().enumerate() {
        let public = game.public();
        if done >= 2 {
            assert!(public.has_passed(C));
            assert_ne!(game.to_act(), Some(C), "C passed and is skipped");
        }
        if done >= 9 {
            assert!(public.has_passed(D));
            assert_ne!(game.to_act(), Some(D), "D passed and is skipped");
        }
        act(&mut game, seat, action);
    }
    assert_eq!(game.phase(), Phase::ContractChoice);
    assert_eq!(game.public().declarer(), Some(A));
    let public = game.public();
    assert_eq!(
        [A, B, C, D].map(|seat| public.has_passed(seat)),
        [false, true, true, true]
    );
}

/// An auction, its declarer and the winning bid.
type Case<'a> = (&'a [(Seat, Action)], Seat, Contract);

#[test]
fn c07_auction_ends_after_three_passes() {
    // (the auction, the declarer, the winning bid)
    let cases: [Case<'_>; 4] = [
        (&[(B, bid(Dve)), (C, Pass), (D, Pass), (A, Pass)], B, Dve),
        (&[(B, Pass), (C, bid(Ena)), (D, Pass), (A, Pass)], C, Ena),
        (
            &[(B, Pass), (C, Pass), (D, bid(SoloTri)), (A, Pass)],
            D,
            SoloTri,
        ),
        (
            &[
                (B, bid(Dve)),
                (C, bid(Ena)),
                (D, Pass),
                (A, Pass),
                (B, Pass),
            ],
            C,
            Ena,
        ),
    ];
    for (steps, declarer, winning) in cases {
        let mut game = new_game();
        // Not over before the third pass.
        let (last, before) = steps.split_last().unwrap();
        auction(&mut game, before);
        assert_eq!(game.phase(), Phase::Bidding);
        assert_eq!(game.public().declarer(), None);
        auction(&mut game, &[*last]);

        assert_eq!(game.phase(), Phase::ContractChoice);
        let public = game.public();
        assert_eq!(public.declarer(), Some(declarer));
        assert_eq!(public.winning_bid(), Some(winning));
        assert_eq!(game.to_act(), Some(declarer));
        let passed = [A, B, C, D]
            .iter()
            .filter(|&&s| public.has_passed(s))
            .count();
        assert_eq!(passed, 3);
        assert!(!public.has_passed(declarer));
        // The bids stay on record, in order.
        assert_eq!(public.bids().len(), steps.len());
        for (event, &(seat, action)) in public.bids().iter().zip(steps) {
            assert_eq!(event.seat, seat);
            let expected = match action {
                Action::Bid(contract) => Some(contract),
                _ => None,
            };
            assert_eq!(event.bid, expected);
        }
    }
}

#[test]
fn c08_forehand_may_equal_raise_or_pass_after_one_bid() {
    for b_bid in [Dve, Ena, SoloTri, SoloDve, SoloEna] {
        let mut game = new_game();
        auction(&mut game, &[(B, bid(b_bid)), (C, Pass), (D, Pass)]);
        assert_eq!(game.to_act(), Some(A));
        // Pass, the same bid, and every higher normal contract.
        let from_bid: Vec<Contract> = NORMAL.into_iter().filter(|&c| c >= b_bid).collect();
        assert_eq!(legal(&game), pass_and(&from_bid));

        // If A passes, B is the declarer.
        auction(&mut game, &[(A, Pass)]);
        assert_eq!(game.phase(), Phase::ContractChoice);
        assert_eq!(game.public().declarer(), Some(B));
        assert_eq!(game.public().winning_bid(), Some(b_bid));
    }

    // If A equals the bid, B must go higher or pass.
    let mut game = new_game();
    auction(
        &mut game,
        &[(B, bid(Ena)), (C, Pass), (D, Pass), (A, bid(Ena))],
    );
    assert_eq!(game.phase(), Phase::Bidding);
    assert_eq!(game.to_act(), Some(B));
    assert_eq!(legal(&game), pass_and(&[SoloTri, SoloDve, SoloEna]));
    auction(&mut game, &[(B, Pass)]);
    assert_eq!(game.public().declarer(), Some(A));
    assert_eq!(game.public().winning_bid(), Some(Ena));
}

#[test]
fn c09_b_holds_c_bid_then_c_must_go_higher_or_pass() {
    let mut game = new_game();
    auction(
        &mut game,
        &[(B, bid(Dve)), (C, bid(Ena)), (D, Pass), (A, Pass)],
    );
    // B has priority over C and may bid ena.
    assert_eq!(game.to_act(), Some(B));
    assert_eq!(legal(&game), pass_and(&[Ena, SoloTri, SoloDve, SoloEna]));
    auction(&mut game, &[(B, bid(Ena))]);
    assert_eq!(game.public().highest_bid(), Some((B, Ena)));

    // C must then go higher or pass.
    assert_eq!(game.to_act(), Some(C));
    assert_eq!(legal(&game), pass_and(&[SoloTri, SoloDve, SoloEna]));
    auction(&mut game, &[(C, Pass)]);
    assert_eq!(game.phase(), Phase::ContractChoice);
    assert_eq!(game.public().declarer(), Some(B));
    assert_eq!(game.public().winning_bid(), Some(Ena));
}

#[test]
fn c10_three_passes_forehand_must_name_tri_or_higher() {
    for named in NORMAL {
        let mut game = new_game();
        auction(&mut game, &[(B, Pass), (C, Pass), (D, Pass)]);
        // Three have passed, but A has not spoken: the auction is not over.
        assert_eq!(game.phase(), Phase::Bidding);
        assert_eq!(game.to_act(), Some(A));
        assert_eq!(
            legal(&game),
            bids(&[Tri, Dve, Ena, SoloTri, SoloDve, SoloEna])
        );
        // Pass is not legal, klop is not legal (klop joins in flow C).
        assert!(!game.legal_actions().contains(Pass));
        assert!(!game.legal_actions().contains(bid(Contract::Klop)));
        let before = game.clone();
        assert_eq!(
            game.apply(Pass),
            Err(tarok_engine::GameError::NotLegal(Pass))
        );
        assert_eq!(game, before);

        auction(&mut game, &[(A, bid(named))]);
        assert_eq!(game.phase(), Phase::ContractChoice);
        assert_eq!(game.public().declarer(), Some(A));
        assert_eq!(game.public().winning_bid(), Some(named));
    }
}

#[test]
fn c11_declarer_names_the_winning_bid_or_a_higher_contract() {
    // Every winning bid of B.
    for winning in [Dve, Ena, SoloTri, SoloDve, SoloEna] {
        let mut game = new_game();
        auction(
            &mut game,
            &[(B, bid(winning)), (C, Pass), (D, Pass), (A, Pass)],
        );
        assert_eq!(game.phase(), Phase::ContractChoice);
        assert_eq!(game.to_act(), Some(B));
        assert_eq!(game.public().contract(), None);
        let allowed: Vec<Contract> = NORMAL.into_iter().filter(|&c| c >= winning).collect();
        assert_eq!(legal(&game), bids(&allowed));

        // A lower contract is refused and changes nothing.
        for lower in NORMAL.into_iter().filter(|&c| c < winning) {
            let before = game.clone();
            assert_eq!(
                game.apply(bid(lower)),
                Err(tarok_engine::GameError::NotLegal(bid(lower)))
            );
            assert_eq!(game, before);
        }
        // Each allowed contract is accepted.
        for contract in allowed {
            let mut game = game.clone();
            act(&mut game, B, bid(contract));
            assert_eq!(game.public().contract(), Some(contract));
            assert_eq!(game.public().winning_bid(), Some(winning));
            assert_ne!(game.phase(), Phase::ContractChoice);
        }
    }

    // Forehand after three passes: named tri, may still play anything from
    // tri up.
    let mut game = new_game();
    auction(&mut game, &[(B, Pass), (C, Pass), (D, Pass), (A, bid(Tri))]);
    assert_eq!(legal(&game), bids(&NORMAL));
}

// ---------------------------------------------------------------------
// Every auction (review of spec 005)
// ---------------------------------------------------------------------

/// A model of the auction written from criteria 1-11, apart from the
/// engine. It keeps the list of calls and works everything else out from
/// that list each time.
///
/// Seats are numbered by their distance from forehand: 0 is forehand (A),
/// 1 is B, 2 is C, 3 is D (the dealer). A lower number is a higher
/// priority (criterion 5).
#[derive(Clone)]
struct Model {
    /// The contracts every seat may bid, low to high.
    open: Vec<Contract>,
    /// The contracts only forehand may name, and only when the three
    /// others have passed (criteria 2 and 10).
    forehand_only: Vec<Contract>,
    /// The calls so far: the seat and its bid (`None` is a pass).
    calls: Vec<(usize, Option<Contract>)>,
}

impl Model {
    fn has_passed(&self, seat: usize) -> bool {
        self.calls.contains(&(seat, None))
    }

    fn passes(&self) -> usize {
        (0..4).filter(|&seat| self.has_passed(seat)).count()
    }

    /// The last bid made. Bids only go up, so it is the highest.
    fn highest(&self) -> Option<(usize, Contract)> {
        self.calls
            .iter()
            .rev()
            .find_map(|&(seat, bid)| bid.map(|contract| (seat, contract)))
    }

    /// Criterion 7: three have passed and the fourth has bid.
    fn is_over(&self) -> bool {
        self.passes() == 3 && self.highest().is_some()
    }

    /// Criteria 1 and 6: B first, then C, D, A, then round the seats that
    /// have not passed.
    fn speaker(&self) -> usize {
        let mut seat = match self.calls.last() {
            None => 1,
            Some(&(last, _)) => (last + 1) % 4,
        };
        while self.has_passed(seat) {
            seat = (seat + 1) % 4;
        }
        seat
    }

    /// The legal calls of the speaker, the pass first, then low to high.
    fn legal(&self) -> Vec<Option<Contract>> {
        let speaker = self.speaker();
        match self.highest() {
            // Nobody has bid. Forehand speaks only after three passes and
            // must name a contract (criterion 10). The others may pass or
            // bid any open contract (criterion 2).
            None if speaker == 0 => self
                .forehand_only
                .iter()
                .chain(&self.open)
                .map(|&contract| Some(contract))
                .collect(),
            None => std::iter::once(None)
                .chain(self.open.iter().map(|&contract| Some(contract)))
                .collect(),
            // Criteria 3 and 4: higher, or the same from a higher priority.
            Some((holder, held)) => std::iter::once(None)
                .chain(
                    self.open
                        .iter()
                        .filter(|&&contract| {
                            contract > held || (contract == held && speaker < holder)
                        })
                        .map(|&contract| Some(contract)),
                )
                .collect(),
        }
    }

    /// Criterion 11: the winning bid or any higher contract.
    fn contract_choices(&self) -> Vec<Contract> {
        let (_, winning) = self.highest().expect("the auction is over");
        self.forehand_only
            .iter()
            .chain(&self.open)
            .copied()
            .filter(|&contract| contract >= winning)
            .collect()
    }

    fn after(&self, call: Option<Contract>) -> Model {
        let mut next = self.clone();
        next.calls.push((self.speaker(), call));
        next
    }

    /// The number of finished auctions from here and the length of the
    /// longest one.
    fn count_and_longest(&self) -> (u64, usize) {
        if self.is_over() {
            return (1, self.calls.len());
        }
        let mut count = 0;
        let mut longest = 0;
        for call in self.legal() {
            let (below, length) = self.after(call).count_and_longest();
            count += below;
            longest = longest.max(length);
        }
        (count, longest)
    }
}

/// The model for this slice: dve to solo ena for all, tri for forehand.
fn slice_model() -> Model {
    Model {
        open: vec![Dve, Ena, SoloTri, SoloDve, SoloEna],
        forehand_only: vec![Tri],
        calls: Vec::new(),
    }
}

fn as_action(call: Option<Contract>) -> Action {
    match call {
        None => Pass,
        Some(contract) => bid(contract),
    }
}

/// Walks every auction from this node. At each node the engine must agree
/// with the model on the seat to speak and on the legal calls; at the end
/// of an auction on the declarer, the winning bid and the contracts the
/// declarer may name. Returns the number of auctions and the longest one.
fn walk(game: &Game, model: &Model, forehand: Seat) -> (u64, usize) {
    let seat_of = |distance: usize| forehand.plus(distance);
    let public = game.public();
    // The record of the calls is the same on both sides.
    assert_eq!(public.bids().len(), model.calls.len());
    for (event, &(seat, call)) in public.bids().iter().zip(&model.calls) {
        assert_eq!((event.seat, event.bid), (seat_of(seat), call));
    }
    for seat in 0..4 {
        assert_eq!(public.has_passed(seat_of(seat)), model.has_passed(seat));
    }

    if model.is_over() {
        let (winner, winning) = model.highest().unwrap();
        assert_eq!(
            game.phase(),
            Phase::ContractChoice,
            "after {:?}",
            model.calls
        );
        assert_eq!(public.declarer(), Some(seat_of(winner)));
        assert_eq!(public.winning_bid(), Some(winning));
        assert_eq!(game.to_act(), Some(seat_of(winner)));
        assert_eq!(legal(game), bids(&model.contract_choices()));
        return (1, model.calls.len());
    }

    assert_eq!(game.phase(), Phase::Bidding, "after {:?}", model.calls);
    assert_eq!(public.declarer(), None);
    assert_eq!(game.to_act(), Some(seat_of(model.speaker())));
    let expected: Vec<Action> = model.legal().into_iter().map(as_action).collect();
    assert_eq!(legal(game), expected, "after {:?}", model.calls);
    assert!(!expected.is_empty());

    let mut count = 0;
    let mut longest = 0;
    for call in model.legal() {
        let mut next = game.clone();
        next.apply(as_action(call))
            .expect("a legal call is accepted");
        let (below, length) = walk(&next, &model.after(call), forehand);
        count += below;
        longest = longest.max(length);
    }
    (count, longest)
}

/// Not one numbered criterion: criteria 1-11 on every auction there is,
/// for each of the four dealers.
#[test]
fn every_auction_agrees_with_a_model_of_criteria_1_to_11() {
    for dealer in Seat::ALL {
        let game = Game::new(&Deal::from_seed(1), dealer);
        let (count, longest) = walk(&game, &slice_model(), dealer.next());
        // 1,297 auctions; the longest has 13 calls.
        assert_eq!(count, 1_297, "dealer {}", dealer.index());
        assert_eq!(longest, 13, "dealer {}", dealer.index());
    }
}

/// The longest auctions, by enumeration in the model.
#[test]
fn longest_auction_of_this_slice_and_of_the_full_ladder() {
    // This slice: 13 calls.
    assert_eq!(slice_model().count_and_longest(), (1_297, 13));

    // One of the longest, written out: B and A raise each other through
    // all five contracts, A holding each time.
    let longest = [
        (B, bid(Dve)),
        (C, Pass),
        (D, Pass),
        (A, bid(Dve)),
        (B, bid(Ena)),
        (A, bid(Ena)),
        (B, bid(SoloTri)),
        (A, bid(SoloTri)),
        (B, bid(SoloDve)),
        (A, bid(SoloDve)),
        (B, bid(SoloEna)),
        (A, bid(SoloEna)),
        (B, Pass),
    ];
    assert_eq!(longest.len(), 13);
    let mut game = new_game();
    auction(&mut game, &longest);
    assert_eq!(game.phase(), Phase::ContractChoice);
    assert_eq!(game.public().declarer(), Some(A));
    assert_eq!(game.public().bids().len(), 13);

    // The full ladder of the Default preset, for the token limit of the
    // encoding (spec 006): dve, ena, the three solos, berač, solo brez and
    // valat for all; klop and tri for forehand after three passes. Odprti
    // berač and barvni valat are off. The same bidding rules are assumed
    // for the contracts of flow C; that spec may change them.
    let full = Model {
        open: vec![
            Dve,
            Ena,
            SoloTri,
            SoloDve,
            SoloEna,
            Contract::Berac,
            Contract::SoloBrez,
            Contract::Valat,
        ],
        forehand_only: vec![Contract::Klop, Tri],
        calls: Vec::new(),
    };
    let (count, longest) = full.count_and_longest();
    assert_eq!(
        (count, longest),
        FULL_LADDER,
        "auctions and longest auction"
    );
}

/// The number of auctions and the length of the longest auction of the
/// full ladder, as the model counts them.
const FULL_LADDER: (u64, usize) = (36_182, 19);
