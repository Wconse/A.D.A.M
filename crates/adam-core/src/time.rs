use core::fmt;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
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
            Self::InvalidDayOfYear(day) => write!(formatter, "day of year must be 1..=365, got {day}"),
            Self::YearOverflow => formatter.write_str("simulation year overflow"),
        }
    }
}

impl std::error::Error for TimeError {}

impl SimDate {
    pub const DAYS_PER_YEAR: u16 = 365;

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

    pub fn advance_one_day(&mut self) -> Result<(), TimeError> {
        if self.day_of_year == Self::DAYS_PER_YEAR {
            self.year = self.year.checked_add(1).ok_or(TimeError::YearOverflow)?;
            self.day_of_year = 1;
        } else {
            self.day_of_year += 1;
        }
        Ok(())
    }

    pub fn advance_years(&mut self, years: u32) -> Result<(), TimeError> {
        let years = i32::try_from(years).map_err(|_| TimeError::YearOverflow)?;
        self.year = self.year.checked_add(years).ok_or(TimeError::YearOverflow)?;
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
    fn rejects_invalid_days() {
        assert_eq!(SimDate::new(2025, 0), Err(TimeError::InvalidDayOfYear(0)));
        assert_eq!(SimDate::new(2025, 366), Err(TimeError::InvalidDayOfYear(366)));
    }
}
