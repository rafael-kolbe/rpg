//! Battle rules of the game: the ATB loop, the action queue, damage, statuses
//! and enemy AI.
//!
//! The engine is deterministic and does no I/O: a [`Battle`] is built from a
//! [`BattleSetup`] and a seed, advanced one tick (100 ms) at a time with
//! [`Battle::step`], and driven by player [`Command`]s. Every change is reported
//! as a [`BattleEvent`]. Given the same setup, seed and commands on the same
//! ticks, the same events come out, on any machine.
//!
//! All arithmetic is integer. Percentages are basis points (10 000 = 100%).
//!
//! # Example
//!
//! ```
//! use std::collections::BTreeMap;
//!
//! use engine::{
//!     ActionCategory, ActionDef, ActionId, AiAct, AiPhase, AiProfile, AiRule, Attributes, Battle,
//!     BattleSetup, CombatantSetup, Condition, Effect, Encounter, Equipment, Opening, PartyControl,
//!     Stats, TargetPick, TargetRule,
//! };
//!
//! let attack = ActionId::new("attack");
//! let fighter = |name: &str| CombatantSetup {
//!     name: name.to_owned(),
//!     stats: Stats {
//!         level: 10,
//!         max_hp: 300,
//!         max_mp: 0,
//!         attributes: Attributes {
//!             strength: 20,
//!             dexterity: 50,
//!             vitality: 15,
//!             magic: 10,
//!             spirit: 10,
//!             luck: 10,
//!         },
//!         equipment: Equipment::default(),
//!     },
//!     attack: attack.clone(),
//!     actions: Vec::new(),
//!     affinities: BTreeMap::new(),
//!     status_immunities: Vec::new(),
//!     // Enemies are always engine-controlled, so they need an AI.
//!     ai: Some(AiProfile {
//!         phases: vec![AiPhase {
//!             from_hp_percent: 100,
//!             rules: vec![AiRule {
//!                 when: Condition::Always,
//!                 weight: 1,
//!                 act: AiAct::Attack,
//!                 target: TargetPick::RandomOpponent,
//!             }],
//!         }],
//!     }),
//!     hp: None,
//!     mp: None,
//!     guest: false,
//!     leader: false,
//! };
//!
//! let setup = BattleSetup {
//!     encounter: Encounter {
//!         opening: Opening::Normal,
//!         inventory: BTreeMap::new(),
//!         party: vec![fighter("hero")],
//!         enemies: vec![fighter("wolf")],
//!         forms: BTreeMap::new(),
//!     },
//!     actions: BTreeMap::from([(
//!         attack,
//!         ActionDef {
//!             category: ActionCategory::Attack,
//!             target: TargetRule::SingleEnemy,
//!             mp_cost: 0,
//!             duration_ticks: 8,
//!             effect: Effect::PhysicalDamage { power: 16, element: None },
//!         },
//!     )]),
//!     items: BTreeMap::new(),
//! };
//!
//! let mut battle = Battle::new(setup, 42, PartyControl::Player).unwrap();
//! let events = battle.step();
//! assert_eq!(battle.tick(), 1);
//! # let _ = events;
//! ```

mod action;
mod ai;
mod atb;
mod battle;
mod combatant;
mod damage;
#[cfg(feature = "sandbox")]
mod debug;
mod effects;
mod event;
mod ids;
mod record;
mod rng;
mod setup;
mod stats;
mod status;

#[cfg(test)]
mod test_support;

pub use action::{ActionCategory, ActionDef, Affinity, Effect, Element, ItemDef, TargetRule};
pub use ai::{
    AiAct, AiPhase, AiProfile, AiRule, AnnouncedIntent, Condition, IntentCommand, IntentTarget,
    TargetPick,
};
pub use atb::{ATB_FULL, TICK_MS};
pub use battle::{Battle, Command, CommandError, CommandKind, Outcome, PartyControl};
pub use combatant::{Combatant, TurnState};
pub use event::{BattleEvent, CancelReason, FailReason, StatusEndReason};
pub use ids::{ActionId, CombatantId, ItemId, Side};
pub use record::{BattleRecord, RecordedCommand, ReplayError};
pub use rng::Rng;
pub use setup::{BattleSetup, CombatantSetup, Encounter, Opening, SetupError};
pub use stats::{Attributes, Equipment, Stats};
pub use status::{StatusKind, StatusTimer};
