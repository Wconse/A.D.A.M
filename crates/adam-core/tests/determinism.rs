use adam_core::{Country, CountryId, DomainEvent, SimDate, World, WorldSeed};

fn sample_world(seed: u64) -> World {
    let mut world = World::new(
        WorldSeed::new(seed),
        SimDate::new(2025, 1).expect("valid date"),
    );
    for (id, name) in [
        (1, "Aster Republic"),
        (2, "Boreal Union"),
        (3, "Cyrene Federation"),
    ] {
        world
            .register_country(Country::new(CountryId::new(id), name).expect("valid country"))
            .expect("unique country");
    }
    world
        .advance_years(50)
        .expect("fifty years fit in date range");
    world
}

#[test]
fn twin_runs_are_identical() {
    let first = sample_world(47);
    let second = sample_world(47);
    assert_eq!(first, second);
    assert_eq!(first.stable_fingerprint(), second.stable_fingerprint());
}

#[test]
fn different_seeds_have_different_fingerprints() {
    assert_ne!(
        sample_world(47).stable_fingerprint(),
        sample_world(48).stable_fingerprint()
    );
}

#[test]
fn fifty_year_run_records_every_year() {
    let world = sample_world(47);
    assert_eq!(world.date().year(), 2075);
    let completed_years = world
        .events()
        .events()
        .iter()
        .filter(|event| matches!(event.event(), DomainEvent::YearAdvanced { .. }))
        .count();
    assert_eq!(completed_years, 50);
}
