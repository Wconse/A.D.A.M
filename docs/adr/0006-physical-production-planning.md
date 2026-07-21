# ADR 0006: Physical production planning

- Status: Accepted foundation
- Date: 2026-07-21

Production is represented by firms, fixed physical recipes, labor requirements, capital batch capacity, intermediate inputs, cash, and inventories. Monthly planning computes the maximum feasible batches as the minimum of labor capacity, capital capacity, and every intermediate-input constraint. Planning is deterministic and read-only; inventory consumption, wages, sales, expectations, ownership, and bankruptcy remain future transactional systems.

This slice deliberately proves physical constraint propagation before adding profit optimization. A firm cannot replace missing energy or materials with money, nor exceed assigned labor or installed capacity.
