//! ATB gauge timing.

/// Duration of one battle tick, in milliseconds.
pub const TICK_MS: u32 = 100;

/// A full ATB gauge, in basis points.
pub const ATB_FULL: u32 = 10_000;

/// Gauge gained per tick at Dexterity 50 with no status: 2.5%, a full gauge
/// every 4 s.
const BASE_GAIN: u32 = 250;

/// How statuses change the gauge speed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Speed {
    Normal,
    Haste,
    Slow,
    Stopped,
}

/// Gauge gained in one tick: `BASE × (Dexterity + 50) / 100 × speed`.
pub(crate) fn gain_per_tick(dexterity: u16, speed: Speed) -> u32 {
    let normal = BASE_GAIN * (u32::from(dexterity) + 50) / 100;
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
    fn dexterity_50_fills_the_gauge_in_4_seconds() {
        assert_eq!(ticks_to_fill(50, Speed::Normal) * TICK_MS, 4000);
    }

    #[test]
    fn dexterity_100_fills_the_gauge_in_about_2_7_seconds() {
        assert_eq!(ticks_to_fill(100, Speed::Normal), 27);
    }

    #[test]
    fn haste_doubles_and_slow_halves_the_speed() {
        assert_eq!(ticks_to_fill(50, Speed::Haste), 20);
        assert_eq!(ticks_to_fill(50, Speed::Slow), 80);
    }

    #[test]
    fn a_stopped_gauge_does_not_move() {
        assert_eq!(gain_per_tick(255, Speed::Stopped), 0);
    }
}
