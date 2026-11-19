//! What the player sees of a [`Game`]: the current instance under its fog
//! of war, the world map, the party, the bags, and the scene on screen.
//! Hidden nodes are never sent; glimpsed ones are sent without their
//! content.

use std::collections::BTreeMap;

use engine::{ActionId, ItemId};
use serde::Serialize;

use crate::def::{NodeKind, Rank, Slot, Terrain};
use crate::game::{Game, Loot, Rewards, Scene};
use crate::ids::{
    DialogueId, EquipmentId, HeroId, InstanceId, NodeId, Position, QuestId, RegionId, ShopId,
};
use crate::save::GameMode;
use crate::{ShopGood, progression};

/// Everything the game screen shows.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameView {
    /// Campaign or Multiplayer.
    pub mode: GameMode,
    /// What is on screen.
    pub scene: SceneView,
    /// The instance the player is in.
    pub instance: InstanceView,
    /// The node the player stands on.
    pub position: NodeId,
    /// World map pieces the player knows.
    pub regions: Vec<RegionView>,
    /// The heroes.
    pub party: Vec<HeroView>,
    /// Money.
    pub kalds: u32,
    /// Items.
    pub inventory: BTreeMap<ItemId, u16>,
    /// Weapons and armor not worn.
    pub bag: Vec<BagEntry>,
    /// Guild rank, once registered.
    pub rank: Option<Rank>,
    /// The campaign's next step (a Fluent message id).
    pub objective: Option<String>,
    /// Missions under way.
    pub quests: Vec<QuestView>,
}

/// A mission under way (name: `quest-<id>`).
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestView {
    /// The mission.
    pub quest: QuestId,
    /// Progress so far.
    pub progress: u16,
    /// Progress that completes it.
    pub count: u16,
}

/// A mission on a board.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardEntry {
    /// The mission.
    pub quest: QuestId,
    /// Lowest rank allowed.
    pub rank: Rank,
    /// Progress that completes it.
    pub count: u16,
    /// Where the player stands with it.
    pub status: BoardStatus,
}

/// Whether a board mission can be taken.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BoardStatus {
    /// Can be taken.
    Open,
    /// Needs a higher rank.
    Locked,
    /// Under way.
    Taken,
    /// Completed.
    Done,
}

/// The current instance.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceView {
    /// Its id (name: `instance-<id>`).
    pub id: InstanceId,
    /// Its region.
    pub region: RegionId,
    /// Background style.
    pub terrain: Terrain,
    /// Visible nodes.
    pub nodes: Vec<NodeView>,
    /// Trails between visible nodes, at least one end explored.
    pub trails: Vec<TrailView>,
}

/// How much of a node the fog lets through.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Fog {
    /// Visited: everything shows.
    Explored,
    /// Next to a visited node: shape only.
    Glimpsed,
    /// Multiplayer: the Campaign never went there; it cannot be entered.
    Barrier,
}

/// A visible node.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeView {
    /// Its id.
    pub id: NodeId,
    /// Position in a 1000×1000 box.
    pub x: u16,
    /// Position in a 1000×1000 box.
    pub y: u16,
    /// How much shows.
    pub fog: Fog,
    /// What it is; `None` unless explored.
    pub kind: Option<NodeKindView>,
    /// Fluent message id of its name, when explored and named.
    pub label: Option<String>,
}

/// What an explored node is.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum NodeKindView {
    /// A step on the way.
    Path,
    /// Leads to another instance.
    Exit {
        /// Where.
        to: InstanceId,
    },
    /// A fixed fight.
    Battle {
        /// A boss.
        boss: bool,
        /// Already won.
        done: bool,
    },
    /// Loot.
    Chest {
        /// Already opened.
        done: bool,
    },
    /// Someone to talk to.
    Npc {
        /// Fluent message id of the name.
        name: String,
    },
    /// A shop.
    Shop,
    /// Heal and return point.
    Temple,
    /// Paid rest.
    Inn {
        /// Kalds per night.
        price: u32,
    },
    /// Missions to take.
    Board,
    /// Does not open yet.
    Sealed,
}

/// A trail.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrailView {
    /// One end.
    pub a: NodeId,
    /// The other end.
    pub b: NodeId,
    /// Walkable now (otherwise blocked by the story).
    pub open: bool,
}

/// A world map piece the player has been to.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionView {
    /// Its id (name: `region-<id>`).
    pub id: RegionId,
    /// SVG path in a 1000×1000 box.
    pub shape: String,
    /// Where its name goes.
    pub label: [u16; 2],
    /// The player is in it.
    pub current: bool,
}

/// A hero, as the party menu shows it.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeroView {
    /// Which hero (name: `hero-<id>`).
    pub hero: HeroId,
    /// Class id (name: `class-<id>`).
    pub class: String,
    /// Level.
    pub level: u8,
    /// Experience toward the next level.
    pub xp: u32,
    /// Experience the next level needs.
    pub xp_next: u32,
    /// HP.
    pub hp: u16,
    /// Max HP.
    pub max_hp: u16,
    /// MP.
    pub mp: u16,
    /// Max MP.
    pub max_mp: u16,
    /// Attack, Defense, Magic Attack, Magic Defense, Dexterity, Luck.
    pub stats: HeroStatsView,
    /// Worn weapon.
    pub weapon: Option<EquipmentId>,
    /// Worn armor.
    pub armor: Option<EquipmentId>,
    /// Technics and spells.
    pub actions: Vec<ActionId>,
}

/// Derived stats for the party menu.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeroStatsView {
    /// Strength + weapon.
    pub attack: u16,
    /// Vitality + armor.
    pub defense: u16,
    /// Magic.
    pub magic_attack: u16,
    /// Spirit + armor.
    pub magic_defense: u16,
    /// ATB speed, accuracy, evasion.
    pub dexterity: u16,
    /// Criticals, status resistance.
    pub luck: u16,
}

/// Unworn equipment.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BagEntry {
    /// Which (name: `equipment-<id>`).
    pub id: EquipmentId,
    /// Weapon or armor.
    pub slot: Slot,
    /// Classes that can wear it.
    pub classes: Vec<String>,
    /// How many.
    pub count: u16,
}

/// The scene on screen.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum SceneView {
    /// Walking the map.
    Map,
    /// A dialogue line.
    Dialogue {
        /// The scene (line text: `dialogue-<id>-<line + 1>`).
        dialogue: DialogueId,
        /// Line index, from 0.
        line: u16,
        /// Fluent message id of the speaker's name; `None` for narration.
        speaker: Option<String>,
        /// Answers to pick after this line (text:
        /// `dialogue-<id>-choice-<n>`); 0 until the last line.
        choices: u8,
    },
    /// A shop window.
    Shop {
        /// The shop.
        shop: ShopId,
        /// Its goods.
        goods: Vec<ShopEntry>,
    },
    /// A battle runs (the battle messages carry it).
    Battle,
    /// What a won battle gave.
    Victory {
        /// The rewards.
        rewards: Rewards,
    },
    /// Beaten: back at the Temple, healed.
    Defeat,
    /// Healed at a Temple.
    Rested,
    /// Opened a chest.
    Found {
        /// What it held.
        loot: Loot,
    },
    /// A mission board.
    Board {
        /// Its missions.
        entries: Vec<BoardEntry>,
    },
    /// Beaten in the Campaign: Continue goes back to the last auto save.
    GameOver,
}

/// A shop line.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ShopEntry {
    /// What.
    pub good: ShopGood,
    /// Price in kalds (selling gives half).
    pub price: u32,
    /// How many the player has.
    pub owned: u16,
}

impl Game {
    /// What the player sees now.
    #[must_use]
    pub fn view(&self) -> GameView {
        let world = &self.world;
        let position = &self.save.position;
        let inventory = self.save.inventory.clone();
        GameView {
            mode: self.mode,
            scene: self.scene_view(),
            instance: self.instance_view(),
            position: position.node.clone(),
            regions: self.region_views(),
            party: self
                .save
                .party
                .iter()
                .filter_map(|hero| {
                    let class_id = world.map.heroes.get(&hero.hero)?.class.clone();
                    let class = world.map.classes.get(&class_id)?;
                    let stats = world.hero_stats(hero)?;
                    Some(HeroView {
                        hero: hero.hero.clone(),
                        class: class_id.to_string(),
                        level: hero.level,
                        xp: hero.xp,
                        xp_next: progression::xp_to_next(hero.level),
                        hp: hero.hp,
                        max_hp: stats.max_hp,
                        mp: hero.mp,
                        max_mp: stats.max_mp,
                        stats: HeroStatsView {
                            attack: stats.attack(),
                            defense: stats.defense(),
                            magic_attack: stats.magic_attack(),
                            magic_defense: stats.magic_defense(),
                            dexterity: stats.attributes.dexterity,
                            luck: stats.attributes.luck,
                        },
                        weapon: hero.weapon.clone(),
                        armor: hero.armor.clone(),
                        actions: progression::known_actions(class, hero.level),
                    })
                })
                .collect(),
            kalds: self.save.kalds,
            inventory,
            bag: self
                .save
                .bag
                .iter()
                .filter_map(|(id, &count)| {
                    let def = world.map.equipment.get(id)?;
                    Some(BagEntry {
                        id: id.clone(),
                        slot: def.slot,
                        classes: def.classes.iter().map(ToString::to_string).collect(),
                        count,
                    })
                })
                .collect(),
            rank: self.save.rank,
            objective: self.save.objective.clone(),
            quests: self
                .save
                .quests
                .iter()
                .filter_map(|(quest, &progress)| {
                    let def = world.map.quests.get(quest)?;
                    Some(QuestView {
                        quest: quest.clone(),
                        progress,
                        count: def.goal.count(),
                    })
                })
                .collect(),
        }
    }

    fn scene_view(&self) -> SceneView {
        match &self.scene {
            Scene::Map => SceneView::Map,
            Scene::Dialogue { dialogue, line } => {
                let def = self.world.map.dialogues.get(dialogue);
                let lines = def.map_or(0, |def| def.lines.len());
                let last = *line + 1 >= lines;
                SceneView::Dialogue {
                    dialogue: dialogue.clone(),
                    line: u16::try_from(*line).unwrap_or(u16::MAX),
                    speaker: def.and_then(|def| def.lines.get(*line).cloned().flatten()),
                    choices: if last {
                        def.map_or(0, |def| u8::try_from(def.choices.len()).unwrap_or(0))
                    } else {
                        0
                    },
                }
            }
            Scene::Shop { shop } => {
                let def = self.world.map.shops.get(shop);
                let items = def
                    .into_iter()
                    .flat_map(|def| &def.items)
                    .map(|p| ShopEntry {
                        good: ShopGood::Item { id: p.id.clone() },
                        price: p.price,
                        owned: self.save.inventory.get(&p.id).copied().unwrap_or(0),
                    });
                let equipment = def
                    .into_iter()
                    .flat_map(|def| &def.equipment)
                    .map(|p| ShopEntry {
                        good: ShopGood::Equipment { id: p.id.clone() },
                        price: p.price,
                        owned: self.save.bag.get(&p.id).copied().unwrap_or(0),
                    });
                SceneView::Shop {
                    shop: shop.clone(),
                    goods: items.chain(equipment).collect(),
                }
            }
            Scene::Battle => SceneView::Battle,
            Scene::Victory { rewards, .. } => SceneView::Victory {
                rewards: rewards.clone(),
            },
            Scene::Defeat => SceneView::Defeat,
            Scene::Rested => SceneView::Rested,
            Scene::Found(loot) => SceneView::Found { loot: loot.clone() },
            Scene::GameOver => SceneView::GameOver,
            Scene::Board { quests } => SceneView::Board {
                entries: quests
                    .iter()
                    .filter_map(|quest| {
                        let def = self.world.map.quests.get(quest)?;
                        let status = if self.save.quests_done.contains(quest) {
                            BoardStatus::Done
                        } else if self.save.quests.contains_key(quest) {
                            BoardStatus::Taken
                        } else if self.save.rank.is_some_and(|rank| rank >= def.rank) {
                            BoardStatus::Open
                        } else {
                            BoardStatus::Locked
                        };
                        Some(BoardEntry {
                            quest: quest.clone(),
                            rank: def.rank,
                            count: def.goal.count(),
                            status,
                        })
                    })
                    .collect(),
            },
        }
    }

    fn fog_of(&self, position: &Position, neighbors_explored: bool) -> Option<Fog> {
        if self.progress.explored.contains(position) {
            Some(Fog::Explored)
        } else if self.mode == GameMode::Multiplayer {
            neighbors_explored.then_some(Fog::Barrier)
        } else {
            neighbors_explored.then_some(Fog::Glimpsed)
        }
    }

    fn instance_view(&self) -> InstanceView {
        let id = &self.save.position.instance;
        let Some(def) = self.world.map.instances.get(id) else {
            return InstanceView {
                id: id.clone(),
                region: RegionId::new(""),
                terrain: Terrain::Fields,
                nodes: Vec::new(),
                trails: Vec::new(),
            };
        };
        let at = |node: &NodeId| Position {
            instance: id.clone(),
            node: node.clone(),
        };
        let explored = |node: &NodeId| self.progress.explored.contains(&at(node));
        let open_trails: Vec<_> = def
            .trails
            .iter()
            .filter(|t| self.node_at(&at(&t.a)).is_some() && self.node_at(&at(&t.b)).is_some())
            .collect();
        let has_explored_neighbor = |node: &NodeId| {
            open_trails.iter().any(|t| {
                self.trail_open(t)
                    && ((&t.a == node && explored(&t.b)) || (&t.b == node && explored(&t.a)))
            })
        };

        let nodes: Vec<NodeView> = def
            .nodes
            .iter()
            .filter(|&(node_id, _)| self.node_at(&at(node_id)).is_some())
            .filter_map(|(node_id, node)| {
                let fog = self.fog_of(&at(node_id), has_explored_neighbor(node_id))?;
                let shown = fog == Fog::Explored;
                Some(NodeView {
                    id: node_id.clone(),
                    x: node.x,
                    y: node.y,
                    fog,
                    kind: shown.then(|| self.kind_view(&node.kind)),
                    label: node.label.clone().filter(|_| shown),
                })
            })
            .collect();
        let visible = |node: &NodeId| nodes.iter().any(|n| &n.id == node);
        let trails = open_trails
            .iter()
            .filter(|t| visible(&t.a) && visible(&t.b) && (explored(&t.a) || explored(&t.b)))
            .map(|t| TrailView {
                a: t.a.clone(),
                b: t.b.clone(),
                open: self.trail_open(t),
            })
            .collect();
        InstanceView {
            id: id.clone(),
            region: def.region.clone(),
            terrain: def.terrain,
            nodes,
            trails,
        }
    }

    fn kind_view(&self, kind: &NodeKind) -> NodeKindView {
        match kind {
            NodeKind::Path => NodeKindView::Path,
            NodeKind::Exit { to } => NodeKindView::Exit {
                to: to.instance.clone(),
            },
            NodeKind::Battle { flag, boss, .. } => NodeKindView::Battle {
                boss: *boss,
                done: self.progress.has(flag),
            },
            NodeKind::Chest { flag, .. } => NodeKindView::Chest {
                done: self.progress.has(flag),
            },
            NodeKind::Npc { name, .. } => NodeKindView::Npc { name: name.clone() },
            NodeKind::Shop { .. } => NodeKindView::Shop,
            NodeKind::Temple => NodeKindView::Temple,
            NodeKind::Inn { price } => NodeKindView::Inn { price: *price },
            NodeKind::Board { .. } => NodeKindView::Board,
            NodeKind::Sealed => NodeKindView::Sealed,
        }
    }

    fn region_views(&self) -> Vec<RegionView> {
        let current = self
            .world
            .map
            .instances
            .get(&self.save.position.instance)
            .map(|def| def.region.clone());
        self.world
            .map
            .regions
            .iter()
            .filter(|&(id, _)| {
                self.progress.explored.iter().any(|position| {
                    self.world
                        .map
                        .instances
                        .get(&position.instance)
                        .is_some_and(|def| &def.region == id)
                })
            })
            .map(|(id, def)| RegionView {
                id: id.clone(),
                shape: def.shape.clone(),
                label: def.label,
                current: current.as_ref() == Some(id),
            })
            .collect()
    }
}
