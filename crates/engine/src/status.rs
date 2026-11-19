//! Status conditions and their timers.

use serde::{Deserialize, Serialize};

/// Ticks between two Poison or Regen pulses (3 s).
pub(crate) const PULSE_INTERVAL_TICKS: u16 = 30;
/// Share of max HP lost to Poison, or restored by Regen, per pulse.
pub(crate) const PULSE_PERCENT: u16 = 5;
/// Chance that Cover takes a single-target attack meant for an ally, in
/// percent.
pub(crate) const COVER_PERCENT: u8 = 70;

/// A status condition.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatusKind {
    /// Loses 5% of max HP every 3 s.
    Poison,
    /// Recovers 5% of max HP every 3 s.
    Regen,
    /// ATB gauge stops until the combatant takes damage or the status ends.
    Sleep,
    /// Cannot use Magic.
    Silence,
    /// ATB gauge fills twice as fast. Replaces Slow.
    Haste,
    /// ATB gauge fills at half speed. Replaces Haste.
    Slow,
    /// Physical damage taken is halved.
    Barrier,
    /// Magical damage taken is halved.
    MagicBarrier,
    /// Turned away: the next hit taken deals 150%. Ends on that hit or when
    /// the combatant's own ATB gauge fills.
    Distracted,
    /// Guards the allies: a single-target attack on one of them has a 70%
    /// chance to hit this combatant instead.
    Cover,
}

impl StatusKind {
    /// Whether the status hurts whoever has it (shown with its time left).
    #[must_use]
    pub fn is_harmful(self) -> bool {
        match self {
            Self::Poison | Self::Sleep | Self::Silence | Self::Slow | Self::Distracted => true,
            Self::Regen | Self::Haste | Self::Barrier | Self::MagicBarrier | Self::Cover => false,
        }
    }

    /// Whether the status must be given a duration when applied.
    #[must_use]
    pub fn needs_duration(self) -> bool {
        self != Self::Distracted
    }
}

/// Time left on a timed status.
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusTimer {
    /// Ticks until it ends.
    pub remaining_ticks: u16,
    /// Ticks it lasted when applied (or last refreshed).
    pub duration_ticks: u16,
}

/// A status currently on a combatant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ActiveStatus {
    pub(crate) kind: StatusKind,
    /// `None` for statuses that end on an event rather than a timer.
    pub(crate) timer: Option<StatusTimer>,
    /// Ticks since the last Poison/Regen pulse.
    pub(crate) pulse_ticks: u16,
}

impl ActiveStatus {
    pub(crate) fn new(kind: StatusKind, duration_ticks: Option<u16>) -> Self {
        Self {
            kind,
            timer: duration_ticks.map(|ticks| StatusTimer {
                remaining_ticks: ticks,
                duration_ticks: ticks,
            }),
            pulse_ticks: 0,
        }
    }
}
