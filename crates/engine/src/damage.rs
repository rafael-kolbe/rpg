//! Damage and healing formulas (FF7 structure, see `PLAN.md` §3 "Dano").
//!
//! Pure integer functions. Multipliers are basis points: 10 000 = 100%.
//! [`crate::effects`] decides which of them apply to a given hit.

/// The damage cap.
pub(crate) const MAX_DAMAGE: u64 = 9999;
/// 100% in basis points.
pub(crate) const FULL_BP: u64 = 10_000;
/// Base critical multiplier: 130%.
pub(crate) const CRITICAL_BP: u64 = 13_000;
/// Distracted multiplier, applied last: 150%.
pub(crate) const DISTRACTED_BP: u64 = 15_000;
/// Weak to the element: 200%.
pub(crate) const WEAK_BP: u64 = 20_000;
/// Halving: elemental resistance, Defend, Barrier, Magic Barrier.
pub(crate) const HALF_BP: u64 = 5_000;

/// Physical base: `Attack + ⌊(Attack + Level) / 32⌋ × ⌊(Attack × Level) / 32⌋`.
pub(crate) fn physical_base(attack: u16, level: u8) -> u64 {
    let attack = u64::from(attack);
    let level = u64::from(level);
    attack + (attack + level) / 32 * (attack * level / 32)
}

/// Magical base: `6 × (Magic Attack + Level)`.
pub(crate) fn magical_base(magic_attack: u16, level: u8) -> u64 {
    6 * (u64::from(magic_attack) + u64::from(level))
}

/// Defense reduction: `Power × (512 − Defense) × base / (16 × 512)`.
pub(crate) fn after_defense(power: u16, defense: u16, base: u64) -> u64 {
    let remaining = 512 - u64::from(defense.min(511));
    u64::from(power) * remaining * base / (16 * 512)
}

/// Random variance: `× (3841 + roll) / 4096`, with `roll` in `0..=255`
/// (between 93.8% and 100%).
pub(crate) fn with_variance(value: u64, roll: u64) -> u64 {
    value * (3841 + roll.min(255)) / 4096
}

/// Healing: `Power × 22 + (Level + Magic) × 6`. Not reduced by any defense.
pub(crate) fn heal(power: u16, level: u8, magic: u16) -> u64 {
    u64::from(power) * 22 + (u64::from(level) + u64::from(magic)) * 6
}

/// Multiplies by a basis-point factor.
pub(crate) fn scale(value: u64, bp: u64) -> u64 {
    value * bp / FULL_BP
}

/// Critical chance in percent:
/// `(Luck + Level − target Level) / 4 + equipment bonus`, clamped to 0..=100.
pub(crate) fn critical_chance(luck: u16, level: u8, target_level: u8, bonus: u8) -> u8 {
    let raw = (i32::from(luck) + i32::from(level) - i32::from(target_level)) / 4 + i32::from(bonus);
    u8::try_from(raw.clamp(0, 100)).unwrap_or(100)
}

/// Lowest hit chance of a physical attack: never impossible.
const MIN_HIT_PERCENT: i32 = 25;
/// Lowest chance of a hostile status taking hold.
const MIN_STATUS_PERCENT: i32 = 5;

/// Physical hit chance in percent:
/// `weapon accuracy + (attacker Dexterity − target Dexterity) / 4 − evasion`,
/// clamped to 25..=100.
pub(crate) fn hit_chance(
    weapon_accuracy: u8,
    attacker_dex: u16,
    target_dex: u16,
    evasion: u8,
) -> u8 {
    let raw = i32::from(weapon_accuracy) + (i32::from(attacker_dex) - i32::from(target_dex)) / 4
        - i32::from(evasion);
    u8::try_from(raw.clamp(MIN_HIT_PERCENT, 100)).unwrap_or(100)
}

/// Chance of a hostile status taking hold, in percent:
/// `status accuracy − target Luck / 4`, clamped to 5..=100.
pub(crate) fn status_chance(accuracy: u8, target_luck: u16) -> u8 {
    let raw = i32::from(accuracy) - i32::from(target_luck) / 4;
    u8::try_from(raw.clamp(MIN_STATUS_PERCENT, 100)).unwrap_or(100)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_base_matches_the_formula() {
        // 200 + ⌊299 / 32⌋ × ⌊19800 / 32⌋ = 200 + 9 × 618
        assert_eq!(physical_base(200, 99), 5762);
        // Low level: 20 + ⌊30 / 32⌋ × … = 20
        assert_eq!(physical_base(20, 10), 20);
    }

    #[test]
    fn magical_base_matches_the_formula() {
        assert_eq!(magical_base(40, 10), 300);
    }

    #[test]
    fn defense_scales_from_full_damage_to_almost_nothing() {
        // Basic attack (power 16) against Defense 0 keeps the whole base.
        assert_eq!(after_defense(16, 0, 5762), 5762);
        // Defense 100: 16 × 412 × 5762 / 8192
        assert_eq!(after_defense(16, 100, 5762), 4636);
        // Defense 511: only 1/512 remains.
        assert_eq!(after_defense(16, 511, 5762), 11);
    }

    #[test]
    fn variance_stays_between_93_8_and_100_percent() {
        assert_eq!(with_variance(4096, 0), 3841);
        assert_eq!(with_variance(4096, 255), 4096);
    }

    #[test]
    fn heal_matches_the_formula() {
        assert_eq!(heal(10, 20, 30), 10 * 22 + 50 * 6);
    }

    #[test]
    fn critical_then_distracted_multiply() {
        let critical = scale(1000, CRITICAL_BP);
        assert_eq!(scale(critical, DISTRACTED_BP), 1950);
    }

    #[test]
    fn hit_chance_compares_dexterity_and_subtracts_evasion() {
        assert_eq!(hit_chance(95, 50, 50, 0), 95);
        assert_eq!(hit_chance(95, 50, 90, 5), 80);
    }

    #[test]
    fn hit_chance_never_drops_below_25_or_exceeds_100() {
        assert_eq!(hit_chance(100, 120, 40, 0), 100);
        assert_eq!(hit_chance(80, 1, 255, 50), 25);
    }

    #[test]
    fn status_chance_is_resisted_by_luck() {
        assert_eq!(status_chance(70, 20), 65);
        assert_eq!(status_chance(70, 100), 45);
        assert_eq!(status_chance(10, 255), 5);
    }

    #[test]
    fn critical_chance_follows_luck_and_level_gap() {
        assert_eq!(critical_chance(20, 10, 10, 0), 5);
        assert_eq!(critical_chance(20, 10, 10, 3), 8);
        // A much stronger target can push it to zero, never below.
        assert_eq!(critical_chance(4, 1, 99, 0), 0);
        assert_eq!(critical_chance(255, 99, 1, 100), 100);
    }
}
