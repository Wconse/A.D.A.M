# ADR 0060: Explicit firm expectations

- Status: Accepted foundation
- Date: 2026-07-23

Firm expectations are authoritative knowledge state separate from realized monthly accounts. A forecast records expected sales revenue, input spending, available financing, horizon, and source. Updating it is a replayable command and typed historical event, but does not move cash, inventory, or workers.

The staffing planner uses realized payroll coverage when no forecast exists. When management expectations exist, it estimates payroll coverage from current cash plus expected sales and financing minus expected input costs, against wages and existing arrears over the forecast horizon. The result remains advisory: only an explicit employment command can hire or dismiss workers.

This preserves the causal chain `observed accounts -> expectations -> actor decision -> employment change` and allows temporary distress and expected deterioration to produce different proposals without introducing an abstract crisis variable.
