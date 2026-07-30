# ADR 0124: Shortage-driven local firm entry

## Status

Accepted for Stage 0.

## Context

Persistent survival shortages could alter public reserves, imports, health, debt, and politics, but private productive capacity remained fixed except for investment by firms that already existed. A region with an applicable technology, idle labor, and wealthy local actors could therefore fail forever without anyone attempting a new enterprise.

Creating firms directly from unmet demand would be a shortcut. Entry must require persistent evidence, a known recipe, observed prices, an actual founder, available labor, and enough liquid capital. It must also remain replayable and must not mint cash, inventory, or output.

## Decision

After household trade, rationing, and reserve release, the monthly economic cycle aggregates residual survival shortages by region and good. Three consecutive months of positive residual shortage create a mature entry opportunity. A cleared month resets the streak.

For a mature opportunity, planning selects deterministically:

1. the lowest-ID existing recipe that produces the missing good;
2. the unemployed local cohort with the most unallocated workers, ties by cohort ID;
3. an observed startup wage from that cohort's annual per-person income, with a one-minor-unit floor;
4. installed-capacity cost equal to two observed output-batch values;
5. working capital equal to one input batch plus three months of the startup wage;
6. the wealthiest local actor able to fund the complete commitment, ties by actor ID;
7. the next canonical firm ID.

The founding crosses `WorldCommand::FoundFirm`. The founder pays both capacity and working-capital commitments. Capacity spending leaves liquid portfolios as abstract construction expenditure; working capital becomes firm cash. The new producer starts with one installed batch, one employed local worker, full founder ownership, a chief-executive appointment, a bounded default policy, and a one-batch production target. It starts with no inventory and must acquire any inputs through ordinary procurement.

If recipe, observed prices, unemployed labor, or founder capital are missing, no firm appears and no resources move. Opportunity pressure remains, allowing a later change in regional conditions to unlock entry.

Typed events record every opportunity review, founding, capital split, wage, worker cohort, technology, and completion count. Pressure, firms, governance, employment, cash, commands, serialization, replay, fingerprints, and chronicle narration are authoritative.

## Invariants

- One shortage month cannot found a firm.
- Only residual survival shortage after trade and public response counts.
- A firm cannot appear without a registered technology and observed local prices.
- Founders cannot spend cash they do not own.
- Entry requires an unemployed local worker and never exceeds cohort population.
- Installed capacity creates neither inventory nor immediate output.
- Working capital is conserved from founder cash into firm cash; capacity cost is explicit investment expenditure.
- Autonomous and external founding use the same command boundary.
- Canonical ordering makes equal worlds produce equal entrants and fingerprints.

## Consequences

The economy can now begin rebuilding local production instead of relying only on imports and the state. Shortage becomes an entrepreneurial signal, but capital scarcity, missing technology, and labor scarcity remain real failure modes. Founders gain ownership and control, creating future political and class consequences.

Deliberate limits: firms enter at a fixed one-batch scale; founder risk preference, competing plans, construction time, skill matching, bank-financed startups, and post-entry survival are not yet modeled. The next gate should deepen labor matching so new and incumbent firms compete for workers and wages rather than receiving only the first unemployed worker.
