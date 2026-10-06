---
id: material-343f27
title: Validated measurement for glass ray bending
status: idea
priority: 2
created: 2026-09-06T11:12:33Z
updated: 2026-10-06T19:24:24Z
depends: []
parent: material-49871a
tags: [rendering, harness]
---

material-37cec9 reports WHERE the rendered image changes; it deliberately does not claim WHY. A bevel-region RMSE is consistent with backdrop refraction being broken: Fresnel f0=(ior-1)/(ior+1) varies with ior everywhere coverage>0 (material.frag:454) and the focus filament refracts through 1.0+(mat_ior-1.0)*mat_light_ior (:342). Two instruments were tried and rejected. Template matching (magick compare -subimage-search, NCC on a unique marker in a constrained window) passed its own synthetic translation and brightness self-checks but reported 0,0 displacement across ior 1 to 3: the template spanned x35-65 while the chamfer was only 40-52, so 18 of 30 columns could not bend at distortion 0 and pinned the match. Widening the bevel or shrinking the template does not fix the deeper problem that refraction through the chamfer is a spatially varying warp, not a rigid translation, and the flat face - the only region where displacement would be rigid - does not bend at distortion 0. A flat-versus-textured backdrop control was also run (see the evidence doc) but subtracting RMSE values does not isolate bending: RMSE combines effects nonlinearly. Needs an instrument whose error is quantified before its output is read as bending.

## Notes

- 2026-09-07T00:03:24Z (materials-26.04): From material-48dc76's thickness sweep: the sweep backdrop is a 20px periodic grid, so cumulative Lab RMSE against the first capture oscillates for any parameter that translates the backdrop (thickness cumulative 0.055 at 10, 0.025 at 20, 0.057 at 80, 0.028 at 120). A displacement instrument needs an aperiodic backdrop, or it will alias the same way; see the Instrument caveat section of docs/materials/2026-09-06-glass-parameter-sweep-evidence.md.
- 2026-09-29T22:38:51Z (materials-26.04): scope: briefed; regional Lab RMSE still cannot identify bending; bounded calibration research will test spatial warp against photometric-only controls; brief: docs/notes/2026-09-29-glass-measurement-brief.md
- 2026-10-06T17:27:19Z (material-bb8480): material-bb8480 (2026-10-06) calibrated an instrument for this: on the seeded aperiodic backdrop, ZNCC matching refined by Gauss-Newton (docs/materials/scripts/warp-calibration.py, 7 px window) reads chamfer shifts of 1-20 px to 0.02 px in the strip interior and rejects photometric-only change; windows across the strip edges are flagged and must be masked from the window rect and bevel. Still needed before a reading counts as bending: render the same glass over two uniform backdrops beside the textured one and divide out att = (R2-R1)/(g2-g1), spec = R1 - att*g1 per pixel (attenuation multiplies, the Fresnel glint adds, neither depends on backdrop content); ior 1 and thickness 0 are confounded controls. The in-process headless fixture behind src/tests/ring_pair.rs may host those renders without the idle host. Corners and distortion > 0 need material-79fb49's affine model. Evidence: docs/materials/2026-10-06-glass-warp-calibration-evidence.md.
- 2026-10-06T19:24:24Z (materials-26.04): scope: briefed; straight-edge estimator calibration is complete, but rendered linear-light flat-field controls must validate photometric isolation before interpreting shader displacement; one bounded rendered-control investigation follows; brief: docs/notes/2026-09-29-glass-measurement-brief.md
