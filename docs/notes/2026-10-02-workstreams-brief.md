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
2. **Shared mechanisms.** Several ideas in different goals are really the same
   abstraction: an ordered layer stack, a named time envelope, the bevel height
   field, or the cost model. Built once, each one unblocks several lanes.

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

- **A. Lanes as parent goals, with a `needs-quiet` tag (taken).** `tasks tree`
  shows each lane. `tasks ready` minus `needs-quiet` lists work that can run at any
  time. Existing goals and their briefs keep their scope.
- B. Lane tags only. This leaves the tree flat, and the 9-29 briefs never get a
  common place to live.
- C. Re-scope everything into new goals. This throws away the 9-29 handoffs for no gain.

## Unanswered questions

- **Order of the optics lane.** Does the ring embedding (`material-4e3e9c`, which
  the owner prioritised on 10-01) wait for the bevel height field, or go first on
  the current chamfer? Owner, after the `material-be611b` plan.
- **Layer-stack contract.** Is the stack a uniform-packed fixed fan-out, as in
  `material-3fcba2`, or a device chain in Prism (`prism-a03862`)? Answer this with
  the noise-layers design, which is the smallest instance.
- **Envelope abstraction.** Can one named-envelope type serve animation profiles
  (`material-9be53d`), transients (`material-d873bf`), impulse shapes
  (`material-1cc048`, shelved), frost-on-idle (`material-4bf8b8`) and the weather
  cycle (`material-f3e4e4`)? A design under the dynamics lane can answer this. It
  is not filed yet; file it when the lane is taken up.

## Proposed decomposition

| Lane | Goal | Host | Shared mechanism it supplies | Next ready step |
|---|---|---|---|---|
| Evidence instruments | `material-2834d7` (new, P1) | none, then shrinks quiet | frozen-clock fixtures; a host class for each check | `material-3a17b8` shared fixture helpers, then `material-22d78f`, `material-bb8480`; `material-925518`/`material-188aaa` cut quiet refusals |
| Quiescence and cost | `material-5d6b2c` (existing) | **quiet** | the cost model; the settle contract | the queue in `tasks quiet`; `material-31074f` once the lifecycle captures clear |
| Glass optics | `material-6062fd` (new, P1) | headless + owner | the bevel height field | `material-be611b` plan (glass-edges worktree); `material-ad1780` diagnosis on the fixture |
| Material library and composition | `material-3aa1f2` (new) | none/headless | the ordered layer stack | `material-3fcba2` as the stack's first instance; `material-bb3fe5` ice |
| Motion and time | `material-53f873` (existing) | headless + owner; idle for clips | named envelopes | `material-77db8a` merge step (needs no host); `material-6d4de5` scope |
| Signals | `material-6f606b` (new) | none | source to store to response | `material-c1330b`, `material-07bac9`, `material-3bdffc` |

Loose bug and hygiene work runs when the larger lanes are blocked, on any host:
`material-698875`, `material-c8732f`, `material-e88df7`, `material-77be96`,
`material-fa4eec`.

How the lanes feed each other: instruments → every lane's acceptance checks. Cost →
budgets for composition, fireflies and lighting, and Fresnel's reflected sample.
Bevel → `material-7f5751`, `material-4e3e9c`, `material-933a8b`. Settle contract →
the motion and signals lanes.

**Pending regrouping.** Another worktree holds a newer copy of each of these tasks,
so they were not edited here. When those branches merge, apply:
`material-be611b --parent material-6062fd` (glass-edges worktree) and
`material-f7eb0b --tag needs-quiet` (the `material-3acc86` worktree).
`material-77db8a` (doing) belongs to the motion lane; reparent it when its claim ends.
