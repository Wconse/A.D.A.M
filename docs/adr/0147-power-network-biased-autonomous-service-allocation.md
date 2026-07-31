# ADR 0147: Power-network-biased autonomous service allocation

## Status

Accepted.

## Context

Step 098 made regional service allocation an unrestricted, authorized command and introduced a prudent autonomous fallback. The fallback considered material and political outcomes but ignored the existing actor, political-office, home-region, and influence graph. That made formal and informal power irrelevant to a recurring distributive decision.

## Decision

The autonomous service allocator now treats existing political power as one decision input. Every holder of a country's political office contributes 1,000 decision-score points toward the holder's home region. Every established actor-to-office influence edge contributes half of its basis-point weight toward that actor's home region. The bonuses join population, service need, persistent regional interests, and low satisfaction before deterministic normalization.

Only political-office nodes in the allocating country participate. An actor whose home region is outside that country creates no regional patronage bonus. Each applied holder or influence contribution is emitted as typed evidence with actor, office, home region, mechanism, source weight, and score bonus. The chronicle names the strongest actor and favored region.

These weights are autonomous decision policy, not engine legality. An explicit authorized allocation bypasses the autonomous weights completely, including all power-network bonuses. No influence edge can overrule or invalidate a player's feasible command, create money, alter the spending envelope, or evade administrative delivery constraints.

## Consequences

Office holders tend to remember their political base, and actors with access to the office can redirect marginal service spending toward their home regions. Material need can still outweigh weak influence, while concentrated influence can produce visible favoritism. Later actor traits may change how strongly a ruler responds to these pressures without changing the shared command rules.
