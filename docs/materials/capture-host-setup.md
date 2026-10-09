# Capture host setup

What a host needs before it can run this repository's capture lanes: the
headless lane (nested niri under headless Weston) and the dedicated real-TTY
lane ([design](../specs/2026-10-02-real-tty-settling-lane-design.md)). The
commands are for Arch Linux; every item is an owner action, not something a
capture script installs or grants. The drivers check these and refuse with
the missing item named.

Recorded on the reference host on 2026-10-02 (NVIDIA RTX 3070,
`nvidia-open-dkms`; one Gigabyte G34WQC A on DP-1).

## Packages

```sh
# Headless lane: host compositor, clients, stimuli, images, JSON.
sudo pacman -S weston kitty grim swaybg imagemagick jq attr iproute2
# Building per-run clients (idle inhibitor, session lock) from protocol XML.
sudo pacman -S base-devel wayland wayland-protocols
# Rebuilding the retained Tracy 0.13.1 capture tools when their hashes
# no longer match (material-signals-smoke.sh tools_ready does it).
sudo pacman -S git cmake
# Real-TTY lane: the screencast consumer and monitor dimming.
sudo pacman -S gst-plugin-pipewire python-gobject ddcutil
```

`wlrctl` (the virtual-pointer stimulus) is in the AUR, not the official
repositories:

```sh
paru -S wlrctl        # or any AUR helper / makepkg
```

`host-budget`, `host-load` and `display-dim` come from the ops repository's
`bin/`, linked onto `~/.local/bin` by its install recipe. `tools/capture-meta` ships with this repository.

## Passwordless VT switching (real-TTY lane)

The lane switches to a spare VT and back to exercise session resume. Allow
exactly `chvt`, nothing else. Check the rule before installing it: a
malformed file in `/etc/sudoers.d` breaks `sudo` itself.

```sh
echo "$USER ALL=(root) NOPASSWD: /usr/bin/chvt" > /tmp/50-chvt
sudo visudo -cf /tmp/50-chvt && sudo install -m 0440 -o root -g root /tmp/50-chvt /etc/sudoers.d/50-chvt
rm /tmp/50-chvt
```

Check without switching (a real `chvt` moves the display away):

```sh
sudo -k; sudo -n -l /usr/bin/chvt     # prints /usr/bin/chvt, no password prompt
```

## Monitor brightness over DDC/CI (overnight dimming)

`ddcutil` installs a udev rule (`60-ddcutil-i2c.rules`) that grants the
active local session, including a TTY login, access to the display's I²C
buses. Existing `/dev/i2c-*` nodes need the rule applied once after install:

```sh
sudo udevadm control --reload
sudo udevadm trigger --subsystem-match=i2c-dev
getfacl -p /dev/i2c-* 2>/dev/null | grep 'user:'"$USER"     # rw- on each bus
ddcutil detect                                             # lists the monitor
ddcutil getvcp 10                                          # brightness and its max
```

If `ddcutil detect` finds nothing, compare `sudo ddcutil detect`: success
there means permissions; failure there too means the monitor or driver does
not answer DDC, and overnight runs simply stay undimmed. The dimming helper
itself is ops's `display-dim` (`ops-a1715a`).

## Other host state the lanes assume

- `.cargo/config.toml` and `target/` are per-machine (see `AGENTS.md`).
- The DRM power lane in niri-experiments' idle-budget fixture also needs a
  NOPASSWD rule for its exact `fuser -v` GPU device inventory; its fixture
  documents the command list.
- Runs that need an idle host start from a TTY login with the desktop
  stopped (`tasks quiet`, `--needs headless`).
- The pixels lane (`docs/specs/2026-10-08-capture-host-conditions-design.md`)
  needs no idle host and no `nvidia-smi`; it holds user timers and the
  declared services only.

## Disturber hold

`capture-meta preflight` holds the host's disturbers for the whole run
([design](../specs/2026-10-04-capture-disturber-hold-design.md)): every
active user timer, the services named in
`${XDG_CONFIG_HOME:-~/.config}/niri-material/capture-hold`, and on the
desktop Noctalia's caffeine and monitor power. `release` restores them and
scans the journal; a disturbed run fails its fixture.

Restore is not schedule-preserving: `systemctl --user start` re-anchors
each timer's `OnActiveSec` trigger at the restore instant (and an
`OnBootSec` trigger whose boot offset has already elapsed fires at once),
so a held run can add off-schedule timer runs after release, even when
nothing elapsed during the run. Observed 2026-10-05: `familiar-reap` fired
at restore and `atoms-recertify` moved its next run five minutes out. The
extra fires land outside the capture window, so no run is disturbed; they
are accepted, because systemd exposes no writable next-elapse for a stopped
timer and no faithful restore exists. `OnUnitActiveSec`, `OnUnitInactiveSec`
and `OnCalendar` triggers keep their schedule across restore, except that a
`Persistent=true` calendar timer that missed its elapse during the hold
fires once after release.

The reference host's file:

```text
# Crash-loops every ~14 s while the desktop is stopped (DISPLAY=:0 override).
dropbox.service
```

After a killed run the guard unit (`capture-meta-guard-*.service`) restores
within seconds. If it could not, `tools/capture-meta restore` restores by
hand; `$XDG_RUNTIME_DIR/capture-meta.hold.json` lists what is still held
under `held`, each item with its restore command.

## Running the dedicated lane

From a TTY login with the desktop stopped, in the execution worktree, with
an identified Tracy snapshot (`niri-tracy` plus its `.identity.json`). The
example is the development check: a `pilot` limited by `CASES` to three
cases, which never passes as a pilot.

```sh
OUT=$NIRI_MATERIAL_WORK_ROOT/optic-settling/tty-dev-$(date +%Y%m%d)-1 \
  CASES='drm-aurora tty-resume screencast' CAPTURE_TASK=material-f7eb0b \
  NIRI_BIN=<snapshot>/niri-tracy DRM_OUTPUT=DP-1 DRM_MODE=3440x1440@59.999 \
  docs/materials/scripts/optic-settling-smoke.sh pilot --lane dedicated
```

The pilot is the same command without `CASES` and with a fresh `OUT`
(for example `.../tty-pilot-$(date +%Y%m%d)-1`). The matrix runs `matrix`
instead of `pilot`, again with a fresh `OUT`, and adds
`PILOT_DIR=<the pilot's OUT>`.
