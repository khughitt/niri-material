# Real-TTY lane for optic settling acceptance

**Status:** accepted (owner review round 3, 2026-10-02), after revisions for
VT restoration on every exit, pixel evidence during the cast, damage-driven
cast samples and screencast in the development check.
**Tasks:** `material-f7eb0b` (TTY resume and unlock), `material-3acc86`
(screencast consumer), under `material-f86183`. The display-dimming helper
is `ops-a1715a` (see §6).
**Extends:** [sustained optic settling design](2026-09-29-sustained-optic-settling-design.md)
§§6–8 and its [acceptance evidence](../materials/2026-09-30-optic-settling-evidence.md).

## 1. Intent

The headless lane passed every case it ran on 2026-10-02; its
`idle-inhibitor` case (`material-80caf4`) is implemented but awaits its
capture. Three acceptance cases remain unverified because they need niri on
real DRM:

- **`tty-resume`:** session activation after a VT switch resumes a held
  timeline, with no catch-up.
- **`unlock`:** the session-lock unlock path resumes it, separately from IPC
  power-on.
- **`screencast`:** a real PipeWire consumer and client updates while idle
  leave the field held. Nested niri cannot cast at all, because
  `Backend::gbm_device` is `None` for winit and headless.

This design adds the `dedicated` lane to the existing settling driver so
those cases run on DP-1 from a TTY, overnight, unattended. A dim monitor
during those runs is part of the protocol (§6).

Success: a lane pilot, then a matrix, pass on this host; each case's
verdict comes from the existing offline analyzer; the monitor is dimmed
while the lane runs and back at its original brightness afterwards.
Out of scope: power measurement, two outputs (`material-1af3c6`), any change
to the installed compositor or its launcher.

## 2. Owner decisions (2026-10-02)

| Question | Decision |
| --- | --- |
| VT switching | A NOPASSWD sudoers rule for `/usr/bin/chvt`, installed by the owner. |
| Unlock | A PAM-free lock client that locks on start and unlocks on a signal. No password exists to type or store. |
| Display | The lane takes DP-1 from the stopped desktop, as the idle-budget power lane does. |
| Screencast consumer | GStreamer `pipewiresrc ! appsink` from `gst-plugin-pipewire`, installed by the owner. |
| Dimming | DDC/CI through `ddcutil`, with the owner's user granted i2c access. |

## 3. Host prerequisites (owner actions)

The commands are recorded in [capture host setup](../materials/capture-host-setup.md),
which covers every capture lane; the reference host completed these on
2026-10-02. The driver checks items 1, 2 and 4 and refuses with the missing
item named; none is installed or granted by an agent.

1. `sudo -n chvt <n>` succeeds (NOPASSWD rule for `/usr/bin/chvt` only).
2. `gst-inspect-1.0 pipewiresrc` finds the element.
3. `ddcutil detect` lists DP-1's monitor without sudo (the package's
   uaccess rule on the active seat). This serves dimming only: a missing
   `ddcutil` or a monitor that rejects DDC leaves the run undimmed (§6).
4. The session starts from a TTY login with the desktop stopped, the
   existing `quiet --needs headless` contract.

## 4. Lane mechanics

The driver's `--lane dedicated` replaces its refusal stub. The case table,
journal, stimulus scheduling, Tracy capture, export, observation file and
analyzer stay as they are; only the host differs.

**Launch.** `start_drm` follows idle-budget's: niri from the agent's TTY
shell with `WAYLAND_DISPLAY`, `WAYLAND_SOCKET`, `DISPLAY` and `NIRI_SOCKET`
unset and a short `XDG_RUNTIME_DIR`. Every case pins DP-1's mode and scale
in its config and records the actual topology from `niri msg -j outputs` in
`observation.json` (today's lane writes `headless-1` unconditionally). The
running executable is compared with the identified snapshot after launch.
Between cases the driver waits for the GPU's P8 rest (idle-budget's
`await_gpu_rest`) before the next settle check.

**Private session bus.** Each case runs its own `dbus-daemon --session
--nofork` under the case's runtime directory, owned and reaped by the
driver. niri starts with `DBUS_SESSION_BUS_ADDRESS` pointing at it and
`debug { dbus-interfaces-in-non-session-instances; }` in the config, so no
compositor interface reaches the user's bus. `PIPEWIRE_RUNTIME_DIR` points
at the user's PipeWire daemon.

**Input.** The headless lane's `wlrctl pointer move` goes through niri's
virtual-pointer protocol and stays the pointer stimulus here.

**VT switch.** `vt_away` runs `sudo -n chvt <spare VT>`, waits, then
`sudo -n chvt <home VT>`. The home VT is the one active when the driver
starts (`/sys/class/tty/tty0/active`), which is the TTY the session runs on
and the one niri takes. The spare VT is the lowest one with no logind
session, chosen before the run and recorded. Each switch is verified by
reading `tty0/active` back within 2 s; a return switch that does not land
fails the case.

**VT restoration on every exit.** libseat's logind backend releases seat
control on exit without switching back, so a TERM or a failed return during
`vt_away` would leave the host showing the spare VT. The driver therefore
records the home VT before the first switch, and `on_exit`, after reaping
its processes, restores it when `tty0/active` differs: up to three
`sudo -n chvt <home>` attempts, each verified within 2 s. The outcome
(`not-needed`, `restored`, or `failed` with the observed VT) is written to
the run's `vt-restore.json` and printed. A failed restoration makes the run
exit nonzero, even when every case passed, and is named in its `run:` note.
Restoration runs before the capture lock is released, so the next queued
run never starts on the wrong VT.

**Lock client.** `session-lock-client.c`, built per run like
`idle-inhibit-client.c` and identified with its source: it binds
`ext_session_lock_manager_v1`, locks, gives every output a solid lock
surface, prints `locked` once the compositor sends `locked`, and on SIGUSR1
calls `unlock_and_destroy` and exits 0. It never authenticates; niri's
`SessionLockHandler::unlock` runs exactly as for any lock client.

**Screencast consumer.** `screencast-consumer.py` (committed in
`material-3acc86`): a ScreenCast session on the private bus, `RecordMonitor`
DP-1, `pipewiresrc ! glupload ! glcolorconvert ! gldownload ! videoconvert ! video/x-raw,format=RGB ! appsink`; one
CLOCK_MONOTONIC line per received frame, `ready <node>` on the first, and on
SIGTERM a summary with consumer identity, node, frame count and capture
interval. It also keeps frames, not only their times: on SIGUSR1 it writes
the **next frame received after the signal** as raw RGB to the path named in
a request file, with that frame's arrival time and the request time in a
sidecar. A sample can therefore never be a frame the consumer already held.
niri offers DMA-BUFs only, so the consumer imports whatever modifier niri
fixates through GStreamer GL on headless EGL (`glupload ! glcolorconvert !
gldownload`) and converts to RGB; a probe on 2026-10-02 imported niri's
NVIDIA-tiled `XR24` buffers this way. The development check (§7) proves
sampling works before the pilot.

**Cleanup.** The driver owns niri, the bus, the consumer and the lock
client; `on_exit` and the per-case reap cover all four, then VT restoration
runs as described above. A run interrupted
while the lock client holds the session leaves niri locked; killing niri
ends that, and the next case starts a fresh niri.

## 5. Cases

All start from the headless lane's scene (probe kitty under Aurora 0.5 at
4 Hz, a focus thief, 5 s threshold, keepalive until about 17.5 s).

| Case | Edges | Stimuli while held | Verdict adds |
| --- | --- | --- | --- |
| `aurora-full` (lane control) | 0, 1 | none; pointer resume | Proves DRM cadence and settling are measured like headless. |
| `tty-resume` | 0, 1, 0 | `vt-away` (switch out about 4 s, then back) | The resume edge falls inside `vt-away`, carries the held logical time, and the timeline settles again before the end. |
| `unlock` | 0, 1, 0 | `lock` (client locks), then `unlock` (SIGUSR1) | No edge during `lock`; the resume edge falls inside `unlock`; a second pause follows without input. |
| `screencast` | 0 | `cast-start`, `sample-1`, `sample-2`, quiet ≥ 2 s, `sample-3`, `cast-stop` | No resume edge; every sample's frame was produced by its own journaled damage; cast samples show changed client content and unchanged Aurora (below). |

**Cast pixel evidence.** Frame arrival alone would pass a stale stream, and
the headless lane's held pair is a screenshot taken after the capture, not
a cast frame. niri renders no cast frame without damage
(`src/screencasting/pw_utils.rs`, "no damage, skipping frame"), so a sample
requested in a static scene would wait forever. Every sample is therefore
one journaled stimulus that **arms the request first, then causes bounded
client damage**, and its window covers both:

- **`sample-1`:** arm, then one line in the focus thief (`gos-other`).
- **`sample-2`:** arm, then one line in the probe: the client change the
  sample must show.
- **`sample-3`:** at least 2 s after `sample-2`'s window ends with no
  stimulus (the held interval the analyzer observes as quiet), arm, then one
  more line in the focus thief.

In this case the focus thief reads lines from a FIFO, like the probe, so
its damage is bounded and falls outside both sampled regions: the thief is
a separate column, never under the probe's crops. The consumer saves the
first frame that arrives after the request, which is the frame that
damage produced. From each sample the driver crops two regions in output
pixels (DP-1's scale is pinned to 1), calibrated once per run from an opaque geometry probe in the same
two-window layout (niri's IPC reports no on-screen position for tiled
windows), as the headless lanes' `calibrate_probe_rect` does:

- **client:** the probe's text rows where the client line appears;
- **aurora:** a glass region of the probe with no text, the region the
  headless `client-damage` case already uses for its Aurora crop.

The verdict requires the client crop to differ from `sample-1` to `sample-2`
(new content reached the cast) and to be equal from `sample-2` to `sample-3`
(the thief's damage did not touch it), the Aurora crop to be equal across all
three samples (the cast renders held optics), and each sample's frame to have
arrived after its request and inside its stimulus window (no stale frame).
A request with no frame inside its window fails the case; it is never
retried.
Equality is exact on decoded bytes, as for the headless pixel pairs.

New analyzer inputs, all generic:

- `edge_in`: a declared edge index must fall inside a named stimulus's
  journaled window (the resume caused by the VT switch or the unlock, not by
  stray input).
- `consumer_frames`: for the screencast case, the consumer's frame journal
  must hold at least one frame inside each named stimulus window, and its
  summary must report a clean signal stop.
- `samples`: a cast sample's frame must arrive after its request and inside
  its stimulus window, and a sample stimulus must hold a frame at all; its
  crops enter the existing pixel pairs (`expect: equal` or `different`).
  The existing stimulus-free window check covers the quiet interval before
  `sample-3`.
- The idle-inhibitor's `messages` check (`material-80caf4`) is reused as is.

The second pause in `tty-resume` and `unlock` shows the timeline returns to
holding after a resume that came from no input: the settling design's §8
requires that "each resumed case settles again without further
notifications".

## 6. Display dimming for overnight runs

A TTY has no display sleep, and a quiet session holds the monitor lit for
hours. Dimming is a host concern every project's quiet runs share, so the
helper lives in ops, not here: `display-dim set <fraction>` /
`display-dim restore` / `display-dim status` (`ops-a1715a`).

Its contract, which this lane relies on:

- `set` reads VCP 0x10 (brightness) and its maximum through `ddcutil`,
  writes the original and the display's identity to
  `$XDG_STATE_HOME/display-dim/<display>.json` **only if no record exists**,
  then sets `round(fraction × max)`. A second `set` keeps the first original.
- `restore` writes the recorded original back and removes the record; with no
  record it does nothing and says so.
- A display that rejects DDC makes `set` exit with a distinct status and no
  record; the caller continues undimmed and notes it. Dimming never gates a
  capture.
- Brightness is the monitor's backlight: scanout pixels, screenshots and
  casts are unaffected, so captures need no correction.

Protocol, added to this repository's quiet-session instructions: an agent
running the quiet queue from a TTY runs `display-dim set 0.25` before the
first run and `display-dim restore` after the last one, before its final
report. The record makes a crashed session's successor restore it:
`display-dim status` at session start reports a stale record. The
fraction stays in the instructions (0.15–0.35 is the owner's range).

## 7. Pilot and matrix

Before the lane pilot, the smallest end-to-end check runs `aurora-full`,
`tty-resume` and `screencast` (`CASES`, a development run), through export,
cast sampling, pixel comparison, analysis and cleanup, and its result is
read. The `screencast` case there is complete: it proves the consumer gets
CPU-readable frames from niri's stream and that every sample request is
answered by its own damage before any longer run depends on either. The pilot then runs the four cases
once; the matrix repeats `tty-resume` and `unlock` three times each, gated
on the passed pilot with identical binary and configs, as the headless
lane's matrix is. Estimated wall-clock: build 4 min, development check
5 min, pilot 8 min, matrix 15 min. Every attempt gets a `run:` note.

The evidence document gains a dedicated-lane verdict table with topology,
consumer identity, lock-client and consumer hashes, and moves the three
cases out of Unverified only on a passed lane.

## 8. Verification of the tooling itself

- Analyzer: unit tests for `edge_in` and `consumer_frames`, in the existing
  `tools/test_optic_settling.py` style, including a resume edge outside its
  stimulus and a consumer with no frame in a window.
- Driver: the existing stub-tools cleanup tests extended so the dedicated
  lane's bus, consumer and lock client are reaped on TERM.
- VT restoration: with a stub `chvt` and a stub `tty0/active` file, TERM
  during `vt_away` ends on the home VT with `restored`; a stub that refuses
  to switch back yields `failed`, a nonzero exit and the observed VT; a run
  that never switched records `not-needed`.
- Consumer sampling: a unit test drives the sample request against a
  synthetic frame source and rejects a frame that arrived before the request.
- Analyzer: a sample stimulus with no frame in its window fails the case,
  and so does a `sample-3` whose preceding quiet interval held a redraw.
- Lock client: built with `-Wall -Wextra -Werror` in the driver; a headless
  Weston smoke shows lock, `locked`, SIGUSR1 unlock and exit 0 before any
  TTY run (nested niri supports session lock).
- Prerequisite checks: each refusal names its missing item (§3).

## 9. Alternatives

- **A separate TTY driver.** Rejected: it would duplicate the manifest,
  journal and analyzer contract the headless lane already proves.
- **The idle-budget fixture in niri-experiments.** Rejected: settling
  evidence would split across repositories, and its power focus differs.
- **Authenticate a real lock client (swaylock with a test password).**
  Rejected by the owner: a stored password and keyboard injection for no
  gain, since niri's unlock handler is the same either way.
- **Dim with a gamma ramp in niri.** Rejected by the owner: the backlight
  stays full, it does nothing on the bare console, and it alters the output
  under capture.
