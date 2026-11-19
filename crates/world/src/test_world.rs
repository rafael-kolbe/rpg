//! A tiny world for unit tests.
//!
//! `field`: `home` (temple) —100%— `road` — `gate` (fight: one slime; the
//! win sets `gate-won` and makes `bea` join) and `road` — `chest`,
//! `road` — `elder` (NPC), `road` — `shop`, `road` — `secret` (trail needs
//! `gate-won`), `road` — `stairs` (exit to `cave/entry`). `chest` plays the
//! `found-chest` scene on first arrival.

use std::collections::BTreeMap;
use std::sync::Arc;

use engine::{
    ActionCategory, ActionDef, ActionId, AiAct, AiPhase, AiProfile, AiRule, Attributes,
    CombatantSetup, Condition as AiCondition, Effect, ItemDef, ItemId, Stats, TargetPick,
    TargetRule,
};

use crate::World;
use crate::def::{
    ChoiceDef, ClassDef, Condition, DialogueDef, EncounterGroup, EquipmentDef, EventDef, HeroDef,
    InstanceDef, Learn, MonsterDef, NodeDef, NodeKind, Priced, RegionDef, ShopDef, Slot, StartDef,
    StatLine, StoryEffect, TalkDef, Terrain, TrailDef, WorldDef,
};
use crate::ids::{
    ClassId, DialogueId, EquipmentId, Flag, HeroId, InstanceId, MonsterId, NodeId, Position,
    RegionId, ShopId,
};

pub(crate) fn position(instance: &str, node: &str) -> Position {
    Position {
        instance: InstanceId::new(instance),
        node: NodeId::new(node),
    }
}

fn ai() -> AiProfile {
    AiProfile {
        phases: vec![AiPhase {
            from_hp_percent: 100,
            rules: vec![AiRule {
                when: AiCondition::Always,
                weight: 1,
                act: AiAct::Attack,
                target: TargetPick::RandomOpponent,
            }],
        }],
    }
}

fn node(x: u16, kind: NodeKind) -> NodeDef {
    NodeDef {
        x,
        y: 500,
        label: None,
        kind,
        when: Condition::default(),
        event: None,
    }
}

fn trail(a: &str, b: &str, chance: u8) -> TrailDef {
    TrailDef {
        a: NodeId::new(a),
        b: NodeId::new(b),
        chance,
        encounters: vec![EncounterGroup {
            weight: 1,
            monsters: vec![MonsterId::new("slime")],
        }],
        when: Condition::default(),
    }
}

fn flag(name: &str) -> Flag {
    Flag::new(name)
}

fn actions() -> BTreeMap<ActionId, ActionDef> {
    let attack = ActionId::new("attack");
    BTreeMap::from([
        (
            attack.clone(),
            ActionDef {
                category: ActionCategory::Attack,
                target: TargetRule::SingleEnemy,
                mp_cost: 0,
                duration_ticks: 8,
                effect: Effect::PhysicalDamage {
                    power: 16,
                    element: None,
                },
            },
        ),
        (
            ActionId::new("potion"),
            ActionDef {
                category: ActionCategory::Item,
                target: TargetRule::SingleAlly,
                mp_cost: 0,
                duration_ticks: 8,
                effect: Effect::RestoreHp { amount: 100 },
            },
        ),
    ])
}

fn class() -> ClassDef {
    let attack = ActionId::new("attack");
    let line = |hp, value| StatLine {
        max_hp: hp,
        max_mp: 10,
        strength: value,
        dexterity: value,
        vitality: value,
        magic: value,
        spirit: value,
        luck: value,
    };
    ClassDef {
        base: line(100, 10),
        growth: line(100, 10),
        attack: attack.clone(),
        learns: vec![Learn {
            level: 2,
            action: attack.clone(),
        }],
        ai: ai(),
    }
}

fn slime() -> MonsterDef {
    let attack = ActionId::new("attack");
    let potion = ItemId::new("potion");
    MonsterDef {
        battle: CombatantSetup {
            name: "monster-slime".to_owned(),
            stats: Stats {
                level: 1,
                max_hp: 10,
                max_mp: 0,
                attributes: Attributes {
                    strength: 1,
                    dexterity: 1,
                    vitality: 1,
                    magic: 1,
                    spirit: 1,
                    luck: 1,
                },
                equipment: engine::Equipment::default(),
            },
            attack: attack.clone(),
            actions: Vec::new(),
            affinities: BTreeMap::new(),
            status_immunities: Vec::new(),
            ai: Some(ai()),
            hp: None,
            mp: None,
        },
        xp: 25,
        crowns: 7,
        drops: vec![crate::def::Drop {
            item: potion.clone(),
            percent: 100,
        }],
    }
}

fn field() -> InstanceDef {
    let potion = ItemId::new("potion");
    let mut chest_node = node(
        600,
        NodeKind::Chest {
            flag: flag("chest-opened"),
            items: BTreeMap::from([(potion.clone(), 2)]),
            equipment: vec![EquipmentId::new("sword")],
            crowns: 50,
        },
    );
    chest_node.event = Some(EventDef {
        dialogue: DialogueId::new("found-chest"),
        when: Condition {
            requires: Vec::new(),
            forbids: vec![flag("chest-seen")],
        },
    });
    let mut secret_trail = trail("road", "secret", 0);
    secret_trail.when.requires = vec![flag("gate-won")];

    InstanceDef {
        region: RegionId::new("land"),
        terrain: Terrain::Fields,
        nodes: BTreeMap::from([
            (NodeId::new("home"), node(100, NodeKind::Temple)),
            (NodeId::new("road"), node(300, NodeKind::Path)),
            (
                NodeId::new("gate"),
                node(
                    500,
                    NodeKind::Battle {
                        monsters: vec![MonsterId::new("slime")],
                        flag: flag("gate-won"),
                        boss: false,
                        on_win: vec![StoryEffect::Join {
                            hero: HeroId::new("bea"),
                        }],
                    },
                ),
            ),
            (NodeId::new("chest"), chest_node),
            (
                NodeId::new("elder"),
                node(
                    700,
                    NodeKind::Npc {
                        name: "npc-elder".to_owned(),
                        talks: vec![TalkDef {
                            dialogue: DialogueId::new("elder"),
                            when: Condition::default(),
                        }],
                    },
                ),
            ),
            (
                NodeId::new("shop"),
                node(
                    800,
                    NodeKind::Shop {
                        shop: ShopId::new("store"),
                    },
                ),
            ),
            (NodeId::new("secret"), node(900, NodeKind::Path)),
            (
                NodeId::new("stairs"),
                node(
                    950,
                    NodeKind::Exit {
                        to: position("cave", "entry"),
                    },
                ),
            ),
        ]),
        trails: vec![
            trail("home", "road", 100),
            trail("road", "gate", 0),
            trail("road", "chest", 0),
            trail("road", "elder", 0),
            trail("road", "shop", 0),
            secret_trail,
            trail("road", "stairs", 0),
        ],
    }
}

fn cave() -> InstanceDef {
    InstanceDef {
        region: RegionId::new("land"),
        terrain: Terrain::Crypt,
        nodes: BTreeMap::from([(
            NodeId::new("entry"),
            node(
                100,
                NodeKind::Exit {
                    to: position("field", "stairs"),
                },
            ),
        )]),
        trails: Vec::new(),
    }
}

fn dialogues() -> BTreeMap<DialogueId, DialogueDef> {
    BTreeMap::from([
        (
            DialogueId::new("found-chest"),
            DialogueDef {
                lines: vec![None],
                choices: Vec::new(),
                effects: vec![StoryEffect::SetFlag {
                    flag: flag("chest-seen"),
                }],
            },
        ),
        (
            DialogueId::new("elder"),
            DialogueDef {
                lines: vec![Some("npc-elder".to_owned()), None],
                choices: vec![
                    ChoiceDef {
                        effects: vec![StoryEffect::Crowns { amount: 5 }],
                    },
                    ChoiceDef {
                        effects: vec![StoryEffect::Battle {
                            monsters: vec![MonsterId::new("slime")],
                            on_win: vec![StoryEffect::Battle {
                                monsters: vec![MonsterId::new("slime"), MonsterId::new("slime")],
                                on_win: Vec::new(),
                            }],
                        }],
                    },
                ],
                effects: Vec::new(),
            },
        ),
    ])
}

pub(crate) fn world() -> Arc<World> {
    let potion = ItemId::new("potion");
    let items = BTreeMap::from([(
        potion.clone(),
        ItemDef {
            action: ActionId::new("potion"),
        },
    )]);
    let (actions, class, slime, field, cave) = (actions(), class(), slime(), field(), cave());
    let map = WorldDef {
        start: StartDef {
            position: position("field", "home"),
            party: vec![HeroId::new("ann")],
            crowns: 100,
            inventory: BTreeMap::from([(potion.clone(), 1)]),
        },
        regions: BTreeMap::from([(
            RegionId::new("land"),
            RegionDef {
                shape: "M0 0Z".to_owned(),
                label: [500, 500],
            },
        )]),
        instances: BTreeMap::from([
            (InstanceId::new("field"), field),
            (InstanceId::new("cave"), cave),
        ]),
        classes: BTreeMap::from([(ClassId::new("fighter"), class)]),
        heroes: BTreeMap::from([
            (
                HeroId::new("ann"),
                HeroDef {
                    class: ClassId::new("fighter"),
                    weapon: None,
                    armor: None,
                },
            ),
            (
                HeroId::new("bea"),
                HeroDef {
                    class: ClassId::new("fighter"),
                    weapon: None,
                    armor: None,
                },
            ),
        ]),
        monsters: BTreeMap::from([(MonsterId::new("slime"), slime)]),
        dialogues: dialogues(),
        shops: BTreeMap::from([(
            ShopId::new("store"),
            ShopDef {
                items: vec![Priced {
                    id: potion,
                    price: 30,
                }],
                equipment: vec![Priced {
                    id: EquipmentId::new("sword"),
                    price: 60,
                }],
            },
        )]),
        equipment: BTreeMap::from([(
            EquipmentId::new("sword"),
            EquipmentDef {
                slot: Slot::Weapon,
                classes: vec![ClassId::new("fighter")],
                attack: 12,
                accuracy: None,
                critical_chance: 0,
                defense: 0,
                magic_defense: 0,
                evasion: 0,
            },
        )]),
    };
    Arc::new(World {
        map,
        actions,
        items,
    })
}
