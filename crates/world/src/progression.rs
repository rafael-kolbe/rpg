//! Levels and experience: what a hero's stats and actions are at a level,
//! and how experience turns into levels (`PLAN.md` §4, "Regras do arco").

use engine::{ActionId, Attributes, Equipment, Stats};

use crate::def::{ClassDef, EquipmentDef, StatLine};

/// Highest level.
pub const MAX_LEVEL: u8 = 99;

/// Experience needed to go from `level` to the next: `20 × level²`.
#[must_use]
pub fn xp_to_next(level: u8) -> u32 {
    20 * u32::from(level) * u32::from(level)
}

/// `base + growth × (level − 1) / 10`, capped at `max`.
fn grown(base: u16, growth_tenths: u16, level: u8, max: u16) -> u16 {
    let gained = u32::from(growth_tenths) * u32::from(level.saturating_sub(1)) / 10;
    u16::try_from(u32::from(base) + gained)
        .unwrap_or(max)
        .min(max)
}

/// A class's HP, MP and attributes at `level`.
#[must_use]
pub fn stat_line_at(class: &ClassDef, level: u8) -> StatLine {
    let (b, g) = (&class.base, &class.growth);
    StatLine {
        max_hp: grown(b.max_hp, g.max_hp, level, 9999),
        max_mp: grown(b.max_mp, g.max_mp, level, 999),
        strength: grown(b.strength, g.strength, level, 255),
        dexterity: grown(b.dexterity, g.dexterity, level, 255),
        vitality: grown(b.vitality, g.vitality, level, 255),
        magic: grown(b.magic, g.magic, level, 255),
        spirit: grown(b.spirit, g.spirit, level, 255),
        luck: grown(b.luck, g.luck, level, 255),
    }
}

/// What a weapon and an armor add, combined into the engine's gear.
#[must_use]
pub fn equipment_of(weapon: Option<&EquipmentDef>, armor: Option<&EquipmentDef>) -> Equipment {
    let mut gear = Equipment::default();
    for piece in [weapon, armor].into_iter().flatten() {
        gear.weapon_attack = gear.weapon_attack.saturating_add(piece.attack);
        if let Some(accuracy) = piece.accuracy {
            gear.weapon_accuracy = accuracy;
        }
        gear.critical_chance = gear.critical_chance.saturating_add(piece.critical_chance);
        gear.armor_defense = gear.armor_defense.saturating_add(piece.defense);
        gear.armor_magic_defense = gear.armor_magic_defense.saturating_add(piece.magic_defense);
        gear.evasion = gear.evasion.saturating_add(piece.evasion).min(100);
    }
    gear
}

/// Battle stats of a hero of `class` at `level` wearing `equipment`.
#[must_use]
pub fn hero_stats(class: &ClassDef, level: u8, equipment: Equipment) -> Stats {
    let line = stat_line_at(class, level);
    Stats {
        level,
        max_hp: line.max_hp.max(1),
        max_mp: line.max_mp,
        attributes: Attributes {
            strength: line.strength.max(1),
            dexterity: line.dexterity.max(1),
            vitality: line.vitality.max(1),
            magic: line.magic.max(1),
            spirit: line.spirit.max(1),
            luck: line.luck.max(1),
        },
        equipment,
    }
}

/// Technics and spells a hero of `class` knows at `level`, in learning order.
#[must_use]
pub fn known_actions(class: &ClassDef, level: u8) -> Vec<ActionId> {
    class
        .learns
        .iter()
        .filter(|learn| learn.level <= level)
        .map(|learn| learn.action.clone())
        .collect()
}

/// Adds `xp` to a hero at `level` with `progress` experience toward the next
/// level. Returns the new level and progress; experience stops at
/// [`MAX_LEVEL`].
#[must_use]
pub fn add_xp(level: u8, progress: u32, xp: u32) -> (u8, u32) {
    let (mut level, mut progress) = (level, progress.saturating_add(xp));
    while level < MAX_LEVEL && progress >= xp_to_next(level) {
        progress -= xp_to_next(level);
        level += 1;
    }
    if level == MAX_LEVEL {
        progress = 0;
    }
    (level, progress)
}

#[cfg(test)]
mod tests {
    use engine::AiProfile;

    use super::*;
    use crate::def::Learn;

    fn line(value: u16) -> StatLine {
        StatLine {
            max_hp: value,
            max_mp: value,
            strength: value,
            dexterity: value,
            vitality: value,
            magic: value,
            spirit: value,
            luck: value,
        }
    }

    fn class() -> ClassDef {
        ClassDef {
            base: line(10),
            growth: line(15),
            attack: ActionId::new("attack"),
            learns: vec![
                Learn {
                    level: 1,
                    action: ActionId::new("first"),
                },
                Learn {
                    level: 5,
                    action: ActionId::new("fifth"),
                },
            ],
            ai: AiProfile { phases: Vec::new() },
        }
    }

    #[test]
    fn the_next_level_costs_twenty_times_the_level_squared() {
        assert_eq!(xp_to_next(1), 20);
        assert_eq!(xp_to_next(10), 2000);
    }

    #[test]
    fn growth_in_tenths_adds_up_over_levels() {
        let stats = hero_stats(&class(), 11, Equipment::default());

        assert_eq!(stats.level, 11);
        assert_eq!(stats.attributes.strength, 25);
        assert_eq!(stats.max_hp, 25);
        assert_eq!(
            hero_stats(&class(), 99, Equipment::default())
                .attributes
                .luck,
            157
        );
    }

    #[test]
    fn attributes_stop_at_255() {
        let mut fast = class();
        fast.growth = line(100);

        assert_eq!(
            hero_stats(&fast, 99, Equipment::default()).attributes.luck,
            255
        );
    }

    #[test]
    fn actions_come_at_their_level() {
        assert_eq!(known_actions(&class(), 4), vec![ActionId::new("first")]);
        assert_eq!(known_actions(&class(), 5).len(), 2);
    }

    #[test]
    fn experience_can_cross_several_levels_at_once() {
        assert_eq!(add_xp(1, 10, 5), (1, 15));
        assert_eq!(add_xp(1, 10, 10), (2, 0));
        assert_eq!(add_xp(1, 0, 20 + 80 + 5), (3, 5));
        assert_eq!(add_xp(98, 0, u32::MAX), (99, 0));
    }

    #[test]
    fn weapon_and_armor_add_up() {
        let piece = |attack, defense| EquipmentDef {
            slot: crate::def::Slot::Weapon,
            classes: Vec::new(),
            attack,
            accuracy: Some(90),
            critical_chance: 2,
            defense,
            magic_defense: 1,
            evasion: 3,
        };

        let gear = equipment_of(Some(&piece(12, 0)), Some(&piece(0, 7)));

        assert_eq!(gear.weapon_attack, 12);
        assert_eq!(gear.armor_defense, 7);
        assert_eq!(gear.weapon_accuracy, 90);
        assert_eq!(gear.evasion, 6);
    }
}
