use std::error::Error;
use std::time::Instant;

use adam_core::{
    Actor, ActorId, BasisPoints, Country, CountryId, Influence, Money, Population, PowerNode,
    PowerNodeId, PowerNodeKind, Region, RegionId, SimDate, World, WorldSeed,
};

fn main() -> Result<(), Box<dyn Error>> {
    let count = std::env::args()
        .nth(1)
        .map_or(Ok(100_000_u32), |value| value.parse())?;
    let started = Instant::now();
    let world = build_scale_world(count)?;
    let elapsed = started.elapsed();

    println!("actors: {}", world.actors().len());
    println!("power nodes: {}", world.power_nodes().len());
    println!("influence edges: {}", world.influences().len());
    println!("events: {}", world.events().len());
    println!("elapsed_ms: {}", elapsed.as_millis());
    println!("fingerprint: {:016x}", world.stable_fingerprint());
    Ok(())
}

fn build_scale_world(count: u32) -> Result<World, Box<dyn Error>> {
    let mut world = World::new(
        WorldSeed::new(47),
        SimDate::new(2025, 1).expect("fixed date is valid"),
    );
    world.register_country(Country::new(CountryId::new(1), "Scale Republic")?)?;
    world.register_region(Region::new(
        RegionId::new(1),
        CountryId::new(1),
        "Scale Capital",
        Population::new(u64::from(count) * 100),
        Money::from_minor_units(i64::from(count) * 1_000_000),
    )?)?;

    for raw_id in 1..=count {
        let actor_id = ActorId::new(raw_id);
        let node_id = PowerNodeId::new(raw_id);
        world.register_actor(Actor::new(
            actor_id,
            format!("Actor {raw_id}"),
            RegionId::new(1),
            1980,
        )?)?;
        world.register_power_node(PowerNode::new(
            node_id,
            CountryId::new(1),
            format!("Node {raw_id}"),
            PowerNodeKind::Capital,
            Some(actor_id),
        )?)?;
        world.establish_influence(Influence::new(
            actor_id,
            node_id,
            BasisPoints::new(5_000).expect("fixed influence is valid"),
        ))?;
    }
    Ok(world)
}
