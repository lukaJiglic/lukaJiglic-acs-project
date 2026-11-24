//! Deals built so that the wanted event cannot be avoided: one seat holds
//! a hand that wins by force, and its cards are played in a written order.
//! The other seats play their lowest legal card.
//!
//! Moved here from `tests/scoring.rs` with spec 009, so the scoring tests
//! of flow B (`tests/scoring_b.rs`) use the same deals. To say something
//! in the round of announcements of one of these deals, wrap the call in
//! `common::flow::with_round`.

use tarok_engine::Contract::{SoloEna, SoloTri};
use tarok_engine::{Action, Card, Contract, Deal, Game, Phase, Rank, Score, Seat, Suit};

use super::flow::{
    A, act, deal_with, declare, exchange_back, finish, game, play_scripted, whole_deal,
};
use super::{SUITS, court, kralj, pip, tarok};

pub const S: Card = Card::SKIS;
pub const MOND: Card = Card::MOND;
pub const PAGAT: Card = Card::PAGAT;

/// Taroks by number, in the order given.
pub fn taroks(numbers: impl IntoIterator<Item = u8>) -> Vec<Card> {
    numbers.into_iter().map(tarok).collect()
}

/// Škis, then the taroks from XXI down to `lowest`.
pub fn top_taroks(lowest: u8) -> Vec<Card> {
    let mut cards = vec![S];
    cards.extend((lowest..=21).rev().map(tarok));
    cards
}

pub fn srce(n: u8) -> Card {
    pip(Suit::Srce, n)
}
pub fn karo(n: u8) -> Card {
    pip(Suit::Karo, n)
}
pub fn pik(n: u8) -> Card {
    pip(Suit::Pik, n)
}
pub fn kriz(n: u8) -> Card {
    pip(Suit::Kriz, n)
}
pub fn fant(suit: Suit) -> Card {
    court(suit, Rank::Fant)
}
pub fn kaval(suit: Suit) -> Card {
    court(suit, Rank::Kaval)
}
pub fn dama(suit: Suit) -> Card {
    court(suit, Rank::Dama)
}

/// The six low karo: a talon without a king, a trula card or a srce.
pub fn karo_talon() -> [Card; 6] {
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
pub fn a_declares(deal: &Deal, contract: Contract, call: Option<Suit>) -> Game {
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

pub fn finished(mut game: Game, scripts: [&[Card]; 4]) -> (Game, Score) {
    play_scripted(&mut game, scripts);
    finish(&mut game);
    let score = game.score().expect("the deal is over");
    (game, score)
}

/// How many tricks each seat (A, B, C, D) won.
pub fn tricks_won(game: &Game) -> [usize; 4] {
    Seat::ALL.map(|seat| {
        game.public()
            .tricks()
            .iter()
            .filter(|t| t.winner == seat)
            .count()
    })
}

/// The trick (1-12) a card was played to.
pub fn trick_of(game: &Game, card: Card) -> usize {
    1 + game
        .public()
        .tricks()
        .iter()
        .position(|t| t.cards.contains(&card))
        .unwrap_or_else(|| panic!("{card} was not played"))
}

// ---------------------------------------------------------------------
// The built deals
// ---------------------------------------------------------------------

/// A holds škis, mond, XX-XIII, the pagat and the srce 4. B holds five low
/// taroks and the seven other srce. A leads the taroks from the top, then
/// the pagat, then the srce 4. A wins eleven tricks; B wins the last one
/// with its last srce.
pub fn trula_for_a() -> (Game, Score) {
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
pub fn trula_for_the_opponents() -> (Game, Score) {
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
pub fn kings_for_a() -> (Game, Score) {
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
pub fn kings_for_the_opponents(srce_king_in_talon: bool) -> (Game, Score) {
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
pub fn ultimo_deal(a_holds_king: bool) -> Deal {
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
pub fn ultimo_script_a() -> Vec<Card> {
    let mut script = top_taroks(13);
    script.extend([karo(4), kralj(Suit::Srce)]);
    script
}

/// B's cards in `ultimo_deal(true)`. With `lead_srce` B keeps the srce dama
/// for the last trick, else the pik kaval.
pub fn ultimo_script_b(lead_srce: bool) -> Vec<Card> {
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
pub fn pagat_ultimo_own_wins() -> (Game, Score) {
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

pub fn pagat_ultimo_own_loses() -> (Game, Score) {
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

pub fn pagat_ultimo_opponent_loses() -> (Game, Score) {
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
pub fn pagat_and_a_higher_tarok(xix: Seat) -> Deal {
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
pub fn pagat_and_a_higher_tarok_scripts(xix: Seat) -> [&'static [Card]; 4] {
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
pub const fn with_last(first: [Card; 11], last: Card) -> [Card; 12] {
    let mut all = [last; 12];
    let mut i = 0;
    while i < 11 {
        all[i] = first[i];
        i += 1;
    }
    all
}

/// The talon used by the mondfang deals: no tarok, no king.
pub fn low_talon() -> [Card; 6] {
    [srce(4), srce(3), srce(2), srce(1), karo(4), karo(3)]
}

/// The trick that holds both the škis and the mond, if there is one.
pub fn skis_mond_trick(game: &Game) -> Option<usize> {
    game.public()
        .tricks()
        .iter()
        .position(|t| t.cards.contains(&S) && t.cards.contains(&MOND))
}
