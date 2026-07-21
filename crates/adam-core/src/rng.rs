#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldSeed(u64);

impl WorldSeed {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
    #[must_use]
    pub fn stream(self, domain: u64) -> RandomStream {
        RandomStream::new(mix64(self.0 ^ mix64(domain)))
    }

    /// Derives an isolated stream for one subsystem, entity, and simulation tick.
    #[must_use]
    pub fn stream_for(self, domain: u64, entity: u32, tick: i32) -> RandomStream {
        let key = mix64(domain)
            ^ mix64(u64::from(entity))
            ^ mix64(u64::from_ne_bytes(i64::from(tick).to_ne_bytes()));
        RandomStream::new(mix64(self.0 ^ key))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RandomStream {
    state: u64,
}

impl RandomStream {
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    #[must_use]
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        mix64(self.state)
    }
    #[must_use]
    pub fn next_bounded(&mut self, upper_exclusive: u64) -> Option<u64> {
        if upper_exclusive == 0 {
            return None;
        }
        let threshold = upper_exclusive.wrapping_neg() % upper_exclusive;
        loop {
            let value = self.next_u64();
            if value >= threshold {
                return Some(value % upper_exclusive);
            }
        }
    }
}

const fn mix64(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn same_key_repeats_exactly() {
        let seed = WorldSeed::new(47);
        assert_eq!(seed.stream_for(1, 2, 2025), seed.stream_for(1, 2, 2025));
    }
    #[test]
    fn subsystem_entity_and_tick_are_isolated() {
        let seed = WorldSeed::new(47);
        let first = seed.stream_for(1, 2, 2025).next_u64();
        assert_ne!(first, seed.stream_for(2, 2, 2025).next_u64());
        assert_ne!(first, seed.stream_for(1, 3, 2025).next_u64());
        assert_ne!(first, seed.stream_for(1, 2, 2026).next_u64());
    }
    #[test]
    fn bounded_values_stay_in_range() {
        let mut stream = RandomStream::new(1);
        for _ in 0..1_000 {
            assert!(stream.next_bounded(7).expect("bound") < 7);
        }
        assert_eq!(stream.next_bounded(0), None);
    }
}
