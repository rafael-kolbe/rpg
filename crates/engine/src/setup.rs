//! What a battle starts from, and the checks that reject a broken setup before
//! the first tick.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::action::{ActionCategory, ActionDef, Affinity, Effect, Element, ItemDef, TargetRule};
use crate::ai::{AiAct, AiProfile, TargetPick};
use crate::battle::PartyControl;
use crate::ids::{ActionId, ItemId, Side};
use crate::stats::Stats;
use crate::status::StatusKind;

/// Largest number of combatants on one side (raids allow up to 8).
pub(crate) const MAX_PER_SIDE: usize = 8;

/// Everything needed to start a battle: who fights, and the definitions of
/// every action and item they may use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleSetup {
    /// Who fights, and how the battle opens.
    pub encounter: Encounter,
    /// Action definitions by id.
    pub actions: BTreeMap<ActionId, ActionDef>,
    /// Item definitions by id.
    pub items: BTreeMap<ItemId, ItemDef>,
}

/// The fighters of a battle and its opening. A sandbox scenario file is an
/// `Encounter`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Encounter {
    /// Normal start or an ambush.
    #[serde(default)]
    pub opening: Opening,
    /// Items the party carries into the battle, by id.
    #[serde(default)]
    pub inventory: BTreeMap<ItemId, u16>,
    /// The player's side, in display order.
    pub party: Vec<CombatantSetup>,
    /// The monsters, in display order.
    pub enemies: Vec<CombatantSetup>,
}

/// How a battle opens.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Opening {
    /// ATB gauges start at a random 0–30%.
    #[default]
    Normal,
    /// The party ambushes: party gauges full, enemies at 0% and Distracted.
    PartyAmbush,
    /// The enemies ambush: enemy gauges full, party at 0% and Distracted.
    EnemyAmbush,
}

/// One combatant as it enters the battle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CombatantSetup {
    /// Fluent message id of the display name (e.g. `monster-forest-wolf`).
    pub name: String,
    /// Level, HP, MP, attributes, equipment.
    pub stats: Stats,
    /// The action the Attack command performs (category `attack`).
    pub attack: ActionId,
    /// Spells and abilities it knows (categories `magic` and `ability`).
    #[serde(default)]
    pub actions: Vec<ActionId>,
    /// Its Limit, if it has one (category `limit`). Combatants without a
    /// Limit do not fill a Limit gauge.
    #[serde(default)]
    pub limit: Option<ActionId>,
    /// Elemental weaknesses and resistances.
    #[serde(default)]
    pub affinities: BTreeMap<Element, Affinity>,
    /// Statuses that never take hold.
    #[serde(default)]
    pub status_immunities: Vec<StatusKind>,
    /// Behavior when the engine controls it. Required for enemies, and for
    /// party members when the party is AI-controlled.
    #[serde(default)]
    pub ai: Option<AiProfile>,
}

/// Why a [`BattleSetup`] was rejected.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SetupError {
    /// A side has no combatants or more than the maximum.
    #[error("{side:?} must have 1..={MAX_PER_SIDE} combatants, has {count}")]
    SideSize {
        /// The side.
        side: Side,
        /// How many it has.
        count: usize,
    },
    /// A combatant's stats break a range rule.
    #[error("combatant `{name}`: {reason}")]
    InvalidStats {
        /// Display name id of the combatant.
        name: String,
        /// The broken rule.
        reason: String,
    },
    /// A reference to an action that has no definition.
    #[error("`{owner}` refers to unknown action `{action}`")]
    UnknownAction {
        /// Who refers to it.
        owner: String,
        /// The missing id.
        action: ActionId,
    },
    /// A reference to an item that has no definition.
    #[error("`{owner}` refers to unknown item `{item}`")]
    UnknownItem {
        /// Who refers to it.
        owner: String,
        /// The missing id.
        item: ItemId,
    },
    /// An action used in a slot meant for another category.
    #[error("`{owner}` uses action `{action}` as {expected}, but it is {found:?}")]
    WrongCategory {
        /// Who uses it.
        owner: String,
        /// The action.
        action: ActionId,
        /// What the slot accepts.
        expected: &'static str,
        /// The action's actual category.
        found: ActionCategory,
    },
    /// An action definition is internally inconsistent.
    #[error("action `{action}`: {reason}")]
    InvalidAction {
        /// The action.
        action: ActionId,
        /// The broken rule.
        reason: String,
    },
    /// A combatant has no AI but the engine must control it.
    #[error("combatant `{name}` is engine-controlled but has no `ai`")]
    MissingAi {
        /// Display name id of the combatant.
        name: String,
    },
    /// An AI profile is malformed or refers to something the combatant cannot do.
    #[error("combatant `{name}` AI: {reason}")]
    InvalidAi {
        /// Display name id of the combatant.
        name: String,
        /// The broken rule.
        reason: String,
    },
}

impl BattleSetup {
    /// Checks every rule a battle relies on.
    pub(crate) fn validate(&self, party_control: PartyControl) -> Result<(), SetupError> {
        for (id, action) in &self.actions {
            validate_action(id, action)?;
        }
        for (id, item) in &self.items {
            let action = self.action(id.as_str(), &item.action)?;
            expect_category(
                id.as_str(),
                &item.action,
                action,
                &[ActionCategory::Item],
                "an item",
            )?;
        }
        for id in self.encounter.inventory.keys() {
            if !self.items.contains_key(id) {
                return Err(SetupError::UnknownItem {
                    owner: "inventory".to_owned(),
                    item: id.clone(),
                });
            }
        }

        for (side, members) in [
            (Side::Party, &self.encounter.party),
            (Side::Enemies, &self.encounter.enemies),
        ] {
            if members.is_empty() || members.len() > MAX_PER_SIDE {
                return Err(SetupError::SideSize {
                    side,
                    count: members.len(),
                });
            }
            let engine_controlled = side == Side::Enemies || party_control == PartyControl::Ai;
            for member in members {
                self.validate_combatant(member, engine_controlled)?;
            }
        }
        Ok(())
    }

    fn validate_combatant(
        &self,
        member: &CombatantSetup,
        engine_controlled: bool,
    ) -> Result<(), SetupError> {
        let name = member.name.as_str();
        if let Some(reason) = member.stats.violation() {
            return Err(SetupError::InvalidStats {
                name: name.to_owned(),
                reason,
            });
        }

        let attack = self.action(name, &member.attack)?;
        expect_category(
            name,
            &member.attack,
            attack,
            &[ActionCategory::Attack],
            "attack",
        )?;
        for id in &member.actions {
            let action = self.action(name, id)?;
            expect_category(
                name,
                id,
                action,
                &[ActionCategory::Magic, ActionCategory::Ability],
                "a spell or ability",
            )?;
        }
        if let Some(id) = &member.limit {
            let action = self.action(name, id)?;
            expect_category(name, id, action, &[ActionCategory::Limit], "limit")?;
        }

        match &member.ai {
            Some(profile) => self.validate_ai(member, profile),
            None if engine_controlled => Err(SetupError::MissingAi {
                name: name.to_owned(),
            }),
            None => Ok(()),
        }
    }

    fn validate_ai(&self, member: &CombatantSetup, profile: &AiProfile) -> Result<(), SetupError> {
        let invalid = |reason: String| SetupError::InvalidAi {
            name: member.name.clone(),
            reason,
        };

        if profile.phases.first().map(|phase| phase.from_hp_percent) != Some(100) {
            return Err(invalid(
                "the first phase must start at fromHpPercent 100".to_owned(),
            ));
        }
        if profile
            .phases
            .windows(2)
            .any(|pair| pair[1].from_hp_percent >= pair[0].from_hp_percent)
        {
            return Err(invalid(
                "phases must have decreasing fromHpPercent".to_owned(),
            ));
        }

        for phase in &profile.phases {
            if phase.rules.is_empty() {
                return Err(invalid(format!(
                    "phase {} has no rules",
                    phase.from_hp_percent
                )));
            }
            for rule in &phase.rules {
                let action_id = match &rule.act {
                    AiAct::Attack => &member.attack,
                    AiAct::Action { id } => {
                        if !member.actions.contains(id) {
                            return Err(invalid(format!("uses `{id}`, which it does not know")));
                        }
                        id
                    }
                    AiAct::Item { id } => match self.items.get(id) {
                        Some(item) => &item.action,
                        None => {
                            return Err(SetupError::UnknownItem {
                                owner: member.name.clone(),
                                item: id.clone(),
                            });
                        }
                    },
                };
                let action = self.action(&member.name, action_id)?;
                if !pick_fits(rule.target, action) {
                    return Err(invalid(format!(
                        "target {:?} does not fit action `{action_id}`",
                        rule.target
                    )));
                }
            }
        }
        Ok(())
    }

    fn action(&self, owner: &str, id: &ActionId) -> Result<&ActionDef, SetupError> {
        self.actions
            .get(id)
            .ok_or_else(|| SetupError::UnknownAction {
                owner: owner.to_owned(),
                action: id.clone(),
            })
    }
}

fn validate_action(id: &ActionId, action: &ActionDef) -> Result<(), SetupError> {
    let invalid = |reason: &str| {
        Err(SetupError::InvalidAction {
            action: id.clone(),
            reason: reason.to_owned(),
        })
    };

    match &action.effect {
        Effect::PercentCurrentHp { percent, .. } if !(1..=100).contains(percent) => {
            invalid("percent must be 1..=100")
        }
        Effect::Revive { hp_percent } if !(1..=100).contains(hp_percent) => {
            invalid("hpPercent must be 1..=100")
        }
        Effect::Revive { .. } if !action.target.on_own_side() => {
            invalid("revive must target the user's side")
        }
        Effect::ApplyStatus { accuracy, .. } if !(1..=100).contains(accuracy) => {
            invalid("accuracy must be 1..=100")
        }
        Effect::ApplyStatus {
            status,
            duration_ticks,
            ..
        } => match (status.needs_duration(), duration_ticks) {
            (true, None | Some(0)) => invalid("this status needs a durationTicks above 0"),
            (false, Some(_)) => invalid("distracted takes no durationTicks"),
            _ => Ok(()),
        },
        _ => Ok(()),
    }
}

fn expect_category(
    owner: &str,
    id: &ActionId,
    action: &ActionDef,
    allowed: &[ActionCategory],
    expected: &'static str,
) -> Result<(), SetupError> {
    if allowed.contains(&action.category) {
        Ok(())
    } else {
        Err(SetupError::WrongCategory {
            owner: owner.to_owned(),
            action: id.clone(),
            expected,
            found: action.category,
        })
    }
}

/// Whether an AI target pick can aim the given action.
fn pick_fits(pick: TargetPick, action: &ActionDef) -> bool {
    let revive = action.effect.targets_knocked_out();
    match action.target {
        TargetRule::SingleEnemy => matches!(
            pick,
            TargetPick::RandomOpponent
                | TargetPick::LowestHpOpponent
                | TargetPick::OpponentWithStatus { .. }
        ),
        TargetRule::SingleAlly if revive => pick == TargetPick::KnockedOutAlly,
        TargetRule::SingleAlly => matches!(
            pick,
            TargetPick::User | TargetPick::RandomAlly | TargetPick::LowestHpAlly
        ),
        TargetRule::AllEnemies | TargetRule::AllAllies | TargetRule::User => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{catalog_setup, fighter};

    #[test]
    fn accepts_the_test_setup() {
        let setup = catalog_setup(vec![fighter("hero")], vec![fighter("wolf")]);

        assert_eq!(setup.validate(PartyControl::Player), Ok(()));
    }

    #[test]
    fn rejects_an_empty_side() {
        let setup = catalog_setup(vec![fighter("hero")], Vec::new());

        assert!(matches!(
            setup.validate(PartyControl::Player),
            Err(SetupError::SideSize {
                side: Side::Enemies,
                count: 0
            })
        ));
    }

    #[test]
    fn rejects_an_unknown_spell() {
        let mut hero = fighter("hero");
        hero.actions.push(ActionId::new("meteor"));
        let setup = catalog_setup(vec![hero], vec![fighter("wolf")]);

        assert!(matches!(
            setup.validate(PartyControl::Player),
            Err(SetupError::UnknownAction { .. })
        ));
    }

    #[test]
    fn rejects_a_spell_in_the_attack_slot() {
        let mut hero = fighter("hero");
        hero.attack = ActionId::new("fire");
        let setup = catalog_setup(vec![hero], vec![fighter("wolf")]);

        assert!(matches!(
            setup.validate(PartyControl::Player),
            Err(SetupError::WrongCategory { .. })
        ));
    }

    #[test]
    fn requires_ai_for_an_ai_controlled_party() {
        let mut hero = fighter("hero");
        hero.ai = None;
        let setup = catalog_setup(vec![hero], vec![fighter("wolf")]);

        assert!(matches!(
            setup.validate(PartyControl::Ai),
            Err(SetupError::MissingAi { .. })
        ));
    }

    #[test]
    fn rejects_a_timed_status_without_duration() {
        let mut setup = catalog_setup(vec![fighter("hero")], vec![fighter("wolf")]);
        let haste = setup.actions.get_mut(&ActionId::new("haste")).unwrap();
        haste.effect = Effect::ApplyStatus {
            status: StatusKind::Haste,
            duration_ticks: None,
            accuracy: 100,
        };

        assert!(matches!(
            setup.validate(PartyControl::Player),
            Err(SetupError::InvalidAction { .. })
        ));
    }
}
