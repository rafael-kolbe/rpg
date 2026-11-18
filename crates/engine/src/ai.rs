//! Rule-based behavior for engine-controlled combatants.
//!
//! When an AI combatant's gauge fills, the active phase is chosen by its HP,
//! then its rules are read in order:
//! 1. the first rule with a condition (anything but `always`) that holds and
//!    can be carried out wins;
//! 2. otherwise one `always` rule is drawn at random, by weight;
//! 3. if nothing applies, it attacks a random opponent.

use serde::{Deserialize, Serialize};

use crate::action::ActionDef;
use crate::battle::{Battle, CommandKind};
use crate::combatant::Combatant;
use crate::ids::{ActionId, CombatantId, ItemId};
use crate::status::StatusKind;

/// How an engine-controlled combatant chooses its actions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiProfile {
    /// Behavior by HP range. The first phase starts at 100%; each next one
    /// takes over when HP drops to its `fromHpPercent` (bosses changing
    /// tactics).
    pub phases: Vec<AiPhase>,
}

/// The rules used while HP is at or below `from_hp_percent`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiPhase {
    /// HP percentage at which this phase starts.
    pub from_hp_percent: u8,
    /// Rules, in priority order.
    pub rules: Vec<AiRule>,
}

/// One possible choice of an AI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AiRule {
    /// When the rule applies.
    #[serde(default)]
    pub when: Condition,
    /// Relative chance among `always` rules.
    #[serde(default = "default_weight")]
    pub weight: u16,
    /// What it does.
    pub act: AiAct,
    /// Who it aims at (ignored for actions that hit a whole side or the user).
    #[serde(default)]
    pub target: TargetPick,
}

fn default_weight() -> u16 {
    1
}

/// When an AI rule applies.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Condition {
    /// Always (drawn by weight).
    #[default]
    Always,
    /// The combatant's own HP is below a percentage.
    UserHpBelow {
        /// Threshold, in percent.
        percent: u8,
    },
    /// Some ally (the user included) has HP below a percentage.
    AllyHpBelow {
        /// Threshold, in percent.
        percent: u8,
    },
    /// Some ally is knocked out.
    AllyKnockedOut,
    /// Some opponent has the status.
    OpponentHasStatus {
        /// The status.
        status: StatusKind,
    },
}

/// What an AI rule does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum AiAct {
    /// The basic attack (or the Limit, when full).
    Attack,
    /// A known spell or ability.
    Action {
        /// The action.
        id: ActionId,
    },
    /// An item from the inventory.
    Item {
        /// The item.
        id: ItemId,
    },
}

/// Who an AI rule aims at.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum TargetPick {
    /// A random standing opponent.
    #[default]
    RandomOpponent,
    /// The standing opponent with the lowest HP percentage.
    LowestHpOpponent,
    /// A standing opponent with the status.
    OpponentWithStatus {
        /// The status.
        status: StatusKind,
    },
    /// The combatant itself.
    User,
    /// A random standing ally.
    RandomAlly,
    /// The standing ally with the lowest HP percentage.
    LowestHpAlly,
    /// A knocked-out ally (for revival).
    KnockedOutAlly,
}

impl Battle {
    /// Chooses a command for an engine-controlled combatant.
    pub(crate) fn decide(&mut self, actor: CombatantId) -> (CommandKind, Option<CombatantId>) {
        let combatant = &self.combatants[usize::from(actor.0)];
        let hp_percent = combatant.hp_percent();
        let phase = combatant.ai.as_ref().and_then(|profile| {
            profile
                .phases
                .iter()
                .rev()
                .find(|phase| hp_percent <= u16::from(phase.from_hp_percent))
                .cloned()
        });

        let mut always = Vec::new();
        for rule in phase.iter().flat_map(|phase| &phase.rules) {
            if !self.holds(actor, rule.when) {
                continue;
            }
            let kind = match &rule.act {
                AiAct::Attack => CommandKind::Attack,
                AiAct::Action { id } => CommandKind::Action { id: id.clone() },
                AiAct::Item { id } => CommandKind::Item { id: id.clone() },
            };
            let Ok((action, _)) = self.resolve_command(actor, &kind) else {
                continue;
            };
            let def = self.actions[&action].clone();
            if self.check_usable(actor, &def).is_err() {
                continue;
            }
            let candidates = self.candidates(actor, rule.target, &def);
            if def.target.needs_target() && candidates.is_empty() {
                continue;
            }

            if rule.when == Condition::Always {
                always.push((rule.weight, kind, rule.target, def));
            } else {
                let target = self.pick(actor, rule.target, &def);
                return (kind, target);
            }
        }

        let total: u64 = always.iter().map(|(weight, ..)| u64::from(*weight)).sum();
        if total > 0 {
            let mut roll = self.rng.below(total);
            for (weight, kind, pick, def) in always {
                if roll < u64::from(weight) {
                    let target = self.pick(actor, pick, &def);
                    return (kind, target);
                }
                roll -= u64::from(weight);
            }
        }

        let attack = self.combatants[usize::from(actor.0)].attack.clone();
        let def = self.actions[&attack].clone();
        let target = self.pick(actor, TargetPick::RandomOpponent, &def);
        (CommandKind::Attack, target)
    }

    fn holds(&self, actor: CombatantId, condition: Condition) -> bool {
        let me = &self.combatants[usize::from(actor.0)];
        let allies = || self.combatants.iter().filter(|c| c.side == me.side);
        let opponents = || {
            self.combatants
                .iter()
                .filter(|c| c.side != me.side && !c.is_knocked_out())
        };
        match condition {
            Condition::Always => true,
            Condition::UserHpBelow { percent } => me.hp_percent() < u16::from(percent),
            Condition::AllyHpBelow { percent } => allies()
                .filter(|c| !c.is_knocked_out())
                .any(|c| c.hp_percent() < u16::from(percent)),
            Condition::AllyKnockedOut => allies().any(Combatant::is_knocked_out),
            Condition::OpponentHasStatus { status } => opponents().any(|c| c.has_status(status)),
        }
    }

    /// Eligible targets for a pick, best first for the "lowest HP" picks.
    fn candidates(
        &self,
        actor: CombatantId,
        pick: TargetPick,
        def: &ActionDef,
    ) -> Vec<CombatantId> {
        let mut eligible: Vec<&Combatant> = self
            .combatants
            .iter()
            .filter(|c| self.is_eligible(actor, def, c.id))
            .collect();
        match pick {
            TargetPick::User => eligible.retain(|c| c.id == actor),
            TargetPick::OpponentWithStatus { status } => eligible.retain(|c| c.has_status(status)),
            TargetPick::LowestHpOpponent | TargetPick::LowestHpAlly => {
                eligible.sort_by_key(|c| (c.hp_percent(), c.id));
            }
            TargetPick::RandomOpponent | TargetPick::RandomAlly | TargetPick::KnockedOutAlly => {}
        }
        eligible.into_iter().map(|c| c.id).collect()
    }

    fn pick(
        &mut self,
        actor: CombatantId,
        pick: TargetPick,
        def: &ActionDef,
    ) -> Option<CombatantId> {
        if !def.target.needs_target() {
            return None;
        }
        let candidates = self.candidates(actor, pick, def);
        match pick {
            TargetPick::LowestHpOpponent | TargetPick::LowestHpAlly | TargetPick::User => {
                candidates.first().copied()
            }
            _ => self.rng.pick(&candidates),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::PartyControl;
    use crate::test_support::{catalog_setup, fighter};

    fn mage_with(rules: Vec<AiRule>) -> crate::setup::CombatantSetup {
        let mut mage = fighter("mage");
        mage.ai = Some(AiProfile {
            phases: vec![AiPhase {
                from_hp_percent: 100,
                rules,
            }],
        });
        mage
    }

    fn rule(when: Condition, act: AiAct, target: TargetPick) -> AiRule {
        AiRule {
            when,
            weight: 1,
            act,
            target,
        }
    }

    #[test]
    fn a_conditional_rule_that_holds_wins_over_always_rules() {
        let enemy = mage_with(vec![
            rule(Condition::Always, AiAct::Attack, TargetPick::RandomOpponent),
            rule(
                Condition::UserHpBelow { percent: 50 },
                AiAct::Action {
                    id: ActionId::new("cure"),
                },
                TargetPick::User,
            ),
        ]);
        let setup = catalog_setup(vec![fighter("hero")], vec![enemy]);
        let mut battle = Battle::new(setup, 5, PartyControl::Player).unwrap();
        let me = CombatantId(1);
        battle.combatant_mut(me).hp = 100;

        let (kind, target) = battle.decide(me);

        assert_eq!(
            kind,
            CommandKind::Action {
                id: ActionId::new("cure")
            }
        );
        assert_eq!(target, Some(me));
    }

    #[test]
    fn rules_it_cannot_afford_are_skipped() {
        let enemy = mage_with(vec![rule(
            Condition::Always,
            AiAct::Action {
                id: ActionId::new("fire"),
            },
            TargetPick::RandomOpponent,
        )]);
        let setup = catalog_setup(vec![fighter("hero")], vec![enemy]);
        let mut battle = Battle::new(setup, 5, PartyControl::Player).unwrap();
        battle.combatant_mut(CombatantId(1)).mp = 0;

        let (kind, target) = battle.decide(CombatantId(1));

        assert_eq!(kind, CommandKind::Attack);
        assert_eq!(target, Some(CombatantId(0)));
    }

    #[test]
    fn a_deeper_phase_takes_over_at_low_hp() {
        let mut boss = fighter("boss");
        boss.ai = Some(AiProfile {
            phases: vec![
                AiPhase {
                    from_hp_percent: 100,
                    rules: vec![rule(
                        Condition::Always,
                        AiAct::Attack,
                        TargetPick::RandomOpponent,
                    )],
                },
                AiPhase {
                    from_hp_percent: 50,
                    rules: vec![rule(
                        Condition::Always,
                        AiAct::Action {
                            id: ActionId::new("fire"),
                        },
                        TargetPick::LowestHpOpponent,
                    )],
                },
            ],
        });
        let setup = catalog_setup(vec![fighter("hero")], vec![boss]);
        let mut battle = Battle::new(setup, 5, PartyControl::Player).unwrap();
        battle.combatant_mut(CombatantId(1)).hp = 150;

        let (kind, _) = battle.decide(CombatantId(1));

        assert_eq!(
            kind,
            CommandKind::Action {
                id: ActionId::new("fire")
            }
        );
    }
}
