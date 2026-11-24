//! Helpers for the deal-flow tests of spec 005.
//!
//! Seats as in the spec's examples: A is forehand, then B, C, D in order of
//! play, and D deals. With seat 3 as the dealer, A is seat 0.

use tarok_engine::{
    Action, Card, CardSet, Contract, Deal, Game, Packet, Phase, Rank, Seat, Suit, Target,
};

use super::{court, kralj, pip, tarok};

pub const A: Seat = Seat::ALL[0];
pub const B: Seat = Seat::ALL[1];
pub const C: Seat = Seat::ALL[2];
pub const D: Seat = Seat::ALL[3];

/// The dealer of the spec's examples.
pub const DEALER: Seat = D;

/// No deal has this many actions (see `MAX_ACTIONS` in `game.rs`). The
/// helpers stop with a failure instead of running on if a deal does not
/// end.
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

/// The declarer takes a packet and discards the listed cards. The game is
/// left in the round of announcements (spec 009), with the declarer to
/// speak.
pub fn exchange_open(game: &mut Game, taken: u8, discards: &[Card]) {
    let declarer = game.public().declarer().expect("a declarer");
    assert_eq!(game.phase(), Phase::TalonPick);
    act(game, declarer, Action::TakePacket(packet(taken)));
    for &card in discards {
        assert_eq!(game.phase(), Phase::Discard);
        act(game, declarer, Action::Discard(card));
    }
    assert_eq!(game.phase(), Phase::Announcements, "all discards made");
    assert_eq!(game.to_act(), Some(declarer));
}

/// The declarer takes a packet and discards the listed cards; then all
/// four seats pass in the round of announcements (spec 009, criterion 5),
/// so the deal goes on as in flow A.
pub fn exchange(game: &mut Game, taken: u8, discards: &[Card]) {
    exchange_open(game, taken, discards);
    speak_or_pass(game);
}

thread_local! {
    /// The turns [`with_round`] wants spoken in the next round.
    static ROUND_SCRIPT: std::cell::RefCell<Vec<(Seat, Vec<Action>)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// Runs `build` with a script for the round of announcements. Where a
/// helper would pass the round ([`exchange`], [`exchange_back`] and
/// everything built on them), the listed turns are spoken instead: every
/// seat passes until it is the turn of the next listed seat, which takes
/// its actions and says "done". After the last listed turn all pass.
///
/// So a deal built for flow A can be played again with something said:
/// `with_round(&[(B, &[Action::Kontra(Target::Game)])], || built_deal())`.
/// The script is for one round: `build` must play exactly one deal.
pub fn with_round<T>(turns: &[(Seat, &[Action])], build: impl FnOnce() -> T) -> T {
    let script: Vec<(Seat, Vec<Action>)> = turns
        .iter()
        .map(|&(seat, actions)| (seat, actions.to_vec()))
        .collect();
    ROUND_SCRIPT.with(|cell| *cell.borrow_mut() = script);
    let built = build();
    let left = ROUND_SCRIPT.with(|cell| std::mem::take(&mut *cell.borrow_mut()));
    assert!(left.is_empty(), "the round script was not used");
    built
}

/// Takes the script of [`with_round`], if there is one.
fn take_round_script() -> Vec<(Seat, Vec<Action>)> {
    ROUND_SCRIPT.with(|cell| std::mem::take(&mut *cell.borrow_mut()))
}

/// The next action of a scripted round: the next action of the first
/// listed turn if its seat is to speak, else "done". A turn whose actions
/// are all taken ends with "done" and leaves the script.
fn scripted_round_action(game: &Game, script: &mut Vec<(Seat, Vec<Action>)>) -> Action {
    match script.first_mut() {
        Some((seat, actions)) if game.to_act() == Some(*seat) => {
            if actions.is_empty() {
                script.remove(0);
                Action::Done
            } else {
                actions.remove(0)
            }
        }
        _ => Action::Done,
    }
}

/// Speaks the script of [`with_round`] if there is one, then passes the
/// round to its end.
fn speak_or_pass(game: &mut Game) {
    let mut script = take_round_script();
    let mut actions = 0;
    while game.phase() == Phase::Announcements {
        actions += 1;
        assert!(actions <= MAX_ACTIONS, "the round does not end");
        let seat = game.to_act().expect("a seat speaks");
        let action = scripted_round_action(game, &mut script);
        act(game, seat, action);
    }
    assert!(
        script.is_empty(),
        "the round ended before the turn of {:?}",
        script[0].0
    );
    assert_eq!(game.phase(), Phase::Play, "the round is over");
}

/// The cards of a talon packet.
fn packet_cards(game: &Game, taken: u8) -> Vec<Card> {
    game.public()
        .packet(packet(taken))
        .expect("the talon is shown")
        .to_vec()
}

/// The declarer takes a packet and discards the same cards again, so the
/// hand stays as dealt. The packet must hold no king and no trula card.
/// Then all four pass in the round of announcements.
pub fn exchange_back(game: &mut Game, taken: u8) {
    let cards = packet_cards(game, taken);
    exchange(game, taken, &cards);
}

/// As [`exchange_back`], but the game is left in the round of
/// announcements.
pub fn exchange_back_open(game: &mut Game, taken: u8) {
    let cards = packet_cards(game, taken);
    exchange_open(game, taken, &cards);
}

/// Every seat says "done" and nothing else until the round of
/// announcements is over. From its start that is four passes.
pub fn pass_round(game: &mut Game) {
    let mut turns = 0;
    while game.phase() == Phase::Announcements {
        turns += 1;
        assert!(turns <= MAX_ACTIONS, "the round does not end");
        game.apply(Action::Done).expect("done is always legal");
    }
    assert_eq!(game.phase(), Phase::Play, "the round is over");
}

/// One turn of the round of announcements: `seat` takes the actions in
/// order and then says "done". With no actions the turn is a pass.
pub fn say(game: &mut Game, seat: Seat, actions: &[Action]) {
    assert_eq!(game.phase(), Phase::Announcements);
    for &action in actions {
        act(game, seat, action);
        assert_eq!(
            game.to_act(),
            Some(seat),
            "the turn goes on after {action:?}"
        );
    }
    act(game, seat, Action::Done);
}

/// The kontra levels of the six targets: game, trula, kralji, kralj
/// ultimo, pagat ultimo, valat.
pub fn levels(game: &Game) -> [u8; 6] {
    Target::ALL.map(|target| game.public().kontra_level(target))
}

/// What the helpers of flow A take when they "take the lowest legal
/// action": the lowest slot, but in the round of announcements "done", so
/// that every seat passes there (spec 009, criterion 5).
pub fn lowest_action(game: &Game) -> Action {
    if game.phase() == Phase::Announcements {
        return Action::Done;
    }
    game.legal_actions().iter().next().expect("a legal action")
}

/// Takes the lowest legal action (lowest slot; "done" in the round of
/// announcements) until `stop` holds or the deal is over.
pub fn run_until(game: &mut Game, stop: impl Fn(&Game) -> bool) {
    let mut actions = 0;
    while game.phase() != Phase::Over && !stop(game) {
        actions += 1;
        assert!(actions <= MAX_ACTIONS, "the deal does not end");
        let action = lowest_action(game);
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
            None => lowest_action(game),
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
/// In the round of announcements every seat passes (spec 009, criterion
/// 5): the deal is one of flow A. A pass uses no choice, so a seed and its
/// choices give the cards and the plays they gave before the round
/// existed. Inside [`with_round`] the script is spoken in place of the
/// passes; the cards and the plays stay the same. [`play_random_b`] speaks
/// at random in the round.
///
/// `before` sees the game before each action.
pub fn play_random(seed: u64, dealer: Seat, choices: &[u8], before: impl FnMut(&Game)) -> Game {
    play_random_with(seed, dealer, choices, false, before)
}

/// As [`play_random`], with announcements and kontras (flow B). In the
/// round a choice that is a multiple of 3 says "done"; any other choice
/// picks one of the other legal actions, if there is one.
pub fn play_random_b(seed: u64, dealer: Seat, choices: &[u8], before: impl FnMut(&Game)) -> Game {
    play_random_with(seed, dealer, choices, true, before)
}

fn play_random_with(
    seed: u64,
    dealer: Seat,
    choices: &[u8],
    speak: bool,
    mut before: impl FnMut(&Game),
) -> Game {
    let mut game = Game::from_seed(seed, dealer);
    let mut script = if speak {
        Vec::new()
    } else {
        take_round_script()
    };
    let mut step = 0;
    let mut actions = 0;
    while game.phase() != Phase::Over {
        assert!(actions < 2 * MAX_ACTIONS, "the deal does not end");
        actions += 1;
        before(&game);
        let legal = game.legal_actions();
        assert!(!legal.is_empty(), "no legal action in {:?}", game.phase());
        if game.phase() == Phase::Announcements {
            assert!(legal.contains(Action::Done), "done is always legal");
            let mut action = scripted_round_action(&game, &mut script);
            if speak {
                let choice = u32::from(choices[step % choices.len()]);
                step += 1;
                // "Done" has the highest slot of the round: the others
                // come first in slot order.
                let others = legal.len() - 1;
                if choice % 3 != 0 && others > 0 {
                    action = legal.nth((choice / 3) % others).expect("in range");
                }
            }
            game.apply(action).expect("a legal action is accepted");
            continue;
        }
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
    assert!(script.is_empty(), "the round script was not used up");
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

/// As [`seeded_game`], with announcements and kontras: a random deal of
/// flow B that depends on one number only.
pub fn seeded_game_b(seed: u64) -> Game {
    seeded_game_b_with(seed, |_| {})
}

/// As [`seeded_game_b`]; `before` sees the game before each action.
pub fn seeded_game_b_with(seed: u64, before: impl FnMut(&Game)) -> Game {
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let choices: Vec<u8> = (0..191)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 24) as u8
        })
        .collect();
    let dealer = Seat::ALL[(seed % 4) as usize];
    play_random_b(seed, dealer, &choices, before)
}

/// The deal of the tests of the round of announcements (spec 009).
///
/// - A: škis, mond, taroks II-XI.
/// - B: pagat, taroks XII-XX, srce fant, srce kaval.
/// - C: srce kralj, srce dama; karo 2, 1, fant, kaval, dama, kralj; pik
///   7, 8, 9, 10.
/// - D: pik fant, kaval, dama, kralj and the eight križ.
/// - Talon: srce 4, 3, 2, 1, karo 4, 3.
///
/// `king` holds the srce kralj. That is C as listed. For another seat the
/// king changes places with one card of that seat: the tarok II (A), the
/// srce kaval (B) or the križ 7 (D).
pub fn round_deal(king: Seat) -> Deal {
    let srce = |number| pip(Suit::Srce, number);
    let talon = [
        srce(4),
        srce(3),
        srce(2),
        srce(1),
        pip(Suit::Karo, 4),
        pip(Suit::Karo, 3),
    ];
    let base = deal_with(
        [
            &[Card::SKIS, Card::MOND],
            &[Card::PAGAT],
            &[kralj(Suit::Srce)],
            &[],
        ],
        &talon,
    );
    let mut hands = base.hands();
    // What the listing above says, so a change of `deal_with` shows here.
    assert!(hands[A.index()].contains(tarok(2)) && hands[A.index()].contains(tarok(11)));
    assert!(hands[B.index()].contains(court(Suit::Srce, Rank::Kaval)));
    assert!(hands[C.index()].contains(kralj(Suit::Karo)));
    assert!(hands[D.index()].contains(pip(Suit::Kriz, 7)));
    assert!((hands[C.index()] | hands[D.index()]).is_disjoint(CardSet::TAROKS));
    let other = match king.index() {
        0 => Some(tarok(2)),
        1 => Some(court(Suit::Srce, Rank::Kaval)),
        3 => Some(pip(Suit::Kriz, 7)),
        _ => None,
    };
    if let Some(card) = other {
        hands[C.index()].remove(kralj(Suit::Srce));
        hands[C.index()].insert(card);
        hands[king.index()].remove(card);
        hands[king.index()].insert(kralj(Suit::Srce));
    }
    Deal::new(hands, talon).expect("a whole deal")
}

/// A plays `contract` on [`round_deal`]: it calls srce where a king is
/// called, takes the first packet and puts the same cards down again. The
/// game is left at the start of the round of announcements, A to speak.
pub fn round_game(king: Seat, contract: Contract) -> Game {
    let mut game = game(&round_deal(king));
    declare(&mut game, A, contract);
    if game.phase() == Phase::KingCall {
        act(&mut game, A, Action::CallKing(Suit::Srce));
    }
    exchange_back_open(&mut game, 0);
    game
}

/// For each seat (A, B, C, D): is `action` legal at that seat's first turn
/// of the round, when every seat before it passes? `game` is at the start
/// of the round and is not changed.
pub fn may_by_seat(game: &Game, action: Action) -> [bool; 4] {
    let mut game = game.clone();
    let mut may = [false; 4];
    for _ in 0..4 {
        assert_eq!(game.phase(), Phase::Announcements);
        let seat = game.to_act().expect("a seat speaks");
        may[seat.index()] = game.legal_actions().contains(action);
        // The last pass ends the round; the game is a copy.
        game.apply(Action::Done).expect("done is always legal");
    }
    may
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
