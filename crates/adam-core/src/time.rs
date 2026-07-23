use core::fmt;

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub struct SimDate {
    year: i32,
    day_of_year: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeError {
    InvalidDayOfYear(u16),
    YearOverflow,
}

impl fmt::Display for TimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDayOfYear(day) => {
                write!(formatter, "day of year must be 1..=365, got {day}")
            }
            Self::YearOverflow => formatter.write_str("simulation year overflow"),
        }
    }
}

impl std::error::Error for TimeError {}

impl SimDate {
    pub const DAYS_PER_YEAR: u16 = 365;
    pub const MONTH_LENGTHS: [u16; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    /// Creates a date in the simplified 365-day simulation calendar.
    ///
    /// # Errors
    ///
    /// Returns [`TimeError::InvalidDayOfYear`] when `day_of_year` is outside `1..=365`.
    pub fn new(year: i32, day_of_year: u16) -> Result<Self, TimeError> {
        if !(1..=Self::DAYS_PER_YEAR).contains(&day_of_year) {
            return Err(TimeError::InvalidDayOfYear(day_of_year));
        }
        Ok(Self { year, day_of_year })
    }

    #[must_use]
    pub const fn year(self) -> i32 {
        self.year
    }

    #[must_use]
    pub const fn day_of_year(self) -> u16 {
        self.day_of_year
    }

    #[must_use]
    pub fn month(self) -> u8 {
        let mut remaining = self.day_of_year;
        for (index, length) in Self::MONTH_LENGTHS.into_iter().enumerate() {
            if remaining <= length {
                return u8::try_from(index + 1).unwrap_or(12);
            }
            remaining -= length;
        }
        12
    }

    #[must_use]
    pub fn day_of_month(self) -> u8 {
        let mut remaining = self.day_of_year;
        for length in Self::MONTH_LENGTHS {
            if remaining <= length {
                return u8::try_from(remaining).unwrap_or(31);
            }
            remaining -= length;
        }
        31
    }

    /// Advances by one calendar month, clamping the day to the target month length.
    /// # Errors
    /// Returns [`TimeError::YearOverflow`] when December crosses an overflowing year.
    pub fn advance_one_month(&mut self) -> Result<(), TimeError> {
        let month = self.month();
        let day = u16::from(self.day_of_month());
        if month == 12 {
            self.year = self.year.checked_add(1).ok_or(TimeError::YearOverflow)?;
            self.day_of_year = day.min(Self::MONTH_LENGTHS[0]);
            return Ok(());
        }
        let target_index = usize::from(month);
        let target_day = day.min(Self::MONTH_LENGTHS[target_index]);
        let preceding: u16 = Self::MONTH_LENGTHS[..target_index].iter().sum();
        self.day_of_year = preceding + target_day;
        Ok(())
    }

    /// Advances the calendar by exactly one simulation day.
    ///
    /// # Errors
    ///
    /// Returns [`TimeError::YearOverflow`] if crossing the year boundary would overflow.
    pub fn advance_one_day(&mut self) -> Result<(), TimeError> {
        if self.day_of_year == Self::DAYS_PER_YEAR {
            self.year = self.year.checked_add(1).ok_or(TimeError::YearOverflow)?;
            self.day_of_year = 1;
        } else {
            self.day_of_year += 1;
        }
        Ok(())
    }

    /// Advances the calendar by whole years without changing the day of year.
    ///
    /// # Errors
    ///
    /// Returns [`TimeError::YearOverflow`] if the resulting year cannot be represented.
    pub fn advance_years(&mut self, years: u32) -> Result<(), TimeError> {
        let years = i32::try_from(years).map_err(|_| TimeError::YearOverflow)?;
        self.year = self
            .year
            .checked_add(years)
            .ok_or(TimeError::YearOverflow)?;
        Ok(())
    }
}

impl fmt::Display for SimDate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}-{:03}", self.year, self.day_of_year)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn year_boundary_is_explicit_and_stable() {
        let mut date = SimDate::new(2025, 365).expect("valid date");
        date.advance_one_day().expect("year can advance");
        assert_eq!(date, SimDate::new(2026, 1).expect("valid date"));
    }

    #[test]
    fn monthly_advance_uses_calendar_boundaries_and_clamps() {
        let mut start = SimDate::new(2025, 1).expect("date");
        start.advance_one_month().expect("month");
        assert_eq!(start, SimDate::new(2025, 32).expect("date"));
        assert_eq!(start.month(), 2);
        assert_eq!(start.day_of_month(), 1);

        let mut january_end = SimDate::new(2025, 31).expect("date");
        january_end.advance_one_month().expect("month");
        assert_eq!(january_end, SimDate::new(2025, 59).expect("date"));
        assert_eq!(january_end.day_of_month(), 28);
    }

    #[test]
    fn twelve_months_cross_exactly_one_year() {
        let mut date = SimDate::new(2025, 1).expect("date");
        for _ in 0..12 {
            date.advance_one_month().expect("month");
        }
        assert_eq!(date, SimDate::new(2026, 1).expect("date"));
    }

    #[test]
    fn rejects_invalid_days() {
        assert_eq!(SimDate::new(2025, 0), Err(TimeError::InvalidDayOfYear(0)));
        assert_eq!(
            SimDate::new(2025, 366),
            Err(TimeError::InvalidDayOfYear(366))
        );
    }
}
