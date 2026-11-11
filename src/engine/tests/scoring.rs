//! Spec 005, criteria 29-44: scoring of the six normal contracts.
//!
//! Seats: A is forehand (seat 0), then B, C, D; D (seat 3) deals.
//!
//! Most tests build a deal in which the wanted event cannot be avoided: one
//! seat holds a hand that wins by force, and its cards are played in a
//! written order. The other seats play their lowest legal card.

mod common;

use common::flow::{
    A, B, C, D, NORMAL, act, deal_with, declare, exchange, exchange_back, finish, game,
    king_taken_and_led, play_random, play_scripted, seeded_game, whole_deal,
};
use common::second_scorer::{assert_scores_agree, second_score};
use common::written::{deal_1_partner, deal_2_solo, deal_3_king_in_talon};
use common::{SUITS, court, kralj, pip, run_cases, seat, tarok};
use proptest::prelude::*;
use tarok_engine::Contract::{Dve, Ena, SoloDve, SoloEna, SoloTri, Tri};
use tarok_engine::{
    Action, Card, CardSet, Contract, Deal, Game, Phase, PublicPartner, Rank, Score, Seat, Suit,
    points, score,
};

const S: Card = Card::SKIS;
const MOND: Card = Card::MOND;
const PAGAT: Card = Card::PAGAT;

/// Taroks by number, in the order given.
fn taroks(numbers: impl IntoIterator<Item = u8>) -> Vec<Card> {
    numbers.into_iter().map(tarok).collect()
}

/// Škis, then the taroks from XXI down to `lowest`.
fn top_taroks(lowest: u8) -> Vec<Card> {
    let mut cards = vec![S];
    cards.extend((lowest..=21).rev().map(tarok));
    cards
}

fn srce(n: u8) -> Card {
    pip(Suit::Srce, n)
}
fn karo(n: u8) -> Card {
    pip(Suit::Karo, n)
}
fn pik(n: u8) -> Card {
    pip(Suit::Pik, n)
}
fn kriz(n: u8) -> Card {
    pip(Suit::Kriz, n)
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

/// The six low karo: a talon without a king, a trula card or a srce.
fn karo_talon() -> [Card; 6] {
    [
        karo(2),
        karo(1),
        fant(Suit::Karo),
        kaval(Suit::Karo),
        dama(Suit::Karo),
        kralj(Suit::Karo),
    ]
}

/// A declares `contract`, calls `call` if the contract asks for a king,
/// takes the first packet and puts the same cards down again. A's hand in
/// the play is the hand as dealt.
fn a_declares(deal: &Deal, contract: Contract, call: Option<Suit>) -> Game {
    let mut game = game(deal);
    declare(&mut game, A, contract);
    if game.phase() == Phase::KingCall {
        act(
            &mut game,
            A,
            Action::CallKing(call.expect("a suit to call")),
        );
    }
    exchange_back(&mut game, 0);
    game
}

fn finished(mut game: Game, scripts: [&[Card]; 4]) -> (Game, Score) {
    play_scripted(&mut game, scripts);
    finish(&mut game);
    let score = game.score().expect("the deal is over");
    (game, score)
}

/// How many tricks each seat (A, B, C, D) won.
fn tricks_won(game: &Game) -> [usize; 4] {
    Seat::ALL.map(|seat| {
        game.public()
            .tricks()
            .iter()
            .filter(|t| t.winner == seat)
            .count()
    })
}

/// The trick (1-12) a card was played to.
fn trick_of(game: &Game, card: Card) -> usize {
    1 + game
        .public()
        .tricks()
        .iter()
        .position(|t| t.cards.contains(&card))
        .unwrap_or_else(|| panic!("{card} was not played"))
}

fn case() -> impl Strategy<Value = (u64, u8, Vec<u8>)> {
    (
        any::<u64>(),
        0..4u8,
        proptest::collection::vec(any::<u8>(), 96),
    )
}

// ---------------------------------------------------------------------
// The built deals
// ---------------------------------------------------------------------

/// A holds škis, mond, XX-XIII, the pagat and the srce 4. B holds five low
/// taroks and the seven other srce. A leads the taroks from the top, then
/// the pagat, then the srce 4. A wins eleven tricks; B wins the last one
/// with its last srce.
fn trula_for_a() -> (Game, Score) {
    let mut a = top_taroks(13);
    a.extend([PAGAT, srce(4)]);
    let mut b = taroks(2..=6);
    b.extend([
        srce(3),
        srce(2),
        srce(1),
        fant(Suit::Srce),
        kaval(Suit::Srce),
        dama(Suit::Srce),
        kralj(Suit::Srce),
    ]);
    let talon = [
        karo(4),
        karo(3),
        karo(2),
        karo(1),
        fant(Suit::Karo),
        kaval(Suit::Karo),
    ];
    let deal = deal_with([&a, &b, &[], &[]], &talon);
    let game = a_declares(&deal, SoloEna, None);
    let mut script = top_taroks(13);
    script.extend([PAGAT, srce(4)]);
    finished(game, [&script, &[], &[], &[]])
}

/// B, an opponent, holds škis, mond, XX-XIII, the pagat and the srce 4.
/// A is the declarer in solo ena and wins the first trick with the srce
/// kralj. B trumps the second trick, leads the taroks from the top and the
/// pagat to the last trick, where nobody has a tarok left.
fn trula_for_the_opponents() -> (Game, Score) {
    let a = [
        kralj(Suit::Srce),
        fant(Suit::Karo),
        kaval(Suit::Karo),
        dama(Suit::Karo),
        kralj(Suit::Karo),
        pik(7),
        pik(8),
        pik(9),
        pik(10),
        fant(Suit::Pik),
        kaval(Suit::Pik),
        dama(Suit::Pik),
    ];
    let mut b = top_taroks(13);
    b.extend([PAGAT, srce(4)]);
    let c = [srce(3), srce(2)];
    let d = [srce(1), fant(Suit::Srce)];
    let talon = [
        kaval(Suit::Srce),
        dama(Suit::Srce),
        karo(4),
        karo(3),
        karo(2),
        karo(1),
    ];
    let deal = deal_with([&a, &b, &c, &d], &talon);
    let game = a_declares(&deal, SoloEna, None);
    let mut b_script = vec![srce(4)];
    b_script.extend(top_taroks(13));
    b_script.push(PAGAT);
    finished(game, [&[kralj(Suit::Srce)], &b_script, &[], &[]])
}

/// A holds škis, mond, XX-XVI, the four kings and the srce 4. B holds six
/// taroks and the six srce between the 4 and the kralj. A leads seven
/// taroks, the four kings and the srce 4; B wins the last trick.
fn kings_for_a() -> (Game, Score) {
    let mut a = top_taroks(16);
    a.extend(SUITS.map(kralj));
    a.push(srce(4));
    let mut b = taroks(2..=7);
    b.extend([
        srce(3),
        srce(2),
        srce(1),
        fant(Suit::Srce),
        kaval(Suit::Srce),
        dama(Suit::Srce),
    ]);
    let c = taroks(8..=12);
    let d = taroks([13, 14, 15, 1]);
    let talon = [
        karo(4),
        karo(3),
        karo(2),
        karo(1),
        fant(Suit::Karo),
        kaval(Suit::Karo),
    ];
    let deal = deal_with([&a, &b, &c, &d], &talon);
    let game = a_declares(&deal, SoloEna, None);
    let mut script = top_taroks(16);
    script.extend([
        kralj(Suit::Karo),
        kralj(Suit::Pik),
        kralj(Suit::Kriz),
        kralj(Suit::Srce),
        srce(4),
    ]);
    finished(game, [&script, &[], &[], &[]])
}

/// The hands of `kings_for_the_opponents`. With `srce_king_in_talon` the
/// srce kralj lies in the talon instead of B's hand.
fn kings_for_the_opponents(srce_king_in_talon: bool) -> (Game, Score) {
    let mut a = taroks(2..=6);
    a.extend([
        srce(3),
        karo(3),
        karo(2),
        karo(1),
        fant(Suit::Karo),
        kaval(Suit::Karo),
        dama(Suit::Karo),
    ]);
    let mut c = taroks(7..=11);
    c.extend([
        srce(4),
        srce(2),
        srce(1),
        dama(Suit::Pik),
        kriz(7),
        kriz(8),
        kriz(9),
    ]);
    let d_suits = [
        fant(Suit::Srce),
        kaval(Suit::Srce),
        dama(Suit::Srce),
        kriz(10),
        fant(Suit::Kriz),
        kaval(Suit::Kriz),
        dama(Suit::Kriz),
    ];
    let kings = [kralj(Suit::Pik), kralj(Suit::Kriz), kralj(Suit::Karo)];

    if !srce_king_in_talon {
        // B: škis, mond, XX-XVI, the four kings, karo 4.
        let mut b = top_taroks(16);
        b.push(kralj(Suit::Srce));
        b.extend(kings);
        b.push(karo(4));
        let mut d = taroks([12, 13, 14, 15, 1]);
        d.extend(d_suits);
        let talon = [
            pik(7),
            pik(8),
            pik(9),
            pik(10),
            fant(Suit::Pik),
            kaval(Suit::Pik),
        ];
        let deal = whole_deal([&a, &b, &c, &d], talon);
        let game = a_declares(&deal, SoloEna, None);
        // B follows the led srce with the kralj, then leads.
        let mut b_script = vec![kralj(Suit::Srce)];
        b_script.extend(top_taroks(16));
        b_script.extend(kings);
        b_script.push(karo(4));
        finished(game, [&[srce(3)], &b_script, &[], &[]])
    } else {
        // B: škis, mond, XX-XV, three kings, karo 4. The srce kralj lies in
        // the first packet of the talon; A takes the second packet.
        let mut b = top_taroks(15);
        b.extend(kings);
        b.push(karo(4));
        let mut d = taroks([12, 13, 14, 1]);
        d.extend(d_suits);
        d.push(pik(7));
        let talon = [
            kralj(Suit::Srce),
            pik(8),
            pik(9),
            pik(10),
            fant(Suit::Pik),
            kaval(Suit::Pik),
        ];
        let deal = whole_deal([&a, &b, &c, &d], talon);
        let mut game = game(&deal);
        declare(&mut game, A, SoloTri);
        exchange_back(&mut game, 1);
        // B trumps the led srce with the XV, then leads.
        let mut b_script = vec![tarok(15)];
        b_script.extend(top_taroks(16));
        b_script.extend(kings);
        b_script.push(karo(4));
        finished(game, [&[srce(3)], &b_script, &[], &[]])
    }
}

/// The hands for the kralj ultimo tests. A holds škis, mond, XX-XIII, the
/// karo 4 and one more card: the srce kralj (`a_holds_king`) or the pik 8.
/// In the second case C holds the srce kralj.
///
/// A leads ten taroks, then the karo 4, which B wins with the karo 3. B
/// leads to the last trick.
fn ultimo_deal(a_holds_king: bool) -> Deal {
    let mut a = top_taroks(13);
    a.push(karo(4));
    let talon = karo_talon();
    if a_holds_king {
        a.push(kralj(Suit::Srce));
        let mut b = taroks(2..=5);
        b.extend([
            karo(3),
            dama(Suit::Srce),
            pik(7),
            pik(8),
            pik(9),
            pik(10),
            fant(Suit::Pik),
            kaval(Suit::Pik),
        ]);
        let mut c = taroks(6..=9);
        c.extend([
            srce(4),
            srce(3),
            srce(2),
            srce(1),
            kriz(7),
            kriz(8),
            kriz(9),
            kriz(10),
        ]);
        let mut d = taroks([10, 11, 12, 1]);
        d.extend([
            fant(Suit::Srce),
            kaval(Suit::Srce),
            dama(Suit::Pik),
            kralj(Suit::Pik),
            fant(Suit::Kriz),
            kaval(Suit::Kriz),
            dama(Suit::Kriz),
            kralj(Suit::Kriz),
        ]);
        whole_deal([&a, &b, &c, &d], talon)
    } else {
        a.push(pik(8));
        let mut b = taroks(2..=5);
        b.extend([
            karo(3),
            dama(Suit::Srce),
            pik(7),
            pik(9),
            pik(10),
            fant(Suit::Pik),
            kaval(Suit::Pik),
            dama(Suit::Pik),
        ]);
        let mut c = taroks(6..=9);
        c.extend([
            kralj(Suit::Srce),
            srce(4),
            srce(3),
            srce(2),
            kriz(7),
            kriz(8),
            kriz(9),
            kriz(10),
        ]);
        let mut d = taroks([10, 11, 12, 1]);
        d.extend([
            srce(1),
            fant(Suit::Srce),
            kaval(Suit::Srce),
            kralj(Suit::Pik),
            fant(Suit::Kriz),
            kaval(Suit::Kriz),
            dama(Suit::Kriz),
            kralj(Suit::Kriz),
        ]);
        whole_deal([&a, &b, &c, &d], talon)
    }
}

/// A's cards in `ultimo_deal(true)`: ten taroks, the karo 4, the srce kralj.
fn ultimo_script_a() -> Vec<Card> {
    let mut script = top_taroks(13);
    script.extend([karo(4), kralj(Suit::Srce)]);
    script
}

/// B's cards in `ultimo_deal(true)`. With `lead_srce` B keeps the srce dama
/// for the last trick, else the pik kaval.
fn ultimo_script_b(lead_srce: bool) -> Vec<Card> {
    let mut script = taroks(2..=5);
    script.extend([pik(7), pik(8), pik(9), pik(10), fant(Suit::Pik)]);
    if lead_srce {
        script.extend([kaval(Suit::Pik), karo(3), dama(Suit::Srce)]);
    } else {
        script.extend([dama(Suit::Srce), karo(3), kaval(Suit::Pik)]);
    }
    script
}

/// The hands for three of the pagat ultimo cases. See each test.
fn pagat_ultimo_own_wins() -> (Game, Score) {
    // A: škis, mond, XX-XIV, pagat, karo 4, srce 4.
    let mut a = top_taroks(14);
    a.extend([PAGAT, karo(4), srce(4)]);
    let mut b = taroks(2..=5);
    b.extend([
        karo(3),
        kralj(Suit::Srce),
        pik(7),
        pik(8),
        pik(9),
        pik(10),
        fant(Suit::Pik),
        kaval(Suit::Pik),
    ]);
    let mut c = taroks(6..=9);
    c.extend([
        srce(3),
        srce(2),
        srce(1),
        fant(Suit::Srce),
        kriz(7),
        kriz(8),
        kriz(9),
        kriz(10),
    ]);
    let mut d = taroks(10..=13);
    d.extend([
        kaval(Suit::Srce),
        dama(Suit::Srce),
        dama(Suit::Pik),
        kralj(Suit::Pik),
        fant(Suit::Kriz),
        kaval(Suit::Kriz),
        dama(Suit::Kriz),
        kralj(Suit::Kriz),
    ]);
    let deal = whole_deal([&a, &b, &c, &d], karo_talon());
    let game = a_declares(&deal, SoloTri, None);
    // A: nine taroks, karo 4 (B wins with the karo 3), srce 4 under B's
    // srce kralj, and the pagat on B's last card, the pik 7.
    let mut a_script = top_taroks(14);
    a_script.extend([karo(4), srce(4), PAGAT]);
    let mut b_script = taroks(2..=5);
    b_script.extend([
        pik(8),
        pik(9),
        pik(10),
        fant(Suit::Pik),
        kaval(Suit::Pik),
        karo(3),
        kralj(Suit::Srce),
        pik(7),
    ]);
    finished(game, [&a_script, &b_script, &[], &[]])
}

fn pagat_ultimo_own_loses() -> (Game, Score) {
    // A as above. D holds ten taroks II-XI, the karo 3 and the srce 3, and
    // keeps the II for the last trick.
    let mut a = top_taroks(14);
    a.extend([PAGAT, karo(4), srce(4)]);
    let mut d = taroks(2..=11);
    d.extend([karo(3), srce(3)]);
    let b = taroks([12, 13]);
    let deal = deal_with([&a, &b, &[], &d], &karo_talon());
    let game = a_declares(&deal, SoloTri, None);
    let mut a_script = top_taroks(14);
    a_script.extend([karo(4), srce(4), PAGAT]);
    let mut d_script = taroks(3..=11);
    d_script.extend([karo(3), srce(3), tarok(2)]);
    finished(game, [&a_script, &[], &[], &d_script])
}

fn pagat_ultimo_opponent_loses() -> (Game, Score) {
    // A: škis, mond, XX-XII, karo 4. D: taroks I-XI and the karo 3. A keeps
    // the XII for the last trick; D wins the eleventh with the karo 3 and
    // must lead the pagat.
    let mut a = top_taroks(12);
    a.push(karo(4));
    let mut d = taroks(1..=11);
    d.push(karo(3));
    let deal = deal_with([&a, &[], &[], &d], &karo_talon());
    let game = a_declares(&deal, SoloTri, None);
    let mut a_script = top_taroks(13);
    a_script.extend([karo(4), tarok(12)]);
    let mut d_script = taroks(2..=11);
    d_script.extend([karo(3), PAGAT]);
    finished(game, [&a_script, &[], &[], &d_script])
}

/// A deal in which the pagat is played to the last trick together with a
/// higher tarok of another seat. Every seat holds two cards of each suit
/// and four taroks; the talon is XIII-XVIII.
///
/// - A: srce kralj, 4; karo kralj, dama; pik kralj, dama; križ dama, fant;
///   škis, XXI, XX and one more tarok.
/// - B: srce dama, 3; karo kaval, 4; pik kaval, 7; križ kaval, 7; II-V.
/// - C: srce kaval, 2; karo fant, 3; pik fant, 8; križ kralj, 8; pagat,
///   VI, VII, VIII.
/// - D: srce fant, 1; karo 2, 1; pik 9, 10; križ 9, 10; IX, X, XI and one
///   more tarok.
///
/// `xix` holds the XIX: A or D. The other one of the two holds the XII.
/// The holder of the XIX takes the last trick, in which C plays the pagat.
fn pagat_and_a_higher_tarok(xix: Seat) -> Deal {
    let (a_tarok, d_tarok) = if xix == A { (19, 12) } else { (12, 19) };
    let a = [
        kralj(Suit::Srce),
        srce(4),
        kralj(Suit::Karo),
        dama(Suit::Karo),
        kralj(Suit::Pik),
        dama(Suit::Pik),
        dama(Suit::Kriz),
        fant(Suit::Kriz),
        S,
        MOND,
        tarok(20),
        tarok(a_tarok),
    ];
    let b = [
        dama(Suit::Srce),
        srce(3),
        kaval(Suit::Karo),
        karo(4),
        kaval(Suit::Pik),
        pik(7),
        kaval(Suit::Kriz),
        kriz(7),
        tarok(2),
        tarok(3),
        tarok(4),
        tarok(5),
    ];
    let c = [
        kaval(Suit::Srce),
        srce(2),
        fant(Suit::Karo),
        karo(3),
        fant(Suit::Pik),
        pik(8),
        kralj(Suit::Kriz),
        kriz(8),
        PAGAT,
        tarok(6),
        tarok(7),
        tarok(8),
    ];
    let d = [
        fant(Suit::Srce),
        srce(1),
        karo(2),
        karo(1),
        pik(9),
        pik(10),
        kriz(9),
        kriz(10),
        tarok(9),
        tarok(10),
        tarok(11),
        tarok(d_tarok),
    ];
    let talon = [13, 14, 15, 16, 17, 18].map(tarok);
    whole_deal([&a, &b, &c, &d], talon)
}

/// The play of `pagat_and_a_higher_tarok`, all 48 cards:
///
///  1-6. A leads srce kralj, karo kralj, karo dama, pik kralj, pik dama,
///       križ dama and wins each; all follow.
///  7.   A križ fant, B križ kaval, C križ kralj, D križ 10: C wins.
///  8.   C srce kaval, D srce fant, A srce 4, B srce dama: B wins.
///  9.   B II, C VI, D IX, A škis: A wins.
/// 10.   A XXI, B III, C VII, D X.
/// 11.   A XX, B IV, C VIII, D XI.
/// 12.   A leads the last tarok, B V, C pagat, D the last tarok. The XIX
///       wins: A's if `xix` is A, else D's.
fn pagat_and_a_higher_tarok_scripts(xix: Seat) -> [&'static [Card]; 4] {
    const fn t(number: u8) -> Card {
        Card::tarok(number).unwrap()
    }
    const fn p(suit: Suit, number: u8) -> Card {
        Card::pip(suit, number).unwrap()
    }
    const fn c(suit: Suit, rank: Rank) -> Card {
        Card::court(suit, rank)
    }
    use Rank::{Dama, Fant, Kaval, Kralj};
    use Suit::{Karo, Kriz, Pik, Srce};
    const A_FIRST: [Card; 11] = [
        c(Srce, Kralj),
        c(Karo, Kralj),
        c(Karo, Dama),
        c(Pik, Kralj),
        c(Pik, Dama),
        c(Kriz, Dama),
        c(Kriz, Fant),
        p(Srce, 4),
        Card::SKIS,
        Card::MOND,
        t(20),
    ];
    const A_WITH_XIX: [Card; 12] = with_last(A_FIRST, t(19));
    const A_WITH_XII: [Card; 12] = with_last(A_FIRST, t(12));
    const B_CARDS: [Card; 12] = [
        p(Srce, 3),
        p(Karo, 4),
        c(Karo, Kaval),
        p(Pik, 7),
        c(Pik, Kaval),
        p(Kriz, 7),
        c(Kriz, Kaval),
        c(Srce, Dama),
        t(2),
        t(3),
        t(4),
        t(5),
    ];
    const C_CARDS: [Card; 12] = [
        p(Srce, 2),
        p(Karo, 3),
        c(Karo, Fant),
        p(Pik, 8),
        c(Pik, Fant),
        p(Kriz, 8),
        c(Kriz, Kralj),
        c(Srce, Kaval),
        t(6),
        t(7),
        t(8),
        Card::PAGAT,
    ];
    const D_FIRST: [Card; 11] = [
        p(Srce, 1),
        p(Karo, 2),
        p(Karo, 1),
        p(Pik, 9),
        p(Pik, 10),
        p(Kriz, 9),
        p(Kriz, 10),
        c(Srce, Fant),
        t(9),
        t(10),
        t(11),
    ];
    const D_WITH_XIX: [Card; 12] = with_last(D_FIRST, t(19));
    const D_WITH_XII: [Card; 12] = with_last(D_FIRST, t(12));
    if xix == A {
        [&A_WITH_XIX, &B_CARDS, &C_CARDS, &D_WITH_XII]
    } else {
        [&A_WITH_XII, &B_CARDS, &C_CARDS, &D_WITH_XIX]
    }
}

/// The eleven cards and one more.
const fn with_last(first: [Card; 11], last: Card) -> [Card; 12] {
    let mut all = [last; 12];
    let mut i = 0;
    while i < 11 {
        all[i] = first[i];
        i += 1;
    }
    all
}

/// The talon used by the mondfang deals: no tarok, no king.
fn low_talon() -> [Card; 6] {
    [srce(4), srce(3), srce(2), srce(1), karo(4), karo(3)]
}

/// The trick that holds both the škis and the mond, if there is one.
fn skis_mond_trick(game: &Game) -> Option<usize> {
    game.public()
        .tricks()
        .iter()
        .position(|t| t.cards.contains(&S) && t.cards.contains(&MOND))
}

// ---------------------------------------------------------------------
// Criteria
// ---------------------------------------------------------------------

#[test]
fn c29_each_side_counts_its_cards_and_the_counts_sum_to_70() {
    run_cases(3_000, case(), |(seed, dealer, choices)| {
        let game = play_random(seed, seat(dealer), &choices, |_| {});
        // The second scorer of the tests collects the two piles one by one
        // from the record of the deal: for the declarer's side the tricks
        // of the declarer and the partner, the discards, and the rest of
        // the talon in the case of criterion 23; for the opponents their
        // tricks and the talon cards that stayed with them. It compares
        // both piles, both counts and every part of every entry with the
        // engine.
        let second = assert_scores_agree(&game);
        // Every card is in one pile, and the two counts sum to 70.
        prop_assert!(second.declarer_pile.is_disjoint(second.opponents_pile));
        prop_assert_eq!(second.declarer_pile | second.opponents_pile, CardSet::FULL);
        prop_assert_eq!(
            points::count(second.declarer_pile) + points::count(second.opponents_pile),
            70
        );
        Ok(())
    });
}

#[test]
fn c30_declarer_wins_with_36_or_more_and_35_is_a_loss() {
    for contract in NORMAL {
        for points in 0..=70u8 {
            let (game, _) = score::game_and_difference(contract, points).unwrap();
            if points >= 36 {
                assert!(game > 0, "{contract:?} with {points} is won");
            } else {
                assert!(game < 0, "{contract:?} with {points} is lost");
            }
        }
    }
    // In whole deals: the written deals with 52, 31 and 54 points.
    let (game, _) = deal_1_partner();
    assert!(game.score().unwrap().declarer_won());
    let (game, _) = deal_2_solo();
    assert!(!game.score().unwrap().declarer_won());
    let (game, _) = deal_3_king_in_talon();
    assert!(game.score().unwrap().declarer_won());

    // The edge in whole deals, by the entries written: 36 points win and
    // 35 points lose.
    whole_deal_case(548, Tri, 36, [0, 10, 10, 0]);
    whole_deal_case(69, Tri, 35, [-10, 0, -10, 0]);
    whole_deal_case(1049, SoloTri, 36, [0, 0, 40, 0]);
    whole_deal_case(476, Dve, 35, [-20, 0, -20, 0]);
}

/// A whole deal that ends with a wanted contract and count and with no
/// bonus and no mondfang, so the entries show the game and the difference
/// alone. The deal is `seeded_game(seed)`: fixed cards, bids and plays. It
/// was found by trying seeds; what it must give is written here from the
/// criteria. The count is taken from the second scorer of the tests, which
/// collects the piles from the tricks on its own.
fn whole_deal_case(seed: u64, contract: Contract, points: u8, entries: [i32; 4]) {
    let game = seeded_game(seed);
    assert_eq!(game.public().contract(), Some(contract), "seed {seed}");
    let second = second_score(&game);
    assert_eq!(second.declarer_points, points, "seed {seed}");
    assert_eq!(second.opponents_points, 70 - points, "seed {seed}");
    // Nothing but game and difference in any entry.
    for parts in second.parts {
        assert_eq!(parts[2..], [0; 6], "seed {seed}: a bonus or a mondfang");
    }
    let score = game.score().unwrap();
    assert_eq!(score.declarer_points(), points, "seed {seed}");
    assert_eq!(score.declarer_won(), points >= 36, "seed {seed}");
    assert_eq!(score.entries(), entries, "seed {seed}");
}

#[test]
fn c31_difference_is_points_minus_35_rounded_to_the_nearest_5() {
    // The source examples.
    assert_eq!(score::rounded_difference(43), 10);
    assert_eq!(score::rounded_difference(37), 0);
    assert_eq!(score::rounded_difference(21), -15);

    // Written out for every count around the middle.
    let around: [(u8, i32); 11] = [
        (30, -5),
        (31, -5),
        (32, -5),
        (33, 0),
        (34, 0),
        (35, 0),
        (36, 0),
        (37, 0),
        (38, 5),
        (39, 5),
        (40, 5),
    ];
    for (points, expected) in around {
        assert_eq!(score::rounded_difference(points), expected, "{points}");
    }
    assert_eq!(score::rounded_difference(0), -35);
    assert_eq!(score::rounded_difference(70), 35);

    // For every count: a multiple of 5, and no other multiple is nearer.
    for points in 0..=70u8 {
        let exact = i32::from(points) - 35;
        let rounded = score::rounded_difference(points);
        assert_eq!(rounded % 5, 0, "{points}");
        assert!((rounded - exact).abs() <= 2, "{points}: {rounded}");
    }
}

#[test]
fn c32_game_values() {
    let values = [
        (Tri, 10),
        (Dve, 20),
        (Ena, 30),
        (SoloTri, 40),
        (SoloDve, 50),
        (SoloEna, 60),
    ];
    for (contract, value) in values {
        assert_eq!(contract.game_value(), Some(value));
        // A bare win and a bare loss (difference 0) are plus and minus it.
        assert_eq!(score::game_and_difference(contract, 36), Some((value, 0)));
        assert_eq!(score::game_and_difference(contract, 35), Some((-value, 0)));
    }
    // The other contracts have no value in this slice.
    for contract in Contract::ALL {
        if !NORMAL.contains(&contract) {
            assert_eq!(contract.game_value(), None);
            assert_eq!(score::game_and_difference(contract, 40), None);
        }
    }
}

#[test]
fn c33_a_win_scores_game_value_plus_rounded_difference() {
    // The source example: ena won with 43 points scores 40.
    let (game, difference) = score::game_and_difference(Ena, 43).unwrap();
    assert_eq!((game, difference), (30, 10));
    assert_eq!(game + difference, 40);

    assert_eq!(score::game_and_difference(Tri, 36), Some((10, 0)));
    assert_eq!(score::game_and_difference(Tri, 70), Some((10, 35)));
    assert_eq!(score::game_and_difference(SoloEna, 48), Some((60, 15)));

    // Whole deals. Deal 1: tri won with 52 points, 10 + 15, plus a trula.
    let (game, _) = deal_1_partner();
    let score = game.score().unwrap();
    assert_eq!(score.declarer_points(), 52);
    assert_eq!((score.parts(A).game, score.parts(A).difference), (10, 15));
    assert_eq!(score.entry(A), 10 + 15 + 10);
    // Deal 3: dve won with 54 points, 20 + 20, plus the kings.
    let (game, _) = deal_3_king_in_talon();
    let score = game.score().unwrap();
    assert_eq!(score.declarer_points(), 54);
    assert_eq!((score.parts(D).game, score.parts(D).difference), (20, 20));

    // The source example as a whole deal: ena won with 43 points. The
    // declarer (seat 0) and the partner (seat 2) write 40.
    whole_deal_case(115, Ena, 43, [40, 0, 40, 0]);
}

#[test]
fn c34_a_loss_scores_minus_game_value_and_difference() {
    // The source examples: dve lost with 21 points scores -35; tri with
    // exactly 35 points scores -10.
    let (game, difference) = score::game_and_difference(Dve, 21).unwrap();
    assert_eq!((game, difference), (-20, -15));
    assert_eq!(game + difference, -35);
    let (game, difference) = score::game_and_difference(Tri, 35).unwrap();
    assert_eq!(game + difference, -10);

    assert_eq!(score::game_and_difference(SoloEna, 0), Some((-60, -35)));
    assert_eq!(score::game_and_difference(Ena, 33), Some((-30, 0)));

    // A whole deal. Deal 2: solo tri lost with 31 points: -(40 + 5).
    let (game, _) = deal_2_solo();
    let score = game.score().unwrap();
    assert_eq!(score.declarer_points(), 31);
    assert_eq!((score.parts(B).game, score.parts(B).difference), (-40, -5));

    // The source examples as whole deals: dve lost with 21 points, the
    // declarer (seat 2) and the partner (seat 3) write -35; tri with
    // exactly 35 points, the declarer (seat 2) and the partner (seat 0)
    // write -10.
    whole_deal_case(5083, Dve, 21, [0, 0, -35, -35]);
    whole_deal_case(69, Tri, 35, [-10, 0, -10, 0]);
}

#[test]
fn c35_partner_writes_the_same_entry_and_opponents_write_no_game() {
    // The written partner deal: A and C write 35, B and D write 0.
    let (game, _) = deal_1_partner();
    let score = game.score().unwrap();
    assert_eq!(game.full().partner(), Some(C));
    assert_eq!(score.parts(A), score.parts(C));
    assert_eq!(score.entries(), [35, 0, 35, 0]);

    // Random deals with a partner.
    let with_partner = std::cell::Cell::new(0u32);
    run_cases(3_000, case(), |(seed, dealer, choices)| {
        let game = play_random(seed, seat(dealer), &choices, |_| {});
        let score = game.score().unwrap();
        let declarer = game.public().declarer().unwrap();
        let partner = game.full().partner();
        if partner.is_some() {
            with_partner.set(with_partner.get() + 1);
        }
        for seat in Seat::ALL {
            let parts = score.parts(seat);
            if Some(seat) == partner {
                // The same entry as the declarer, part by part. Only the
                // mondfang is each player's own.
                let mut same = *score.parts(declarer);
                same.mondfang = parts.mondfang;
                prop_assert_eq!(*parts, same);
            } else if seat != declarer {
                // An opponent: nothing for the game or the bonuses.
                prop_assert_eq!(
                    [
                        parts.game,
                        parts.difference,
                        parts.trula,
                        parts.kralji,
                        parts.kralj_ultimo,
                        parts.pagat_ultimo,
                        parts.valat
                    ],
                    [0; 7]
                );
            }
        }
        Ok(())
    });
    assert!(with_partner.get() > 300, "{} deals", with_partner.get());
}

#[test]
fn c36_silent_trula() {
    // +10: škis, mond and pagat all in the declarer's tricks.
    let (game, score) = trula_for_a();
    assert_eq!(tricks_won(&game), [11, 1, 0, 0]);
    assert_eq!(score.parts(A).valat, 0);
    for card in [S, MOND, PAGAT] {
        assert!(game.public().tricks_won(A).contains(card));
    }
    assert_eq!(score.parts(A).trula, 10);
    for seat in [B, C, D] {
        assert_eq!(score.parts(seat).trula, 0);
    }

    // -10: all three in the opponents' tricks.
    let (game, score) = trula_for_the_opponents();
    assert_eq!(tricks_won(&game), [1, 11, 0, 0]);
    for card in [S, MOND, PAGAT] {
        assert!(game.public().tricks_won(B).contains(card));
    }
    assert_eq!(score.parts(A).trula, -10);

    // -10 also when the third card lies in the talon that was not taken
    // (answer B). The pagat is in the first packet; A takes the second.
    // B holds škis and mond; each wins the trick it is played to.
    let deal = deal_with(
        [&taroks([20, 19, 18, 17]), &[S, MOND], &[], &[]],
        &[PAGAT, karo(4), karo(3), srce(4), srce(3), srce(2)],
    );
    let mut game = common::flow::game(&deal);
    declare(&mut game, A, SoloTri);
    exchange_back(&mut game, 1);
    finish(&mut game);
    let score = game.score().unwrap();
    assert_eq!(score.parts(A).valat, 0);
    assert!(game.full().opponents_pile().contains(PAGAT));
    assert!(game.public().tricks_won(B).contains(S));
    assert!(game.public().tricks_won(B).contains(MOND));
    assert_eq!(score.parts(A).trula, -10);

    // 0 when the three are split: A wins škis and mond, D wins the pagat.
    let (game, score) = pagat_ultimo_own_loses();
    assert!(game.public().tricks_won(A).contains(S));
    assert!(game.public().tricks_won(A).contains(MOND));
    assert!(game.public().tricks_won(D).contains(PAGAT));
    assert_eq!(score.parts(A).trula, 0);
}

#[test]
fn c37_silent_kralji() {
    let kings: CardSet = SUITS.into_iter().map(kralj).collect();

    // +10: the four kings in the declarer's tricks.
    let (game, score) = kings_for_a();
    assert_eq!(tricks_won(&game), [11, 1, 0, 0]);
    assert!(kings.is_subset(game.public().tricks_won(A)));
    assert_eq!(score.parts(A).kralji, 10);
    for seat in [B, C, D] {
        assert_eq!(score.parts(seat).kralji, 0);
    }

    // -10: the four kings in the opponents' tricks.
    let (game, score) = kings_for_the_opponents(false);
    assert_eq!(tricks_won(&game), [1, 11, 0, 0]);
    assert!(kings.is_subset(game.public().tricks_won(B)));
    assert_eq!(score.parts(A).kralji, -10);

    // -10 also when one king lies in the talon that was not taken
    // (answer B).
    let (game, score) = kings_for_the_opponents(true);
    assert_eq!(tricks_won(&game), [1, 11, 0, 0]);
    assert_eq!(game.public().tricks_won(B).intersection(kings).len(), 3);
    assert!(game.full().opponents_pile().contains(kralj(Suit::Srce)));
    assert!(!game.public().taken_cards().contains(kralj(Suit::Srce)));
    assert_eq!(score.parts(A).kralji, -10);

    // 0 when the kings are split: A wins the srce kralj, B the others.
    let (game, score) = trula_for_the_opponents();
    assert!(game.public().tricks_won(A).contains(kralj(Suit::Srce)));
    assert_eq!(score.parts(A).kralji, 0);
}

#[test]
fn c38_silent_kralj_ultimo() {
    let king = kralj(Suit::Srce);

    // +10: A calls srce, holds the srce kralj (answer C) and wins the last
    // trick with it.
    let game = a_declares(&ultimo_deal(true), Tri, Some(Suit::Srce));
    let (game, score) = finished(game, [&ultimo_script_a(), &ultimo_script_b(true), &[], &[]]);
    assert_eq!(tricks_won(&game), [11, 1, 0, 0]);
    assert_eq!(trick_of(&game, king), 12);
    assert_eq!(game.public().tricks()[11].winner, A);
    assert_eq!(score.parts(A).kralj_ultimo, 10);
    for seat in [B, C, D] {
        assert_eq!(score.parts(seat).kralj_ultimo, 0);
    }

    // -10: the same, but B leads a pik to the last trick and wins it.
    let game = a_declares(&ultimo_deal(true), Tri, Some(Suit::Srce));
    let (game, score) = finished(
        game,
        [&ultimo_script_a(), &ultimo_script_b(false), &[], &[]],
    );
    assert_eq!(trick_of(&game, king), 12);
    assert_eq!(game.public().tricks()[11].winner, B);
    assert_eq!(score.parts(A).kralj_ultimo, -10);

    // +10 with a partner: C holds the called king and wins the last trick
    // with it. Both write the bonus.
    let game = a_declares(&ultimo_deal(false), Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), Some(C));
    let mut a_script = top_taroks(13);
    a_script.extend([karo(4), pik(8)]);
    let mut b_script = taroks(2..=5);
    b_script.extend([
        pik(7),
        pik(9),
        pik(10),
        fant(Suit::Pik),
        kaval(Suit::Pik),
        dama(Suit::Pik),
        karo(3),
        dama(Suit::Srce),
    ]);
    let mut c_script = taroks(6..=9);
    c_script.extend([
        srce(4),
        srce(3),
        srce(2),
        kriz(7),
        kriz(8),
        kriz(9),
        kriz(10),
        king,
    ]);
    let (game, score) = finished(game, [&a_script, &b_script, &c_script, &[]]);
    assert_eq!(trick_of(&game, king), 12);
    assert_eq!(game.public().tricks()[11].winner, C);
    assert_eq!(game.public().partner(), PublicPartner::With(C));
    assert_eq!(score.parts(A).kralj_ultimo, 10);
    assert_eq!(score.parts(C).kralj_ultimo, 10);
    assert_eq!(score.entry(A), score.entry(C));

    // 0 when the called king is played earlier: deal 1, first trick.
    let (game, _) = deal_1_partner();
    assert_eq!(trick_of(&game, kralj(Suit::Kriz)), 1);
    assert_eq!(game.score().unwrap().parts(A).kralj_ultimo, 0);

    // Never in a solo contract: the same last trick, no bonus.
    let game = a_declares(&ultimo_deal(true), SoloTri, None);
    let (game, score) = finished(game, [&ultimo_script_a(), &ultimo_script_b(true), &[], &[]]);
    assert_eq!(trick_of(&game, king), 12);
    assert_eq!(game.public().tricks()[11].winner, A);
    assert_eq!(score.parts(A).kralj_ultimo, 0);
}

#[test]
fn c39_silent_pagat_ultimo() {
    // +25: the declarer's pagat wins the last trick.
    let (game, score) = pagat_ultimo_own_wins();
    assert_eq!(trick_of(&game, PAGAT), 12);
    assert_eq!(game.public().tricks()[11].winner, A);
    assert_eq!(game.public().tricks()[11].cards[3], PAGAT);
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(A).pagat_ultimo, 25);
    for seat in [B, C, D] {
        assert_eq!(score.parts(seat).pagat_ultimo, 0);
    }

    // -25: the declarer's pagat is played to the last trick and loses it.
    let (game, score) = pagat_ultimo_own_loses();
    assert_eq!(trick_of(&game, PAGAT), 12);
    assert_eq!(game.public().tricks()[11].winner, D);
    assert!(game.private(A).hand().is_empty());
    assert_eq!(score.parts(A).pagat_ultimo, -25);

    // -25: an opponent's pagat wins the last trick.
    let (game, score) = trula_for_the_opponents();
    assert_eq!(trick_of(&game, PAGAT), 12);
    assert_eq!(game.public().tricks()[11].winner, B);
    assert_eq!(game.public().tricks()[11].cards[0], PAGAT);
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(A).pagat_ultimo, -25);

    // +25: an opponent's pagat is played to the last trick and loses it.
    let (game, score) = pagat_ultimo_opponent_loses();
    assert_eq!(trick_of(&game, PAGAT), 12);
    assert_eq!(game.public().tricks()[11].cards[0], PAGAT);
    assert_eq!(game.public().tricks()[11].winner, A);
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(A).pagat_ultimo, 25);

    // The pagat "loses" whenever it is not the card that takes the trick,
    // also when a player of its own side takes it.
    //
    // +25: C's pagat is played to the last trick and D takes the trick with
    // the XIX. A plays solo, so C and D are both opponents: an opponent's
    // pagat loses.
    let game = a_declares(&pagat_and_a_higher_tarok(D), SoloTri, None);
    let (game, score) = finished(game, pagat_and_a_higher_tarok_scripts(D));
    assert_eq!(tricks_won(&game), [9, 1, 1, 1]);
    assert_eq!(trick_of(&game, PAGAT), 12);
    assert_eq!(game.public().tricks()[11].cards[2], PAGAT);
    assert_eq!(game.public().tricks()[11].winner, D);
    assert_eq!(score.parts(A).pagat_ultimo, 25);

    // -25: the same hands in tri with križ called. C holds the križ kralj
    // and is A's partner, so the pagat is the declarer's side's own, and
    // the opponent D takes the last trick.
    let game = a_declares(&pagat_and_a_higher_tarok(D), Tri, Some(Suit::Kriz));
    assert_eq!(game.full().partner(), Some(C));
    let (game, score) = finished(game, pagat_and_a_higher_tarok_scripts(D));
    assert_eq!(game.public().tricks()[11].winner, D);
    assert_eq!(score.parts(A).pagat_ultimo, -25);
    assert_eq!(score.parts(C).pagat_ultimo, -25);

    // -25 (reading 3 of the Results; question 2 for the user): the partner
    // C plays the pagat to the last trick and the declarer A takes the
    // trick with the XIX. The side wins the trick, but not with the pagat.
    let game = a_declares(&pagat_and_a_higher_tarok(A), Tri, Some(Suit::Kriz));
    assert_eq!(game.full().partner(), Some(C));
    let (game, score) = finished(game, pagat_and_a_higher_tarok_scripts(A));
    assert_eq!(tricks_won(&game), [10, 1, 1, 0]);
    assert_eq!(trick_of(&game, PAGAT), 12);
    assert_eq!(game.public().tricks()[11].winner, A);
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(A).pagat_ultimo, -25);
    assert_eq!(score.parts(C).pagat_ultimo, -25);

    // 0 when the pagat falls earlier: trick 11 here, trick 6 in deal 1.
    let (game, score) = trula_for_a();
    assert_eq!(trick_of(&game, PAGAT), 11);
    assert_eq!(score.parts(A).pagat_ultimo, 0);
    let (game, _) = deal_1_partner();
    assert_eq!(trick_of(&game, PAGAT), 6);
    assert_eq!(game.score().unwrap().parts(A).pagat_ultimo, 0);
}

#[test]
fn c40_game_and_bonuses_are_independent() {
    // Deal 2: B loses solo tri with 31 points and has the whole trula.
    let (game, _) = deal_2_solo();
    let score = game.score().unwrap();
    let parts = score.parts(B);
    assert!(!score.declarer_won());
    assert_eq!((parts.game, parts.difference), (-40, -5));
    assert_eq!(parts.trula, 10);
    // The loss plus 10.
    assert_eq!(score.entry(B), -45 + 10);

    // A won game with a bonus on top: A wins solo ena and has the four
    // kings.
    let (_, score) = kings_for_a();
    let parts = score.parts(A);
    assert!(parts.game > 0);
    assert_eq!(parts.kralji, 10);

    // The entry is always the sum of the parts.
    for (_, score) in [
        trula_for_a(),
        trula_for_the_opponents(),
        kings_for_a(),
        kings_for_the_opponents(false),
        pagat_ultimo_own_wins(),
        pagat_ultimo_own_loses(),
    ] {
        for seat in Seat::ALL {
            let p = score.parts(seat);
            let sum = p.game
                + p.difference
                + p.trula
                + p.kralji
                + p.kralj_ultimo
                + p.pagat_ultimo
                + p.valat
                + p.mondfang;
            assert_eq!(score.entry(seat), sum);
            assert_eq!(p.entry(), sum);
        }
    }
    // A lost game with two bonuses against: every part is there.
    let (_, score) = trula_for_the_opponents();
    let parts = score.parts(A);
    assert!(parts.game < 0);
    assert_eq!(parts.trula, -10);
    assert_eq!(parts.pagat_ultimo, -25);
    assert_eq!(score.entry(A), parts.game + parts.difference - 10 - 25);
}

/// A holds škis, mond, XX-XII and the pagat: twelve taroks that win twelve
/// tricks from the lead.
fn all_twelve_for_a() -> Deal {
    let mut a = top_taroks(12);
    a.push(PAGAT);
    deal_with([&a, &[], &[], &[]], &low_talon())
}

/// B holds the same twelve taroks; A is the declarer.
fn all_twelve_for_b() -> Deal {
    let mut b = top_taroks(12);
    b.push(PAGAT);
    deal_with([&[], &b, &[], &[]], &low_talon())
}

/// Škis, mond, XX-XII from the top, then the pagat.
fn twelve_from_the_top() -> Vec<Card> {
    let mut script = top_taroks(12);
    script.push(PAGAT);
    script
}

fn only_valat(score: &Score, seat: Seat, value: i32) {
    let parts = score.parts(seat);
    assert_eq!(parts.valat, value);
    assert_eq!(
        [
            parts.game,
            parts.difference,
            parts.trula,
            parts.kralji,
            parts.kralj_ultimo,
            parts.pagat_ultimo,
            parts.mondfang
        ],
        [0; 7],
        "nothing else is scored"
    );
    assert_eq!(score.entry(seat), value);
}

#[test]
fn c41_silent_valat_scores_250_and_nothing_else() {
    // Alone, in solo ena.
    let game = a_declares(&all_twelve_for_a(), SoloEna, None);
    let (game, score) = finished(game, [&twelve_from_the_top(), &[], &[], &[]]);
    assert_eq!(tricks_won(&game), [12, 0, 0, 0]);
    // The trula is A's and the pagat wins the last trick: neither scores.
    assert_eq!(trick_of(&game, PAGAT), 12);
    only_valat(&score, A, 250);
    assert_eq!(score.entries(), [250, 0, 0, 0]);

    // With a partner, in tri: C holds the called srce kralj. The partner
    // writes the same.
    let game = a_declares(&all_twelve_for_a(), Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), Some(C));
    let (game, score) = finished(game, [&twelve_from_the_top(), &[], &[], &[]]);
    assert_eq!(tricks_won(&game), [12, 0, 0, 0]);
    only_valat(&score, A, 250);
    only_valat(&score, C, 250);
    assert_eq!(score.entries(), [250, 0, 250, 0]);

    // Eleven tricks are not a valat: the game is scored as usual.
    let (game, score) = trula_for_a();
    assert_eq!(tricks_won(&game), [11, 1, 0, 0]);
    assert_eq!(score.parts(A).valat, 0);
    assert!(score.parts(A).game > 0);
}

#[test]
fn c41_opponents_take_all_twelve_tricks_scores_minus_250() {
    // Answer A (D118). A plays solo ena; B trumps the first trick and takes
    // every trick.
    let game = a_declares(&all_twelve_for_b(), SoloEna, None);
    let mut b_script = vec![tarok(12), S];
    b_script.extend((13..=21).rev().map(tarok));
    b_script.push(PAGAT);
    let (game, score) = finished(game, [&[], &b_script, &[], &[]]);
    assert_eq!(tricks_won(&game), [0, 12, 0, 0]);
    // An opponent's pagat wins the last trick and the trula is theirs:
    // neither is scored.
    assert_eq!(trick_of(&game, PAGAT), 12);
    only_valat(&score, A, -250);
    assert_eq!(score.entries(), [-250, 0, 0, 0]);

    // With a partner, in tri: A calls karo, C holds the karo kralj. Both
    // write -250.
    let game = a_declares(&all_twelve_for_b(), Tri, Some(Suit::Karo));
    assert_eq!(game.full().partner(), Some(C));
    let (game, score) = finished(game, [&[], &b_script, &[], &[]]);
    assert_eq!(tricks_won(&game), [0, 12, 0, 0]);
    only_valat(&score, A, -250);
    only_valat(&score, C, -250);
    assert_eq!(score.entries(), [-250, 0, -250, 0]);

    // Eleven tricks for the opponents are not a valat against.
    let (game, score) = trula_for_the_opponents();
    assert_eq!(tricks_won(&game), [1, 11, 0, 0]);
    assert_eq!(score.parts(A).valat, 0);
    assert!(score.parts(A).game < 0);
}

#[test]
fn c42_mondfang_the_mond_player_writes_minus_20() {
    // An opponent's škis takes the declarer's mond.
    let deal = deal_with([&[MOND], &[S], &[], &[]], &low_talon());
    let game = a_declares(&deal, SoloEna, None);
    let (game, score) = finished(game, [&[MOND], &[S], &[], &[]]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    assert_eq!(game.public().tricks()[0].winner, B);
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(A).mondfang, -20);
    // Alone: nobody else writes it.
    for seat in [B, C, D] {
        assert_eq!(score.parts(seat).mondfang, 0);
        assert_eq!(score.entry(seat), 0);
    }
    let parts = score.parts(A);
    assert_eq!(score.entry(A), parts.entry());
    assert_eq!(
        parts.entry(),
        parts.game + parts.difference + parts.trula + parts.kralji + parts.pagat_ultimo - 20
    );

    // The partner's škis takes the declarer's mond: the declarer writes
    // -20 all the same, the partner does not.
    let deal = deal_with([&[MOND], &[], &[S, kralj(Suit::Srce)], &[]], &low_talon());
    let game = a_declares(&deal, Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), Some(C));
    let (game, score) = finished(game, [&[MOND], &[], &[S], &[]]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    assert_eq!(game.public().tricks()[0].winner, C);
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(A).mondfang, -20);
    assert_eq!(score.parts(C).mondfang, 0);
    assert_eq!(score.entry(A), score.entry(C) - 20);
    assert_eq!([score.entry(B), score.entry(D)], [0, 0]);

    // Škis, mond and pagat in one trick: the pagat wins it and the mond's
    // player still writes -20 (S22 a).
    let deal = deal_with([&[MOND], &[S], &[PAGAT], &[]], &low_talon());
    let game = a_declares(&deal, SoloEna, None);
    let (game, score) = finished(game, [&[MOND], &[S], &[PAGAT], &[]]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    assert!(game.public().tricks()[0].cards.contains(&PAGAT));
    assert_eq!(game.public().tricks()[0].winner, C);
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(A).mondfang, -20);
    for seat in [B, C, D] {
        assert_eq!(score.parts(seat).mondfang, 0);
    }

    // No mondfang when the mond is not in a trick with the škis.
    let (game, score) = trula_for_a();
    assert_eq!(skis_mond_trick(&game), None);
    for seat in Seat::ALL {
        assert_eq!(score.parts(seat).mondfang, 0);
    }
}

#[test]
fn c43_an_opponent_who_loses_the_mond_writes_minus_20() {
    // The declarer's škis takes B's mond.
    let deal = deal_with([&[S], &[MOND], &[], &[]], &low_talon());
    let game = a_declares(&deal, SoloEna, None);
    let (game, score) = finished(game, [&[S], &[MOND], &[], &[]]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    assert_eq!(game.public().tricks()[0].winner, A);
    assert_eq!(score.parts(A).valat, 0);
    // B is an opponent: no game entry, and -20 on its own.
    let parts = score.parts(B);
    assert_eq!((parts.game, parts.difference), (0, 0));
    assert_eq!(parts.mondfang, -20);
    assert_eq!(score.entry(B), -20);
    assert_eq!([score.entry(C), score.entry(D)], [0, 0]);
    assert_eq!(score.parts(A).mondfang, 0);

    // Another opponent's škis takes B's mond: B writes -20 again, C nothing.
    let deal = deal_with([&[tarok(2)], &[MOND], &[S], &[]], &low_talon());
    let game = a_declares(&deal, SoloEna, None);
    let (game, score) = finished(game, [&[tarok(2)], &[MOND], &[S], &[]]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    assert_eq!(game.public().tricks()[0].winner, C);
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.entry(B), -20);
    assert_eq!(score.entry(C), 0);
    assert_eq!(score.parts(A).mondfang, 0);

    // The written deal 3: B loses the mond to the declarer D.
    let (game, _) = deal_3_king_in_talon();
    assert_eq!(skis_mond_trick(&game), Some(1));
    assert_eq!(game.score().unwrap().entry(B), -20);
}

#[test]
fn c44_mond_in_a_packet_not_taken_costs_the_declarer_20() {
    // The mond lies in the first packet; A (solo tri, holding the škis)
    // takes the second.
    let talon = [MOND, karo(4), karo(3), srce(4), srce(3), srce(2)];
    let deal = deal_with([&[S], &[], &[], &[]], &talon);
    let mut game = game(&deal);
    declare(&mut game, A, SoloTri);
    exchange_back(&mut game, 1);
    finish(&mut game);
    let score = game.score().unwrap();
    assert_eq!(score.parts(A).valat, 0);
    assert!(game.full().opponents_pile().contains(MOND));
    assert_eq!(skis_mond_trick(&game), None);
    assert_eq!(score.parts(A).mondfang, -20);
    for seat in [B, C, D] {
        assert_eq!(score.entry(seat), 0);
    }

    // A takes the packet with the mond: no penalty.
    let mut game = common::flow::game(&deal);
    declare(&mut game, A, SoloTri);
    exchange(&mut game, 0, &[karo(4), karo(3), tarok(2)]);
    assert!(game.private(A).hand().contains(MOND));
    finish(&mut game);
    assert_eq!(game.score().unwrap().parts(A).mondfang, 0);

    // The called king lies in the talon too (criterion 23): A takes the
    // king's packet and wins a trick with the king. The rest of the talon,
    // with the mond, is A's: no penalty.
    let game = king_taken_and_led(false);
    assert!(game.full().talon_rest_won());
    assert!(game.full().declarer_pile().contains(MOND));
    let score = game.score().unwrap();
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(A).mondfang, 0);

    // The king loses its trick (criterion 24): the rest stays with the
    // opponents and the mond costs 20.
    let game = king_taken_and_led(true);
    assert!(!game.full().talon_rest_won());
    assert!(game.full().opponents_pile().contains(MOND));
    let score = game.score().unwrap();
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(A).mondfang, -20);
}

// ---------------------------------------------------------------------
// Partner games in which the partner's tricks decide (review of spec 005)
// ---------------------------------------------------------------------

/// A plays tri and calls srce. The partner C holds škis, mond, XX-XII and
/// the srce kralj and takes all twelve tricks; the declarer takes none.
fn partner_takes_all_twelve() -> (Game, Score) {
    let mut c = top_taroks(12);
    c.push(kralj(Suit::Srce));
    let deal = deal_with([&[], &[], &c, &[]], &low_talon());
    let game = a_declares(&deal, Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), Some(C));
    // C overtakes A's first lead with the XII, leads the taroks from the
    // top and the srce kralj last.
    let mut c_script = vec![tarok(12), S];
    c_script.extend((13..=21).rev().map(tarok));
    c_script.push(kralj(Suit::Srce));
    finished(game, [&[], &[], &c_script, &[]])
}

#[test]
fn c41_valat_counts_the_tricks_of_the_side_not_of_the_declarer() {
    // The declarer takes no trick and the partner takes all twelve: a
    // valat for the side, +250 for both, not -250.
    let (game, score) = partner_takes_all_twelve();
    assert_eq!(tricks_won(&game), [0, 0, 12, 0]);
    only_valat(&score, A, 250);
    only_valat(&score, C, 250);
    assert_eq!(score.entries(), [250, 0, 250, 0]);

    // The declarer takes no trick, the partner takes one and the opponents
    // eleven: no valat at all. B, an opponent, holds škis, XX-XI and the
    // pagat. C, the partner, holds the mond and the called srce kralj and
    // wins one trick with the mond.
    let mut b = vec![S];
    b.extend((11..=20).rev().map(tarok));
    b.push(PAGAT);
    let deal = deal_with([&[], &b, &[MOND, kralj(Suit::Srce)], &[]], &low_talon());
    let game = a_declares(&deal, Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), Some(C));
    // A leads a tarok; B plays the XI and C takes the trick with the mond.
    let (game, score) = finished(game, [&[], &[tarok(11)], &[MOND], &[]]);
    let won = tricks_won(&game);
    assert_eq!(won[A.index()], 0, "the declarer takes no trick");
    assert!(won[C.index()] >= 1, "the partner takes a trick");
    assert!(
        won[B.index()] + won[D.index()] >= 1,
        "the opponents take a trick"
    );
    for seat in Seat::ALL {
        assert_eq!(score.parts(seat).valat, 0);
    }
    assert_ne!(score.parts(A).game, 0);
    assert_eq!(score.parts(A).game, score.parts(C).game);
}

#[test]
fn c38_kralj_ultimo_follows_the_side_that_takes_the_last_trick() {
    // Criterion 38 as written: the called king is played to the last trick
    // and the declarer's side wins that trick, here with another card.
    //
    // A plays tri and calls srce: škis, mond, XX-XII and the karo 4. The
    // partner C holds the srce kralj and keeps it for the last trick. A
    // leads nine taroks, loses the tenth trick to B's karo 3, trumps B's
    // srce dama with the XII and leads the XIII to the last trick, on
    // which C plays the srce kralj.
    let mut a = top_taroks(12);
    a.push(karo(4));
    let mut b = taroks(2..=5);
    b.extend([
        karo(3),
        dama(Suit::Srce),
        pik(7),
        pik(8),
        pik(9),
        pik(10),
        fant(Suit::Pik),
        kaval(Suit::Pik),
    ]);
    let mut c = taroks(6..=9);
    c.extend([
        kralj(Suit::Srce),
        srce(4),
        srce(3),
        srce(2),
        kriz(7),
        kriz(8),
        kriz(9),
        kriz(10),
    ]);
    let mut d = taroks([10, 11, 1]);
    d.extend([
        srce(1),
        fant(Suit::Srce),
        kaval(Suit::Srce),
        dama(Suit::Pik),
        kralj(Suit::Pik),
        fant(Suit::Kriz),
        kaval(Suit::Kriz),
        dama(Suit::Kriz),
        kralj(Suit::Kriz),
    ]);
    let deal = whole_deal([&a, &b, &c, &d], karo_talon());
    let game = a_declares(&deal, Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), Some(C));

    let mut a_script = top_taroks(14);
    a_script.extend([karo(4), tarok(12), tarok(13)]);
    let mut b_script = taroks(2..=5);
    b_script.extend([
        pik(7),
        pik(8),
        pik(9),
        pik(10),
        fant(Suit::Pik),
        karo(3),
        dama(Suit::Srce),
        kaval(Suit::Pik),
    ]);
    let mut c_script = taroks(6..=9);
    c_script.extend([
        kriz(7),
        kriz(8),
        kriz(9),
        kriz(10),
        srce(4),
        srce(3),
        srce(2),
        kralj(Suit::Srce),
    ]);
    let (game, score) = finished(game, [&a_script, &b_script, &c_script, &[]]);
    assert_eq!(tricks_won(&game), [11, 1, 0, 0]);
    let last = game.public().tricks()[11];
    assert_eq!(last.cards[0], tarok(13));
    assert_eq!(last.cards[2], kralj(Suit::Srce));
    assert_eq!(last.winner, A);
    // The king did not take the trick; its side did: +10 for both.
    assert_eq!(score.parts(A).kralj_ultimo, 10);
    assert_eq!(score.parts(C).kralj_ultimo, 10);
    assert_eq!(
        [score.parts(B).kralj_ultimo, score.parts(D).kralj_ultimo],
        [0, 0]
    );
}

#[test]
fn c42_a_partner_who_loses_the_mond_pays_alone() {
    // A plays tri and calls srce; the partner C holds the mond and the
    // srce kralj. A leads the tarok II, the opponent B plays the škis and
    // C plays the mond. C writes -20; the declarer does not.
    let deal = deal_with(
        [&[tarok(2)], &[S], &[MOND, kralj(Suit::Srce)], &[]],
        &low_talon(),
    );
    let game = a_declares(&deal, Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), Some(C));
    let (game, score) = finished(game, [&[tarok(2)], &[S], &[MOND], &[]]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    assert_eq!(game.public().tricks()[0].winner, B);
    assert_eq!(score.parts(A).valat, 0);
    assert_eq!(score.parts(C).mondfang, -20);
    assert_eq!(score.parts(A).mondfang, 0);
    assert_eq!([score.parts(B).mondfang, score.parts(D).mondfang], [0, 0]);
    // The same entry apart from the mondfang.
    assert_eq!(score.entry(C), score.entry(A) - 20);
    assert_eq!([score.entry(B), score.entry(D)], [0, 0]);
}

#[test]
fn c36_c37_trula_and_kralji_count_the_tricks_of_the_partner() {
    // A plays tri and calls križ; the partner C wins one trick, with the
    // križ kralj. The three other kings are in A's tricks. All four are
    // with the declarer's side: +10 for both.
    let game = a_declares(&pagat_and_a_higher_tarok(A), Tri, Some(Suit::Kriz));
    assert_eq!(game.full().partner(), Some(C));
    let (game, score) = finished(game, pagat_and_a_higher_tarok_scripts(A));
    assert_eq!(tricks_won(&game), [10, 1, 1, 0]);
    assert!(game.public().tricks_won(C).contains(kralj(Suit::Kriz)));
    for suit in [Suit::Srce, Suit::Karo, Suit::Pik] {
        assert!(game.public().tricks_won(A).contains(kralj(suit)));
    }
    assert_eq!(score.parts(A).kralji, 10);
    assert_eq!(score.parts(C).kralji, 10);

    // The trula the same way. A plays tri and calls srce. The partner C
    // holds the škis and the srce kralj; A holds the mond, the pagat, XI,
    // XII, XIV-XX and the srce 4; B holds II-X and D the XIII.
    // The škis is in a trick of C, the mond and the pagat in tricks of A.
    let mut a = vec![MOND, PAGAT, tarok(11), tarok(12), srce(4)];
    a.extend((14..=20).map(tarok));
    let talon = [
        karo(4),
        karo(3),
        karo(2),
        karo(1),
        fant(Suit::Karo),
        kaval(Suit::Karo),
    ];
    let deal = deal_with([&a, &[], &[S, kralj(Suit::Srce)], &[tarok(13)]], &talon);
    let game = a_declares(&deal, Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), Some(C));
    // A leads the XI; C takes it with the škis and leads the pik 7, which
    // A trumps with the XII. Then A leads the mond and the taroks from the
    // top, the pagat in the eleventh trick and the srce 4 last, which B
    // wins with its last srce.
    let mut a_script = vec![tarok(11), tarok(12), MOND];
    a_script.extend((14..=20).rev().map(tarok));
    a_script.extend([PAGAT, srce(4)]);
    let (game, score) = finished(game, [&a_script, &[], &[S, pik(7)], &[]]);
    assert_eq!(tricks_won(&game), [10, 1, 1, 0]);
    assert!(game.public().tricks_won(C).contains(S));
    assert!(game.public().tricks_won(A).contains(MOND));
    assert!(game.public().tricks_won(A).contains(PAGAT));
    assert_eq!(trick_of(&game, PAGAT), 11);
    assert_eq!(score.parts(A).valat, 0, "the opponents take a trick");
    assert_eq!(skis_mond_trick(&game), None);
    assert_eq!(score.parts(A).trula, 10);
    assert_eq!(score.parts(C).trula, 10);
}

// ---------------------------------------------------------------------
// Reading 2 of the Results (D118, open with the user): a valat cancels
// every mondfang. These four tests fix what the code does now in each
// case. A ruling of the user changes exactly these tests (and the three
// switches at the top of `tests/common/second_scorer.rs`).
// ---------------------------------------------------------------------

#[test]
fn reading2_valat_and_an_opponent_who_loses_the_mond() {
    // A (solo ena) holds škis, XX-XI and the pagat and takes all twelve
    // tricks. B's only tarok is the mond: it falls under the škis in the
    // first trick. Now: +250 for A and nothing for B.
    let mut a = vec![S];
    a.extend((11..=20).rev().map(tarok));
    a.push(PAGAT);
    let deal = deal_with([&a, &[MOND], &taroks(2..=10), &[]], &low_talon());
    let game = a_declares(&deal, SoloEna, None);
    let (game, score) = finished(game, [&a, &[MOND], &[], &[]]);
    assert_eq!(tricks_won(&game), [12, 0, 0, 0]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    only_valat(&score, A, 250);
    assert_eq!(
        score.parts(B).mondfang,
        0,
        "reading 2: no mondfang in a valat"
    );
    assert_eq!(score.entries(), [250, 0, 0, 0]);
}

#[test]
fn reading2_valat_and_the_declarers_mond_taken_by_the_partner() {
    // A plays tri and calls srce: mond, XX-XI and the pagat. The partner
    // C holds the škis and the srce kralj. A leads the mond, C must play
    // the škis. The two take all twelve tricks. Now: +250 for both and no
    // -20 for A.
    let mut a = vec![MOND];
    a.extend((11..=20).rev().map(tarok));
    a.push(PAGAT);
    let deal = deal_with([&a, &[], &[S, kralj(Suit::Srce)], &[]], &low_talon());
    let game = a_declares(&deal, Tri, Some(Suit::Srce));
    assert_eq!(game.full().partner(), Some(C));
    // C leads the srce kralj to the second trick; A trumps it with the XX
    // and leads the rest from the top, the pagat last.
    let (game, score) = finished(game, [&a, &[], &[S, kralj(Suit::Srce)], &[]]);
    assert_eq!(tricks_won(&game), [11, 0, 1, 0]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    assert_eq!(game.public().tricks()[0].winner, C);
    only_valat(&score, A, 250);
    only_valat(&score, C, 250);
    assert_eq!(
        score.parts(A).mondfang,
        0,
        "reading 2: no mondfang in a valat"
    );
    assert_eq!(score.entries(), [250, 0, 250, 0]);
}

/// B, an opponent, holds škis, XX-XI and the pagat and takes all twelve
/// tricks against the declarer A: the XI on A's first lead, eight taroks
/// from the XX down, the škis, the XII and the pagat.
fn b_takes_all_twelve_script() -> Vec<Card> {
    let mut script = vec![tarok(11)];
    script.extend((13..=20).rev().map(tarok));
    script.extend([S, tarok(12), PAGAT]);
    script
}

fn b_strong_hand() -> Vec<Card> {
    let mut b = vec![S];
    b.extend((11..=20).rev().map(tarok));
    b.push(PAGAT);
    b
}

#[test]
fn reading2_valat_against_and_the_declarers_lost_mond() {
    // A (solo ena) holds the mond and II-X. B takes all twelve tricks and
    // catches the mond with the škis in the tenth. Now: -250 for A and no
    // further -20.
    let deal = deal_with([&[MOND], &b_strong_hand(), &[], &[]], &low_talon());
    let game = a_declares(&deal, SoloEna, None);
    let (game, score) = finished(game, [&[tarok(2)], &b_takes_all_twelve_script(), &[], &[]]);
    assert_eq!(tricks_won(&game), [0, 12, 0, 0]);
    assert_eq!(skis_mond_trick(&game), Some(9));
    only_valat(&score, A, -250);
    assert_eq!(
        score.parts(A).mondfang,
        0,
        "reading 2: no mondfang in a valat"
    );
    assert_eq!(score.entries(), [-250, 0, 0, 0]);
}

#[test]
fn reading2_valat_against_and_the_mond_left_in_the_talon() {
    // The mond lies in the first packet; A (solo tri) takes the second. B
    // takes all twelve tricks. Now: -250 for A and no -20 for the mond.
    let talon = [MOND, karo(4), karo(3), srce(4), srce(3), srce(2)];
    let deal = deal_with([&[], &b_strong_hand(), &[], &[]], &talon);
    let mut game = common::flow::game(&deal);
    declare(&mut game, A, SoloTri);
    exchange_back(&mut game, 1);
    // B: the XI on A's first lead, then škis, XX-XII, pagat.
    let mut b_script = vec![tarok(11), S];
    b_script.extend((12..=20).rev().map(tarok));
    b_script.push(PAGAT);
    let (game, score) = finished(game, [&[], &b_script, &[], &[]]);
    assert_eq!(tricks_won(&game), [0, 12, 0, 0]);
    assert!(game.full().opponents_pile().contains(MOND));
    assert!(!game.public().taken_cards().contains(MOND));
    only_valat(&score, A, -250);
    assert_eq!(
        score.parts(A).mondfang,
        0,
        "reading 2: no mondfang in a valat"
    );
    assert_eq!(score.entries(), [-250, 0, 0, 0]);
}

#[test]
fn reading2_valat_against_and_an_opponents_mond_taken_by_an_opponent() {
    // A fourth case of the same reading. A plays solo ena. C holds škis,
    // XX-XI and the pagat; B's only tarok is the mond. A leads a tarok, B
    // must play the mond and C takes it with the škis. C takes all twelve
    // tricks. Now: -250 for A and nothing for B.
    let deal = deal_with([&[], &[MOND], &b_strong_hand(), &[]], &low_talon());
    let game = a_declares(&deal, SoloEna, None);
    let mut c_script = vec![S];
    c_script.extend((11..=20).rev().map(tarok));
    c_script.push(PAGAT);
    let (game, score) = finished(game, [&[], &[MOND], &c_script, &[]]);
    assert_eq!(tricks_won(&game), [0, 0, 12, 0]);
    assert_eq!(skis_mond_trick(&game), Some(0));
    only_valat(&score, A, -250);
    assert_eq!(
        score.parts(B).mondfang,
        0,
        "reading 2: no mondfang in a valat"
    );
    assert_eq!(score.entries(), [-250, 0, 0, 0]);
}
