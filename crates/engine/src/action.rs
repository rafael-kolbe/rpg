//! What an action does: its category, targets, cost and effect. Items are
//! actions too: an [`ItemDef`] points at the action it performs.

use serde::{Deserialize, Serialize};

use crate::ids::ActionId;
use crate::status::StatusKind;

/// Definition of an action, loaded from `content/actions.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActionDef {
    /// Which command performs it; decides what Silence blocks.
    pub category: ActionCategory,
    /// Who it can be aimed at.
    pub target: TargetRule,
    /// MP spent when it executes.
    #[serde(default)]
    pub mp_cost: u16,
    /// Ticks the action occupies the battle's action queue (the time the
    /// client has to animate it). Other ATB gauges keep filling meanwhile.
    pub duration_ticks: u16,
    /// What happens to each target.
    pub effect: Effect,
}

/// The command family an action belongs to.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionCategory {
    /// A combatant's basic attack (the Attack command).
    Attack,
    /// A spell (the Magic command). Blocked by Silence.
    Magic,
    /// A physical class technique (the Technic command). Not blocked by
    /// Silence.
    Technic,
    /// What an item does when used (the Item command).
    Item,
}

/// Who an action can be aimed at, relative to the user.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetRule {
    /// One combatant on the other side.
    SingleEnemy,
    /// Every combatant on the other side.
    AllEnemies,
    /// One combatant on the user's side (the user included).
    SingleAlly,
    /// Every combatant on the user's side.
    AllAllies,
    /// The user only.
    User,
}

impl TargetRule {
    /// Whether the command must name a target.
    #[must_use]
    pub fn needs_target(self) -> bool {
        matches!(self, Self::SingleEnemy | Self::SingleAlly)
    }

    /// Whether the targets are on the user's own side.
    #[must_use]
    pub fn on_own_side(self) -> bool {
        !matches!(self, Self::SingleEnemy | Self::AllEnemies)
    }
}

/// What an action does to each target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Effect {
    /// Physical damage (Attack against Defense); can be critical.
    PhysicalDamage {
        /// Strength of the action; a basic attack is 16.
        power: u16,
        /// Element, if any.
        #[serde(default)]
        element: Option<Element>,
    },
    /// Magical damage (Magic Attack against Magic Defense).
    MagicalDamage {
        /// Strength of the spell.
        power: u16,
        /// Element, if any.
        #[serde(default)]
        element: Option<Element>,
    },
    /// Damage equal to a share of the target's current HP (Gravity spells).
    PercentCurrentHp {
        /// Share of current HP, `1..=100`.
        percent: u8,
        /// Element, usually Gravity.
        #[serde(default)]
        element: Option<Element>,
    },
    /// Healing scaled by the user's Magic and level.
    Heal {
        /// Strength of the spell.
        power: u16,
    },
    /// Fixed HP restoration (potions).
    RestoreHp {
        /// HP restored.
        amount: u16,
    },
    /// Fixed MP restoration (ethers).
    RestoreMp {
        /// MP restored.
        amount: u16,
    },
    /// Brings a knocked-out ally back.
    Revive {
        /// HP after revival, as a share of max HP, `1..=100`.
        hp_percent: u8,
    },
    /// Inflicts or grants a status.
    ApplyStatus {
        /// The status.
        status: StatusKind,
        /// How long it lasts. Required for every status except Distracted,
        /// which ends on a hit or a full ATB gauge instead.
        #[serde(default)]
        duration_ticks: Option<u16>,
        /// Chance to take hold on an opponent, in percent (`1..=100`), before
        /// the target's Luck resistance. Ignored on allies: buffs always land.
        #[serde(default = "full_accuracy")]
        accuracy: u8,
    },
    /// Removes statuses (antidotes, Esuna).
    RemoveStatus {
        /// Statuses removed, when present.
        statuses: Vec<StatusKind>,
    },
    /// Consumes another ally: it is knocked out and the user recovers the HP
    /// it had (a Lich feeding on its summon).
    Devour,
    /// Raises a knocked-out ally as another creature, at full HP: the
    /// encounter's form `into` takes its place.
    Reanimate {
        /// Key of the form in [`crate::Encounter::forms`].
        into: String,
    },
}

fn full_accuracy() -> u8 {
    100
}

impl Effect {
    /// Whether this effect is aimed at knocked-out combatants.
    #[must_use]
    pub fn targets_knocked_out(&self) -> bool {
        matches!(self, Self::Revive { .. } | Self::Reanimate { .. })
    }

    /// The element of a damaging effect, if any.
    #[must_use]
    pub fn element(&self) -> Option<Element> {
        match self {
            Self::PhysicalDamage { element, .. }
            | Self::MagicalDamage { element, .. }
            | Self::PercentCurrentHp { element, .. } => *element,
            Self::Heal { .. }
            | Self::RestoreHp { .. }
            | Self::RestoreMp { .. }
            | Self::Revive { .. }
            | Self::ApplyStatus { .. }
            | Self::RemoveStatus { .. }
            | Self::Devour
            | Self::Reanimate { .. } => None,
        }
    }
}

/// Elemental property of an attack.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Element {
    /// Fire.
    Fire,
    /// Ice.
    Ice,
    /// Lightning.
    Lightning,
    /// Gravity: damage based on the target's current HP.
    Gravity,
}

/// How a combatant reacts to an element.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Affinity {
    /// Takes double damage.
    Weak,
    /// Takes half damage.
    Resist,
    /// Takes no damage.
    Immune,
}

/// Definition of an item, loaded from `content/items.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ItemDef {
    /// The action performed when the item is used; its category must be
    /// [`ActionCategory::Item`].
    pub action: ActionId,
}
