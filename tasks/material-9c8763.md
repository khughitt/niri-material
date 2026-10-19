---
id: material-9c8763
title: "Review the glass-facts contract with prism, then generate and install the facts bundle"
status: todo
priority: 2
size: m
complexity: mid
process: planned
created: 2026-10-19T11:55:19Z
updated: 2026-10-19T11:55:19Z
depends: []
tags: []
source: ops-630a87 boundary-proposal-r3.md A2
agent: claude-code/claude-opus-5-5
---

Why: prism keeps byte copies of material's glass schema (defs/rack/pipeline.json, glass.yaml, dry.yaml) and compares them against material's checkout, while material's package installs KDL only, not a facts bundle (revision 3, A2). Contract review first, with prism: the artifact's format, install path and versioning, how prism reads it in place of the checkout, and the transition while both copies exist. Then material generates the bundle from params.rs and pipeline.rs and installs it with the package. Done: reviewed contract; the package installs the bundle; material's tests pin fixture or neutral values, not the live look. Before landing, pass the flow-trial-1 gate (revision 3, change 5): judge the step's effect on what enrolled sessions see and what obs and the verdict tools measure; a step that changes either waits for readout (2027-02-16).
