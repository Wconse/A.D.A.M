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
    fn same_seed_and_domain_repeat_exactly() {
        let seed = WorldSeed::new(47);
        let mut left = seed.stream(11);
        let mut right = seed.stream(11);
        let left_values: Vec<_> = (0..16).map(|_| left.next_u64()).collect();
        let right_values: Vec<_> = (0..16).map(|_| right.next_u64()).collect();
        assert_eq!(left_values, right_values);
    }

    #[test]
    fn domains_are_isolated() {
        let seed = WorldSeed::new(47);
        assert_ne!(seed.stream(1).next_u64(), seed.stream(2).next_u64());
    }

    #[test]
    fn bounded_values_stay_in_range() {
        let mut stream = RandomStream::new(1);
        for _ in 0..1_000 {
            let value = stream.next_bounded(7).expect("non-zero bound");
            assert!(value < 7);
        }
        assert_eq!(stream.next_bounded(0), None);
    }
}
