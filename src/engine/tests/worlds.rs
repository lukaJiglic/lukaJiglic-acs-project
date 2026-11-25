//! Spec 009, criterion 16 with the standing rule of D151: the round of
//! announcements played in four worlds at once.
//!
//! The four worlds are four deals that look the same from outside and
//! differ only in who holds the called king: the declarer, or one of the
//! three other seats. A world stays alive while everything said so far is
//! legal in it. What every seat can work out strictly is what is true in
//! every alive world. The search goes through every order of things said,
//! up to a number of announcements and kontras, and holds the engine to
//! that at every node:
//!
//! 1. The public partner and the public side of every seat are the same in
//!    all alive worlds.
//! 2. They are exactly what is true in all alive worlds: a seat is shown
//!    as the partner (or the declarer as alone) when one world is alive,
//!    and as an opponent when the world in which it holds the king is
//!    dead. Nothing more is shown and nothing that follows is left out.
//! 3. What is shown never changes later.
//! 4. What a seat knows about the other seats is the public state, its own
//!    side, and for a holder of the called king everything. Nothing else:
//!    a private conclusion is not used (D151, item 2).
//! 5. A seat's legal set is the same in all alive worlds in which it does
//!    not hold the called king.
//!
//! The search came from the second review pass of spec 009, which ran it
//! deeper (to eight things said) on the code before D151. This file is the
//! same search with the equality of point 2, which holds since D151. It is
//! independent of the engine's own rules for what an action shows: it
//! reads only which actions are legal in which world.
//!
//! The deal is `common::flow::round_deal`. One seat (B) holds the pagat in
//! every world. Only the round is searched, not the play after it.

mod common;

use common::flow::{A, B, C, act, declare, exchange_back_open, game, round_deal};
use tarok_engine::Contract::{self, Dve, Ena, Tri};
use tarok_engine::{Action, Bonus, Game, Phase, PublicPartner, Seat, Suit};

/// World `w`: the seat with index `w` holds the called king. `None`: dead.
type Worlds = [Option<Game>; 4];

/// What all know: the partner, and the side of each seat.
type Shown = (PublicPartner, [Option<bool>; 4]);

struct Search<'a> {
    declarer: Seat,
    /// The bonuses that may be announced in this search.
    bonuses: &'a [Bonus],
    nodes: u64,
    leaves: u64,
    /// Nodes with the sides hidden and a seat shown as an opponent.
    with_an_opponent_shown: u64,
    /// Nodes in which a declarer without the king sees two seats shown as
    /// opponents (D151, item 2).
    declarer_could_conclude: u64,
}

/// The declarer plays `contract` and calls srce; the called king is in the
/// hand of a different seat in each world.
fn start(declarer: Seat, contract: Contract) -> Worlds {
    Seat::ALL.map(|king| {
        let mut game = game(&round_deal(king));
        declare(&mut game, declarer, contract);
        act(&mut game, declarer, Action::CallKing(Suit::Srce));
        exchange_back_open(&mut game, 0);
        assert_eq!(game.phase(), Phase::Announcements);
        Some(game)
    })
}

fn shown(game: &Game) -> Shown {
    let public = game.public();
    (public.partner(), Seat::ALL.map(|seat| public.side_of(seat)))
}

fn said(path: &[(Seat, Action)]) -> String {
    path.iter()
        .filter(|(_, action)| *action != Action::Done)
        .map(|(seat, action)| format!("{}: {action:?}", seat.index()))
        .collect::<Vec<_>>()
        .join(", ")
}

impl Search<'_> {
    fn check(&mut self, worlds: &Worlds, parent: Option<Shown>, path: &[(Seat, Action)]) {
        let declarer = self.declarer;
        let alive: Vec<Seat> = Seat::ALL
            .into_iter()
            .filter(|seat| worlds[seat.index()].is_some())
            .collect();
        assert!(!alive.is_empty(), "the true deal is always alive");
        let world = |holder: Seat| worlds[holder.index()].as_ref().expect("an alive world");
        let first = world(alive[0]);
        let here = shown(first);

        // 1. The same in every alive world.
        for &holder in &alive {
            let game = world(holder);
            assert_eq!(shown(game), here, "after [{}]", said(path));
            assert_eq!(game.to_act(), first.to_act());
            assert_eq!(game.phase(), first.phase());
            assert_eq!(
                game.full().partner(),
                Some(holder).filter(|&seat| seat != declarer),
                "world {holder:?}"
            );
        }

        // 2. Exactly what is true in all alive worlds.
        let partner = match alive[..] {
            [holder] if holder == declarer => PublicPartner::Alone,
            [holder] => PublicPartner::With(holder),
            _ => PublicPartner::NotKnown,
        };
        assert_eq!(here.0, partner, "alive {alive:?} after [{}]", said(path));
        for seat in Seat::ALL {
            let side = if seat == declarer {
                Some(true)
            } else if !alive.contains(&seat) {
                Some(false)
            } else if alive.len() == 1 {
                Some(true)
            } else {
                None
            };
            assert_eq!(
                here.1[seat.index()],
                side,
                "{seat:?}, alive {alive:?} after [{}]",
                said(path)
            );
        }
        // Nothing said shows that the declarer has a partner without
        // showing who it is.
        if alive.len() > 1 {
            assert!(alive.contains(&declarer), "after [{}]", said(path));
            let opponents = here.1.iter().filter(|&&side| side == Some(false)).count();
            self.with_an_opponent_shown += u64::from(opponents > 0);
            self.declarer_could_conclude += u64::from(opponents == 2);
        }

        // 3. What is shown stays.
        if let Some((partner, sides)) = parent {
            if partner != PublicPartner::NotKnown {
                assert_eq!(here.0, partner, "after [{}]", said(path));
            }
            for seat in Seat::ALL {
                if sides[seat.index()].is_some() {
                    assert_eq!(here.1[seat.index()], sides[seat.index()]);
                }
            }
        }

        for seat in Seat::ALL {
            // 4. What the seat knows, in every alive world.
            for &holder in &alive {
                let game = world(holder);
                for other in Seat::ALL {
                    let knows = if other == declarer {
                        Some(true)
                    } else if holder == seat {
                        // A holder of the called king knows every side:
                        // every other seat but the declarer is an opponent.
                        Some(other == seat)
                    } else if other == seat {
                        // Its own side: it does not hold the king.
                        Some(false)
                    } else {
                        // Only what all know. Also for a declarer without
                        // the king that sees two seats shown as opponents.
                        here.1[other.index()]
                    };
                    assert_eq!(
                        game.private(seat).knows_side_of(other),
                        knows,
                        "{seat:?} about {other:?}, the king with {holder:?}, after [{}]",
                        said(path)
                    );
                }
            }
            // 5. Its legal set, in the worlds it cannot tell apart.
            if first.phase() == Phase::Announcements && first.to_act() == Some(seat) {
                let mut sets = alive
                    .iter()
                    .filter(|&&holder| holder != seat)
                    .map(|&holder| world(holder).legal_actions());
                if let Some(one) = sets.next() {
                    for other in sets {
                        assert_eq!(other, one, "{seat:?} after [{}]", said(path));
                    }
                }
            }
        }
    }

    /// Every action that is legal in an alive world, then the same from
    /// there. `left`: how many more announcements and kontras.
    fn go(
        &mut self,
        worlds: &Worlds,
        left: usize,
        parent: Option<Shown>,
        path: &mut Vec<(Seat, Action)>,
    ) {
        self.nodes += 1;
        self.check(worlds, parent, path);
        let first = worlds.iter().flatten().next().expect("an alive world");
        if first.phase() != Phase::Announcements {
            self.leaves += 1;
            return;
        }
        let here = shown(first);
        let seat = first.to_act().expect("a seat speaks");
        let mut options: Vec<Action> = Vec::new();
        for game in worlds.iter().flatten() {
            for action in game.legal_actions().iter() {
                if !options.contains(&action) {
                    options.push(action);
                }
            }
        }
        for action in options {
            let allowed = match action {
                Action::Done => true,
                Action::Announce(bonus) => left > 0 && self.bonuses.contains(&bonus),
                _ => left > 0,
            };
            if !allowed {
                continue;
            }
            // The worlds in which the action is legal stay alive.
            let next: Worlds = Seat::ALL.map(|holder| {
                let game = worlds[holder.index()].as_ref()?;
                game.legal_actions().contains(action).then(|| {
                    let mut game = game.clone();
                    game.apply(action).expect("a legal action");
                    game
                })
            });
            path.push((seat, action));
            let left = if action == Action::Done {
                left
            } else {
                left - 1
            };
            self.go(&next, left, Some(here), path);
            path.pop();
        }
    }
}

/// Runs one search and prints its size.
fn search(declarer: Seat, contract: Contract, things: usize, bonuses: &[Bonus]) -> u64 {
    let mut search = Search {
        declarer,
        bonuses,
        nodes: 0,
        leaves: 0,
        with_an_opponent_shown: 0,
        declarer_could_conclude: 0,
    };
    search.go(&start(declarer, contract), things, None, &mut Vec::new());
    println!(
        "four worlds, declarer {declarer:?}, {contract:?}, up to {things} things said, {} \
         bonuses: {} nodes, {} ends of the round; {} nodes with the sides hidden and a seat \
         shown as an opponent, {} of them with two such seats",
        bonuses.len(),
        search.nodes,
        search.leaves,
        search.with_an_opponent_shown,
        search.declarer_could_conclude
    );
    assert!(search.with_an_opponent_shown > 0);
    assert!(search.declarer_could_conclude > 0);
    search.nodes
}

#[test]
fn c16_four_worlds_every_order_of_four_things_with_all_five_bonuses() {
    // The declarer is forehand and plays tri.
    let nodes = search(A, Tri, 4, &Bonus::ALL);
    assert!(nodes > 100_000, "{nodes} nodes");
}

#[test]
fn c16_four_worlds_every_order_of_six_things_with_two_bonuses() {
    // Deeper, with the two bonuses that matter most for the sides: trula
    // (any seat) and kralj ultimo (the holder of the called king).
    let nodes = search(A, Tri, 6, &[Bonus::Trula, Bonus::KraljUltimo]);
    assert!(nodes > 100_000, "{nodes} nodes");
}

#[test]
fn c16_four_worlds_with_another_declarer_and_another_contract() {
    // The declarer is not forehand: B plays dve and speaks first, then C,
    // D and A. And C plays ena. The round has the same rules in every
    // contract that calls a king.
    let bonuses = [Bonus::Trula, Bonus::Kralji, Bonus::KraljUltimo];
    let nodes = search(B, Dve, 4, &bonuses) + search(C, Ena, 4, &bonuses);
    assert!(nodes > 50_000, "{nodes} nodes");
}
