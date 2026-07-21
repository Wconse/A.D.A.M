use std::env;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::PathBuf;

use adam_content::WorldBlueprint;
use adam_core::WorldSeed;

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let arguments = Arguments::parse(env::args().skip(1))?;
    let source = fs::read_to_string(&arguments.config)?;
    let blueprint = WorldBlueprint::parse_toml(&source)?;
    let mut world = blueprint.build_world(WorldSeed::new(arguments.seed))?;
    world.advance_years(arguments.years)?;
    world.validate_population_accounting()?;
    let demand = world.plan_monthly_household_demand()?;
    let desired_quantity: u128 = demand
        .iter()
        .map(|intent| u128::from(intent.desired().get()))
        .sum();
    let budgeted_quantity: u128 = demand
        .iter()
        .map(|intent| u128::from(intent.budgeted().get()))
        .sum();
    let reserved_spend: i128 = demand
        .iter()
        .map(|intent| i128::from(intent.reserved_spend().minor_units()))
        .sum();

    println!("A.D.A.M Stage 0 foundation");
    println!("world: {}", blueprint.name());
    println!("seed: {}", world.seed().get());
    println!("date: {}", world.date());
    println!("countries: {}", world.countries().len());
    println!("regions: {}", world.regions().len());
    println!("household cohorts: {}", world.household_cohorts().len());
    println!("actors: {}", world.actors().len());
    println!("power nodes: {}", world.power_nodes().len());
    println!("influence edges: {}", world.influences().len());
    println!("goods: {}", world.goods().len());
    println!("monthly demand intents: {}", demand.len());
    println!("monthly desired quantity milli: {desired_quantity}");
    println!("monthly budgeted quantity milli: {budgeted_quantity}");
    println!("monthly reserved spend minor: {reserved_spend}");
    println!("events: {}", world.events().len());
    for country in world.countries().values() {
        let state = country.indicators();
        println!(
            "country_state: {} treasury={} debt={} legitimacy_bps={} cohesion_bps={}",
            country.name(),
            state.treasury().minor_units(),
            state.public_debt().minor_units(),
            state.legitimacy().get(),
            state.elite_cohesion().get(),
        );
    }
    for region in world.regions().values() {
        println!(
            "region_state: {} population={} output_minor={}",
            region.name(),
            region.population().people(),
            region.annual_output().minor_units(),
        );
    }
    println!("fingerprint: {:016x}", world.stable_fingerprint());
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Arguments {
    seed: u64,
    years: u32,
    config: PathBuf,
}

impl Arguments {
    fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Self, ArgumentError> {
        let mut seed = 1;
        let mut years = 50;
        let mut config = PathBuf::from("config/world.example.toml");
        let mut arguments = arguments.into_iter();

        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--seed" => seed = parse_value("--seed", arguments.next())?,
                "--years" => years = parse_value("--years", arguments.next())?,
                "--config" => {
                    config = PathBuf::from(
                        arguments
                            .next()
                            .ok_or(ArgumentError::MissingValue("--config"))?,
                    );
                }
                "--help" | "-h" => {
                    println!("Usage: adam-cli [--seed <u64>] [--years <u32>] [--config <path>]");
                    std::process::exit(0);
                }
                _ => return Err(ArgumentError::UnknownArgument(argument)),
            }
        }

        Ok(Self {
            seed,
            years,
            config,
        })
    }
}

fn parse_value<T>(name: &'static str, value: Option<String>) -> Result<T, ArgumentError>
where
    T: std::str::FromStr,
{
    let value = value.ok_or(ArgumentError::MissingValue(name))?;
    value
        .parse()
        .map_err(|_| ArgumentError::InvalidValue { name, value })
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ArgumentError {
    UnknownArgument(String),
    MissingValue(&'static str),
    InvalidValue { name: &'static str, value: String },
}

impl fmt::Display for ArgumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownArgument(value) => write!(formatter, "unknown argument: {value}"),
            Self::MissingValue(name) => write!(formatter, "missing value for {name}"),
            Self::InvalidValue { name, value } => {
                write!(formatter, "invalid value for {name}: {value}")
            }
        }
    }
}

impl Error for ArgumentError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_stage_zero_run() {
        assert_eq!(
            Arguments::parse(Vec::new()).expect("defaults parse"),
            Arguments {
                seed: 1,
                years: 50,
                config: PathBuf::from("config/world.example.toml"),
            }
        );
    }

    #[test]
    fn parses_explicit_values() {
        let args = vec![
            "--seed".to_owned(),
            "47".to_owned(),
            "--years".to_owned(),
            "12".to_owned(),
            "--config".to_owned(),
            "config/custom.toml".to_owned(),
        ];
        assert_eq!(
            Arguments::parse(args).expect("arguments parse"),
            Arguments {
                seed: 47,
                years: 12,
                config: PathBuf::from("config/custom.toml"),
            }
        );
    }
}
