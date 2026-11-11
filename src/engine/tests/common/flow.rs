//! Helpers for the deal-flow tests of spec 005.
//!
//! Seats as in the spec's examples: A is forehand, then B, C, D in order of
//! play, and D deals. With seat 3 as the dealer, A is seat 0.

use tarok_engine::{Action, Card, CardSet, Contract, Deal, Game, Packet, Phase, Rank, Seat, Suit};

use super::{court, kralj, pip, tarok};

pub const A: Seat = Seat::ALL[0];
pub const B: Seat = Seat::ALL[1];
pub const C: Seat = Seat::ALL[2];
pub const D: Seat = Seat::ALL[3];

/// The dealer of the spec's examples.
pub const DEALER: Seat = D;

/// No deal has this many actions (the longest has 67). The helpers stop
/// with a failure instead of running on if a deal does not end.
pub const MAX_ACTIONS: usize = 200;

pub const NORMAL: [Contract; 6] = [
    Contract::Tri,
    Contract::Dve,
    Contract::Ena,
    Contract::SoloTri,
    Contract::SoloDve,
    Contract::SoloEna,
];

pub fn packet(index: u8) -> Packet {
    Packet::new(index).expect("packet index 0-5")
}

/// `Action::Bid` for each contract.
pub fn bids(contracts: &[Contract]) -> Vec<Action> {
    contracts.iter().map(|&c| Action::Bid(c)).collect()
}

/// A deal from the cards named for each seat (A, B, C, D) and for the talon.
/// The cards not named are dealt in index order: first to A until A holds
/// twelve, then to B, C, D, and the last ones to the talon. The named talon
/// cards lie first, in the order given.
pub fn deal_with(fixed: [&[Card]; 4], talon: &[Card]) -> Deal {
    let mut hands: [Vec<Card>; 4] = fixed.map(|cards| cards.to_vec());
    let mut talon = talon.to_vec();
    let mut named = CardSet::EMPTY;
    for &card in hands.iter().flatten().chain(talon.iter()) {
        assert!(named.insert(card), "{card} is named twice");
    }
    for card in Card::all().filter(|&c| !named.contains(c)) {
        match hands.iter_mut().find(|hand| hand.len() < 12) {
            Some(hand) => hand.push(card),
            None => talon.push(card),
        }
    }
    let hands = hands.map(|hand| hand.into_iter().collect::<CardSet>());
    let talon: [Card; 6] = talon.try_into().expect("six talon cards");
    Deal::new(hands, talon).expect("a whole deal")
}

/// A deal with every card named. Panics if a card is missing or named twice.
pub fn whole_deal(hands: [&[Card]; 4], talon: [Card; 6]) -> Deal {
    for (index, hand) in hands.iter().enumerate() {
        assert_eq!(hand.len(), 12, "hand of seat {index}");
    }
    deal_with(hands, &talon)
}

/// A game with D (seat 3) as the dealer, so A (seat 0) is forehand.
pub fn game(deal: &Deal) -> Game {
    Game::new(deal, DEALER)
}

/// The legal actions in slot order.
pub fn legal(game: &Game) -> Vec<Action> {
    game.legal_actions().iter().collect()
}

/// `seat` takes `action`. Panics if it is not that seat's turn or the action
/// is refused.
pub fn act(game: &mut Game, seat: Seat, action: Action) {
    assert_eq!(game.to_act(), Some(seat), "whose turn before {action:?}");
    if let Err(error) = game.apply(action) {
        panic!("{action:?} by seat {} was refused: {error}", seat.index());
    }
}

/// Runs the auction so that `declarer` wins it with one bid and everyone
/// else passes, then names `contract`. A declarer who is forehand bids the
/// contract itself (the others have passed, so tri is allowed); any other
/// declarer bids dve.
pub fn declare(game: &mut Game, declarer: Seat, contract: Contract) {
    let forehand = game.public().forehand();
    let mut calls = 0;
    while game.phase() == Phase::Bidding {
        calls += 1;
        assert!(calls <= MAX_ACTIONS, "the auction does not end");
        let seat = game.to_act().expect("someone bids");
        let action = if seat != declarer {
            Action::Pass
        } else if seat == forehand {
            Action::Bid(contract)
        } else {
            Action::Bid(Contract::Dve)
        };
        act(game, seat, action);
    }
    assert_eq!(game.phase(), Phase::ContractChoice);
    assert_eq!(game.public().declarer(), Some(declarer));
    act(game, declarer, Action::Bid(contract));
}

/// The declarer takes a packet and discards the listed cards.
pub fn exchange(game: &mut Game, taken: u8, discards: &[Card]) {
    let declarer = game.public().declarer().expect("a declarer");
    assert_eq!(game.phase(), Phase::TalonPick);
    act(game, declarer, Action::TakePacket(packet(taken)));
    for &card in discards {
        assert_eq!(game.phase(), Phase::Discard);
        act(game, declarer, Action::Discard(card));
    }
    assert_eq!(game.phase(), Phase::Play, "all discards made");
}

/// The declarer takes a packet and discards the same cards again, so the
/// hand stays as dealt. The packet must hold no king and no trula card.
pub fn exchange_back(game: &mut Game, taken: u8) {
    let cards: Vec<Card> = game
        .public()
        .packet(packet(taken))
        .expect("the talon is shown")
        .to_vec();
    exchange(game, taken, &cards);
}

/// Takes the lowest legal action (lowest slot) until `stop` holds or the
/// deal is over.
pub fn run_until(game: &mut Game, stop: impl Fn(&Game) -> bool) {
    let mut actions = 0;
    while game.phase() != Phase::Over && !stop(game) {
        actions += 1;
        assert!(actions <= MAX_ACTIONS, "the deal does not end");
        let action = game.legal_actions().iter().next().expect("a legal action");
        game.apply(action).expect("a legal action is accepted");
    }
}

/// Takes the lowest legal action until the deal is over.
pub fn finish(game: &mut Game) {
    run_until(game, |_| false);
}

/// Plays the cards. Each seat (A, B, C, D) has a list: at its turn the seat
/// plays the next card of its list, which must be legal. A seat whose list
/// is used up plays its lowest legal card. Stops when the deal is over.
pub fn play_scripted(game: &mut Game, scripts: [&[Card]; 4]) {
    assert_eq!(game.phase(), Phase::Play);
    let mut next = [0usize; 4];
    while game.phase() == Phase::Play {
        let seat = game.to_act().expect("someone plays");
        let script = scripts[seat.index()];
        let action = match script.get(next[seat.index()]) {
            Some(&card) => {
                next[seat.index()] += 1;
                Action::Play(card)
            }
            None => game.legal_actions().iter().next().expect("a legal card"),
        };
        if let Err(error) = game.apply(action) {
            let trick = game.public().tricks().len() + 1;
            panic!(
                "trick {trick}: {action:?} by seat {} was refused: {error}",
                seat.index()
            );
        }
    }
    for (index, script) in scripts.iter().enumerate() {
        assert_eq!(next[index], script.len(), "unused script of seat {index}");
    }
}

/// A random deal played with the given choices. Choice `i` picks action
/// `i` from the legal actions in slot order, modulo their number.
///
/// One exception, so that low contracts come up as often as high ones: in
/// the bidding and in the choice of the contract the lowest legal action
/// is taken three times out of four (unless the choice is a multiple of
/// 4). That is a pass where a pass is legal, tri for forehand after three
/// passes, and the winning bid in the contract choice. With plain uniform
/// choices nearly every auction climbs to solo ena and partner games
/// almost never happen.
///
/// `before` sees the game before each action.
pub fn play_random(seed: u64, dealer: Seat, choices: &[u8], mut before: impl FnMut(&Game)) -> Game {
    let mut game = Game::from_seed(seed, dealer);
    let mut step = 0;
    while game.phase() != Phase::Over {
        assert!(step < MAX_ACTIONS, "the deal does not end");
        before(&game);
        let legal = game.legal_actions();
        assert!(!legal.is_empty(), "no legal action in {:?}", game.phase());
        let choice = u32::from(choices[step % choices.len()]);
        let naming = matches!(game.phase(), Phase::Bidding | Phase::ContractChoice);
        let index = match (naming, choice % 4) {
            (false, _) => choice % legal.len(),
            (true, 0) => (choice / 4) % legal.len(),
            (true, _) => 0,
        };
        let action = legal.nth(index).expect("in range");
        game.apply(action).expect("a legal action is accepted");
        step += 1;
    }
    game
}

/// A random deal that depends on one number only. Seat `seed mod 4` deals
/// the cards of `seed`, and the choices come from a small generator of
/// their own (xorshift) started from the seed. Used where a test needs
/// fixed deals: the same seed is the same deal on every run.
pub fn seeded_game(seed: u64) -> Game {
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let choices: Vec<u8> = (0..96)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 24) as u8
        })
        .collect();
    let dealer = Seat::ALL[(seed % 4) as usize];
    play_random(seed, dealer, &choices, |_| {})
}

/// The srce kralj lies in the first packet of the talon and the mond in the
/// second: srce kralj, karo 4, karo 3 | mond, karo 2, karo 1.
///
/// A holds taroks I-XII. With `b_void` false, B, C and D all hold a srce,
/// so a led srce kralj wins. With `b_void` true, B holds no srce and trumps.
pub fn king_in_talon_deal(b_void: bool) -> Deal {
    let talon = [
        kralj(Suit::Srce),
        pip(Suit::Karo, 4),
        pip(Suit::Karo, 3),
        Card::MOND,
        pip(Suit::Karo, 2),
        pip(Suit::Karo, 1),
    ];
    let srce = |n| pip(Suit::Srce, n);
    if b_void {
        let c = [srce(4), srce(3), srce(1), court(Suit::Srce, Rank::Fant)];
        let d = [
            srce(2),
            court(Suit::Srce, Rank::Kaval),
            court(Suit::Srce, Rank::Dama),
        ];
        deal_with([&[], &[], &c, &d], &talon)
    } else {
        deal_with([&[], &[srce(4)], &[srce(3)], &[srce(2)]], &talon)
    }
}

pub const REST_WITHOUT_KING: [Card; 3] = [
    Card::MOND,
    Card::pip(Suit::Karo, 2).unwrap(),
    Card::pip(Suit::Karo, 1).unwrap(),
];

/// A plays tri, calls srce, takes the king's packet, discards karo 4,
/// karo 3 and tarok II, and leads the king to the first trick.
pub fn king_taken_and_led(b_void: bool) -> Game {
    let mut game = game(&king_in_talon_deal(b_void));
    declare(&mut game, A, Contract::Tri);
    act(&mut game, A, Action::CallKing(Suit::Srce));
    exchange(
        &mut game,
        0,
        &[pip(Suit::Karo, 4), pip(Suit::Karo, 3), tarok(2)],
    );
    assert!(game.private(A).hand().contains(kralj(Suit::Srce)));
    play_scripted(&mut game, [&[kralj(Suit::Srce)], &[], &[], &[]]);
    game
}
