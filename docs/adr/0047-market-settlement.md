# ADR 0047: Atomic local market settlement

- Status: Accepted foundation
- Date: 2026-07-22

Pre-cleared fills settle atomically on cloned household and firm ledgers. Buyers lose liquid wealth, sellers lose physical inventory and gain cash; any invalid fill aborts the entire batch. Successful trades emit typed events. Consumption memory and unmet-demand persistence follow separately.
