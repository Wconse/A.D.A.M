//! Console chronicle runner for A.D.A.M Stage 0.
//!
//! Runs the embedded demo scenario for a number of deterministic economic
//! years and prints a yearly chronicle derived from the domain event log.
//!
//! Usage: `adam-cli [--seed N] [--years N]` (defaults: seed 1, 50 years).

use std::fmt::Write as _;
use std::process::ExitCode;

use adam_core::{DomainEvent, World, WorldError};

struct Args {
    seed: u64,
    years: u32,
}

fn parse_args() -> Result<Args, String> {
    let mut args = Args { seed: 1, years: 50 };
    let mut input = std::env::args().skip(1);
    while let Some(flag) = input.next() {
        let value = input
            .next()
            .ok_or_else(|| format!("{flag} requires a value"))?;
        match flag.as_str() {
            "--seed" => {
                args.seed = value
                    .parse()
                    .map_err(|error| format!("invalid --seed: {error}"))?;
            }
            "--years" => {
                args.years = value
                    .parse()
                    .map_err(|error| format!("invalid --years: {error}"))?;
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    Ok(args)
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(args) => args,
        Err(error) => {
            eprintln!("error: {error}");
            eprintln!("usage: adam-cli [--seed N] [--years N]");
            return ExitCode::from(2);
        }
    };
    match run(&args) {
        Ok(chronicle) => {
            print!("{chronicle}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("simulation error: {error:?}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &Args) -> Result<String, WorldError> {
    let mut world = adam_content::demo_world(args.seed)?;
    world.advance_economic_years(args.years)?;
    Ok(render_chronicle(&world, args))
}

#[derive(Default)]
struct YearStats {
    months: u32,
    household_fills: u64,
    household_spend: i64,
    b2b_trades: u64,
    b2b_spend: i64,
    taxes_paid: i64,
    closure_lines: Vec<String>,
}

fn render_chronicle(world: &World, args: &Args) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "A.D.A.M Stage 0 chronicle | seed {} | {} years | money in minor units",
        args.seed, args.years
    );
    let mut stats = YearStats::default();
    for envelope in world.events().events() {
        collect_event(&mut out, &mut stats, envelope.event());
    }
    let _ = writeln!(out, "\nfinal fingerprint: {:?}", world.stable_fingerprint());
    out
}

fn collect_event(out: &mut String, stats: &mut YearStats, event: &DomainEvent) {
    match event {
        DomainEvent::MonthlyEconomicCycleCompleted { .. } => stats.months += 1,
        DomainEvent::MarketTrade { spend, .. } => {
            stats.household_fills += 1;
            stats.household_spend += spend.minor_units();
        }
        DomainEvent::FirmProcurementTrade { spend, .. } => {
            stats.b2b_trades += 1;
            stats.b2b_spend += spend.minor_units();
        }
        DomainEvent::FirmSalesTaxPaid { paid, .. } => stats.taxes_paid += paid.minor_units(),
        DomainEvent::RegionalOutputMeasured {
            region,
            final_consumption,
            inventory_change,
            annual_output,
        } => {
            stats.closure_lines.push(format!(
                "  {region:?}: output {} = consumption {} + inventory change {}",
                annual_output.minor_units(),
                final_consumption.minor_units(),
                inventory_change.minor_units()
            ));
        }
        DomainEvent::RegionPopulationChanged {
            region, population, ..
        } => {
            stats
                .closure_lines
                .push(format!("  {region:?}: population {}", population.people()));
        }
        DomainEvent::CountryFiscalYearClosed {
            country,
            revenue,
            spending,
            treasury,
            debt,
        } => {
            stats.closure_lines.push(format!(
                "  {country:?}: revenue {}, spending {}, treasury {}, debt {}",
                revenue.minor_units(),
                spending.minor_units(),
                treasury.minor_units(),
                debt.minor_units()
            ));
        }
        DomainEvent::CountryPoliticsChanged {
            country,
            legitimacy,
            elite_cohesion,
        } => {
            stats.closure_lines.push(format!(
                "  {country:?}: legitimacy {} bp, elite cohesion {} bp",
                legitimacy.get(),
                elite_cohesion.get()
            ));
        }
        DomainEvent::YearAdvanced { year } => flush_year(out, stats, *year - 1),
        _ => {}
    }
}

fn flush_year(out: &mut String, stats: &mut YearStats, closed_year: i32) {
    let _ = writeln!(
        out,
        "\n=== year {closed_year} ({} monthly cycles) ===",
        stats.months
    );
    let _ = writeln!(
        out,
        "  households: {} purchases, total spend {}",
        stats.household_fills, stats.household_spend
    );
    let _ = writeln!(
        out,
        "  firm-to-firm: {} trades, total spend {}",
        stats.b2b_trades, stats.b2b_spend
    );
    if stats.taxes_paid != 0 {
        let _ = writeln!(out, "  sales taxes paid: {}", stats.taxes_paid);
    }
    for line in &stats.closure_lines {
        let _ = writeln!(out, "{line}");
    }
    *stats = YearStats::default();
}
