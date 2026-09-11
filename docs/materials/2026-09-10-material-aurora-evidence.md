# Glass aurora: verification evidence

**Result:** PASS, 2026-09-10, headless Weston 15.0.1 using Mesa llvmpipe.
All captures and traces completed without material renderer errors, shader
fallbacks, or panics in `niri.log`.

## Pinned revisions

- Source commit `07163e5b37a42c94ad53ae04e6430e24a7cc15df`.
- Release binary SHA-256
  `c8308145313e7c0c1ac0fde7410f7db20fadf24cdf8ae192d2f6d4504f85946d`.
- Tracy build SHA-256
  `84660e29599a3efd620595c236d44bfe0a43f3ff6eb1dbfa187f1e5315d1b0f4`.

## Metrics

| Metric | Value |
| --- | ---: |
| `zero_vs_plain_ae` | 0 |
| `pinned_ae` | 0 |
| `face_rmse` | 6039.94 |
| `lit_face_mean` | 0.540334 |
| `zero_face_mean` | 0.464484 |
| `one_code` | 257 |
| `moving_ae_1` | 0 |
| `moving_ae_2` | 0 |
| `moving_ae_3` | 0 |
| `within_bucket_min_ae` | 0 |
| `across_bucket_ae` | 651.984 |
| `redraws_20s_plain` | 0 |
| `redraws_20s_pinned` | 0 |
| `redraws_20s_4hz` | 80 |
| `redraws_20s_reduced` | 40 |
| `redraws_20s_off` | 0 |
| `gpu_plain_ms` | 4.308 |
| `gpu_zero_ms` | 3.888 |
| `gpu_on_ms` | 5.552 |
| `gpu_zero_vs_plain_pct` | -9.8% |
| `gpu_on_vs_plain_pct` | +28.9% |

The amount-zero capture matched the unconfigured material decoded pixel for
pixel (AE 0), and two captures with `drift-hz 0` also matched (AE 0). The
pinned field changed the face by 6039.94 Q16 RMSE and raised its mean from
0.464484 to 0.540334.

The scheduling check from the optics design §11 passed on an unfocused,
signal-free window. All three consecutive 1 Hz pairs were identical within
their bucket (AE 0), while the capture 2.5 seconds later changed by 651.984.

Over the final 20 seconds, plain and pinned windows caused 0 redraws, the 4 Hz
field caused 80 (4/s), reduced motion caused 40 (2/s), and motion off caused
0. The material draw cost 4.308 ms plain, 3.888 ms at amount 0, and 5.552 ms
lit at 4 Hz, a -9.8% and +28.9% delta from plain respectively. These are
llvmpipe software-renderer measurements; they do not establish physical-GPU
cost.

The window measured `456x640+40+40` in screenshot space. The face ROI was
`200x400+100+160`.

## Preset tuning

| Value | Step | Bevel neighbor | Bevel per step | Bevel cumulative | Bevel normalized | Face neighbor | Face per step | Face cumulative | Face normalized |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | - | - | - | 0 | - | - | - | 0 | - |
| 0.2 | 0.2 | 0.0343366 | 0.171683 | 0.0343366 | 1.0000 | 0.0339464 | 0.169732 | 0.0339464 | 1.0000 |
| 0.35 | 0.15 | 0.0177961 | 0.118641 | 0.0547975 | 0.6910 | 0.0174992 | 0.116661 | 0.0540751 | 0.6873 |
| 0.5 | 0.15 | 0.0154905 | 0.10327 | 0.0725679 | 0.6015 | 0.0151066 | 0.100711 | 0.0714082 | 0.5934 |
| 0.7 | 0.2 | 0.0180843 | 0.0904215 | 0.0932882 | 0.5267 | 0.0176138 | 0.088069 | 0.0915897 | 0.5189 |
| 1 | 0.3 | 0.0233522 | 0.0778407 | 0.120016 | 0.4534 | 0.0227062 | 0.0756873 | 0.11758 | 0.4459 |

The face response had not flattened by 0.5: its normalized per-step change
was 0.5934 there and remained 0.4459 at amount 1. The `aurora` preset therefore
keeps amount 0.5. Its capture reads as a soft green-to-violet wash inside the
slab rather than a flat tint or patchy field.

The field loops over a circle of radius 2.0 noise units, about 5 px/s at the
0.004 scale. The design's original radius of 40 would move about 105 px/s and
was therefore too fast for a slow field.

Captures, traces, logs, hashes, and sweep data are retained at
`/mnt/ssd3/tmp/glass-aurora-07163e5b-20260910T211100`; the smoke's complete
stdout and stderr are retained beside it in
`glass-aurora-07163e5b-20260910T211100.runner.log`, and the sweep runner log is
`sweep.runner.log` inside the run directory.
