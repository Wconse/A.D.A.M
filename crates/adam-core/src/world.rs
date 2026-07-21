use std::collections::BTreeMap;
use std::fmt;

use crate::{CountryId, DomainEvent, EventLog, SimDate, TimeError, WorldSeed};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Country {
    id: CountryId,
    name: String,
}

impl Country {
    /// Creates a country with a stable identity.
    ///
    /// # Errors
    ///
    /// Returns [`WorldError::EmptyCountryName`] when the name is empty or whitespace.
    pub fn new(id: CountryId, name: impl Into<String>) -> Result<Self, WorldError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(WorldError::EmptyCountryName);
        }
        Ok(Self { id, name })
    }

    #[must_use]
    pub const fn id(&self) -> CountryId {
        self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldError {
    DuplicateCountry(CountryId),
    EmptyCountryName,
    Time(TimeError),
}

impl fmt::Display for WorldError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateCountry(id) => write!(formatter, "country {id} already exists"),
            Self::EmptyCountryName => formatter.write_str("country name cannot be empty"),
            Self::Time(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for WorldError {}

impl From<TimeError> for WorldError {
    fn from(value: TimeError) -> Self {
        Self::Time(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct World {
    seed: WorldSeed,
    date: SimDate,
    countries: BTreeMap<CountryId, Country>,
    events: EventLog,
}

impl World {
    /// Creates an empty world and records its founding event.
    #[must_use]
    pub fn new(seed: WorldSeed, start_date: SimDate) -> Self {
        let mut events = EventLog::default();
        events.append(start_date, DomainEvent::WorldFounded { seed: seed.get() });
        Self {
            seed,
            date: start_date,
            countries: BTreeMap::new(),
            events,
        }
    }

    /// Adds a country and records the successful registration.
    ///
    /// # Errors
    ///
    /// Returns [`WorldError::DuplicateCountry`] when the ID is already registered.
    pub fn register_country(&mut self, country: Country) -> Result<(), WorldError> {
        if self.countries.contains_key(&country.id()) {
            return Err(WorldError::DuplicateCountry(country.id()));
        }
        self.events.append(
            self.date,
            DomainEvent::CountryRegistered {
                country: country.id(),
                name: country.name().to_owned(),
            },
        );
        self.countries.insert(country.id(), country);
        Ok(())
    }

    /// Advances the world clock and records one event for each completed year.
    ///
    /// # Errors
    ///
    /// Returns [`WorldError::Time`] if the simulation date overflows.
    pub fn advance_years(&mut self, years: u32) -> Result<(), WorldError> {
        for _ in 0..years {
            self.date.advance_years(1)?;
            self.events.append(
                self.date,
                DomainEvent::YearAdvanced {
                    year: self.date.year(),
                },
            );
        }
        Ok(())
    }

    #[must_use]
    pub const fn seed(&self) -> WorldSeed {
        self.seed
    }

    #[must_use]
    pub const fn date(&self) -> SimDate {
        self.date
    }

    #[must_use]
    pub fn countries(&self) -> &BTreeMap<CountryId, Country> {
        &self.countries
    }

    #[must_use]
    pub const fn events(&self) -> &EventLog {
        &self.events
    }

    #[must_use]
    pub fn stable_fingerprint(&self) -> u64 {
        let mut hash = StableHasher::new();
        hash.write_u32(crate::SIMULATION_VERSION);
        hash.write_u64(self.seed.get());
        hash.write_i32(self.date.year());
        hash.write_u16(self.date.day_of_year());
        hash.write_u64(self.countries.len() as u64);
        for (id, country) in &self.countries {
            hash.write_u64(id.get());
            hash.write_str(country.name());
        }
        hash.finish()
    }
}

#[derive(Clone, Copy, Debug)]
struct StableHasher(u64);

impl StableHasher {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    const fn new() -> Self {
        Self(Self::OFFSET)
    }

    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(Self::PRIME);
        }
    }

    fn write_u16(&mut self, value: u16) {
        self.write(&value.to_le_bytes());
    }

    fn write_u32(&mut self, value: u32) {
        self.write(&value.to_le_bytes());
    }

    fn write_i32(&mut self, value: i32) {
        self.write(&value.to_le_bytes());
    }

    fn write_u64(&mut self, value: u64) {
        self.write(&value.to_le_bytes());
    }

    fn write_str(&mut self, value: &str) {
        self.write_u64(value.len() as u64);
        self.write(value.as_bytes());
    }

    const fn finish(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn country_order_does_not_change_fingerprint() {
        let build = |reverse: bool| {
            let mut world = World::new(
                WorldSeed::new(1),
                SimDate::new(2025, 1).expect("valid date"),
            );
            let countries = if reverse {
                [(2, "B"), (1, "A")]
            } else {
                [(1, "A"), (2, "B")]
            };
            for (id, name) in countries {
                world
                    .register_country(
                        Country::new(CountryId::new(id), name).expect("valid country"),
                    )
                    .expect("unique country");
            }
            world
        };

        assert_eq!(
            build(false).stable_fingerprint(),
            build(true).stable_fingerprint()
        );
    }

    #[test]
    fn duplicate_country_is_rejected_without_an_event() {
        let mut world = World::new(
            WorldSeed::new(1),
            SimDate::new(2025, 1).expect("valid date"),
        );
        world
            .register_country(Country::new(CountryId::new(1), "A").expect("valid country"))
            .expect("first insert succeeds");
        let event_count = world.events().len();
        let error = world
            .register_country(Country::new(CountryId::new(1), "B").expect("valid country"))
            .expect_err("duplicate must fail");
        assert_eq!(error, WorldError::DuplicateCountry(CountryId::new(1)));
        assert_eq!(world.events().len(), event_count);
    }
}
