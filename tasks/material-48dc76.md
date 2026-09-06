---
id: material-48dc76
title: Run parameter sweeps for the remaining Prism slider candidates
status: todo
priority: 2
size: s
created: 2026-09-06T11:29:42Z
updated: 2026-09-06T11:29:42Z
depends: []
tags: [harness, prism]
---

material-37cec9 shipped docs/materials/scripts/glass-parameter-sweep.sh and the ior run. prism-5758d3 also names depth, blur, noise and saturation. Run the sweep for glass thickness (Prism's depth), glass noise, glass saturation, and the blur block's noise/saturation/passes/offset - the blur ones exercise the BLOCK=blur path and its backdrop-blur gate, which has not been run yet. Record each table in the evidence doc alongside the ior run. Flex and ripple stay out: material-36e968. Reading a table as evidence of ray bending stays out: material-343f27.
