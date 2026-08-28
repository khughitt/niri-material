# Native materials v1 physical DRM acceptance: design

**Status:** implemented and passed 2026-08-28. The corrected physical DRM run
passed all 19 machine gates and all nine physical observations at
`niri-experiments` result commit
`c0caa944db2edc5dc4844e6720951d7f32652b76`. Native materials v1 is
accepted. Corrective fixture commit `901b5a4` isolated every kitty client with
`--config NONE`; the candidate binary was unchanged.
**Parent design:** `docs/materials/2026-08-22-v1-design.md`

## Goal

Close the final native-materials v1 gate with a controlled smoke test on the
real DRM backend. The run must combine machine-checkable evidence with a human
observation of the physical display: compositor screenshots cannot establish
that scanout showed no blank frame, untreated flash, detached slab, or corrupt
retained pixels.

This is a one-time acceptance procedure for the frozen v1 implementation. It
is not a general DRM test framework, another reference-parity pass, or a new
production feature.

## Accepted starting state

| Input | Pin or requirement |
| --- | --- |
| Candidate repository | `niri-material` |
| Candidate source | `138697be4cbb779c80425fe2a366ceca3610f38e` |
| Final production implementation | `b8fe7b84cd74d306f3bf491b5313161d4eae80fc` |
| Evidence repository base | `niri-experiments` branch `results/slice3`, commit `c4b71a4ebfbe3c82c56f964bfc24d4f7de1bde4f` |
| Seat and GPU | `seat0`, NVIDIA GeForce RTX 3070 using the `nvidia` kernel driver |
| Output | `DP-1`, GIGABYTE G34WQC A, 3440×1440 at 60 Hz, scale 1, normal transform, VRR off |
| Test virtual terminal | active VT2 login session |
| Recovery session | the existing niri session on VT1 |
| Work root | required `NIRI_MATERIAL_WORK_ROOT=/mnt/ssd3/niri-material` |

There are no production-source changes between `b8fe7b84` and `138697be`;
the intervening commits update only documentation. The candidate release
binary is built from the latter pin so its version and SHA-256 identify the
exact accepted tree.

The config-surface review and two independent frozen-reference captures are
already complete. The final parity result at `c4b71a4` passed integrity, all
28 implementation rows, and all 14 combined parameters. This design does not
repeat those proofs.

## Repositories and files

The acceptance fixture and evidence belong in `niri-experiments` on a fresh
`results/v1-drm-acceptance` branch from the pinned evidence base:

- `fixtures/v1-drm-smoke.sh` — VT2 launcher, action sequence, capture, and
  machine gates;
- `fixtures/v1-drm-smoke.kdl` — isolated current-schema acceptance config;
- `docs/results/2026-08-27-v1-drm-acceptance.md` — pinned environment,
  observations, measurements, hashes, and verdict.

This repository owns this design, its implementation plan, and the v1 status
surfaces updated after the run. No PNG, raw log, runtime directory, or Cargo
output is committed to either repository.

## Execution architecture

### Candidate build

Build `138697be` with `cargo build --release` and an explicit dedicated target
directory:

```text
/mnt/ssd3/niri-material/targets/v1-drm-acceptance
```

Record the source commit, `niri --version`, binary SHA-256, Rust/Cargo
versions, and the clean source state before the run. The launcher accepts that
absolute binary path; it does not build or choose a fallback binary.

### VT2 ownership and recovery

The launcher runs from an interactive login on VT2 and starts plain `niri`
with the acceptance config. It must not use `niri --session`: the existing
user-systemd graphical session on VT1 remains intact and is the recovery
route.

Before starting niri, the launcher fails unless all of these are true:

- stdin is `/dev/tty2` and the caller's logind session is active on VT2;
- the session belongs to `seat0`;
- the expected NVIDIA DRM device and `DP-1` are present;
- no earlier acceptance process owns the run lock;
- the pinned candidate binary and committed fixture hashes match;
- `niri validate` accepts the runtime config;
- `NIRI_MATERIAL_WORK_ROOT` is set to the accepted external root and is
  writable.

There is no fallback to another VT, seat, GPU, connector, mode, work root, or
`/tmp`. After niri starts, the launcher confirms that `DP-1` is exactly
3440×1440 at 60 Hz, scale 1, normal transform, and VRR disabled before it maps
the test clients.

The config provides a dedicated exit binding. The launcher's
`EXIT`, `HUP`, `INT`, and `TERM` traps terminate only its owned niri,
wallpaper, and client processes and remove only its runtime state. If the test
display becomes unusable, switching back to VT1 is the recovery path.

### Isolation and artifact location

Each run uses fresh runtime, config, and cache directories plus a unique
artifact directory below:

```text
/mnt/ssd3/niri-material/v1-drm-acceptance.XXXXXX
```

The launcher copies the committed config into that directory before applying
controlled reload edits. It never reads or writes the personal niri config.
The existing VT1 Wayland socket, niri IPC socket, and graphical-session
targets are not modified.

## Scene and actions

The scene uses the committed 40×40
`niri-experiments/fixtures/diagnostic-grid.png` tile as its tiled wallpaper.
That PNG is reproducibly derived from the retained 200×200
`diagnostic-grid.svg`; the runtime copy is named `grid-tile.png`. One
translucent kitty probe is selected for the `frost` material, and one opaque
control has no material. The material uses the reviewed v1 names and a
representative visible combination of refraction, attenuation, distortion,
anisotropic blur, jelly, bevel, and offset. There is no frozen-reference
client in this run.

Both kitty clients use `--config NONE`, disable cursor blinking, hide the
cursor, and run a static sleep payload. They render no clock, status, changing
title, watcher, or other time-varying content. This isolation and suppression
is a prerequisite for the settled-frame identity gate, matching the earlier
slice-2 fixture.

The launcher waits for the compositor, output, wallpaper, and exact expected
window set before beginning. It records each IPC action and its completion
time, then performs this sequence at normal animation speed:

1. Capture two settled source frames with the probe and control at rest.
2. Change the same material name's thickness and attenuation color, reload,
   confirm the mapped probe window ID is unchanged, and capture the visible
   result.
3. Write an invalid offset wider than the bevel, request reload, prove that
   niri rejects it and preserves the last valid config and mapped window, then
   restore the committed config.
4. Move the probe's column right and left repeatedly, capturing a motion burst
   and two settled frames.
5. Resize the column wider and back, capturing the transition and settled
   state.
6. Close and remap a probe with the same app ID.
7. Enter and leave overview.
8. Switch one workspace down and back up, including the strip end.
9. Capture the final physical output through niri's screenshot action.
10. Leave the session running for the operator's final visual check, then exit
    through the dedicated binding.

Repeating the short motion operations makes physical artifacts observable
without changing the normal spring timing. An animation slowdown is not used:
acceptance covers the production timing shown by the real 60 Hz output.

## Machine gates

The launcher produces a machine summary only after the owned compositor has
exited and cleanup has completed. It passes only when:

- all preflight pins, hardware facts, output facts, and config validation
  checks pass;
- every IPC action succeeds and every expected capture is present and
  decodable at 3440×1440;
- the same-name valid reload and rejected invalid reload retain the original
  probe window ID;
- close removes the original probe ID, remap produces exactly one new probe ID
  distinct from the original, and its settled capture shows the material
  treatment again;
- the valid reload changes the probe region while the invalid reload retains
  the last valid appearance;
- motion and resize captures differ from their settled frames;
- paired settled frames are byte-identical;
- overview and workspace captures contain the probe and expected backdrop;
- the final output screenshot succeeds;
- the niri log contains no panic, material shader compilation failure,
  material fallback, DRM/page-flip failure, or renderer error;
- all recorded capture hashes verify after the run; and
- every owned child is gone and the run lock is released.

Expected harmless driver or client messages must be enumerated from the
actual run before they can be excluded. The launcher does not contain a broad
warning allowlist.

These gates establish that the controlled scene and capture path behaved
correctly. They do not substitute for observing the monitor.

The byte-identical settled-frame gate is deliberately a new test of the
physical DRM/NVIDIA path. Once the static-client prerequisite and evidence
integrity checks pass, a settled-frame mismatch is an acceptance failure, not
an integrity failure.

## Physical observation gate

The operator watches the whole sequence on `DP-1` and records an explicit
PASS or FAIL for each of these claims:

- no blank or black frame;
- no untreated window flash during map, reload, move, resize, or workspace
  and overview transitions;
- the slab remains attached to the window throughout every transform;
- retained window pixels remain spatially correct, without smear, jump,
  corruption, or stale content;
- refraction and attenuation visibly update after the valid reload;
- the invalid reload leaves the previous valid appearance intact;
- move and resize show jelly deformation and settle cleanly;
- overview and workspace boundaries show no void, detached band, or clipped
  strip-end overhang; and
- the compositor remains responsive through exit.

The result document records the operator, date, and each observation
separately. A blanket "looks good" is not acceptance evidence. A machine PASS
without this completed observation table remains incomplete.

## Evidence contract

The result document records:

- candidate source, version, binary hash, fixture hashes, and evidence-base
  commit;
- kernel, NVIDIA driver, DRM device, seat, VT, connector, monitor, mode,
  scale, transform, and VRR state;
- exact commands and ordered action trace;
- machine-gate table with the measured values and artifact hashes supporting
  each row;
- physical-observation table with an explicit verdict and concise note per
  row;
- full capture inventory hash and line count;
- log hash plus every warning or error classification;
- cleanup result; and
- final integrity and acceptance verdict.

Screenshots and logs remain outside Dropbox until the result has been
independently reviewed. Hashes are audit anchors, not a substitute for the
operator's observation.

## Verdicts and failure handling

The run has three possible outcomes:

1. **Integrity failure.** A prerequisite, capture, hash, action, identity,
   cleanup, or evidence-completeness check fails. Stop without interpreting
   the physical smoke and do not claim acceptance.
2. **Acceptance failure.** Evidence is valid, but a machine gate or physical
   observation fails. Record the complete failure, keep v1 blocked, and
   diagnose or fix it in a separate branch. The acceptance branch does not
   acquire an opportunistic production-code fix.
3. **PASS.** Every machine gate passes and every physical observation is an
   explicit PASS. Mark native materials v1 accepted.

On PASS, update the parent v1 design, parity design, static-preflight design,
their executed plans, and `docs/materials/README.md` in the same material-doc
change. Grep all current user-facing material docs for propagated claims that
physical DRM is pending or v1 remains unaccepted. Historical instructions may
remain in future-tense only when their status header makes their execution
state unambiguous.

## Artifact lifecycle

Before VT2, verify the fixture's argument handling, refusal outside active
VT2, config validation, hash generation, and owned-child cleanup without
acquiring DRM. The actual run retains its raw artifact directory and dedicated
Cargo target through result review.

After the accepted result and status changes are reviewed, move only the
exact run directory, launcher handoff files, and dedicated target to trash,
then verify that no acceptance process, unit, socket, or lock remains. Failed
evidence stays until its diagnosis and review no longer need it, then follows
the same cleanup rule.

## Alternatives rejected

### Fully automated DRM replay

Screenshots cannot prove what the physical scanout showed between captures.
Automation supplies the action trace and machine gates, but a human must watch
the monitor for the load-bearing physical observations.

### Personal-session config

It is realistic but not controlled: unrelated rules, services, clients, and
reloads would enter the evidence. The isolated fixture exercises the reviewed
surface without modifying personal state.

### Reuse the headless or nested parity host

Those paths already passed and cannot close the explicit real-hardware gate.
The physical run tests the TTY/DRM renderer and scanout behavior that nested
winit and Weston do not represent.

### General DRM harness

V1 needs one pinned acceptance run on one known machine. Multi-GPU, multi-seat,
multi-output, VRR, HDR, CI, and reusable scenario APIs add no evidence for
that decision.

## Explicitly deferred

This run does not add mipmapped backdrop storage or `roughness`, measure GPU
performance, test direct scanout, repeat semantic parity, cover VRR or HDR,
or broaden the supported material surface. Damage-aware mip/prefiltered
backdrop storage remains the first post-v1 follow-up after acceptance.
