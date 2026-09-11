# Glass iridescence: verification evidence

**Result:** PASS, 2026-09-10, headless Weston 15.0.1 using Mesa llvmpipe.
All captures and traces completed without material renderer errors, shader
fallbacks, or panics in `niri.log`.

## Pinned revisions

- Source commit `48d42c395b31aedba7f95c37dce2fdfe0b7dc4ad`.
- Release binary SHA-256
  `399e23b76eab3583386305096a37545f12e1d6b3e7657a19129a30abfcb15ff0`.
- Tracy build SHA-256
  `34db6f1aabb1a1a246ec857bcb48c1e616cdda4d980a89519ca84b74eb86600c`.

## Metrics

| Metric | Value |
| --- | ---: |
| `zero_vs_plain_ae` | 0 |
| `chamfer_ab_rmse` | 621.468 |
| `face_ab_rmse` | 98.9579 |
| `chamfer_rmse` | 1113.34 |
| `face_rmse` | 257 |
| `one_code` | 257 |
| `gpu_plain_ms` | 4.070 |
| `gpu_zero_ms` | 4.014 |
| `gpu_on_ms` | 3.918 |
| `gpu_zero_vs_plain_pct` | -1.4% |
| `gpu_on_vs_plain_pct` | -3.7% |

The zero capture matched the unconfigured material decoded pixel for pixel
(AE 0). The visible chamfer gained 621.468 Q16 of Oklab a/b RMSE against
98.9579 on the face. The material draw cost 4.070 ms plain, 4.014 ms at
amount 0, and 3.918 ms at amount 0.8. These are llvmpipe software-renderer
measurements; they do not establish physical-GPU cost.

The window measured `456x640+40+40` in screenshot space. The face ROI was
`200x400+100+160`; the visible chamfer ROI was `20x400+36+160`. The
headless output transform made the brief's proposed right-edge band nearly
flat: its RGB RMSE was 257 Q16, against 964.178 for the visible left-edge band
and 222.569 for the diagnostic face crop. The smoke therefore samples the
visible chamfer at the measured window's left edge.

## Preset tuning

| Value | Step | Bevel neighbor | Bevel per step | Bevel cumulative | Bevel normalized | Face neighbor | Face per step | Face cumulative | Face normalized |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | - | - | - | 0 | - | - | - | 0 | - |
| 0.2 | 0.2 | 0.00410682 | 0.0205341 | 0.00410682 | 0.9796 | 0.00210914 | 0.0105457 | 0.00210914 | 0.8921 |
| 0.4 | 0.2 | 0.0039156 | 0.019578 | 0.00769681 | 0.9339 | 0.00160759 | 0.00803795 | 0.00288245 | 0.6800 |
| 0.6 | 0.2 | 0.00409275 | 0.0204637 | 0.0115789 | 0.9762 | 0.00209733 | 0.0104867 | 0.00449334 | 0.8871 |
| 0.8 | 0.2 | 0.00378792 | 0.0189396 | 0.0150017 | 0.9035 | 0.00162524 | 0.0081262 | 0.00497495 | 0.6874 |
| 1 | 0.2 | 0.00419253 | 0.0209626 | 0.0189462 | 1.0000 | 0.00236418 | 0.0118209 | 0.0068536 | 1.0000 |

The bevel response did not flatten by 0.6; its normalized per-step change
remained between 0.9035 and 1.0000 through amount 1. The `rainbow` preset
therefore keeps `iridescence 0.8`.

Captures and traces are retained at
`/mnt/ssd3/tmp/glass-iridescence-48d42c39-20260910T204246`;
`SHA256SUMS` there lists every capture. The original runner log is retained
as `runner.invalid.log`: its trace files and visual measurements are valid,
but a missing newline concatenated each case's three GPU samples and made its
reported GPU metrics invalid. The corrected `task-7-replay.sh` re-exported
the same nine traces into three newline-delimited samples per case; its output
is `replay.log`, and `metrics.txt` contains the corrected results. The replay
command was:

```bash
OUT=/mnt/ssd3/tmp/glass-iridescence-48d42c39-20260910T204246 \
NIRI_MATERIAL_WORK_ROOT=/mnt/ssd3/niri-material \
bash task-7-replay.sh
```
