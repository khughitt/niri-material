# Workstreams: parallel lanes over the open material work

Status: tracker migration, 2026-10-03 (material-5595fe); refreshed against the
tracker on 2026-10-06. Not a design; it groups
existing goals and loose tasks into lanes that can run side by side. Lanes and
needs now live in the tracker. When one lane is waiting on a quiet host or a
review by the owner, work continues in the others.

## Problem

At the 2026-10-02 scoping pass there were 72 open tasks: 8 goals from the
2026-09-29 scope pass and about 25 loose ideas. They were grouped by subsystem,
which hides two things that decide what can
run at once:

1. **Host class.** A check needs one of three hosts. `none` is code, frozen-clock
   fixtures, docs and design. `headless` is a nested niri on the headless weston unit.
   `quiet` means an idle host, using a TTY with the desktop stopped when the run
   requires it, for power, timing and settle evidence.
   On top of any of these, the owner may have to judge sheets.
   Only the `quiet` class serialises.
2. **Shared mechanisms.** Two are established: frozen-time fixtures and the bevel
   height field. Each one, built once, unblocks several lanes. Two are hypotheses
   to test against a second concrete consumer before extracting them: an ordered
   layer stack and named time envelopes. The cost model is shared as measurements
   and conditions, not as summed per-effect prices.

## Current behaviour and evidence

- The settle-lifecycle captures `material-80caf4`, `material-3acc86` and
  `material-f7eb0b` are done; two-output removal (`material-1af3c6`) remains.
  View-tilt (`material-77db8a`) was rejected in owner review on 2026-10-06; the
  rigid tilt follows the [transient depth brief](2026-10-06-transient-depth-brief.md).
  The glass-edge contact sheet (`material-124f1f`) ran headless on 2026-10-04
  and passed; `material-be611b` is parked for the owner's review of it. The
  cost study `material-31074f` is the wake hub for
  `material-2ebf2c`, `material-a0cbb0`, `material-a91346`, `material-0c7eed` and
  `material-f6e284`, and it needs a quiet host.
- Deterministic in-process rendering already works. `src/tests/ring_pair.rs`
  (`material-0e80c1`) renders byte-identical pairs at a frozen instant on
  surfaceless GLES. `src/layout/tests/drag_dynamics.rs` (`material-b3ce14`) records
  motion on a pinned clock. Neither run needs a quiet host or a GPU preflight.
- Preflight: the GPU thresholds are meetable with the glass desktop up (an
  empty-workspace preflight passed at 3% GPU/P8 on 2026-09-16), but per-case
  settles still lose to desktop transients (`material-6bd4a3`). The only
  llvmpipe fixture left with the dropped view tilt, so a software lane
  (`material-925518`) is shelved. Host disturbers are now held for the length
  of a quiet capture (`material-188aaa`, done).
- Edge diagnosis (`material-be611b`, 2026-09-30): the chamfer was a planar ramp,
  so one normal covered its whole width, with global refraction thickness. The
  implementation is reviewed in the glass-edges worktree and its contact sheet
  has run; the owner's appearance review and the final review remain before merge.

## Constraints

- Pass order: `docs/materials/render-pipeline.md`.
- The v1 contract says opaque pixels bypass the material. `material-7f5751` and
  `material-987655` both break it for windows that opt in, so they need one shared
  opt-in contract, not two; `material-d257d9` designs it.
- Settle contract (`material-f86183`, `material-0db905`): every continuous dynamic
  in the motion and signals lanes has to declare how it takes part in settle mode
  and how it resumes.
- Quiet runs go one at a time, with the lane's pilot before any full run
  (see AGENTS.md).

## Alternatives

- **A. Lanes as parent goals, scheduled by step (taken).** `tasks tree` shows each
  lane, and existing goals and their briefs keep their scope. A lane mixes
  preparation, shared renderer changes and quiet verification, so a step is
  scheduled by its own constraints, not by its lane. The six goals carry `lane: true`;
  steps record known gates in `needs`: `quiet`, `nested` (a nested compositor)
  and `owner` (the owner supplies or judges an image). These replace the former
  `lane` and `needs-*` tags. They are not exhaustive: a task without a need is not
  automatically runnable anytime, and its checks name their host class. Where a task
  bundles preparation with a quiet run, split the two when it is taken up
  (`material-31074f`).
- B. Lane tags only. This leaves the tree flat, and the 9-29 briefs never get a
  common place to live.
- C. Re-scope everything into new goals. This throws away the 9-29 handoffs for no gain.

Use `tasks lanes` (or `tasks --pretty lanes`) for guidance, state and the next
step in each lane; `tasks prime` includes that view. `tasks next --under <lane>`
selects within a lane. On a busy desktop, `TASKS_WITHOUT=quiet tasks ready` or
`TASKS_WITHOUT=quiet tasks next` excludes quiet-bound steps. Setting that variable
for desktop sessions belongs to dotfiles and is outside this migration.

The vocabulary is in `tasks/.config.toml`. `quiet` is exclusive across sessions;
`nested` and `owner` are not. The current nested runners create unique Weston
units and sockets (`material-signals-smoke.sh::start_nested` and
`glass-optic-smoke-lib.sh`), so there is no single shared Weston unit to reserve.
Measured captures still need `quiet`: the capture protocol's own serialization
and preflight remain in force. An owner need describes the requirement; a review
park still records the actual handoff to the owner.

## Unanswered questions

- **Order of the optics lane.** Answered: bevel first. `material-4e3e9c` already
  scopes itself after `material-be611b`'s edge profile, because the band follows
  innerDist/chamfer. It now depends on `material-be611b`; its comparison scenes and
  acceptance criteria can be prepared meanwhile.
- **Layer-stack contract (hypothesis).** Noise modifies the transmitted backdrop
  before attenuation; interior light, embedded textures and profile composition
  have different semantics. An ordered list with weights is not yet a common
  rendering operation. Build bounded noise layers (`material-3fcba2`) first and
  extract a contract only when a second concrete consumer fits it.
- **Envelope abstraction (hypothesis).** Reuse native animations and named
  responses first, as the dynamics brief recommends. A focus transient
  (`material-d873bf`), a signal expiry and a weather cycle (`material-f3e4e4`) may
  share curve evaluation but need different clocks and settle behaviour. Revisit
  with `material-9be53d` only when an existing mechanism demonstrably cannot express
  a wanted behaviour.
- **Cost model.** Share measurements and their conditions. Preset estimates must
  account for shared blur/prefilter caches and interactions; summing per-effect
  prices will miss them.

## Lane decomposition

| Lane | Goal | Host | Established shared ground | First milestone |
|---|---|---|---|---|
| Evidence instruments | `material-2834d7` (P1) | none, then shrinks quiet | frozen-time fixtures (visual behaviour only) | `material-3a17b8` helpers; per-lane host conditions (`material-18c2a1`); render-anomaly attribution (`material-7ff4bc`) |
| Quiescence and cost | `material-5d6b2c` (existing) | **quiet** for verification; preparation none | measurements and their conditions; the settle contract | two-output settle evidence (`material-1af3c6`), then a bounded cost pilot |
| Glass optics | `material-6062fd` (P1) | headless, plus owner review | the bevel height field | `material-be611b` reviewed by the owner and merged |
| Material library and composition | `material-3aa1f2` | none or headless; quiet for measured cost (`material-bb3fe5`) | none yet; layer stack is a hypothesis | `material-3fcba2` with single-layer identity preserved; `material-56f91a` reading-texture proof |
| Motion and time | `material-53f873` (existing, retitled "Responsive glass that settles completely") | headless/owner; idle for clips | native animations and named responses | owner-judged stills of a rigid tilt (`material-89fb6b`) |
| Signals | `material-6f606b` | none; `material-07bac9` needs a nested compositor | source to store to response | `material-c1330b` replay finding (done) and `material-07bac9` transport/attribution finding |

Each lane's current scoping brief: instruments, the
[capture lifecycle](2026-10-06-capture-lifecycle-brief.md) and
[render anomalies](2026-10-06-render-anomalies-brief.md) briefs; optics, the
[glass optics brief](2026-10-06-glass-optics-brief.md); composition, the
[texture composition brief](2026-10-06-material-texture-composition-brief.md);
motion, the [transient depth brief](2026-10-06-transient-depth-brief.md).
They carry the detail; this brief stays the index.

**Next parallel push.** Instruments: `material-3a17b8`; `material-18c2a1` and
`material-282edc`; `material-86c841` and `material-aaa714`.
(`material-22d78f`, `material-bb8480` and `material-77be96` are done.) Signals:
`material-07bac9` is the remaining nested-host demonstration after
`material-c1330b`. Optics: the owner reviews the glass-edge contact sheet, then
`material-be611b` gets its final review and merges; `material-d257d9`'s design
can start meanwhile, and `material-f4143a` follows the merge. Composition:
`material-3fcba2`, and `material-56f91a` independently. Motion: frozen stills of
a rigid tilt (`material-89fb6b`) for the owner, before its design
(`material-abc08c`). Optics, motion and noise layers still compete for shader and
configuration code ownership.

The opaque-content opt-in contract stays visible across lanes: `material-d257d9`
designs one boundary for content depth (`material-7f5751`) and inactive
desaturation (`material-987655`).

Loose bug and hygiene work fills gaps when the larger lanes are blocked:
`material-e88df7`, and `material-fa4eec` (`needs: [owner]`: the owner supplies the
screenshot). `material-698875` is done; `material-c8732f` moved under
`material-7ff4bc` in the instruments lane.

How the lanes feed each other: instruments → the visual acceptance checks of every lane. Cost
measurements → budgets for composition, fireflies and lighting, and Fresnel's reflected sample.
Bevel → `material-7f5751`, `material-4e3e9c`, `material-933a8b`. Settle contract →
the motion and signals lanes.

**Regrouping (applied 2026-10-06).** `glass-edges` merged into materials-26.04
(`23a2e37b`); `material-be611b` is done and now sits under `material-6062fd`.
Its child `material-124f1f` (the contact sheet) was already done.
`material-77db8a` was dropped unmerged on 2026-10-06.
