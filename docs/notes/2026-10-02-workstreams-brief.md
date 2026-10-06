# Workstreams: parallel lanes over the open material work

Status: tracker migration, 2026-10-03 (material-5595fe). Not a design; it groups
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
  rigid tilt follows the [transient depth brief](2026-10-06-transient-depth-brief.md). The quiet queue now holds the glass-edge contact
  sheet (`material-124f1f`). The cost study `material-31074f` is the wake hub for
  `material-2ebf2c`, `material-a0cbb0`, `material-a91346`, `material-0c7eed` and
  `material-f6e284`, and it needs a quiet host.
- Deterministic in-process rendering already works. `src/tests/ring_pair.rs`
  (`material-0e80c1`) renders byte-identical pairs at a frozen instant on
  surfaceless GLES. `src/layout/tests/drag_dynamics.rs` (`material-b3ce14`) records
  motion on a pinned clock. Neither run needs a quiet host or a GPU preflight.
- Preflight refusals: GPU thresholds are refused with the glass desktop up
  (`material-6bd4a3`). Fixtures pinned to llvmpipe still gate on the NVIDIA GPU
  (`material-925518`). Timers and the idle lock disturb runs that are already under
  way (`material-188aaa`).
- Edge diagnosis (`material-be611b`, 2026-09-30): the chamfer was a planar ramp,
  so one normal covered its whole width, with global refraction thickness. The
  implementation is reviewed in the glass-edges worktree; its contact sheet and
  owner appearance review remain before merge.

## Constraints

- Pass order: `docs/materials/render-pipeline.md`.
- The v1 contract says opaque pixels bypass the material. `material-7f5751` and
  `material-987655` both break it for windows that opt in, so they need one shared
  opt-in contract, not two.
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
| Evidence instruments | `material-2834d7` (P1) | none, then shrinks quiet | frozen-time fixtures (visual behaviour only) | `material-3a17b8` helpers, then `material-22d78f` migrated; `material-bb8480` calibration runs independently; `material-77be96` is done |
| Quiescence and cost | `material-5d6b2c` (existing) | **quiet** for verification; preparation none | measurements and their conditions; the settle contract | remaining settle lifecycle evidence, then a bounded cost pilot; `material-233295` needs no host |
| Glass optics | `material-6062fd` (P1) | headless, plus owner review | the bevel height field | `material-be611b` implemented and reviewed |
| Material library and composition | `material-3aa1f2` | none or headless; quiet for measured cost (`material-bb3fe5`) | none yet; layer stack is a hypothesis | `material-3fcba2` with single-layer identity preserved |
| Motion and time | `material-53f873` (existing, retitled "Responsive glass that settles completely") | headless/owner; idle for clips | native animations and named responses | owner-judged stills of a rigid tilt (`material-89fb6b`) |
| Signals | `material-6f606b` | none; `material-07bac9` needs a nested compositor | source to store to response | `material-c1330b` replay finding (done) and `material-07bac9` transport/attribution finding |

**Next parallel push.** Instruments: `material-3a17b8`, then `material-22d78f`;
`material-bb8480` can calibrate independently. The shared-target-dir issue
(`material-77be96`) is resolved: worktrees build into their own target dirs and
`tools/target-dir-check` refuses sharing. Signals: `material-07bac9` is the remaining
nested-host demonstration after `material-c1330b`. Optics: run the quiet contact
sheet, obtain owner review, then merge `material-be611b`; prepare ring comparison
cases meanwhile. Motion: frozen stills of a rigid tilt (`material-89fb6b`) for the
owner, before its design (`material-abc08c`). Optics and motion still
compete for shader and configuration code ownership.

The opaque-content opt-in contract stays visible across lanes: content depth
(`material-7f5751`) and inactive desaturation (`material-987655`) settle that
boundary together.

Loose bug and hygiene work fills gaps when the larger lanes are blocked:
`material-698875`, `material-c8732f`, `material-e88df7`, and `material-fa4eec`
(`needs: [owner]`: the owner supplies the screenshot). `material-77be96` moved to the
instruments lane.

How the lanes feed each other: instruments → the visual acceptance checks of every lane. Cost
measurements → budgets for composition, fireflies and lighting, and Fresnel's reflected sample.
Bevel → `material-7f5751`, `material-4e3e9c`, `material-933a8b`. Settle contract →
the motion and signals lanes.

**Pending regrouping (checked 2026-10-03).** `material-be611b` still refuses an edit
here with `stale_copy`: `.worktrees/glass-edges` has the newer record.
`material-77db8a` was dropped unmerged on 2026-10-06. After `material-be611b`'s
branch merges, the integrating agent applies:

```sh
tasks edit material-be611b --parent material-6062fd --need owner
tasks edit material-124f1f --need quiet --need nested
```

`material-124f1f` is the glass-edges-only contact-sheet task and remains parked in
the quiet queue until then. `material-f7eb0b` is done; no regrouping is needed.
