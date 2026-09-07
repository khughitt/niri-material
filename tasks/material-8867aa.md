---
id: material-8867aa
title: Aperiodic backdrop for the parameter sweep
status: idea
priority: 2
created: 2026-09-07T00:03:24Z
updated: 2026-09-07T00:03:24Z
depends: []
tags: [harness, rendering]
---

glass-parameter-sweep.sh paints a 20px grid backdrop. material-48dc76's thickness run showed that cumulative RMSE against the first capture aliases with that period for displacing parameters, while neighbouring deltas are only partly exposed. Replace or supplement the grid with an aperiodic field (blue noise, or a texture with no repeat within the maximum displacement) so cumulative is valid for thickness and any future displacing key. Keep the current backdrop reproducible: generate it deterministically. Depends on whether material-343f27 builds its own instrument first; if it does, this may fold into it.
