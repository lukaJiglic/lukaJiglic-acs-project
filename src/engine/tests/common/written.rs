//! Three whole deals written out by hand (spec 005, criterion 45).
//!
//! Hands, bids, discards and all 48 plays are chosen here. The expected
//! points and entries were worked out on paper from the listed tricks and
//! stand next to each deal. Card values: kralj 5, dama 4, kaval 3, fant 2,
//! škis, mond and pagat 5, every other card 1; a pile counts the sum of its
//! values minus 2 per three cards and minus 1 for one or two cards left.
//!
//! Seats: A is forehand (seat 0), then B, C, D; D (seat 3) deals.

use tarok_engine::Contract::{Dve, Ena, SoloTri, Tri};
use tarok_engine::{Action, Action::Pass, Card, Game, Phase, Rank, Seat, Suit};

use super::flow::{A, B, C, D, act, exchange, game, whole_deal};
use super::{court, pip, tarok};

const S: Card = Card::SKIS;

fn t(number: u8) -> Card {
    tarok(number)
}
fn srce(number: u8) -> Card {
    pip(Suit::Srce, number)
}
fn karo(number: u8) -> Card {
    pip(Suit::Karo, number)
}
fn pik(number: u8) -> Card {
    pip(Suit::Pik, number)
}
fn kriz(number: u8) -> Card {
    pip(Suit::Kriz, number)
}
fn fant(suit: Suit) -> Card {
    court(suit, Rank::Fant)
}
fn kaval(suit: Suit) -> Card {
    court(suit, Rank::Kaval)
}
fn dama(suit: Suit) -> Card {
    court(suit, Rank::Dama)
}
fn kralj(suit: Suit) -> Card {
    court(suit, Rank::Kralj)
}

use Suit::{Karo, Kriz, Pik, Srce};

/// One trick as written: the seat that leads and the four cards in the
/// order of play.
pub type WrittenTrick = (Seat, [Card; 4]);

/// Plays the twelve written tricks. Each card must be accepted, the leader
/// must be the one written, and the deal must be over at the end.
fn play_written(game: &mut Game, tricks: &[WrittenTrick; 12]) {
    assert_eq!(game.phase(), Phase::Play);
    for (number, (leader, cards)) in tricks.iter().enumerate() {
        assert_eq!(
            game.public().leader(),
            Some(*leader),
            "leader of trick {}",
            number + 1
        );
        for (position, &card) in cards.iter().enumerate() {
            let seat = leader.plus(position);
            assert_eq!(game.to_act(), Some(seat));
            if let Err(error) = game.apply(Action::Play(card)) {
                panic!(
                    "trick {}: {card} by seat {} was refused: {error}",
                    number + 1,
                    seat.index()
                );
            }
        }
    }
    assert_eq!(game.phase(), Phase::Over);
}

/// What a written deal must give.
pub struct Expected {
    /// The seat that wins each trick, first to last.
    pub winners: [Seat; 12],
    pub declarer_points: u8,
    pub opponents_points: u8,
    /// The entry each seat writes: A, B, C, D.
    pub entries: [i32; 4],
}

/// Deal 1: a partner game. A plays tri, calls križ, C holds the križ kralj.
/// The declarer's side wins with 52 points and a silent trula.
///
/// Bidding: B, C and D pass; A names tri and plays tri.
/// Talon: XI, pik fant, IV | V, VI, križ fant. A takes the first packet and
/// discards pik fant, pik 7 and karo 1 (all hidden).
///
/// Tricks (leader first; the winner in capitals):
///  1. A križ 8, B križ 7, C KRIŽ KRALJ, D križ 10. The partner is public.
///  2. C pik 9, D pik 10, A IV (void in pik), B pik 8.
///  3. A ŠKIS, B II, C VIII, D III.
///  4. A XXI, B IX, C XIV, D VII.
///  5. A XX, B XVI, C XV, D XII.
///  6. A XIX, B XVII, C I (the last tarok of C), D XIII.
///  7. A SRCE KRALJ, B srce 3, C srce 2, D srce 1.
///  8. A srce 4, B SRCE DAMA, C srce kaval, D srce fant.
///  9. B KARO KRALJ, C karo 3, D karo 4, A karo dama.
/// 10. B pik kralj, C pik dama, D pik kaval, A X (void in pik).
/// 11. A XVIII, B karo 2, C križ 9, D karo fant.
/// 12. A XI, B križ dama, C karo kaval, D križ kaval.
///
/// Opponents (B and D): trick 8 = 1+4+3+2 = 10, trick 9 = 5+1+1+4 = 11, the
/// packet not taken = 1+1+2 = 4. Values 25 in 11 cards: 25 - 6 - 1 = 18.
/// Declarer's side: 70 - 18 = 52 (values 81 in 43 cards: 81 - 28 - 1).
///
/// Game: tri won, 10. Difference 52 - 35 = 17, rounded 15. Trula: škis
/// (trick 3), mond (4) and pagat (6) are all in A's tricks, +10. Kralji: the
/// karo kralj is in B's trick 9, 0. No ultimo, no mondfang.
/// Entries: A 35, C 35, B 0, D 0.
pub fn deal_1_partner() -> (Game, Expected) {
    let a = [
        S,
        t(21),
        t(20),
        t(19),
        t(18),
        t(10),
        kralj(Srce),
        srce(4),
        dama(Karo),
        karo(1),
        pik(7),
        kriz(8),
    ];
    let b = [
        t(17),
        t(16),
        t(9),
        t(2),
        dama(Srce),
        srce(3),
        kralj(Karo),
        karo(2),
        kralj(Pik),
        pik(8),
        dama(Kriz),
        kriz(7),
    ];
    let c = [
        t(15),
        t(14),
        t(8),
        t(1),
        kaval(Srce),
        srce(2),
        kaval(Karo),
        karo(3),
        dama(Pik),
        pik(9),
        kralj(Kriz),
        kriz(9),
    ];
    let d = [
        t(13),
        t(12),
        t(7),
        t(3),
        fant(Srce),
        srce(1),
        fant(Karo),
        karo(4),
        kaval(Pik),
        pik(10),
        kaval(Kriz),
        kriz(10),
    ];
    let talon = [t(11), fant(Pik), t(4), t(5), t(6), fant(Kriz)];
    let mut game = game(&whole_deal([&a, &b, &c, &d], talon));

    act(&mut game, B, Pass);
    act(&mut game, C, Pass);
    act(&mut game, D, Pass);
    act(&mut game, A, Action::Bid(Tri));
    act(&mut game, A, Action::Bid(Tri));
    act(&mut game, A, Action::CallKing(Kriz));
    exchange(&mut game, 0, &[fant(Pik), pik(7), karo(1)]);

    let tricks: [WrittenTrick; 12] = [
        (A, [kriz(8), kriz(7), kralj(Kriz), kriz(10)]),
        (C, [pik(9), pik(10), t(4), pik(8)]),
        (A, [S, t(2), t(8), t(3)]),
        (A, [t(21), t(9), t(14), t(7)]),
        (A, [t(20), t(16), t(15), t(12)]),
        (A, [t(19), t(17), t(1), t(13)]),
        (A, [kralj(Srce), srce(3), srce(2), srce(1)]),
        (A, [srce(4), dama(Srce), kaval(Srce), fant(Srce)]),
        (B, [kralj(Karo), karo(3), karo(4), dama(Karo)]),
        (B, [kralj(Pik), dama(Pik), kaval(Pik), t(10)]),
        (A, [t(18), karo(2), kriz(9), fant(Karo)]),
        (A, [t(11), dama(Kriz), kaval(Karo), kaval(Kriz)]),
    ];
    play_written(&mut game, &tricks);

    let expected = Expected {
        winners: [C, A, A, A, A, A, A, B, B, A, A, A],
        declarer_points: 52,
        opponents_points: 18,
        entries: [35, 0, 35, 0],
    };
    (game, expected)
}

/// Deal 2: a solo, lost, with a silent trula. B plays solo tri alone and
/// takes 31 points.
///
/// Bidding: B dve, C pass, D ena, A pass, B ena (B is above D and may hold),
/// D pass. B then names solo tri, a higher contract than the winning bid.
/// Talon: srce kaval, karo 3, III | križ kralj, XI, pik 7. B takes the first
/// packet and discards karo 4, karo 3 and križ 7 (all hidden).
///
/// Tricks:
///  1. A karo kaval, B V (void in karo), C karo 2, D karo 1.
///  2. B ŠKIS, C II, D VI, A IV.
///  3. B XXI, C VII, D VIII, A X.
///  4. B SRCE KRALJ, C srce 3, D srce 2, A srce 4.
///  5. B srce dama, C srce fant, D XIV (void in srce), A srce 1.
///  6. D križ fant, A križ dama, B III (void in križ), C križ 10.
///  7. B XX, C XIII, D I (the last tarok of D), A XVI.
///  8. B pik 10, C pik dama, D pik fant, A PIK KRALJ.
///  9. A XIX, B IX, C XV, D križ 8.
/// 10. A XVII, B XII, C XVIII, D križ 9.
/// 11. C karo dama, D karo fant, A KARO KRALJ, B srce kaval.
/// 12. A pik 8, B pik 9, C križ kaval, D PIK KAVAL.
///
/// Declarer B: tricks 1, 2, 3, 4, 6, 7 = 6 + 8 + 8 + 8 + 8 + 8 = 46, the
/// discards 3. Values 49 in 27 cards: 49 - 18 = 31.
/// Opponents: tricks 5, 8, 9, 10, 11, 12 = 8 + 12 + 4 + 4 + 14 + 8 = 50, the
/// packet not taken 5+1+1 = 7. Values 57 in 27 cards: 57 - 18 = 39.
///
/// Game: solo tri lost, -40. Difference 31 - 35 = -4, rounded -5. Trula:
/// škis (trick 2), mond (3) and pagat (7) are all in B's tricks, +10.
/// Kralji: B has only the srce kralj, 0. Entries: B -35, the others 0.
pub fn deal_2_solo() -> (Game, Expected) {
    let a = [
        t(19),
        t(17),
        t(16),
        t(10),
        t(4),
        srce(1),
        srce(4),
        kralj(Karo),
        kaval(Karo),
        kralj(Pik),
        pik(8),
        dama(Kriz),
    ];
    let b = [
        S,
        t(21),
        t(20),
        t(12),
        t(9),
        t(5),
        kralj(Srce),
        dama(Srce),
        karo(4),
        pik(10),
        pik(9),
        kriz(7),
    ];
    let c = [
        t(18),
        t(15),
        t(13),
        t(7),
        t(2),
        fant(Srce),
        srce(3),
        dama(Karo),
        karo(2),
        dama(Pik),
        kaval(Kriz),
        kriz(10),
    ];
    let d = [
        t(14),
        t(8),
        t(6),
        t(1),
        srce(2),
        fant(Karo),
        karo(1),
        kaval(Pik),
        fant(Pik),
        fant(Kriz),
        kriz(9),
        kriz(8),
    ];
    let talon = [kaval(Srce), karo(3), t(3), kralj(Kriz), t(11), pik(7)];
    let mut game = game(&whole_deal([&a, &b, &c, &d], talon));

    act(&mut game, B, Action::Bid(Dve));
    act(&mut game, C, Pass);
    act(&mut game, D, Action::Bid(Ena));
    act(&mut game, A, Pass);
    act(&mut game, B, Action::Bid(Ena));
    act(&mut game, D, Pass);
    assert_eq!(game.public().winning_bid(), Some(Ena));
    act(&mut game, B, Action::Bid(SoloTri));
    exchange(&mut game, 0, &[karo(4), karo(3), kriz(7)]);

    let tricks: [WrittenTrick; 12] = [
        (A, [kaval(Karo), t(5), karo(2), karo(1)]),
        (B, [S, t(2), t(6), t(4)]),
        (B, [t(21), t(7), t(8), t(10)]),
        (B, [kralj(Srce), srce(3), srce(2), srce(4)]),
        (B, [dama(Srce), fant(Srce), t(14), srce(1)]),
        (D, [fant(Kriz), dama(Kriz), t(3), kriz(10)]),
        (B, [t(20), t(13), t(1), t(16)]),
        (B, [pik(10), dama(Pik), fant(Pik), kralj(Pik)]),
        (A, [t(19), t(9), t(15), kriz(8)]),
        (A, [t(17), t(12), t(18), kriz(9)]),
        (C, [dama(Karo), fant(Karo), kralj(Karo), kaval(Srce)]),
        (A, [pik(8), pik(9), kaval(Kriz), kaval(Pik)]),
    ];
    play_written(&mut game, &tricks);

    let expected = Expected {
        winners: [B, B, B, B, D, B, B, A, A, C, A, D],
        declarer_points: 31,
        opponents_points: 39,
        entries: [0, -35, 0, 0],
    };
    (game, expected)
}

/// Deal 3: the called king lies in the talon. D plays dve, calls pik, takes
/// the packet with the pik kralj and wins a trick with it, so the rest of
/// the talon is D's. B loses the mond to D's škis.
///
/// Bidding: B and C pass, D bids dve, A passes. D plays dve.
/// Talon: karo fant, IV | pik kralj, VI | križ dama, XIII. D takes the
/// second packet and discards karo 2 and križ 9 (hidden).
///
/// Tricks:
///  1. A križ 10, B križ 8, C križ 7, D VI (void in križ).
///  2. D ŠKIS, A III, B XXI (the only tarok of B), C II. Mondfang for B.
///  3. D XX, A IX, B srce 3, C V.
///  4. D PIK KRALJ, A pik 7, B pik 9, C pik 8. The called king wins.
///  5. D pik dama, A pik fant, B pik kaval, C VIII (void in pik).
///  6. C srce 2, D SRCE KRALJ, A srce 4, B srce kaval.
///  7. D XIX, A XII, B karo 4, C X.
///  8. D XVIII, A XVI, B križ kaval, C XIV.
///  9. D XV, A XVII, B srce dama, C I (the last tarok of C).
/// 10. A karo kaval, B karo kralj, C karo 1, D VII (void in karo).
/// 11. D XI, A srce fant, B karo dama, C srce 1.
/// 12. D PIK 10, A karo 3, B križ kralj, C križ fant.
///
/// Declarer D: tricks 1, 2, 3, 4, 6, 7, 8, 10, 11, 12
/// = 4 + 12 + 4 + 8 + 10 + 4 + 6 + 10 + 8 + 9 = 75, the discards 2, the
/// rest of the talon 2+1+4+1 = 8. Values 85 in 46 cards: 85 - 30 - 1 = 54.
/// Opponents: trick 5 = 10, trick 9 = 11. Values 21 in 8 cards:
/// 21 - 4 - 1 = 16.
///
/// Game: dve won, 20. Difference 54 - 35 = 19, rounded 20. Kralji: pik
/// (trick 4), srce (6), karo (10) and križ (12) are all in D's tricks, +10.
/// Trula: the pagat is in A's trick 9, 0. Mondfang: B writes -20.
/// Entries: D 50, B -20, A 0, C 0.
pub fn deal_3_king_in_talon() -> (Game, Expected) {
    let a = [
        t(17),
        t(16),
        t(12),
        t(9),
        t(3),
        fant(Srce),
        srce(4),
        kaval(Karo),
        karo(3),
        fant(Pik),
        pik(7),
        kriz(10),
    ];
    let b = [
        t(21),
        dama(Srce),
        kaval(Srce),
        srce(3),
        kralj(Karo),
        dama(Karo),
        karo(4),
        kaval(Pik),
        pik(9),
        kralj(Kriz),
        kaval(Kriz),
        kriz(8),
    ];
    let c = [
        t(14),
        t(10),
        t(8),
        t(5),
        t(2),
        t(1),
        srce(2),
        srce(1),
        karo(1),
        pik(8),
        fant(Kriz),
        kriz(7),
    ];
    let d = [
        S,
        t(20),
        t(19),
        t(18),
        t(15),
        t(11),
        t(7),
        dama(Pik),
        pik(10),
        kralj(Srce),
        karo(2),
        kriz(9),
    ];
    let talon = [fant(Karo), t(4), kralj(Pik), t(6), dama(Kriz), t(13)];
    let mut game = game(&whole_deal([&a, &b, &c, &d], talon));

    act(&mut game, B, Pass);
    act(&mut game, C, Pass);
    act(&mut game, D, Action::Bid(Dve));
    act(&mut game, A, Pass);
    act(&mut game, D, Action::Bid(Dve));
    act(&mut game, D, Action::CallKing(Pik));
    exchange(&mut game, 1, &[karo(2), kriz(9)]);

    let tricks: [WrittenTrick; 12] = [
        (A, [kriz(10), kriz(8), kriz(7), t(6)]),
        (D, [S, t(3), t(21), t(2)]),
        (D, [t(20), t(9), srce(3), t(5)]),
        (D, [kralj(Pik), pik(7), pik(9), pik(8)]),
        (D, [dama(Pik), fant(Pik), kaval(Pik), t(8)]),
        (C, [srce(2), kralj(Srce), srce(4), kaval(Srce)]),
        (D, [t(19), t(12), karo(4), t(10)]),
        (D, [t(18), t(16), kaval(Kriz), t(14)]),
        (D, [t(15), t(17), dama(Srce), t(1)]),
        (A, [kaval(Karo), kralj(Karo), karo(1), t(7)]),
        (D, [t(11), fant(Srce), dama(Karo), srce(1)]),
        (D, [pik(10), karo(3), kralj(Kriz), fant(Kriz)]),
    ];
    play_written(&mut game, &tricks);

    let expected = Expected {
        winners: [D, D, D, D, C, D, D, D, A, D, D, D],
        declarer_points: 54,
        opponents_points: 16,
        entries: [0, -20, 0, 50],
    };
    (game, expected)
}
