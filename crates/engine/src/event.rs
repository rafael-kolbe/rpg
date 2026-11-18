//! Everything that happens in a battle, as reported to the outside.

use serde::Serialize;

use crate::action::Affinity;
use crate::battle::Outcome;
use crate::ids::{ActionId, CombatantId, ItemId};
use crate::status::StatusKind;

/// One change in a battle. The client animates these; it never recomputes
/// rules from them.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum BattleEvent {
    /// A gauge filled: the combatant can act.
    TurnReady {
        /// Who.
        combatant: CombatantId,
    },
    /// A command was chosen and joined the action queue.
    ActionQueued {
        /// Who.
        actor: CombatantId,
        /// The action that will run.
        action: ActionId,
        /// The item being used, already taken from the inventory.
        item: Option<ItemId>,
    },
    /// The combatant chose Defend: damage taken is halved until its next
    /// action, and its gauge starts over.
    Defending {
        /// Who.
        combatant: CombatantId,
    },
    /// A queued action began executing; the queue is busy for the action's
    /// `durationTicks`.
    ActionStarted {
        /// Who.
        actor: CombatantId,
        /// The action.
        action: ActionId,
        /// The item used, if any.
        item: Option<ItemId>,
        /// Final targets, after retargeting.
        targets: Vec<CombatantId>,
        /// MP spent.
        mp_spent: u16,
    },
    /// A queued action was dropped before executing. A reserved item went
    /// back to the inventory.
    ActionCancelled {
        /// Who.
        actor: CombatantId,
        /// The action.
        action: ActionId,
        /// The item returned to the inventory, if any.
        item: Option<ItemId>,
        /// Why.
        reason: CancelReason,
    },
    /// A queued action reached the front of the queue but could not run; the
    /// turn is spent. A reserved item went back to the inventory.
    ActionFailed {
        /// Who.
        actor: CombatantId,
        /// The action.
        action: ActionId,
        /// The item returned to the inventory, if any.
        item: Option<ItemId>,
        /// Why.
        reason: FailReason,
    },
    /// Damage from an action.
    Damaged {
        /// Who took it.
        target: CombatantId,
        /// HP lost (0 when an elemental immunity cancels it, shown as "0").
        amount: u16,
        /// Whether it was a critical hit.
        critical: bool,
        /// Whether the 150% Distracted multiplier applied.
        distracted: bool,
        /// Elemental affinity that changed the damage, if any.
        affinity: Option<Affinity>,
    },
    /// HP restored by an action.
    Healed {
        /// Who.
        target: CombatantId,
        /// HP actually restored (capped at max HP).
        amount: u16,
    },
    /// MP restored by an action.
    MpRestored {
        /// Who.
        target: CombatantId,
        /// MP actually restored (capped at max MP).
        amount: u16,
    },
    /// A knocked-out combatant came back.
    Revived {
        /// Who.
        target: CombatantId,
        /// HP after revival.
        hp: u16,
    },
    /// Poison pulse.
    PoisonDamaged {
        /// Who.
        target: CombatantId,
        /// HP lost.
        amount: u16,
    },
    /// Regen pulse.
    Regenerated {
        /// Who.
        target: CombatantId,
        /// HP restored.
        amount: u16,
    },
    /// A status took hold (or was refreshed).
    StatusApplied {
        /// Who.
        target: CombatantId,
        /// The status.
        status: StatusKind,
    },
    /// A physical attack did not land (shown as "Miss").
    Missed {
        /// Who was aimed at.
        target: CombatantId,
    },
    /// A hostile status failed against the target's Luck (shown as "Resist").
    StatusResisted {
        /// Who was aimed at.
        target: CombatantId,
        /// The status.
        status: StatusKind,
    },
    /// A status did not take hold: the target is immune (shown as "Immune").
    StatusImmune {
        /// Who.
        target: CombatantId,
        /// The status.
        status: StatusKind,
    },
    /// A status ended.
    StatusRemoved {
        /// Who.
        target: CombatantId,
        /// The status.
        status: StatusKind,
        /// Why it ended.
        reason: StatusEndReason,
    },
    /// The Limit gauge filled: the next Attack is the Limit.
    LimitReady {
        /// Who.
        combatant: CombatantId,
    },
    /// HP reached 0. Statuses are cleared and queued actions cancelled.
    KnockedOut {
        /// Who.
        combatant: CombatantId,
    },
    /// One side is out of combatants.
    BattleEnded {
        /// Who won.
        outcome: Outcome,
    },
}

/// Why a queued action was dropped.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CancelReason {
    /// The actor was knocked out.
    ActorKnockedOut,
    /// The actor fell asleep.
    ActorAsleep,
}

/// Why a queued action could not run.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FailReason {
    /// MP dropped below the cost while the action waited.
    NotEnoughMp,
    /// The actor was silenced while a spell waited.
    Silenced,
    /// No valid target was left on the targeted side.
    NoValidTarget,
}

/// Why a status ended.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusEndReason {
    /// Its duration ran out.
    Expired,
    /// An item or spell removed it.
    Cured,
    /// Replaced by its opposite (Haste ↔ Slow).
    Replaced,
    /// A hit ended it (Distracted, Sleep).
    Hit,
    /// The combatant's own gauge filled (Distracted).
    GaugeFull,
}
