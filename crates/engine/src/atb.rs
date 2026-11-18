//! ATB gauge timing.

/// Duration of one battle tick, in milliseconds.
pub const TICK_MS: u32 = 100;

/// A full ATB gauge, in basis points.
pub const ATB_FULL: u32 = 10_000;

/// Gauge gained per tick at Dexterity 50 with no status: 1.25%, a full gauge
/// every 8 s. Slow enough to think between turns.
const BASE_GAIN: u32 = 125;

/// Highest Dexterity that still speeds the gauge up: a full gauge every 4 s.
const MAX_DEXTERITY: u32 = 255;

/// How statuses change the gauge speed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Speed {
    Normal,
    Haste,
    Slow,
    Stopped,
}

/// Gauge gained in one tick: `BASE × (Dexterity + 155) / 205 × speed`, with
/// Dexterity capped at 255. The line goes through 8 s at Dexterity 50 and
/// 4 s at 255, so speed grows with Dexterity but never past twice the base.
pub(crate) fn gain_per_tick(dexterity: u16, speed: Speed) -> u32 {
    let dexterity = u32::from(dexterity).min(MAX_DEXTERITY);
    let normal = BASE_GAIN * (dexterity + 155) / 205;
    match speed {
        Speed::Normal => normal,
        Speed::Haste => normal * 2,
        Speed::Slow => normal / 2,
        Speed::Stopped => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ticks_to_fill(dexterity: u16, speed: Speed) -> u32 {
        ATB_FULL.div_ceil(gain_per_tick(dexterity, speed))
    }

    #[test]
    fn dexterity_50_fills_the_gauge_in_8_seconds() {
        assert_eq!(ticks_to_fill(50, Speed::Normal) * TICK_MS, 8000);
    }

    #[test]
    fn dexterity_255_fills_the_gauge_in_4_seconds() {
        assert_eq!(ticks_to_fill(255, Speed::Normal) * TICK_MS, 4000);
    }

    #[test]
    fn dexterity_above_255_is_no_faster() {
        assert_eq!(
            gain_per_tick(999, Speed::Normal),
            gain_per_tick(255, Speed::Normal)
        );
    }

    #[test]
    fn dexterity_100_fills_the_gauge_in_about_6_5_seconds() {
        assert_eq!(ticks_to_fill(100, Speed::Normal), 65);
    }

    #[test]
    fn dexterity_1_fills_the_gauge_in_about_10_6_seconds() {
        assert_eq!(ticks_to_fill(1, Speed::Normal), 106);
    }

    #[test]
    fn haste_doubles_and_slow_halves_the_speed() {
        assert_eq!(ticks_to_fill(50, Speed::Haste), 40);
        assert_eq!(ticks_to_fill(50, Speed::Slow), 162); // 62.5 per tick, truncated
    }

    #[test]
    fn a_stopped_gauge_does_not_move() {
        assert_eq!(gain_per_tick(255, Speed::Stopped), 0);
    }
}
