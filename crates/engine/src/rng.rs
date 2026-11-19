//! Seeded pseudo-random numbers.
//!
//! `SplitMix64`, implemented here instead of taken from a crate: battles must
//! replay identically forever, and a dependency update must never be able to
//! change the sequence. It is not cryptographic, and does not need to be.

/// Deterministic random number generator, owned by a battle (and by the
/// world rules, for encounters and drops). The whole sequence follows from
/// [`Rng::state`], so saving it resumes the same sequence.
#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// A generator starting from `seed` (or from a saved [`Rng::state`]).
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// The current state; [`Rng::new`] with it continues the sequence.
    #[must_use]
    pub fn state(&self) -> u64 {
        self.state
    }

    /// The next raw 64-bit value.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform value in `0..bound`.
    ///
    /// # Panics
    ///
    /// When `bound` is 0.
    ///
    /// Rejection sampling keeps the distribution exactly uniform (a plain
    /// modulo would favor small values).
    pub fn below(&mut self, bound: u64) -> u64 {
        debug_assert!(bound > 0, "Rng::below called with bound 0");
        let zone = u64::MAX - (u64::MAX % bound);
        loop {
            let value = self.next_u64();
            if value < zone {
                return value % bound;
            }
        }
    }

    /// A uniformly chosen element of `items`, or `None` when it is empty.
    pub fn pick<T: Copy>(&mut self, items: &[T]) -> Option<T> {
        if items.is_empty() {
            return None;
        }
        let index = self.below(items.len() as u64);
        items.get(usize::try_from(index).ok()?).copied()
    }

    /// `true` with probability `percent` / 100 (clamped to 0..=100).
    pub fn chance_percent(&mut self, percent: u8) -> bool {
        self.below(100) < u64::from(percent.min(100))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_gives_the_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);

        let first: Vec<u64> = (0..5).map(|_| a.next_u64()).collect();
        let second: Vec<u64> = (0..5).map(|_| b.next_u64()).collect();

        assert_eq!(first, second);
    }

    #[test]
    fn sequence_is_stable_across_versions() {
        // Pinned output: if this changes, every saved replay changes with it.
        let mut rng = Rng::new(0);

        assert_eq!(rng.next_u64(), 0xE220_A839_7B1D_CDAF);
        assert_eq!(rng.next_u64(), 0x6E78_9E6A_A1B9_65F4);
    }

    #[test]
    fn below_stays_in_range() {
        let mut rng = Rng::new(7);

        assert!((0..1000).all(|_| rng.below(3) < 3));
    }

    #[test]
    fn pick_returns_an_element_or_none() {
        let mut rng = Rng::new(7);

        assert_eq!(rng.pick::<u8>(&[]), None);
        assert!((0..100).all(|_| rng.pick(&[3, 5]).is_some_and(|x| x == 3 || x == 5)));
    }

    #[test]
    fn chance_percent_respects_the_extremes() {
        let mut rng = Rng::new(7);

        assert!((0..100).all(|_| !rng.chance_percent(0)));
        assert!((0..100).all(|_| rng.chance_percent(100)));
    }
}
