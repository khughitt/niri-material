---
id: material-343f27
title: Validated measurement for glass ray bending
status: idea
priority: 2
created: 2026-09-06T11:12:33Z
updated: 2026-09-06T11:12:33Z
depends: []
tags: [rendering, harness]
---

material-37cec9 reports WHERE the rendered image changes; it deliberately does not claim WHY. A bevel-region RMSE is consistent with backdrop refraction being broken: Fresnel f0=(ior-1)/(ior+1) varies with ior everywhere coverage>0 (material.frag:454) and the focus filament refracts through 1.0+(mat_ior-1.0)*mat_light_ior (:342). Two instruments were tried and rejected. Template matching (magick compare -subimage-search, NCC on a unique marker in a constrained window) passed its own synthetic translation and brightness self-checks but reported 0,0 displacement across ior 1 to 3: the template spanned x35-65 while the chamfer was only 40-52, so 18 of 30 columns could not bend at distortion 0 and pinned the match. Widening the bevel or shrinking the template does not fix the deeper problem that refraction through the chamfer is a spatially varying warp, not a rigid translation, and the flat face - the only region where displacement would be rigid - does not bend at distortion 0. A flat-versus-textured backdrop control was also run (see the evidence doc) but subtracting RMSE values does not isolate bending: RMSE combines effects nonlinearly. Needs an instrument whose error is quantified before its output is read as bending.
