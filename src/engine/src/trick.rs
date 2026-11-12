//! Trick winner and legal cards for normal contracts.
//!
//! Rules source: research/01 B3 "Play" and the baseline in rule-switches.md.
//! Klop, berač and barvni valat have other rules (must overtake, pagat last,
//! taroks do not beat the led suit). Those come with later specs.

use crate::card::Card;
use crate::cardset::CardSet;

/// The position (0-3) of the card that wins the trick. The cards are in the
/// order of play; position 0 is the led card.
///
/// - Škis, mond and pagat all in the trick: the pagat wins if it falls
///   last (spec 013, D131; toggle S40, of which this is the Default).
///   Otherwise the škis wins, as the highest tarok.
/// - Otherwise the highest tarok wins. The škis is the highest tarok.
/// - With no tarok, the highest card of the led suit wins.
pub fn winner(cards: &[Card; 4]) -> usize {
    let position = |wanted: Card| cards.iter().position(|&c| c == wanted);
    if let (Some(skis), Some(mond), Some(pagat)) = (
        position(Card::SKIS),
        position(Card::MOND),
        position(Card::PAGAT),
    ) && pagat_falls_last(skis, mond, pagat)
    {
        return pagat;
    }

    let mut best = 0;
    for (position, &card) in cards.iter().enumerate().skip(1) {
        if beats(card, cards[best]) {
            best = position;
        }
    }
    best
}

/// Škis, mond and pagat are in one trick, at these positions (0-3) in the
/// order of play. True if the pagat "falls last".
///
/// Reading C12 (decided, D132): the pagat was played after both the škis
/// and the mond; a fourth card may still follow it. The reading that was
/// not chosen, "only as the fourth card of the trick", is `pagat == 3`.
const fn pagat_falls_last(skis: usize, mond: usize, pagat: usize) -> bool {
    pagat > skis && pagat > mond
}

/// True if `card`, played later, takes the trick from `best`. `best` is the
/// led card or a card that already beat it, so it is a tarok or a card of
/// the led suit.
fn beats(card: Card, best: Card) -> bool {
    match (card.is_tarok(), best.is_tarok()) {
        // Two taroks: a higher index is a higher tarok.
        (true, true) => card.index() > best.index(),
        // A tarok beats any suit card.
        (true, false) => true,
        // A suit card never beats a tarok.
        (false, true) => false,
        // Two suit cards: `best` is of the led suit. A card of another suit
        // never wins. Within a suit a higher index is a stronger card.
        (false, false) => card.suit() == best.suit() && card.index() > best.index(),
    }
}

/// The cards of `hand` that may be played. `led` is the first card of the
/// trick, or `None` when the player is on the lead.
///
/// - On the lead: any card.
/// - A suit is led: a card of that suit. If void: a tarok. If void and
///   without taroks: any card.
/// - A tarok is led: a tarok. Without taroks: any card.
///
/// There is no duty to overtake in normal contracts.
pub fn legal_cards(hand: CardSet, led: Option<Card>) -> CardSet {
    let Some(led) = led else {
        return hand;
    };
    if let Some(suit) = led.suit() {
        let following = hand & CardSet::suit(suit);
        if !following.is_empty() {
            return following;
        }
    }
    let taroks = hand & CardSet::TAROKS;
    if taroks.is_empty() { hand } else { taroks }
}
