//! A whole deal of a normal contract: bidding, contract choice, king call,
//! talon exchange, twelve tricks, score.
//!
//! `Game` is a state machine. At each step one seat is to act
//! ([`Game::to_act`]), [`Game::legal_actions`] lists what it may do and
//! [`Game::apply`] does it. An action that is not legal returns an error
//! and changes nothing.
//!
//! What a seat may know is kept apart in three views:
//!
//! - [`Game::public`]: what all four seats see.
//! - [`Game::private`]: what one seat sees in addition: its hand and, for
//!   the declarer, its discards.
//! - [`Game::full`]: everything, also the true partner and the hidden
//!   discards. For tests, for the score and later for the critic. A player
//!   or an actor network must not read it.
//!
//! The rules are those of spec 005 (flow A). Announcements and kontra come
//! with spec 009; klop, berač and the higher contracts with spec 010.

use std::fmt;

use crate::action::{Action, ActionSet, Packet};
use crate::bidding::{BidEvent, Bidding};
use crate::card::{Card, Rank, Suit};
use crate::cardset::CardSet;
use crate::contract::Contract;
use crate::deal::{Deal, Seat};
use crate::play::{PlayState, Trick};
use crate::score::{self, Outcome, Score};
use crate::talon;

/// Where the deal stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Phase {
    /// The auction.
    Bidding,
    /// The declarer names the contract: the winning bid or a higher one.
    ContractChoice,
    /// The declarer calls a king (tri, dve, ena only).
    KingCall,
    /// The talon is shown; the declarer takes a packet.
    TalonPick,
    /// The declarer puts cards down, one per action.
    Discard,
    /// The twelve tricks.
    Play,
    /// The deal is finished and has a score.
    Over,
}

/// What everyone knows about the declarer's partner.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PublicPartner {
    /// Not known yet: no king is called, or the called king has not been
    /// played.
    NotKnown,
    /// The declarer plays alone and all know it: a solo contract, the
    /// called king in the shown talon, or the declarer played the called
    /// king.
    Alone,
    /// The called king was played by this seat.
    With(Seat),
}

/// What a public event is. Every event is seen by all four seats.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EventKind {
    /// The deal starts. Always the first event.
    Start,
    /// A call in the auction: a bid, or `None` for a pass.
    Bid(Option<Contract>),
    /// The declarer names the contract to play.
    Contract(Contract),
    /// The declarer calls the king of a suit.
    King(Suit),
    /// A talon card is turned up. Six in a row, in the talon's order.
    TalonShown(Card),
    /// The declarer takes a packet.
    TalonTaken(Packet),
    /// The declarer puts down a tarok, face up (S14 a). Other discards are
    /// hidden and are not events.
    DiscardShown(Card),
    /// A card is played to a trick.
    Play(Card),
}

/// One public event of a deal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Event {
    pub kind: EventKind,
    /// The seat that did it. `None` for the start and the talon turned up.
    pub seat: Option<Seat>,
    /// The trick number, 1 to 12, of a played card. 0 for everything else.
    pub trick: u8,
    /// The position in the trick, 1 to 4, of a played card. 0 otherwise.
    pub position: u8,
}

/// Room for the events of a deal: the sequence limit of the encoding
/// (`docs/encoding.md` section 3). The longest deal of this slice has 71.
pub const MAX_EVENTS: usize = 128;

/// Room for the actions of a deal. The longest deal of this slice has 64:
/// 13 calls, the contract, a packet, 1 discard and 48 cards (solo ena
/// after the longest auction; a test walks every auction).
pub const MAX_ACTIONS: usize = 96;

/// Why an action was refused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameError {
    /// The deal is over.
    Over,
    /// The action is not among the legal actions of the seat to act.
    NotLegal(Action),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameError::Over => write!(f, "the deal is over"),
            GameError::NotLegal(action) => write!(f, "{action:?} is not legal now"),
        }
    }
}

impl std::error::Error for GameError {}

/// One deal from the first bid to the score. It holds no heap data, so a
/// clone is cheap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Game {
    dealer: Seat,
    phase: Phase,
    /// The hands up to the end of the discards. From then on the hands are
    /// in `play`.
    hands: [CardSet; 4],
    talon: [Card; talon::TALON_SIZE],
    bidding: Bidding,
    declarer: Option<Seat>,
    contract: Option<Contract>,
    called: Option<Suit>,
    /// The seat that held the called king when it was called, if that is
    /// not the declarer. Hidden from the public view until the king falls.
    partner: Option<Seat>,
    /// The called king has been played to a trick.
    called_king_played: bool,
    taken: Option<Packet>,
    discards: CardSet,
    play: Option<PlayState>,
    /// The public events so far. Only the first `n_events` entries count.
    events: [Event; MAX_EVENTS],
    n_events: usize,
    /// The slots of the actions taken so far, hidden discards included.
    /// Only the first `n_actions` entries count.
    actions: [u8; MAX_ACTIONS],
    n_actions: usize,
}

/// The first event of every deal, and the filler of the unused entries.
const START: Event = Event {
    kind: EventKind::Start,
    seat: None,
    trick: 0,
    position: 0,
};

impl Game {
    /// A new deal. `dealer` deals; the next seat is forehand. The hands go
    /// to the seats by seat index, whoever deals.
    pub fn new(deal: &Deal, dealer: Seat) -> Game {
        Game {
            dealer,
            phase: Phase::Bidding,
            hands: deal.hands(),
            talon: deal.talon(),
            bidding: Bidding::new(dealer.next()),
            declarer: None,
            contract: None,
            called: None,
            partner: None,
            called_king_played: false,
            taken: None,
            discards: CardSet::EMPTY,
            play: None,
            events: [START; MAX_EVENTS],
            n_events: 1,
            actions: [0; MAX_ACTIONS],
            n_actions: 0,
        }
    }

    /// A new deal of the cards of a seed (see [`Deal::from_seed`]).
    pub fn from_seed(seed: u64, dealer: Seat) -> Game {
        Game::new(&Deal::from_seed(seed), dealer)
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    /// The seat that acts next, or `None` when the deal is over.
    pub fn to_act(&self) -> Option<Seat> {
        match self.phase {
            Phase::Bidding => self.bidding.to_speak(),
            Phase::ContractChoice | Phase::KingCall | Phase::TalonPick | Phase::Discard => {
                self.declarer
            }
            Phase::Play => self.play.as_ref().map(PlayState::to_move),
            Phase::Over => None,
        }
    }

    /// What the seat to act may do. Empty when the deal is over.
    pub fn legal_actions(&self) -> ActionSet {
        let mut legal = ActionSet::EMPTY;
        match self.phase {
            Phase::Bidding => {
                if self.bidding.may_pass() {
                    legal.insert(Action::Pass);
                }
                for contract in Contract::NORMAL {
                    if self.bidding.may_bid(contract) {
                        legal.insert(Action::Bid(contract));
                    }
                }
            }
            Phase::ContractChoice => {
                // The winning bid or any higher normal contract (S3 a).
                if let Some((_, winning)) = self.bidding.winner() {
                    for contract in Contract::NORMAL {
                        if contract >= winning {
                            legal.insert(Action::Bid(contract));
                        }
                    }
                }
            }
            Phase::KingCall => {
                // All four suits, also one whose king the declarer holds.
                for suit in Suit::ALL {
                    legal.insert(Action::CallKing(suit));
                }
            }
            Phase::TalonPick => {
                if let Some(size) = self.packet_size() {
                    for packet in &Packet::ALL[..talon::packet_count(size)] {
                        legal.insert(Action::TakePacket(*packet));
                    }
                }
            }
            Phase::Discard => {
                if let Some(declarer) = self.declarer {
                    legal = ActionSet::discards(talon::discardable(self.hand(declarer)));
                }
            }
            Phase::Play => {
                if let Some(play) = &self.play {
                    legal = ActionSet::plays(play.legal_cards());
                }
            }
            Phase::Over => {}
        }
        legal
    }

    /// The seat to act takes an action. On an error nothing changes.
    pub fn apply(&mut self, action: Action) -> Result<(), GameError> {
        if self.phase == Phase::Over {
            return Err(GameError::Over);
        }
        if !self.legal_actions().contains(action) {
            return Err(GameError::NotLegal(action));
        }
        let seat = self.to_act();
        match (self.phase, action) {
            (Phase::Bidding, Action::Pass) => {
                self.log(EventKind::Bid(None), seat);
                self.bid(None);
            }
            (Phase::Bidding, Action::Bid(contract)) => {
                self.log(EventKind::Bid(Some(contract)), seat);
                self.bid(Some(contract));
            }
            (Phase::ContractChoice, Action::Bid(contract)) => {
                self.contract = Some(contract);
                self.log(EventKind::Contract(contract), seat);
                // After the choice the talon is shown, unless a king is
                // called first.
                self.phase = if contract.calls_king() {
                    Phase::KingCall
                } else {
                    self.log_talon();
                    Phase::TalonPick
                };
            }
            (Phase::KingCall, Action::CallKing(suit)) => {
                self.log(EventKind::King(suit), seat);
                self.log_talon();
                self.called = Some(suit);
                let king = Card::court(suit, Rank::Kralj);
                // The partner is the seat that holds the king now. A
                // declarer who holds it, or a king in the talon: no partner.
                self.partner = Seat::ALL
                    .into_iter()
                    .find(|&seat| self.hands[seat.index()].contains(king))
                    .filter(|&holder| Some(holder) != self.declarer);
                self.phase = Phase::TalonPick;
            }
            (Phase::TalonPick, Action::TakePacket(packet)) => {
                self.log(EventKind::TalonTaken(packet), seat);
                self.taken = Some(packet);
                if let Some(declarer) = self.declarer {
                    let taken = self.taken_cards();
                    self.hands[declarer.index()] = self.hands[declarer.index()] | taken;
                }
                self.phase = Phase::Discard;
            }
            (Phase::Discard, Action::Discard(card)) => {
                if let Some(declarer) = self.declarer {
                    self.hands[declarer.index()].remove(card);
                }
                self.discards.insert(card);
                // A discarded tarok is shown to all; any other card is not.
                if card.is_tarok() {
                    self.log(EventKind::DiscardShown(card), seat);
                }
                if Some(self.discards.len() as usize) == self.packet_size() {
                    // Every hand is back to twelve. Forehand leads,
                    // whoever the declarer is.
                    let play = PlayState::new(self.hands, self.dealer.next())
                        .expect("four separate hands of twelve after the exchange");
                    self.play = Some(play);
                    self.phase = Phase::Play;
                }
            }
            (Phase::Play, Action::Play(card)) => {
                if Some(card) == self.called_king() {
                    self.called_king_played = true;
                }
                if let Some(play) = &self.play {
                    let trick = play.trick_history().len() + 1;
                    let position = play.current_trick().len() + 1;
                    self.log_play(card, seat, trick, position);
                }
                if let Some(play) = &mut self.play {
                    play.play(card).expect("a legal card is accepted");
                    if play.is_over() {
                        self.phase = Phase::Over;
                    }
                }
            }
            // A legal action always belongs to the phase, so nothing else
            // gets here.
            _ => return Err(GameError::NotLegal(action)),
        }
        self.push_action(action);
        Ok(())
    }

    /// The score, once the deal is over.
    pub fn score(&self) -> Option<Score> {
        if self.phase != Phase::Over {
            return None;
        }
        let full = self.full();
        let talon_with_opponents = if full.talon_rest_won() {
            CardSet::EMPTY
        } else {
            full.talon_rest()
        };
        Some(score::score(&Outcome {
            contract: self.contract?,
            declarer: self.declarer?,
            partner: self.partner,
            called_king: self.called_king(),
            tricks: self.play.as_ref()?.trick_history(),
            declarer_pile: full.declarer_pile(),
            talon_with_opponents,
        }))
    }

    /// What all four seats see.
    pub fn public(&self) -> PublicView<'_> {
        PublicView { game: self }
    }

    /// What one seat sees besides the public state.
    pub fn private(&self, seat: Seat) -> PrivateView<'_> {
        PrivateView { game: self, seat }
    }

    /// Everything, the hidden things too. Not for players.
    pub fn full(&self) -> FullView<'_> {
        FullView { game: self }
    }

    /// Adds a public event that is not a played card.
    ///
    /// Reading B12-7 (Batch 12, open): the contract, the called king, the
    /// taken packet and a shown discard carry the declarer's seat. The
    /// start and the six shown talon cards carry none.
    fn log(&mut self, kind: EventKind, seat: Option<Seat>) {
        self.push_event(Event {
            kind,
            seat,
            trick: 0,
            position: 0,
        });
    }

    /// Adds the six talon cards, turned up, in the talon's order.
    fn log_talon(&mut self) {
        for card in self.talon {
            self.log(EventKind::TalonShown(card), None);
        }
    }

    fn log_play(&mut self, card: Card, seat: Option<Seat>, trick: usize, position: usize) {
        // A trick number is at most 12 and a position at most 4.
        self.push_event(Event {
            kind: EventKind::Play(card),
            seat,
            trick: trick as u8,
            position: position as u8,
        });
    }

    /// No deal of the rules built so far has more than [`MAX_EVENTS`]
    /// events; a test walks the longest. A rule that adds events (spec 009)
    /// may pass the limit. Then this panics: a log that silently stopped
    /// would give the network a sequence with its end cut off.
    fn push_event(&mut self, event: Event) {
        assert!(
            self.n_events < MAX_EVENTS,
            "a deal has more than {MAX_EVENTS} public events (MAX_EVENTS in game.rs, the \
             sequence limit of docs/encoding.md): the limit must be raised"
        );
        self.events[self.n_events] = event;
        self.n_events += 1;
    }

    /// Adds the slot of an action to the history. Panics past
    /// [`MAX_ACTIONS`], for the same reason as [`Game::push_event`]: a
    /// history that silently stopped could not be replayed.
    fn push_action(&mut self, action: Action) {
        assert!(
            self.n_actions < MAX_ACTIONS,
            "a deal has more than {MAX_ACTIONS} actions (MAX_ACTIONS in game.rs): the limit \
             must be raised"
        );
        // The slot of an action is below 160 and fits a byte.
        self.actions[self.n_actions] = action.slot() as u8;
        self.n_actions += 1;
    }

    fn bid(&mut self, bid: Option<Contract>) {
        self.bidding.call(bid);
        if let Some((declarer, _)) = self.bidding.winner() {
            self.declarer = Some(declarer);
            self.phase = Phase::ContractChoice;
        }
    }

    fn hand(&self, seat: Seat) -> CardSet {
        match &self.play {
            Some(play) => play.hand(seat),
            None => self.hands[seat.index()],
        }
    }

    fn packet_size(&self) -> Option<usize> {
        self.contract?.packet_size()
    }

    fn called_king(&self) -> Option<Card> {
        self.called.map(|suit| Card::court(suit, Rank::Kralj))
    }

    /// The talon is face up from the talon pick on.
    fn talon_shown(&self) -> bool {
        matches!(
            self.phase,
            Phase::TalonPick | Phase::Discard | Phase::Play | Phase::Over
        )
    }

    fn talon_set(&self) -> CardSet {
        self.talon.iter().copied().collect()
    }

    fn taken_cards(&self) -> CardSet {
        let cards = self
            .packet_size()
            .zip(self.taken)
            .and_then(|(size, packet)| talon::packet_cards(&self.talon, size, packet));
        cards.unwrap_or(&[]).iter().copied().collect()
    }

    fn tricks(&self) -> &[Trick] {
        match &self.play {
            Some(play) => play.trick_history(),
            None => &[],
        }
    }
}

/// The public state: what all four seats see. See [`Game::public`].
///
/// `Debug` is written by hand and prints the public state only. A derived
/// one would print the whole `Game` behind the view, hands included.
#[derive(Clone, Copy)]
pub struct PublicView<'a> {
    game: &'a Game,
}

impl fmt::Debug for PublicView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PublicView")
            .field("phase", &self.phase())
            .field("to_act", &self.to_act())
            .field("dealer", &self.dealer())
            .field("bids", &self.bids())
            .field("declarer", &self.declarer())
            .field("contract", &self.contract())
            .field("called_suit", &self.called_suit())
            .field("talon", &self.talon())
            .field("taken_packet", &self.taken_packet())
            .field("discards_made", &self.discards_made())
            .field("shown_discards", &self.shown_discards())
            .field("partner", &self.partner())
            .field("tricks", &self.tricks())
            .field("current_trick", &self.current_trick())
            .field("leader", &self.leader())
            .field("hand_sizes", &Seat::ALL.map(|seat| self.hand_size(seat)))
            .finish()
    }
}

impl<'a> PublicView<'a> {
    pub fn phase(&self) -> Phase {
        self.game.phase
    }

    /// The seat that acts next, or `None` when the deal is over.
    pub fn to_act(&self) -> Option<Seat> {
        self.game.to_act()
    }

    pub fn dealer(&self) -> Seat {
        self.game.dealer
    }

    /// The seat after the dealer. It has the highest priority in the
    /// bidding and leads the first trick.
    pub fn forehand(&self) -> Seat {
        self.game.dealer.next()
    }

    /// The public events of the deal so far, in the order they happened.
    /// The first is always [`EventKind::Start`]. The tokens of the encoding
    /// are these events, one token each.
    pub fn events(&self) -> &'a [Event] {
        &self.game.events[..self.game.n_events]
    }

    /// True once the called king has been played to a trick.
    pub fn called_king_played(&self) -> bool {
        self.game.called_king_played
    }

    /// The calls of the auction, in order.
    pub fn bids(&self) -> &'a [BidEvent] {
        self.game.bidding.calls()
    }

    pub fn has_passed(&self, seat: Seat) -> bool {
        self.game.bidding.has_passed(seat)
    }

    /// The highest bid so far and its seat.
    pub fn highest_bid(&self) -> Option<(Seat, Contract)> {
        self.game.bidding.highest()
    }

    /// The winner of the auction, once it is over.
    pub fn declarer(&self) -> Option<Seat> {
        self.game.declarer
    }

    /// The bid that won the auction.
    pub fn winning_bid(&self) -> Option<Contract> {
        self.game.bidding.winner().map(|(_, bid)| bid)
    }

    /// The contract the declarer plays: the winning bid or a higher one.
    pub fn contract(&self) -> Option<Contract> {
        self.game.contract
    }

    pub fn called_suit(&self) -> Option<Suit> {
        self.game.called
    }

    /// The king of the called suit.
    pub fn called_king(&self) -> Option<Card> {
        self.game.called_king()
    }

    /// The six talon cards in the order they lie, once they are shown.
    pub fn talon(&self) -> Option<[Card; 6]> {
        self.game.talon_shown().then_some(self.game.talon)
    }

    /// Cards per talon packet (3, 2 or 1), once the contract is named.
    pub fn packet_size(&self) -> Option<usize> {
        self.game.packet_size()
    }

    /// The cards of a talon packet in the talon's order, once the talon is
    /// shown. `None` if the contract has no packet with this index.
    pub fn packet(&self, packet: Packet) -> Option<&'a [Card]> {
        if !self.game.talon_shown() {
            return None;
        }
        talon::packet_cards(&self.game.talon, self.game.packet_size()?, packet)
    }

    /// The packet the declarer took.
    pub fn taken_packet(&self) -> Option<Packet> {
        self.game.taken
    }

    /// The cards of the packet the declarer took. Empty before that.
    pub fn taken_cards(&self) -> CardSet {
        self.game.taken_cards()
    }

    /// How many cards the declarer has put down.
    pub fn discards_made(&self) -> usize {
        self.game.discards.len() as usize
    }

    /// The discarded taroks: they are shown to all (S14 a). The other
    /// discards are hidden.
    pub fn shown_discards(&self) -> CardSet {
        self.game.discards & CardSet::TAROKS
    }

    /// True once the talon is shown and the called king lies in it.
    pub fn king_in_talon(&self) -> bool {
        self.game.talon_shown()
            && self
                .called_king()
                .is_some_and(|king| self.game.talon.contains(&king))
    }

    /// What everyone knows about the partner.
    pub fn partner(&self) -> PublicPartner {
        let Some(contract) = self.game.contract else {
            return PublicPartner::NotKnown;
        };
        if !contract.calls_king() || self.king_in_talon() {
            return PublicPartner::Alone;
        }
        if !self.game.called_king_played {
            return PublicPartner::NotKnown;
        }
        match self.game.partner {
            Some(partner) => PublicPartner::With(partner),
            None => PublicPartner::Alone,
        }
    }

    /// The finished tricks, first to last.
    pub fn tricks(&self) -> &'a [Trick] {
        self.game.tricks()
    }

    /// The cards on the table, in the order of play.
    pub fn current_trick(&self) -> &'a [Card] {
        match &self.game.play {
            Some(play) => play.current_trick(),
            None => &[],
        }
    }

    /// The seat that led the current trick; after a finished trick, its
    /// winner. `None` before the play.
    pub fn leader(&self) -> Option<Seat> {
        self.game.play.as_ref().map(PlayState::leader)
    }

    /// The cards of the tricks a seat has won.
    pub fn tricks_won(&self, seat: Seat) -> CardSet {
        match &self.game.play {
            Some(play) => play.pile(seat),
            None => CardSet::EMPTY,
        }
    }

    /// How many cards a seat holds.
    pub fn hand_size(&self, seat: Seat) -> u32 {
        self.game.hand(seat).len()
    }
}

/// What one seat sees besides the public state. See [`Game::private`].
///
/// `Debug` is written by hand and prints the seat, its hand and its own
/// discards. Nothing of another seat.
#[derive(Clone, Copy)]
pub struct PrivateView<'a> {
    game: &'a Game,
    seat: Seat,
}

impl fmt::Debug for PrivateView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PrivateView")
            .field("seat", &self.seat())
            .field("hand", &self.hand())
            .field("discards", &self.discards())
            .finish()
    }
}

impl PrivateView<'_> {
    pub fn seat(&self) -> Seat {
        self.seat
    }

    /// The cards the seat holds.
    pub fn hand(&self) -> CardSet {
        self.game.hand(self.seat)
    }

    /// The cards the seat has put down: all of them for the declarer,
    /// nothing for anyone else.
    pub fn discards(&self) -> CardSet {
        if self.game.declarer == Some(self.seat) {
            self.game.discards
        } else {
            CardSet::EMPTY
        }
    }

    /// True if this seat is the declarer's partner: it held the called king
    /// when the king was called. The seat itself knows that from its hand,
    /// long before the others do.
    pub fn is_called_partner(&self) -> bool {
        self.game.partner == Some(self.seat)
    }
}

/// Everything about the deal, hidden things included. See [`Game::full`].
/// Its `Debug` prints the whole game. So does the `Debug` of [`Game`].
#[derive(Clone, Copy, Debug)]
pub struct FullView<'a> {
    game: &'a Game,
}

impl FullView<'_> {
    /// The cards each seat holds now.
    pub fn hands(&self) -> [CardSet; 4] {
        Seat::ALL.map(|seat| self.game.hand(seat))
    }

    pub fn hand(&self, seat: Seat) -> CardSet {
        self.game.hand(seat)
    }

    /// The six talon cards in the order they lie, shown or not.
    pub fn talon(&self) -> [Card; 6] {
        self.game.talon
    }

    /// The true partner: the seat that held the called king when it was
    /// called. `None` before the call, in a solo, when the declarer held
    /// the king and when the king lay in the talon.
    pub fn partner(&self) -> Option<Seat> {
        self.game.partner
    }

    /// True for the declarer and the true partner.
    pub fn on_declarer_side(&self, seat: Seat) -> bool {
        self.game.declarer == Some(seat) || self.game.partner == Some(seat)
    }

    /// All discards, the hidden ones too.
    pub fn discards(&self) -> CardSet {
        self.game.discards
    }

    /// Every action taken so far, in order, the hidden discards included.
    /// Replaying them on the same deal with the same dealer gives this
    /// game again.
    pub fn actions(&self) -> impl Iterator<Item = Action> + '_ {
        // Every stored slot is the slot of an action: nothing is dropped.
        self.action_slots()
            .iter()
            .filter_map(|&slot| Action::from_slot(usize::from(slot)))
    }

    /// The slots of the actions taken so far (`docs/encoding.md` section 5).
    pub fn action_slots(&self) -> &[u8] {
        &self.game.actions[..self.game.n_actions]
    }

    /// The talon cards the declarer did not take. All six before the pick.
    pub fn talon_rest(&self) -> CardSet {
        self.game.talon_set() - self.game.taken_cards()
    }

    /// True if the rest of the talon has gone to the declarer: the called
    /// king lay in the talon, the declarer took its packet, played the
    /// king and won that trick (S12 a).
    pub fn talon_rest_won(&self) -> bool {
        let Some(king) = self.game.called_king() else {
            return false;
        };
        self.game.taken_cards().contains(king)
            && self
                .game
                .tricks()
                .iter()
                .find(|trick| trick.cards.contains(&king))
                .is_some_and(|trick| Some(trick.winner) == self.game.declarer)
    }

    /// The cards that count for the declarer's side so far: the tricks of
    /// the declarer and the partner, the discards, and the rest of the
    /// talon if it was won.
    pub fn declarer_pile(&self) -> CardSet {
        let mut pile = self.game.discards;
        for trick in self.game.tricks() {
            if self.on_declarer_side(trick.winner) {
                pile = pile | trick.cards.into_iter().collect();
            }
        }
        if self.talon_rest_won() {
            pile = pile | self.talon_rest();
        }
        pile
    }

    /// The cards that count for the opponents so far: their tricks, and
    /// from the talon pick on the rest of the talon unless the declarer
    /// won it.
    pub fn opponents_pile(&self) -> CardSet {
        let mut pile = CardSet::EMPTY;
        for trick in self.game.tricks() {
            if !self.on_declarer_side(trick.winner) {
                pile = pile | trick.cards.into_iter().collect();
            }
        }
        if self.game.taken.is_some() && !self.talon_rest_won() {
            pile = pile | self.talon_rest();
        }
        pile
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh() -> Game {
        Game::from_seed(0, Seat::ALL[0])
    }

    /// The logs take exactly their limit.
    #[test]
    fn the_logs_have_room_up_to_their_limits() {
        let mut game = fresh();
        // The start event is there from the beginning.
        for _ in 1..MAX_EVENTS {
            game.push_event(START);
        }
        assert_eq!(game.n_events, MAX_EVENTS);
        for _ in 0..MAX_ACTIONS {
            game.push_action(Action::Pass);
        }
        assert_eq!(game.n_actions, MAX_ACTIONS);
    }

    /// A full event log is a panic that names the limit, not a log that
    /// silently stops (spec 006, review item D11).
    #[test]
    #[should_panic(expected = "more than 128 public events")]
    fn the_event_log_panics_when_it_is_full() {
        let mut game = fresh();
        for _ in 0..MAX_EVENTS {
            game.push_event(START);
        }
    }

    #[test]
    #[should_panic(expected = "more than 96 actions")]
    fn the_action_history_panics_when_it_is_full() {
        let mut game = fresh();
        for _ in 0..=MAX_ACTIONS {
            game.push_action(Action::Pass);
        }
    }
}
