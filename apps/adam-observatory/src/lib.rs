//! Read-only presentation adapter between `adam-core` and the Bevy observatory.
//!
//! The adapter owns no simulation rules: it captures an immutable deterministic
//! snapshot and assigns stable map positions from canonical region order.

use adam_core::{BasisPoints, CountryId, Money, RegionId, World};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservatorySnapshot {
    pub date_year: i32,
    pub date_day: u16,
    pub fingerprint: u64,
    pub regions: Vec<RegionSnapshot>,
    pub programs: Vec<ProgramSnapshot>,
    pub chronicle: Vec<ChronicleSnapshot>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RegionSnapshot {
    pub id: RegionId,
    pub country: CountryId,
    pub name: String,
    pub population: u64,
    pub annual_output: Money,
    pub satisfaction: BasisPoints,
    pub map_x: i32,
    pub map_y: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProgramSnapshot {
    pub id: adam_core::ProgramId,
    pub name: String,
    pub status: adam_core::GovernmentProgramStatus,
    pub promised: Money,
    pub appropriated: Money,
    pub delivered: Money,
    pub carryover: Money,
    pub years_delayed: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChronicleSnapshot {
    pub year: i32,
    pub importance: u16,
    pub text: String,
}

impl ObservatorySnapshot {
    #[must_use]
    pub fn capture(world: &World) -> Self {
        let regions = world
            .regions()
            .values()
            .enumerate()
            .map(|(index, region)| {
                let column = i32::try_from(index % 4).unwrap_or(i32::MAX);
                let row = i32::try_from(index / 4).unwrap_or(i32::MAX);
                RegionSnapshot {
                    id: region.id(),
                    country: region.country(),
                    name: region.name().to_owned(),
                    population: region.population().people(),
                    annual_output: region.annual_output(),
                    satisfaction: world
                        .regional_interests()
                        .get(&region.id())
                        .copied()
                        .unwrap_or_default()
                        .satisfaction(),
                    map_x: column * 240 - 360,
                    map_y: 180 - row * 180,
                }
            })
            .collect();
        let programs = world
            .government_programs()
            .values()
            .map(|program| ProgramSnapshot {
                id: program.id(),
                name: program.name().to_owned(),
                status: program.status(),
                promised: program.promised_annual_funding(),
                appropriated: program.appropriated_funding(),
                delivered: program.delivered_funding(),
                carryover: program.carryover_funding(),
                years_delayed: program.years_delayed(),
            })
            .collect();
        let chronicle = world
            .chronicle()
            .into_iter()
            .map(|entry| ChronicleSnapshot {
                year: entry.year,
                importance: entry.importance,
                text: entry.text,
            })
            .collect();
        Self {
            date_year: world.date().year(),
            date_day: world.date().day_of_year(),
            fingerprint: world.stable_fingerprint(),
            regions,
            programs,
            chronicle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_worlds_produce_equal_observatory_snapshots() {
        let first = adam_content::demo_world(7).expect("world");
        let second = adam_content::demo_world(7).expect("world");
        assert_eq!(
            ObservatorySnapshot::capture(&first),
            ObservatorySnapshot::capture(&second)
        );
    }

    #[test]
    fn program_desk_command_changes_authoritative_world_before_snapshot_refresh() {
        let mut world = adam_content::observatory_world(3).expect("world");
        let before = ObservatorySnapshot::capture(&world);
        adam_core::WorldCommand::CancelGovernmentProgram {
            actor: adam_content::OBSERVATORY_ACTOR_ID,
            program: adam_content::OBSERVATORY_PROGRAM_ID,
        }
        .apply(&mut world)
        .expect("command");
        let after = ObservatorySnapshot::capture(&world);
        assert_ne!(before.fingerprint, after.fingerprint);
        assert_eq!(
            after.programs[0].status,
            adam_core::GovernmentProgramStatus::Cancelled
        );
    }

    #[test]
    fn political_timeline_snapshot_is_rebuilt_from_authoritative_chronicle() {
        let mut world = adam_content::observatory_world(11).expect("world");
        adam_core::WorldCommand::CancelGovernmentProgram {
            actor: adam_content::OBSERVATORY_ACTOR_ID,
            program: adam_content::OBSERVATORY_PROGRAM_ID,
        }
        .apply(&mut world)
        .expect("command");
        let snapshot = ObservatorySnapshot::capture(&world);
        assert!(
            snapshot
                .chronicle
                .iter()
                .any(|entry| entry.text.contains("program"))
        );
        assert!(
            snapshot
                .chronicle
                .iter()
                .any(|entry| entry.text.contains("cancelled"))
        );
    }

    #[test]
    fn anniversary_program_flow_replays_to_identical_visual_snapshot() {
        let mut direct = adam_content::observatory_world(100).expect("world");
        let mut replayed = adam_content::observatory_world(100).expect("world");
        let commands = [
            adam_core::WorldCommand::AppropriateGovernmentProgram {
                actor: adam_content::OBSERVATORY_ACTOR_ID,
                program: adam_content::OBSERVATORY_PROGRAM_ID,
                amount: Money::from_minor_units(0),
                source: adam_core::ProgramFundingSource::Treasury,
            },
            adam_core::WorldCommand::ExecuteGovernmentProgram {
                actor: adam_content::OBSERVATORY_ACTOR_ID,
                program: adam_content::OBSERVATORY_PROGRAM_ID,
            },
        ];
        for command in &commands {
            command.apply(&mut direct).expect("direct command");
        }
        adam_core::replay_commands(&mut replayed, &commands).expect("replay");
        assert_eq!(direct.stable_fingerprint(), replayed.stable_fingerprint());
        assert_eq!(
            ObservatorySnapshot::capture(&direct),
            ObservatorySnapshot::capture(&replayed)
        );
        assert!(!ObservatorySnapshot::capture(&direct).chronicle.is_empty());
    }

    #[test]
    fn map_layout_is_canonical_and_non_overlapping() {
        let world = adam_content::demo_world(1).expect("world");
        let snapshot = ObservatorySnapshot::capture(&world);
        let positions: std::collections::BTreeSet<_> = snapshot
            .regions
            .iter()
            .map(|region| (region.map_x, region.map_y))
            .collect();
        assert_eq!(positions.len(), snapshot.regions.len());
        assert!(
            snapshot
                .regions
                .windows(2)
                .all(|pair| pair[0].id < pair[1].id)
        );
    }
}
