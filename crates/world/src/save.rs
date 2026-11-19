//! What the server stores between sessions (`PLAN.md` §4, "Regras do arco"):
//! the account's shared [`Progress`] and one [`Save`] per [`GameMode`].

use std::collections::{BTreeMap, BTreeSet};

use crate::def::Rank;

use engine::{BattleRecord, ItemId, Opening};
use serde::{Deserialize, Serialize};

use crate::World;
use crate::def::StoryEffect;
use crate::ids::{DialogueId, EquipmentId, Flag, HeroId, MonsterId, Position, QuestId};
use crate::progression;

/// Campaign (Story Mode) or Multiplayer (Online Mode).
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameMode {
    /// The single-player story; the only mode that advances [`Progress`].
    Campaign,
    /// Co-op; walks only where the Campaign has been.
    Multiplayer,
}

/// Story progress shared by both modes of an account. Only the Campaign
/// adds to it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Progress {
    /// Content version it was made with ([`crate::def::StartDef::epoch`]).
    #[serde(default)]
    pub epoch: u32,
    /// Story flags set so far.
    pub flags: BTreeSet<Flag>,
    /// Nodes the player has stood on.
    pub explored: BTreeSet<Position>,
}

/// The story flag that opens the Multiplayer; a later arc sets it.
pub const MULTIPLAYER_FLAG: &str = "multiplayer-unlocked";

impl Progress {
    /// No progress, for the world's current content.
    #[must_use]
    pub fn new(world: &World) -> Self {
        Self {
            epoch: world.map.start.epoch,
            ..Self::default()
        }
    }

    /// Whether it was made with the world's current content.
    #[must_use]
    pub fn is_current(&self, world: &World) -> bool {
        self.epoch == world.map.start.epoch
    }

    /// Whether `flag` is set.
    #[must_use]
    pub fn has(&self, flag: &Flag) -> bool {
        self.flags.contains(flag)
    }

    /// Whether the story has opened the Multiplayer.
    #[must_use]
    pub fn multiplayer_unlocked(&self) -> bool {
        self.has(&Flag::new(MULTIPLAYER_FLAG))
    }
}

/// One mode's game: where the player is and what the party carries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Save {
    /// Content version it was made with ([`crate::def::StartDef::epoch`]).
    #[serde(default)]
    pub epoch: u32,
    /// Where the player stands.
    pub position: Position,
    /// Where a defeat sends the party back (the last Temple).
    pub temple: Position,
    /// The heroes, in battle order.
    pub party: Vec<HeroState>,
    /// Heroes who left the party, as they were, for when they come back.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub benched: BTreeMap<HeroId, HeroState>,
    /// Items, by id (never zero).
    pub inventory: BTreeMap<ItemId, u16>,
    /// Weapons and armor not worn, by id (never zero).
    pub bag: BTreeMap<EquipmentId, u16>,
    /// Money.
    pub kalds: u32,
    /// State of the random generator for encounters and drops.
    pub rng: u64,
    /// The battle under way, or a boss's next phase waiting behind the
    /// rewards. Resuming the game resumes it where it was, so leaving the
    /// page neither skips nor restarts a fight.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub battle: Option<SavedBattle>,
    /// Guild rank, once registered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<Rank>,
    /// The campaign's next step (a Fluent message id).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub objective: Option<String>,
    /// Missions under way and their progress.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub quests: BTreeMap<QuestId, u16>,
    /// Missions completed.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub quests_done: BTreeSet<QuestId>,
    /// Scenes waiting to play once the screen is free (a mission's scene
    /// reached during a battle).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub queued: Vec<DialogueId>,
    /// The Campaign's last auto save: where a Game Over sends the player.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint: Option<Box<Checkpoint>>,
}

/// A moment the Campaign can return to: the save and the story as they were.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Checkpoint {
    /// The save (without its own checkpoint).
    pub save: Save,
    /// The story progress.
    pub progress: Progress,
}

/// A battle the game owes the player: enough to rebuild it exactly where it
/// was (the party's HP, MP and items at its start are those of the save,
/// which only change when the battle ends).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SavedBattle {
    /// The enemies.
    pub monsters: Vec<MonsterId>,
    /// Set on victory (a fixed fight).
    pub flag: Option<Flag>,
    /// Story effects of winning.
    pub on_win: Vec<StoryEffect>,
    /// How it opens.
    pub opening: Opening,
    /// Seed of the battle.
    pub seed: u64,
    /// What happened so far.
    #[serde(default)]
    pub record: BattleRecord,
}

/// A hero's growth and condition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HeroState {
    /// Which hero.
    pub hero: HeroId,
    /// Level.
    pub level: u8,
    /// Experience toward the next level.
    pub xp: u32,
    /// Current HP (at least 1 outside battle).
    pub hp: u16,
    /// Current MP.
    pub mp: u16,
    /// Worn weapon.
    pub weapon: Option<EquipmentId>,
    /// Worn armor.
    pub armor: Option<EquipmentId>,
    /// A guest: fights on its own AI; the player does not command it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub guest: bool,
}

impl Save {
    /// Whether it was made with the world's current content.
    #[must_use]
    pub fn is_current(&self, world: &World) -> bool {
        self.epoch == world.map.start.epoch
    }

    /// A new game, from `content/world/start.json`, with `seed` for the
    /// random generator.
    #[must_use]
    pub fn new(world: &World, seed: u64) -> Self {
        let start = &world.map.start;
        let mut save = Self {
            epoch: start.epoch,
            position: start.position.clone(),
            temple: start.position.clone(),
            party: Vec::new(),
            benched: BTreeMap::new(),
            inventory: start.inventory.clone(),
            bag: BTreeMap::new(),
            kalds: start.kalds,
            rng: seed,
            battle: None,
            rank: None,
            objective: None,
            quests: BTreeMap::new(),
            quests_done: BTreeSet::new(),
            queued: Vec::new(),
            checkpoint: None,
        };
        for hero in &start.party {
            save.party.push(HeroState::new(world, hero, 1));
        }
        save
    }
}

impl HeroState {
    /// A hero at `level`, healthy, in its starting gear.
    #[must_use]
    pub fn new(world: &World, hero: &HeroId, level: u8) -> Self {
        let def = world.map.heroes.get(hero);
        let mut state = Self {
            hero: hero.clone(),
            level,
            xp: 0,
            hp: 0,
            mp: 0,
            weapon: def.and_then(|def| def.weapon.clone()),
            armor: def.and_then(|def| def.armor.clone()),
            guest: false,
        };
        if let Some(stats) = world.hero_stats(&state) {
            state.hp = stats.max_hp;
            state.mp = stats.max_mp;
        }
        state
    }

    /// Restores HP and MP to full.
    pub fn heal(&mut self, world: &World) {
        if let Some(stats) = world.hero_stats(self) {
            self.hp = stats.max_hp;
            self.mp = stats.max_mp;
        }
    }

    /// Adds experience; returns the levels gained. HP and MP grow by what
    /// the new levels add.
    pub fn gain_xp(&mut self, world: &World, xp: u32) -> u8 {
        let before = world.hero_stats(self);
        let (level, progress) = progression::add_xp(self.level, self.xp, xp);
        let gained = level - self.level;
        self.level = level;
        self.xp = progress;
        if let (Some(before), Some(after)) = (before, world.hero_stats(self)) {
            self.hp = self
                .hp
                .saturating_add(after.max_hp.saturating_sub(before.max_hp));
            self.mp = self
                .mp
                .saturating_add(after.max_mp.saturating_sub(before.max_mp));
        }
        gained
    }
}
