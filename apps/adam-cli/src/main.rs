use std::env;
use std::error::Error;
use std::fmt;

use adam_core::{Country, CountryId, SimDate, World, WorldSeed};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(2);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let arguments = Arguments::parse(env::args().skip(1))?;
    let mut world = World::new(
        WorldSeed::new(arguments.seed),
        SimDate::new(2025, 1)?,
    );

    for (id, name) in [(1, "Aster Republic"), (2, "Boreal Union"), (3, "Cyrene Federation")] {
        world.register_country(Country::new(CountryId::new(id), name)?)?;
    }
    world.advance_years(arguments.years)?;

    println!("A.D.A.M Stage 0 foundation");
    println!("seed: {}", world.seed().get());
    println!("date: {}", world.date());
    println!("countries: {}", world.countries().len());
    println!("events: {}", world.events().len());
    println!("fingerprint: {:016x}", world.stable_fingerprint());
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Arguments {
    seed: u64,
    years: u32,
}

impl Arguments {
    fn parse(arguments: impl IntoIterator<Item = String>) -> Result<Self, ArgumentError> {
        let mut seed = 1;
        let mut years = 50;
        let mut arguments = arguments.into_iter();

        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--seed" => seed = parse_value("--seed", arguments.next())?,
                "--years" => years = parse_value("--years", arguments.next())?,
                "--help" | "-h" => {
                    println!("Usage: adam-cli [--seed <u64>] [--years <u32>]");
                    std::process::exit(0);
                }
                _ => return Err(ArgumentError::UnknownArgument(argument)),
            }
        }

        Ok(Self { seed, years })
    }
}

fn parse_value<T>(name: &'static str, value: Option<String>) -> Result<T, ArgumentError>
where
    T: std::str::FromStr,
{
    let value = value.ok_or(ArgumentError::MissingValue(name))?;
    value.parse().map_err(|_| ArgumentError::InvalidValue { name, value })
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
            Self::InvalidValue { name, value } => write!(formatter, "invalid value for {name}: {value}"),
        }
    }
}

impl Error for ArgumentError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_stage_zero_run() {
        assert_eq!(Arguments::parse(Vec::new()).expect("defaults parse"), Arguments { seed: 1, years: 50 });
    }

    #[test]
    fn parses_explicit_values() {
        let args = vec!["--seed".to_owned(), "47".to_owned(), "--years".to_owned(), "12".to_owned()];
        assert_eq!(Arguments::parse(args).expect("arguments parse"), Arguments { seed: 47, years: 12 });
    }
}
