# ADR 0100: Owner-funded firm recapitalization

## Status

Accepted.

## Context

The monthly economy already records firm cash, owner cash, ownership rights, payroll obligations, and persistent wage arrears. A cash-exhausted firm could nevertheless remain in an indefinite half-alive state: wages accumulated without a governed financing response, while workers retained claims and production decayed. The next Stage 0 gate requires firm distress to create a real decision and resource transfer before introducing abstract bank credit or costless rescue.

## Decision

A firm enters observable distress when its employment agreements carry unpaid wage arrears after payroll. The world stores a canonical consecutive-distress counter per firm. A clean payroll removes the counter.

After three consecutive distressed payrolls, the owner with the greatest economic stake may recapitalize the firm from authoritative personal cash. Equal stakes resolve by the lowest stable actor id. The transfer is bounded by both available owner cash and total wage arrears. It debits owner cash, credits firm cash, resets the distress counter, and emits `FirmRecapitalized` with the owner, firm, amount, and outstanding worker claim.

Recapitalization occurs immediately after payroll in the atomic monthly economic cycle. It does not erase or directly settle arrears: workers keep their full claim, and the injected liquidity reaches them only through ordinary payroll in a later cycle. If no owner has cash, no synthetic money appears and distress remains capped at the trigger threshold for reconsideration.

## Consequences

- Firm survival now competes with owner liquidity; rescue is neither free nor guaranteed.
- Wage arrears become causal financing evidence instead of a passive statistic.
- Worker claims remain senior in the modeled causal chain because recapitalization cannot cancel them.
- The rule is deterministic, serialized, replayable through the monthly command, and fingerprinted.
- The aggregate owner is a Stage 0 approximation. Shareholder votes, dilution, creditor priority, secured lending, insolvency, asset sales, and closure remain later gates.
