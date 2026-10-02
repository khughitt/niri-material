# Workstreams: parallel lanes over the open material work

Status: scoping handoff, 2026-10-02. Not a design; it groups existing goals and
loose tasks into lanes that can run side by side. When one lane is waiting on a
quiet host or a review by the owner, work continues in the others.

## Problem

There are 72 open tasks: 8 goals from the 2026-09-29 scope pass and about 25 loose
ideas. They were grouped by subsystem, which hides two things that decide what can
run at once:

1. **Host class.** A check needs one of three hosts. `none` is code, frozen-clock
   fixtures, docs and design. `headless` is a nested niri on the headless weston unit.
   `quiet` means a TTY with the desktop stopped, for power, timing and settle evidence.
   On top of any of these, the owner may have to judge sheets.
   Only the `quiet` class serialises.
2. **Shared mechanisms.** Two are established: frozen-time fixtures and the bevel
   height field. Each one, built once, unblocks several lanes. Two are hypotheses
   to test against a second concrete consumer before extracting them: an ordered
   layer stack and named time envelopes. The cost model is shared as measurements
   and conditions, not as summed per-effect prices.

## Current behaviour and evidence

- The quiet queue holds the settle-lifecycle captures (`material-80caf4`,
  `material-3acc86`, `material-1af3c6`, `material-f7eb0b`) and the view-tilt smoke
  (`material-77db8a`). The cost study `material-31074f` is the wake hub for
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
- Edge diagnosis (`material-be611b`, 2026-09-30): the chamfer is a planar ramp, so
  one normal covers its whole width. Refraction uses a global thickness. The spec is
  accepted, and its plan is being written in the glass-edges worktree.

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
  scheduled by its own constraints, not by its lane. Tags mark known gates:
  `needs-quiet`, `needs-nested` (a nested compositor) and `needs-owner` (the owner
  supplies or judges an image). They are not exhaustive: a task without a tag is not
  automatically runnable anytime, and its checks name their host class. Where a task
  bundles preparation with a quiet run, split the two when it is taken up
  (`material-31074f`).
- B. Lane tags only. This leaves the tree flat, and the 9-29 briefs never get a
  common place to live.
- C. Re-scope everything into new goals. This throws away the 9-29 handoffs for no gain.

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

## Proposed decomposition

| Lane | Goal | Host | Established shared ground | First milestone |
|---|---|---|---|---|
| Evidence instruments | `material-2834d7` (new, P1) | none, then shrinks quiet | frozen-time fixtures (visual behaviour only) | `material-3a17b8` helpers, then `material-22d78f` migrated; `material-bb8480` calibration runs independently; `material-77be96` before more concurrent builds |
| Quiescence and cost | `material-5d6b2c` (existing) | **quiet** for verification; preparation none | measurements and their conditions; the settle contract | remaining settle lifecycle evidence, then a bounded cost pilot; `material-233295` needs no host |
| Glass optics | `material-6062fd` (new, P1) | headless, plus owner review | the bevel height field | `material-be611b` implemented and reviewed |
| Material library and composition | `material-3aa1f2` (new) | none or headless | none yet; layer stack is a hypothesis | `material-3fcba2` with single-layer identity preserved |
| Motion and time | `material-53f873` (existing, retitled "Responsive glass that settles completely") | headless/owner; idle for clips | native animations and named responses | `material-77db8a` brought current and verified |
| Signals | `material-6f606b` (new) | none; `material-07bac9` needs a nested compositor | source to store to response | `material-c1330b` replay finding and `material-07bac9` transport/attribution finding |

**Next parallel push.** Instruments: `material-3a17b8`, then `material-22d78f`, and
investigate `material-77be96` first. All three checkouts (main, glass-edges,
material-77db8a) share one `target-dir`, checked 2026-10-02; that the configuration
persists is established, the cause of the reported failures is not. Signals:
`material-c1330b`, with `material-07bac9` as a separate nested-host demonstration.
Optics: finish and review the `material-be611b` plan and prepare ring comparison
cases. Bring `material-77db8a` current soon: its recorded conflicts touch the shader
and configuration files the optics work will change, so it competes for code
ownership although it needs no quiet host.

The opaque-content opt-in contract stays visible across lanes: content depth
(`material-7f5751`) and inactive desaturation (`material-987655`) settle that
boundary together.

Loose bug and hygiene work fills gaps when the larger lanes are blocked:
`material-698875`, `material-c8732f`, `material-e88df7`, and `material-fa4eec`
(`needs-owner`: the owner supplies the screenshot). `material-77be96` moved to the
instruments lane.

How the lanes feed each other: instruments → the visual acceptance checks of every lane. Cost
measurements → budgets for composition, fireflies and lighting, and Fresnel's reflected sample.
Bevel → `material-7f5751`, `material-4e3e9c`, `material-933a8b`. Settle contract →
the motion and signals lanes.

**Pending regrouping.** Another worktree holds a newer copy of each of these tasks,
so they were not edited here. When those branches merge, apply:
`material-be611b --parent material-6062fd` (glass-edges worktree) and
`material-f7eb0b --tag needs-quiet` (the `material-3acc86` worktree).
`material-77db8a` (doing) belongs to the motion lane; reparent it when its claim ends.
