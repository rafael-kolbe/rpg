//! Game content and sandbox scenarios, loaded from JSON and checked.
//!
//! Layout of the repository this crate reads:
//!
//! | Path | Contents |
//! | --- | --- |
//! | `content/actions.json` | Every action, by id ([`engine::ActionDef`]) |
//! | `content/items.json` | Every item, by id ([`engine::ItemDef`]) |
//! | `content/world/*.json` | The campaign world ([`world::WorldDef`]): `start`, `regions`, `classes`, `heroes`, `monsters`, `dialogues`, `shops`, `equipment` |
//! | `content/world/instances/<id>.json` | One instance each ([`world::def::InstanceDef`]) |
//! | `sandbox/<scenario>.json` | Test battles ([`engine::Encounter`]) |
//! | `locales/<locale>/*.ftl` | Display names of everything above |
//!
//! [`check`] validates all of it; the `content-check` binary runs it for
//! `make check`.

mod validation;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use engine::{ActionDef, ActionId, BattleSetup, Encounter, ItemDef, ItemId};
use serde::de::DeserializeOwned;
use world::{InstanceId, World, WorldDef};

pub use validation::{Summary, check};

/// Why content could not be loaded.
#[derive(Debug, thiserror::Error)]
pub enum ContentError {
    /// The file could not be read.
    #[error("cannot read {path}: {source}")]
    Read {
        /// The file.
        path: PathBuf,
        /// The I/O error.
        source: std::io::Error,
    },
    /// The file is not valid JSON for its type (unknown fields included).
    #[error("invalid {path}: {source}")]
    Parse {
        /// The file.
        path: PathBuf,
        /// The parse error, with line and column.
        source: serde_json::Error,
    },
}

/// Action and item definitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Content {
    /// Every action, by id.
    pub actions: BTreeMap<ActionId, ActionDef>,
    /// Every item, by id.
    pub items: BTreeMap<ItemId, ItemDef>,
}

impl Content {
    /// Loads `actions.json` and `items.json` from `dir` (the repository's
    /// `content/` directory).
    ///
    /// # Errors
    ///
    /// [`ContentError`] when a file is missing or malformed.
    pub fn load(dir: &Path) -> Result<Self, ContentError> {
        Ok(Self {
            actions: read_json(&dir.join("actions.json"))?,
            items: read_json(&dir.join("items.json"))?,
        })
    }

    /// Pairs an encounter with these definitions, ready for
    /// [`engine::Battle::new`].
    #[must_use]
    pub fn battle_setup(&self, encounter: Encounter) -> BattleSetup {
        BattleSetup {
            encounter,
            actions: self.actions.clone(),
            items: self.items.clone(),
        }
    }
}

impl Content {
    /// Loads the campaign world from `dir/world` (the repository's
    /// `content/` directory), bundled with these definitions.
    ///
    /// # Errors
    ///
    /// [`ContentError`] when a file is missing or malformed.
    pub fn load_world(&self, dir: &Path) -> Result<World, ContentError> {
        let world = dir.join("world");
        let instances_dir = world.join("instances");
        let entries = fs::read_dir(&instances_dir).map_err(|source| ContentError::Read {
            path: instances_dir.clone(),
            source,
        })?;
        let mut instances = BTreeMap::new();
        for entry in entries {
            let path = entry
                .map_err(|source| ContentError::Read {
                    path: instances_dir.clone(),
                    source,
                })?
                .path();
            let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            if path.extension().is_some_and(|ext| ext == "json") {
                instances.insert(InstanceId::new(stem), read_json(&path)?);
            }
        }
        Ok(World {
            map: WorldDef {
                start: read_json(&world.join("start.json"))?,
                regions: read_json(&world.join("regions.json"))?,
                instances,
                classes: read_json(&world.join("classes.json"))?,
                heroes: read_json(&world.join("heroes.json"))?,
                monsters: read_json(&world.join("monsters.json"))?,
                dialogues: read_json(&world.join("dialogues.json"))?,
                shops: read_json(&world.join("shops.json"))?,
                equipment: read_json(&world.join("equipment.json"))?,
            },
            actions: self.actions.clone(),
            items: self.items.clone(),
        })
    }
}

/// Loads a sandbox scenario.
///
/// # Errors
///
/// [`ContentError`] when the file is missing or malformed.
pub fn load_encounter(path: &Path) -> Result<Encounter, ContentError> {
    read_json(path)
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, ContentError> {
    let text = fs::read_to_string(path).map_err(|source| ContentError::Read {
        path: path.to_owned(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| ContentError::Parse {
        path: path.to_owned(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use engine::{Battle, PartyControl};

    use super::*;

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn basic_battle(seed: u64) -> Battle {
        let root = repo_root();
        let content = Content::load(&root.join("content")).unwrap();
        let encounter = load_encounter(&root.join("sandbox/battle-basic.json")).unwrap();
        Battle::new(content.battle_setup(encounter), seed, PartyControl::Ai).unwrap()
    }

    /// Replay test on the real sandbox scenario: same seed, same battle.
    #[test]
    fn the_basic_sandbox_battle_replays_identically_and_ends() {
        let run = |seed| {
            let mut battle = basic_battle(seed);
            let mut log = Vec::new();
            while battle.outcome().is_none() && battle.tick() < 20_000 {
                log.extend(battle.step());
            }
            (battle.outcome(), battle.tick(), log)
        };

        let first = run(42);
        let second = run(42);

        assert!(first.0.is_some(), "battle did not end");
        assert_eq!(first, second);
    }

    #[test]
    fn the_repository_content_passes_the_check() {
        let summary = check(&repo_root()).unwrap_or_else(|problems| {
            panic!("content problems:\n{}", problems.join("\n"));
        });

        assert!(summary.scenarios >= 1);
    }
}
