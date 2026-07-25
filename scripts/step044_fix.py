import io

P = 'crates/adam-core/src/commerce.rs'
with io.open(P, encoding='utf-8') as f:
    c = f.read()

old = '        let demand_intents =\n            next.plan_monthly_household_demand_against_offers(&offers, &mut route_capacity)?;'
new = ('        // Demand planning receives a clone: it consumes capacity internally to model\n'
       '        // first-canonical-buyer reservation during budgeting. The original\n'
       '        // post-procurement route_capacity is kept intact for market clearing,\n'
       '        // where actual import fills are bounded by whatever procurement left unused.\n'
       '        let demand_intents = next.plan_monthly_household_demand_against_offers(\n'
       '            &offers,\n'
       '            &mut route_capacity.clone(),\n'
       '        )?;')

assert c.count(old) == 1, f'expected 1, got {c.count(old)}'
c = c.replace(old, new)
with io.open(P, 'w', encoding='utf-8', newline='\n') as f:
    f.write(c)
print('OK commerce.rs')
