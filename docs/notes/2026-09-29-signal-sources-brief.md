# External material signal sources

Scope pass: 2026-09-29; terminal transport evidence added 2026-10-11.
Establish terminal event transport and window attribution before implementing
more sources against the shipped signals contract.
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
  window. The 2026-10-11 probe below demonstrates kitty transport with an
  explicit per-OS-window identity; the general launch policy remains unresolved.
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
hooks were not inspected in this checkout. `material-07bac9` is the bounded
terminal transport investigation; its finding is recorded below. The familiar
bridge is completed work.

## Alternatives

1. **Use terminal or shell hooks plus existing IPC (current lean).** Kitty's
   transport is demonstrated below. Design window identity and source cleanup
   before production work; Ghostty needs a separate adapter investigation.
2. Start with cast indication through existing cast events or native state.
   Observable targets exist, but active versus selected semantics and
   microphone/camera scope remain unresolved.
3. Build one watcher framework or PTY proxy for all sources. Defer this option:
   the sources need different evidence, and no shared transport requirement
   has been demonstrated. Ambient global tuning may need no signal writer.

## Unanswered questions

- **Terminal commands and OSC:** Which supported interfaces deliver start/exit
  and notifications, and how do they identify a niri window when one process
  owns several? Kitty works with explicit launch identity in the probe below;
  Ghostty's external event interface and window join remain unverified. Progress still
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
  its result commit. Its probe is complete; it does not deploy a watcher.
- Privacy, audio and ambient remain briefed. File their bounded evidence or
  design tasks when taking up that direction. This handoff settles no shared
  watcher framework or progress implementation.

## Terminal transport finding — 2026-10-11

`material-07bac9` ran an opt-in probe under headless Weston and nested niri,
using private runtime/config directories and a per-run Weston unit. No host
shell/terminal configuration or launcher was changed. The controller reaped
its processes and stopped the unit. The task's `terminal-transport-probe.md`
attachment contains the assert-based reproduction, verdict, and event log.
This is evidence for a follow-up design, not an approved production design.

### Coverage

| Installed terminal | Command start/exit | Notifications | Numeric progress | Window attribution |
|---|---|---|---|---|
| kitty 0.49.2 | Live Bash integration: `on_cmd_startstop`, exit 0 and 1 in both windows | Live OSC 9/99 through a separate `notifications.py` filter; suppressed before desktop delivery | Live OSC 9;4 value 42, sampled through `on_tab_bar_dirty` | Exact PID + unique app ID, two OS windows in one process |
| Ghostty 1.3.1-arch2 (tip channel) | Installed zsh integration emits OSC 133 C/D with exit status; no external callback demonstrated | OSC 9 documented; installed-build OSC 99 coverage unverified | OSC 9;4 documented; no external delivery demonstrated | Unknown; no live Ghostty probe |

Kitty's [watcher interface](https://sw.kovidgoyal.net/kitty/launch/#watchers)
supplies the terminal window and start/stop data. Its
[notification filter](https://github.com/kovidgoyal/kitty/blob/master/docs/notifications.py)
has a separate callback with a channel ID. Notification and progress updates
are not command-watcher events. The progress sample uses kitty internals,
not a dedicated stable progress callback. Internal field access in the probe
is version-specific; a production adapter should prefer the documented remote
control interface where available.

Ghostty's [shell integration](https://ghostty.org/docs/features/shell-integration)
and [command-finish settings](https://ghostty.org/docs/config/reference#notify-on-command-finish)
establish terminal-side command handling, not export to a niri writer. Its
[OSC 9](https://ghostty.org/docs/vt/osc/9) and
[OSC 9;4](https://ghostty.org/docs/vt/osc/conemu) documentation establishes
notification/progress consumption. Inspection of the installed CLI actions,
default configuration, and shell integration found no equivalent external
watcher or compositor window identity. Record that interface as unknown,
rather than assume unsupported. The [Linux activation documentation](https://ghostty.org/docs/linux/systemd)
also means a new invocation need not imply a new terminal process.

### Identity and lifecycle

The successful run used niri 26.04 (`99196eba`), the installed binary selected
by explicit path, not a build of this task's worktree. Its SHA-256 was
`4b13e677894b228cf4dedee5e6bad91fa826d3ed9d5019ed11b1c400886dfab3`.
The two niri window IDs were 2 and 3, sharing terminal PID 2933031. Kitty's
OS-window IDs were 1 and 2; those are a different namespace. Its remote-control
`platform_window_id` was null on Wayland.

The fixture supplied a distinct app ID per OS window using `kitty --class`
and `kitten @ launch --type=os-window --os-window-class`. It matched that
identity plus the terminal PID to exactly one niri window, requiring uniqueness.
Titles changed normally. PID ancestry alone cannot distinguish these windows:
the familiar join in `integrations/niri/window.js` builds a PID-to-window map
and would collapse their shared PID to one entry. Do not reuse that join for
arbitrary terminal events. This is an applicability limit, not evidence that
the already accepted familiar hue test failed.

The app-ID approach is explicit and works for controlled launches, but changes
existing exact app-ID window-rule matches. It is not a transparent solution
for existing ordinary kitty/Ghostty windows. Production must choose launch
identity or restrict attribution to a unique process/window match and reject
ambiguity; it must not guess from focus or mutable titles. Multiple panes
inside an OS window also need an aggregation policy before sharing one slot.

Normal close emitted `on_close`. The OS-window lookup was already removed
at that point, so the probe retained the identity while the window was live.
Writes to closed niri IDs failed. Restarting kitty created window ID 4 with
no signal, and the old ID still refused writes. Abrupt terminal death and
compositor/bridge disconnect were not injected; those remain lifecycle design
and verification requirements. A compositor restart must invalidate cached
IDs, which are not global identities.

### IPC and next design

The controller consumed real watcher events, wrote `Active` while each
`sleep 1; true` / `sleep 1; false` command was still running, then set the
completion baseline before pulsing `Done` / `Error`. Assertions checked the
target signal and absence of writes to its sibling. It then cleared its own
`terminal-probe` source. These levels are test inputs, not settled policy.

Pulse without a prior set failed without mutation. Clear removed both slot
and impulses. A 150 ms TTL demoted `Active` to `Quiet`/`Static` but retained
the source: TTL does not delete a disconnected writer's slot. Normal window
unmap destroys its store; a bridge that dies while windows survive needs
explicit reconciliation/clearing on restart. A finite TTL limits stale activity
but cannot substitute for clearing. No numeric progress field exists in the
current slot/IPC schema, and impulse animation age is not job percentage.

Recommend a kitty-first adapter using existing set/pulse/clear IPC, owned
here with opt-in terminal/shell installation owned by `dots`. Keep command
facts independent of familiar and appearance tuning in Prism. Prefer this
over a PTY proxy or common watcher framework: one supported transport already
works, and the terminals expose different interfaces.

Request design before implementing `material-d277d0`: choose identity without
silently breaking window rules, define pane aggregation, finite activity leases,
clear/reconnect behavior, and a separate Ghostty shell-hook demonstration.
`material-79d1de` can reuse the identity decision, but notifications need their
own adapter; numeric progress still needs representation/arbitration alongside
`material-5d854f`. No production watcher or progress extension landed here.
