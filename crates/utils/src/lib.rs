//! Generic helpers shared by every crate.
//!
//! What belongs here: small, pure functions that know nothing about the game
//! (no combatants, items, ticks) and are useful in more than one place, or are
//! a well-known general operation. Game rules stay in their own crate, even when
//! they are short.
//!
//! Rules (see `CLAUDE.md`, "Utils"):
//! - search here before writing a helper; extend instead of duplicating;
//! - one module per topic, descriptive names;
//! - every public function documents what it does, how, and has an
//!   `# Examples` section (compiled and run by `cargo test`).
//!
//! | Module | Topic |
//! | --- | --- |
//! | [`numbers`] | Clamping and saturating integer conversions |
//! | [`text`] | String format checks |

pub mod numbers;
pub mod text;
