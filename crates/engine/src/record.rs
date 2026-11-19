//! A battle's record: the player's commands and how far time went. The
//! engine is deterministic, so a record with the battle's setup and seed
//! rebuilds the battle exactly — how a game resumes a battle the player
//! left (a reload, a dropped connection) without storing its whole state.

use serde::{Deserialize, Serialize};

use crate::battle::{Battle, Command, CommandError, PartyControl};
use crate::setup::{BattleSetup, SetupError};

/// Everything the player did in a battle, and its tick. Sandbox debug tools
/// are not recorded: their battles are never resumed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BattleRecord {
    /// Ticks elapsed.
    pub ticks: u32,
    /// Accepted commands, in order.
    pub commands: Vec<RecordedCommand>,
}

/// A command and the tick it came in (after that tick's step).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecordedCommand {
    /// [`Battle::tick`] when it was given.
    pub tick: u32,
    /// The command.
    pub command: Command,
}

/// Why a record could not be replayed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ReplayError {
    /// The setup itself is invalid.
    #[error(transparent)]
    Setup(#[from] SetupError),
    /// The battle went another way than the record (the content changed
    /// since it was made).
    #[error("the record does not fit this battle: {0}")]
    Diverged(#[from] CommandError),
    /// The battle ended before the record did.
    #[error("the battle ended before the record did")]
    EndedEarly,
}

impl Battle {
    /// Rebuilds a battle from its setup, seed and record: the same battle,
    /// at the same tick, as when the record was taken. An empty record
    /// gives [`Battle::new`].
    ///
    /// # Errors
    ///
    /// [`ReplayError`] when the setup is invalid or the record does not
    /// fit it.
    pub fn replay(
        setup: BattleSetup,
        seed: u64,
        party_control: PartyControl,
        record: &BattleRecord,
    ) -> Result<Self, ReplayError> {
        let mut battle = Self::new(setup, seed, party_control)?;
        for entry in &record.commands {
            battle.advance_to(entry.tick)?;
            battle.submit(&entry.command)?;
        }
        battle.advance_to(record.ticks)?;
        Ok(battle)
    }

    fn advance_to(&mut self, tick: u32) -> Result<(), ReplayError> {
        while self.tick < tick {
            if self.outcome.is_some() {
                return Err(ReplayError::EndedEarly);
            }
            self.step();
        }
        Ok(())
    }

    /// The record so far: with the setup and seed, enough to
    /// [`Battle::replay`] this battle.
    #[must_use]
    pub fn record(&self) -> BattleRecord {
        BattleRecord {
            ticks: self.tick,
            commands: self.commands.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::battle::CommandKind;
    use crate::ids::CombatantId;
    use crate::test_support::{catalog_setup, fighter, until_turn};

    const HERO: CombatantId = CombatantId(0);
    const WOLF: CombatantId = CombatantId(1);

    fn setup() -> BattleSetup {
        catalog_setup(vec![fighter("hero")], vec![fighter("wolf")])
    }

    fn attack() -> Command {
        Command {
            actor: HERO,
            kind: CommandKind::Attack,
            target: Some(WOLF),
        }
    }

    #[test]
    fn a_replayed_record_rebuilds_the_same_battle() {
        let mut battle = Battle::new(setup(), 7, PartyControl::Player).unwrap();
        for _ in 0..3 {
            until_turn(&mut battle, HERO);
            battle.submit(&attack()).unwrap();
        }
        for _ in 0..5 {
            battle.step();
        }

        let mut replayed =
            Battle::replay(setup(), 7, PartyControl::Player, &battle.record()).unwrap();

        assert_eq!(replayed.record(), battle.record());
        assert_eq!(
            format!("{:?}", replayed.combatants()),
            format!("{:?}", battle.combatants())
        );
        for _ in 0..200 {
            assert_eq!(replayed.step(), battle.step(), "and goes on the same way");
        }
    }

    #[test]
    fn refused_commands_are_not_recorded() {
        let mut battle = Battle::new(setup(), 7, PartyControl::Player).unwrap();

        assert!(battle.submit(&attack()).is_err());
        assert_eq!(battle.record(), BattleRecord::default());
    }

    #[test]
    fn a_record_that_does_not_fit_is_refused() {
        let record = BattleRecord {
            ticks: 1,
            commands: vec![RecordedCommand {
                tick: 1,
                command: attack(),
            }],
        };

        assert!(matches!(
            Battle::replay(setup(), 7, PartyControl::Player, &record),
            Err(ReplayError::Diverged(_))
        ));
    }
}
