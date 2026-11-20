//! The battle as the client sees it: everything the battle screen draws.
//!
//! Built from an [`engine::Battle`] by the server after every change. The
//! client renders it as-is; it never recomputes rules.

use std::collections::BTreeMap;

use engine::{
    ATB_FULL, ActionCategory, ActionId, AnnouncedIntent, Battle, CombatantId, Element, ItemId,
    Outcome, Side, StatusKind, StatusTimer, TargetRule, TurnState,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::messages::TimeMode;

/// Snapshot of a running battle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BattleState {
    /// Ticks elapsed (100 ms each).
    pub tick: u32,
    /// Set once the battle ended.
    pub outcome: Option<Outcome>,
    /// Every combatant, party first.
    pub combatants: Vec<CombatantState>,
    /// Items left in the party inventory, in id order.
    pub inventory: Vec<InventoryEntry>,
    /// Active or Wait mode.
    pub time_mode: TimeMode,
    /// Battle speed in percent (100 = real time).
    pub speed_percent: u16,
    /// Stopped by the sandbox debug tools.
    pub paused: bool,
}

/// One combatant on the battle screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CombatantState {
    /// Position in the battle.
    pub id: CombatantId,
    /// Side it fights on.
    pub side: Side,
    /// Fluent message id of its name.
    pub name: String,
    /// Level.
    pub level: u8,
    /// Current HP.
    pub hp: u16,
    /// Max HP.
    pub max_hp: u16,
    /// Current MP.
    pub mp: u16,
    /// Max MP.
    pub max_mp: u16,
    /// ATB gauge, `0..=atbFull`.
    pub atb: u32,
    /// Value of a full gauge.
    pub atb_full: u32,
    /// Turn cycle.
    pub turn: TurnState,
    /// Active statuses.
    pub statuses: Vec<StatusState>,
    /// Halving damage with Defend.
    pub defending: bool,
    /// Ran off (a thief with its loot): out of the battle.
    pub fled: bool,
    /// Commands come from the player.
    pub player_controlled: bool,
    /// Engine-controlled only: its next action and who it will hit,
    /// announced while its gauge charges.
    pub intent: Option<AnnouncedIntent>,
    /// What its Attack command does.
    pub attack: ActionId,
    /// Spells and abilities it knows.
    pub actions: Vec<ActionId>,
}

/// An inventory line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct InventoryEntry {
    /// The item.
    pub item: ItemId,
    /// How many are left.
    pub count: u16,
}

/// What the command menu needs to know about actions and items. Sent once
/// per battle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Catalog {
    /// Actions known by anyone in the battle (and those items perform).
    pub actions: BTreeMap<ActionId, ActionInfo>,
    /// Items in the inventory.
    pub items: BTreeMap<ItemId, ItemInfo>,
}

/// The menu-relevant part of an action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ActionInfo {
    /// Command family.
    pub category: ActionCategory,
    /// Who it can be aimed at.
    pub target: TargetRule,
    /// MP cost.
    pub mp_cost: u16,
    /// Aimed at knocked-out allies (revival).
    pub targets_knocked_out: bool,
    /// Element of its damage, if any (picks its sound and visual).
    pub element: Option<Element>,
}

/// The menu-relevant part of an item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ItemInfo {
    /// The action it performs.
    pub action: ActionId,
}

/// One status on a combatant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StatusState {
    /// Which status.
    pub kind: StatusKind,
    /// Hurts its bearer (drawn in red).
    pub harmful: bool,
    /// Time left; `None` for statuses that end on an event.
    pub timer: Option<StatusTimer>,
}

/// Body of `GET /api/sandbox`: scenario names (file names without `.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SandboxList {
    /// Scenario names, sorted.
    pub scenarios: Vec<String>,
}

impl BattleState {
    /// Snapshot of `battle` with the session's time settings.
    #[must_use]
    pub fn of(battle: &Battle, time_mode: TimeMode, speed_percent: u16, paused: bool) -> Self {
        let combatants = battle
            .combatants()
            .iter()
            .map(|c| CombatantState {
                id: c.id(),
                side: c.side(),
                name: c.name().to_owned(),
                level: c.stats().level,
                hp: c.hp(),
                max_hp: c.stats().max_hp,
                mp: c.mp(),
                max_mp: c.stats().max_mp,
                atb: c.atb(),
                atb_full: ATB_FULL,
                turn: c.state(),
                statuses: c
                    .statuses()
                    .map(|(kind, timer)| StatusState {
                        kind,
                        harmful: kind.is_harmful(),
                        timer,
                    })
                    .collect(),
                defending: c.is_defending(),
                fled: c.has_fled(),
                player_controlled: c.is_player_controlled(),
                intent: battle.announced_intent(c.id()),
                attack: c.attack_action().clone(),
                actions: c.known_actions().to_vec(),
            })
            .collect();
        let inventory = battle
            .inventory()
            .iter()
            .map(|(item, &count)| InventoryEntry {
                item: item.clone(),
                count,
            })
            .collect();
        Self {
            tick: battle.tick(),
            outcome: battle.outcome(),
            combatants,
            inventory,
            time_mode,
            speed_percent,
            paused,
        }
    }
}

impl Catalog {
    /// Every action and item a combatant of `battle` may use.
    #[must_use]
    pub fn of(battle: &Battle) -> Self {
        let mut actions = BTreeMap::new();
        let mut add = |id: &ActionId| {
            if let Some(def) = battle.action(id) {
                actions.insert(
                    id.clone(),
                    ActionInfo {
                        category: def.category,
                        target: def.target,
                        mp_cost: def.mp_cost,
                        targets_knocked_out: def.effect.targets_knocked_out(),
                        element: def.effect.element(),
                    },
                );
            }
        };
        for c in battle.combatants() {
            add(c.attack_action());
            c.known_actions().iter().for_each(&mut add);
        }

        let mut items = BTreeMap::new();
        for id in battle.inventory().keys() {
            if let Some(item) = battle.item(id) {
                add(&item.action);
                items.insert(
                    id.clone(),
                    ItemInfo {
                        action: item.action.clone(),
                    },
                );
            }
        }
        Self { actions, items }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statuses_serialize_in_snake_case() {
        let json = serde_json::to_string(&StatusKind::MagicBarrier).unwrap();

        assert_eq!(json, "\"magic_barrier\"");
    }
}
