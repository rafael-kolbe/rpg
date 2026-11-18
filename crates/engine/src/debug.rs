//! Sandbox-only tools to put a battle in a given state (feature `sandbox`).
//!
//! They bypass the rules on purpose, to reach situations that would take long
//! to happen naturally. Production builds do not contain them.

use crate::atb::ATB_FULL;
use crate::battle::Battle;
use crate::combatant::TurnState;
use crate::event::BattleEvent;
use crate::ids::CombatantId;
use crate::status::StatusKind;

impl Battle {
    /// Knocks a combatant out, as if it had taken lethal damage.
    pub fn debug_knock_out(&mut self, id: CombatantId) -> Vec<BattleEvent> {
        let mut events = Vec::new();
        if self.combatant(id).is_some_and(|c| !c.is_knocked_out()) {
            self.knock_out(id, &mut events);
        }
        events
    }

    /// Applies a status directly (no accuracy roll, immunities still hold).
    /// `duration_ticks` is ignored for Distracted.
    pub fn debug_apply_status(
        &mut self,
        id: CombatantId,
        status: StatusKind,
        duration_ticks: u16,
    ) -> Vec<BattleEvent> {
        let mut events = Vec::new();
        if self.combatant(id).is_some_and(|c| !c.is_knocked_out()) {
            let duration = status.needs_duration().then_some(duration_ticks.max(1));
            self.apply_status(id, status, duration, &mut events);
        }
        events
    }

    /// Fills a charging combatant's ATB gauge: its turn comes on the next tick.
    pub fn debug_fill_gauge(&mut self, id: CombatantId) {
        if let Some(index) = self.combatants.iter().position(|c| c.id == id) {
            let combatant = &mut self.combatants[index];
            if !combatant.is_knocked_out() && combatant.state == TurnState::Charging {
                combatant.atb = ATB_FULL;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::battle::{Battle, PartyControl};
    use crate::ids::CombatantId;
    use crate::status::StatusKind;
    use crate::test_support::{catalog_setup, fighter, until_turn};

    #[test]
    fn debug_tools_change_the_battle() {
        let setup = catalog_setup(
            vec![fighter("hero")],
            vec![fighter("wolf"), fighter("wolf")],
        );
        let mut battle = Battle::new(setup, 1, PartyControl::Player).unwrap();
        let (hero, wolf) = (CombatantId(0), CombatantId(1));

        battle.debug_knock_out(wolf);
        battle.debug_apply_status(hero, StatusKind::Haste, 100);
        battle.debug_fill_gauge(hero);
        until_turn(&mut battle, hero);

        assert!(battle.combatant(wolf).unwrap().is_knocked_out());
        assert!(
            battle
                .combatant(hero)
                .unwrap()
                .has_status(StatusKind::Haste)
        );
        assert_eq!(battle.tick(), 1);
    }
}
