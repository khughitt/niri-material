# Hold host disturbers for the length of a quiet capture

**Status:** draft for owner review, 2026-10-04 (revised after agent review
rounds 1–3 and owner-relayed review round 4).
**Task:** `material-188aaa`, under `material-2834d7`.
**Extends:** [capture protocol design](2026-09-11-material-capture-protocol-design.md)
(`tools/capture-meta`: preflight, settle, release).

## 1. Intent

`capture-meta preflight` samples the host once, before the run. Anything the
host schedules after that sample lands inside the measurement unseen. Three
captures have been lost or put at risk this way:

- **2026-09-24 16:06:58** — `wali-rotate.timer` (every 15 min) rotated the
  wallpaper, ran Prism's apply and reloaded the desktop niri; the view-tilt
  smoke's nested kitty never mapped and the smoke failed after 13 minutes.
- **2026-09-24 16:54:05** — the desktop idle lock raised the GPU's P8 floor
  from 9.9 W to about 11.5 W mid-run; the settle gate refused
  `material-4241c3` at case 12 of 24 (22 minutes lost). A monitor wake at
  about 05:08 that morning would have done the same to a running capture.
- **2026-10-03** — with the desktop stopped, `dropbox.service` crash-loops
  (its override sets `DISPLAY=:0`; xwayland-satellite panics without a
  compositor) and restarts every ~14 s; one restart put a P5 sample in a
  settle window and refused the run.

This design makes preflight hold the host's known disturbers for the whole
run, record what it held and how to restore it, and restore it on every exit.
Release then scans the journal over the run window, so a result taken while
something fired anyway is marked as disturbed and fails its fixture rather
than being silently accepted.

Success: on this host, a capture started from a TTY or from the desktop runs
with every user timer, the declared services and (on the desktop) idle and
monitor power held; `capture.json` lists each held item; every item is back
in its prior state after the run, including after a refused preflight and
after the fixture is killed (§4.4); a timer that fires during the run is
named in `capture.json` and fails the fixture, and so does a held monitor
that wakes; `SHA256SUMS` covers the finished `capture.json`.

Out of scope: system timers (stopping them needs root; §5 detects them
instead) and lanes on other hosts beyond what §3.2's config file allows.

## 2. Decisions

| Question | Decision | Rejected |
| --- | --- | --- |
| Which timers | Every active, non-transient *user* timer, discovered at hold time. | A curated list: it misses the next timer someone installs (this host has 13; `familiar-reap` fires every minute) and puts host unit names in the repository. |
| Which services | Units named in a per-host file (§3.2). Reference host: `dropbox.service`. | Holding every user service: most are the session itself. An environment variable: easily missing in a fresh TTY login. |
| Desktop idle | `noctalia msg caffeine-enable` for the run, `caffeine-disable` after. Idle then neither locks nor blanks mid-run. | Locking first: the lock surface itself renders and changes state when it times out. A Wayland idle-inhibit client: it needs a visible surface on the desktop. Refusing while already locked: a held lock does not change state, and refusing would block the common "lock and walk away" start. |
| Monitors | `niri msg action power-off-monitors` after caffeine, `power-on-monitors` on restore. Input still wakes them (`src/input/mod.rs`), so the guard watches each connected connector's kernel DPMS state and a wake marks the run disturbed (§4.4, §5). | Leaving them on: the desktop's redraws failed the GPU P8/IQR gate on 2026-09-24 until the monitors were off. Suppressing the wake in niri: a compositor change and an installed-desktop rollout for a capture-side problem. |
| Where | Inside `capture-meta preflight` and `release`, always on. Every fixture already calls both. | A wrapper fixtures opt into: one forgotten fixture is an unheld run. |
| Disturbed run | `release` restores, records, then exits 1. The four fixtures whose cleanup runs `capture_meta release "$OUT" \|\| true` change to `\|\| rc=1`, as `optic-settling-smoke.sh` already does. | Exit 0 with a warning: four of five fixtures would pass a disturbed run. |
| Checksums | Release completes `capture.json`, so it runs before any manifest that covers it: `glass-optic-smoke-lib.sh`'s `finish` releases, then writes `SHA256SUMS`; `optic-settling-smoke.sh`'s exit path releases before `write_sums`, which then covers `capture.json` too. A second release of a finished run changes nothing and exits with the recorded verdict's code (§4.3). | Excluding `capture.json` from every manifest: the record of the hold would be the one unprotected file. |
| A killed fixture | A guard unit, started and confirmed *before* the first change to the host, restores the hold when the owner dies (§4.4). | Waiting for the next preflight or a manual `restore`: timers, caffeine and dark monitors could stay held for days. |

## 3. What is held

### 3.1 User timers

At hold time, `systemctl --user list-timers --output=json` lists the active
timers; only `unit` and `activates` are read (systemd 262 gives `next` as
absent or 0 for some timers). A timer whose `Transient` property is `yes` is
recorded as `not_held`: stopping a transient timer unloads it, so it could
not be started again. Each other timer is stopped with
`systemctl --user stop <timer>`; restore runs `systemctl --user start
<timer>`. A stopped timer does not fire. On start, a monotonic timer whose
interval elapsed during the run, or a `Persistent=true` calendar timer that
missed its elapse, fires once, after the run. A service a timer activated
that is still running at hold time is recorded (`active_at_hold`) but not
stopped: the preflight's own samples judge whether it disturbs.

### 3.2 Declared services

`${XDG_CONFIG_HOME:-~/.config}/niri-material/capture-hold` lists one unit
per line (`#` comments and blank lines allowed). Each listed unit that is
`active` or `activating` is stopped and started again on restore; a listed
unit that is inactive is recorded as `not_running` and left alone. A missing
file holds no services and is recorded as `config: absent`; a listed unit
that does not exist refuses preflight (CannotRun), since the file names it
on purpose. [Capture host setup](../materials/capture-host-setup.md) records
the reference host's file.

### 3.3 Desktop idle and monitors

The desktop's socket is the inherited `NIRI_SOCKET` when it is set and
nonempty. Otherwise (a TTY login while the desktop still runs, or a clip
fixture that clears `NIRI_SOCKET=` before preflight) it is the one
`niri.*.sock` directly in `$XDG_RUNTIME_DIR` on which `niri msg version`
answers; nested sockets live in per-run subdirectories and are not matched.
More than one live socket refuses preflight (CannotRun) rather than
guessing. When a socket answers, a desktop is up. The hold then records
`noctalia msg status`'s `locked` state, runs `noctalia msg caffeine-enable`
and `niri msg action power-off-monitors`, and records the socket path so
`release`, `restore` and the guard reach the same compositor whatever their
own environment says. After powering off, the hold waits up to 5 s for every connector whose
`/sys/class/drm/card*-*/status` is `connected` to read `Off` in its `dpms`
file, and records those connectors. If any still reads `On`, the kernel is
not reporting niri's atomic CRTC disable on this driver, a wake could not be
seen, and preflight fails (CannotRun) after restoring. Restore runs
`niri msg action power-on-monitors` and `noctalia msg caffeine-disable`. Noctalia has no query for caffeine, so its
prior state is recorded as `unknown` and restore always disables it;
caffeine is not left on by habit on this host.

Without a desktop (the dedicated lane, or a TTY session with the desktop
stopped) this part is recorded as `desktop: absent`. With a desktop but no
`noctalia` on `PATH`, idle is recorded as `not_held` and monitors are still
powered off and watched; nothing watches for idle locks then, so `show`
prints the gap.

### 3.4 Not held

System timers (`man-db`, `plocate-updatedb`, `logrotate`, …) need root to
stop. Their names are recorded (`system_timers`), and §5 flags any that
fire.

## 4. Lifecycle

### 4.1 The host hold file

`$XDG_RUNTIME_DIR/capture-meta.hold.json`, beside the lock:
`{run_id, run_dir, owner_pid, started_us, socket, connectors, items}`. It is the source of truth for
what is held; `capture.json` is the record of it. Each item is appended to
the file *before* its action runs (write-ahead), so a kill at any point
leaves a file naming everything that may have changed. Restoring an item
whose action never ran is harmless: starting a running unit, disabling
caffeine that is off, powering on monitors that are on. Every reader that
would restore or remove it checks that both its `run_id` and its resolved
`run_dir` are the run in hand (`run_id` alone is a directory basename, which
two runs can share), except recovery (§4.4), which runs only after the
owner is dead. Restore and removal of the file happen under the lock's
`guarded()` directory, so a guard and a recovering preflight never act on it
at once; a hold file already gone counts as restored.

### 4.2 Preflight

The order becomes: write `run` → acquire the lock → recover a stale hold
(§4.4) → write the hold file with no items → **start the guard and confirm
it is running** (§4.4) → **hold** → wait `--hold-settle` seconds (default
10) → `environment` → sample → judge. Nothing on the host changes until a
guard is watching, so a kill at any point after the first change is
recovered. Holding before sampling means
the quiet verdict is taken on the held host, the host the run measures; the
settle wait keeps the P-state transition after DPMS-off and the service
stops out of the samples.

The `hold` section is written once the hold is complete:

```json
"hold": {
  "started": "2026-10-04T22:10:03-04:00",
  "started_us": 1791180603000000,
  "items": [
    {"kind": "timer", "unit": "wali-rotate.timer", "action": "stopped", "restore": "systemctl --user start wali-rotate.timer"},
    {"kind": "service", "unit": "dropbox.service", "action": "stopped", "restore": "systemctl --user start dropbox.service"},
    {"kind": "idle", "action": "caffeine-enabled", "prior": "unknown", "locked": false, "restore": "noctalia msg caffeine-disable"},
    {"kind": "monitors", "action": "powered-off", "restore": "niri msg action power-on-monitors"}
  ],
  "not_held": [],
  "socket": "/run/user/1000/niri.wayland-1.571310.sock",
  "config": "~/.config/niri-material/capture-hold",
  "desktop": "present",
  "timer_map": {"user": {"wali-rotate.timer": "wali-rotate.service"}, "system": {"plocate-updatedb.timer": "plocate-updatedb.service"}},
  "system_timers": ["plocate-updatedb.timer"],
  "guard": "capture-meta-guard-<run id>.service",
  "recovered": null
}
```

If the hold fails partway (a `systemctl stop` errors) or preflight fails
after it for any reason (Refused or CannotRun), preflight restores the
hold, writes `hold_end` with verdict `restored-at-preflight`, stops the
guard, removes the hold file and releases the lock. A guard that fails to
start fails preflight (CannotRun) before anything is held. Release then finds
`hold_end` and skips restore and scan.

### 4.3 Release

Release restores from the host hold file whenever the file names this run
(§4.1), whatever sections `capture.json` has. That covers a preflight
process killed after holding but before writing `hold` or `preflight`,
while the fixture shell still lives to run its trap. In that case the
window starts at the hold file's `started_us`, and the lock is released for
the hold file's `owner_pid`, since `preflight.lock` was never written. Then, unless `hold_end` already
exists (a second release, or one after a preflight failure, changes nothing
and exits with the code of the recorded verdict):

1. Restore every item in reverse order, best-effort per item. A unit that
   no longer exists counts as restored, with a note.
2. Scan (§5) the window from `started_us` to the start of step 1, and read
   the guard's wake log.
3. Write `hold_end`, stop the guard, remove the hold file, release the lock.

```json
"hold_end": {
  "restored": "2026-10-04T23:02:41-04:00",
  "failures": [],
  "disturbances": [
    {"at": "2026-10-04T22:23:41-04:00", "manager": "system", "unit": "plocate-updatedb.service", "kind": "timer-fired"}
  ],
  "verdict": "disturbed"
}
```

Verdicts and exit codes: any restore failure keeps the hold file holding the
unrestored items, prints each with its `restore` command, and exits 2
(verdict `restore-failed`). Otherwise `disturbed` exits 1 after everything
is restored and the lock released, `unscanned` (§5) exits 2, and `clean`
exits 0. `capture-meta show` prints the hold and its verdict above the
sub-runs.

### 4.4 Guard and recovery

**Guard.** Before the first hold action, preflight starts a transient user
unit, `systemd-run --user --unit=capture-meta-guard-<run id> --collect
--property=Type=exec`, running `capture-meta guard` by the absolute path of
the `capture-meta` that ran preflight, and confirms with `systemctl --user
is-active` that it runs. The guard loops once a second: it checks that the
lock owner is alive and reads the `dpms` file of each connector the hold
file lists (none until the monitors are held). A connector reading `On`
appends `{at, connector}` to the wake log,
`$XDG_RUNTIME_DIR/capture-meta.wakes.<run id>.jsonl`, once per transition.
When the owner dies, the guard runs the restore below and exits. Its cost is
a `kill(pid, 0)` and a few sysfs reads per second. The guard runs in the user manager's environment, not the
fixture's, so preflight passes `XDG_RUNTIME_DIR`, `WAYLAND_DISPLAY`,
`NIRI_SOCKET` (the recorded desktop socket) and `PATH` with `--setenv`. Normal release stops the guard before restoring, so
the guard never sees the power-on as a wake, and removes the wake log after
recording it. When the fixture is killed outright, its trap never runs; the
guard then restores within a second of the owner's death. The guard is a
service start, not a timer, so §5 does not flag it.

**`capture-meta restore`** (what the guard runs) restores the host hold file, refusing while the
lock's owner is alive. With no hold file it does nothing and exits 0. It
writes `hold_end` (verdict `restored-by-guard` or `restored-by-hand`) into
the file's `run_dir` when that record exists and has none, so the run is
never left looking held. It is also what a person runs if the guard itself
failed.

**Next preflight.** After taking the lock, preflight restores any hold file
it finds (its owner is necessarily dead: a live owner would still hold the
lock) and records `recovered: {run_id, items, failures}` in the new run's
`hold`. A failed recovery refuses preflight with each unrestored item named.

## 5. Disturbance scan

Release reads `journalctl --user -o json` and `journalctl --system -o json`
from `--since` one second before the window, then keeps entries whose
`__REALTIME_TIMESTAMP` (microseconds) falls inside it. This host's user is
in `adm`, which reads the system journal. User-manager entries name their
unit in `USER_UNIT`, system entries in `UNIT`. It flags:

| Kind | Journal evidence |
| --- | --- |
| `timer-fired` | A service activation (below) of a unit some timer activates. The map is the union of the timer→service maps taken at hold time and at release, both managers, plus each timer added in the window mapped to the service of the same name (how `systemd-run` names the pair; a transient timer is unloaded once it elapses, so neither map has it). |
| `timer-added` | A timer started ("Started …", `MESSAGE_ID=39f53479d3a045ac8e11786248231fbf`; timers never log a job start): a timer created or started during the run (`systemd-run --on-active=…`, a package install). |
| `restart` | A scheduled restart (`MESSAGE_ID=5eb03494b6584870a536b337290809b3`) of any unit: a crash-looping service, declared or not. |
| `held-started` | A unit the hold stopped starting again: `39f53479…` for a timer, an activation for a service. |
| `monitor-woke` | An entry in the guard's wake log (§4.4): a held monitor powered on mid-run, by input or anything else. |

A *service activation* is a job start ("Starting …", `7d4958e8…`), or a
unit-started or finished entry (`39f53479…`) or failed start
(`be02cf6855d2428ba40df7e9d022f03d`) not preceded, for the same unit, by a
`7d4958e8…` that no completion has yet closed. systemd skips "Starting" for
a job that completes at once: this host's `Type=simple` `dropbox.service`
and `systemd-run`'s transient services log only "Started". Pairing keeps a
"Starting"→"Started" sequence one activation.

The fixtures' own transient units (`systemd-run --user --unit=… --collect
weston`, per-run clients) start services, are not timer-activated and do
not restart, so they are not flagged. An unreadable journal is a scan
failure: `hold_end.verdict` is `unscanned`, release exits 2 after
restoring, and the run is not evidence until rescanned by hand.

Idle locks have no entry the scan reads; caffeine (§3.3) is the
protection.

## 6. Testing and the first live run

Unit tests in `tools/test_capture_meta.py` drive the hold through an
injected host adapter (systemctl, journal, niri, noctalia and the guard
launch) and cover: timer and service discovery (transient timers not held),
write-ahead ordering, the guard confirmed before the first action and a
guard that fails to start, a kill after each hold action (the guard's
restore leaves the host as found), a hold action failing midway, preflight failure after
the hold, release from the hold file with no `preflight` section, release
skipping restore after `hold_end`, each scan kind against journal entries
with the real `MESSAGE_ID`s per unit type (a "Started"-only service, a
paired "Starting"/"Started" counted once, a oneshot's "Finished"), wake
logging and the DPMS-off check against a fake sysfs tree, the realtime window, a
restore failure keeping the hold file, a vanished unit, recovery by the next
preflight and by `restore` (and both racing under the guard directory), a
hold file whose `run_id` matches but `run_dir` does not, the socket rules (empty `NIRI_SOCKET`, two live
sockets) and `hold`/`hold_end` validation. Fixture tests that check the
cleanup trap cover `|| rc=1`, and a stubbed fixture run checks that
`SHA256SUMS` verifies against the finished `capture.json`.

Live, from a TTY with the desktop stopped (the first quiet run):

1. **Round trip** (about 2 min): `preflight` then `release` on a scratch
   run directory with `--owner-pid` of the calling shell; user timers
   stopped during and started after; `dropbox` held; verdict `clean`.
2. **Positive control** (about 2 min): the same, with
   `systemd-run --user --on-active=30s true` started inside the window;
   release names `timer-added` and `timer-fired` (the transient service
   logs only "Started") and exits 1.
3. **Kill recovery** (about 1 min): preflight from a subshell, then kill
   that subshell; the guard restores within seconds and writes
   `restored-by-guard`.
4. **Evidence run:** the dedicated-lane development check
   (`optic-settling-smoke.sh pilot`, three cases; "Running the dedicated
   lane" in capture host setup) with the hold in place.

The desktop part (§3.3), including the DPMS-off check on this NVIDIA
driver, a wake caught by moving the mouse, and the guard restoring caffeine
and monitors after a killed desktop run, is exercised the next time a
headless-lane run is taken from the desktop; until then it is tested only
through the adapter.
