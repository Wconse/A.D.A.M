use crate::{ActorId, FirmId, FirmPolicy, World, WorldError};
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum WorldCommand {
    AdvanceYears(u32),
    SetFirmPolicy {
        actor: ActorId,
        firm: FirmId,
        policy: FirmPolicy,
    },
}
impl WorldCommand {
    /// Applies the same deterministic command regardless of player or AI origin.
    /// # Errors
    /// Returns [`WorldError`] if the authoritative transition cannot complete.
    pub fn apply(&self, world: &mut World) -> Result<(), WorldError> {
        match self {
            Self::AdvanceYears(years) => world.advance_years(*years),
            Self::SetFirmPolicy {
                actor,
                firm,
                policy,
            } => world.set_firm_policy(*actor, *firm, *policy),
        }
    }
}
/// Applies commands strictly in recorded order.
/// # Errors
/// Returns the first transition error and does not apply later commands.
pub fn replay_commands(world: &mut World, commands: &[WorldCommand]) -> Result<(), WorldError> {
    for command in commands {
        command.apply(world)?;
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Country, CountryId, SimDate, WorldSeed};
    #[test]
    fn ordered_commands_match_direct_transition() {
        let mut direct = World::new(WorldSeed::new(1), SimDate::new(2025, 1).expect("date"));
        direct
            .register_country(Country::new(CountryId::new(1), "A").expect("country"))
            .expect("register");
        let mut replayed = direct.clone();
        direct.advance_years(5).expect("years");
        replay_commands(
            &mut replayed,
            &[WorldCommand::AdvanceYears(2), WorldCommand::AdvanceYears(3)],
        )
        .expect("replay");
        assert_eq!(direct, replayed);
    }
}
