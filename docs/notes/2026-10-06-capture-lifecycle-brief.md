# Capture lanes and lifecycle: scoping brief

Scope pass: 2026-10-06. Handoff for `material-2834d7`, not an approved design.

## Problem

Run each evidence check on the least host it needs and make retained capture
records describe what actually completed. This pass covers GPU preflight
requirements (`material-6bd4a3`), software rendering (`material-925518`),
external DRM sub-run completion (`material-a9a455`) and fixture outcome
recording (`material-e1ef98`). Three remain ideas while their protocol or
consumer questions are resolved; `material-925518` is shelved (see below).

## Current behaviour and evidence

- `material-6bd4a3` records a desktop-up preflight pass at 3% GPU/P8 on an
  empty workspace (`f4a4302a`). Meetability is established for that sample;
  reliability of later settles is separate. The
  [idle-budget evidence](../materials/2026-09-11-idle-budget-evidence.md#findings-along-the-way)
  records three desktop attempts losing settle checks to different transients.
- `tools/capture-meta::sample_stream`, `judge_quiet` and `judge_settled`
  require NVIDIA telemetry for both current lanes. `GpuReader` refuses without
  `nvidia-smi`. No software lane exists. The only named llvmpipe consumer,
  `glass-view-tilt-smoke.sh` (`material-cd0e1d`), exists only on the dropped
  `material-77db8a` branch, and its smoke passed under the existing GPU gates
  on 2026-09-24 (`renderer_verified_launches=15`). No current fixture needs
  software-rendered timing.
- The [workstreams brief](2026-10-02-workstreams-brief.md) already routes
  visual identity toward frozen-clock fixtures such as `src/tests/ring_pair.rs`.
  A software-rendered nested fixture can still involve GPU-rendered Weston
  or clients; a Mesa environment variable alone does not establish isolation.
- `10e8e5e5` added run/sub-run timing; `2fcf022a` added recovery finish times.
  The local smoke lib's `stop_nested` and `optic-settling-smoke.sh::stop_drm`
  call `finish_sub_run`. The external idle-budget consumer's current code
  is unavailable within this fixed checkout. Its
  [implemented design](../specs/2026-09-27-idle-budget-fail-fast-design.md)
  explicitly anticipates reruns after dynamics changes.
- `glass-optic-smoke-lib.sh::summarize` releases before PASS; cleanup releases
  again and may fail while writing checksums. `finish_run` changes the record
  only on its first finish; lifecycle tests require later release immutability.
  Release success therefore cannot stand in for final fixture success.

## Constraints

Preserve the [capture protocol](../specs/2026-09-11-material-capture-protocol-design.md),
lock ownership, disturber restoration, manifest integrity and truthful recovery
timestamps. Keep hardware thresholds unchanged pending a reviewed change.
Existing visibility attribution belongs to `material-7afc31`; input prechecks
belong to `material-5dbf17`. Reuse those tasks rather than duplicate them.
These follow-ups need no host capture; later measured execution declares
`quiet` and runs a pilot before a matrix. Live desktop use needs separate authorization.

## Alternatives

1. **Current lean: separate evidence classes and lifecycle boundaries.** Use
   frozen pixels for visual identity, retain strict GPU measurement lanes,
   and design software gating only for a demonstrated remaining consumer.
   Record a numeric outcome at an explicit finalization boundary.
2. Add a software flag and `release --exit-status` immediately. This leaves
   participating renderers, early release and subsequent cleanup unresolved.
3. Relax GPU thresholds globally or add a broad outcome taxonomy. Neither
   follows from the evidence; preserve measurements and keep the outcome proposal small.

## Unanswered questions

- Which remaining consumer needs software-rendered timing? Answered in review:
  none. The view-tilt smoke was the only one named, and it left with
  `material-77db8a`. The hardware routes' host conditions are settled in the
  [per-route host conditions design](../specs/2026-10-08-capture-host-conditions-design.md);
  a software lane waits for a demonstrated consumer.
- Is the external idle-budget DRM consumer still supported, and where should
  normal completion call `finish_sub_run`? `material-282edc` checks the actual code.
- Which outcome can be recorded truthfully after hold restoration and checksum
  work, including early release and interrupted cleanup? `material-232940`
  designs the boundary; recovery without an observed status stays unknown.

## Proposed decomposition

### Renderer-aware capture

- `material-6bd4a3`: **briefed**, waiting on `material-3d48b0` (P1, small,
  mid complexity, direct, needs quiet): the desktop-idle pilot of nested
  measurements against the pinned TTY reference. Until it passes, nested
  measurements stay on a TTY.
- `material-18c2a1`: **implemented**. The
  [per-route host conditions design](../specs/2026-10-08-capture-host-conditions-design.md)
  adds the `pixels` lane, pre-hold lane checks, the observed `host_condition`,
  per-launch renderer verification, and build-before-preflight ordering. The
  pixels-lane pilot is `material-52dc6e`; the niri-experiments fixtures adopt
  it under `material-9cf378`.
- `material-925518`: **shelved** in review, with no consumer. Unshelve when a
  new fixture needs software-rendered timing.

### External idle-budget consumer

- `material-a9a455`: **briefed**, waiting on `material-282edc`
  (P2, small, mid complexity, direct): bounded external consumer inventory,
  with no production patch or capture.

### Fixture outcome

- `material-e1ef98`: **briefed**, waiting on `material-232940`
  (P2, small, high complexity, planned): reviewed fixture-outcome design and
  plan, after `material-282edc` establishes the external lifecycle.
- Reuse the evidence-instruments lane. Each follow-up updates this brief and
  adds finding notes to its named waiting ideas in its result commit.
