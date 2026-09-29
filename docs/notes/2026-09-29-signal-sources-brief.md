# External material signal sources

Scope pass: 2026-09-29. Establish terminal event transport and window attribution
before implementing more sources against the shipped signals contract.
Goal: `material-9b8bf9`. This brief is not an approved design.

## Problem

Make command completion, terminal notifications/progress, audio activity,
privacy state and ambient lighting useful to material rendering. The five
ideas share §11 of the original signals design; their full outcomes remain
in their task bodies. They need source ownership and lifecycle decisions
before implementation.

## Current behaviour and evidence

- [Signals design](../materials/2026-09-02-material-signals-design.md) §§1–2
  and `src/window/signal.rs` establish source slots, levels, motion,
  set-before-pulse, expiry and explicit clearing. Foundation `material-a54d89`
  landed at `663202b1`; these external sources were deferred.
- `src/cli.rs` and `niri-ipc/src/lib.rs` expose set/pulse/clear requests and
  window signal events. `Slot` and wire `Signal` have no numeric job progress.
  The [glass responses brief](2026-09-29-glass-signal-responses-brief.md)
  already records progress representation/arbitration as unresolved for
  `material-5d854f`; reuse that question before designing an OSC progress channel.
- `Window.pid` is the PID that opened the Wayland connection and is optional.
  It does not establish a unique mapping from a command or media stream to a
  window. No terminal event transport was demonstrated during this pass.
- `material-930c55` records the familiar bridge and successful per-window hue
  checks after restart on 2026-09-24. This establishes an existing join to
  investigate, not evidence that every terminal event carries that identity.
- Cast requests/events expose target and `is_active` in `niri-ipc/src/lib.rs`;
  `src/ipc/server.rs` publishes changes. `refresh_mapped_cast_window_rules`
  in `src/screencasting/mod.rs` marks window targets regardless of activity;
  `Mapped::set_is_window_cast_target` recomputes rules without writing signals.

## Constraints

Sources state facts; configuration chooses the effect. Preserve identity
accents, motion policy, visibility/input-idle gates and settled redraw behavior.
A source clears its own slot; expiry demotes level/motion but retains the slot,
accent and tag. Define restart, window-close and lost-connection behavior so
obsolete state cannot persist indefinitely.

Cast targets and recording applications are different subjects. The cast API
does not establish microphone/camera attribution or full-output privacy coverage.
Prism remains the original design's global parameter owner; its current external
hooks were not inspected in this checkout. No existing open research task here
answers terminal transport; the familiar bridge is completed work.

## Alternatives

1. **Establish terminal transport first (current lean).** Prefer available
   terminal or shell hooks plus existing IPC and a proven window join.
   Investigate coverage before choosing the component's owning project.
2. Start with cast indication through existing cast events or native state.
   Observable targets exist, but active versus selected semantics and
   microphone/camera scope remain unresolved.
3. Build one watcher framework or PTY proxy for all sources. Defer this option:
   the sources need different evidence, and no shared transport requirement
   has been demonstrated. Ambient global tuning may need no signal writer.

## Unanswered questions

- **Terminal commands and OSC:** Which supported interfaces deliver start/exit
  and notifications, and how do they identify a niri window when one process
  owns several? `material-07bac9` answers the transport question. Progress still
  needs representation, expiry and arbitration coordinated with
  `material-5d854f`; impulse animation age is not completion percentage.
- **Privacy:** Indicate selected or actively streaming targets, captured
  windows or consuming apps, and how should output capture be represented?
  A later privacy design settles scope; PipeWire/portal evidence must establish
  microphone/camera ownership before implementation.
- **Audio:** Can stream identity map uniquely to a window, including absent
  PIDs, shared processes and multiple streams? A bounded attribution probe
  must answer before choosing a watcher and clearing policy.
- **Ambient:** Should time/scheme change global attenuation temperature or
  per-window state? Lean toward Prism for a global parameter; a later design
  must settle the intended appearance and verify available scheme/time hooks.

## Proposed decomposition

- `material-9b8bf9` groups `material-d277d0`, `material-79d1de`,
  `material-588ca9`, `material-e91ae9` and `material-f41c54`. They remain
  unclaimed ideas with their original source material preserved.
- `material-07bac9` is priority 2, small, mid complexity and direct process.
  It records terminal event coverage, window mapping, lifecycle and ownership
  here and wakes `material-d277d0` and `material-79d1de` with finding notes in
  its result commit. It does not implement or deploy a watcher.
- Privacy, audio and ambient remain briefed. File their bounded evidence or
  design tasks when taking up that direction. This handoff settles no shared
  watcher framework or progress implementation.
