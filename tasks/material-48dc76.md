---
id: material-48dc76
title: Run parameter sweeps for the remaining Prism slider candidates
status: done
priority: 2
size: s
owner: materials-26.04
created: 2026-09-06T11:29:42Z
updated: 2026-09-06T23:39:12Z
depends: []
tags: [harness, prism]
---

material-37cec9 shipped docs/materials/scripts/glass-parameter-sweep.sh and the ior run. prism-5758d3 also names depth, blur, noise and saturation. Run the sweep for glass thickness (Prism's depth), glass noise, glass saturation, and the blur block's noise/saturation/passes/offset - the blur ones exercise the BLOCK=blur path and its backdrop-blur gate, which has not been run yet. Record each table in the evidence doc alongside the ior run. Flex and ripple stay out: material-36e968. Reading a table as evidence of ray bending stays out: material-343f27.

## Notes

- 2026-09-06T23:31:09Z (materials-26.04): Scope check against prism defs/glass.yaml: Prism's Blur slider is glass.roughness [0,1], not the blur block, so roughness is swept too. The blur block sweeps stay, as written, to exercise BLOCK=blur. Binary: installed /usr/bin/niri 5dbe182d, the same as the ior run; no rendering-relevant file changed between it and HEAD 740d062f. Values span each Prism range end to end, denser at the low end.
- 2026-09-06T23:39:12Z (materials-26.04): Eight sweeps run on /usr/bin/niri 5dbe182d: glass thickness, roughness (Prism's Blur slider), noise, saturation, and blur noise, saturation, passes, offset via BLOCK=blur. Recorded as Part 2 of docs/materials/2026-09-06-glass-parameter-sweep-evidence.md with a per-slider summary for prism-5758d3: roughness is the steepest (61% of its change in the first tenth, log-scale candidate), thickness is front-loaded at the edge with a useful region of 0-40 of 200, noise is near linear, saturation is linear and clips past 2.5. BLOCK=blur path verified: inherited noise/saturation reproduce the written pair's shape. Instrument caveat found: the 20px periodic backdrop makes cumulative RMSE unreliable for displacing parameters (thickness); filed under material-343f27's scope in the doc. Roughness re-run byte-identical.
