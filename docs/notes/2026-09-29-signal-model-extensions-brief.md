# Deferred signal-model extensions

Scope passes: 2026-09-29 and 2026-10-06. Goal: `material-b5cbd6`. This brief is not an
approved design. The three ideas share §11 of the original signals design;
their original bodies and dependency on `material-a54d89` are preserved.

## Problem

Decide whether the shipped signal model needs distinct timing for Ping,
Done and Error, workspace summaries for bars/overviews, or window rules
that select on attention level. Each extends an existing mechanism, but
none yet has a settled consumer or visual contract. The 2026-10-06 pass pairs
`material-6cca0a` with inactive desaturation (`material-987655`) in the
[glass-response brief](2026-09-29-glass-signal-responses-brief.md): both need
a consumer or appearance contract before extending the shipped signal system.

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
  no folded material signal. Completed replay `material-c1330b` (`4a586bc8`)
  establishes a correct client-side maximum-level fold through moves, TTL
  demotion, clear, close and reconnect. It establishes neither a consumer,
  refresh-atomic summaries nor measured cost; details remain below.
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
extensions. Workspace replay is complete; do not repeat it or file a compositor
API design without a consumer requirement that the existing fold cannot meet.

## Alternatives

1. **Reuse current mechanisms first (current lean).** Use the demonstrated
   client-side workspace fold, keep the shared impulse envelope, and
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
  reconnect? Yes, with the gaps recorded under
  [Workspace replay](#workspace-replay) (`material-c1330b`). Which bar/overview needs which
  fields or cross-workspace consistency? The consuming feature's design
  must answer; no consumer implementation was established here.
- **Level rules:** What appearance should external Demand trigger beyond
  existing responses/native urgency? A concrete use case and its design
  must settle exact equality versus threshold, raw versus rendered level,
  and missing versus Quiet. A later bounded check must verify lifecycle
  invalidation and cost; no benchmark result is claimed here.
- **Impulse timing:** Which kind is poorly served by the shared envelope?
  A repeatable visual comparison and the owner's judgment can answer.

### Workspace replay

Result of `material-c1330b` (2026-10-02). The replay is
`niri-ipc/tests/workspace_signal_replay.rs`: it feeds compositor-shaped
events through the existing reducer (`niri_ipc::state::EventStreamState`),
keeps a compositor-side mirror beside one client as `server.rs` does, and
folds `max(Window.signal.level)` per known workspace from the reducer's window
map, with `None` (no signalled window) kept distinct from `Some(Quiet)`. The
fold is a candidate summary, not an approved accent, motion or impulse fold.

**Emit order, from the compositor code** (line numbers at `6c62457f`):

- One refresh per event-loop dispatch (`src/main.rs:270`) runs
  `refresh_signal_deadlines` then `ipc_refresh_layout` (`src/niri.rs:844-846`),
  which runs workspaces, then windows, then overview
  (`src/ipc/server.rs:674-678`). Each event is applied to the compositor's
  mirror, then sent to every stream individually (`server.rs:785-788`,
  `897-900`, `send_event` at `115`). Nothing on the wire marks a refresh
  boundary.
- A known window whose `workspace_id`, floating state, title or app id changed
  gets one full `WindowOpenedOrChanged` built by `make_ipc_window`, which folds
  the signal at refresh time (`server.rs:824-835`, `617-620`); the function
  returns, so no `WindowSignalChanged` for that window follows in that
  refresh. A move therefore updates both workspaces in one event.
- Otherwise a folded-signal difference emits `WindowSignalChanged`
  (`server.rs:857-865`). The compared value includes impulses, so impulse
  appearance and expiry emit events whose level is unchanged.
- TTL demotion: `signal_deadline_fired` (`src/niri.rs:2381-2393`) mutates the
  slots via `advance` and queues a redraw; it sends no IPC itself. The refresh
  after that dispatch emits `WindowSignalChanged`. `fold` also applies an
  expired TTL lazily (`src/window/signal.rs:339-342`), so any refresh at or
  after the deadline already reports the after-level. Demotion keeps the slot,
  so the window keeps `Some(signal)`.
- Clearing the last slot makes `fold` return `None`
  (`signal.rs:242-251`, `335-338`) and emits `WindowSignalChanged` with
  `signal: None`.
- Close: windows missing from the layout get a bare `WindowClosed`, after all
  per-window events and the layout batch (`server.rs:879-885`).
- An interactively moved window is reported with `workspace_id: None`
  (`src/layout/mod.rs:1696-1700`), which changes `workspace_id` and so emits
  `WindowOpenedOrChanged` on pickup and drop.
- New stream client: the reply, then `replicate()` of the mirror —
  `WorkspacesChanged`, `WindowsChanged`, then the other parts
  (`niri-ipc/src/state.rs:99-108`) — queued before the sender joins the
  stream list, with no await in between (`server.rs:246-264`). The snapshot
  is the mirror as of the last refresh. A client that falls 64 events behind
  is dropped (`server.rs:42`, `115-131`) and must reconnect.

**Replay and observed levels** (workspaces 1, 2, 3; windows 10, 11 on 1, 20
on 2, 30 alone on 3 with Demand before connect):

| Step | Event(s) | ws1 | ws2 | ws3 |
| --- | --- | --- | --- | --- |
| S0 connect | snapshot | none | none | Demand |
| S1 Quiet on 20 | `WindowSignalChanged` | none | Quiet | Demand |
| S2 Notice on 10 | `WindowSignalChanged` | Notice | Quiet | Demand |
| S3 Demand+TTL+Ping on 11 | `WindowSignalChanged` | Demand | Quiet | Demand |
| S4 impulse expiry on 11 | `WindowSignalChanged`, same level | Demand | Quiet | Demand |
| S5 11 to ws2 | one `WindowOpenedOrChanged` | Notice | Demand | Demand |
| S6 TTL demotes 11 to Active | `WindowSignalChanged` | Notice | Active | Demand |
| S7a ws3 removed | `WorkspacesChanged` | Notice | Active | gone; 30 unattributed |
| S7b 30 to ws1 | `WindowOpenedOrChanged` | Demand | Active | — |
| S8a drag 30 | `WindowOpenedOrChanged`, ws `None` | Notice | Active | 30 unattributed |
| S8b drop 30 on ws2 | `WindowOpenedOrChanged` | Notice | Demand | — |
| S9 clear 10's last source | `WindowSignalChanged`, `None` | none | Demand | — |
| S10 close 30, close 11 | `WindowClosed` ×2 | none | Active, then Quiet | — |
| S11 reconnect after missing a clear on 20 and a Notice window 12 opening on ws1 | snapshot | none, then Notice | none | — |

At every step the client's fold equals the fold of the compositor mirror.
In S11 the stale client would have kept ws2 at Quiet; a fresh state folds
every workspace to `none` after the snapshot's first event and to the correct
levels after `WindowsChanged`. Reusing the old reducer state also converges,
because `WindowsChanged` replaces the whole window map.

**State gaps.**

- No refresh boundary on the wire: per-event folds expose intermediate
  states. S7 shows one: if a workspace removal and the move off it land in
  one refresh, `WorkspacesChanged` arrives first and the moved window's
  Demand belongs to no known workspace until its `WindowOpenedOrChanged`.
  Whether two windows changing in one refresh can be observed as one
  cross-workspace update is unknown; the replay cannot show it, and nothing
  in the code guarantees it.
- Reconnect has a transient all-`none` state between `WorkspacesChanged` and
  `WindowsChanged`.
- A dragged window has no workspace, so its level leaves every workspace for
  the duration of the drag.
- Impulse-only changes cost an event and a re-fold without a level change.
- Nothing measured cost or latency; none is claimed.

**Recommendation.** Keep the summary client-side; do not add a compositor
summary event now. The existing events carry enough to maintain a correct
candidate maximum through moves, TTL demotion, clearing, closing and
reconnect, provided the client recomputes from reducer state rather than
keeping incremental counters, folds only over known workspace ids, keeps
`None` distinct from `Quiet`, and resyncs from a fresh snapshot on reconnect.
A compositor summary becomes worth designing only if a consumer needs
refresh-atomic summaries across workspaces, a defined attribution for dragged
windows, a fold beyond level defined in one place, or a measured client cost
it cannot bear.

**Consumer requirements still unknown.** Which bar or overview consumes the
summary, and which fields (level only, or accent, motion, impulses, source
count); whether intermediate per-event states and the reconnect blank are
acceptable or need coalescing; how a dragged window should count; whether
Quiet should render differently from no signal; whether native urgency
(already `Workspace.is_urgent`) should merge with the summary; and the cost
and latency budget per event.

## Proposed decomposition

- `material-b5cbd6` groups `material-6cca0a`, `material-d88a8f` and
  `material-1cc048`; none was started or claimed during this pass.
- `material-c1330b` is complete (`4a586bc8`); its result and wake-up note
  support client-side aggregation. No replacement research task is needed.
- `material-6cca0a` is shelved until a named bar/overview requires
  refresh-atomic cross-workspace summaries, defined drag attribution, a shared
  fold beyond maximum level, or demonstrates unacceptable client folding
  cost. Record the requirement and budget before unshelving.
- `material-d88a8f` remains a briefed idea, unchanged by the follow-up pass.
  A concrete rule use case must justify a design; this brief authorizes none.
- Related `material-987655` remains under its existing response goal and
  reuses `material-d257d9` for the opt-in client-content boundary. No new
  goal, summary event or duplicate design task was created in this pass.
- `material-1cc048` is shelved until a repeatable comparison identifies a
  specific shared-envelope usability problem and the kind needing different
  timing. Unshelve it with that evidence before reviewing it again.
