use std::collections::BTreeMap;

use crate::{
    ActorId, BasisPoints, CountryId, DomainEvent, PowerNodeId, RegionId, World, WorldError,
};

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum ServiceAllocationSource {
    AutonomousPrudent,
    ExplicitPoliticalDecision,
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
pub enum ServiceAllocationInfluenceKind {
    OfficeHolder,
    InfluenceEdge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ServiceAllocationInfluence {
    pub(crate) actor: ActorId,
    pub(crate) office: PowerNodeId,
    pub(crate) region: RegionId,
    pub(crate) kind: ServiceAllocationInfluenceKind,
    pub(crate) weight: BasisPoints,
    pub(crate) score_bonus: u16,
}

const OFFICE_HOLDER_SCORE_BONUS: u16 = 1_000;
const INFLUENCE_SCORE_DIVISOR: u16 = 2;

/// Exact shares of a country's discretionary public-service budget.
///
/// Omitted regions receive zero. A single region may legally receive 100%.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CountryServiceAllocation {
    shares: BTreeMap<RegionId, BasisPoints>,
}

impl CountryServiceAllocation {
    /// Creates an allocation whose shares sum to exactly 10,000 basis points.
    /// Zero shares and a 100% share are valid.
    /// # Errors
    /// Rejects an empty map or a total other than 10,000.
    pub fn new(shares: BTreeMap<RegionId, BasisPoints>) -> Result<Self, WorldError> {
        let total: u32 = shares.values().map(|share| u32::from(share.get())).sum();
        if shares.is_empty() || total != u32::from(BasisPoints::FULL.get()) {
            return Err(WorldError::InvalidServiceAllocation(
                "service allocation shares must sum to exactly 10000 basis points",
            ));
        }
        Ok(Self { shares })
    }

    #[must_use]
    pub fn shares(&self) -> &BTreeMap<RegionId, BasisPoints> {
        &self.shares
    }
}

impl World {
    /// Replaces the discretionary regional service allocation under a political office holder.
    ///
    /// This validates feasibility and authority, not fairness. A valid office holder may direct
    /// the entire budget to one region and leave all others at zero.
    /// # Errors
    /// Rejects unknown countries or regions, foreign regions, or unauthorized actors.
    pub fn set_country_service_allocation(
        &mut self,
        actor: ActorId,
        country: CountryId,
        allocation: CountryServiceAllocation,
    ) -> Result<(), WorldError> {
        if !self.countries.contains_key(&country) {
            return Err(WorldError::UnknownCountry(country));
        }
        if !self.can_authorize_emergency_relief(actor, country) {
            return Err(WorldError::UnauthorizedGovernmentAction { actor, country });
        }
        for region in allocation.shares().keys() {
            let registered = self
                .regions
                .get(region)
                .ok_or(WorldError::UnknownRegion(*region))?;
            if registered.country() != country {
                return Err(WorldError::InvalidServiceAllocation(
                    "service allocation cannot include a foreign region",
                ));
            }
        }
        let shares = allocation.shares().clone();
        self.country_service_allocations.insert(country, allocation);
        self.events.append(
            self.date,
            DomainEvent::CountryServiceAllocationSet {
                actor,
                country,
                shares,
            },
        );
        Ok(())
    }

    #[must_use]
    pub fn country_service_allocations(&self) -> &BTreeMap<CountryId, CountryServiceAllocation> {
        &self.country_service_allocations
    }

    pub(crate) fn plan_regional_service_budgets(
        &self,
        country: CountryId,
        service_budget: i128,
    ) -> (
        ServiceAllocationSource,
        BTreeMap<RegionId, BasisPoints>,
        BTreeMap<RegionId, i128>,
        Vec<ServiceAllocationInfluence>,
    ) {
        let (source, shares, influences) =
            if let Some(explicit) = self.country_service_allocations.get(&country) {
                (
                    ServiceAllocationSource::ExplicitPoliticalDecision,
                    explicit.shares().clone(),
                    Vec::new(),
                )
            } else {
                let (shares, influences) = self.derive_prudent_service_shares(country);
                (
                    ServiceAllocationSource::AutonomousPrudent,
                    shares,
                    influences,
                )
            };
        let budgets = allocate_integer_budget(service_budget.max(0), &shares);
        (source, shares, budgets, influences)
    }

    fn derive_prudent_service_shares(
        &self,
        country: CountryId,
    ) -> (
        BTreeMap<RegionId, BasisPoints>,
        Vec<ServiceAllocationInfluence>,
    ) {
        let (political_bonus, influences) = self.service_allocation_political_bonus(country);
        let mut weights = BTreeMap::<RegionId, u128>::new();
        for region in self
            .regions
            .values()
            .filter(|region| region.country() == country)
        {
            let population = u128::from(region.population().people()).max(1);
            let interest = self
                .regional_interests
                .get(&region.id())
                .copied()
                .unwrap_or_default();
            let pressure = self
                .regional_social_pressure
                .get(&region.id())
                .copied()
                .unwrap_or_default();
            let direct_need =
                if interest.priority() == crate::RegionalPolicyPriority::PublicServices {
                    u128::from(interest.priority_pressure().get())
                } else {
                    u128::from(pressure.public_service_shortfall().get()) / 2
                };
            let low_confidence =
                u128::from(10_000_u16.saturating_sub(interest.satisfaction().get()));
            let prudence_score = 5_000_u128
                .saturating_add(direct_need)
                .saturating_add(low_confidence / 2)
                .saturating_add(
                    political_bonus
                        .get(&region.id())
                        .copied()
                        .unwrap_or_default(),
                );
            weights.insert(region.id(), population.saturating_mul(prudence_score));
        }
        (normalize_weights(&weights), influences)
    }

    fn service_allocation_political_bonus(
        &self,
        country: CountryId,
    ) -> (BTreeMap<RegionId, u128>, Vec<ServiceAllocationInfluence>) {
        let mut bonuses = BTreeMap::<RegionId, u128>::new();
        let mut evidence = Vec::new();
        for office in self.power_nodes().values().filter(|node| {
            node.country() == country && node.kind() == crate::PowerNodeKind::PoliticalOffice
        }) {
            if let Some(holder) = office.holder() {
                self.record_service_allocation_influence(
                    holder,
                    office.id(),
                    ServiceAllocationInfluenceKind::OfficeHolder,
                    BasisPoints::FULL,
                    OFFICE_HOLDER_SCORE_BONUS,
                    country,
                    &mut bonuses,
                    &mut evidence,
                );
            }
            for influence in self
                .influences()
                .values()
                .filter(|influence| influence.node() == office.id())
            {
                let score_bonus = influence.weight().get() / INFLUENCE_SCORE_DIVISOR;
                if score_bonus == 0 {
                    continue;
                }
                self.record_service_allocation_influence(
                    influence.actor(),
                    office.id(),
                    ServiceAllocationInfluenceKind::InfluenceEdge,
                    influence.weight(),
                    score_bonus,
                    country,
                    &mut bonuses,
                    &mut evidence,
                );
            }
        }
        (bonuses, evidence)
    }

    #[allow(clippy::too_many_arguments)]
    fn record_service_allocation_influence(
        &self,
        actor: ActorId,
        office: PowerNodeId,
        kind: ServiceAllocationInfluenceKind,
        weight: BasisPoints,
        score_bonus: u16,
        country: CountryId,
        bonuses: &mut BTreeMap<RegionId, u128>,
        evidence: &mut Vec<ServiceAllocationInfluence>,
    ) {
        let Some(actor_state) = self.actors().get(&actor) else {
            return;
        };
        let region = actor_state.home_region();
        if self
            .regions
            .get(&region)
            .is_none_or(|home| home.country() != country)
        {
            return;
        }
        bonuses
            .entry(region)
            .and_modify(|bonus| *bonus = bonus.saturating_add(u128::from(score_bonus)))
            .or_insert(u128::from(score_bonus));
        evidence.push(ServiceAllocationInfluence {
            actor,
            office,
            region,
            kind,
            weight,
            score_bonus,
        });
    }
}

fn normalize_weights(weights: &BTreeMap<RegionId, u128>) -> BTreeMap<RegionId, BasisPoints> {
    let total: u128 = weights.values().copied().sum();
    if weights.is_empty() {
        return BTreeMap::new();
    }
    let denominator = total.max(1);
    let mut rows = Vec::with_capacity(weights.len());
    let mut assigned = 0_u16;
    for (region, weight) in weights {
        let scaled = weight.saturating_mul(10_000);
        let floor = u16::try_from(scaled / denominator).unwrap_or(10_000);
        let remainder = scaled % denominator;
        assigned = assigned.saturating_add(floor);
        rows.push((*region, floor, remainder));
    }
    rows.sort_by_key(|(region, _, remainder)| (std::cmp::Reverse(*remainder), *region));
    for row in rows
        .iter_mut()
        .take(usize::from(10_000_u16.saturating_sub(assigned)))
    {
        row.1 = row.1.saturating_add(1);
    }
    rows.sort_by_key(|(region, _, _)| *region);
    rows.into_iter()
        .map(|(region, share, _)| {
            (
                region,
                BasisPoints::new(share).expect("normalized share is bounded"),
            )
        })
        .collect()
}

fn allocate_integer_budget(
    budget: i128,
    shares: &BTreeMap<RegionId, BasisPoints>,
) -> BTreeMap<RegionId, i128> {
    let mut rows = Vec::with_capacity(shares.len());
    let mut assigned = 0_i128;
    for (region, share) in shares {
        let scaled = budget.saturating_mul(i128::from(share.get()));
        let floor = scaled / 10_000;
        let remainder = scaled % 10_000;
        assigned = assigned.saturating_add(floor);
        rows.push((*region, floor, remainder));
    }
    rows.sort_by_key(|(region, _, remainder)| (std::cmp::Reverse(*remainder), *region));
    let unassigned = usize::try_from(budget.saturating_sub(assigned)).unwrap_or(0);
    for row in rows.iter_mut().take(unassigned) {
        row.1 = row.1.saturating_add(1);
    }
    rows.sort_by_key(|(region, _, _)| *region);
    rows.into_iter()
        .map(|(region, amount, _)| (region, amount))
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{
        Actor, Country, Money, Population, PowerNode, PowerNodeId, PowerNodeKind, Region, SimDate,
        WorldSeed,
    };

    use super::*;

    #[test]
    fn explicit_allocation_may_give_everything_to_one_region() {
        let allocation = CountryServiceAllocation::new(BTreeMap::from([
            (RegionId::new(1), BasisPoints::FULL),
            (RegionId::new(2), BasisPoints::ZERO),
        ]))
        .expect("allocation");
        let budgets = allocate_integer_budget(101, allocation.shares());
        assert_eq!(budgets[&RegionId::new(1)], 101);
        assert_eq!(budgets[&RegionId::new(2)], 0);
    }

    #[test]
    fn integer_budget_is_conserved_exactly() {
        let shares = BTreeMap::from([
            (RegionId::new(1), BasisPoints::new(3_333).expect("share")),
            (RegionId::new(2), BasisPoints::new(3_333).expect("share")),
            (RegionId::new(3), BasisPoints::new(3_334).expect("share")),
        ]);
        let budgets = allocate_integer_budget(10, &shares);
        assert_eq!(budgets.values().sum::<i128>(), 10);
        assert_eq!(budgets[&RegionId::new(3)], 4);
    }

    fn political_world() -> World {
        let mut world = World::new(WorldSeed::new(12), SimDate::new(2025, 1).expect("date"));
        world
            .register_country(Country::new(CountryId::new(1), "Union").expect("country"))
            .expect("country");
        for id in 1..=2 {
            world
                .register_region(
                    Region::new(
                        RegionId::new(id),
                        CountryId::new(1),
                        format!("Region {id}"),
                        Population::new(100),
                        Money::from_minor_units(10_000),
                    )
                    .expect("region"),
                )
                .expect("region");
        }
        world
            .register_actor(
                Actor::new(ActorId::new(1), "President", RegionId::new(1), 1980).expect("actor"),
            )
            .expect("actor");
        world
            .register_power_node(
                PowerNode::new(
                    PowerNodeId::new(1),
                    CountryId::new(1),
                    "Presidency",
                    PowerNodeKind::PoliticalOffice,
                    Some(ActorId::new(1)),
                )
                .expect("office"),
            )
            .expect("office");
        world
    }

    #[test]
    fn authorized_office_holder_can_command_extreme_allocation() {
        let mut world = political_world();
        let allocation = CountryServiceAllocation::new(BTreeMap::from([
            (RegionId::new(1), BasisPoints::FULL),
            (RegionId::new(2), BasisPoints::ZERO),
        ]))
        .expect("allocation");
        world
            .set_country_service_allocation(ActorId::new(1), CountryId::new(1), allocation)
            .expect("authorized allocation");
        let (source, shares, budgets, influences) =
            world.plan_regional_service_budgets(CountryId::new(1), 101);
        assert_eq!(source, ServiceAllocationSource::ExplicitPoliticalDecision);
        assert_eq!(shares[&RegionId::new(1)], BasisPoints::FULL);
        assert_eq!(budgets[&RegionId::new(1)], 101);
        assert_eq!(budgets[&RegionId::new(2)], 0);
        assert!(influences.is_empty());
    }

    #[test]
    fn unauthorized_allocation_is_atomic() {
        let mut world = political_world();
        let before = world.clone();
        let allocation =
            CountryServiceAllocation::new(BTreeMap::from([(RegionId::new(1), BasisPoints::FULL)]))
                .expect("allocation");
        assert!(matches!(
            world.set_country_service_allocation(ActorId::new(2), CountryId::new(1), allocation),
            Err(WorldError::UnauthorizedGovernmentAction { .. })
        ));
        assert_eq!(world, before);
    }

    #[test]
    fn autonomous_prudence_favors_observed_service_need_without_engine_caps() {
        let mut world = political_world();
        world.regional_social_pressure.insert(
            RegionId::new(1),
            crate::RegionalSocialPressure::from_components_for_test(0, 0, 1_000, 166),
        );
        world.regional_social_pressure.insert(
            RegionId::new(2),
            crate::RegionalSocialPressure::from_components_for_test(0, 0, 9_000, 1_500),
        );
        let (source, shares, budgets, influences) =
            world.plan_regional_service_budgets(CountryId::new(1), 100);
        assert_eq!(source, ServiceAllocationSource::AutonomousPrudent);
        assert!(shares[&RegionId::new(2)].get() > shares[&RegionId::new(1)].get());
        assert_eq!(
            shares
                .values()
                .map(|share| u32::from(share.get()))
                .sum::<u32>(),
            10_000
        );
        assert_eq!(budgets.values().sum::<i128>(), 100);
        assert!(
            influences
                .iter()
                .any(|evidence| evidence.kind == ServiceAllocationInfluenceKind::OfficeHolder)
        );
    }

    #[test]
    fn political_influence_biases_autonomous_allocation_toward_actor_home() {
        let mut world = political_world();
        world
            .register_actor(
                Actor::new(ActorId::new(2), "Industrial patron", RegionId::new(2), 1975)
                    .expect("actor"),
            )
            .expect("actor");
        world
            .establish_influence(crate::Influence::new(
                ActorId::new(2),
                PowerNodeId::new(1),
                BasisPoints::new(8_000).expect("influence"),
            ))
            .expect("influence");
        let (source, shares, budgets, evidence) =
            world.plan_regional_service_budgets(CountryId::new(1), 100);
        assert_eq!(source, ServiceAllocationSource::AutonomousPrudent);
        assert!(shares[&RegionId::new(2)].get() > shares[&RegionId::new(1)].get());
        assert_eq!(budgets.values().sum::<i128>(), 100);
        assert!(evidence.iter().any(|item| {
            item.actor == ActorId::new(2)
                && item.region == RegionId::new(2)
                && item.kind == ServiceAllocationInfluenceKind::InfluenceEdge
                && item.score_bonus == 4_000
        }));
    }
}
