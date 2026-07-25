import io

def edit(path, pairs):
    with io.open(path, 'r', encoding='utf-8') as f:
        content = f.read()
    for i, (old, new) in enumerate(pairs):
        n = content.count(old)
        assert n == 1, f'{path} edit[{i}]: expected 1 match, got {n}'
        content = content.replace(old, new)
    with io.open(path, 'w', encoding='utf-8', newline='\n') as f:
        f.write(content)
    print('OK', path)

# demand.rs
P = 'crates/adam-core/src/demand.rs'

OLD1 = '    pub fn plan_monthly_household_demand(&self) -> Result<Vec<DemandIntent>, WorldError> {\n        self.plan_monthly_household_demand_against_offers(&[])\n    }'
NEW1 = '    pub fn plan_monthly_household_demand(&self) -> Result<Vec<DemandIntent>, WorldError> {\n        let mut cap = self.market_spot_route_capacity();\n        self.plan_monthly_household_demand_against_offers(&[], &mut cap)\n    }'

OLD2 = '    pub(crate) fn plan_monthly_household_demand_against_offers(\n        &self,\n        offers: &[MarketOffer],\n    ) -> Result<Vec<DemandIntent>, WorldError> {'
NEW2 = '    pub(crate) fn plan_monthly_household_demand_against_offers(\n        &self,\n        offers: &[MarketOffer],\n        route_capacity: &mut BTreeMap<RouteId, u64>,\n    ) -> Result<Vec<DemandIntent>, WorldError> {'

OLD3 = '        let mut route_capacity = self.market_spot_route_capacity();\n        let mut ledger = SurvivalSupplyLedger {\n            supply: &supply,\n            remaining: &mut remaining_supply,\n            routes: &mut route_capacity,'
NEW3 = '        let mut ledger = SurvivalSupplyLedger {\n            supply: &supply,\n            remaining: &mut remaining_supply,\n            routes: route_capacity,'

edit(P, [(OLD1, NEW1), (OLD2, NEW2), (OLD3, NEW3)])

# commerce.rs
P2 = 'crates/adam-core/src/commerce.rs'

OLD4 = '        let production_plans = next.execute_monthly_production()?;\n        let procurement = next.execute_monthly_firm_procurement()?;\n        let offer_plans = next.plan_firm_market_offers()?;\n        let offers: Vec<_> = offer_plans\n            .iter()\n            .filter_map(|plan| plan.market_offer())\n            .collect();\n        let demand_intents = next.plan_monthly_household_demand_against_offers(&offers)?;'
NEW4 = '        let production_plans = next.execute_monthly_production()?;\n        // One shared route-capacity pool for the whole commercial cycle:\n        // B2B firm procurement fills consume it first, then household survival\n        // imports compete for the remainder, and market clearing uses what is\n        // left. All import flows are bounded by the same physical monthly cap.\n        let mut route_capacity = next.market_spot_route_capacity();\n        let procurement = next.execute_monthly_firm_procurement(&mut route_capacity)?;\n        let offer_plans = next.plan_firm_market_offers()?;\n        let offers: Vec<_> = offer_plans\n            .iter()\n            .filter_map(|plan| plan.market_offer())\n            .collect();\n        let demand_intents =\n            next.plan_monthly_household_demand_against_offers(&offers, &mut route_capacity)?;'

OLD5 = '        let rationing = next.apply_survival_rationing(&mut orders, &offers)?;\n        let mut market_route_capacity = next.market_spot_route_capacity();\n        let mut clearing = clear_market_with_delivery(\n            &orders,\n            &offers,\n            &mut market_route_capacity,'
NEW5 = '        let rationing = next.apply_survival_rationing(&mut orders, &offers)?;\n        let mut clearing = clear_market_with_delivery(\n            &orders,\n            &offers,\n            &mut route_capacity,'

edit(P2, [(OLD4, NEW4), (OLD5, NEW5)])

print('all edits done')
