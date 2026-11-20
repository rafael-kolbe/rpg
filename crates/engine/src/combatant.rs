//! A combatant's state during a battle.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::action::{Affinity, Element};
use crate::ai::{AiProfile, Intent};
use crate::ids::{ActionId, CombatantId, ItemId, Side};
use crate::setup::CombatantSetup;
use crate::stats::Stats;
use crate::status::{ActiveStatus, StatusKind, StatusTimer};
use utils::numbers::saturating_u16;

/// Where a combatant is in its turn cycle.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TurnState {
    /// The ATB gauge is filling (or stopped by Sleep or a knockout).
    Charging,
    /// The gauge is full and the player has not chosen a command yet.
    AwaitingCommand,
    /// A command was chosen and waits in the action queue.
    Queued,
    /// Left the battle (a thief running off with its loot): out of reach,
    /// no more turns, and no reward.
    Fled,
}

/// A combatant in a battle. Read-only outside the engine.
#[derive(Debug, Clone)]
pub struct Combatant {
    pub(crate) id: CombatantId,
    pub(crate) side: Side,
    pub(crate) name: String,
    pub(crate) stats: Stats,
    pub(crate) attack: ActionId,
    pub(crate) actions: Vec<ActionId>,
    pub(crate) affinities: BTreeMap<Element, Affinity>,
    pub(crate) status_immunities: Vec<StatusKind>,
    pub(crate) ai: Option<AiProfile>,
    pub(crate) player_controlled: bool,
    /// Its fall ends the battle for its side ([`CombatantSetup::leader`]).
    pub(crate) leader: bool,

    pub(crate) hp: u16,
    pub(crate) mp: u16,
    /// ATB gauge, `0..=ATB_FULL`.
    pub(crate) atb: u32,
    pub(crate) state: TurnState,
    pub(crate) statuses: Vec<ActiveStatus>,
    pub(crate) defending: bool,
    /// Engine-controlled only: the command chosen for its next turn.
    pub(crate) intent: Option<Intent>,
    /// Items it stole: it flees with them on its next turn, or drops them
    /// when knocked out first.
    pub(crate) stolen: Vec<ItemId>,
}

impl Combatant {
    pub(crate) fn new(
        id: CombatantId,
        side: Side,
        setup: CombatantSetup,
        player_controlled: bool,
    ) -> Self {
        Self {
            id,
            side,
            hp: setup.hp.unwrap_or(setup.stats.max_hp),
            mp: setup.mp.unwrap_or(setup.stats.max_mp),
            name: setup.name,
            stats: setup.stats,
            attack: setup.attack,
            actions: setup.actions,
            affinities: setup.affinities,
            status_immunities: setup.status_immunities,
            ai: setup.ai,
            leader: setup.leader,
            player_controlled,
            atb: 0,
            state: TurnState::Charging,
            statuses: Vec::new(),
            defending: false,
            intent: None,
            stolen: Vec::new(),
        }
    }

    /// Position in the battle.
    #[must_use]
    pub fn id(&self) -> CombatantId {
        self.id
    }

    /// Side it fights on.
    #[must_use]
    pub fn side(&self) -> Side {
        self.side
    }

    /// Fluent message id of its display name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Level, max HP/MP, attributes and equipment.
    #[must_use]
    pub fn stats(&self) -> &Stats {
        &self.stats
    }

    /// The action its Attack command performs.
    #[must_use]
    pub fn attack_action(&self) -> &ActionId {
        &self.attack
    }

    /// Spells and abilities it knows.
    #[must_use]
    pub fn known_actions(&self) -> &[ActionId] {
        &self.actions
    }

    /// Current HP.
    #[must_use]
    pub fn hp(&self) -> u16 {
        self.hp
    }

    /// Current MP.
    #[must_use]
    pub fn mp(&self) -> u16 {
        self.mp
    }

    /// ATB gauge, `0..=`[`ATB_FULL`](crate::ATB_FULL).
    #[must_use]
    pub fn atb(&self) -> u32 {
        self.atb
    }

    /// Where it is in its turn cycle.
    #[must_use]
    pub fn state(&self) -> TurnState {
        self.state
    }

    /// Whether Defend is halving the damage it takes.
    #[must_use]
    pub fn is_defending(&self) -> bool {
        self.defending
    }

    /// Statuses currently on it, with the time left on timed ones.
    pub fn statuses(&self) -> impl Iterator<Item = (StatusKind, Option<StatusTimer>)> + '_ {
        self.statuses
            .iter()
            .map(|status| (status.kind, status.timer))
    }

    /// Whether it has the given status.
    #[must_use]
    pub fn has_status(&self, kind: StatusKind) -> bool {
        self.statuses.iter().any(|status| status.kind == kind)
    }

    /// Whether it is knocked out (0 HP).
    #[must_use]
    pub fn is_knocked_out(&self) -> bool {
        self.hp == 0
    }

    /// Whether it fled the battle.
    #[must_use]
    pub fn has_fled(&self) -> bool {
        self.state == TurnState::Fled
    }

    /// Whether it is no longer in the fight: knocked out or fled.
    #[must_use]
    pub fn is_out(&self) -> bool {
        self.hp == 0 || self.has_fled()
    }

    /// Whether the player chooses its commands.
    #[must_use]
    pub fn is_player_controlled(&self) -> bool {
        self.player_controlled
    }

    /// Current HP as a percentage of max HP, rounded up so that a living
    /// combatant is never at 0%.
    pub(crate) fn hp_percent(&self) -> u16 {
        let percent = u32::from(self.hp) * 100;
        saturating_u16(u64::from(percent.div_ceil(u32::from(self.stats.max_hp))))
    }

    /// Back to an empty gauge after acting, defending or falling asleep.
    pub(crate) fn reset_turn(&mut self) {
        self.atb = 0;
        self.state = TurnState::Charging;
    }

    pub(crate) fn remove_status(&mut self, kind: StatusKind) -> bool {
        let before = self.statuses.len();
        self.statuses.retain(|status| status.kind != kind);
        self.statuses.len() != before
    }
}
