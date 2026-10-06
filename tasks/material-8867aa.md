---
id: material-8867aa
title: Aperiodic backdrop for the parameter sweep
status: todo
priority: 2
size: s
complexity: mid
process: direct
created: 2026-09-07T00:03:24Z
updated: 2026-10-06T19:24:25Z
depends: []
parent: material-49871a
tags: [harness, rendering]
---

Why: the periodic grid aliases displacement, while material-bb8480 has already calibrated a deterministic grayscale aperiodic field. Supplement the sweep with that existing artifact rather than inventing another generator or tinting it. Keep the colored grid for historical and color-sensitive sweeps.

Done: glass-parameter-sweep.sh accepts explicit BACKDROP=grid20 or BACKDROP=aperiodic-s20260929; retain grid20 as the default and its existing generator unchanged. The aperiodic option calls warp-calibration.py backdrop with the calibrated seed and writes the same 1280x720 grayscale PNG. Reject unknown names before launching a compositor. Record the selected name, seed where applicable, generator identity and generated PNG hash in retained run metadata, and show backdrop identity beside the table. TABLE_ONLY reads retained identity and never regenerates or relabels captures from the current environment; for older records without identity, explicitly report it as unrecorded. Preserve prior table columns and never imply that aperiodic RMSE is monotonic displacement or proof of bending. Reject the grayscale stimulus for KEY=saturation because it cannot excite saturation; retain grid20 for that measurement.

Where to look: docs/materials/scripts/glass-parameter-sweep.sh::make_backdrop and TABLE_ONLY/table output; docs/materials/scripts/warp-calibration.py::backdrop; tools/test_warp_calibration.py; docs/materials/2026-10-06-glass-warp-calibration-evidence.md; docs/specs/2026-09-06-glass-parameter-sweep-design.md. Update the sweep guide/spec and its misleading periodic-is-fine-for-RMSE comment when implementation lands.

Check: through just test-one with the Python one_cmd override, add a focused offline shell/generator check for option routing, deterministic grayscale artifact/seed, invalid names, grayscale saturation refusal, retained TABLE_ONLY identity and unchanged grid20 behavior. Stub compositor launches; no new live parameter sweep is required for this wiring task. Then use the full tooling test-fast override before committing. No new dependency or backdrop algorithm.

Out of scope: affine fitting, a rendered bending claim, colorizing the calibrated field, a full GPU sweep, and repinning src/tests/ring_look.rs's owner-accepted look.

## Original captured request

glass-parameter-sweep.sh paints a 20px grid backdrop. material-48dc76's thickness run showed that cumulative RMSE against the first capture aliases with that period for displacing parameters, while neighbouring deltas are only partly exposed. Replace or supplement the grid with an aperiodic field (blue noise, or a texture with no repeat within the maximum displacement) so cumulative is valid for thickness and any future displacing key. Keep the current backdrop reproducible: generate it deterministically. Depends on whether material-343f27 builds its own instrument first; if it does, this may fold into it.

## Notes

- 2026-09-29T22:38:51Z (materials-26.04): scope: briefed; 20 px backdrop still aliases displacement; share backdrop calibration with the ray-bending study before selecting a sweep change; brief: docs/notes/2026-09-29-glass-measurement-brief.md
- 2026-10-02T00:54:22Z (materials-26.04): Consumer: src/tests/ring_look.rs (accepted_ring_look) renders over flat gray, so refraction of the backdrop never shows in the accepted-look reference. An aperiodic backdrop there would let it cover edge refraction (material-be611b) too.
- 2026-10-06T17:27:19Z (material-bb8480): material-bb8480 (2026-10-06): reuse its artifact rather than a separate backdrop. 'warp-calibration.py backdrop <png>' writes the calibrated field (aperiodic-s20260929, 1280x720, seed and octaves in a tEXt chunk); it has no repeat within the 24 px search and reads a 20 px shift exactly, where the grid refuses every pixel. Keep grid20 available by name so earlier sweep tables stay comparable, and record the backdrop name in each run's provenance. Open choice for this task: colour. The PNG is grey because tinting it by the grid's quadrant colours clips the bright quadrants, which the estimator flags as unfittable.
- 2026-10-06T19:24:24Z (materials-26.04): scope: scoped; adopt the calibrated grayscale backdrop as an explicit sweep option; retain grid20 default and color-sensitive stimulus, preserve table-only provenance and historical tables; P2 small mid direct; brief: docs/notes/2026-09-29-glass-measurement-brief.md
