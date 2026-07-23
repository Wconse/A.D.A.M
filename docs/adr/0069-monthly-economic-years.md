# ADR 0069: Monthly economic years with annual closure

- Status: Accepted foundation
- Date: 2026-07-23

A complete economic year now consists of exactly twelve monthly economic cycles followed by one annual demographic, regional-output, fiscal, and political closure. `AdvanceEconomicYear` executes the whole year atomically; `advance_economic_years` commits completed years in sequence and leaves the first failing year unchanged.

Annual planning now receives the closed year explicitly. After twelve calendar advances move the date into the following year, demographic, economic, and political RNG streams still use the year whose twelve months were simulated. This prevents an off-by-one change in deterministic history. Legacy `advance_one_year` retains its prior behavior through the same annual closure implementation.

The last closed annual year is authoritative save state and participates in stable fingerprints. Duplicate closure for the same year is rejected. Detailed monthly events remain in the archive, while `EconomicYearCompleted` records the closed year and the required twelve monthly cycles alongside the existing `YearAdvanced` event.

A deterministic fifty-year economic test now executes 600 monthly cycles and 50 annual closures. Firm operating history remains bounded to its latest twelve observations even though the append-only event archive retains the full sequence.
