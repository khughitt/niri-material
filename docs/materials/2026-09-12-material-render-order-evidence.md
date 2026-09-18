# Material render order: execution evidence

**Status:** behind implementation is committed at `e33aa968` and its
candidate pixel matrix plus formula regressions passed under the explicitly
approved GPU waiver for pixel scopes. Signal behavior and strict frame-cost
acceptance passed on 2026-09-18. The old-build baseline records the required
measured additive failure and identical repeat images. Task 2 implementation,
its linked within pixel matrix and strict within cost passed; older ring pixel
acceptance passed with an outer-settle-only limitation. Older aurora visual,
cadence, and strict cost-only evidence passed. Final software gates and review
passed; review and commit remain.

**Tasks:** `material-5b3107`; Task 1 (`material-f8b6e9`) completed at
`e33aa968`. Task 2 (`material-92edaf`) awaits review and commit.

## Capture readiness, 2026-09-12

Retained artifact: `$NIRI_MATERIAL_WORK_ROOT/render-order-readiness.pCbsOC/capture.json`.
SHA-256: `e56e846026688734b097b2a4e9113e6fe375610e2b220a5a95f0c0a7341c59bb`.
The record identifies host `titan`, run start `2026-09-12T16:32:19-04:00`,
the headless lane and Task 1. The separate readiness probe names the planned
smoke as its fixture; at that time the smoke had not been created or launched.

The command used the default 20-second sample and unmodified thresholds:

```bash
python3 tools/capture-meta preflight "$run_dir" --lane headless \
    --task material-f8b6e9 --fixture glass-render-order-smoke.sh \
    --owner-pid $$ --tool weston --tool kitty --tool tracy=0.13.1
```

Exit status was **1, preflight refused**, with these observations:

| Measure | Observed | Required |
| --- | --- | --- |
| CPU busy | 16.6% | <=10% |
| Load average | 5.53 | <=2 |
| GPU utilization | 26% | <=5% |
| GPU performance states | P3, P5, P8 | P8 throughout |
| GPU power interquartile range | 7.463 W | <=1 W |
| Compute clients | BitwigStudio | None |

The capture lock was released. No client was terminated and no threshold
was changed. **This is not the required old-shader additive failure.**
Resume with a fresh readiness record after the host is quiet, snapshot the
reviewed baseline, and obtain the actual additive regression before shader
implementation. Task 2 must still retain a positive wide-core face-strip
test; a reach upper bound alone would accept the old mask.

## Retry after closing the Bitwig window

Run start: `2026-09-12T17:05:22-04:00`. Retained artifact:
`$NIRI_MATERIAL_WORK_ROOT/render-order-readiness.03bDro/capture.json`.
SHA-256: `5a3905985deb6aee3b93c16604d0da1858de469a46f5d2819c1f5dd17bb6435a`.
The same 20-second command and default thresholds returned **1, preflight
refused**: CPU 18.3%, load 7.31, GPU utilization 41%, performance state P5,
and a BitwigStudio compute client. Power IQR was 0.838 W, below its 1 W
threshold this time.

An independent `nvidia-smi --query-compute-apps=pid,process_name,used_gpu_memory
--format=csv,noheader` query confirmed live BitwigStudio PID `2467300`,
using 293 MiB. The process table also showed its active audio engine PID
`2467825`. This was a remaining application process, not a stale probe
result. No process was terminated. The capture lock was released; Task 1
remains parked before the old-build additive capture and shader edits.

## Bitwig exited; remaining desktop GPU activity

The next direct compute-client query returned no clients. Two further
20-second preflights retained the same thresholds and released their locks:

| Artifact under `$NIRI_MATERIAL_WORK_ROOT` | Start, local time | CPU | Load | GPU | P-states | Power IQR |
| --- | --- | --- | --- | --- | --- | --- |
| `render-order-readiness.ZVFSho` | 2026-09-12 17:07:36 | 7.4% | 5.06 | 32% | P0/P3/P5/P8 | 21.633 W |
| `render-order-readiness.TFm260` | 2026-09-12 17:09:23 | 6.3% | 1.91 | 23.5% | P5/P8 | 0.537 W |

Both report no compute clients. The second run followed a settling interval:
CPU, load and power variance now pass. It still exits **1, preflight
refused** for GPU utilization above 5% and P-state not remaining P8.

SHA-256 of each retained `capture.json`, in table order:

```text
eeca0df08c0646f3140e01fbe4165d900a8e9d678156624ef0fc6f977883f5fb
2676565f614fa59d3bb2309ff0d1fb3594f8f8bdf4633dea3f055bea684a3e7e
```

A separate three-sample `nvidia-smi pmon -c 3 -s u` diagnostic observed
GPU activity in niri and Noctalia; this short sample does not attribute the
whole preflight median to either process. Desktop applications were left
running. These are still readiness refusals, not old-shader additive
regression failures. Task 1 remains parked before shader edits.

## Empty workspace and baseline build, 2026-09-16

The empty-workspace readiness probe `render-order-readiness.SxYENG` passed
unchanged thresholds: CPU 2.3%, load 0.53, GPU 3%, P8 throughout. Its
`capture.json` SHA-256 is
`c15bbcb58d356c86b0415af9832588038237df2c97f1b1409660320f61b2d297`.
This establishes a quiet desktop state; it does not establish which
off-workspace client or compositor operation caused the earlier GPU load.

Built from detached revision `522a09fe4d6785c402e99ba08d5da4b92dde080b`
in `.worktrees/material-5b3107-baseline`. Retained directory:
`$NIRI_MATERIAL_WORK_ROOT/render-order-baseline-522a09fe`.

| Binary | Build | SHA-256 |
| --- | --- | --- |
| `niri` | `cargo build --release` | `539969898744c39a7dead2f2d4ac83500d15c79f8fe3fbb3f4ca102678b6bac9` |
| `niri-tracy` | `cargo build --release --features profile-with-tracy` | `0afb49f21459f72794d3b64f24ac397d18686de1b48def867983855b37902bda` |

Both builds exited 0. Build logs, source revision and binary hash manifest
are retained beside the snapshots.

The new smoke's `PHASE=behind MODE=red` uses those snapshots, without
rebuilding. Its first launch accidentally placed its redirected log inside
`OUT`; the library correctly refused the nonempty directory. Retried with
the log outside `OUT`, through `just test-fast`, in
`render-order-behind-red.0QXIZU`. Preflight refused before capture: CPU
31.1% (limit 10%), load 21.52 (limit 2). GPU passed at 5%, P8 throughout,
power IQR 0.28 W. Separate process inspection showed active Python workers
from another project. No process was stopped or threshold changed.
`capture.json` SHA-256:
`c3aefcdad2be2425e14016228ad71f0da691635de8afac530e69086828f550e3`.

**These attempts supplied no measured old-shader additive failure.** The fixture
checks repeated captures, uses the same creation order and pinned aurora,
and retains the additive metric's exact exit status. A preflight refusal
does not produce that metric or count as RED evidence.

The updated shader assembly regression fails against the old registry as
intended: `just test` exits 101, with 338 passing niri tests and one failure,
`material_source_is_prelude_then_optics_in_order_then_main` reporting
`iridescence out of order`. The log is retained at
`$NIRI_MATERIAL_WORK_ROOT/render-order-assembly-red-test.log`. The focused
capture tooling suite passes all 10 tests through `just test-fast`, and
`bash -n` accepts the new smoke. The shader assembly failure is separate
from the measured additive failure subsequently obtained below.

`just check` exits 0: formatting, clippy, all 128 tooling tests, task
validation and generated-file checks pass. Existing clippy warnings and the
unrelated `material-07a816` missing-process warning remain. At that checkpoint,
the intentionally failing assembly expectation was left uncommitted for the
implementation; production shaders were unchanged.

## Retry with one remaining window

Run `render-order-behind-red.aloE6t` used the unchanged baseline binaries
and thresholds. It exited 1 at preflight: CPU 2.0% and load 0.99 pass;
GPU utilization 35.5% and P5/P8 fail the 5% and P8-only requirements.
Power IQR 0.562 W passes, and there are no compute clients.
The `capture.json` SHA-256 is
`fc0bd83ee3c18cdf7427a0b9bd5f0fbf11cbbfee9d25efa1c102d2f37f8b54ee`.

IPC confirms exactly one mapped window, the focused Kitty. A separate
five-sample `nvidia-smi pmon` diagnostic, retained as `gpu-processes.txt`,
observed niri at 5–24% SM and Noctalia at 8–21% in its populated samples;
Kitty registered 3% in one sample. These are short process observations,
not attribution of the full preflight median. Closing application windows
alone did not make this desktop quiet. No process was stopped, no capture
was taken, and no production shader was edited.

## Measured old-build pixel baseline, 2026-09-16

After disabling the live desktop's main glass layer and switching workspace,
the user authorized retrying and relaxing thresholds as needed until a
baseline was obtained. This overrides the original execution plan's ban on
relaxation for this pixel baseline; it does not qualify performance evidence.

The default run `render-order-behind-red.Un7xI9` passed preflight at GPU 2%,
CPU 1.5%, load 0.15, P8 throughout. Geometry and the first material capture
succeeded. The next sub-run refused at GPU 3% solely for P5/P8 transitions.
Run `render-order-behind-red.qucX7K` allowed those two states, then refused
after geometry because GPU utilization 7.5% exceeded its 3% baseline by more
than the default 3-percentage-point tolerance. Both records are retained.

**Completed run:** `$NIRI_MATERIAL_WORK_ROOT/render-order-behind-red.064YZ3`.
Baseline binary source remains `522a09fe`; the separate fixture source is
the dirty `material-5b3107` checkout recorded in `capture.json`. No shader
was edited between the refused attempts and this completed baseline.

The retained `capture-meta-pixels-gpu-waiver` wrapper permits any measured
GPU P-state, utilization up to 100%, utilization drift up to 100 percentage
points, and power IQR/drift up to 1000 W. These explicitly waive GPU
quietness for pixel correctness. CPU, load, available-memory, compute-client
and exclusive-lock gates remain unchanged. All observed values remain in
`capture.json`; defaults in `tools/capture-meta` are unchanged. Wrapper hash:
`b7d19d5ceb6241df278a508297f44b534dd50c9317d9cca979441d43a92aca93`.

Preflight observed GPU 2%, CPU 1.5%, load 0.62, P8; subsequent readiness
samples observed GPU 2–12% and P5/P8. No compute clients were present.
The run calibrated a 456×640 window at (40, 40) and compared a fully covered
200×400 face crop at (100, 160). Four configurations cross fine noise
0/0.5 with pinned aurora 0/0.5. Every configuration's repeated decoded
image is identical; logs show no material compile error, fallback or panic.

| Additive metric | Result |
| --- | --- |
| Valid channel comparisons | 239,988 |
| Informative channel comparisons | 234,141 |
| Excluded clipped channels | 12 |
| Quantization-interval failures | 153,383 |
| Maximum absolute linear residual | 0.14503774127786598 |
| Metric / smoke exit status | **1 — expected measured old-shader failure** |

This is the required RED evidence, not a readiness refusal. The run supports
pixel comparisons only; no timing or power conclusion is claimed.
Artifacts include all configurations and images, metric JSON and exact exit
status, logs, source diff, fixture/metric/library/tool snapshots, the GPU
waiver wrapper and qualification, baseline source revision, and hashes.

- `capture.json`: `65aa74d557ab59733f907bded68d86f18d43aebbbe2893b86864ac6457e6c59a`
- `SHA256SUMS`: `c3c0c6f4ed39cb8eeba2acbe83fb47ee162e1d95be9fcdc821c2dd88b2d25775`

## Offline preparation

Added `glass-render-order-metrics.py` beside the existing smoke scripts:

- `grain`: signed sRGB luma residual standard deviation.
- `additive`: four-image linear-light comparison with per-code quantization
  intervals, clipping counts and minimum informative-sample requirements.
- Exit 0 means a reported grain measure or passing additive gate; exit 1
  means an informative additive comparison failed; exit 2 means invalid or
  uninformative input. These statuses are separate from capture preflight.

The tests use signed positive/negative residuals, unchanged and deliberately
changed additive light, and real ImageMagick decoding of synthetic PPM
images. They distinguish passing comparisons, measured failure, clipping,
invalid crops and corrupt image files. ImageMagick-dependent checks skip
on machines without that capture dependency; they ran on this host.

Validation command:

```bash
just --set fast_cmd 'python3 -m unittest discover -s tools -p test_glass_render_order_metrics.py' test-fast
```

Both tests passed. The earlier test run failed because the metric module
did not exist yet; it is only tooling-development history, not evidence
about the shader. No grain-preservation, additive-invariance, opaque-pixel,
reach, motion or cost acceptance result is claimed here.

## Behind implementation and requested capture pause, 2026-09-17

Implemented saturation/noise at `behind`, once after averaged taps and
before attenuation. Both hooks return linear light and preserve their
neutral early returns. White/fine retain signed grain; lightness retains
its existing gamut clamp. The transfer helpers clamp only power bases,
preserving signed linear branches. Both registries and parameter aggregation
now order saturation, noise, iridescence, aurora. Ring, aurora, sweeps,
opaque bypass, KDL and the independent postprocess shader are unchanged.

The source-assembly regression now also places the behind calls after the
tap average. Pipeline/optic/config docs and superseded-spec notices are
updated, and the generated parameter table matches registry order. Older
grain and saturation fixtures now use white attenuation, IOR 1, and no
response lights so their formula checks remain meaningful.

Offline verification:

- `MATERIAL_DOCS_UPDATE=1 just test`, followed by final `just test`: exit 0;
  339 niri, 89 niri-config, 1 wiki parsing integration, 4 niri-ipc and 1 doctest
  passed. The prior failing shader-order regression is now green.
- Final `just check`: exit 0; formatting, clippy, all 128 tooling tests,
  task validation and staged generated-file checks pass. Existing warnings
  remain, including the unrelated missing process on `material-07a816`.
- `bash -n` on the three changed smoke entry scripts and `git diff --check`:
  exit 0.

Logs: `$NIRI_MATERIAL_WORK_ROOT/render-order-behind-offline-test.log` and
`render-order-behind-offline-check.log`. The user asked to pause before the
next period requiring exclusive CPU/GPU use. No candidate GLES capture,
noise acceptance matrix, signal smoke or frame-cost measurement was run.
The working changes remain uncommitted; Task 1 is not complete.

First resume command (default readiness checks; allow roughly 10–15 minutes
for builds and the first additive capture pass):

```bash
export BASE_NIRI="$NIRI_MATERIAL_WORK_ROOT/render-order-baseline-522a09fe/niri"
export BASE_NIRI_TRACY="$NIRI_MATERIAL_WORK_ROOT/render-order-baseline-522a09fe/niri-tracy"
PHASE=behind MODE=verify CAPTURE_TASK=material-f8b6e9 \
OUT=$(mktemp -d "$NIRI_MATERIAL_WORK_ROOT/render-order-behind-candidate.XXXXXX") \
just --set fast_cmd 'bash docs/materials/scripts/glass-render-order-smoke.sh' test-fast
```

At that checkpoint the smoke covered the initial pinned-aurora additive gate
only. The prior baseline-only waiver is not silently applied to performance
runs. The later offline preparation below extends the candidate entrypoint.

## Readiness refusal and offline acceptance preparation, 2026-09-17

The first resumed action was one fresh 20-second headless preflight with the
default thresholds and no wrapper:

```bash
python3 tools/capture-meta preflight "$run_dir" --lane headless \
    --task material-f8b6e9 --fixture glass-render-order-smoke.sh \
    --owner-pid $$ --tool weston --tool kitty --tool tracy=0.13.1
```

It exited **1, preflight refused**. Retained artifact:
`$NIRI_MATERIAL_WORK_ROOT/render-order-readiness.yREof0/capture.json`.
SHA-256: `7b4b602ece3bf072e3c9945e95ebda47493a172950e04149e499d880f8b44353`.

| Measure | Observed | Required |
| --- | --- | --- |
| CPU busy | 10.5% | <=10% |
| Load average | 7.3 | <=2 |
| GPU utilization | 20% | <=5% |
| GPU performance states | P3, P5, P8 | P8 throughout |
| GPU power interquartile range | 1.937 W | <=1 W |
| Compute clients | BitwigStudio | None |

Available memory was 69.1%, above its 20% minimum. The run began at
`2026-09-17T20:05:04-04:00` on `titan`. No second preflight, threshold waiver,
capture, build, desktop change or process termination followed.

Offline work extended `glass-render-order-smoke.sh` with focused candidate
checks for neutral baseline identity, baseline/candidate signed grain at
`0.02`, aurora/ring/iridescence additive separation, dense face/bevel grain,
black and near-black signed-transfer probes requiring above-one-code grain and
unclipped pixels, analytic saturation-3 output within one code, opaque-client
and glyph identity at window-rule opacity 1 and 0.7, and rotated three-round
baseline/neutral/active Tracy cost. Each trace checks the shader log before
decoding its GPU median. Ring and iridescence additive metrics use
the straight-chamfer crop; aurora uses the fully covered face crop. Every
capture checks its repeat and scans the accumulated log for compile errors,
fallbacks and panics before a metric can pass.

The existing noise-type and noise/saturation smokes remain separate required
runs; nesting them would omit the shared per-launch settling and identity
contract. The existing signal smoke's `visual` and `cases` modes also remain
separate required runs for error/done sweeps. No result from any prepared
case is claimed until those hardware commands run on a host that passes the
unchanged readiness gate.

Initial focused offline verification, through `just test-fast`, passed 14 tests
across `tools.test_glass_optic_smoke` and
`tools.test_glass_render_order_metrics`. After two offline review rounds added
the signed-transfer invariants, corrected their normalized one-code threshold,
and ordered the per-trace log check, the same focused suite passed 16 tests.
`bash -n docs/materials/scripts/glass-render-order-smoke.sh` and
`git diff --check` also exited 0. No full Rust build or broad test suite ran on
the busy host.

## Fresh default readiness refusal, 2026-09-17 20:46

After the user reported the host mostly inactive, one fresh unchanged
20-second preflight retained
`$NIRI_MATERIAL_WORK_ROOT/render-order-readiness.ipLyDM/capture.json`
(SHA-256
`959f65f982e71bd377777744367ba665a63863c4df0f2b2fac7d28ae2fcfd307`).
It exited **1, preflight refused** only on GPU utilization and state:

| Measure | Observed | Required |
| --- | --- | --- |
| CPU busy | 2.1% | <=10% |
| Load average | 1.07 | <=2 |
| Available memory | 85.0% | >=20% |
| GPU utilization | 29% | <=5% |
| GPU performance states | P5, P8 | P8 throughout |
| GPU power interquartile range | 0.255 W | <=1 W |
| Compute clients | None | None |

One read-only five-sample `nvidia-smi pmon -s u` diagnostic observed niri at
24–47% SM in each populated sample. A separate controller sample observed niri
at 16–49% and the visible Kitty at 7–12% in two samples; other listed graphics
clients had no reported utilization. IPC showed the focused Kitty on workspace
index 1, two other windows on index 2, and empty workspace index 3 available.
These short samples are supporting context, not attribution of the whole
preflight median. No immediate readiness retry, threshold waiver, process
termination, build or capture followed.

After the user moved to empty workspace index 3 and reported ready, a fresh
unchanged readiness run passed at `2026-09-17T20:47:44-04:00`: CPU 1.7%, load
0.62, memory 85.0%, GPU 3%, P8 throughout, power IQR 0.268 W and no compute
clients. Retained artifact:
`$NIRI_MATERIAL_WORK_ROOT/render-order-readiness.iCx8Zn/capture.json`;
SHA-256:
`2acf303432bc83ad604b88df6ff099335ed7df905e8551203661577ea35fda3e`.

The candidate command then started with a fresh output directory, as required:

```bash
PHASE=behind MODE=verify CAPTURE_TASK=material-f8b6e9 \
OUT="$NIRI_MATERIAL_WORK_ROOT/render-order-behind-candidate.oFYXCN" \
just --set fast_cmd 'bash docs/materials/scripts/glass-render-order-smoke.sh' test-fast
```

Its own fresh preflight began at `20:48:26-04:00` and refused before
`build_binaries`: GPU utilization was 8%, above 5%. CPU 1.8%, load 0.48,
memory 84.9%, P8 throughout, power IQR 0.183 W and no compute clients all
passed. `capture.json` SHA-256:
`9ac5ad2dbce00e86ee0eaeb6b209f162b29ff6501f7d015e9cc07f68a9d6820e`.
The output contains only `capture.json` and the generated warm wallpaper; no
candidate binary or screenshot exists. The lock record reclaimed the exited
standalone readiness process, not a live competitor. No retry or waiver
followed.

Because the preceding standalone run passed at 3% and the candidate refusal
was otherwise quiet and P8-only, one bounded normal retry was authorized after
a 30-second settle, with a fresh output and unchanged thresholds. Run
`$NIRI_MATERIAL_WORK_ROOT/render-order-behind-candidate.1mK7AE` passed its
preflight at CPU 1.7%, load 0.33, memory 84.9%, GPU 2%, P8 throughout, power
IQR 0.297 W and no compute clients. It built and snapshotted both candidate
binaries:

| Binary | SHA-256 |
| --- | --- |
| `niri` | `72ae9ca8278c57d308d35b8852b7806597a62f28a0851a712983576cc4db9f88` |
| `niri-tracy` | `d7485d0de24f703383672b89c36170c177e6481a69ea93e459fb91e64b374210` |

Geometry settled and calibrated successfully. The settle before the first
acceptance case, `neutral-baseline`, then refused: GPU power was 15.675 W,
2.49 W above the run's 13.185 W baseline and over the 1.5 W drift allowance;
the GPU also moved through P5/P8 rather than remaining P8. GPU utilization was
4%, CPU 1.9%, memory 84.8%, and no compute client was present.

The run's `capture.json` SHA-256 is
`33f4fe5e4bc81209dd05440a45bc1af5af088827c7494c46c31436c30c2466a9`.
It retains the two binaries, source/input identities, geometry configuration
and image, calibrated rectangle, first-case configuration and logs. It contains
no neutral-case image, acceptance metric or Tracy cost. No further retry or
threshold waiver followed.

## Pixel matrix, regression smokes and strict-cost refusal, 2026-09-17

The approved pixel-only GPU waiver was isolated behind `SCOPE=pixels`.
`SCOPE=cost` and `SCOPE=all` reject a capture-tool override before preflight,
and explicit retained candidate inputs avoid rebuilding while recording the
originating build record in provenance. Focused shell/tooling verification
passed 18 tests through `just test-fast`.

The complete pixel matrix passed at
`$NIRI_MATERIAL_WORK_ROOT/render-order-behind-pixels.tyHcpH` (`capture.json`
SHA-256 `9e100988ebf411e1f9611d5d4c947cbe9c94ce8f641bedfc5c5d3c514c8500c3`).
Its identity records the release and Tracy candidate hashes, the originating
run record hash, and wrapper hash
`b7d19d5ceb6241df278a508297f44b534dd50c9317d9cca979441d43a92aca93`.
There are no Tracy traces in this scope. Neutral identity was `AE=0`; additive
aurora/ring/iridescence had zero failures; dark signed-grain SD was
`0.119595`–`0.166443` with non-clipped shares `0.442212`–`0.531775`;
saturation-3 error was `0.488` code values; and opaque identity was `AE=0` at
opacity `1.0` and `0.7`.

The existing noise-type and noise/saturation smokes passed separately at
`$NIRI_MATERIAL_WORK_ROOT/render-order-noise-type.rQuR6J` and
`$NIRI_MATERIAL_WORK_ROOT/render-order-noise-saturation.SnjVx7`. Signal
`visual` passed at
`$NIRI_MATERIAL_WORK_ROOT/material-signals-cc8134e2/signals-2466883-1789695970`.
Signal `cases` passed its first 21 behavioral redraw/cadence cases, then exited
1 at `impulse-none`: 10 redraws versus maximum 6. Six redraws align with the
three impulses; four form a separate late cluster. The retained trace does not
establish the cluster's cause, so the gate remains failed rather than waived.
Artifact:
`$NIRI_MATERIAL_WORK_ROOT/material-signals-cc8134e2/signals-2467759-1789695992`.
These are behavioral traces, not frame-cost measurements.

The separate strict cost run passed its initial preflight after a 30-second
cooldown, then refused before `gpu-baseline-1` because its settle sampled P5
and P8 rather than P8 throughout. No timing trace was accepted and no
threshold changed. Artifact:
`$NIRI_MATERIAL_WORK_ROOT/render-order-behind-cost.bJ3Ghy/capture.json`;
SHA-256 `52f3660558022473f4161b36496d83b077b647b855dbcb3b6827684f786e5359`.

Task 1 remains incomplete and uncommitted pending a strict cost matrix and a
passing, explained signal `impulse-none` result. Task 2 remains unimplemented.

Final repository verification exited 0 for `just test`,
`MATERIAL_DOCS_UPDATE=1 just test`, and `just check`. The test runs passed 339
niri, 89 niri-config, one wiki parsing integration, four niri-ipc and one
doctest. The check passed 138 tooling tests plus formatting, clippy, task,
upstream-report and package-pin checks; only the pre-existing unrelated
`material-07a816` task warning and existing compiler warnings remain.

## Signal acceptance and partial strict timing, 2026-09-18

Resumed using the retained candidate binaries from `1mK7AE`; the Tracy
binary still hashes to `d7485d0de24f703383672b89c36170c177e6481a69ea93e459fb91e64b374210`.
No production source, fixture assertion or readiness threshold changed.

Strict cost run `render-order-behind-cost.NiPgCC` passed preflight at GPU
2.5%, CPU 1.7%, load 0.30, P8 throughout. It completed baseline/neutral/active
round 1 and neutral round 2, then refused before active round 2 because the
settle sampled P5/P8. Per-trace medians were 22,528 ns for baseline and
21,504 ns for each of the three candidate traces. These are **partial data**;
the required three rotated rounds are incomplete and no aggregate cost
comparison is accepted. Retained `capture.json` hash:
`6f8bcbc069961f2311222db45a1dba5999fe3496a4077addf45d2b14c2f6a095`.

The failed signal trace was examined before rerunning. Its expected six
redraws occur at 15.343–17.844 s alongside three IPC requests. Its extra
four occur at 31.875–32.009 s alongside 90 `CompositorHandler::commit` zones,
with no corresponding IPC request. This establishes client surface activity
in that separate late cluster; it does not identify which client setting or
external action caused the commits.

A retained resume driver copies the original signal fixture, substitutes
the already verified Tracy binary for the build, and runs the failed case
twice plus its remaining successors. It preserves the original configurations,
setup, timing, counting and assertions. It recalibrates the demand-pulse
control before checking slowdown. No Kitty configuration or production code
was changed to obtain a pass.

Artifact: `material-signals-cc8134e2/signals-2609088-1789723230` under
`$NIRI_MATERIAL_WORK_ROOT`; driver source and the original fixture are retained
there as `resume.sh` and `original.sh`. Through `just test-fast`, exit **0**:

| Case | Observation |
| --- | --- |
| `impulse-none` | 6 redraws, 0 afterward |
| `impulse-none-repeat` | 6 redraws, 0 afterward |
| demand-pulse control | 534 redraws in 20 seconds |
| `done-pulse` | 93 redraws in the burst, 0 afterward |
| slowdown | 533 redraws, within the existing 10% control tolerance |

Both none-impulse traces have zero client commits in their final 14 seconds.
Together with the prior 21 passing cases, this completes the behavior checks;
the earlier failure remains retained as transient client-damage evidence,
without assigning an unproven cause. Signal manifest SHA-256:
`452a89c566bce719508a4ea144fcb25cf8b78ac4c8bfdbacc585b6755b1be90c`.

One fresh cost retry, `render-order-behind-cost-cool.4gBlTh`, used a retained
copy of the cost fixture adding a logged 30-second cooldown before each case;
it also waited 30 seconds before preflight. Thresholds were unchanged.
It refused before any trace: median GPU utilization was **0%**, CPU 1.9%,
load 0.92 and power IQR 0.435 W, but sampled states were P5/P8. The P8-only
gate was the sole refusal. `capture.json` hash:
`273feb87a479209d03ba1c920141ef90f2a0225ad6d2df8dcf9643b433c267c4`.

All runs released their resources. No timing waiver or additional retry was
used. Task 1 remains open and uncommitted pending the strict cost matrix;
pixel and signal acceptance no longer block it. A more isolated desktop/GPU
session is the next timing attempt to prepare; the cooldown alone did not
resolve the state transitions.

## TTY strict-cost observation and bounded cooldown fixture, 2026-09-18

`render-order-behind-cost-tty.kGH75c` retained one baseline Tracy trace after
the initial P8 preflight. Weston exited at `07:44:31-04:00`; the following
neutral settle ended at `07:44:44-04:00` with 0% GPU utilization but sampled
both P5 and P8, so it refused before a neutral trace. This supports, but does
not prove, a post-trace recovery hypothesis; it is not a timing acceptance.

The fixture now writes timestamped `cooldown start (30s)` and `cooldown
complete` records to `cost-cooldown.log`, waits 30 seconds immediately before
every `trace_run`, then leaves the existing strict settle gate and failure
propagation unchanged. No retry or threshold waiver is added. Hardware
validation is recorded by the subsequent strict TTY matrix below.

## Strict TTY cost matrix accepted, 2026-09-18

Retained artifact: `$NIRI_MATERIAL_WORK_ROOT/render-order-behind-cost-tty.3PgVZO`.
Its `SHA256SUMS` validates all 45 retained files. The run used the pinned
`522a09fe` baseline binaries (`539969…6bac9`, `0afb49…02bda`) and retained
candidate binaries (`72ae9c…b9f88`, `d7485d…74210`); the logged launches match
baseline `522a09fe` and candidate `cc8134e2-modified`. The three fixture input
hashes match the current worktree files.

The retained hashes bind the binaries and fixture inputs, but not a clean
snapshot of the dirty candidate build source. Shader continuity is established
by the session record, not an independently recoverable source snapshot.

The unchanged strict preflight was quiet (0% GPU, P8, no graphics or compute
clients), and all nine rotated baseline/neutral/active sub-runs settled for 10
seconds on P8 only. Every retained capture log is clean. Each case has three
raw samples: baseline `20480, 20480, 20480` ns; neutral `21504, 20480, 20480`
ns; active `20480, 20480, 20480` ns. Their recorded medians are all 0.020 ms;
neutral and active are both `+0.0%` versus baseline. This records the measured
cost without inventing a pass threshold. No waiver, retry, or further capture
is implied.

## Within and older-optic acceptance, 2026-09-18

Task 2 uses the retained V12 candidate bundle at
`$NIRI_MATERIAL_WORK_ROOT/render-order-within-pixels.V12xVh`: its release is
`29e7df7477e88a3eb5630e53cfd925fd53279e77f1b517e584f621e05653a912` and
Tracy binary
`c867e490f008143d498c4b78e38218f8c71ebf40542fec5d100529b6869346c9`.
Its originating build record is
`c28210be107eac9edaf4e6b3bd8c26a0c456737335be983e42c4805d887a8bac`; the
original V12 source patch is
`71ea7cdfce90f5755e1a44a3647d7a3ba6e1926cb2b5a01ed13aea781a1dd1c1`,
against `e33aa968`. This is dirty-build provenance, not a clean-build claim.

| Evidence | Result | Record SHA-256 |
| --- | --- | --- |
| `$NIRI_MATERIAL_WORK_ROOT/render-order-within-pixels.eS4KzB/run` | Neutral identity; ring bounds 37/37/61/61/56/59; rough FWHM 4→6 and peak 104→61; face and static aurora pixel gates passed before the known motion-KDL stop. | `e4a9c71d6b57149e1959b79b4255f0d2ef48b65619fc82bd91dc72d59064294b` |
| `$NIRI_MATERIAL_WORK_ROOT/render-order-within-resume.C2DA1f/run` | Reviewed motion repair; attenuation, opaque AE-zero, and three additive checks passed. Its complete `SHA256SUMS` was verified. | `78b8c73103da74fd36defb2adb0e03071c314fbd1c07ae752d912d54448ed9ec` |
| `$NIRI_MATERIAL_WORK_ROOT/render-order-within-cost.A43SyN/run` | Strict within cost: 45 manifest entries, nine settled P8-only sub-runs; all-median 20,480 ns and +0.0%. | `71e53209ea056e89db0bf9e0e7b08ce7d32781334b0d19f9a36d642a8f4fa05e` |
| `$NIRI_MATERIAL_WORK_ROOT/render-order-old-ring.FH88OR/evidence` | Older-ring pixel evidence passed; 218 manifest files. Outer record only: no per-nested-config capture-meta settle and no strict-cost claim. | capture `27c8e12587fbc639898233b96ebf6ff2c4bafe40ba9df77fab7543518db5ed5d`; manifest `99e8ab805c701281ed2c20dcda8d2b527342b412a3950a8b77de87ce75ece3c2` |
| `$NIRI_MATERIAL_WORK_ROOT/render-order-aurora-strict.xGIUBC/run` | Visual assertions and five cadence traces passed; the first three cost traces completed, then strict settle before `gpu-zero-2` refused P5/P8 only. | `e6244f3e075d22682691fafeff0bb41a82603a8319945c68235dad649c26a042` |
| `$NIRI_MATERIAL_WORK_ROOT/render-order-aurora-cost-resume.BYb5Fq/run` | Strict cost-only continuation: 49 manifest entries; quiet preflight plus ten P8-only settles (geometry and nine rotations); all raw medians 20,480 ns and all case differences +0.0%. | capture `77617c9541fca3ccac665834c446ef1bd6840e099e0d3df3866ec7c998cb0d42`; manifest `c8db6c3364168d76c97755378e948e662f3520f44d76a976ccf8b8253d86ded7` |

The static aurora light-IOR fixture originally had a one-code chamfer delta
at thickness 20. Its thickness-80 diagnostic at
`$NIRI_MATERIAL_WORK_ROOT/render-order-aurora-depth.WzKHWH/run` retained the
same failing left-chamfer threshold (artifact manifest
`5e0c8d759491583e6b92c8ec9dbb791cc0be370fe55681a34649e91d604aefe6`),
while thickness 200 at
`$NIRI_MATERIAL_WORK_ROOT/render-order-aurora-depth200.MZzHGt/run` passed the
unchanged threshold with 341 changed pixels, flat-face identity, and stable
repeats (record
`a62720ae5a91e5f261bdec8f51f261924b440189aad482dfdb5371c94ffe7ccc`; manifest
`34c9caea0a5c9dad75dbd3f38bfc0d227b5749ca076ca0b526c9c845716e470e`).
**Ruling:** retain thickness 200 for all three static aurora configs—light-IOR
1, light-IOR 6, and distortion—so distortion compares against its matched
IOR-6 reference. Motion remains thickness 20.

The first full within run rejected `jelly-flex 1`, outside the parser's
0–0.02 range. **Ruling:** the motion fixture uses the established valid,
nonzero default `0.004`; it introduces no oscillator or cadence change.

Older-ring rest, selector and crossfade gates passed with a 46 px bound and
no pixel outside its slab bound. Pinned and drifting move/resize frames retain
screenshot-request timestamps, adjacent IPC target layouts, and whole-frame
paired-host deltas. Rendered animated slab geometry and interior reach are
unavailable and are not inferred or gated; the tiny zero-chamfer fixture
remains `NOT VERIFIED BY RENDER`. Those frames are handed to
`material-0e130e`, which owns motion judgment rather than this task.

The stock-inset capture is retained at
`$NIRI_MATERIAL_WORK_ROOT/render-order-within-pixels.eS4KzB/run/within-pinned-on.png`;
the face-placement comparison is
`within-face-on.png` in the same directory. It is evidence for a user-facing
tuning decision, not an automatic default change. Defaults remain unchanged.
Follow-up `material-9306b5` records the observed inset-5 luminous rim/inward
halo and the deliberately extreme inset-20 face-separated control; it depends
on this task and requires user visual judgment before any default change.

The older aurora run's saved cadence counts are plain 0, pinned 0, 4 Hz 80,
reduced 40 and off 0 redraws in the final 20 seconds. **Ruling:** the reviewed
fixed 30-second logged cooldown before every cost trace reuses the accepted
cost pacing; thresholds, cadence, rotations and retry policy are unchanged.
The original xGIUBC refusal remains evidence. The strict, no-`CAPTURE_META`
cost-only continuation
`$NIRI_MATERIAL_WORK_ROOT/render-order-aurora-cost-resume.BYb5Fq/run`
completed successfully: its quiet preflight and all ten settles (geometry
plus nine rotations) were P8-only, and all nine raw medians were 20,480 ns.
Plain, zero, and on all have a 20,480 ns median, so zero and on are each
+0.0% versus plain. Controller independently recomputed the nine medians
from the first 14 `MaterialRenderElement::draw` samples in each 20–28 s
window using `csv.DictReader` and `statistics.median`; those calculations
match the retained median files exactly.
