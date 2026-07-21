use core::fmt;

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Population(u64);

impl Population {
    #[must_use]
    pub const fn new(people: u64) -> Self {
        Self(people)
    }

    #[must_use]
    pub const fn people(self) -> u64 {
        self.0
    }

    #[must_use]
    pub const fn checked_add(self, other: Self) -> Option<Self> {
        match self.0.checked_add(other.0) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Money(i64);

impl Money {
    #[must_use]
    pub const fn from_minor_units(value: i64) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn minor_units(self) -> i64 {
        self.0
    }

    #[must_use]
    pub const fn checked_add(self, other: Self) -> Option<Self> {
        match self.0.checked_add(other.0) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BasisPoints(u16);

impl BasisPoints {
    pub const MAX: u16 = 10_000;

    /// Creates a fixed-point share where 10,000 means 100%.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::BasisPointsOutOfRange`] for values above 10,000.
    pub const fn new(value: u16) -> Result<Self, ValueError> {
        if value <= Self::MAX {
            Ok(Self(value))
        } else {
            Err(ValueError::BasisPointsOutOfRange(value))
        }
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueError {
    BasisPointsOutOfRange(u16),
}

impl fmt::Display for ValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BasisPointsOutOfRange(value) => {
                write!(formatter, "basis points must be 0..=10000, got {value}")
            }
        }
    }
}

impl std::error::Error for ValueError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_values_have_expected_layout() {
        assert_eq!(size_of::<Population>(), 8);
        assert_eq!(size_of::<Money>(), 8);
        assert_eq!(size_of::<BasisPoints>(), 2);
    }

    #[test]
    fn basis_points_reject_invalid_percentages() {
        assert_eq!(
            BasisPoints::new(10_000).expect("100% is valid").get(),
            10_000
        );
        assert_eq!(
            BasisPoints::new(10_001),
            Err(ValueError::BasisPointsOutOfRange(10_001))
        );
    }

    #[test]
    fn arithmetic_is_explicitly_checked() {
        assert_eq!(
            Population::new(u64::MAX).checked_add(Population::new(1)),
            None
        );
        assert_eq!(
            Money::from_minor_units(i64::MAX).checked_add(Money::from_minor_units(1)),
            None
        );
    }
}
