//! Applying an action's [`Effect`] to one target.
//!
//! Random rolls happen here; the arithmetic is in [`crate::damage`]. The order
//! of damage modifiers follows `PLAN.md` §3: element and critical are part of
//! the damage, Defend and the barriers halve it, Distracted multiplies the
//! result last, then the 9999 cap applies.

use crate::action::{Affinity, Effect, Element};
use crate::battle::Battle;
use crate::damage::{
    self, CRITICAL_BP, DISTRACTED_BP, HALF_BP, MAX_DAMAGE, WEAK_BP, after_defense, critical_chance,
    heal, hit_chance, magical_base, physical_base, status_chance, with_variance,
};
use crate::event::{BattleEvent, CancelReason, StatusEndReason};
use crate::ids::CombatantId;
use crate::status::{ActiveStatus, StatusKind};
use utils::numbers::saturating_u16;

/// Everything that changes a hit after the raw formula.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Modifiers {
    pub(crate) affinity: Option<Affinity>,
    /// Critical multiplier in basis points, when the hit is critical.
    pub(crate) critical_bp: Option<u64>,
    pub(crate) defending: bool,
    pub(crate) barrier: bool,
    pub(crate) distracted: bool,
}

/// Applies the modifiers to a raw amount: element, critical, Defend, barrier,
/// Distracted (last), then the cap. Anything not immune deals at least 1.
pub(crate) fn finish(raw: u64, m: Modifiers) -> u64 {
    let mut amount = match m.affinity {
        Some(Affinity::Immune) => return 0,
        Some(Affinity::Weak) => damage::scale(raw, WEAK_BP),
        Some(Affinity::Resist) => damage::scale(raw, HALF_BP),
        None => raw,
    };
    if let Some(bp) = m.critical_bp {
        amount = damage::scale(amount, bp);
    }
    if m.defending {
        amount = damage::scale(amount, HALF_BP);
    }
    if m.barrier {
        amount = damage::scale(amount, HALF_BP);
    }
    if m.distracted {
        amount = damage::scale(amount, DISTRACTED_BP);
    }
    amount.clamp(1, MAX_DAMAGE)
}

#[derive(Clone, Copy)]
enum DamageKind {
    Physical,
    Magical,
}

impl Battle {
    pub(crate) fn apply_effect(
        &mut self,
        actor: CombatantId,
        target: CombatantId,
        effect: &Effect,
        events: &mut Vec<BattleEvent>,
    ) {
        match effect {
            Effect::PhysicalDamage { power, element } => {
                self.strike(
                    actor,
                    target,
                    DamageKind::Physical,
                    *power,
                    *element,
                    events,
                );
            }
            Effect::MagicalDamage { power, element } => {
                self.strike(actor, target, DamageKind::Magical, *power, *element, events);
            }
            Effect::PercentCurrentHp { percent, element } => {
                let defender = &self.combatants[usize::from(target.0)];
                let raw = (u64::from(defender.hp) * u64::from(*percent) / 100).max(1);
                let modifiers = Modifiers {
                    affinity: element.and_then(|e| defender.affinities.get(&e).copied()),
                    distracted: defender.has_status(StatusKind::Distracted),
                    ..Modifiers::default()
                };
                self.take_damage(target, finish(raw, modifiers), false, modifiers, events);
            }
            Effect::Heal { power } => {
                let healer = &self.combatants[usize::from(actor.0)].stats;
                let amount = heal(*power, healer.level, healer.attributes.magic);
                self.restore_hp(target, amount, events);
            }
            Effect::RestoreHp { amount } => self.restore_hp(target, u64::from(*amount), events),
            Effect::RestoreMp { amount } => {
                let combatant = self.combatant_mut(target);
                let restored = (*amount).min(combatant.stats.max_mp - combatant.mp);
                combatant.mp += restored;
                events.push(BattleEvent::MpRestored {
                    target,
                    amount: restored,
                });
            }
            Effect::Revive { hp_percent } => {
                let combatant = self.combatant_mut(target);
                let hp = u32::from(combatant.stats.max_hp) * u32::from(*hp_percent) / 100;
                combatant.hp = saturating_u16(u64::from(hp)).max(1);
                combatant.reset_turn();
                events.push(BattleEvent::Revived {
                    target,
                    hp: combatant.hp,
                });
            }
            Effect::ApplyStatus {
                status,
                duration_ticks,
                accuracy,
            } => self.inflict(actor, target, *status, *duration_ticks, *accuracy, events),
            Effect::RemoveStatus { statuses } => {
                for &status in statuses {
                    if self.combatant_mut(target).remove_status(status) {
                        events.push(BattleEvent::StatusRemoved {
                            target,
                            status,
                            reason: StatusEndReason::Cured,
                        });
                    }
                }
            }
            Effect::Devour => {
                let hp = self.combatants[usize::from(target.0)].hp;
                self.knock_out(target, events);
                self.restore_hp(actor, u64::from(hp), events);
            }
            Effect::Steal => self.steal(actor, target, events),
            Effect::Reanimate { into } => {
                // Checked against the forms when the battle was set up.
                let Some(form) = self.forms.get(into).cloned() else {
                    return;
                };
                let side = self.combatants[usize::from(target.0)].side;
                let raised = crate::combatant::Combatant::new(target, side, form, false);
                let hp = raised.hp;
                self.combatants[usize::from(target.0)] = raised;
                events.push(BattleEvent::Revived { target, hp });
            }
        }
    }

    /// Takes one random item from the party's bag into the thief's hands
    /// (only enemies steal: the bag is the party's).
    fn steal(&mut self, actor: CombatantId, target: CombatantId, events: &mut Vec<BattleEvent>) {
        let thief = &self.combatants[usize::from(actor.0)];
        let victim = &self.combatants[usize::from(target.0)];
        let chance = damage::steal_chance(
            thief.stats.attributes.dexterity,
            thief.stats.level,
            victim.stats.attributes.dexterity,
            victim.stats.level,
        );
        let held: Vec<_> = self
            .inventory
            .iter()
            .filter(|&(_, &count)| count > 0)
            .map(|(item, _)| item.clone())
            .collect();
        let robbing = thief.side == crate::ids::Side::Enemies;
        if !robbing || held.is_empty() || !self.rng.chance_percent(chance) {
            events.push(BattleEvent::Missed { target });
            return;
        }
        let kinds = u64::try_from(held.len()).unwrap_or(u64::MAX);
        let index = usize::try_from(self.rng.below(kinds)).unwrap_or(0);
        let Some(item) = held.get(index).cloned() else {
            return;
        };
        if let Some(count) = self.inventory.get_mut(&item) {
            *count -= 1;
        }
        self.combatant_mut(actor).stolen.push(item.clone());
        events.push(BattleEvent::Stole {
            actor,
            target,
            item,
        });
    }

    fn strike(
        &mut self,
        actor: CombatantId,
        target: CombatantId,
        kind: DamageKind,
        power: u16,
        element: Option<Element>,
        events: &mut Vec<BattleEvent>,
    ) {
        let attacker = self.combatants[usize::from(actor.0)].stats;
        let defender = &self.combatants[usize::from(target.0)];
        let defender_level = defender.stats.level;

        let (raw, barrier) = match kind {
            DamageKind::Physical => (
                after_defense(
                    power,
                    defender.stats.defense(),
                    physical_base(attacker.attack(), attacker.level),
                ),
                defender.has_status(StatusKind::Barrier),
            ),
            DamageKind::Magical => (
                after_defense(
                    power,
                    defender.stats.magic_defense(),
                    magical_base(attacker.magic_attack(), attacker.level),
                ),
                defender.has_status(StatusKind::MagicBarrier),
            ),
        };
        let mut modifiers = Modifiers {
            affinity: element.and_then(|e| defender.affinities.get(&e).copied()),
            critical_bp: None,
            defending: defender.defending,
            barrier,
            distracted: defender.has_status(StatusKind::Distracted),
        };

        if matches!(kind, DamageKind::Physical) {
            // Turned away or asleep: no chance to evade.
            let can_evade = !modifiers.distracted && !defender.has_status(StatusKind::Sleep);
            let chance = hit_chance(
                attacker.equipment.weapon_accuracy,
                attacker.attributes.dexterity,
                defender.stats.attributes.dexterity,
                defender.stats.equipment.evasion,
            );
            if can_evade && !self.rng.chance_percent(chance) {
                events.push(BattleEvent::Missed { target });
                return;
            }
        }

        let raw = with_variance(raw, self.rng.below(256));
        if matches!(kind, DamageKind::Physical) {
            let chance = critical_chance(
                attacker.attributes.luck,
                attacker.level,
                defender_level,
                attacker.equipment.critical_chance,
            );
            if self.rng.chance_percent(chance) {
                modifiers.critical_bp =
                    Some(CRITICAL_BP + u64::from(attacker.equipment.critical_damage));
            }
        }

        let amount = finish(raw, modifiers);
        self.take_damage(
            target,
            amount,
            modifiers.critical_bp.is_some(),
            modifiers,
            events,
        );
    }

    /// HP loss from an action, and everything a hit triggers: Distracted and
    /// Sleep end, a knockout at 0 HP.
    fn take_damage(
        &mut self,
        target: CombatantId,
        amount: u64,
        critical: bool,
        modifiers: Modifiers,
        events: &mut Vec<BattleEvent>,
    ) {
        let combatant = self.combatant_mut(target);
        let lost = saturating_u16(amount).min(combatant.hp);
        combatant.hp -= lost;
        events.push(BattleEvent::Damaged {
            target,
            amount: lost,
            critical,
            distracted: modifiers.distracted && amount > 0,
            affinity: modifiers.affinity,
        });
        if amount == 0 {
            return;
        }

        for status in [StatusKind::Distracted, StatusKind::Sleep] {
            if combatant.remove_status(status) {
                events.push(BattleEvent::StatusRemoved {
                    target,
                    status,
                    reason: StatusEndReason::Hit,
                });
            }
        }

        if combatant.hp == 0 {
            self.knock_out(target, events);
        }
    }

    fn restore_hp(&mut self, target: CombatantId, amount: u64, events: &mut Vec<BattleEvent>) {
        let combatant = self.combatant_mut(target);
        let missing = combatant.stats.max_hp - combatant.hp;
        let restored = saturating_u16(amount).min(missing);
        combatant.hp += restored;
        events.push(BattleEvent::Healed {
            target,
            amount: restored,
        });
    }

    /// A status from an action: immunity first, then, against an opponent,
    /// the accuracy roll resisted by Luck. Allies always receive it.
    fn inflict(
        &mut self,
        actor: CombatantId,
        target: CombatantId,
        status: StatusKind,
        duration_ticks: Option<u16>,
        accuracy: u8,
        events: &mut Vec<BattleEvent>,
    ) {
        let defender = &self.combatants[usize::from(target.0)];
        let hostile = defender.side != self.combatants[usize::from(actor.0)].side;
        let immune = defender.status_immunities.contains(&status);
        if hostile && !immune {
            let chance = status_chance(accuracy, defender.stats.attributes.luck);
            if !self.rng.chance_percent(chance) {
                events.push(BattleEvent::StatusResisted { target, status });
                return;
            }
        }
        self.apply_status(target, status, duration_ticks, events);
    }

    pub(crate) fn apply_status(
        &mut self,
        target: CombatantId,
        status: StatusKind,
        duration_ticks: Option<u16>,
        events: &mut Vec<BattleEvent>,
    ) {
        let combatant = self.combatant_mut(target);
        if combatant.status_immunities.contains(&status) {
            events.push(BattleEvent::StatusImmune { target, status });
            return;
        }

        let opposite = match status {
            StatusKind::Haste => Some(StatusKind::Slow),
            StatusKind::Slow => Some(StatusKind::Haste),
            _ => None,
        };
        if let Some(opposite) = opposite
            && combatant.remove_status(opposite)
        {
            events.push(BattleEvent::StatusRemoved {
                target,
                status: opposite,
                reason: StatusEndReason::Replaced,
            });
        }

        // Re-applying refreshes the timer instead of stacking.
        combatant.remove_status(status);
        combatant
            .statuses
            .push(ActiveStatus::new(status, duration_ticks));
        events.push(BattleEvent::StatusApplied { target, status });

        if status == StatusKind::Sleep {
            combatant.reset_turn();
            self.cancel_queued(target, CancelReason::ActorAsleep, events);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::PartyControl;
    use crate::test_support::{catalog_setup, fighter};

    fn battle() -> Battle {
        let setup = catalog_setup(vec![fighter("hero")], vec![fighter("wolf")]);
        Battle::new(setup, 9, PartyControl::Player).unwrap()
    }

    #[test]
    fn modifiers_apply_in_order_and_distracted_multiplies_last() {
        let critical = Modifiers {
            critical_bp: Some(CRITICAL_BP),
            ..Modifiers::default()
        };
        let critical_and_distracted = Modifiers {
            distracted: true,
            ..critical
        };

        assert_eq!(finish(1000, critical), 1300);
        assert_eq!(finish(1000, critical_and_distracted), 1950);
    }

    #[test]
    fn defend_and_barrier_each_halve() {
        let both = Modifiers {
            defending: true,
            barrier: true,
            ..Modifiers::default()
        };

        assert_eq!(finish(1000, both), 250);
    }

    #[test]
    fn affinities_double_halve_or_cancel() {
        let with = |affinity| Modifiers {
            affinity: Some(affinity),
            ..Modifiers::default()
        };

        assert_eq!(finish(1000, with(Affinity::Weak)), 2000);
        assert_eq!(finish(1000, with(Affinity::Resist)), 500);
        assert_eq!(finish(1000, with(Affinity::Immune)), 0);
    }

    #[test]
    fn damage_is_capped_at_9999_and_at_least_1() {
        assert_eq!(finish(50_000, Modifiers::default()), 9999);
        assert_eq!(finish(0, Modifiers::default()), 1);
    }

    #[test]
    fn a_hit_wakes_a_sleeping_target() {
        let mut battle = battle();
        let hero = CombatantId(0);
        let mut events = Vec::new();
        battle.apply_status(hero, StatusKind::Sleep, Some(100), &mut events);

        battle.take_damage(hero, 30, false, Modifiers::default(), &mut events);

        assert!(
            !battle
                .combatant(hero)
                .unwrap()
                .has_status(StatusKind::Sleep)
        );
    }

    #[test]
    fn haste_replaces_slow() {
        let mut battle = battle();
        let hero = CombatantId(0);
        let mut events = Vec::new();

        battle.apply_status(hero, StatusKind::Slow, Some(100), &mut events);
        battle.apply_status(hero, StatusKind::Haste, Some(100), &mut events);

        let statuses: Vec<_> = battle
            .combatant(hero)
            .unwrap()
            .statuses()
            .map(|(kind, _)| kind)
            .collect();
        assert_eq!(statuses, vec![StatusKind::Haste]);
    }

    #[test]
    fn immune_targets_resist_statuses() {
        let mut battle = battle();
        let wolf = CombatantId(1);
        battle
            .combatant_mut(wolf)
            .status_immunities
            .push(StatusKind::Sleep);
        let mut events = Vec::new();

        battle.apply_status(wolf, StatusKind::Sleep, Some(100), &mut events);

        assert_eq!(
            events,
            vec![BattleEvent::StatusImmune {
                target: wolf,
                status: StatusKind::Sleep
            }]
        );
    }

    #[test]
    fn a_distracted_target_cannot_evade() {
        let mut battle = battle();
        let wolf = CombatantId(1);
        battle.combatant_mut(wolf).stats.equipment.evasion = 100;
        battle
            .combatant_mut(wolf)
            .statuses
            .push(ActiveStatus::new(StatusKind::Distracted, None));
        let mut events = Vec::new();

        battle.strike(
            CombatantId(0),
            wolf,
            DamageKind::Physical,
            16,
            None,
            &mut events,
        );

        assert!(matches!(
            events.first(),
            Some(BattleEvent::Damaged {
                distracted: true,
                ..
            })
        ));
    }

    #[test]
    fn evasion_makes_physical_attacks_miss_but_not_spells() {
        let mut battle = battle();
        let wolf = CombatantId(1);
        battle.combatant_mut(wolf).stats.equipment.evasion = 100;
        let mut physical_misses = 0;
        let mut spell_misses = 0;

        for _ in 0..200 {
            let mut events = Vec::new();
            battle.strike(
                CombatantId(0),
                wolf,
                DamageKind::Physical,
                1,
                None,
                &mut events,
            );
            physical_misses += usize::from(matches!(events[..], [BattleEvent::Missed { .. }]));
            events.clear();
            battle.strike(
                CombatantId(0),
                wolf,
                DamageKind::Magical,
                1,
                None,
                &mut events,
            );
            spell_misses += usize::from(matches!(events[..], [BattleEvent::Missed { .. }]));
            battle.combatant_mut(wolf).hp = 300;
        }

        // Evasion 100 leaves the 25% hit floor: about 150 misses out of 200.
        assert!(
            (120..=180).contains(&physical_misses),
            "{physical_misses} misses"
        );
        assert_eq!(spell_misses, 0);
    }

    #[test]
    fn hostile_statuses_can_be_resisted_but_buffs_always_land() {
        let mut battle = battle();
        let (hero, wolf) = (CombatantId(0), CombatantId(1));
        battle.combatant_mut(wolf).stats.attributes.luck = 255; // 5% floor
        let mut events = Vec::new();

        for _ in 0..20 {
            battle.inflict(hero, wolf, StatusKind::Silence, Some(10), 10, &mut events);
        }
        battle.inflict(hero, hero, StatusKind::Haste, Some(100), 1, &mut events);

        let resisted = events
            .iter()
            .filter(|e| matches!(e, BattleEvent::StatusResisted { .. }))
            .count();
        assert!(resisted >= 15, "{resisted} resisted");
        assert!(
            battle
                .combatant(hero)
                .unwrap()
                .has_status(StatusKind::Haste)
        );
    }

    #[test]
    fn gravity_takes_a_share_of_current_hp() {
        let mut battle = battle();
        let wolf = CombatantId(1);
        let mut events = Vec::new();

        battle.apply_effect(
            CombatantId(0),
            wolf,
            &Effect::PercentCurrentHp {
                percent: 25,
                element: Some(Element::Gravity),
            },
            &mut events,
        );

        assert_eq!(battle.combatant(wolf).unwrap().hp(), 225);
    }
}
