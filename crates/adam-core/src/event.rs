use crate::{
    ActorId, BasisPoints, CohortId, CountryId, Money, Population, PowerNodeId, RatePpm, RegionId,
    SimDate,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainEvent {
    WorldFounded {
        seed: u64,
    },
    CountryRegistered {
        country: CountryId,
        name: String,
    },
    RegionRegistered {
        region: RegionId,
        country: CountryId,
        name: String,
    },
    ActorRegistered {
        actor: ActorId,
        home_region: RegionId,
        name: String,
    },
    PowerNodeRegistered {
        node: PowerNodeId,
        country: CountryId,
        name: String,
    },
    InfluenceEstablished {
        actor: ActorId,
        node: PowerNodeId,
        weight: BasisPoints,
    },
    HouseholdCohortRegistered {
        cohort: CohortId,
        region: RegionId,
        people: Population,
    },
    HouseholdCohortPopulationChanged {
        cohort: CohortId,
        people: Population,
    },
    RegionPopulationChanged {
        region: RegionId,
        population: Population,
        rate: RatePpm,
    },
    RegionOutputChanged {
        region: RegionId,
        annual_output: Money,
        rate: RatePpm,
    },
    CountryFiscalYearClosed {
        country: CountryId,
        revenue: Money,
        spending: Money,
        treasury: Money,
        debt: Money,
    },
    CountryPoliticsChanged {
        country: CountryId,
        legitimacy: BasisPoints,
        elite_cohesion: BasisPoints,
    },
    YearAdvanced {
        year: i32,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventEnvelope {
    sequence: u64,
    date: SimDate,
    event: DomainEvent,
}
impl EventEnvelope {
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }
    #[must_use]
    pub const fn date(&self) -> SimDate {
        self.date
    }
    #[must_use]
    pub const fn event(&self) -> &DomainEvent {
        &self.event
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EventLog {
    events: Vec<EventEnvelope>,
}
impl EventLog {
    pub fn append(&mut self, date: SimDate, event: DomainEvent) {
        let sequence = self.events.len() as u64;
        self.events.push(EventEnvelope {
            sequence,
            date,
            event,
        });
    }
    #[must_use]
    pub fn events(&self) -> &[EventEnvelope] {
        &self.events
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sequence_numbers_are_monotonic() {
        let date = SimDate::new(2025, 1).expect("valid date");
        let mut log = EventLog::default();
        log.append(date, DomainEvent::WorldFounded { seed: 1 });
        log.append(date, DomainEvent::YearAdvanced { year: 2026 });
        assert_eq!(log.events()[0].sequence(), 0);
        assert_eq!(log.events()[1].sequence(), 1);
    }
}
