//! Spec 009, criteria 11-16: kontra in the round of announcements.
//!
//! Seats: A is forehand (seat 0), then B, C, D; D deals. The tests use
//! `common::flow::round_deal`: A declares, B holds the pagat, C holds the
//! srce kralj unless the test moves it. In tri with srce called, C is the
//! hidden partner and B and D are the opponents.

mod common;

use common::flow::{
    A, B, C, D, act, levels, may_by_seat, pass_round, round_deal, round_game, say,
    seeded_game_b_with,
};
use tarok_engine::Contract::{SoloTri, Tri};
use tarok_engine::{
    Action, ActionSet, Bonus, Card, Deal, Game, GameError, Phase, PublicPartner, Seat, Suit, Target,
};

fn may(game: &Game, action: Action) -> bool {
    game.legal_actions().contains(action)
}

fn announce(bonus: Bonus) -> Action {
    Action::Announce(bonus)
}

fn kontra(target: Target) -> Action {
    Action::Kontra(target)
}

/// `seat` takes one kontra action and exactly that target rises by one
/// level.
fn raise(game: &mut Game, seat: Seat, target: Target) {
    let before = levels(game);
    act(game, seat, kontra(target));
    let mut expected = before;
    expected[target.index()] += 1;
    assert_eq!(levels(game), expected, "{seat:?} raises {target:?}");
}

#[test]
fn c11_six_targets_four_levels_one_action_raises_one_level() {
    // The six targets in the order of the slots 82-87.
    assert_eq!(
        Target::ALL,
        [
            Target::Game,
            Target::Trula,
            Target::Kralji,
            Target::KraljUltimo,
            Target::PagatUltimo,
            Target::Valat
        ]
    );
    for (index, target) in Target::ALL.into_iter().enumerate() {
        assert_eq!(target.index(), index);
        assert_eq!(kontra(target).slot(), 82 + index);
    }

    // A plays solo tri: the sides are public. A announces trula. B, an
    // opponent, and A take the game and the trula up the four levels:
    // kontra, re, sub, mort.
    let mut game = round_game(C, SoloTri);
    assert_eq!(levels(&game), [0; 6]);
    say(&mut game, A, &[announce(Bonus::Trula)]);
    assert_eq!(levels(&game), [0; 6], "an announcement raises nothing");
    // Kontra.
    raise(&mut game, B, Target::Game);
    raise(&mut game, B, Target::Trula);
    assert_eq!(levels(&game), [1, 1, 0, 0, 0, 0]);
    act(&mut game, B, Action::Done);
    say(&mut game, C, &[]);
    say(&mut game, D, &[]);
    // Re.
    raise(&mut game, A, Target::Game);
    raise(&mut game, A, Target::Trula);
    assert_eq!(levels(&game), [2, 2, 0, 0, 0, 0]);
    act(&mut game, A, Action::Done);
    // Sub.
    raise(&mut game, B, Target::Trula);
    raise(&mut game, B, Target::Game);
    assert_eq!(levels(&game), [3, 3, 0, 0, 0, 0]);
    act(&mut game, B, Action::Done);
    say(&mut game, C, &[]);
    say(&mut game, D, &[]);
    // Mort.
    raise(&mut game, A, Target::Game);
    assert_eq!(levels(&game), [4, 3, 0, 0, 0, 0]);
    raise(&mut game, A, Target::Trula);
    assert_eq!(levels(&game), [4, 4, 0, 0, 0, 0]);
    act(&mut game, A, Action::Done);
    pass_round(&mut game);
    assert_eq!(game.phase(), Phase::Play);
    // The levels stay through the play.
    assert_eq!(levels(&game), [4, 4, 0, 0, 0, 0]);
    assert_eq!(game.public().kontra_level(Target::Game), 4);

    // Each of the five bonuses has its own ladder. A announces valat and
    // kralji, B pagat ultimo; in tri C, the partner, kralj ultimo.
    let mut game = round_game(C, Tri);
    say(
        &mut game,
        A,
        &[
            announce(Bonus::Trula),
            announce(Bonus::Kralji),
            announce(Bonus::Valat),
        ],
    );
    raise(&mut game, B, Target::Trula);
    raise(&mut game, B, Target::Kralji);
    raise(&mut game, B, Target::Valat);
    act(&mut game, B, announce(Bonus::PagatUltimo));
    act(&mut game, B, Action::Done);
    // C announces kralj ultimo and is the public partner from then on.
    act(&mut game, C, announce(Bonus::KraljUltimo));
    raise(&mut game, C, Target::PagatUltimo);
    act(&mut game, C, Action::Done);
    raise(&mut game, D, Target::KraljUltimo);
    assert_eq!(levels(&game), [0, 1, 1, 1, 1, 1]);
}

#[test]
fn c12_kontra_and_sub_by_the_other_side_re_and_mort_by_the_owner() {
    // A plays solo tri against B, C and D. B announces kralji: the bonus
    // belongs to the opponents.
    let target = Target::Kralji;
    let mut game = round_game(C, SoloTri);
    // The game belongs to the declarer's side: kontra by an opponent only.
    assert_eq!(
        may_by_seat(&game, kontra(Target::Game)),
        [false, true, true, true]
    );
    say(&mut game, A, &[]);
    act(&mut game, B, announce(Bonus::Kralji));
    assert_eq!(game.public().announced(Bonus::Kralji), Some(B));
    // Kontra is said by the other side: not by the announcer, not by its
    // side.
    assert!(!may(&game, kontra(target)));
    act(&mut game, B, Action::Done);
    assert!(!may(&game, kontra(target)), "C is on B's side");
    say(&mut game, C, &[]);
    assert!(!may(&game, kontra(target)), "D is on B's side");
    say(&mut game, D, &[]);
    assert!(may(&game, kontra(target)), "A is the other side");
    raise(&mut game, A, target);
    // Re is said by the owning side: not by A.
    assert!(!may(&game, kontra(target)));
    act(&mut game, A, Action::Done);
    assert!(may(&game, kontra(target)), "B may say re");
    say(&mut game, B, &[]);
    assert!(may(&game, kontra(target)), "C may say re");
    raise(&mut game, C, target);
    assert!(
        !may(&game, kontra(target)),
        "sub is not for the owning side"
    );
    act(&mut game, C, Action::Done);
    assert!(!may(&game, kontra(target)));
    say(&mut game, D, &[]);
    // Sub by the other side.
    assert!(may(&game, kontra(target)));
    raise(&mut game, A, target);
    assert!(!may(&game, kontra(target)));
    act(&mut game, A, Action::Done);
    say(&mut game, B, &[]);
    say(&mut game, C, &[]);
    // Mort by the owning side.
    assert!(may(&game, kontra(target)));
    raise(&mut game, D, target);
    assert_eq!(game.public().kontra_level(target), 4);

    // The game: after an opponent's kontra only the declarer may raise it.
    let mut game = round_game(C, SoloTri);
    say(&mut game, A, &[]);
    raise(&mut game, B, Target::Game);
    assert!(!may(&game, kontra(Target::Game)));
    act(&mut game, B, Action::Done);
    assert!(!may(&game, kontra(Target::Game)));
    say(&mut game, C, &[]);
    assert!(!may(&game, kontra(Target::Game)));
    say(&mut game, D, &[]);
    assert!(may(&game, kontra(Target::Game)));
    raise(&mut game, A, Target::Game);
    act(&mut game, A, Action::Done);
    // Sub by any opponent, mort by the declarer.
    say(&mut game, B, &[]);
    assert!(may(&game, kontra(Target::Game)));
    raise(&mut game, C, Target::Game);
    act(&mut game, C, Action::Done);
    assert!(!may(&game, kontra(Target::Game)));
    say(&mut game, D, &[]);
    raise(&mut game, A, Target::Game);
    assert_eq!(game.public().kontra_level(Target::Game), 4);
}

#[test]
fn c13_a_bonus_not_announced_cannot_be_kontrad_and_mort_cannot_be_raised() {
    // Nothing is announced: no seat may kontra a bonus, in a partner game
    // and in a solo.
    for game in [
        round_game(C, Tri),
        round_game(A, Tri),
        round_game(C, SoloTri),
    ] {
        for target in Target::ALL {
            if target == Target::Game {
                continue;
            }
            assert_eq!(may_by_seat(&game, kontra(target)), [false; 4], "{target:?}");
        }
        let mut game = game;
        assert_eq!(
            game.apply(kontra(Target::Trula)),
            Err(GameError::NotLegal(kontra(Target::Trula)))
        );
    }
    // Only the announced one: A announces trula, B may kontra the trula
    // but not the kralji.
    let mut game = round_game(C, SoloTri);
    say(&mut game, A, &[announce(Bonus::Trula)]);
    assert!(may(&game, kontra(Target::Trula)));
    for target in [
        Target::Kralji,
        Target::KraljUltimo,
        Target::PagatUltimo,
        Target::Valat,
    ] {
        assert!(!may(&game, kontra(target)), "{target:?}");
    }

    // Mort cannot be raised. A announces valat; B and A go up to mort.
    let mut game = round_game(C, SoloTri);
    say(&mut game, A, &[announce(Bonus::Valat)]);
    say(&mut game, B, &[kontra(Target::Valat), kontra(Target::Game)]);
    say(&mut game, C, &[]);
    say(&mut game, D, &[]);
    say(&mut game, A, &[kontra(Target::Valat), kontra(Target::Game)]);
    say(&mut game, B, &[kontra(Target::Valat), kontra(Target::Game)]);
    say(&mut game, C, &[]);
    say(&mut game, D, &[]);
    say(&mut game, A, &[kontra(Target::Valat), kontra(Target::Game)]);
    assert_eq!(levels(&game), [4, 0, 0, 0, 0, 4]);
    for seat in [B, C, D] {
        assert_eq!(game.to_act(), Some(seat));
        for target in [Target::Valat, Target::Game] {
            assert!(!may(&game, kontra(target)), "{seat:?} on {target:?}");
            let before = game.clone();
            assert_eq!(
                game.apply(kontra(target)),
                Err(GameError::NotLegal(kontra(target)))
            );
            assert_eq!(game, before);
        }
        say(&mut game, seat, &[]);
    }
    assert_eq!(game.phase(), Phase::Play);
    assert_eq!(levels(&game), [4, 0, 0, 0, 0, 4]);
}

#[test]
fn c14_the_game_ladder_and_a_partner_shown_by_re_or_mort() {
    let on_game = kontra(Target::Game);
    // Tri, srce called, C holds the king: B and D are the opponents.
    let game = round_game(C, Tri);
    assert_eq!(game.full().partner(), Some(C));
    assert_eq!(may_by_seat(&game, on_game), [false, true, false, true]);
    // The declarer called its own king: B, C and D are opponents.
    assert_eq!(
        may_by_seat(&round_game(A, Tri), on_game),
        [false, true, true, true]
    );

    // B says kontra. Re is for the declarer and for the partner.
    let mut game = round_game(C, Tri);
    say(&mut game, A, &[]);
    assert!(may(&game, on_game));
    raise(&mut game, B, Target::Game);
    assert!(!may(&game, on_game), "B cannot say re");
    act(&mut game, B, Action::Done);
    assert_eq!(game.public().partner(), PublicPartner::NotKnown);
    let at_c = game.clone();

    // C, the partner, says re: C is public from that action on.
    assert!(may(&game, on_game));
    raise(&mut game, C, Target::Game);
    assert_eq!(game.public().kontra_level(Target::Game), 2);
    assert_eq!(game.public().partner(), PublicPartner::With(C));
    act(&mut game, C, Action::Done);
    // Sub by an opponent, here D.
    assert!(may(&game, on_game));
    raise(&mut game, D, Target::Game);
    assert_eq!(game.public().partner(), PublicPartner::With(C));

    // The other way: C passes, D cannot say re, the declarer says re. No
    // partner is shown.
    let mut game = at_c;
    say(&mut game, C, &[]);
    assert!(!may(&game, on_game), "D cannot say re");
    say(&mut game, D, &[]);
    assert!(may(&game, on_game));
    raise(&mut game, A, Target::Game);
    assert_eq!(game.public().partner(), PublicPartner::NotKnown);
    act(&mut game, A, Action::Done);
    // Sub by B. An opponent's kontra shows no partner either.
    raise(&mut game, B, Target::Game);
    assert_eq!(game.public().partner(), PublicPartner::NotKnown);
    act(&mut game, B, Action::Done);
    // Mort by C: C is public from that action on.
    assert!(may(&game, on_game));
    raise(&mut game, C, Target::Game);
    assert_eq!(game.public().kontra_level(Target::Game), 4);
    assert_eq!(game.public().partner(), PublicPartner::With(C));
    act(&mut game, C, Action::Done);
    assert!(!may(&game, on_game));

    // The same on a bonus the declarer announced (D148, case ii): re and
    // mort belong to the declarer's side. A announces trula, B says
    // kontra, which shows nothing; C says re: C is the partner.
    let on_trula = kontra(Target::Trula);
    let mut game = round_game(C, Tri);
    say(&mut game, A, &[announce(Bonus::Trula)]);
    raise(&mut game, B, Target::Trula);
    assert_eq!(game.public().partner(), PublicPartner::NotKnown);
    act(&mut game, B, Action::Done);
    let at_c = game.clone();
    assert!(may(&game, on_trula));
    raise(&mut game, C, Target::Trula);
    assert_eq!(game.public().kontra_level(Target::Trula), 2);
    assert_eq!(game.public().partner(), PublicPartner::With(C));

    // The declarer's own re shows nothing, and neither does an opponent's
    // sub. C's mort shows C.
    let mut game = at_c;
    say(&mut game, C, &[]);
    assert!(!may(&game, on_trula), "D cannot say re");
    say(&mut game, D, &[]);
    raise(&mut game, A, Target::Trula);
    assert_eq!(game.public().partner(), PublicPartner::NotKnown);
    act(&mut game, A, Action::Done);
    raise(&mut game, B, Target::Trula);
    assert_eq!(game.public().partner(), PublicPartner::NotKnown);
    act(&mut game, B, Action::Done);
    assert!(may(&game, on_trula));
    raise(&mut game, C, Target::Trula);
    assert_eq!(game.public().kontra_level(Target::Trula), 4);
    assert_eq!(game.public().partner(), PublicPartner::With(C));
}

#[test]
fn c15_a_bonus_is_raised_only_by_a_seat_that_knows_the_announcers_side() {
    let on_trula = kontra(Target::Trula);
    let on_kralji = kontra(Target::Kralji);

    // 1. The announcer is the declarer: every seat knows its side. The
    // opponents B and D may say kontra; C, the hidden partner, is on the
    // owning side and may not.
    let mut game = round_game(C, Tri);
    say(&mut game, A, &[announce(Bonus::Trula)]);
    assert!(may(&game, on_trula), "B");
    say(&mut game, B, &[]);
    assert!(!may(&game, on_trula), "C is on the declarer's side");
    say(&mut game, C, &[]);
    assert!(may(&game, on_trula), "D");

    // 2. The announcer is B, an opponent, and the partner is hidden.
    let mut game = round_game(C, Tri);
    say(&mut game, A, &[]);
    say(&mut game, B, &[announce(Bonus::Kralji)]);
    let after_b = game.clone();
    // C holds the called king: it knows that B is an opponent.
    assert!(may(&game, on_kralji), "C knows the sides");
    say(&mut game, C, &[]);
    // D is on B's side; kontra is not for it anyway. D says kontra on the
    // game, which keeps the round going.
    assert!(!may(&game, on_kralji));
    say(&mut game, D, &[kontra(Target::Game)]);
    // A is on the other side but does not know B's side: no kontra.
    assert!(!may(&game, on_kralji), "A does not know B's side");
    say(&mut game, A, &[]);
    assert!(!may(&game, on_kralji), "B cannot kontra its own bonus");
    say(&mut game, B, &[]);
    // C says kontra. While the partner is hidden, a seat that raises a
    // bonus of another seat that is not the declarer must hold the called
    // king. C is not the declarer: it is the partner, and all know it
    // from this action on (D148, case i).
    assert_eq!(game.public().partner(), PublicPartner::NotKnown);
    raise(&mut game, C, Target::Kralji);
    assert_eq!(game.public().partner(), PublicPartner::With(C));
    act(&mut game, C, Action::Done);
    // Re is for B's side. D is on it, and knows it now.
    assert!(may(&game, on_kralji), "D knows B's side now");
    say(&mut game, D, &[]);
    assert!(!may(&game, on_kralji), "A is not on the owning side");
    say(&mut game, A, &[]);
    // The announcer knows its own side: B may say re on its own bonus.
    assert!(may(&game, on_kralji), "the announcer itself");
    raise(&mut game, B, Target::Kralji);
    act(&mut game, B, Action::Done);
    // Sub by C, who knows.
    assert!(may(&game, on_kralji));

    // 3. The partner is public: every seat knows every side. C announces
    // kralj ultimo right after B's announcement.
    let mut game = after_b;
    say(&mut game, C, &[announce(Bonus::KraljUltimo)]);
    assert_eq!(game.public().partner(), PublicPartner::With(C));
    assert!(!may(&game, on_kralji), "D is on B's side: no kontra");
    say(&mut game, D, &[]);
    assert!(may(&game, on_kralji), "A knows B's side now");
    raise(&mut game, A, Target::Kralji);
    act(&mut game, A, Action::Done);
    say(&mut game, B, &[]);
    say(&mut game, C, &[]);
    assert!(may(&game, on_kralji), "D knows it is on B's side: re");
    raise(&mut game, D, Target::Kralji);

    // 4. The declarer holds the called king: it knows it plays alone, so
    // every other seat is an opponent for it.
    let mut game = round_game(A, Tri);
    assert_eq!(game.full().partner(), None);
    say(&mut game, A, &[]);
    say(&mut game, B, &[announce(Bonus::Kralji)]);
    // C and D are B's fellow opponents: no kontra.
    assert!(!may(&game, on_kralji));
    say(&mut game, C, &[]);
    assert!(!may(&game, on_kralji));
    say(&mut game, D, &[]);
    assert!(may(&game, on_kralji), "the declarer knows it is alone");
    // The declarer raises a bonus of another seat while the partner is
    // hidden: it must hold the called king, so it plays alone and all
    // know it from this action on (D148, case iii).
    assert_eq!(game.public().partner(), PublicPartner::NotKnown);
    raise(&mut game, A, Target::Kralji);
    assert_eq!(game.public().partner(), PublicPartner::Alone);
    act(&mut game, A, Action::Done);
    say(&mut game, B, &[]);
    // C is on B's side, and knows it now: re.
    assert!(may(&game, on_kralji), "C knows B's side now");

    // 5. The same deal, but the declarer announces kralj ultimo first. It
    // is then publicly alone (D147): every seat knows every side.
    let mut game = round_game(A, Tri);
    say(&mut game, A, &[announce(Bonus::KraljUltimo)]);
    assert_eq!(game.public().partner(), PublicPartner::Alone);
    say(&mut game, B, &[announce(Bonus::Kralji)]);
    assert!(!may(&game, on_kralji), "C is on B's side: no kontra");
    say(&mut game, C, &[]);
    assert!(!may(&game, on_kralji));
    say(&mut game, D, &[]);
    raise(&mut game, A, Target::Kralji);
    act(&mut game, A, Action::Done);
    say(&mut game, B, &[]);
    // C knows now that it is on B's side: re.
    assert!(may(&game, on_kralji), "C knows B's side");
    raise(&mut game, C, Target::Kralji);
    act(&mut game, C, Action::Done);
    // And D may not say sub: it is on the owning side too.
    assert!(!may(&game, on_kralji));
    // The opponents may kontra the declarer's kralj ultimo.
    assert!(may(&game, kontra(Target::KraljUltimo)));
}

/// The swap of two cards between two seats.
fn swapped(deal: &Deal, one: (Seat, Card), two: (Seat, Card)) -> Deal {
    let mut hands = deal.hands();
    assert!(hands[one.0.index()].remove(one.1));
    assert!(hands[two.0.index()].remove(two.1));
    hands[one.0.index()].insert(two.1);
    hands[two.0.index()].insert(one.1);
    Deal::new(hands, deal.talon()).expect("a whole deal")
}

/// What `seat` may do in the round, worked out from what it sees: the
/// public view and its own view. Written from criteria 2, 6-9 and 12-15.
fn legal_from_what_the_seat_sees(game: &Game, seat: Seat) -> ActionSet {
    let public = game.public();
    let hand = game.private(seat).hand();
    let declarer = public.declarer().expect("a declarer");
    let holds_king = public.called_king().is_some_and(|king| hand.contains(king));
    // The declarer's side as far as the seat knows it. Its own side: it is
    // the declarer, or it holds the called king.
    let own_side = seat == declarer || holds_king;
    let side_of = |other: Seat| -> Option<bool> {
        if other == declarer {
            return Some(true);
        }
        if other == seat {
            return Some(own_side);
        }
        match public.partner() {
            PublicPartner::With(partner) => Some(other == partner),
            PublicPartner::Alone => Some(false),
            // A seat that holds the called king is the partner, or the
            // declarer alone: every other seat but the declarer is an
            // opponent.
            PublicPartner::NotKnown if holds_king => Some(false),
            PublicPartner::NotKnown => None,
        }
    };

    let mut legal = ActionSet::EMPTY;
    legal.insert(Action::Done);
    for bonus in Bonus::ALL {
        let holder = match bonus {
            Bonus::Trula | Bonus::Kralji | Bonus::Valat => true,
            Bonus::KraljUltimo => holds_king,
            Bonus::PagatUltimo => hand.contains(Card::PAGAT),
        };
        if holder && public.announced(bonus).is_none() {
            legal.insert(Action::Announce(bonus));
        }
    }
    for target in Target::ALL {
        let owner = match target.bonus() {
            None => Some(true),
            Some(bonus) => match public.announced(bonus) {
                Some(announcer) => side_of(announcer),
                None => continue,
            },
        };
        let Some(owner) = owner else { continue };
        let level = public.kontra_level(target);
        // Kontra (1) and sub (3) by the other side, re (2) and mort (4) by
        // the owning side.
        let by_owner = level % 2 == 1;
        if level < 4 && by_owner == (own_side == owner) {
            legal.insert(Action::Kontra(target));
        }
    }
    legal
}

#[test]
fn c16_legal_actions_depend_only_on_what_the_seat_knows() {
    // Two deals. In the first C holds the called srce kralj and B the srce
    // kaval; in the second they have changed the two cards. So once C is
    // the partner and B an opponent, once the other way round. The hands
    // of A and D are the same, and so is everything that is said.
    let first = round_deal(C);
    let king = Card::court(Suit::Srce, tarok_engine::Rank::Kralj);
    let kaval = Card::court(Suit::Srce, tarok_engine::Rank::Kaval);
    let second = swapped(&first, (C, king), (B, kaval));
    assert_eq!(second, round_deal(B));

    let play = |deal: &Deal| {
        let mut game = common::flow::game(deal);
        common::flow::declare(&mut game, A, Tri);
        act(&mut game, A, Action::CallKing(Suit::Srce));
        common::flow::exchange_back_open(&mut game, 0);
        game
    };
    let mut games = [play(&first), play(&second)];
    assert_eq!(games[0].full().partner(), Some(C));
    assert_eq!(games[1].full().partner(), Some(B));
    for seat in [A, D] {
        assert_eq!(games[0].private(seat).hand(), games[1].private(seat).hand());
    }

    // The round. Only things that B and C may say in both deals.
    let turns: [(Seat, &[Action]); 11] = [
        (A, &[announce(Bonus::Trula)]),
        (B, &[announce(Bonus::Kralji)]),
        (C, &[announce(Bonus::Valat)]),
        (D, &[kontra(Target::Game), kontra(Target::Trula)]),
        (A, &[kontra(Target::Game)]),
        (B, &[]),
        (C, &[]),
        (D, &[kontra(Target::Game)]),
        (A, &[kontra(Target::Trula)]),
        (B, &[]),
        (C, &[]),
    ];
    let mut compared = 0;
    for (seat, actions) in turns {
        for step in 0..=actions.len() {
            for game in &games {
                assert_eq!(game.phase(), Phase::Announcements);
                assert_eq!(game.to_act(), Some(seat));
            }
            if seat == A || seat == D {
                // The same hand and the same public history: the same
                // legal set, whoever the partner is.
                assert_eq!(
                    games[0].legal_actions(),
                    games[1].legal_actions(),
                    "{seat:?}, action {step} of its turn"
                );
                compared += 1;
                // D never may raise B's kralji or C's valat: it does not
                // know their sides. In the first deal it would be the
                // owner's side for the kralji, in the second for the valat.
                if seat == D {
                    assert!(!may(&games[0], kontra(Target::Kralji)));
                    assert!(!may(&games[0], kontra(Target::Valat)));
                }
            }
            let action = actions.get(step).copied().unwrap_or(Action::Done);
            for game in &mut games {
                game.apply(action).expect("legal in both deals");
            }
        }
    }
    assert!(compared >= 9);
    // What all see is the same in both deals, to the end of the round.
    assert_eq!(
        format!("{:?}", games[0].public()),
        format!("{:?}", games[1].public())
    );

    // The same from the other side, on random deals of flow B: at every
    // decision of the round the legal set is what follows from the public
    // view and the seat's own view.
    let mut decisions = 0u32;
    let mut with_a_kontra = 0u32;
    for seed in 0..400u64 {
        seeded_game_b_with(seed, |game| {
            if game.phase() != Phase::Announcements {
                return;
            }
            let seat = game.to_act().expect("a seat speaks");
            let expected = legal_from_what_the_seat_sees(game, seat);
            assert_eq!(game.legal_actions(), expected, "seed {seed}, {seat:?}");
            decisions += 1;
            let kontras = Target::ALL
                .into_iter()
                .filter(|&target| expected.contains(Action::Kontra(target)))
                .count();
            with_a_kontra += u32::from(kontras > 0);
        });
    }
    assert!(decisions > 4_000, "{decisions} decisions in the round");
    assert!(with_a_kontra > 1_000, "{with_a_kontra} with a legal kontra");
}

/// Review point 1, with D147 and D148: what is said in the round and shows
/// no partner. Written out by hand, in two worlds that differ in who the
/// partner is. A rule that shows more than `c10`, `c14` and `c15` say
/// fails here: the public partner would no longer be "not known".
#[test]
fn c14_what_is_said_and_shows_no_partner() {
    let start = |deal: &Deal| {
        let mut game = common::flow::game(deal);
        common::flow::declare(&mut game, A, Tri);
        act(&mut game, A, Action::CallKing(Suit::Srce));
        common::flow::exchange_back_open(&mut game, 0);
        game
    };
    // C holds the called king in the first world, B in the second. D is
    // an opponent in both.
    let mut views = Vec::new();
    for (deal, partner) in [(round_deal(C), C), (round_deal(B), B)] {
        let mut game = start(&deal);
        assert_eq!(game.full().partner(), Some(partner));
        // One action, and the partner is still hidden after it.
        let hidden = |game: &mut Game, seat: Seat, action: Action| {
            act(game, seat, action);
            assert_eq!(
                game.public().partner(),
                PublicPartner::NotKnown,
                "after {action:?} by {seat:?}, the partner being {partner:?}"
            );
        };
        // Every announcement but kralj ultimo: by the declarer...
        hidden(&mut game, A, announce(Bonus::Trula));
        hidden(&mut game, A, Action::Done);
        // ...by B and by C, of whom one is the partner and one an opponent.
        hidden(&mut game, B, announce(Bonus::Kralji));
        hidden(&mut game, B, announce(Bonus::PagatUltimo));
        hidden(&mut game, B, Action::Done);
        hidden(&mut game, C, announce(Bonus::Valat));
        hidden(&mut game, C, Action::Done);
        // Neither the declarer nor the opponent D knows the side of B or
        // of C: they may not raise the kralji, the pagat ultimo or the
        // valat. Such a raise would show the sides (D148).
        let not_the_bonuses_of_b_and_c = |game: &Game| {
            for target in [Target::Kralji, Target::PagatUltimo, Target::Valat] {
                assert!(!may(game, kontra(target)), "{target:?}");
            }
        };
        // Kontra by an opponent, on the game and on a bonus of the
        // declarer.
        not_the_bonuses_of_b_and_c(&game);
        hidden(&mut game, D, kontra(Target::Game));
        hidden(&mut game, D, kontra(Target::Trula));
        hidden(&mut game, D, Action::Done);
        // Re by the declarer itself, on both.
        not_the_bonuses_of_b_and_c(&game);
        hidden(&mut game, A, kontra(Target::Game));
        hidden(&mut game, A, kontra(Target::Trula));
        hidden(&mut game, A, Action::Done);
        // Passes.
        hidden(&mut game, B, Action::Done);
        hidden(&mut game, C, Action::Done);
        // Sub by the opponent, mort by the declarer.
        not_the_bonuses_of_b_and_c(&game);
        hidden(&mut game, D, kontra(Target::Game));
        hidden(&mut game, D, kontra(Target::Trula));
        hidden(&mut game, D, Action::Done);
        not_the_bonuses_of_b_and_c(&game);
        hidden(&mut game, A, kontra(Target::Game));
        hidden(&mut game, A, kontra(Target::Trula));
        hidden(&mut game, A, Action::Done);
        // The round ends on three passes.
        hidden(&mut game, B, Action::Done);
        hidden(&mut game, C, Action::Done);
        hidden(&mut game, D, Action::Done);
        assert_eq!(game.phase(), Phase::Play);
        assert_eq!(levels(&game), [4, 4, 0, 0, 0, 0]);
        views.push(format!("{:?}", game.public()));
    }
    // What all see is the same in both worlds.
    assert_eq!(views[0], views[1]);

    // The other side of it: what does show the sides. The same things are
    // said in both worlds up to C's second turn: A announces trula, B
    // announces kralji, D says kontra on the game and on the trula. Then
    // C takes one action: kralj ultimo; re on the game; re on the
    // declarer's trula; kontra on B's kralji. Each is legal only for the
    // holder of the called king, so it shows C as the partner. In the
    // world in which B is the partner, C may take none of them.
    let up_to_c = |deal: &Deal| {
        let mut game = start(deal);
        say(&mut game, A, &[announce(Bonus::Trula)]);
        say(&mut game, B, &[announce(Bonus::Kralji)]);
        say(&mut game, C, &[]);
        say(&mut game, D, &[kontra(Target::Game), kontra(Target::Trula)]);
        say(&mut game, A, &[]);
        say(&mut game, B, &[]);
        assert_eq!(game.to_act(), Some(C));
        assert_eq!(game.public().partner(), PublicPartner::NotKnown);
        game
    };
    for action in [
        announce(Bonus::KraljUltimo),
        kontra(Target::Game),
        kontra(Target::Trula),
        kontra(Target::Kralji),
    ] {
        let mut game = up_to_c(&round_deal(C));
        act(&mut game, C, action);
        assert_eq!(
            game.public().partner(),
            PublicPartner::With(C),
            "after {action:?} by C"
        );
        let other = up_to_c(&round_deal(B));
        assert!(!may(&other, action), "{action:?} by an opponent");
    }
}
