//! Rule-based behavior for engine-controlled combatants.
//!
//! An AI combatant decides as soon as its gauge starts charging, so the screen
//! can announce its target (its [`Intent`]); the choice is made again whenever
//! it can no longer be carried out (target knocked out, MP gone) and runs when
//! the gauge fills. To decide, the active phase is chosen by its HP, then its
//! rules are read in order:
//! 1. the first rule with a condition (anything but `always`) that holds and
//!    can be carried out wins;
//! 2. otherwise one `always` rule is drawn at random, by weight;
//! 3. if nothing applies, it attacks a random opponent.

use serde::{Deserialize, Serialize};

use crate::action::{ActionDef, TargetRule};
use crate::battle::{Battle, CommandKind};
use crate::combatant::Combatant;
use crate::ids::{ActionId, CombatantId, ItemId, Side};
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
    /// The combatant itself does not have the status (renewing Cover).
    UserLacksStatus {
        /// The status.
        status: StatusKind,
    },
}

/// What an AI rule does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum AiAct {
    /// The basic attack.
    Attack,
    /// A known spell or technic.
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
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
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
    /// A standing ally (never the user) with this display name: a Lich
    /// devouring its Zombie, never its other servants.
    AllyNamed {
        /// Fluent message id of the name (e.g. `monster-zombie`).
        name: String,
    },
    /// A knocked-out ally with this display name.
    KnockedOutAllyNamed {
        /// Fluent message id of the name.
        name: String,
    },
}

/// What an engine-controlled combatant will do when its gauge fills.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Intent {
    pub(crate) kind: CommandKind,
    pub(crate) target: Option<CombatantId>,
}

/// What an engine-controlled combatant announces while its gauge charges:
/// the action and who it will hit.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnouncedIntent {
    /// The command it will use.
    pub command: IntentCommand,
    /// Who it will hit.
    pub target: IntentTarget,
}

/// The command of an [`AnnouncedIntent`]. Attack is announced as the action
/// it performs, so the screen names it like any other action.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum IntentCommand {
    /// An action: the basic attack, a spell or a technic.
    Action {
        /// Which.
        id: ActionId,
    },
    /// An item from the inventory.
    Item {
        /// Which.
        id: ItemId,
    },
    /// Defend.
    Defend,
}

/// Who an announced action will hit.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum IntentTarget {
    /// One combatant.
    One {
        /// Who.
        target: CombatantId,
    },
    /// Everyone standing on a side.
    Side {
        /// Which side.
        side: Side,
    },
    /// The combatant itself.
    User,
}

impl Battle {
    /// Keeps the intent of an engine-controlled combatant current: decided
    /// when it starts charging, decided again once it cannot be carried out.
    pub(crate) fn refresh_intent(&mut self, actor: CombatantId) {
        let index = usize::from(actor.0);
        let valid = self.combatants[index]
            .intent
            .as_ref()
            .is_some_and(|intent| self.can_perform(actor, &intent.kind, intent.target));
        if !valid {
            let (kind, target) = self.decide(actor);
            self.combatants[index].intent = Some(Intent { kind, target });
        }
    }

    /// What `id` announced it will do and to whom; `None` while it has no
    /// intent (player-controlled, knocked out, acting).
    #[must_use]
    pub fn announced_intent(&self, id: CombatantId) -> Option<AnnouncedIntent> {
        let combatant = self.combatant(id)?;
        let intent = combatant.intent.as_ref()?;
        if intent.kind == CommandKind::Defend {
            return Some(AnnouncedIntent {
                command: IntentCommand::Defend,
                target: IntentTarget::User,
            });
        }
        let (action, item) = self.resolve_command(id, &intent.kind).ok()?;
        let target = match self.actions.get(&action)?.target {
            TargetRule::SingleEnemy | TargetRule::SingleAlly => IntentTarget::One {
                target: intent.target?,
            },
            TargetRule::AllEnemies => IntentTarget::Side {
                side: combatant.side.opponent(),
            },
            TargetRule::AllAllies => IntentTarget::Side {
                side: combatant.side,
            },
            TargetRule::User => IntentTarget::User,
        };
        let command = match item {
            Some(id) => IntentCommand::Item { id },
            None => IntentCommand::Action { id: action },
        };
        Some(AnnouncedIntent { command, target })
    }

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
            let candidates = self.candidates(actor, &rule.target, &def);
            if def.target.needs_target() && candidates.is_empty() {
                continue;
            }

            if rule.when == Condition::Always {
                always.push((rule.weight, kind, rule.target.clone(), def));
            } else {
                let target = self.pick(actor, &rule.target, &def);
                return (kind, target);
            }
        }

        let total: u64 = always.iter().map(|(weight, ..)| u64::from(*weight)).sum();
        if total > 0 {
            let mut roll = self.rng.below(total);
            for (weight, kind, pick, def) in always {
                if roll < u64::from(weight) {
                    let target = self.pick(actor, &pick, &def);
                    return (kind, target);
                }
                roll -= u64::from(weight);
            }
        }

        let attack = self.combatants[usize::from(actor.0)].attack.clone();
        let def = self.actions[&attack].clone();
        let target = self.pick(actor, &TargetPick::RandomOpponent, &def);
        (CommandKind::Attack, target)
    }

    fn holds(&self, actor: CombatantId, condition: Condition) -> bool {
        let me = &self.combatants[usize::from(actor.0)];
        let allies = || self.combatants.iter().filter(|c| c.side == me.side);
        let opponents = || {
            self.combatants
                .iter()
                .filter(|c| c.side != me.side && !c.is_out())
        };
        match condition {
            Condition::Always => true,
            Condition::UserHpBelow { percent } => me.hp_percent() < u16::from(percent),
            Condition::AllyHpBelow { percent } => allies()
                .filter(|c| !c.is_out())
                .any(|c| c.hp_percent() < u16::from(percent)),
            Condition::AllyKnockedOut => allies().any(|c| c.is_knocked_out() && !c.has_fled()),
            Condition::OpponentHasStatus { status } => opponents().any(|c| c.has_status(status)),
            Condition::UserLacksStatus { status } => !me.has_status(status),
        }
    }

    /// Eligible targets for a pick, best first for the "lowest HP" picks.
    fn candidates(
        &self,
        actor: CombatantId,
        pick: &TargetPick,
        def: &ActionDef,
    ) -> Vec<CombatantId> {
        let mut eligible: Vec<&Combatant> = self
            .combatants
            .iter()
            .filter(|c| self.is_eligible(actor, def, c.id))
            .collect();
        match pick {
            TargetPick::User => eligible.retain(|c| c.id == actor),
            TargetPick::OpponentWithStatus { status } => eligible.retain(|c| c.has_status(*status)),
            TargetPick::AllyNamed { name } | TargetPick::KnockedOutAllyNamed { name } => {
                eligible.retain(|c| c.id != actor && &c.name == name);
            }
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
        pick: &TargetPick,
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
    use crate::event::BattleEvent;
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
    fn an_enemy_announces_its_action_and_target_while_charging_and_does_it() {
        let setup = catalog_setup(vec![fighter("a"), fighter("b")], vec![fighter("wolf")]);
        let mut battle = Battle::new(setup, 7, PartyControl::Player).unwrap();
        let wolf = CombatantId(2);

        battle.step();
        let Some(AnnouncedIntent {
            command: IntentCommand::Action { id: announced },
            target: IntentTarget::One { target },
        }) = battle.announced_intent(wolf)
        else {
            panic!("the wolf announces no action on a single target");
        };
        let events = crate::test_support::run_until(
            &mut battle,
            |e| matches!(e, BattleEvent::ActionStarted { actor, .. } if *actor == wolf),
        );

        assert!(events.iter().any(|e| matches!(
            e,
            BattleEvent::ActionStarted { actor, action, targets, .. }
                if *actor == wolf && *action == announced && *targets == vec![target]
        )));
    }

    #[test]
    fn a_knocked_out_target_makes_the_enemy_choose_again() {
        let setup = catalog_setup(vec![fighter("a"), fighter("b")], vec![fighter("wolf")]);
        let mut battle = Battle::new(setup, 7, PartyControl::Player).unwrap();
        let wolf = CombatantId(2);
        battle.step();
        let Some(IntentTarget::One { target }) = battle.announced_intent(wolf).map(|i| i.target)
        else {
            panic!("the wolf announces no single target");
        };

        battle.knock_out(target, &mut Vec::new());
        battle.step();

        let other = CombatantId(1 - target.0);
        assert_eq!(
            battle.announced_intent(wolf).map(|i| i.target),
            Some(IntentTarget::One { target: other })
        );
    }

    #[test]
    fn heroes_the_player_controls_announce_nothing() {
        let setup = catalog_setup(vec![fighter("hero")], vec![fighter("wolf")]);
        let mut battle = Battle::new(setup, 7, PartyControl::Player).unwrap();

        battle.step();

        assert_eq!(battle.announced_intent(CombatantId(0)), None);
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
