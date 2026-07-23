# ADR 0068: Calendar-month economic cycle

- Status: Accepted foundation
- Date: 2026-07-23

The simulation now has deterministic calendar-month advancement over the simplified 365-day calendar. Month lengths follow the civil 31/28/31/30 pattern; advancing a month preserves the day when possible and clamps it to the target month. Twelve advances from January 1 reach January 1 of the next year.

`ExecuteMonthlyEconomicCycle` atomically sequences payroll, household cashflows, the commercial cycle, cohort experience, derived social stress, stress memory, and one calendar-month advance. The command runs on a cloned world and commits only after every stage succeeds. Detailed stage events remain authoritative, supplemented by monthly economic and calendar-advance summary events.

Payroll, household cashflow, commercial settlement, and cohort-experience updates may each execute at most once per simulation date. Their completion dates are serialized and included in stable fingerprints. A partially pre-executed month causes the aggregate cycle to fail without repeating money flows or changing the original world.

Household demand is now funded from concrete liquid wealth after payroll and household cashflows rather than recalculating purchasing power from annual income. This removes phantom duplicate income: annual fallback income enters wealth through the cashflow stage, contracted wages enter through payroll, and market settlement spends the resulting balance.
