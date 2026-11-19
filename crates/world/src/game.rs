//! A game in progress: the player's commands outside battle, the scene on
//! screen, the battles it asks the server to run, and what winning or
//! losing them does.

use std::collections::BTreeMap;
use std::sync::Arc;

use engine::{BattleRecord, BattleSetup, Effect, Encounter, ItemId, Opening, Outcome, Rng};
use serde::{Deserialize, Serialize};

use crate::World;
use crate::def::Slot;
use crate::def::{Condition, NodeDef, NodeKind, StoryEffect, TrailDef};
use crate::ids::{DialogueId, EquipmentId, Flag, MonsterId, NodeId, Position, ShopId};
use crate::progression;
use crate::save::{GameMode, HeroState, Progress, Save, SavedBattle};

/// Chance that a random encounter is an enemy ambush, in percent.
const ENEMY_AMBUSH_PERCENT: u8 = 5;
/// Chance that the player's ambush on a fixed fight works, in percent.
const PARTY_AMBUSH_PERCENT: u8 = 20;
/// Most heroes in the party.
pub const MAX_PARTY: usize = 3;
/// Most units of one item the bag holds.
const MAX_STACK: u16 = 99;

/// What the player does outside battle.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum GameCommand {
    /// Walks a trail to a neighboring node.
    Move {
        /// The node at the other end.
        node: NodeId,
    },
    /// Uses the current node: talk, open, fight, enter, shop, rest.
    Interact {
        /// On a fixed fight: try to ambush it.
        #[serde(default)]
        ambush: bool,
    },
    /// Next dialogue line; closes a notice, the rewards, a shop.
    Next,
    /// Picks a dialogue answer.
    Choose {
        /// Index of the answer.
        index: u8,
    },
    /// Buys from the open shop.
    Buy {
        /// What.
        good: ShopGood,
        /// How many.
        count: u16,
    },
    /// Sells to the open shop, for half its price there.
    Sell {
        /// What.
        good: ShopGood,
        /// How many.
        count: u16,
    },
    /// Puts on a weapon or armor from the bag (`None` takes it off).
    Equip {
        /// Party index of the hero.
        hero: u8,
        /// Weapon or armor.
        slot: Slot,
        /// The piece, from the bag.
        id: Option<EquipmentId>,
    },
    /// Uses a potion or ether outside battle.
    UseItem {
        /// The item.
        item: ItemId,
        /// Party index of the hero.
        hero: u8,
    },
}

/// Something a shop trades.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ShopGood {
    /// An item.
    Item {
        /// Which.
        id: ItemId,
    },
    /// A weapon or armor.
    Equipment {
        /// Which.
        id: EquipmentId,
    },
}

/// Why a command was refused; nothing changed.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[serde(rename_all = "snake_case")]
pub enum GameError {
    /// The current scene does not take this command.
    #[error("not now")]
    Busy,
    /// No such node here, or it is hidden.
    #[error("no such node")]
    NoSuchNode,
    /// No trail joins the two nodes.
    #[error("no trail")]
    NoTrail,
    /// The trail is blocked by the story.
    #[error("trail closed")]
    TrailClosed,
    /// Multiplayer cannot go where the Campaign has not been.
    #[error("barrier")]
    Barrier,
    /// The node has nothing to do (an empty chest, a won fight).
    #[error("nothing here")]
    NothingHere,
    /// No such dialogue answer.
    #[error("no such choice")]
    NoSuchChoice,
    /// The shop does not trade it.
    #[error("not for sale")]
    NotForSale,
    /// Not enough money.
    #[error("not enough crowns")]
    NotEnoughCrowns,
    /// Not in the bag (or not enough of it).
    #[error("not owned")]
    NotOwned,
    /// Wrong slot or class.
    #[error("cannot equip")]
    CannotEquip,
    /// The item does nothing outside battle.
    #[error("not usable here")]
    NotUsableHere,
    /// No such party member.
    #[error("no such hero")]
    NoSuchHero,
    /// The bag holds at most 99 of an item.
    #[error("bag full")]
    BagFull,
}

/// A battle the server must run, then report with [`Game::finish_battle`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleOrder {
    /// Fighters and definitions.
    pub setup: BattleSetup,
    /// Seed of the battle.
    pub seed: u64,
    /// What already happened in it (empty for a new battle): replay it with
    /// [`engine::Battle::replay`].
    pub record: BattleRecord,
}

/// What a won battle gave.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rewards {
    /// Experience for each standing hero.
    pub xp: u32,
    /// Money.
    pub crowns: u32,
    /// Dropped items.
    pub items: BTreeMap<ItemId, u16>,
    /// Heroes who reached a new level.
    pub level_ups: Vec<LevelUp>,
}

/// A hero's new level after a battle.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelUp {
    /// The hero.
    pub hero: crate::HeroId,
    /// The new level.
    pub level: u8,
    /// Actions learned on the way.
    pub learned: Vec<engine::ActionId>,
}

/// Loot from a chest.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Loot {
    /// Items.
    pub items: BTreeMap<ItemId, u16>,
    /// Weapons and armor.
    pub equipment: Vec<EquipmentId>,
    /// Money.
    pub crowns: u32,
}

/// What is on screen, and so which commands make sense.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Scene {
    /// Walking the instance map.
    Map,
    /// A dialogue at line `line` (0-based).
    Dialogue { dialogue: DialogueId, line: usize },
    /// A shop window.
    Shop { shop: ShopId },
    /// The server is running the save's battle ([`Save::battle`]).
    Battle,
    /// The rewards of a won battle. The win's story effects are already
    /// applied, except a follow-up battle, which waits in [`Save::battle`]
    /// and starts on [`GameCommand::Next`]; otherwise closing the rewards
    /// plays the node's arrival scene, if due (after a random encounter on
    /// the way, or a boss's aftermath).
    Victory { rewards: Rewards },
    /// Beaten: back at the Temple.
    Defeat,
    /// Healed at a Temple.
    Rested,
    /// Opened a chest.
    Found(Loot),
}

/// A player's game in one mode.
#[derive(Debug, Clone)]
pub struct Game {
    pub(crate) world: Arc<World>,
    pub(crate) mode: GameMode,
    pub(crate) progress: Progress,
    pub(crate) save: Save,
    pub(crate) scene: Scene,
    rng: Rng,
}

impl Game {
    /// Resumes a saved game on the map. A fresh Campaign save explores its
    /// starting node and plays its arrival scene. A save with a battle owed
    /// resumes in that battle: start it with [`Game::battle_order`].
    #[must_use]
    pub fn new(world: Arc<World>, mode: GameMode, progress: Progress, save: Save) -> Self {
        let rng = Rng::new(save.rng);
        let mut game = Self {
            world,
            mode,
            progress,
            save,
            scene: Scene::Map,
            rng,
        };
        game.explore();
        if game.save.battle.is_some() {
            game.scene = Scene::Battle;
        } else {
            game.arrive();
        }
        game
    }

    /// The mode.
    #[must_use]
    pub fn mode(&self) -> GameMode {
        self.mode
    }

    /// The shared progress, to store (changes only in the Campaign).
    #[must_use]
    pub fn progress(&self) -> &Progress {
        &self.progress
    }

    /// This mode's save, to store.
    #[must_use]
    pub fn save(&self) -> Save {
        Save {
            rng: self.rng.state(),
            ..self.save.clone()
        }
    }

    /// Whether a battle is waiting for [`Game::finish_battle`].
    #[must_use]
    pub fn in_battle(&self) -> bool {
        self.scene == Scene::Battle
    }

    /// The battle the server must run now, if any: the one [`Game::act`]
    /// just ordered, or the one a resumed game owes. The same save always
    /// gives the same battle.
    #[must_use]
    pub fn battle_order(&self) -> Option<BattleOrder> {
        let battle = self.save.battle.as_ref().filter(|_| self.in_battle())?;
        let party = self
            .save
            .party
            .iter()
            .filter_map(|hero| self.world.hero_setup(hero))
            .collect();
        let enemies = battle
            .monsters
            .iter()
            .filter_map(|monster| self.world.monster_setup(monster))
            .collect();
        let setup = BattleSetup {
            encounter: Encounter {
                opening: battle.opening,
                inventory: self.save.inventory.clone(),
                party,
                enemies,
            },
            actions: self.world.actions.clone(),
            items: self.world.items.clone(),
        };
        Some(BattleOrder {
            setup,
            seed: battle.seed,
            record: battle.record.clone(),
        })
    }

    /// Keeps how far the running battle went, so a resumed game picks it up
    /// there. Ignored when no battle is running.
    pub fn record_battle(&mut self, record: BattleRecord) {
        if let Some(battle) = self
            .save
            .battle
            .as_mut()
            .filter(|_| self.scene == Scene::Battle)
        {
            battle.record = record;
        }
    }

    /// Carries out a command. `Ok(Some(order))` means a battle starts: run
    /// it and report with [`Game::finish_battle`].
    ///
    /// # Errors
    ///
    /// [`GameError`] when the command makes no sense now; nothing changes.
    pub fn act(&mut self, command: GameCommand) -> Result<Option<BattleOrder>, GameError> {
        match command {
            GameCommand::Move { node } => self.walk(&node),
            GameCommand::Interact { ambush } => self.interact(ambush),
            GameCommand::Next => self.next(),
            GameCommand::Choose { index } => self.choose(index),
            GameCommand::Buy { good, count } => self.buy(&good, count).map(|()| None),
            GameCommand::Sell { good, count } => self.sell(&good, count).map(|()| None),
            GameCommand::Equip { hero, slot, id } => self.equip(hero, slot, id).map(|()| None),
            GameCommand::UseItem { item, hero } => self.use_item(&item, hero).map(|()| None),
        }
    }

    /// Applies the end of the battle [`Game::act`] ordered: each party
    /// member's HP and MP (in party order), the inventory as it ended, and
    /// who won. Ignored when no battle was ordered.
    pub fn finish_battle(
        &mut self,
        outcome: Outcome,
        party: &[(u16, u16)],
        inventory: &BTreeMap<ItemId, u16>,
    ) {
        if !self.in_battle() {
            return;
        }
        self.scene = Scene::Map;
        let Some(battle) = self.save.battle.take() else {
            return;
        };
        self.save.inventory = inventory
            .iter()
            .filter(|&(_, &count)| count > 0)
            .map(|(item, &count)| (item.clone(), count))
            .collect();
        match outcome {
            Outcome::Defeat => self.defeat(),
            Outcome::Victory => self.victory(battle, party),
        }
    }

    // --- map ---------------------------------------------------------------

    fn holds(&self, condition: &Condition) -> bool {
        condition
            .requires
            .iter()
            .all(|flag| self.progress.has(flag))
            && !condition.forbids.iter().any(|flag| self.progress.has(flag))
    }

    /// The node at `position`, when it exists for the story so far.
    pub(crate) fn node_at(&self, position: &Position) -> Option<&NodeDef> {
        let node = self
            .world
            .map
            .instances
            .get(&position.instance)?
            .nodes
            .get(&position.node)?;
        self.holds(&node.when).then_some(node)
    }

    fn here(&self) -> Option<&NodeDef> {
        self.node_at(&self.save.position)
    }

    fn trail(&self, a: &NodeId, b: &NodeId) -> Option<&TrailDef> {
        self.world
            .map
            .instances
            .get(&self.save.position.instance)?
            .trails
            .iter()
            .find(|t| (&t.a == a && &t.b == b) || (&t.a == b && &t.b == a))
    }

    /// Whether the trail is open for the story so far.
    pub(crate) fn trail_open(&self, trail: &TrailDef) -> bool {
        self.holds(&trail.when)
    }

    fn explore(&mut self) {
        if self.mode == GameMode::Campaign {
            self.progress.explored.insert(self.save.position.clone());
        }
    }

    fn set_flag(&mut self, flag: &Flag) {
        if self.mode == GameMode::Campaign {
            self.progress.flags.insert(flag.clone());
        }
    }

    /// Plays the current node's arrival scene, if its time has come.
    fn arrive(&mut self) {
        if self.mode != GameMode::Campaign {
            return;
        }
        let event = self
            .here()
            .and_then(|node| node.event.as_ref())
            .filter(|event| self.holds(&event.when))
            .map(|event| event.dialogue.clone());
        if let Some(dialogue) = event {
            self.scene = Scene::Dialogue { dialogue, line: 0 };
        }
    }

    fn require_map(&self) -> Result<(), GameError> {
        if self.scene == Scene::Map {
            Ok(())
        } else {
            Err(GameError::Busy)
        }
    }

    fn walk(&mut self, to: &NodeId) -> Result<Option<BattleOrder>, GameError> {
        self.require_map()?;
        let target = Position {
            instance: self.save.position.instance.clone(),
            node: to.clone(),
        };
        if self.node_at(&target).is_none() {
            return Err(GameError::NoSuchNode);
        }
        let trail = self
            .trail(&self.save.position.node, to)
            .ok_or(GameError::NoTrail)?;
        if !self.trail_open(trail) {
            return Err(GameError::TrailClosed);
        }
        if self.mode == GameMode::Multiplayer && !self.progress.explored.contains(&target) {
            return Err(GameError::Barrier);
        }
        let (chance, groups) = (trail.chance, trail.encounters.clone());
        self.save.position = target;
        self.explore();

        let total: u64 = groups.iter().map(|g| u64::from(g.weight)).sum();
        if total > 0 && self.rng.chance_percent(chance) {
            let mut roll = self.rng.below(total);
            let group = groups.iter().find(|g| {
                let weight = u64::from(g.weight);
                if roll < weight {
                    return true;
                }
                roll -= weight;
                false
            });
            if let Some(group) = group {
                let opening = if self.rng.chance_percent(ENEMY_AMBUSH_PERCENT) {
                    Opening::EnemyAmbush
                } else {
                    Opening::Normal
                };
                let monsters = group.monsters.clone();
                return Ok(self.start_battle(monsters, None, Vec::new(), opening));
            }
        }
        self.arrive();
        Ok(None)
    }

    fn interact(&mut self, ambush: bool) -> Result<Option<BattleOrder>, GameError> {
        self.require_map()?;
        let kind = self.here().ok_or(GameError::NoSuchNode)?.kind.clone();
        match kind {
            NodeKind::Path => Err(GameError::NothingHere),
            NodeKind::Exit { to } => {
                if self.mode == GameMode::Multiplayer && !self.progress.explored.contains(&to) {
                    return Err(GameError::Barrier);
                }
                self.save.position = to;
                self.explore();
                self.arrive();
                Ok(None)
            }
            NodeKind::Battle {
                monsters,
                flag,
                boss,
                on_win,
            } => {
                if self.progress.has(&flag) {
                    return Err(GameError::NothingHere);
                }
                let opening = if ambush && !boss && self.rng.chance_percent(PARTY_AMBUSH_PERCENT) {
                    Opening::PartyAmbush
                } else {
                    Opening::Normal
                };
                Ok(self.start_battle(monsters, Some(flag), on_win, opening))
            }
            NodeKind::Chest {
                flag,
                items,
                equipment,
                crowns,
            } => {
                if self.mode != GameMode::Campaign || self.progress.has(&flag) {
                    return Err(GameError::NothingHere);
                }
                self.set_flag(&flag);
                for (item, count) in &items {
                    self.add_item(item, *count);
                }
                for piece in &equipment {
                    self.add_equipment(piece);
                }
                self.save.crowns = self.save.crowns.saturating_add(crowns);
                self.scene = Scene::Found(Loot {
                    items,
                    equipment,
                    crowns,
                });
                Ok(None)
            }
            NodeKind::Npc { talks, .. } => {
                let talk = talks
                    .iter()
                    .find(|talk| self.holds(&talk.when))
                    .ok_or(GameError::NothingHere)?;
                self.scene = Scene::Dialogue {
                    dialogue: talk.dialogue.clone(),
                    line: 0,
                };
                Ok(None)
            }
            NodeKind::Shop { shop } => {
                self.scene = Scene::Shop { shop };
                Ok(None)
            }
            NodeKind::Temple => {
                for hero in &mut self.save.party {
                    hero.heal(&self.world);
                }
                self.save.temple = self.save.position.clone();
                self.scene = Scene::Rested;
                Ok(None)
            }
        }
    }

    // --- dialogues and story ---------------------------------------------

    fn next(&mut self) -> Result<Option<BattleOrder>, GameError> {
        match std::mem::replace(&mut self.scene, Scene::Map) {
            Scene::Dialogue { dialogue, line } => {
                let def = self.world.map.dialogues.get(&dialogue);
                let lines = def.map_or(0, |def| def.lines.len());
                let has_choices = def.is_some_and(|def| !def.choices.is_empty());
                if line + 1 < lines {
                    self.scene = Scene::Dialogue {
                        dialogue,
                        line: line + 1,
                    };
                    return Ok(None);
                }
                if has_choices {
                    self.scene = Scene::Dialogue { dialogue, line };
                    return Err(GameError::Busy);
                }
                Ok(self.end_dialogue(&dialogue, &[]))
            }
            Scene::Victory { .. } => {
                if self.save.battle.is_some() {
                    self.scene = Scene::Battle;
                    return Ok(self.battle_order());
                }
                self.arrive();
                Ok(None)
            }
            Scene::Shop { .. } | Scene::Defeat | Scene::Rested | Scene::Found(_) => Ok(None),
            scene @ (Scene::Map | Scene::Battle) => {
                self.scene = scene;
                Err(GameError::Busy)
            }
        }
    }

    fn choose(&mut self, index: u8) -> Result<Option<BattleOrder>, GameError> {
        let Scene::Dialogue { dialogue, line } = &self.scene else {
            return Err(GameError::Busy);
        };
        let def = self
            .world
            .map
            .dialogues
            .get(dialogue)
            .ok_or(GameError::NoSuchChoice)?;
        if *line + 1 < def.lines.len() {
            return Err(GameError::Busy);
        }
        let choice = def
            .choices
            .get(usize::from(index))
            .ok_or(GameError::NoSuchChoice)?
            .effects
            .clone();
        let dialogue = dialogue.clone();
        self.scene = Scene::Map;
        Ok(self.end_dialogue(&dialogue, &choice))
    }

    fn end_dialogue(
        &mut self,
        dialogue: &DialogueId,
        choice: &[StoryEffect],
    ) -> Option<BattleOrder> {
        let mut effects = choice.to_vec();
        if let Some(def) = self.world.map.dialogues.get(dialogue) {
            effects.extend(def.effects.iter().cloned());
        }
        self.apply(&effects)
    }

    /// Runs story effects in order. A battle effect is always last (checked
    /// with the content) and starts the battle.
    fn apply(&mut self, effects: &[StoryEffect]) -> Option<BattleOrder> {
        for effect in effects {
            match effect {
                StoryEffect::SetFlag { flag } => self.set_flag(flag),
                StoryEffect::Join { hero } => self.join(hero),
                StoryEffect::Give { item, count } => self.add_item(item, *count),
                StoryEffect::GiveEquipment { id } => self.add_equipment(id),
                StoryEffect::Crowns { amount } => {
                    self.save.crowns = self.save.crowns.saturating_add(*amount);
                }
                StoryEffect::Heal => {
                    for hero in &mut self.save.party {
                        hero.heal(&self.world);
                    }
                }
                StoryEffect::Battle { monsters, on_win } => {
                    return self.start_battle(
                        monsters.clone(),
                        None,
                        on_win.clone(),
                        Opening::Normal,
                    );
                }
            }
        }
        None
    }

    fn join(&mut self, hero: &crate::HeroId) {
        if self.save.party.len() >= MAX_PARTY || self.save.party.iter().any(|h| &h.hero == hero) {
            return;
        }
        let levels: u32 = self.save.party.iter().map(|h| u32::from(h.level)).sum();
        let count = u32::try_from(self.save.party.len()).unwrap_or(1).max(1);
        let level = u8::try_from((levels + count / 2) / count)
            .unwrap_or(1)
            .max(1);
        let joined = HeroState::new(&self.world, hero, level);
        self.save.party.push(joined);
    }

    fn add_item(&mut self, item: &ItemId, count: u16) {
        let held = self.save.inventory.entry(item.clone()).or_insert(0);
        *held = held.saturating_add(count).min(MAX_STACK);
    }

    fn add_equipment(&mut self, id: &EquipmentId) {
        let held = self.save.bag.entry(id.clone()).or_insert(0);
        *held = held.saturating_add(1).min(MAX_STACK);
    }

    // --- battles -----------------------------------------------------------

    /// Owes the player a battle (saved, so it survives a dropped
    /// connection) without starting it yet.
    fn owe_battle(
        &mut self,
        monsters: Vec<MonsterId>,
        flag: Option<Flag>,
        on_win: Vec<StoryEffect>,
        opening: Opening,
    ) {
        self.save.battle = Some(SavedBattle {
            monsters,
            flag,
            on_win,
            opening,
            seed: self.rng.next_u64(),
            record: BattleRecord::default(),
        });
    }

    fn start_battle(
        &mut self,
        monsters: Vec<MonsterId>,
        flag: Option<Flag>,
        on_win: Vec<StoryEffect>,
        opening: Opening,
    ) -> Option<BattleOrder> {
        self.owe_battle(monsters, flag, on_win, opening);
        self.scene = Scene::Battle;
        self.battle_order()
    }

    fn defeat(&mut self) {
        for hero in &mut self.save.party {
            hero.heal(&self.world);
        }
        self.save.position = self.save.temple.clone();
        self.scene = Scene::Defeat;
    }

    fn victory(&mut self, battle: SavedBattle, party: &[(u16, u16)]) {
        let mut rewards = Rewards::default();
        for monster in battle
            .monsters
            .iter()
            .filter_map(|m| self.world.map.monsters.get(m))
        {
            rewards.xp = rewards.xp.saturating_add(monster.xp);
            rewards.crowns = rewards.crowns.saturating_add(monster.crowns);
            for drop in &monster.drops {
                if self.rng.chance_percent(drop.percent) {
                    *rewards.items.entry(drop.item.clone()).or_insert(0) += 1;
                }
            }
        }
        for (item, count) in &rewards.items {
            self.add_item(item, *count);
        }
        self.save.crowns = self.save.crowns.saturating_add(rewards.crowns);

        let world = Arc::clone(&self.world);
        for (hero, &(hp, mp)) in self.save.party.iter_mut().zip(party) {
            hero.mp = mp;
            if hp == 0 {
                hero.hp = 1;
                continue;
            }
            hero.hp = hp;
            let class = world
                .map
                .heroes
                .get(&hero.hero)
                .and_then(|def| world.map.classes.get(&def.class));
            let before = class.map(|c| progression::known_actions(c, hero.level));
            if hero.gain_xp(&world, rewards.xp) > 0 {
                let learned = match (class, before) {
                    (Some(class), Some(before)) => progression::known_actions(class, hero.level)
                        .into_iter()
                        .filter(|action| !before.contains(action))
                        .collect(),
                    _ => Vec::new(),
                };
                rewards.level_ups.push(LevelUp {
                    hero: hero.hero.clone(),
                    level: hero.level,
                    learned,
                });
            }
        }
        if let Some(flag) = &battle.flag {
            self.set_flag(flag);
        }
        // Applied now, so a dropped connection cannot lose them; only a
        // follow-up battle waits for the player to close the rewards.
        let mut effects = battle.on_win;
        let follow_up = effects.pop_if(|effect| matches!(effect, StoryEffect::Battle { .. }));
        self.apply(&effects);
        if let Some(StoryEffect::Battle { monsters, on_win }) = follow_up {
            self.owe_battle(monsters, None, on_win, Opening::Normal);
        }
        self.scene = Scene::Victory { rewards };
    }

    // --- shop, equipment, items -------------------------------------------

    fn price(&self, good: &ShopGood) -> Result<u32, GameError> {
        let Scene::Shop { shop } = &self.scene else {
            return Err(GameError::Busy);
        };
        let def = self
            .world
            .map
            .shops
            .get(shop)
            .ok_or(GameError::NotForSale)?;
        let price = match good {
            ShopGood::Item { id } => def.items.iter().find(|p| &p.id == id).map(|p| p.price),
            ShopGood::Equipment { id } => {
                def.equipment.iter().find(|p| &p.id == id).map(|p| p.price)
            }
        };
        price.ok_or(GameError::NotForSale)
    }

    fn held(&self, good: &ShopGood) -> u16 {
        match good {
            ShopGood::Item { id } => self.save.inventory.get(id).copied().unwrap_or(0),
            ShopGood::Equipment { id } => self.save.bag.get(id).copied().unwrap_or(0),
        }
    }

    fn buy(&mut self, good: &ShopGood, count: u16) -> Result<(), GameError> {
        let total = self.price(good)?.saturating_mul(u32::from(count));
        if count == 0 {
            return Err(GameError::NotForSale);
        }
        if total > self.save.crowns {
            return Err(GameError::NotEnoughCrowns);
        }
        if self.held(good).saturating_add(count) > MAX_STACK {
            return Err(GameError::BagFull);
        }
        self.save.crowns -= total;
        match good {
            ShopGood::Item { id } => self.add_item(id, count),
            ShopGood::Equipment { id } => {
                for _ in 0..count {
                    self.add_equipment(id);
                }
            }
        }
        Ok(())
    }

    fn sell(&mut self, good: &ShopGood, count: u16) -> Result<(), GameError> {
        let price = self.price(good)? / 2;
        if count == 0 || self.held(good) < count {
            return Err(GameError::NotOwned);
        }
        let stock = match good {
            ShopGood::Item { id } => self.save.inventory.get_mut(id),
            ShopGood::Equipment { id } => self.save.bag.get_mut(id),
        };
        if let Some(held) = stock {
            *held -= count;
        }
        self.save.inventory.retain(|_, held| *held > 0);
        self.save.bag.retain(|_, held| *held > 0);
        self.save.crowns = self
            .save
            .crowns
            .saturating_add(price.saturating_mul(u32::from(count)));
        Ok(())
    }

    fn equip(&mut self, hero: u8, slot: Slot, id: Option<EquipmentId>) -> Result<(), GameError> {
        if !matches!(self.scene, Scene::Map | Scene::Shop { .. }) {
            return Err(GameError::Busy);
        }
        let index = usize::from(hero);
        let state = self.save.party.get(index).ok_or(GameError::NoSuchHero)?;
        let class = self
            .world
            .map
            .heroes
            .get(&state.hero)
            .map(|def| def.class.clone())
            .ok_or(GameError::NoSuchHero)?;
        if let Some(id) = &id {
            let def = self
                .world
                .map
                .equipment
                .get(id)
                .ok_or(GameError::CannotEquip)?;
            if def.slot != slot || !def.classes.contains(&class) {
                return Err(GameError::CannotEquip);
            }
            if self.save.bag.get(id).copied().unwrap_or(0) == 0 {
                return Err(GameError::NotOwned);
            }
            let held = self.save.bag.entry(id.clone()).or_insert(0);
            *held -= 1;
            self.save.bag.retain(|_, held| *held > 0);
        }
        let worn = match slot {
            Slot::Weapon => &mut self.save.party[index].weapon,
            Slot::Armor => &mut self.save.party[index].armor,
        };
        if let Some(old) = std::mem::replace(worn, id) {
            self.add_equipment(&old);
        }
        let world = Arc::clone(&self.world);
        let state = &mut self.save.party[index];
        if let Some(stats) = world.hero_stats(state) {
            state.hp = state.hp.min(stats.max_hp);
            state.mp = state.mp.min(stats.max_mp);
        }
        Ok(())
    }

    fn use_item(&mut self, item: &ItemId, hero: u8) -> Result<(), GameError> {
        self.require_map()?;
        if self.save.inventory.get(item).copied().unwrap_or(0) == 0 {
            return Err(GameError::NotOwned);
        }
        let effect = self
            .world
            .items
            .get(item)
            .and_then(|def| self.world.actions.get(&def.action))
            .map(|action| action.effect.clone())
            .ok_or(GameError::NotUsableHere)?;
        let world = Arc::clone(&self.world);
        let state = self
            .save
            .party
            .get_mut(usize::from(hero))
            .ok_or(GameError::NoSuchHero)?;
        let full = world.hero_stats(state).ok_or(GameError::NoSuchHero)?;
        match effect {
            Effect::RestoreHp { amount } if state.hp < full.max_hp => {
                state.hp = state.hp.saturating_add(amount).min(full.max_hp);
            }
            Effect::RestoreMp { amount } if state.mp < full.max_mp => {
                state.mp = state.mp.saturating_add(amount).min(full.max_mp);
            }
            _ => return Err(GameError::NotUsableHere),
        }
        if let Some(held) = self.save.inventory.get_mut(item) {
            *held -= 1;
        }
        self.save.inventory.retain(|_, held| *held > 0);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use engine::{Battle, PartyControl};

    use super::*;
    use crate::test_world::{position, world};
    use crate::view::{Fog, NodeKindView, SceneView};

    fn new_game() -> Game {
        let world = world();
        let save = Save::new(&world, 1);
        Game::new(world, GameMode::Campaign, Progress::default(), save)
    }

    fn walk(game: &mut Game, node: &str) -> Option<BattleOrder> {
        game.act(GameCommand::Move {
            node: NodeId::new(node),
        })
        .unwrap()
    }

    /// Plays the battle to its end with the engine on both sides.
    fn fight(game: &mut Game, order: BattleOrder) -> Outcome {
        let mut battle = Battle::new(order.setup, order.seed, PartyControl::Ai).unwrap();
        while battle.outcome().is_none() {
            battle.step();
        }
        let party: Vec<_> = battle
            .combatants()
            .iter()
            .filter(|c| c.side() == engine::Side::Party)
            .map(|c| (c.hp(), c.mp()))
            .collect();
        let outcome = battle.outcome().unwrap();
        game.finish_battle(outcome, &party, battle.inventory());
        outcome
    }

    fn finish(game: &mut Game, outcome: Outcome, party: &[(u16, u16)]) {
        let inventory = game.save.inventory.clone();
        game.finish_battle(outcome, party, &inventory);
    }

    #[test]
    fn walking_a_trail_explores_and_may_meet_monsters() {
        let mut game = new_game();

        let order = walk(&mut game, "road").expect("a 100% trail always meets monsters");

        assert_eq!(game.save.position, position("field", "road"));
        assert!(game.progress.explored.contains(&position("field", "road")));
        assert_eq!(order.setup.encounter.enemies[0].name, "monster-slime");
        assert!(game.in_battle());
        assert_eq!(
            game.act(GameCommand::Move {
                node: NodeId::new("home")
            }),
            Err(GameError::Busy)
        );
    }

    #[test]
    fn only_open_trails_to_existing_nodes_can_be_walked() {
        let mut game = new_game();
        let order = walk(&mut game, "road").unwrap();
        fight(&mut game, order);
        game.act(GameCommand::Next).unwrap();

        let refused = |game: &mut Game, node: &str| {
            game.act(GameCommand::Move {
                node: NodeId::new(node),
            })
        };
        assert_eq!(refused(&mut game, "nowhere"), Err(GameError::NoSuchNode));
        assert_eq!(refused(&mut game, "road"), Err(GameError::NoTrail));
        assert_eq!(refused(&mut game, "secret"), Err(GameError::TrailClosed));
    }

    #[test]
    fn a_victory_pays_experience_money_and_drops() {
        let mut game = new_game();
        walk(&mut game, "road").unwrap();

        finish(&mut game, Outcome::Victory, &[(40, 3)]);

        let hero = &game.save.party[0];
        assert_eq!((hero.level, hero.xp), (2, 5));
        assert_eq!(hero.hp, 40 + 10, "a new level adds its HP");
        assert_eq!(game.save.crowns, 107);
        assert_eq!(game.save.inventory[&ItemId::new("potion")], 2);
        let SceneView::Victory { rewards } = game.view().scene else {
            panic!("expected the rewards");
        };
        assert_eq!(
            rewards.level_ups[0].learned,
            vec![engine::ActionId::new("attack")]
        );
    }

    #[test]
    fn a_knocked_out_hero_gets_up_with_1_hp_and_no_experience() {
        let mut game = new_game();
        walk(&mut game, "road").unwrap();

        finish(&mut game, Outcome::Victory, &[(0, 0)]);

        let hero = &game.save.party[0];
        assert_eq!((hero.hp, hero.level, hero.xp), (1, 1, 0));
    }

    #[test]
    fn a_defeat_heals_and_returns_to_the_temple() {
        let mut game = new_game();
        walk(&mut game, "road").unwrap();

        finish(&mut game, Outcome::Defeat, &[(0, 0)]);

        assert_eq!(game.save.position, position("field", "home"));
        assert_eq!(game.save.party[0].hp, 100);
        assert_eq!(game.view().scene, SceneView::Defeat);
    }

    #[test]
    fn a_fixed_fight_is_won_once_and_its_story_applies_at_once() {
        let mut game = new_game();
        walk(&mut game, "road").unwrap();
        finish(&mut game, Outcome::Victory, &[(100, 10)]);
        game.act(GameCommand::Next).unwrap();
        walk(&mut game, "gate");

        let order = game.act(GameCommand::Interact { ambush: false }).unwrap();
        assert!(order.is_some());
        finish(&mut game, Outcome::Victory, &[(100, 10)]);

        assert!(game.progress.has(&Flag::new("gate-won")));
        assert_eq!(
            game.save.party.len(),
            2,
            "bea joined before the rewards closed"
        );
        assert_eq!(game.save.party[1].level, game.save.party[0].level);
        game.act(GameCommand::Next).unwrap();
        assert_eq!(
            game.act(GameCommand::Interact { ambush: false }),
            Err(GameError::NothingHere)
        );
        walk(&mut game, "road");
        assert!(
            walk(&mut game, "secret").is_none(),
            "the won fight opened the trail"
        );
    }

    #[test]
    fn an_arrival_scene_plays_once_and_a_chest_opens_once() {
        let mut game = new_game();
        walk(&mut game, "road").unwrap();
        finish(&mut game, Outcome::Victory, &[(100, 10)]);
        game.act(GameCommand::Next).unwrap();

        walk(&mut game, "chest");
        assert!(matches!(game.view().scene, SceneView::Dialogue { .. }));
        assert_eq!(
            game.act(GameCommand::Interact { ambush: false }),
            Err(GameError::Busy)
        );
        game.act(GameCommand::Next).unwrap();
        game.act(GameCommand::Interact { ambush: false }).unwrap();
        game.act(GameCommand::Next).unwrap();

        assert_eq!(game.save.crowns, 157);
        assert_eq!(game.save.bag[&EquipmentId::new("sword")], 1);
        assert_eq!(
            game.act(GameCommand::Interact { ambush: false }),
            Err(GameError::NothingHere)
        );
        walk(&mut game, "road");
        walk(&mut game, "chest");
        assert_eq!(game.view().scene, SceneView::Map, "the scene played once");
    }

    #[test]
    fn a_dialogue_waits_for_its_answer_and_answers_have_effects() {
        let mut game = new_game();
        walk(&mut game, "road").unwrap();
        finish(&mut game, Outcome::Victory, &[(100, 10)]);
        game.act(GameCommand::Next).unwrap();
        walk(&mut game, "elder");
        game.act(GameCommand::Interact { ambush: false }).unwrap();

        assert_eq!(
            game.act(GameCommand::Choose { index: 0 }),
            Err(GameError::Busy),
            "answers come after the last line"
        );
        game.act(GameCommand::Next).unwrap();
        assert!(matches!(
            game.view().scene,
            SceneView::Dialogue { choices: 2, .. }
        ));
        assert_eq!(game.act(GameCommand::Next), Err(GameError::Busy));
        assert_eq!(
            game.act(GameCommand::Choose { index: 5 }),
            Err(GameError::NoSuchChoice)
        );
        let crowns = game.save.crowns;
        assert!(
            game.act(GameCommand::Choose { index: 0 })
                .unwrap()
                .is_none()
        );
        assert_eq!(game.save.crowns, crowns + 5);

        game.act(GameCommand::Interact { ambush: false }).unwrap();
        game.act(GameCommand::Next).unwrap();
        assert!(
            game.act(GameCommand::Choose { index: 1 })
                .unwrap()
                .is_some()
        );
    }

    #[test]
    fn the_shop_buys_sells_and_the_bag_equips() {
        let mut game = new_game();
        walk(&mut game, "road").unwrap();
        finish(&mut game, Outcome::Victory, &[(100, 10)]);
        game.act(GameCommand::Next).unwrap();
        walk(&mut game, "shop");
        game.act(GameCommand::Interact { ambush: false }).unwrap();
        let sword = ShopGood::Equipment {
            id: EquipmentId::new("sword"),
        };

        assert_eq!(
            game.act(GameCommand::Buy {
                good: sword.clone(),
                count: 2
            }),
            Err(GameError::NotEnoughCrowns)
        );
        game.act(GameCommand::Buy {
            good: sword.clone(),
            count: 1,
        })
        .unwrap();
        assert_eq!(game.save.crowns, 107 - 60);
        game.act(GameCommand::Equip {
            hero: 0,
            slot: Slot::Weapon,
            id: Some(EquipmentId::new("sword")),
        })
        .unwrap();
        assert!(game.save.bag.is_empty());
        assert_eq!(game.view().party[0].stats.attack, 10 + 1 + 12);
        assert_eq!(
            game.act(GameCommand::Sell {
                good: sword.clone(),
                count: 1
            }),
            Err(GameError::NotOwned),
            "a worn sword is not in the bag"
        );
        game.act(GameCommand::Equip {
            hero: 0,
            slot: Slot::Weapon,
            id: None,
        })
        .unwrap();
        game.act(GameCommand::Sell {
            good: sword,
            count: 1,
        })
        .unwrap();
        assert_eq!(game.save.crowns, 47 + 30);
    }

    #[test]
    fn a_potion_heals_outside_battle_but_not_a_healthy_hero() {
        let mut game = new_game();
        let potion = ItemId::new("potion");
        let use_potion = |game: &mut Game| {
            game.act(GameCommand::UseItem {
                item: ItemId::new("potion"),
                hero: 0,
            })
        };

        assert_eq!(use_potion(&mut game), Err(GameError::NotUsableHere));
        game.save.party[0].hp = 30;
        use_potion(&mut game).unwrap();

        assert_eq!(game.save.party[0].hp, 100);
        assert!(!game.save.inventory.contains_key(&potion));
        assert_eq!(use_potion(&mut game), Err(GameError::NotOwned));
    }

    #[test]
    fn exits_lead_to_another_instance() {
        let mut game = new_game();
        walk(&mut game, "road").unwrap();
        finish(&mut game, Outcome::Victory, &[(100, 10)]);
        game.act(GameCommand::Next).unwrap();
        walk(&mut game, "stairs");

        game.act(GameCommand::Interact { ambush: false }).unwrap();

        assert_eq!(game.save.position, position("cave", "entry"));
        assert_eq!(game.view().instance.id.as_str(), "cave");
    }

    #[test]
    fn the_fog_shows_neighbors_without_their_content() {
        let game = new_game();

        let view = game.view();
        let road = view.instance.nodes.iter().find(|n| n.id.as_str() == "road");

        assert_eq!(view.instance.nodes.len(), 2, "home and its only neighbor");
        assert_eq!(
            road.map(|n| (n.fog, n.kind.clone())),
            Some((Fog::Glimpsed, None))
        );
        assert_eq!(view.instance.trails.len(), 1);
        assert_eq!(view.regions.len(), 1);
    }

    #[test]
    fn multiplayer_cannot_pass_where_the_campaign_has_not_been() {
        let world = world();
        let mut game = Game::new(
            Arc::clone(&world),
            GameMode::Multiplayer,
            Progress {
                flags: std::collections::BTreeSet::new(),
                explored: std::collections::BTreeSet::from([position("field", "home")]),
            },
            Save::new(&world, 1),
        );

        assert_eq!(
            game.act(GameCommand::Move {
                node: NodeId::new("road")
            }),
            Err(GameError::Barrier)
        );
        let view = game.view();
        assert_eq!(
            view.instance
                .nodes
                .iter()
                .find(|n| n.id.as_str() == "road")
                .map(|n| n.fog),
            Some(Fog::Barrier)
        );
        assert!(matches!(
            view.instance.nodes[0].kind,
            Some(NodeKindView::Temple)
        ));
    }

    #[test]
    fn the_save_resumes_the_same_random_sequence() {
        let mut game = new_game();
        let first = walk(&mut game, "road").unwrap();
        let saved = game.save();

        let mut again = Game::new(
            world(),
            GameMode::Campaign,
            Progress::default(),
            Save::new(&world(), 1),
        );
        let replay = walk(&mut again, "road").unwrap();

        assert_eq!(first.seed, replay.seed);
        assert_eq!(saved.rng, again.save().rng);
    }

    fn resume(game: &Game) -> Game {
        Game::new(world(), game.mode, game.progress.clone(), game.save())
    }

    #[test]
    fn a_battle_left_unfinished_resumes_where_it_was() {
        let mut game = new_game();
        let order = walk(&mut game, "road").unwrap();
        let hp = game.save.party[0].hp;
        let mut battle = Battle::new(order.setup, order.seed, PartyControl::Player).unwrap();
        for _ in 0..40 {
            battle.step();
        }
        game.record_battle(battle.record());

        let mut resumed = resume(&game);

        assert!(matches!(resumed.view().scene, SceneView::Battle));
        let again = resumed.battle_order().unwrap();
        assert_eq!(again.record.ticks, 40);
        let rebuilt = Battle::replay(again.setup, again.seed, PartyControl::Player, &again.record);
        assert_eq!(rebuilt.unwrap().record(), battle.record());
        assert_eq!(
            resumed.act(GameCommand::Move {
                node: NodeId::new("home")
            }),
            Err(GameError::Busy),
            "no walking away from it"
        );
        finish(&mut resumed, Outcome::Victory, &[(hp - 1, 10)]);
        assert_eq!(resumed.save().battle, None);
        assert_eq!(resume(&resumed).battle_order(), None);
    }

    #[test]
    fn a_follow_up_battle_waiting_behind_the_rewards_survives_a_resume() {
        let mut game = new_game();
        walk(&mut game, "road").unwrap();
        finish(&mut game, Outcome::Victory, &[(100, 10)]);
        game.act(GameCommand::Next).unwrap();
        walk(&mut game, "elder");
        game.act(GameCommand::Interact { ambush: false }).unwrap();
        game.act(GameCommand::Next).unwrap();
        game.act(GameCommand::Choose { index: 1 }).unwrap().unwrap();
        finish(&mut game, Outcome::Victory, &[(100, 10)]);
        assert!(matches!(game.view().scene, SceneView::Victory { .. }));

        let resumed = resume(&game);
        let order = resumed.battle_order().unwrap();

        assert_eq!(order.setup.encounter.enemies.len(), 2);
        assert_eq!(game.act(GameCommand::Next).unwrap(), Some(order));
    }

    #[test]
    fn engine_battles_can_be_won_and_resolved() {
        let mut game = new_game();
        let order = walk(&mut game, "road").unwrap();

        assert_eq!(fight(&mut game, order), Outcome::Victory);
        assert!(matches!(game.view().scene, SceneView::Victory { .. }));
    }
}
