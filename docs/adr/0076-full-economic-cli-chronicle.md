# ADR 0076: Stage 0 CLI runs the full economic loop and prints a causal chronicle

- Status: Accepted foundation
- Date: 2026-07-23

The console executable no longer advances the legacy annual-only simulation. It now calls `advance_economic_years`, so every requested year executes twelve monthly cycles—income, household coping, production, market clearing, firm management, health, social memory, emergency policy—and then the annual demographic, output, fiscal, and political closure.

A deterministic `World::chronicle` read model aggregates the append-only event archive into yearly entries. It reports only facts carried by authoritative events: minimum survival fulfillment, excess deaths, household survival borrowing, rationing, emergency transfers and debt issuance, produced and traded quantities, and annual political changes. Importance is derived deterministically from those facts. The read model does not mutate state and does not invent events or causal links.

The first real run against `world.example.toml` is intentionally diagnostic. A one-year run produced 884 events and reported zero minimum survival fulfillment followed by 3,834,272 excess deaths. A 50-year run produced 40,255 events and ended with regional populations 27, 40, and 33 while regional output continued growing and public debt exploded. The reason is now visible rather than hidden: the Stage 0 content blueprint defines cohorts, prices, countries, and offices but no firms, recipes, employment agreements, ownership, appointments, or production targets. The full monthly loop therefore has demand but no supply.

This result is not accepted as plausible history. It is a closure gate exposing that additional policy mechanics would be premature. The next mandatory slice is content support for a minimal producing economy and a revised example world. Only after the same 50-year chronicle contains production, trade, shortages, adaptation, and political response should procurement or deeper political mechanics resume.
