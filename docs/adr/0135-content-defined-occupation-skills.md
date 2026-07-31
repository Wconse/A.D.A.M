# ADR 0135: Content-defined occupation skills

## Status
Accepted.

## Decision
Recipes may optionally require a typed `SkillId` in addition to education. Cohorts retain bounded proficiency by skill. Hiring and switching validate both requirements; recipes without a skill remain backward-compatible. Skill requirements and proficiencies are serialized and fingerprinted.

## Consequences
Equal education no longer makes unrelated professions interchangeable. Occupational evidence and skill-producing curricula can build on the new typed identity without hard-coded professions.
