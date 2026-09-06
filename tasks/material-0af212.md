---
id: material-0af212
title: Ring-of-light drift may make existing capture-harness assertions flaky
status: doing
priority: 1
size: s
owner: harness/ring-drift
created: 2026-09-06T11:29:23Z
updated: 2026-09-06T11:41:31Z
depends: []
tags: [harness, bug]
---

The focus response defaults to RingLight, which drifts over time and is drawn at the window edge. In material-37cec9 this moved a window-edge ROI between two runs of the same sweep by Lab RMSE 0.077 while the interior stayed byte-identical; frame-wide about 171 of 921600 pixels differed, all at the edge. docs/materials/scripts/glass-noise-saturation-smoke.sh predates the ring merge (f8bcb34c), defines no response block, and asserts exact equality on full-frame captures - assert_zero omitted_identity_ae compares two whole PNGs, and its ROI/channel assertions cover the window. Those assertions can now differ run to run for reasons unrelated to what they test. Check whether that script (and focus-glass-spike.sh, material-signals-smoke.sh) still pass reliably, pin response default focus none wherever the ring is not the thing under test, and re-check any recorded figure taken at a window edge after f8bcb34c. focus-ring-light.sh is exempt: the ring is its subject.
