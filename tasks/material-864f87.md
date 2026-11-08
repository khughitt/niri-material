---
id: material-864f87
title: "Material cost study: quiet pilot and capture matrix"
status: todo
priority: 2
size: m
complexity: mid
process: direct
needs: [quiet, nested]
created: 2026-10-15T18:41:01Z
updated: 2026-10-15T18:41:01Z
depends: [material-0df22c]
parent: material-31074f
tags: [performance]
agent: claude-code/claude-opus-5-5
---

Why: the measurement half of material-31074f, after the design and preparation in material-0df22c.

Done: the pilot runs every gate end to end and is read before the full run; the matrix runs under the designed repeat protocol on a quiet host, from a TTY with the desktop stopped (AGENTS.md session protocol; display-dim around the runs); every attempt gets a run: note; the results are written to the evidence doc named by the design.
