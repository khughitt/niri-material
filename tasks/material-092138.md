---
id: material-092138
title: Backfill complexity ratings for open tasks
status: todo
priority: 2
size: s
complexity: high
created: 2026-09-12T15:09:51Z
updated: 2026-09-12T15:09:51Z
depends: []
tags: []
source: tasks-be447b
---

Rate every open scoped task in this project — status todo, doing, or blocked; ideas are rated when they are scoped, not before — plus any recurring task (`tasks list --periodic`), with the rubric in the tasks skill: low = approach established, relevant context identified, correctness has a clear check; mid = bounded investigation or implementation choices remain, scope and acceptance criteria clear; high = substantial discovery, subtle reasoning about interacting behaviour, or an unresolved architectural call. Read each task body, its notes, and any linked spec or plan first, and rate the judgment that remains after that preparation — not the size: a large mechanical change is low, a one-line subtle fix can be high. `tasks list --status todo --status doing --status blocked` lists the set; `tasks edit <id> --complexity <level>` sets each. Done when `tasks ready --max-complexity high` reports no "unassessed hidden" warning. Rated high itself because rating is judgment work reserved for a frontier session (tasks docs/specs/2026-09-12-task-complexity-design.md §3.3); it must not be picked under a cutoff.
