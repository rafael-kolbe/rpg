//! Fixtures shared by the engine's unit tests: a small action catalog and a
//! generic fighter. Gameplay content lives in `content/`, not here.

use std::collections::BTreeMap;

use crate::action::{ActionCategory, ActionDef, Effect, Element, ItemDef, TargetRule};
use crate::ai::{AiAct, AiPhase, AiProfile, AiRule, Condition, TargetPick};
use crate::battle::Battle;
use crate::event::BattleEvent;
use crate::ids::{ActionId, CombatantId, ItemId};
use crate::setup::{BattleSetup, CombatantSetup, Encounter, Opening};
use crate::stats::{Attributes, Equipment, Stats};
use crate::status::StatusKind;

pub(crate) fn stats(level: u8, dexterity: u16) -> Stats {
    Stats {
        level,
        max_hp: 300,
        max_mp: 50,
        attributes: Attributes {
            strength: 20,
            dexterity,
            vitality: 15,
            magic: 20,
            spirit: 15,
            luck: 10,
        },
        equipment: Equipment::default(),
    }
}

/// Level 10, 300 HP, knows Fire and Cure, has a Limit, attacks when AI-driven.
pub(crate) fn fighter(name: &str) -> CombatantSetup {
    CombatantSetup {
        name: name.to_owned(),
        stats: stats(10, 50),
        attack: ActionId::new("attack"),
        actions: vec![ActionId::new("fire"), ActionId::new("cure")],
        limit: Some(ActionId::new("twin-cleave")),
        affinities: BTreeMap::new(),
        status_immunities: Vec::new(),
        ai: Some(AiProfile {
            phases: vec![AiPhase {
                from_hp_percent: 100,
                rules: vec![AiRule {
                    when: Condition::Always,
                    weight: 1,
                    act: AiAct::Attack,
                    target: TargetPick::RandomOpponent,
                }],
            }],
        }),
    }
}

fn action(category: ActionCategory, target: TargetRule, mp_cost: u16, effect: Effect) -> ActionDef {
    ActionDef {
        category,
        target,
        mp_cost,
        duration_ticks: 8,
        effect,
    }
}

pub(crate) fn catalog_setup(
    party: Vec<CombatantSetup>,
    enemies: Vec<CombatantSetup>,
) -> BattleSetup {
    use ActionCategory as C;
    use TargetRule as T;

    let actions = BTreeMap::from([
        (
            ActionId::new("attack"),
            action(
                C::Attack,
                T::SingleEnemy,
                0,
                Effect::PhysicalDamage {
                    power: 16,
                    element: None,
                },
            ),
        ),
        (
            ActionId::new("fire"),
            action(
                C::Magic,
                T::SingleEnemy,
                4,
                Effect::MagicalDamage {
                    power: 8,
                    element: Some(Element::Fire),
                },
            ),
        ),
        (
            ActionId::new("cure"),
            action(C::Magic, T::SingleAlly, 5, Effect::Heal { power: 2 }),
        ),
        (
            ActionId::new("haste"),
            action(
                C::Magic,
                T::SingleAlly,
                5,
                Effect::ApplyStatus {
                    status: StatusKind::Haste,
                    duration_ticks: Some(300),
                },
            ),
        ),
        (
            ActionId::new("twin-cleave"),
            action(
                C::Limit,
                T::SingleEnemy,
                0,
                Effect::PhysicalDamage {
                    power: 48,
                    element: None,
                },
            ),
        ),
        (
            ActionId::new("potion"),
            action(C::Item, T::SingleAlly, 0, Effect::RestoreHp { amount: 100 }),
        ),
        (
            ActionId::new("revive-feather"),
            action(C::Item, T::SingleAlly, 0, Effect::Revive { hp_percent: 25 }),
        ),
    ]);
    let items = BTreeMap::from([
        (
            ItemId::new("potion"),
            ItemDef {
                action: ActionId::new("potion"),
            },
        ),
        (
            ItemId::new("revive-feather"),
            ItemDef {
                action: ActionId::new("revive-feather"),
            },
        ),
    ]);

    BattleSetup {
        encounter: Encounter {
            opening: Opening::Normal,
            inventory: BTreeMap::from([(ItemId::new("potion"), 2)]),
            party,
            enemies,
        },
        actions,
        items,
    }
}

/// Steps until `id` is waiting for a command; returns every event seen.
pub(crate) fn until_turn(battle: &mut Battle, id: CombatantId) -> Vec<BattleEvent> {
    run_until(battle, |event| {
        *event == BattleEvent::TurnReady { combatant: id }
    })
}

/// Steps until an event matches (at most 10 000 ticks); returns every event seen.
pub(crate) fn run_until(
    battle: &mut Battle,
    done: impl Fn(&BattleEvent) -> bool,
) -> Vec<BattleEvent> {
    let mut seen = Vec::new();
    for _ in 0..10_000 {
        let events = battle.step();
        let finished = events.iter().any(&done);
        seen.extend(events);
        if finished {
            return seen;
        }
    }
    panic!("no matching event within 10 000 ticks");
}
