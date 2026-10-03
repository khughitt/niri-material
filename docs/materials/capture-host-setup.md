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

`host-budget` and `host-load` come from the ops repository's `bin/`, on
`~/.local/bin`. `tools/capture-meta` ships with this repository.

## Passwordless VT switching (real-TTY lane)

The lane switches to a spare VT and back to exercise session resume. Allow
exactly `chvt`, nothing else:

```sh
echo "$USER ALL=(root) NOPASSWD: /usr/bin/chvt" | sudo tee /etc/sudoers.d/50-chvt
sudo chmod 0440 /etc/sudoers.d/50-chvt
sudo visudo -cf /etc/sudoers.d/50-chvt
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

## Running the dedicated lane

From a TTY login with the desktop stopped, in the execution worktree, with
an identified Tracy snapshot (`niri-tracy` plus its `.identity.json`):

```sh
OUT=$NIRI_MATERIAL_WORK_ROOT/optic-settling/tty-dev-$(date +%Y%m%d)-1 \
  CASES='drm-aurora tty-resume screencast' CAPTURE_TASK=material-f7eb0b \
  NIRI_BIN=<snapshot>/niri-tracy DRM_OUTPUT=DP-1 DRM_MODE=3440x1440@59.999 \
  docs/materials/scripts/optic-settling-smoke.sh pilot --lane dedicated
```

Drop `CASES` for the pilot; the matrix adds `PILOT_DIR` and uses `matrix`.
