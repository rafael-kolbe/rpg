//! The world as content describes it (`content/world/`): regions,
//! instances, nodes and trails, heroes and their classes, monsters,
//! dialogues, shops and equipment. Read-only at runtime.
//!
//! Display names are Fluent messages derived from ids (`region-<id>`,
//! `instance-<id>`, `hero-<id>`, `class-<id>`, `monster-<id>`,
//! `equipment-<id>`); dialogue lines are `dialogue-<id>-<n>` (from 1) and
//! choices `dialogue-<id>-choice-<n>`.

use std::collections::BTreeMap;

use engine::{ActionId, AiProfile, CombatantSetup, ItemId};
use serde::{Deserialize, Serialize};

use crate::ids::{
    ClassId, DialogueId, EquipmentId, Flag, HeroId, InstanceId, MonsterId, NodeId, Position,
    QuestId, RegionId, ShopId,
};

/// Everything in `content/world/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldDef {
    /// Where a new game starts.
    pub start: StartDef,
    /// World map pieces.
    pub regions: BTreeMap<RegionId, RegionDef>,
    /// Close-up maps.
    pub instances: BTreeMap<InstanceId, InstanceDef>,
    /// Hero classes.
    pub classes: BTreeMap<ClassId, ClassDef>,
    /// Story heroes.
    pub heroes: BTreeMap<HeroId, HeroDef>,
    /// Monster kinds.
    pub monsters: BTreeMap<MonsterId, MonsterDef>,
    /// Dialogue scenes.
    pub dialogues: BTreeMap<DialogueId, DialogueDef>,
    /// Shop stocks.
    pub shops: BTreeMap<ShopId, ShopDef>,
    /// Weapons and armor.
    pub equipment: BTreeMap<EquipmentId, EquipmentDef>,
    /// Missions (`content/world/quests.json`).
    pub quests: BTreeMap<QuestId, QuestDef>,
}

/// A Guild rank: of an adventurer, a mission, a monster or an area
/// (`STORY.md`, Guilda de Aventureiros).
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rank {
    /// Levels 1–15.
    Bronze,
    /// Levels 16–30.
    Silver,
    /// Levels 31–45.
    Gold,
    /// Levels 46–60.
    Platinum,
    /// Levels 61–80.
    Diamond,
    /// Levels 81–99.
    Orichalcum,
    /// Level 100 and above: whoever has slain a legendary monster.
    Dracite,
}

/// A mission (name: `quest-<id>`, e.g. "Kill 20 rats").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct QuestDef {
    /// Lowest Guild rank allowed to take it from a board.
    pub rank: Rank,
    /// What completes it.
    pub goal: QuestGoal,
    /// Money paid when it is completed.
    #[serde(default)]
    pub kalds: u32,
    /// Scenes on the way, played once the progress reaches `at`.
    #[serde(default)]
    pub events: Vec<QuestEvent>,
}

/// What completes a mission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum QuestGoal {
    /// Defeat `count` monsters of these kinds.
    Kill {
        /// The kinds that count.
        monsters: Vec<MonsterId>,
        /// How many.
        count: u16,
    },
}

impl QuestGoal {
    /// Progress needed to complete it.
    #[must_use]
    pub fn count(&self) -> u16 {
        match self {
            Self::Kill { count, .. } => *count,
        }
    }
}

/// A scene a mission plays on the way.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct QuestEvent {
    /// Progress that triggers it.
    pub at: u16,
    /// The scene.
    pub dialogue: DialogueId,
}

/// A new game (`content/world/start.json`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StartDef {
    /// Content version. Raised when a change leaves old saves pointing at
    /// places or story that no longer exist: saves of another version start
    /// over.
    #[serde(default)]
    pub epoch: u32,
    /// Where the player appears; also the first return point after a defeat.
    pub position: Position,
    /// The heroes in the party, in order.
    pub party: Vec<HeroId>,
    /// Starting money.
    pub kalds: u32,
    /// Starting items.
    #[serde(default)]
    pub inventory: BTreeMap<ItemId, u16>,
}

/// A world map piece.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegionDef {
    /// Outline on the world map: an SVG path in a 1000×1000 box.
    pub shape: String,
    /// Where the name is written, `[x, y]` in the same box.
    pub label: [u16; 2],
}

/// The look of an instance's background.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Terrain {
    /// Open fields, farms, a village.
    Fields,
    /// Dense forest.
    Forest,
    /// Inside a city.
    Town,
    /// Underground tunnels and tombs.
    Crypt,
}

/// A close-up map: nodes joined by trails.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InstanceDef {
    /// The world map piece it belongs to.
    pub region: RegionId,
    /// Background style.
    pub terrain: Terrain,
    /// Its points.
    pub nodes: BTreeMap<NodeId, NodeDef>,
    /// Paths between two nodes, usable both ways.
    pub trails: Vec<TrailDef>,
}

/// Flags a node, trail, event or talk needs (`requires`) or must not have
/// (`forbids`) to be available; written as its `when` field.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct Condition {
    /// All of these must be set.
    pub requires: Vec<Flag>,
    /// None of these may be set.
    pub forbids: Vec<Flag>,
}

/// A point of an instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NodeDef {
    /// Position in a 1000×1000 box.
    pub x: u16,
    /// Position in a 1000×1000 box.
    pub y: u16,
    /// Fluent message id of its name, when it has one ("Temple").
    #[serde(default)]
    pub label: Option<String>,
    /// What it is.
    pub kind: NodeKind,
    /// When it exists at all (a hidden camp).
    #[serde(default)]
    pub when: Condition,
    /// A story scene played on arrival while its condition holds.
    #[serde(default)]
    pub event: Option<EventDef>,
}

/// A scene a node plays when the player arrives.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EventDef {
    /// The scene.
    pub dialogue: DialogueId,
    /// When it plays; its own effects normally set a flag it forbids, so it
    /// plays once.
    #[serde(default)]
    pub when: Condition,
}

/// What a node does when the player interacts with it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum NodeKind {
    /// Nothing; a step on the way.
    Path,
    /// Leads to a node of another instance (a road out, stairs).
    Exit {
        /// Where it leads.
        to: Position,
    },
    /// A fixed fight, won once.
    Battle {
        /// The enemies, in order.
        monsters: Vec<MonsterId>,
        /// Set when won; the fight is gone afterwards.
        flag: Flag,
        /// A boss: no ambush, its own music later.
        #[serde(default)]
        boss: bool,
        /// Story effects of winning.
        #[serde(default)]
        on_win: Vec<StoryEffect>,
    },
    /// Loot, taken once.
    Chest {
        /// Set when opened.
        flag: Flag,
        /// Items inside.
        #[serde(default)]
        items: BTreeMap<ItemId, u16>,
        /// Weapons or armor inside.
        #[serde(default)]
        equipment: Vec<EquipmentId>,
        /// Money inside.
        #[serde(default)]
        kalds: u32,
    },
    /// Someone to talk to; the first talk whose condition holds plays.
    Npc {
        /// Fluent message id of the name, shown on the map.
        name: String,
        /// The talks, in priority order.
        talks: Vec<TalkDef>,
    },
    /// Buy and sell.
    Shop {
        /// The stock.
        shop: ShopId,
    },
    /// Heals the party for free and becomes the return point after a defeat.
    Temple,
    /// Heals the party for a price.
    Inn {
        /// Kalds per night.
        price: u32,
    },
    /// Missions to take, those the player's rank allows.
    Board {
        /// The missions posted.
        quests: Vec<QuestId>,
    },
    /// A place that does not open yet (a locked door, a shop for later);
    /// its label says what it is (or "???").
    Sealed,
}

/// One possible talk of an NPC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TalkDef {
    /// The scene.
    pub dialogue: DialogueId,
    /// When this talk is the one.
    #[serde(default)]
    pub when: Condition,
}

/// A path between two nodes of the same instance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrailDef {
    /// One end.
    pub a: NodeId,
    /// The other end.
    pub b: NodeId,
    /// Chance of a battle when walking it, in percent.
    #[serde(default)]
    pub chance: u8,
    /// Monsters met on it: each of the 1–3 enemies (up to the party's size
    /// when it is larger) is drawn from here by weight.
    #[serde(default)]
    pub monsters: Vec<Weighted>,
    /// When it is open.
    #[serde(default)]
    pub when: Condition,
}

/// A monster a trail can throw at the party, and how often.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Weighted {
    /// The monster.
    pub monster: MonsterId,
    /// Relative chance among the trail's monsters.
    pub weight: u16,
}

/// Something the story does: after a dialogue, a choice or a won fight.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum StoryEffect {
    /// Sets a story flag.
    SetFlag {
        /// The flag.
        flag: Flag,
    },
    /// A hero joins the party. One who left before comes back as they were;
    /// otherwise at `level`, or at the party's average level.
    Join {
        /// The hero.
        hero: HeroId,
        /// Level of a hero who never joined before.
        #[serde(default)]
        level: Option<u8>,
        /// A guest: fights on its own; the player does not command it.
        #[serde(default)]
        guest: bool,
    },
    /// A hero leaves the party, keeping their level and gear for a return.
    Leave {
        /// The hero.
        hero: HeroId,
    },
    /// Shows the campaign's next step at the top of the screen (a Fluent
    /// message id), or hides it.
    Objective {
        /// The message; `None` clears it.
        #[serde(default)]
        message: Option<String>,
    },
    /// Sets the player's Guild rank.
    SetRank {
        /// The new rank.
        rank: Rank,
    },
    /// Gives the player a mission, without the board.
    StartQuest {
        /// The mission.
        quest: QuestId,
    },
    /// Items into the bag.
    Give {
        /// The item.
        item: ItemId,
        /// How many.
        count: u16,
    },
    /// A weapon or armor into the bag.
    GiveEquipment {
        /// Which.
        id: EquipmentId,
    },
    /// Money.
    Kalds {
        /// How much.
        amount: u32,
    },
    /// Heals the whole party.
    Heal,
    /// Starts a fight; `on_win` runs after a victory.
    Battle {
        /// The enemies, in order.
        monsters: Vec<MonsterId>,
        /// A boss: the Campaign saves right before it starts.
        #[serde(default)]
        boss: bool,
        /// Effects of winning.
        #[serde(default)]
        on_win: Vec<StoryEffect>,
    },
    /// The Campaign saves here: a defeat comes back to this moment.
    Autosave,
    /// Moves the party to another place without walking (a cut to the next
    /// scene of the story).
    Travel {
        /// Where.
        to: Position,
    },
    /// Plays a scene once the screen is free (after a battle's rewards).
    Scene {
        /// The scene.
        dialogue: DialogueId,
    },
}

/// HP, MP and attributes: a class's level-1 values or its growth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StatLine {
    /// Max HP.
    pub max_hp: u16,
    /// Max MP.
    pub max_mp: u16,
    /// Strength.
    pub strength: u16,
    /// Dexterity.
    pub dexterity: u16,
    /// Vitality.
    pub vitality: u16,
    /// Magic.
    pub magic: u16,
    /// Spirit.
    pub spirit: u16,
    /// Luck.
    pub luck: u16,
}

/// A hero class: how it grows and what it learns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClassDef {
    /// Values at level 1.
    pub base: StatLine,
    /// Gained per level, in tenths (`15` = 1.5 per level), so slow
    /// attributes still grow.
    pub growth: StatLine,
    /// The class's basic attack.
    pub attack: ActionId,
    /// Technics and spells, by the level they come at.
    pub learns: Vec<Learn>,
    /// How the class fights when the engine plays it (auto-battle,
    /// simulator).
    pub ai: AiProfile,
}

/// An action a class learns at a level.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Learn {
    /// The level.
    pub level: u8,
    /// The action.
    pub action: ActionId,
}

/// A story hero.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HeroDef {
    /// The class.
    pub class: ClassId,
    /// Weapon on joining.
    #[serde(default)]
    pub weapon: Option<EquipmentId>,
    /// Armor on joining.
    #[serde(default)]
    pub armor: Option<EquipmentId>,
}

/// A monster kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MonsterDef {
    /// How it fights (its `name` must be `monster-<id>`).
    pub battle: CombatantSetup,
    /// Experience for each standing hero.
    pub xp: u32,
    /// Money it drops.
    pub kalds: u32,
    /// Items it may drop.
    #[serde(default)]
    pub drops: Vec<Drop>,
}

/// An item a monster may drop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Drop {
    /// The item.
    pub item: ItemId,
    /// Chance, in percent.
    pub percent: u8,
}

/// A dialogue scene.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DialogueDef {
    /// Who says each line, in order: a Fluent message id of the speaker's
    /// name (`hero-lyra`, `npc-bram`), or `null` for narration. Line `n`'s
    /// text is `dialogue-<id>-<n>`.
    pub lines: Vec<Option<String>>,
    /// Answers offered after the last line; empty for none.
    #[serde(default)]
    pub choices: Vec<ChoiceDef>,
    /// Effects when the scene ends (after the choice's own).
    #[serde(default)]
    pub effects: Vec<StoryEffect>,
}

/// An answer in a dialogue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChoiceDef {
    /// Effects of choosing it.
    #[serde(default)]
    pub effects: Vec<StoryEffect>,
}

/// What a shop sells, and for how much.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShopDef {
    /// Items.
    #[serde(default)]
    pub items: Vec<Priced<ItemId>>,
    /// Weapons and armor.
    #[serde(default)]
    pub equipment: Vec<Priced<EquipmentId>>,
}

/// A shop entry. Selling gives back half the price.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Priced<T> {
    /// What.
    pub id: T,
    /// Price in kalds.
    pub price: u32,
}

/// Where a piece of equipment goes.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Slot {
    /// Adds attack.
    Weapon,
    /// Adds defenses.
    Armor,
}

/// A weapon or armor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EquipmentDef {
    /// Weapon or armor.
    pub slot: Slot,
    /// Classes that can use it.
    pub classes: Vec<ClassId>,
    /// Weapon attack.
    #[serde(default)]
    pub attack: u16,
    /// Weapon hit chance in percent; 95 when absent.
    #[serde(default)]
    pub accuracy: Option<u8>,
    /// Extra critical chance (weapons).
    #[serde(default)]
    pub critical_chance: u8,
    /// Armor defense.
    #[serde(default)]
    pub defense: u16,
    /// Armor magic defense.
    #[serde(default)]
    pub magic_defense: u16,
    /// Armor evasion in percent.
    #[serde(default)]
    pub evasion: u8,
}
