use core::fmt;

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    serde::Serialize,
    serde::Deserialize,
)]
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

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    serde::Serialize,
    serde::Deserialize,
)]
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

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub struct BasisPoints(u16);

impl BasisPoints {
    pub const MAX: u16 = 10_000;
    pub const HALF: Self = Self(5_000);

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

    #[must_use]
    pub fn shifted(self, delta: i32) -> Self {
        let value = (i32::from(self.0) + delta).clamp(0, i32::from(Self::MAX));
        Self(u16::try_from(value).unwrap_or(Self::MAX))
    }
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub struct RatePpm(i32);

impl RatePpm {
    pub const ONE: i32 = 1_000_000;

    /// Creates a signed fixed-point rate where 1,000,000 means +100%.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::RatePpmOutOfRange`] outside -100%..=+100%.
    pub const fn new(value: i32) -> Result<Self, ValueError> {
        if value >= -Self::ONE && value <= Self::ONE {
            Ok(Self(value))
        } else {
            Err(ValueError::RatePpmOutOfRange(value))
        }
    }

    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct QuantityMilli(u64);

impl QuantityMilli {
    pub const SCALE: u64 = 1_000;
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueError {
    BasisPointsOutOfRange(u16),
    RatePpmOutOfRange(i32),
}

impl fmt::Display for ValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BasisPointsOutOfRange(value) => {
                write!(formatter, "basis points must be 0..=10000, got {value}")
            }
            Self::RatePpmOutOfRange(value) => write!(
                formatter,
                "rate ppm must be -1000000..=1000000, got {value}"
            ),
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
        assert_eq!(size_of::<RatePpm>(), 4);
        assert_eq!(size_of::<QuantityMilli>(), 8);
    }

    #[test]
    fn bounded_values_reject_invalid_ranges() {
        assert_eq!(
            BasisPoints::new(10_001),
            Err(ValueError::BasisPointsOutOfRange(10_001))
        );
        assert_eq!(
            RatePpm::new(1_000_001),
            Err(ValueError::RatePpmOutOfRange(1_000_001))
        );
    }

    #[test]
    fn basis_point_shifts_saturate_at_physical_bounds() {
        assert_eq!(
            BasisPoints::new(9_900).expect("valid").shifted(500).get(),
            10_000
        );
        assert_eq!(BasisPoints::new(100).expect("valid").shifted(-500).get(), 0);
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
