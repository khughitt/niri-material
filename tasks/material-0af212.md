---
id: material-0af212
title: Ring-of-light drift may make existing capture-harness assertions flaky
status: done
priority: 1
size: s
owner: harness/ring-drift
created: 2026-09-06T11:29:23Z
updated: 2026-09-06T12:01:34Z
depends: []
tags: [harness, bug]
---

The focus response defaults to RingLight, which drifts over time and is drawn at the window edge. In material-37cec9 this moved a window-edge ROI between two runs of the same sweep by Lab RMSE 0.077 while the interior stayed byte-identical; frame-wide about 171 of 921600 pixels differed, all at the edge. docs/materials/scripts/glass-noise-saturation-smoke.sh predates the ring merge (f8bcb34c), defines no response block, and asserts exact equality on full-frame captures - assert_zero omitted_identity_ae compares two whole PNGs, and its ROI/channel assertions cover the window. Those assertions can now differ run to run for reasons unrelated to what they test. Check whether that script (and focus-glass-spike.sh, material-signals-smoke.sh) still pass reliably, pin response default focus none wherever the ring is not the thing under test, and re-check any recorded figure taken at a window edge after f8bcb34c. focus-ring-light.sh is exempt: the ring is its subject.

## Notes

- 2026-09-06T12:01:13Z (harness/ring-drift): Audit run on a0559ebf, headless weston, two independent runs of each script. Only focus-glass-spike.sh is affected: as shipped two runs differed by AE 994 full frame and Lab RMSE 0.015 on the bottom corner crop. Attribution isolated - adding animations off gives AE 0, and pinning focus none with animations left on also gives AE 0, so the ring is the source and drift is the mechanism. Fixed with animations off, which keeps the ring visible but static; that matches the other capture scripts and the ring is part of what focus now looks like. Residual after the fix is one 7x6 character cell inside the newly focused terminal where kitty repaints its cursor, ~195px in from the edge and outside every crop the script takes; all six crops are AE 0.
- 2026-09-06T12:01:13Z (harness/ring-drift): The task premise is wrong and the docs it came from have been corrected. glass-parameter-sweep.sh and glass-noise-saturation-smoke.sh both set animations off, and drift_rate returns 0 when animations are off (src/render_helpers/signal.rs:233-236), so the ring cannot drift in either. Measured: two runs of the sweep with the ring left at its default are AE 0 with identical sweep.tsv, and so are two runs of the pre-pin script d69fd99b, which still carried the backdrop marker. 0b6b4ce3 pinned the ring and removed that marker in one commit, so the pin was never isolated; neither change is needed for reproducibility. The 0.077 did not reproduce and its cause is not established. The pin stays as a precaution, with the justification rewritten in the script, the design spec and the evidence doc.
- 2026-09-06T12:01:13Z (harness/ring-drift): glass-noise-saturation-smoke.sh is not flaky - it passed twice with metrics identical to the last digit - but a separate defect surfaced: assert_zero omitted_identity_ae compares full frames from BASE and IMPL, and a BASE older than f8bcb34c draws no ring while IMPL does. The ring's own contribution in that fixture measures AE 3675, so the assertion cannot hold. It cannot be pinned around either: FocusResponse does not exist before f8bcb34c, so writing focus none into the shared config would make BASE fail validation instead. Both binaries in the recorded evidence run (7185ee51, ff2b922d) predate the ring, so that run stays valid on its own terms. Constraint documented in the script header and both evidence docs. material-signals-smoke.sh ipc mode passes and has no pixel assertions.
