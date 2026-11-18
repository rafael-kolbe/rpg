//! Attributes and the stats derived from them (FF7 model, see `PLAN.md` §3).

use serde::{Deserialize, Serialize};

/// Highest level a character can reach.
pub(crate) const MAX_LEVEL: u8 = 99;
/// Highest value of a primary attribute.
pub(crate) const MAX_ATTRIBUTE: u16 = 255;
/// Highest max HP.
pub(crate) const MAX_HP: u16 = 9999;
/// Highest max MP.
pub(crate) const MAX_MP: u16 = 999;
/// Highest Defense / Magic Defense. 512 would mean immunity in the damage
/// formula, so it is never reachable.
pub(crate) const MAX_DEFENSE: u16 = 511;
/// Weapon accuracy when the equipment does not say otherwise.
pub(crate) const DEFAULT_WEAPON_ACCURACY: u8 = 95;

/// Primary attributes, each in `1..=255`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Attributes {
    /// Physical power: Attack = Strength + weapon.
    pub strength: u16,
    /// ATB speed, accuracy and evasion.
    pub dexterity: u16,
    /// Physical toughness: Defense = Vitality + armor.
    pub vitality: u16,
    /// Magic Attack and healing power.
    pub magic: u16,
    /// Magical toughness: Magic Defense = Spirit + armor.
    pub spirit: u16,
    /// Critical chance, status resistance, stealing.
    pub luck: u16,
}

/// What equipped gear adds to a combatant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct Equipment {
    /// Added to Strength to form Attack.
    pub weapon_attack: u16,
    /// Base hit chance of physical attacks, in percent (`1..=100`; 95 by
    /// default, light weapons higher, heavy ones lower).
    pub weapon_accuracy: u8,
    /// Chance subtracted from physical attacks aimed at the wearer, in
    /// percent (`0..=100`; armor and shields).
    pub evasion: u8,
    /// Added to Vitality to form Defense.
    pub armor_defense: u16,
    /// Added to Spirit to form Magic Defense.
    pub armor_magic_defense: u16,
    /// Extra critical chance, in percentage points.
    pub critical_chance: u8,
    /// Extra critical damage, in basis points on top of the 130% base
    /// (e.g. `1000` makes criticals deal 140%).
    pub critical_damage: u16,
}

impl Default for Equipment {
    fn default() -> Self {
        Self {
            weapon_attack: 0,
            weapon_accuracy: DEFAULT_WEAPON_ACCURACY,
            evasion: 0,
            armor_defense: 0,
            armor_magic_defense: 0,
            critical_chance: 0,
            critical_damage: 0,
        }
    }
}

/// Everything about a combatant that the damage and ATB formulas read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Stats {
    /// Level, `1..=99`. Enters the damage formulas.
    pub level: u8,
    /// Maximum HP, `1..=9999`.
    pub max_hp: u16,
    /// Maximum MP, `0..=999`.
    pub max_mp: u16,
    /// Primary attributes.
    pub attributes: Attributes,
    /// Equipped gear.
    #[serde(default)]
    pub equipment: Equipment,
}

impl Stats {
    /// Strength + weapon.
    #[must_use]
    pub fn attack(&self) -> u16 {
        self.attributes
            .strength
            .saturating_add(self.equipment.weapon_attack)
    }

    /// Vitality + armor, at most 511.
    #[must_use]
    pub fn defense(&self) -> u16 {
        self.attributes
            .vitality
            .saturating_add(self.equipment.armor_defense)
            .min(MAX_DEFENSE)
    }

    /// Magic.
    #[must_use]
    pub fn magic_attack(&self) -> u16 {
        self.attributes.magic
    }

    /// Spirit + armor, at most 511.
    #[must_use]
    pub fn magic_defense(&self) -> u16 {
        self.attributes
            .spirit
            .saturating_add(self.equipment.armor_magic_defense)
            .min(MAX_DEFENSE)
    }

    /// The first rule these stats break, if any.
    pub(crate) fn violation(&self) -> Option<String> {
        let a = &self.attributes;
        let attributes = [
            ("strength", a.strength),
            ("dexterity", a.dexterity),
            ("vitality", a.vitality),
            ("magic", a.magic),
            ("spirit", a.spirit),
            ("luck", a.luck),
        ];

        if !(1..=MAX_LEVEL).contains(&self.level) {
            return Some(format!("level must be 1..={MAX_LEVEL}, got {}", self.level));
        }
        if !(1..=MAX_HP).contains(&self.max_hp) {
            return Some(format!("maxHp must be 1..={MAX_HP}, got {}", self.max_hp));
        }
        if self.max_mp > MAX_MP {
            return Some(format!("maxMp must be 0..={MAX_MP}, got {}", self.max_mp));
        }
        let gear = &self.equipment;
        if !(1..=100).contains(&gear.weapon_accuracy) {
            let accuracy = gear.weapon_accuracy;
            return Some(format!("weaponAccuracy must be 1..=100, got {accuracy}"));
        }
        if gear.evasion > 100 {
            return Some(format!("evasion must be 0..=100, got {}", gear.evasion));
        }
        attributes
            .iter()
            .find(|(_, value)| !(1..=MAX_ATTRIBUTE).contains(value))
            .map(|(name, value)| format!("{name} must be 1..={MAX_ATTRIBUTE}, got {value}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::stats;

    #[test]
    fn derived_stats_add_equipment_to_attributes() {
        let mut s = stats(10, 50);
        s.attributes.strength = 30;
        s.attributes.vitality = 20;
        s.attributes.spirit = 25;
        s.equipment.weapon_attack = 12;
        s.equipment.armor_defense = 8;
        s.equipment.armor_magic_defense = 5;

        assert_eq!(s.attack(), 42);
        assert_eq!(s.defense(), 28);
        assert_eq!(s.magic_defense(), 30);
    }

    #[test]
    fn defense_never_reaches_immunity() {
        let mut s = stats(10, 50);
        s.attributes.vitality = 255;
        s.equipment.armor_defense = 400;

        assert_eq!(s.defense(), MAX_DEFENSE);
    }

    #[test]
    fn rejects_attributes_out_of_range() {
        let mut s = stats(10, 50);
        s.attributes.luck = 0;

        assert_eq!(s.violation().unwrap(), "luck must be 1..=255, got 0");
    }

    #[test]
    fn rejects_a_level_above_99() {
        let mut s = stats(10, 50);
        s.level = 100;

        assert!(s.violation().unwrap().starts_with("level"));
    }
}
