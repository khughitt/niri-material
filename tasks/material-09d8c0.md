---
id: material-09d8c0
title: Detect when a packaged build can no longer validate what Prism emits
status: done
priority: 2
size: m
owner: materials-26.04
created: 2026-09-08T15:13:04Z
updated: 2026-09-09T09:40:00Z
depends: []
tags: [packaging, contract]
---

Found bringing europa (laptop) up to date on 2026-09-08 after ~3 weeks. All three packages in packaging/arch/ predated the noise type= config property Prism emits, so the newest available package could not have worked and nothing said so. Outcome: a check ties a built package to the config schema Prism generates and fails when they diverge.

## Notes

- 2026-09-09T00:32:28Z (materials-26.04): prism's probe-material covers a niri that does not know the material node at all; a niri-material build too old for a property prism emits (the type= on noise) stays this task's case
- 2026-09-09T09:40:00Z (materials-26.04): Landed in prism (5201259), where the check has to live: the divergence is between the installed package and what prism emits, and only prism knows the second half. integrations/niri/probe-material now renders its probe with renderNiriFragment from the defs' defaults, both glass states on, so it is the same text the sink writes and a new property is probed the moment it can be emitted. Verified against the installed package (accepts) and against a fake niri that knows the node and refuses type= (fails, naming the property).
- 2026-09-09T09:40:00Z (materials-26.04): prism's material probe is now the config prism emits, so a package too old for a property it emits fails the requirement instead of failing at apply time.
