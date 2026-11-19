//! A game in progress: the player's commands outside battle, the scene on
//! screen, the battles it asks the server to run, and what winning or
//! losing them does.

use std::collections::BTreeMap;
use std::sync::Arc;

use engine::{BattleRecord, BattleSetup, Effect, Encounter, ItemId, Opening, Outcome, Rng};
use serde::{Deserialize, Serialize};

use crate::World;
use crate::def::Slot;
use crate::def::{Condition, NodeDef, NodeKind, QuestGoal, StoryEffect, TrailDef, Weighted};
use crate::ids::{DialogueId, EquipmentId, Flag, MonsterId, NodeId, Position, QuestId, ShopId};
use crate::progression;
use crate::save::{Checkpoint, GameMode, HeroState, Progress, Save, SavedBattle};

/// Chance that a random encounter is an enemy ambush, in percent.
const ENEMY_AMBUSH_PERCENT: u8 = 5;
/// Chance that the player's ambush on a fixed fight works, in percent.
const PARTY_AMBUSH_PERCENT: u8 = 20;
/// Most heroes in the party.
pub const MAX_PARTY: usize = 4;
/// Most enemies a trail throws at a party of up to this size.
const BASE_ENEMIES: usize = 3;
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
    /// Takes a mission from the board on screen.
    AcceptQuest {
        /// The mission.
        quest: QuestId,
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
    #[error("not enough kalds")]
    NotEnoughKalds,
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
    /// The mission is not on this board.
    #[error("not on the board")]
    NotOnBoard,
    /// The player's Guild rank is too low for it.
    #[error("rank too low")]
    RankTooLow,
    /// Already under way or completed.
    #[error("mission already taken")]
    QuestTaken,
    /// The place does not open yet.
    #[error("closed")]
    Sealed,
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
    pub kalds: u32,
    /// Dropped items.
    pub items: BTreeMap<ItemId, u16>,
    /// Heroes who reached a new level.
    pub level_ups: Vec<LevelUp>,
    /// Missions this battle completed (their pay is in `kalds`).
    pub quests_done: Vec<QuestId>,
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
    pub kalds: u32,
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
    /// Beaten in the Multiplayer: back at the Temple.
    Defeat,
    /// Beaten in the Campaign: back to the last auto save.
    GameOver,
    /// Healed at a Temple.
    Rested,
    /// Opened a chest.
    Found(Loot),
    /// A mission board.
    Board { quests: Vec<QuestId> },
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
        if game.save.checkpoint.is_none() {
            game.autosave();
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
        let enemies: Vec<_> = battle
            .monsters
            .iter()
            .filter_map(|monster| self.world.monster_setup(monster))
            .collect();
        let setup = BattleSetup {
            encounter: Encounter {
                forms: self.world.forms_for(&enemies),
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
            GameCommand::AcceptQuest { quest } => self.accept_quest(&quest).map(|()| None),
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

    /// Plays a scene waiting in line, or the current node's arrival scene
    /// if its time has come.
    fn arrive(&mut self) {
        if self.mode != GameMode::Campaign {
            return;
        }
        if !self.save.queued.is_empty() {
            let dialogue = self.save.queued.remove(0);
            self.scene = Scene::Dialogue { dialogue, line: 0 };
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
        let (chance, pool) = (trail.chance, trail.monsters.clone());
        // Saved one step before a boss ambush: a Game Over leaves the player
        // free to rest and train before walking in again.
        if self.boss_ambush_at(&target) {
            self.autosave();
        }
        self.save.position = target;
        self.explore();

        if !pool.is_empty() && self.rng.chance_percent(chance) {
            let most = BASE_ENEMIES.max(self.save.party.len());
            let count = 1 + self.rng.below(u64::try_from(most).unwrap_or(3));
            let monsters: Vec<MonsterId> = (0..count)
                .filter_map(|_| self.draw(&pool).cloned())
                .collect();
            let opening = if self.rng.chance_percent(ENEMY_AMBUSH_PERCENT) {
                Opening::EnemyAmbush
            } else {
                Opening::Normal
            };
            return Ok(self.start_battle(monsters, None, Vec::new(), opening));
        }
        self.arrive();
        Ok(None)
    }

    /// One monster from a trail's pool, by weight.
    fn draw<'a>(&mut self, pool: &'a [Weighted]) -> Option<&'a MonsterId> {
        let total: u64 = pool.iter().map(|w| u64::from(w.weight)).sum();
        if total == 0 {
            return None;
        }
        let mut roll = self.rng.below(total);
        pool.iter()
            .find(|w| {
                let weight = u64::from(w.weight);
                if roll < weight {
                    return true;
                }
                roll -= weight;
                false
            })
            .map(|w| &w.monster)
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
                self.autosave();
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
                Ok(self.start_fight(monsters, Some(flag), on_win, opening, boss))
            }
            NodeKind::Chest {
                flag,
                items,
                equipment,
                kalds,
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
                self.save.kalds = self.save.kalds.saturating_add(kalds);
                self.scene = Scene::Found(Loot {
                    items,
                    equipment,
                    kalds,
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
                self.autosave();
                self.scene = Scene::Rested;
                Ok(None)
            }
            NodeKind::Inn { price } => {
                if self.save.kalds < price {
                    return Err(GameError::NotEnoughKalds);
                }
                self.save.kalds -= price;
                for hero in &mut self.save.party {
                    hero.heal(&self.world);
                }
                self.autosave();
                self.scene = Scene::Rested;
                Ok(None)
            }
            NodeKind::Board { quests } => {
                self.scene = Scene::Board { quests };
                Ok(None)
            }
            NodeKind::Sealed => Err(GameError::Sealed),
        }
    }

    // --- missions ------------------------------------------------------------

    fn accept_quest(&mut self, quest: &QuestId) -> Result<(), GameError> {
        let Scene::Board { quests } = &self.scene else {
            return Err(GameError::Busy);
        };
        if !quests.contains(quest) {
            return Err(GameError::NotOnBoard);
        }
        let def = self
            .world
            .map
            .quests
            .get(quest)
            .ok_or(GameError::NotOnBoard)?;
        if self.save.rank.is_none_or(|rank| rank < def.rank) {
            return Err(GameError::RankTooLow);
        }
        if self.save.quests.contains_key(quest) || self.save.quests_done.contains(quest) {
            return Err(GameError::QuestTaken);
        }
        self.save.quests.insert(quest.clone(), 0);
        Ok(())
    }

    fn start_quest(&mut self, quest: &QuestId) {
        if !self.save.quests.contains_key(quest) && !self.save.quests_done.contains(quest) {
            self.save.quests.insert(quest.clone(), 0);
        }
    }

    /// Counts the monsters a won battle defeated toward the missions under
    /// way; queues the scenes reached and pays the missions completed.
    fn count_kills(&mut self, monsters: &[MonsterId], rewards: &mut Rewards) {
        let world = Arc::clone(&self.world);
        let active: Vec<QuestId> = self.save.quests.keys().cloned().collect();
        for quest in active {
            let Some(def) = world.map.quests.get(&quest) else {
                continue;
            };
            let QuestGoal::Kill {
                monsters: kinds,
                count,
            } = &def.goal;
            let killed = monsters.iter().filter(|m| kinds.contains(m)).count();
            let before = self.save.quests[&quest];
            let after = before
                .saturating_add(u16::try_from(killed).unwrap_or(u16::MAX))
                .min(*count);
            for event in &def.events {
                if before < event.at && event.at <= after {
                    self.save.queued.push(event.dialogue.clone());
                }
            }
            if after >= *count {
                self.save.quests.remove(&quest);
                self.save.quests_done.insert(quest.clone());
                self.save.kalds = self.save.kalds.saturating_add(def.kalds);
                rewards.kalds = rewards.kalds.saturating_add(def.kalds);
                rewards.quests_done.push(quest);
            } else {
                self.save.quests.insert(quest, after);
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
            Scene::Shop { .. } | Scene::Board { .. } | Scene::Defeat => Ok(None),
            Scene::GameOver => Ok(self.load_checkpoint()),
            Scene::Rested | Scene::Found(_) => {
                self.arrive();
                Ok(None)
            }
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
        let order = self.apply(&effects);
        // A scene the story queued, or the arrival scene of where it sent
        // the party, plays next.
        if order.is_none() && self.scene == Scene::Map {
            self.arrive();
        }
        order
    }

    /// Runs story effects in order. A battle effect is always last (checked
    /// with the content) and starts the battle. Before a boss, the Campaign
    /// saves ahead of every effect, so a Game Over replays the whole scene.
    fn apply(&mut self, effects: &[StoryEffect]) -> Option<BattleOrder> {
        // A boss that ambushes on arrival was saved for before the walk in.
        if has_boss(effects) && !self.boss_ambush_at(&self.save.position) {
            self.autosave();
        }
        for effect in effects {
            match effect {
                StoryEffect::SetFlag { flag } => self.set_flag(flag),
                StoryEffect::Join { hero, level, guest } => self.join(hero, *level, *guest),
                StoryEffect::Leave { hero } => self.leave(hero),
                StoryEffect::Objective { message } => self.save.objective.clone_from(message),
                StoryEffect::SetRank { rank } => self.save.rank = Some(*rank),
                StoryEffect::StartQuest { quest } => self.start_quest(quest),
                StoryEffect::Give { item, count } => self.add_item(item, *count),
                StoryEffect::GiveEquipment { id } => self.add_equipment(id),
                StoryEffect::Kalds { amount } => {
                    self.save.kalds = self.save.kalds.saturating_add(*amount);
                }
                StoryEffect::Heal => {
                    for hero in &mut self.save.party {
                        hero.heal(&self.world);
                    }
                }
                StoryEffect::Battle {
                    monsters, on_win, ..
                } => {
                    return self.start_battle(
                        monsters.clone(),
                        None,
                        on_win.clone(),
                        Opening::Normal,
                    );
                }
                StoryEffect::Autosave => self.autosave(),
                StoryEffect::Travel { to } => {
                    self.save.position = to.clone();
                    self.explore();
                }
                StoryEffect::Scene { dialogue } => self.save.queued.push(dialogue.clone()),
            }
        }
        None
    }

    fn join(&mut self, hero: &crate::HeroId, level: Option<u8>, guest: bool) {
        if self.save.party.len() >= MAX_PARTY || self.save.party.iter().any(|h| &h.hero == hero) {
            return;
        }
        let mut joined = self.save.benched.remove(hero).unwrap_or_else(|| {
            let level = level.unwrap_or_else(|| {
                let levels: u32 = self.save.party.iter().map(|h| u32::from(h.level)).sum();
                let count = u32::try_from(self.save.party.len()).unwrap_or(1).max(1);
                u8::try_from((levels + count / 2) / count).unwrap_or(1)
            });
            HeroState::new(&self.world, hero, level.max(1))
        });
        joined.guest = guest;
        self.save.party.push(joined);
    }

    fn leave(&mut self, hero: &crate::HeroId) {
        if let Some(index) = self.save.party.iter().position(|h| &h.hero == hero) {
            let left = self.save.party.remove(index);
            self.save.benched.insert(left.hero.clone(), left);
        }
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
        self.start_fight(monsters, flag, on_win, opening, false)
    }

    /// Starts a battle; before a boss, the Campaign saves first, so a Game
    /// Over comes back to the moment before it (free to prepare).
    fn start_fight(
        &mut self,
        monsters: Vec<MonsterId>,
        flag: Option<Flag>,
        on_win: Vec<StoryEffect>,
        opening: Opening,
        boss: bool,
    ) -> Option<BattleOrder> {
        if boss {
            self.autosave();
        }
        self.owe_battle(monsters, flag, on_win, opening);
        self.scene = Scene::Battle;
        self.battle_order()
    }

    /// The Campaign ends in a Game Over; the Multiplayer brings the party
    /// back at its Temple.
    fn defeat(&mut self) {
        if self.mode == GameMode::Campaign {
            self.scene = Scene::GameOver;
            return;
        }
        for hero in &mut self.save.party {
            hero.heal(&self.world);
        }
        self.save.position = self.save.temple.clone();
        self.scene = Scene::Defeat;
    }

    /// Whether arriving at `position` plays a scene that starts a boss
    /// battle (an ambush).
    fn boss_ambush_at(&self, position: &Position) -> bool {
        self.node_at(position)
            .and_then(|node| node.event.as_ref())
            .filter(|event| self.holds(&event.when))
            .and_then(|event| self.world.map.dialogues.get(&event.dialogue))
            .is_some_and(|dialogue| has_boss(&dialogue.effects))
    }

    /// Campaign only: remembers this moment for a Game Over.
    fn autosave(&mut self) {
        if self.mode != GameMode::Campaign {
            return;
        }
        let mut save = self.save();
        save.checkpoint = None;
        self.save.checkpoint = Some(Box::new(Checkpoint {
            save,
            progress: self.progress.clone(),
        }));
    }

    /// After a Game Over: back to the last auto save, the story as it was. A
    /// follow-up boss saved with its battle resumes in it. The encounter luck
    /// goes on rather than replaying, so the same ambush does not wait
    /// right after every Game Over.
    fn load_checkpoint(&mut self) -> Option<BattleOrder> {
        let checkpoint = self.save.checkpoint.take()?;
        self.save = checkpoint.save.clone();
        self.progress = checkpoint.progress.clone();
        self.save.checkpoint = Some(checkpoint);
        self.explore();
        if self.save.battle.is_some() {
            self.scene = Scene::Battle;
            return self.battle_order();
        }
        self.scene = Scene::Map;
        self.arrive();
        None
    }

    fn victory(&mut self, battle: SavedBattle, party: &[(u16, u16)]) {
        let mut rewards = Rewards::default();
        for monster in battle
            .monsters
            .iter()
            .filter_map(|m| self.world.map.monsters.get(m))
        {
            rewards.xp = rewards.xp.saturating_add(monster.xp);
            rewards.kalds = rewards.kalds.saturating_add(monster.kalds);
            for drop in &monster.drops {
                if self.rng.chance_percent(drop.percent) {
                    *rewards.items.entry(drop.item.clone()).or_insert(0) += 1;
                }
            }
        }
        for (item, count) in &rewards.items {
            self.add_item(item, *count);
        }
        self.save.kalds = self.save.kalds.saturating_add(rewards.kalds);
        self.count_kills(&battle.monsters, &mut rewards);

        let world = Arc::clone(&self.world);
        for (hero, &(hp, mp)) in self.save.party.iter_mut().zip(party) {
            hero.mp = mp;
            if hp == 0 {
                hero.hp = 1;
                continue;
            }
            hero.hp = hp;
            if hero.guest {
                continue;
            }
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
        if let Some(StoryEffect::Battle {
            monsters,
            boss,
            on_win,
        }) = follow_up
        {
            self.owe_battle(monsters, None, on_win, Opening::Normal);
            if boss {
                self.autosave();
            }
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
        if total > self.save.kalds {
            return Err(GameError::NotEnoughKalds);
        }
        if self.held(good).saturating_add(count) > MAX_STACK {
            return Err(GameError::BagFull);
        }
        self.save.kalds -= total;
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
        self.save.kalds = self
            .save
            .kalds
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

/// Whether the effects start a boss battle.
fn has_boss(effects: &[StoryEffect]) -> bool {
    effects
        .iter()
        .any(|effect| matches!(effect, StoryEffect::Battle { boss: true, .. }))
}

#[cfg(test)]
mod tests {
    use engine::{Battle, PartyControl};

    use super::*;
    use crate::def::Rank;
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

    /// How many slimes a road battle brought (1 to 3).
    fn slimes_met(order: &BattleOrder) -> u32 {
        u32::try_from(order.setup.encounter.enemies.len()).unwrap()
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
        let slimes = slimes_met(&walk(&mut game, "road").unwrap());

        finish(&mut game, Outcome::Victory, &[(40, 3)]);

        let hero = &game.save.party[0];
        let (level, xp) = progression::add_xp(1, 0, 25 * slimes);
        assert_eq!((hero.level, hero.xp), (level, xp));
        assert_eq!(hero.hp, 40 + 10 * u16::from(level - 1), "new levels add HP");
        assert_eq!(game.save.kalds, 100 + 7 * slimes);
        assert_eq!(
            u32::from(game.save.inventory[&ItemId::new("potion")]),
            1 + slimes,
            "each slime drops a potion"
        );
        let SceneView::Victory { rewards } = game.view().scene else {
            panic!("expected the rewards");
        };
        assert_eq!(
            rewards.level_ups[0].learned,
            vec![engine::ActionId::new("attack")]
        );
    }

    #[test]
    fn a_trail_brings_one_to_three_monsters() {
        let mut game = new_game();
        let mut sizes = std::collections::BTreeSet::new();
        for _ in 0..60 {
            for node in ["road", "home"] {
                if let Some(order) = walk(&mut game, node) {
                    sizes.insert(order.setup.encounter.enemies.len());
                    finish(&mut game, Outcome::Victory, &[(100, 10)]);
                    game.act(GameCommand::Next).unwrap();
                }
                while game.scene != Scene::Map {
                    game.act(GameCommand::Next).unwrap();
                }
            }
        }

        assert_eq!(sizes, std::collections::BTreeSet::from([1, 2, 3]));
    }

    #[test]
    fn a_hero_who_leaves_comes_back_as_they_were_and_a_guest_acts_alone() {
        let mut game = new_game();
        let bea = crate::HeroId::new("bea");
        game.apply(&[StoryEffect::Join {
            hero: bea.clone(),
            level: Some(3),
            guest: false,
        }]);
        assert_eq!(game.save.party[1].level, 3);

        game.apply(&[StoryEffect::Leave { hero: bea.clone() }]);
        assert_eq!(game.save.party.len(), 1);
        game.apply(&[StoryEffect::Join {
            hero: bea,
            level: Some(9),
            guest: true,
        }]);

        let back = &game.save.party[1];
        assert_eq!(back.level, 3, "a returning hero keeps their level");
        assert!(game.world.hero_setup(back).unwrap().guest);
    }

    #[test]
    fn the_story_can_move_the_party_and_queue_a_scene() {
        let mut game = new_game();
        game.apply(&[
            StoryEffect::Travel {
                to: position("cave", "entry"),
            },
            StoryEffect::Scene {
                dialogue: DialogueId::new("found-chest"),
            },
        ]);
        game.arrive();

        assert_eq!(game.save.position, position("cave", "entry"));
        assert!(game.progress.explored.contains(&position("cave", "entry")));
        assert!(
            matches!(&game.scene, Scene::Dialogue { dialogue, .. } if dialogue.as_str() == "found-chest")
        );
    }

    #[test]
    fn a_board_gives_the_missions_the_rank_allows() {
        let mut game = new_game();
        walk(&mut game, "road").unwrap();
        finish(&mut game, Outcome::Victory, &[(100, 10)]);
        game.act(GameCommand::Next).unwrap();
        walk(&mut game, "board");
        game.act(GameCommand::Interact { ambush: false }).unwrap();
        let accept = |game: &mut Game, quest: &str| {
            game.act(GameCommand::AcceptQuest {
                quest: QuestId::new(quest),
            })
        };

        assert_eq!(accept(&mut game, "slimes"), Err(GameError::RankTooLow));
        game.apply(&[StoryEffect::SetRank { rank: Rank::Bronze }]);
        assert_eq!(accept(&mut game, "slimes"), Ok(None));
        assert_eq!(accept(&mut game, "slimes"), Err(GameError::QuestTaken));
        assert_eq!(accept(&mut game, "dragon"), Err(GameError::RankTooLow));
        assert_eq!(accept(&mut game, "nowhere"), Err(GameError::NotOnBoard));
        assert_eq!(game.view().quests[0].count, 4);
    }

    #[test]
    fn kills_advance_a_mission_play_its_scene_and_pay_it_once() {
        let mut game = new_game();
        let slimes = QuestId::new("slimes");
        game.apply(&[StoryEffect::StartQuest {
            quest: slimes.clone(),
        }]);
        let mut scenes = 0;
        let mut paid = None;
        for _ in 0..20 {
            for node in ["road", "home"] {
                if walk(&mut game, node).is_some() {
                    let before = game.save.kalds;
                    finish(&mut game, Outcome::Victory, &[(100, 10)]);
                    if let SceneView::Victory { rewards } = game.view().scene
                        && rewards.quests_done == [slimes.clone()]
                    {
                        paid = Some(game.save.kalds - before);
                    }
                    game.act(GameCommand::Next).unwrap();
                }
                while game.scene != Scene::Map {
                    if matches!(&game.scene, Scene::Dialogue { dialogue, .. } if dialogue.as_str() == "found-chest")
                    {
                        scenes += 1;
                    }
                    game.act(GameCommand::Next).unwrap();
                }
            }
        }

        assert!(game.save.quests_done.contains(&slimes));
        assert!(!game.save.quests.contains_key(&slimes));
        assert!(
            paid.is_some_and(|kalds| kalds >= 20),
            "the pay comes with the rewards"
        );
        assert_eq!(scenes, 1, "the mission's scene plays once");
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
    fn a_multiplayer_defeat_heals_and_returns_to_the_temple() {
        let world = world();
        let explored = ["home", "road"].map(|node| position("field", node));
        let mut game = Game::new(
            Arc::clone(&world),
            GameMode::Multiplayer,
            Progress {
                epoch: 0,
                flags: std::collections::BTreeSet::new(),
                explored: explored.into_iter().collect(),
            },
            Save::new(&world, 1),
        );
        walk(&mut game, "road").unwrap();

        finish(&mut game, Outcome::Defeat, &[(0, 0)]);

        assert_eq!(game.save.position, position("field", "home"));
        assert_eq!(game.save.party[0].hp, 100);
        assert_eq!(game.view().scene, SceneView::Defeat);
    }

    #[test]
    fn a_campaign_defeat_is_a_game_over_back_to_the_last_auto_save() {
        let mut game = new_game();
        walk(&mut game, "road").unwrap();
        finish(&mut game, Outcome::Victory, &[(100, 10)]);
        game.act(GameCommand::Next).unwrap();
        let kalds_at_start = 100;
        assert!(game.save.kalds > kalds_at_start);
        game.apply(&[StoryEffect::SetFlag {
            flag: Flag::new("later"),
        }]);

        walk(&mut game, "home").unwrap();
        finish(&mut game, Outcome::Defeat, &[(0, 0)]);
        assert_eq!(game.view().scene, SceneView::GameOver);
        assert_eq!(game.act(GameCommand::Next), Ok(None));

        assert_eq!(game.save.position, position("field", "home"));
        assert_eq!(
            game.save.kalds, kalds_at_start,
            "back to the new game's save"
        );
        assert!(!game.progress.has(&Flag::new("later")), "and its story");
    }

    #[test]
    fn a_boss_saved_before_it_starts_again_after_a_game_over() {
        let mut game = new_game();
        walk(&mut game, "road").unwrap();
        finish(&mut game, Outcome::Victory, &[(100, 10)]);
        game.act(GameCommand::Next).unwrap();
        walk(&mut game, "elder");
        game.act(GameCommand::Interact { ambush: false }).unwrap();
        game.act(GameCommand::Next).unwrap();
        game.act(GameCommand::Choose { index: 1 }).unwrap().unwrap();
        finish(&mut game, Outcome::Victory, &[(100, 10)]);
        let boss = game.act(GameCommand::Next).unwrap().unwrap();

        finish(&mut game, Outcome::Defeat, &[(0, 0)]);
        let again = game.act(GameCommand::Next).unwrap();

        assert_eq!(again, Some(boss), "the same boss battle, from its start");
        assert!(game.in_battle());
    }

    #[test]
    fn a_game_over_at_a_story_boss_comes_back_before_its_scene() {
        let mut game = new_game();
        let order = game.apply(&[
            StoryEffect::SetFlag {
                flag: Flag::new("ambushed"),
            },
            StoryEffect::Battle {
                monsters: vec![MonsterId::new("slime")],
                boss: true,
                on_win: Vec::new(),
            },
        ]);
        assert!(order.is_some());

        finish(&mut game, Outcome::Defeat, &[(0, 0)]);

        assert_eq!(game.act(GameCommand::Next), Ok(None), "free to prepare");
        assert!(
            !game.progress.has(&Flag::new("ambushed")),
            "the scene plays again"
        );
        assert!(!game.in_battle());
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
        let slimes = slimes_met(&walk(&mut game, "road").unwrap());
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

        assert_eq!(game.save.kalds, 150 + 7 * slimes);
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
        let kalds = game.save.kalds;
        assert!(
            game.act(GameCommand::Choose { index: 0 })
                .unwrap()
                .is_none()
        );
        assert_eq!(game.save.kalds, kalds + 5);

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
        let slimes = slimes_met(&walk(&mut game, "road").unwrap());
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
            Err(GameError::NotEnoughKalds)
        );
        game.act(GameCommand::Buy {
            good: sword.clone(),
            count: 1,
        })
        .unwrap();
        assert_eq!(game.save.kalds, 100 + 7 * slimes - 60);
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
        assert_eq!(game.save.kalds, 100 + 7 * slimes - 60 + 30);
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
                epoch: 0,
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
