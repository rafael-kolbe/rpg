//! The battle loop: ATB gauges, commands, the action queue and the end of the
//! battle. [`Battle::step`] documents the order of a tick.

use std::collections::{BTreeMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::action::{ActionCategory, ActionDef, ItemDef, TargetRule};
use crate::atb::{ATB_FULL, Speed, gain_per_tick};
use crate::combatant::{Combatant, TurnState};
use crate::event::{BattleEvent, CancelReason, FailReason, StatusEndReason};
use crate::ids::{ActionId, CombatantId, ItemId, Side};
use crate::rng::Rng;
use crate::setup::{BattleSetup, Opening, SetupError};
use crate::status::{ActiveStatus, StatusKind};

/// Who chooses the party's commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartyControl {
    /// The player, through [`Battle::submit`].
    Player,
    /// The engine, using each member's `ai` (simulations, auto-battle).
    Ai,
}

/// How a battle ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// Every enemy was knocked out.
    Victory,
    /// Every party member was knocked out.
    Defeat,
}

/// A player's choice for a combatant whose turn is ready.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Command {
    /// The combatant acting.
    pub actor: CombatantId,
    /// What it does.
    pub kind: CommandKind,
    /// The chosen target, for single-target actions; `None` otherwise.
    #[serde(default)]
    pub target: Option<CombatantId>,
}

/// The battle menu commands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum CommandKind {
    /// Basic attack, or the Limit when the Limit gauge is full.
    Attack,
    /// Halve damage taken until the next action; spends the turn at once.
    Defend,
    /// A known spell or ability.
    Action {
        /// The action.
        id: ActionId,
    },
    /// Use an item from the inventory.
    Item {
        /// The item.
        id: ItemId,
    },
}

/// Why a command was refused. Nothing changes in the battle when it is.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum CommandError {
    /// The battle already ended.
    #[error("the battle is over")]
    BattleOver,
    /// No combatant has this id.
    #[error("no combatant {0}")]
    UnknownCombatant(CombatantId),
    /// The engine controls this combatant.
    #[error("{0} is not controlled by the player")]
    NotPlayerControlled(CombatantId),
    /// The combatant's gauge is not full, or it already has a command queued.
    #[error("{0} is not waiting for a command")]
    NotAwaitingCommand(CombatantId),
    /// The combatant does not know this spell or ability.
    #[error("{actor} does not know `{action}`")]
    UnknownAction {
        /// Who.
        actor: CombatantId,
        /// The action.
        action: ActionId,
    },
    /// Silenced combatants cannot cast spells.
    #[error("{0} is silenced")]
    Silenced(CombatantId),
    /// Not enough MP for the action.
    #[error("{actor} needs {needed} MP and has {available}")]
    NotEnoughMp {
        /// Who.
        actor: CombatantId,
        /// The action's cost.
        needed: u16,
        /// Current MP.
        available: u16,
    },
    /// The inventory has none of this item.
    #[error("no `{0}` left")]
    OutOfItem(ItemId),
    /// A single-target action needs a target.
    #[error("this action needs a target")]
    MissingTarget,
    /// This action does not take a target.
    #[error("this action does not take a target")]
    UnexpectedTarget,
    /// The target is on the wrong side, knocked out (or not, for revival).
    #[error("{0} is not a valid target")]
    InvalidTarget(CombatantId),
}

/// A command turned into a queue entry.
#[derive(Debug, Clone)]
pub(crate) struct QueuedAction {
    pub(crate) actor: CombatantId,
    pub(crate) action: ActionId,
    pub(crate) item: Option<ItemId>,
    pub(crate) target: Option<CombatantId>,
}

/// A battle in progress. [`Battle::step`] documents the order of a tick.
#[derive(Debug, Clone)]
pub struct Battle {
    pub(crate) tick: u32,
    pub(crate) combatants: Vec<Combatant>,
    pub(crate) actions: BTreeMap<ActionId, ActionDef>,
    pub(crate) items: BTreeMap<ItemId, ItemDef>,
    pub(crate) inventory: BTreeMap<ItemId, u16>,
    pub(crate) queue: VecDeque<QueuedAction>,
    /// Ticks left before the queue may start its next action.
    pub(crate) busy_ticks: u16,
    pub(crate) rng: Rng,
    pub(crate) outcome: Option<Outcome>,
}

impl Battle {
    /// Builds a battle at tick 0.
    ///
    /// # Errors
    ///
    /// [`SetupError`] when the setup breaks a rule: unknown or misplaced
    /// actions, stats out of range, missing AI, malformed AI profiles.
    pub fn new(
        setup: BattleSetup,
        seed: u64,
        party_control: PartyControl,
    ) -> Result<Self, SetupError> {
        setup.validate(party_control)?;
        let BattleSetup {
            encounter,
            actions,
            items,
        } = setup;

        let party = encounter
            .party
            .into_iter()
            .map(|member| (Side::Party, member));
        let enemies = encounter
            .enemies
            .into_iter()
            .map(|member| (Side::Enemies, member));
        let combatants = party
            .chain(enemies)
            .zip(0u8..)
            .map(|((side, member), index)| {
                let player = side == Side::Party && party_control == PartyControl::Player;
                Combatant::new(CombatantId(index), side, member, player)
            })
            .collect();

        let mut battle = Self {
            tick: 0,
            combatants,
            actions,
            items,
            inventory: encounter.inventory,
            queue: VecDeque::new(),
            busy_ticks: 0,
            rng: Rng::new(seed),
            outcome: None,
        };
        battle.open(encounter.opening);
        Ok(battle)
    }

    fn open(&mut self, opening: Opening) {
        let ambushed = match opening {
            Opening::Normal => None,
            Opening::PartyAmbush => Some(Side::Enemies),
            Opening::EnemyAmbush => Some(Side::Party),
        };
        for index in 0..self.combatants.len() {
            let side = self.combatants[index].side;
            let atb = match ambushed {
                None => u32::try_from(self.rng.below(3001)).unwrap_or(0),
                Some(victims) if side == victims => 0,
                Some(_) => ATB_FULL,
            };
            let combatant = &mut self.combatants[index];
            combatant.atb = atb;
            if ambushed == Some(side) {
                combatant
                    .statuses
                    .push(ActiveStatus::new(StatusKind::Distracted, None));
            }
        }
    }

    /// Ticks elapsed (100 ms each).
    #[must_use]
    pub fn tick(&self) -> u32 {
        self.tick
    }

    /// How the battle ended, once it has.
    #[must_use]
    pub fn outcome(&self) -> Option<Outcome> {
        self.outcome
    }

    /// Every combatant: party first, then enemies.
    #[must_use]
    pub fn combatants(&self) -> &[Combatant] {
        &self.combatants
    }

    /// One combatant.
    #[must_use]
    pub fn combatant(&self, id: CombatantId) -> Option<&Combatant> {
        self.combatants.get(usize::from(id.0))
    }

    /// Items left, by id (items reserved by queued actions excluded).
    #[must_use]
    pub fn inventory(&self) -> &BTreeMap<ItemId, u16> {
        &self.inventory
    }

    /// Player-controlled combatants whose turn is waiting for a command.
    pub fn awaiting_command(&self) -> impl Iterator<Item = CombatantId> + '_ {
        self.combatants
            .iter()
            .filter(|c| c.player_controlled && c.state == TurnState::AwaitingCommand)
            .map(|c| c.id)
    }

    /// Gives a command to a player-controlled combatant whose turn is ready.
    ///
    /// # Errors
    ///
    /// [`CommandError`] when the command is not allowed right now; the battle
    /// is left untouched.
    pub fn submit(&mut self, command: &Command) -> Result<Vec<BattleEvent>, CommandError> {
        if self.outcome.is_some() {
            return Err(CommandError::BattleOver);
        }
        let actor = self
            .combatant(command.actor)
            .ok_or(CommandError::UnknownCombatant(command.actor))?;
        if !actor.player_controlled {
            return Err(CommandError::NotPlayerControlled(command.actor));
        }
        if actor.state != TurnState::AwaitingCommand {
            return Err(CommandError::NotAwaitingCommand(command.actor));
        }

        let mut events = Vec::new();
        self.perform(command.actor, &command.kind, command.target, &mut events)?;
        Ok(events)
    }

    /// Advances the battle by one tick (100 ms), in a fixed order:
    /// 1. status timers and Poison/Regen pulses;
    /// 2. ATB gauges fill; full gauges become turns (the AI decides at once,
    ///    the player is asked through [`BattleEvent::TurnReady`]);
    /// 3. the action queue runs: one action at a time, each occupying the
    ///    queue for its `durationTicks` while every gauge keeps filling;
    /// 4. the battle ends when one side has no combatant standing.
    ///
    /// Does nothing once the battle is over.
    pub fn step(&mut self) -> Vec<BattleEvent> {
        let mut events = Vec::new();
        if self.outcome.is_some() {
            return events;
        }
        self.tick += 1;

        self.tick_statuses(&mut events);
        if self.settle(&mut events) {
            return events;
        }
        self.fill_gauges(&mut events);
        self.run_queue(&mut events);
        self.settle(&mut events);
        events
    }

    /// Validates a command and either applies Defend or queues the action.
    /// Shared by the player ([`Battle::submit`]) and the AI.
    pub(crate) fn perform(
        &mut self,
        actor: CombatantId,
        kind: &CommandKind,
        target: Option<CombatantId>,
        events: &mut Vec<BattleEvent>,
    ) -> Result<(), CommandError> {
        if *kind == CommandKind::Defend {
            let combatant = self.combatant_mut(actor);
            combatant.defending = true;
            combatant.reset_turn();
            events.push(BattleEvent::Defending { combatant: actor });
            return Ok(());
        }

        let (action, item) = self.resolve_command(actor, kind)?;
        let def = &self.actions[&action];
        self.check_usable(actor, def)?;
        self.check_target(actor, def, target)?;

        if let Some(item) = &item
            && let Some(count) = self.inventory.get_mut(item)
        {
            *count -= 1;
        }
        self.combatant_mut(actor).state = TurnState::Queued;
        events.push(BattleEvent::ActionQueued {
            actor,
            action: action.clone(),
            item: item.clone(),
        });
        self.queue.push_back(QueuedAction {
            actor,
            action,
            item,
            target,
        });
        Ok(())
    }

    /// The action a command runs, and the item it consumes.
    pub(crate) fn resolve_command(
        &self,
        actor: CombatantId,
        kind: &CommandKind,
    ) -> Result<(ActionId, Option<ItemId>), CommandError> {
        let combatant = &self.combatants[usize::from(actor.0)];
        match kind {
            CommandKind::Attack => {
                let action = match &combatant.limit {
                    Some(limit) if combatant.has_full_limit() => limit.clone(),
                    _ => combatant.attack.clone(),
                };
                Ok((action, None))
            }
            CommandKind::Action { id } => {
                if combatant.actions.contains(id) {
                    Ok((id.clone(), None))
                } else {
                    Err(CommandError::UnknownAction {
                        actor,
                        action: id.clone(),
                    })
                }
            }
            CommandKind::Item { id } => {
                let in_stock = self.inventory.get(id).copied().unwrap_or(0) > 0;
                match self.items.get(id) {
                    Some(item) if in_stock => Ok((item.action.clone(), Some(id.clone()))),
                    _ => Err(CommandError::OutOfItem(id.clone())),
                }
            }
            CommandKind::Defend => unreachable!("Defend never reaches the queue"),
        }
    }

    /// Silence and MP checks.
    pub(crate) fn check_usable(
        &self,
        actor: CombatantId,
        def: &ActionDef,
    ) -> Result<(), CommandError> {
        let combatant = &self.combatants[usize::from(actor.0)];
        if def.category == ActionCategory::Magic && combatant.has_status(StatusKind::Silence) {
            return Err(CommandError::Silenced(actor));
        }
        if combatant.mp < def.mp_cost {
            return Err(CommandError::NotEnoughMp {
                actor,
                needed: def.mp_cost,
                available: combatant.mp,
            });
        }
        Ok(())
    }

    fn check_target(
        &self,
        actor: CombatantId,
        def: &ActionDef,
        target: Option<CombatantId>,
    ) -> Result<(), CommandError> {
        match (def.target.needs_target(), target) {
            (true, None) => Err(CommandError::MissingTarget),
            (false, Some(_)) => Err(CommandError::UnexpectedTarget),
            (false, None) => Ok(()),
            (true, Some(target)) => {
                if self.is_eligible(actor, def, target) {
                    Ok(())
                } else {
                    Err(CommandError::InvalidTarget(target))
                }
            }
        }
    }

    /// Whether `target` can receive `def` from `actor` right now.
    pub(crate) fn is_eligible(
        &self,
        actor: CombatantId,
        def: &ActionDef,
        target: CombatantId,
    ) -> bool {
        let Some(target) = self.combatant(target) else {
            return false;
        };
        let actor_side = self.combatants[usize::from(actor.0)].side;
        let side = if def.target.on_own_side() {
            actor_side
        } else {
            actor_side.opponent()
        };
        target.side == side && target.is_knocked_out() == def.effect.targets_knocked_out()
    }

    /// Final targets of an action about to execute. A single target that is
    /// no longer eligible is replaced by a random eligible one on the same side.
    fn resolve_targets(
        &mut self,
        actor: CombatantId,
        def: &ActionDef,
        requested: Option<CombatantId>,
    ) -> Vec<CombatantId> {
        let eligible: Vec<CombatantId> = self
            .combatants
            .iter()
            .map(|c| c.id)
            .filter(|&id| self.is_eligible(actor, def, id))
            .collect();

        match def.target {
            TargetRule::AllEnemies | TargetRule::AllAllies => eligible,
            TargetRule::User => eligible.into_iter().filter(|&id| id == actor).collect(),
            TargetRule::SingleEnemy | TargetRule::SingleAlly => match requested {
                Some(id) if eligible.contains(&id) => vec![id],
                _ if eligible.is_empty() => Vec::new(),
                _ => {
                    let index = self.rng.below(eligible.len() as u64);
                    vec![eligible[usize::try_from(index).unwrap_or(0)]]
                }
            },
        }
    }

    fn tick_statuses(&mut self, events: &mut Vec<BattleEvent>) {
        for index in 0..self.combatants.len() {
            let combatant = &mut self.combatants[index];
            if combatant.is_knocked_out() {
                continue;
            }

            let mut poison = false;
            let mut regen = false;
            let mut expired = Vec::new();
            for status in &mut combatant.statuses {
                if matches!(status.kind, StatusKind::Poison | StatusKind::Regen) {
                    status.pulse_ticks += 1;
                    if status.pulse_ticks >= crate::status::PULSE_INTERVAL_TICKS {
                        status.pulse_ticks = 0;
                        poison |= status.kind == StatusKind::Poison;
                        regen |= status.kind == StatusKind::Regen;
                    }
                }
                if let Some(remaining) = &mut status.remaining_ticks {
                    *remaining = remaining.saturating_sub(1);
                    if *remaining == 0 {
                        expired.push(status.kind);
                    }
                }
            }
            combatant
                .statuses
                .retain(|status| status.remaining_ticks != Some(0));

            let id = combatant.id;
            for status in expired {
                events.push(BattleEvent::StatusRemoved {
                    target: id,
                    status,
                    reason: StatusEndReason::Expired,
                });
            }
            if poison {
                self.pulse(id, true, events);
            }
            if regen {
                self.pulse(id, false, events);
            }
        }
    }

    /// One Poison (damage) or Regen (healing) pulse: 5% of max HP, at least 1.
    fn pulse(&mut self, id: CombatantId, poison: bool, events: &mut Vec<BattleEvent>) {
        let combatant = self.combatant_mut(id);
        if combatant.is_knocked_out() {
            return;
        }
        let amount = (combatant.stats.max_hp * crate::status::PULSE_PERCENT / 100).max(1);
        if poison {
            let lost = amount.min(combatant.hp);
            combatant.hp -= lost;
            events.push(BattleEvent::PoisonDamaged {
                target: id,
                amount: lost,
            });
            if combatant.hp == 0 {
                self.knock_out(id, events);
            }
        } else {
            let restored = amount.min(combatant.stats.max_hp - combatant.hp);
            combatant.hp += restored;
            events.push(BattleEvent::Regenerated {
                target: id,
                amount: restored,
            });
        }
    }

    fn fill_gauges(&mut self, events: &mut Vec<BattleEvent>) {
        for index in 0..self.combatants.len() {
            let combatant = &mut self.combatants[index];
            if combatant.is_knocked_out() || combatant.state != TurnState::Charging {
                continue;
            }

            let speed = if combatant.has_status(StatusKind::Sleep) {
                Speed::Stopped
            } else if combatant.has_status(StatusKind::Haste) {
                Speed::Haste
            } else if combatant.has_status(StatusKind::Slow) {
                Speed::Slow
            } else {
                Speed::Normal
            };
            let gain = gain_per_tick(combatant.stats.attributes.dexterity, speed);
            combatant.atb = (combatant.atb + gain).min(ATB_FULL);
            if combatant.atb < ATB_FULL {
                continue;
            }

            let id = combatant.id;
            if combatant.remove_status(StatusKind::Distracted) {
                events.push(BattleEvent::StatusRemoved {
                    target: id,
                    status: StatusKind::Distracted,
                    reason: StatusEndReason::GaugeFull,
                });
            }
            combatant.state = TurnState::AwaitingCommand;
            let player_controlled = combatant.player_controlled;
            events.push(BattleEvent::TurnReady { combatant: id });

            if !player_controlled {
                let (kind, target) = self.decide(id);
                if self.perform(id, &kind, target, events).is_err() {
                    // The AI only proposes validated choices; if one still
                    // fails, defending is always legal and keeps the battle moving.
                    let _ = self.perform(id, &CommandKind::Defend, None, events);
                }
            }
        }
    }

    fn run_queue(&mut self, events: &mut Vec<BattleEvent>) {
        if self.busy_ticks > 0 {
            self.busy_ticks -= 1;
        }
        while self.busy_ticks == 0 && !self.side_defeated() {
            let Some(queued) = self.queue.pop_front() else {
                break;
            };
            self.execute(queued, events);
        }
    }

    fn execute(&mut self, queued: QueuedAction, events: &mut Vec<BattleEvent>) {
        let QueuedAction {
            actor,
            action,
            item,
            target,
        } = queued;
        let def = self.actions[&action].clone();

        if self
            .combatant(actor)
            .is_some_and(|c| c.has_status(StatusKind::Sleep))
        {
            self.cancel(actor, action, item, CancelReason::ActorAsleep, events);
            return;
        }

        let failure = match self.check_usable(actor, &def) {
            Err(CommandError::Silenced(_)) => Some(FailReason::Silenced),
            Err(_) => Some(FailReason::NotEnoughMp),
            Ok(()) => None,
        };
        let targets = if failure.is_none() {
            self.resolve_targets(actor, &def, target)
        } else {
            Vec::new()
        };
        let failure = failure.or_else(|| targets.is_empty().then_some(FailReason::NoValidTarget));

        let combatant = self.combatant_mut(actor);
        combatant.reset_turn();
        combatant.defending = false;

        if let Some(reason) = failure {
            self.refund(item.as_ref());
            events.push(BattleEvent::ActionFailed {
                actor,
                action,
                item,
                reason,
            });
            return;
        }

        combatant.mp -= def.mp_cost;
        if def.category == ActionCategory::Limit {
            combatant.limit_gauge = 0;
        }
        events.push(BattleEvent::ActionStarted {
            actor,
            action,
            item,
            targets: targets.clone(),
            mp_spent: def.mp_cost,
        });
        for target in targets {
            self.apply_effect(actor, target, &def.effect, events);
        }
        self.busy_ticks = def.duration_ticks;
    }

    fn cancel(
        &mut self,
        actor: CombatantId,
        action: ActionId,
        item: Option<ItemId>,
        reason: CancelReason,
        events: &mut Vec<BattleEvent>,
    ) {
        self.refund(item.as_ref());
        events.push(BattleEvent::ActionCancelled {
            actor,
            action,
            item,
            reason,
        });
    }

    fn refund(&mut self, item: Option<&ItemId>) {
        if let Some(item) = item {
            *self.inventory.entry(item.clone()).or_insert(0) += 1;
        }
    }

    /// Drops every queued action of `actor` (knocked out or asleep).
    pub(crate) fn cancel_queued(
        &mut self,
        actor: CombatantId,
        reason: CancelReason,
        events: &mut Vec<BattleEvent>,
    ) {
        let (dropped, kept): (Vec<_>, Vec<_>) = self
            .queue
            .drain(..)
            .partition(|queued| queued.actor == actor);
        self.queue = kept.into();
        for queued in dropped {
            self.cancel(queued.actor, queued.action, queued.item, reason, events);
        }
    }

    pub(crate) fn knock_out(&mut self, id: CombatantId, events: &mut Vec<BattleEvent>) {
        let combatant = self.combatant_mut(id);
        combatant.hp = 0;
        combatant.statuses.clear();
        combatant.defending = false;
        combatant.reset_turn();
        events.push(BattleEvent::KnockedOut { combatant: id });
        self.cancel_queued(id, CancelReason::ActorKnockedOut, events);
    }

    fn side_defeated(&self) -> bool {
        [Side::Party, Side::Enemies].into_iter().any(|side| {
            self.combatants
                .iter()
                .filter(|c| c.side == side)
                .all(Combatant::is_knocked_out)
        })
    }

    /// Ends the battle if a side is out. Party defeat wins a tie.
    fn settle(&mut self, events: &mut Vec<BattleEvent>) -> bool {
        if self.outcome.is_some() {
            return true;
        }
        let side_out = |side: Side| {
            self.combatants
                .iter()
                .filter(|c| c.side == side)
                .all(Combatant::is_knocked_out)
        };
        let outcome = if side_out(Side::Party) {
            Outcome::Defeat
        } else if side_out(Side::Enemies) {
            Outcome::Victory
        } else {
            return false;
        };
        self.outcome = Some(outcome);
        events.push(BattleEvent::BattleEnded { outcome });
        true
    }

    pub(crate) fn combatant_mut(&mut self, id: CombatantId) -> &mut Combatant {
        &mut self.combatants[usize::from(id.0)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Effect;
    use crate::test_support::{catalog_setup, fighter, run_until, until_turn};

    fn player_battle(seed: u64) -> Battle {
        let setup = catalog_setup(vec![fighter("hero")], vec![fighter("wolf")]);
        Battle::new(setup, seed, PartyControl::Player).unwrap()
    }

    const HERO: CombatantId = CombatantId(0);
    const WOLF: CombatantId = CombatantId(1);

    /// Keeps an enemy from acting during a test.
    fn put_to_sleep(battle: &mut Battle, id: CombatantId) {
        battle
            .combatant_mut(id)
            .statuses
            .push(ActiveStatus::new(StatusKind::Sleep, Some(10_000)));
    }

    #[test]
    fn ai_battles_are_deterministic_and_finish() {
        let run = |seed| {
            let setup = catalog_setup(
                vec![fighter("hero"), fighter("mage")],
                vec![fighter("wolf"), fighter("wolf")],
            );
            let mut battle = Battle::new(setup, seed, PartyControl::Ai).unwrap();
            let mut log = Vec::new();
            for _ in 0..5000 {
                log.extend(battle.step());
                if battle.outcome().is_some() {
                    break;
                }
            }
            (battle.outcome(), log)
        };

        let (outcome, first) = run(42);
        let (_, second) = run(42);

        assert!(outcome.is_some(), "battle did not finish");
        assert_eq!(first, second);
    }

    #[test]
    fn a_full_gauge_waits_for_the_player() {
        let mut battle = player_battle(1);

        until_turn(&mut battle, HERO);

        assert_eq!(battle.awaiting_command().collect::<Vec<_>>(), vec![HERO]);
        assert_eq!(battle.combatant(HERO).unwrap().atb(), ATB_FULL);
    }

    #[test]
    fn an_attack_is_queued_then_executed() {
        let mut battle = player_battle(1);
        put_to_sleep(&mut battle, WOLF);
        until_turn(&mut battle, HERO);

        let queued = battle
            .submit(&Command {
                actor: HERO,
                kind: CommandKind::Attack,
                target: Some(WOLF),
            })
            .unwrap();
        let events = run_until(&mut battle, |e| matches!(e, BattleEvent::Damaged { .. }));

        assert!(matches!(queued[..], [BattleEvent::ActionQueued { .. }]));
        assert!(events.iter().any(|e| matches!(
            e,
            BattleEvent::ActionStarted { actor: HERO, targets, .. } if targets == &[WOLF]
        )));
        assert!(battle.combatant(WOLF).unwrap().hp() < 300);
    }

    #[test]
    fn commands_are_refused_outside_the_players_turn() {
        let mut battle = player_battle(1);
        let attack = |actor| Command {
            actor,
            kind: CommandKind::Attack,
            target: Some(WOLF),
        };

        assert_eq!(
            battle.submit(&attack(HERO)),
            Err(CommandError::NotAwaitingCommand(HERO))
        );
        assert_eq!(
            battle.submit(&attack(WOLF)),
            Err(CommandError::NotPlayerControlled(WOLF))
        );
    }

    #[test]
    fn targets_must_match_the_action() {
        let mut battle = player_battle(1);
        until_turn(&mut battle, HERO);

        let at_self = battle.submit(&Command {
            actor: HERO,
            kind: CommandKind::Attack,
            target: Some(HERO),
        });
        let without_target = battle.submit(&Command {
            actor: HERO,
            kind: CommandKind::Attack,
            target: None,
        });

        assert_eq!(at_self, Err(CommandError::InvalidTarget(HERO)));
        assert_eq!(without_target, Err(CommandError::MissingTarget));
    }

    #[test]
    fn spells_need_mp_and_no_silence() {
        let mut battle = player_battle(1);
        until_turn(&mut battle, HERO);
        let fire = Command {
            actor: HERO,
            kind: CommandKind::Action {
                id: ActionId::new("fire"),
            },
            target: Some(WOLF),
        };

        battle.combatant_mut(HERO).mp = 1;
        assert!(matches!(
            battle.submit(&fire),
            Err(CommandError::NotEnoughMp { needed: 4, .. })
        ));

        battle.combatant_mut(HERO).mp = 50;
        battle
            .combatant_mut(HERO)
            .statuses
            .push(ActiveStatus::new(StatusKind::Silence, Some(100)));
        assert_eq!(battle.submit(&fire), Err(CommandError::Silenced(HERO)));
    }

    #[test]
    fn items_are_reserved_when_queued() {
        let mut battle = player_battle(1);
        until_turn(&mut battle, HERO);
        let potion = ItemId::new("potion");

        battle
            .submit(&Command {
                actor: HERO,
                kind: CommandKind::Item { id: potion.clone() },
                target: Some(HERO),
            })
            .unwrap();

        assert_eq!(battle.inventory()[&potion], 1);
    }

    #[test]
    fn a_knockout_cancels_queued_actions_and_returns_items() {
        let mut battle = player_battle(1);
        until_turn(&mut battle, HERO);
        let potion = ItemId::new("potion");
        battle.busy_ticks = 50; // keep the action waiting in the queue
        battle
            .submit(&Command {
                actor: HERO,
                kind: CommandKind::Item { id: potion.clone() },
                target: Some(HERO),
            })
            .unwrap();

        let mut events = Vec::new();
        battle.knock_out(HERO, &mut events);

        assert!(battle.queue.is_empty());
        assert_eq!(battle.inventory()[&potion], 2);
        assert!(events.iter().any(|e| matches!(
            e,
            BattleEvent::ActionCancelled {
                reason: CancelReason::ActorKnockedOut,
                ..
            }
        )));
    }

    #[test]
    fn defend_spends_the_turn_at_once() {
        let mut battle = player_battle(1);
        until_turn(&mut battle, HERO);

        let events = battle
            .submit(&Command {
                actor: HERO,
                kind: CommandKind::Defend,
                target: None,
            })
            .unwrap();

        let hero = battle.combatant(HERO).unwrap();
        assert_eq!(events, vec![BattleEvent::Defending { combatant: HERO }]);
        assert!(hero.is_defending());
        assert_eq!(hero.atb(), 0);
        assert_eq!(hero.state(), TurnState::Charging);
    }

    #[test]
    fn enemy_ambush_turns_the_party_away_until_its_gauges_fill() {
        let mut setup = catalog_setup(vec![fighter("hero")], vec![fighter("wolf")]);
        setup.encounter.opening = Opening::EnemyAmbush;
        let mut battle = Battle::new(setup, 1, PartyControl::Player).unwrap();

        assert!(
            battle
                .combatant(HERO)
                .unwrap()
                .has_status(StatusKind::Distracted)
        );
        assert_eq!(battle.combatant(HERO).unwrap().atb(), 0);
        assert_eq!(battle.combatant(WOLF).unwrap().atb(), ATB_FULL);

        // The wolf acts at once; its first hit lands on a Distracted hero.
        let events = run_until(&mut battle, |e| matches!(e, BattleEvent::Damaged { .. }));
        assert!(events.iter().any(|e| matches!(
            e,
            BattleEvent::Damaged {
                target: HERO,
                distracted: true,
                ..
            }
        )));
        assert!(
            !battle
                .combatant(HERO)
                .unwrap()
                .has_status(StatusKind::Distracted)
        );
    }

    #[test]
    fn distracted_ends_when_the_own_gauge_fills() {
        let mut battle = player_battle(1);
        put_to_sleep(&mut battle, WOLF);
        battle
            .combatant_mut(HERO)
            .statuses
            .push(ActiveStatus::new(StatusKind::Distracted, None));

        let events = until_turn(&mut battle, HERO);

        assert!(events.contains(&BattleEvent::StatusRemoved {
            target: HERO,
            status: StatusKind::Distracted,
            reason: StatusEndReason::GaugeFull,
        }));
    }

    #[test]
    fn poison_takes_5_percent_every_3_seconds() {
        let mut battle = player_battle(1);
        battle
            .combatant_mut(HERO)
            .statuses
            .push(ActiveStatus::new(StatusKind::Poison, Some(1000)));

        let events = run_until(&mut battle, |e| {
            matches!(e, BattleEvent::PoisonDamaged { .. })
        });

        assert_eq!(battle.tick(), 30);
        assert!(events.contains(&BattleEvent::PoisonDamaged {
            target: HERO,
            amount: 15
        }));
    }

    #[test]
    fn sleep_stops_the_gauge() {
        let mut battle = player_battle(1);
        battle
            .combatant_mut(HERO)
            .statuses
            .push(ActiveStatus::new(StatusKind::Sleep, Some(1000)));
        let before = battle.combatant(HERO).unwrap().atb();

        battle.step();

        assert_eq!(battle.combatant(HERO).unwrap().atb(), before);
    }

    #[test]
    fn a_full_limit_gauge_turns_attack_into_the_limit() {
        let mut battle = player_battle(1);
        battle.combatant_mut(HERO).limit_gauge = ATB_FULL;
        until_turn(&mut battle, HERO);

        let events = battle
            .submit(&Command {
                actor: HERO,
                kind: CommandKind::Attack,
                target: Some(WOLF),
            })
            .unwrap();

        assert!(matches!(
            &events[..],
            [BattleEvent::ActionQueued { action, .. }] if action.as_str() == "twin-cleave"
        ));
    }

    #[test]
    fn a_target_knocked_out_before_execution_is_replaced() {
        let setup = catalog_setup(
            vec![fighter("hero")],
            vec![fighter("wolf"), fighter("wolf")],
        );
        let mut battle = Battle::new(setup, 3, PartyControl::Player).unwrap();
        until_turn(&mut battle, HERO);
        battle.busy_ticks = 2;
        battle
            .submit(&Command {
                actor: HERO,
                kind: CommandKind::Attack,
                target: Some(WOLF),
            })
            .unwrap();
        let mut ignored = Vec::new();
        battle.knock_out(WOLF, &mut ignored);

        let events = run_until(&mut battle, |e| {
            matches!(e, BattleEvent::ActionStarted { actor: HERO, .. })
        });

        assert!(events.iter().any(|e| matches!(
            e,
            BattleEvent::ActionStarted { actor: HERO, targets, .. } if targets == &[CombatantId(2)]
        )));
    }

    #[test]
    fn revive_only_targets_knocked_out_allies() {
        let setup = catalog_setup(
            vec![fighter("hero"), fighter("mage")],
            vec![fighter("wolf")],
        );
        let mut battle = Battle::new(setup, 1, PartyControl::Player).unwrap();
        let feather = battle.actions[&ActionId::new("revive-feather")].clone();
        assert!(matches!(feather.effect, Effect::Revive { .. }));

        assert!(!battle.is_eligible(HERO, &feather, CombatantId(1)));
        let mut ignored = Vec::new();
        battle.knock_out(CombatantId(1), &mut ignored);
        assert!(battle.is_eligible(HERO, &feather, CombatantId(1)));
    }
}
