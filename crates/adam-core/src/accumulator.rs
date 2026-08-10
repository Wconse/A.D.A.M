//! Leaky accumulation of standing pressure.
//!
//! Several places in the simulation already needed the same shape: evidence
//! arrives every month, it builds up while it keeps arriving, it fades when it
//! stops, and at some level of accumulated evidence somebody finally acts.
//! Each of those places grew its own counter, and every one of them was
//! cruder than the thing it modelled - entry pressure counted months as bare
//! integers and threw the whole history away the first month a shortage eased,
//! so one good month erased a year of hunger, and a shortfall of one gram
//! counted exactly as much as a shortfall of a month's supply.
//!
//! This type keeps the same idea honestly. Charge is dimensionless: one full
//! month of complete, unrelieved shortage is [`FULL_MONTH_BPS`], and a month
//! that went half-supplied contributes half of it. What accumulates is
//! therefore severity over time rather than a tally of calendar months.
//!
//! All arithmetic is integer and saturating, so the value is reproducible and
//! safe to fold into the determinism fingerprint.

/// One month of complete, unrelieved shortage.
pub const FULL_MONTH_BPS: u32 = 10_000;

/// Accumulated evidence that has not yet been acted on.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct LeakyPressure {
    potential_bps: u32,
}

impl LeakyPressure {
    /// Restores a stored potential.
    #[must_use]
    pub const fn from_bps(potential_bps: u32) -> Self {
        Self { potential_bps }
    }

    /// The accumulated potential in basis points of a full month.
    #[must_use]
    pub const fn bps(self) -> u32 {
        self.potential_bps
    }

    /// The potential expressed in whole months of complete shortage, for
    /// reporting to observers that still speak in months.
    #[must_use]
    pub const fn months(self) -> u8 {
        let months = self.potential_bps / FULL_MONTH_BPS;
        if months > u8::MAX as u32 {
            u8::MAX
        } else {
            months as u8
        }
    }

    /// Adds this month's evidence, or lets older evidence fade if this month
    /// brought none.
    ///
    /// Evidence does not decay in the same month it arrives - a shortage seen
    /// today is not yet a memory - so a run of unrelieved months accumulates
    /// at full strength and the threshold keeps the plain meaning it had when
    /// pressure was counted in months.
    #[must_use]
    pub const fn accumulate(self, charge_bps: u32, leak_bps: u32) -> Self {
        if charge_bps > 0 {
            return Self {
                potential_bps: self.potential_bps.saturating_add(charge_bps),
            };
        }
        Self {
            potential_bps: self.potential_bps.saturating_sub(leak_bps),
        }
    }

    /// Whether the accumulated evidence has reached the level that justifies
    /// acting.
    #[must_use]
    pub const fn reached(self, threshold_bps: u32) -> bool {
        self.potential_bps >= threshold_bps
    }

    /// Spends the accumulated evidence once it has been acted on.
    #[must_use]
    pub const fn discharged() -> Self {
        Self { potential_bps: 0 }
    }
}

/// Converts a shortfall and the demand it fell short of into one month of
/// charge, saturating at a full month.
///
/// A shortage is measured against what was actually asked for, so severity is
/// comparable between a bakery short of grain and a household short of bread,
/// and neither the size of the region nor the units of the good leak into the
/// signal.
#[must_use]
pub fn shortage_charge_bps(shortfall: u64, demanded: u64) -> u32 {
    if shortfall == 0 || demanded == 0 {
        return 0;
    }
    let ratio =
        u128::from(shortfall).saturating_mul(u128::from(FULL_MONTH_BPS)) / u128::from(demanded);
    u32::try_from(ratio.min(u128::from(FULL_MONTH_BPS))).unwrap_or(FULL_MONTH_BPS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_half_supplied_month_charges_half_a_month() {
        assert_eq!(shortage_charge_bps(500, 1_000), FULL_MONTH_BPS / 2);
        assert_eq!(shortage_charge_bps(0, 1_000), 0);
        assert_eq!(shortage_charge_bps(2_000, 1_000), FULL_MONTH_BPS);
    }

    #[test]
    fn a_single_good_month_does_not_erase_a_year_of_evidence() {
        let mut pressure = LeakyPressure::default();
        for _ in 0..12 {
            pressure = pressure.accumulate(FULL_MONTH_BPS, 2_500);
        }
        let after_hunger = pressure;
        let after_relief = pressure.accumulate(0, 2_500);
        assert!(
            after_relief.bps() > 0,
            "one eased month must not wipe the record"
        );
        assert!(after_relief.bps() < after_hunger.bps());
    }

    #[test]
    fn evidence_that_stops_arriving_eventually_fades_to_nothing() {
        let mut pressure = LeakyPressure::from_bps(FULL_MONTH_BPS);
        for _ in 0..4 {
            pressure = pressure.accumulate(0, 2_500);
        }
        assert_eq!(pressure.bps(), 0);
    }
}
