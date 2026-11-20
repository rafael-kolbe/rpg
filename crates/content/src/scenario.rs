//! Sandbox scenarios: a battle written out in full (an [`Encounter`]), or one
//! built from the campaign's own heroes and monsters, so a balance change
//! shows in the sandbox at once and the party's level can be chosen per run.

use std::collections::BTreeMap;
use std::path::Path;

use engine::{Encounter, ItemId, Opening};
use serde::Deserialize;
use world::{HeroId, HeroState, MonsterId, World};

use crate::{ContentError, read_json};

/// A test battle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scenario {
    /// Every combatant written in the file.
    Written(Encounter),
    /// Heroes and monsters of the campaign, by id.
    Campaign(CampaignBattle),
}

/// A battle of campaign heroes against campaign monsters (`{"campaign": …}`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CampaignBattle {
    /// The heroes, in order.
    pub party: Vec<Member>,
    /// The monsters, in order.
    pub enemies: Vec<MonsterId>,
    /// Items the party carries.
    #[serde(default)]
    pub inventory: BTreeMap<ItemId, u16>,
    /// How the battle opens.
    #[serde(default)]
    pub opening: Opening,
}

/// A hero of a [`CampaignBattle`].
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Member {
    /// Which hero.
    pub hero: HeroId,
    /// Level, unless the run asks for another.
    pub level: u8,
    /// A guest: fixed level, fights on its own.
    #[serde(default)]
    pub guest: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CampaignFile {
    campaign: CampaignBattle,
}

impl Scenario {
    /// The battle to run. `level`, when given, replaces the level of every
    /// hero who is not a guest (written scenarios ignore it).
    ///
    /// # Errors
    ///
    /// A message when a hero or monster does not exist in the world.
    pub fn encounter(&self, world: &World, level: Option<u8>) -> Result<Encounter, String> {
        let battle = match self {
            Self::Written(encounter) => return Ok(encounter.clone()),
            Self::Campaign(battle) => battle,
        };
        let mut party = Vec::new();
        for member in &battle.party {
            let level = match level {
                Some(level) if !member.guest => level,
                _ => member.level,
            };
            let mut state = HeroState::new(world, &member.hero, level.clamp(1, 99));
            state.guest = member.guest;
            party.push(
                world
                    .hero_setup(&state)
                    .ok_or_else(|| format!("unknown hero `{}`", member.hero))?,
            );
        }
        let enemies = battle
            .enemies
            .iter()
            .map(|id| {
                world
                    .monster_setup(id)
                    .ok_or_else(|| format!("unknown monster `{id}`"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Encounter {
            opening: battle.opening,
            inventory: battle.inventory.clone(),
            forms: world.forms_for(&enemies),
            party,
            enemies,
        })
    }
}

/// Loads a sandbox scenario: a written [`Encounter`], or `{"campaign": …}`.
///
/// # Errors
///
/// [`ContentError`] when the file is missing or malformed.
pub fn load_scenario(path: &Path) -> Result<Scenario, ContentError> {
    let value: serde_json::Value = read_json(path)?;
    let parse = |source| ContentError::Parse {
        path: path.to_owned(),
        source,
    };
    if value.get("campaign").is_some() {
        let file: CampaignFile = serde_json::from_value(value).map_err(parse)?;
        Ok(Scenario::Campaign(file.campaign))
    } else {
        Ok(Scenario::Written(
            serde_json::from_value(value).map_err(parse)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Content;

    #[test]
    fn a_campaign_scenario_takes_the_chosen_level_except_for_guests() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let content = Content::load(&root.join("content")).unwrap();
        let world = content.load_world(&root.join("content")).unwrap();
        let scenario = load_scenario(&root.join("sandbox/ch1-sewer-1.json")).unwrap();

        let as_written = scenario.encounter(&world, None).unwrap();
        let at_twelve = scenario.encounter(&world, Some(12)).unwrap();

        let levels = |encounter: &Encounter| {
            encounter
                .party
                .iter()
                .map(|member| member.stats.level)
                .collect::<Vec<_>>()
        };
        assert_eq!(levels(&as_written), [6, 6, 20, 10]);
        assert_eq!(levels(&at_twelve), [12, 12, 20, 10]);
        assert!(at_twelve.party[2].guest);
    }
}
