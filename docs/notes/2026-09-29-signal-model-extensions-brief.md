# Deferred signal-model extensions

Scope pass: 2026-09-29. Goal: `material-b5cbd6`. This brief is not an
approved design. The three ideas share §11 of the original signals design;
their original bodies and dependency on `material-a54d89` are preserved.

## Problem

Decide whether the shipped signal model needs distinct timing for Ping,
Done and Error, workspace summaries for bars/overviews, or window rules
that select on attention level. Each extends an existing mechanism, but
none yet has a settled consumer or visual contract.

## Current behaviour and evidence

- The [signals design](../materials/2026-09-02-material-signals-design.md)
  records acceptance at `663202b1` and explicitly defers these three
  questions. Its old ring-placement descriptions are superseded; use the
  [current render pipeline](../materials/render-pipeline.md).
- `src/render_helpers/signal.rs::envelope` gives all impulses an 80 ms
  linear attack and exponential decay with a 350 ms time constant.
  `src/window/signal.rs` expires them after 1.5 s. Kind selects the response
  before solving; `EffectiveImpulse` retains selector, accent and timestamp,
  rather than kind. Different response selectors already distinguish the
  kinds. No comparative evidence of a timing problem was found in the
  selected records. Completed `material-743692` deliberately left impulse
  constants native; its closure does not promise timing controls.
- `niri-ipc/src/lib.rs` exposes `Window.signal` and `Window.workspace_id`.
  `src/ipc/server.rs::ipc_refresh_windows` publishes membership changes as
  `WindowOpenedOrChanged`, folded signal changes as `WindowSignalChanged`,
  and closes as `WindowClosed`. `niri-ipc/src/state.rs` maintains window
  snapshots and applies these events. Workspaces carry native urgency but
  no folded material signal. This supplies a plausible client aggregation
  path; it does not demonstrate a complete consumer or atomic summaries.
- `niri-config/src/window_rule.rs::Match` has source/tag and native urgency
  matches, but no signal-level match. `src/window/mod.rs::window_matches`
  folds the raw signal for source/tag matching. Signal mutations already
  mark rules dirty in `src/window/mapped.rs`; focus demotion and deadline
  expiry also invalidate rules. Existing `src/tests/signal.rs` covers
  native urgency and source/tag selection. This establishes reuse points,
  not a measured cost bound for new level rules.

## Constraints

Sources report facts; configuration chooses appearance. Preserve motion
policy, finite impulse expiry, visibility/input-idle gates and quiescent
redraw behavior. A missing signal differs from an existing Quiet slot;
TTL demotion retains accent and tag. A level rule therefore cannot be
assumed equivalent to a tag that a writer updates only on initial set.

Raw folded level and the rendered, crossfaded level are different inputs.
Matching the latter could introduce animation-driven rule churn. Workspace
summaries also need a defined treatment of absent membership and reconnects.
The sources and glass-response briefs already cover transport and visual
extensions; no open local research task was found answering workspace replay.

## Alternatives

1. **Reuse current mechanisms first (current lean).** Replay workspace
   aggregation in an IPC client, keep the shared impulse envelope, and
   establish an actual level-rule use case before designing syntax.
2. Add native workspace summary events and raw folded-level matching.
   This centralizes semantics, but requires a consumer contract, fold
   definition, snapshot/update behavior and recompute verification.
3. Add per-kind timing controls and rendered-level rules now. Defer this:
   timing lacks a demonstrated problem, and rendered-level matching adds
   state transitions whose need and cost have not been established.

## Unanswered questions

- **Workspace summaries:** Can snapshot/event replay maintain a candidate
  maximum attention level through moves, expiry, clearing, closing and
  reconnect? `material-c1330b` answers this. Which bar/overview needs which
  fields or cross-workspace consistency? The consuming feature's design
  must answer; no consumer implementation was established here.
- **Level rules:** What appearance should external Demand trigger beyond
  existing responses/native urgency? A concrete use case and its design
  must settle exact equality versus threshold, raw versus rendered level,
  and missing versus Quiet. A later bounded check must verify lifecycle
  invalidation and cost; no benchmark result is claimed here.
- **Impulse timing:** Which kind is poorly served by the shared envelope?
  A repeatable visual comparison and the owner's judgment can answer.

## Proposed decomposition

- `material-b5cbd6` groups `material-6cca0a`, `material-d88a8f` and
  `material-1cc048`; none was started or claimed during this pass.
- `material-c1330b` is priority 2, small, mid complexity, direct process.
  It traces and replays the existing IPC reducer, records evidence and a
  recommendation here, and wakes `material-6cca0a` with a finding note in
  its result commit. It adds no compositor API or bar integration.
- `material-6cca0a` and `material-d88a8f` remain briefed ideas. File a
  design follow-up if replay or a concrete rule use case justifies one;
  this brief does not authorize an implementation design.
- `material-1cc048` is shelved until a repeatable comparison identifies a
  specific shared-envelope usability problem and the kind needing different
  timing. Unshelve it with that evidence before reviewing it again.
