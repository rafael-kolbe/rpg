//! Clamping and saturating integer conversions.
//!
//! `as` casts silently wrap or truncate; these helpers make the intended
//! behavior (clamp or saturate) explicit and impossible to get wrong.

/// Clamps `value` into `min..=max`, then converts it to `u8`.
///
/// Use it for percentages or small counters computed with signed arithmetic
/// (where intermediate values may go negative or above the range).
///
/// # Examples
///
/// ```
/// use utils::numbers::clamp_to_u8;
///
/// // A chance computed as 95 + bonus − penalty, kept within 25..=100.
/// assert_eq!(clamp_to_u8(140, 25, 100), 100);
/// assert_eq!(clamp_to_u8(-30, 25, 100), 25);
/// assert_eq!(clamp_to_u8(80, 25, 100), 80);
/// ```
#[must_use]
pub fn clamp_to_u8(value: i32, min: u8, max: u8) -> u8 {
    let clamped = value.clamp(i32::from(min), i32::from(max));
    u8::try_from(clamped).unwrap_or(max)
}

/// Converts to `u16`, saturating at `u16::MAX` instead of wrapping.
///
/// Use it when a wide intermediate (`u64` arithmetic) must land in a `u16`
/// field and anything too large simply means "as much as possible".
///
/// # Examples
///
/// ```
/// use utils::numbers::saturating_u16;
///
/// assert_eq!(saturating_u16(42), 42);
/// assert_eq!(saturating_u16(1_000_000), u16::MAX);
/// ```
#[must_use]
pub fn saturating_u16(value: u64) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
}
