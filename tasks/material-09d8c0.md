---
id: material-09d8c0
title: Detect when a packaged build can no longer validate what Prism emits
status: todo
priority: 2
size: m
created: 2026-09-08T15:13:04Z
updated: 2026-09-08T15:13:04Z
depends: []
tags: [packaging, contract]
---

Found bringing europa (laptop) up to date on 2026-09-08 after ~3 weeks. All three packages in packaging/arch/ predated the noise type= config property Prism emits, so the newest available package could not have worked and nothing said so. Outcome: a check ties a built package to the config schema Prism generates and fails when they diverge.
