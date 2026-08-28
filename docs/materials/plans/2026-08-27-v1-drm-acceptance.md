# Native Materials v1 Physical DRM Acceptance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Status:** executed and integrated 2026-08-28. The run recorded at
`niri-experiments` commit `dbb277557f454b803af15e7dcd933a97aeea2e7b`
was invalidated by audit commit
`f10432f876c37576d8f27b1dad0fba624f6d445f`: kitty clients inherited
operator configuration, violating the opaque-control precondition. Physical
observations 1–9 passed. Native materials v1 remains unaccepted pending an
isolated rerun with corrective fixture commit
`901b5a41e7e18f2a3fe73d5d630342fa21e6a45e`. Task 7 review, recoverable
external-artifact cleanup, and branch integration are complete.

**Goal:** Produce a controlled, auditable physical-DRM smoke result for the
frozen native-materials v1 candidate and reconcile the v1 acceptance status
with that evidence.

**Architecture:** Build the exact accepted candidate into a dedicated SSD
target, then run one purpose-built POSIX-shell fixture from an active VT2
login on the real RTX 3070/DP-1 DRM path. The fixture combines deterministic
IPC actions and image/log gates with a separately recorded human observation
of the monitor; it never edits the personal session or generalizes into a DRM
test framework.

**Tech Stack:** POSIX shell, niri IPC, KDL, kitty, swaybg, ImageMagick, `jq`,
`sha256sum`, logind, sysfs DRM facts, Cargo/Rust, Git worktrees.

**Spec:** `docs/materials/2026-08-27-v1-drm-acceptance-design.md`

## Global Constraints

- Candidate source is exactly
  `138697be4cbb779c80425fe2a366ceca3610f38e`; no production source change is
  allowed in this work.
- Evidence starts from `niri-experiments` branch `results/slice3` at
  `c4b71a4ebfbe3c82c56f964bfc24d4f7de1bde4f`.
- The physical run requires active VT2 on `seat0`, PCI device `0000:09:00.0`
  (`10de:2484`, NVIDIA RTX 3070), and `card1-DP-1` connected.
- After startup, niri must report only `DP-1` at 3440×1440@59.999 Hz
  (nominal 60 Hz), scale 1, normal transform, VRR off.
- `NIRI_MATERIAL_WORK_ROOT` is required and must resolve exactly to
  `/mnt/ssd3/niri-material`; there is no `/tmp` or alternate-device fallback.
- Start plain niri, never `niri --session`; VT1 remains the recovery session.
- Raw captures, logs, runtime state, handoff files, and Cargo outputs stay
  outside Dropbox and remain untracked.
- The personal niri config, VT1 sockets, and user-systemd graphical targets
  are read-only and must not be modified.
- Integrity failures are not interpreted. Valid machine or physical failures
  block v1 and are recorded without adding a production fix to the acceptance
  branches.
- The machine verdict is incomplete until the operator records every physical
  observation row explicitly.
- No new dependency, broad warning allowlist, performance benchmark, parity
  rerun, VRR/HDR test, or post-v1 feature is in scope.

---

## File structure

| File | Responsibility | Task |
| --- | --- | --- |
| `niri-experiments/fixtures/v1-drm-smoke.kdl` | Exact current-schema physical scene and recovery binding | 1 |
| `niri-experiments/fixtures/v1-drm-smoke.sh` | Prepare/run/analyze/self-test entry point, owned lifecycle, and machine gates | 2–3 |
| `niri-experiments/docs/results/2026-08-27-v1-drm-acceptance.md` | Actual environment, machine evidence, physical observations, and verdict | 6 |
| `docs/materials/2026-08-27-v1-drm-acceptance-design.md` | Implemented status and evidence pin | 6 |
| `docs/materials/plans/2026-08-27-v1-drm-acceptance.md` | Executed status and checked steps supported by evidence | 6 |
| Existing material status surfaces named in Task 6 | Propagated v1 verdict | 6 |

The shell fixture remains one file because its preflight, owned-process
lifecycle, capture sequence, and analyzer share one small state contract and
are never reused independently. A second shell library or general scenario
format would add an interface with no second consumer.

---

### Task 1: Create the evidence branch and current-schema scene

**Files:**
- Create worktree: `niri-experiments/.worktrees/results-v1-drm-acceptance`
- Create: `niri-experiments/fixtures/v1-drm-smoke.kdl`

**Interfaces:**
- Consumes: candidate config grammar at `138697be`; committed
  `fixtures/diagnostic-grid.png` from evidence base `c4b71a4`.
- Produces: a KDL file accepted by the candidate binary; app IDs
  `v1-drm-probe` and `v1-drm-control`; exit bind `Mod+Shift+E`.

- [x] **Step 1: Verify both source trees and create the evidence worktree**

Run from the material design worktree:

```sh
test "$(git rev-parse 138697be)" = \
    138697be4cbb779c80425fe2a366ceca3610f38e
test -z "$(git diff --name-only b8fe7b84..138697be -- . ':(exclude)docs')"

experiments_repo=$(git -C ../../../niri-experiments rev-parse --show-toplevel)
test "$(git -C "$experiments_repo" rev-parse results/slice3)" = \
    c4b71a4ebfbe3c82c56f964bfc24d4f7de1bde4f
test -z "$(git -C "$experiments_repo/.worktrees/results-slice3" status --short)"
test ! -e "$experiments_repo/.worktrees/results-v1-drm-acceptance"
test -z "$(git -C "$experiments_repo" branch --list results/v1-drm-acceptance)"
git -C "$experiments_repo" worktree add \
    "$experiments_repo/.worktrees/results-v1-drm-acceptance" \
    -b results/v1-drm-acceptance results/slice3
```

Expected: both pin checks pass; the new worktree is clean at `c4b71a4`.

- [x] **Step 2: Write the exact physical scene**

Create `fixtures/v1-drm-smoke.kdl` in the evidence worktree with:

```kdl
output "DP-1" {
    mode "3440x1440@59.999"
    scale 1
    transform "normal"
    focus-at-startup
}

prefer-no-csd

layout {
    gaps 40
    background-color "#243447"
    default-column-width { proportion 0.4; }
    focus-ring { off; }
    border { off; }
    shadow { off; }
}

overview { backdrop-color "#503050"; }
hotkey-overlay { skip-at-startup; }
config-notification { disable-failed; }

layer-rule {
    match namespace="^wallpaper$"
    place-within-backdrop true
}

animations {
    workspace-switch {
        spring damping-ratio=1.0 stiffness=400 epsilon=0.0001
    }
    window-movement {
        spring damping-ratio=1.0 stiffness=100 epsilon=0.0001
    }
    window-resize {
        spring damping-ratio=1.0 stiffness=800 epsilon=0.0001
    }
}

material "frost" {
    glass {
        ior 1.5
        thickness 20
        attenuation-color "#dfe8ff"
        attenuation-distance 60
        chromatic-aberration 0.1
        distortion 0.15 scale=0.5
        anisotropic-blur 0.2
        jelly-flex 0.004
        jelly-ripple 0.06
        bevel 12
        offset-x 6
        offset-y 6
    }
}

window-rule {
    match app-id="^v1-drm-probe$"
    material "frost"
    geometry-corner-radius 16
}

window-rule {
    match app-id="^v1-drm-control$"
    open-focused false
}

binds {
    Mod+Shift+E { quit skip-confirmation=true; }
}
```

The failed-config overlay is disabled so the invalid-reload screenshot can
measure retained appearance rather than the expected notification.
Omitting `variable-refresh-rate` keeps VRR off.

- [x] **Step 3: Validate the scene and its retained diagnostic input**

Run with any binary built from the accepted production tree:

```sh
candidate_debug=/mnt/ssd3/niri-material/target/debug/niri
"$candidate_debug" validate --config fixtures/v1-drm-smoke.kdl
magick identify -format '%wx%h\n' fixtures/diagnostic-grid.png
sha256sum fixtures/diagnostic-grid.png
```

Expected: config valid; dimensions `40x40`; SHA-256
`6fafae8c6cf3e3815346128ffdb402d5c730ae3a8749cb060013821ba0fe0316`.

- [x] **Step 4: Commit the scene**

```sh
git add fixtures/v1-drm-smoke.kdl
git diff --cached --check
git commit -m "test(fixtures): define physical DRM scene"
```

---

### Task 2: Implement fail-closed preparation and lifecycle checks

**Files:**
- Create: `niri-experiments/fixtures/v1-drm-smoke.sh`

**Interfaces:**
- CLI:
  - `v1-drm-smoke.sh --self-test NIRI_BINARY`
  - `v1-drm-smoke.sh --prepare NIRI_BINARY ARTIFACT_DIR HANDOFF_JSON`
  - `v1-drm-smoke.sh --run HANDOFF_JSON`
  - `v1-drm-smoke.sh --analyze HANDOFF_JSON`
- Produces from `--prepare`: exact-schema handoff JSON pinning absolute paths,
  commits, hashes, version, and artifact directory, plus the exact external
  VT2 wrapper.
- Produces from `--run`: owned runtime state, captures, logs, state JSON, and
  an eventual machine summary through the Task 3 analyzer.

- [x] **Step 1: Write the failing integrated self-test first**

Start `fixtures/v1-drm-smoke.sh` with `set -eu`, the four-mode `usage`, and a
`self_test` function that calls not-yet-defined pure helpers. The first test
body must cover:

```sh
self_test() {
    st_binary=$1
    st_root=${NIRI_MATERIAL_WORK_ROOT:?NIRI_MATERIAL_WORK_ROOT is required}
    st_dir=$(mktemp -d "$st_root/v1-drm-self-test.XXXXXX")
    trap 'rm -rf "$st_dir"' EXIT HUP INT TERM

    validate_session_facts /dev/tty2 tty yes 2 seat0
    ! validate_session_facts /dev/pts/0 wayland yes 1 seat0
    ! validate_session_facts /dev/tty2 tty no 2 seat0
    ! validate_session_facts /dev/tty2 tty yes 2 seat1

    make_test_handoff "$st_binary" "$st_dir/handoff.json" "$st_dir/run"
    validate_handoff "$st_dir/handoff.json"
    cp "$st_dir/handoff.json" "$st_dir/tampered.json"
    jq '.pins.binary_sha256 = ("0" * 64)' "$st_dir/tampered.json" \
        >"$st_dir/tampered.next"
    mv "$st_dir/tampered.next" "$st_dir/tampered.json"
    ! validate_handoff "$st_dir/tampered.json"

    sleep 600 & st_child=$!
    stop_owned_children "$st_child"
    ! kill -0 "$st_child" 2>/dev/null
}
```

Dispatch `--self-test` to this function. Run:

```sh
env NIRI_MATERIAL_WORK_ROOT=/mnt/ssd3/niri-material \
    sh fixtures/v1-drm-smoke.sh --self-test \
    /mnt/ssd3/niri-material/target/debug/niri
```

Expected: FAIL because `validate_session_facts`, `make_test_handoff`,
`validate_handoff`, and `stop_owned_children` do not exist yet.

- [x] **Step 2: Implement exact root, path, session, and hardware guards**

Add helpers with these contracts:

```sh
require_work_root() {
    [ "${NIRI_MATERIAL_WORK_ROOT-}" = /mnt/ssd3/niri-material ] || {
        echo "NIRI_MATERIAL_WORK_ROOT must be /mnt/ssd3/niri-material" >&2
        return 2
    }
    [ -d "$NIRI_MATERIAL_WORK_ROOT" ] \
        && [ -w "$NIRI_MATERIAL_WORK_ROOT" ] || return 2
    work_root=$NIRI_MATERIAL_WORK_ROOT
}

validate_session_facts() {
    [ "$1" = /dev/tty2 ] && [ "$2" = tty ] && [ "$3" = yes ] \
        && [ "$4" = 2 ] && [ "$5" = seat0 ]
}

require_vt2() {
    session_id=${XDG_SESSION_ID:?XDG_SESSION_ID is required on VT2}
    validate_session_facts "$(tty)" \
        "$(loginctl show-session "$session_id" -p Type --value)" \
        "$(loginctl show-session "$session_id" -p Active --value)" \
        "$(loginctl show-session "$session_id" -p VTNr --value)" \
        "$(loginctl show-session "$session_id" -p Seat --value)" || {
        echo "physical DRM run requires active seat0 VT2" >&2
        return 2
    }
}

require_hardware() {
    [ "$(cat /sys/class/drm/card1-DP-1/status)" = connected ] || return 2
    [ "$(basename "$(readlink -f /sys/class/drm/card1/device)")" = \
        0000:09:00.0 ] || return 2
    lspci -Dnns 0000:09:00.0 | grep -Fq '[10de:2484]' || return 2
}
```

Also require `jq`, `magick`, `kitty`, `swaybg`, `sha256sum`, `loginctl`,
`lspci`, `cmp`, `awk`, and `sed` before creating state. Validate every supplied
path as absolute. Accept artifact directories only under
`$work_root/v1-drm-acceptance.*` and the handoff only under `$work_root`.

- [x] **Step 3: Implement exact-schema handoff preparation and validation**

`--prepare` must refuse a dirty fixture worktree or nonempty artifact
directory, validate the committed config, and write schema 1 from values
computed in the same process:

```sh
test -z "$(git -C "$repository_root" status --short)" || {
    echo "fixture worktree must be clean" >&2
    return 2
}
fixture_commit=$(git -C "$repository_root" rev-parse HEAD)
candidate_version=$("$binary" --version)
[ "$candidate_version" = 'niri 26.04 (v26.04-95-g138697be)' ] || return 2
binary_sha256=$(sha256sum "$binary" | awk '{print $1}')
script_sha256=$(sha256sum "$script_path" | awk '{print $1}')
config_sha256=$(sha256sum "$config_path" | awk '{print $1}')

jq -n \
    --arg artifact "$artifact_dir" --arg binary "$binary" \
    --arg version "$candidate_version" --arg binary_hash "$binary_sha256" \
    --arg fixture "$fixture_commit" --arg script_hash "$script_sha256" \
    --arg config_hash "$config_sha256" '
    {
      schema: 1,
      artifact_dir: $artifact,
      binary: $binary,
      pins: {
        material_source_commit:
          "138697be4cbb779c80425fe2a366ceca3610f38e",
        candidate_version: $version,
        binary_sha256: $binary_hash,
        evidence_base_commit:
          "c4b71a4ebfbe3c82c56f964bfc24d4f7de1bde4f",
        evidence_fixture_commit: $fixture,
        script_sha256: $script_hash,
        config_sha256: $config_hash,
        diagnostic_sha256:
          "6fafae8c6cf3e3815346128ffdb402d5c730ae3a8749cb060013821ba0fe0316"
      }
    }' >"$handoff.next"
mv "$handoff.next" "$handoff"
```

Use `jq -n` to write the JSON and `jq -e` to require exactly these keys on
read. `validate_handoff` recomputes every file hash, checks the binary's
current version against the recorded version, verifies that the evidence base
is an ancestor of the clean fixture commit, and rejects symlinks for the
binary, config, script, artifact directory, and handoff. `--prepare` alone
also requires the literal accepted candidate version shown above. It must
never trust a hash merely because it appears in the same handoff.

After writing the handoff, `--prepare` writes
`$work_root/run-v1-drm-acceptance.sh` from the already resolved absolute
`$script_path` and `$handoff` values, mode 700. Its entire payload is
`set -eu`, the exact work-root export, and `exec "$script_path" --run
"$handoff"`; no path is edited by hand later. It also writes
`$artifact_dir/prepare.sha256` over the binary, script, config, diagnostic
tile, handoff, and wrapper so Task 4 can recheck the whole handoff before VT2.

The script hash is computed before writing the handoff; the fixture worktree
must be clean so the recorded evidence commit identifies those bytes.

- [x] **Step 4: Implement owned cleanup and atomic run locking**

Use an atomic directory lock at
`$work_root/v1-drm-acceptance.lock`, storing the runner PID inside it. Cleanup
may remove it only when that PID equals `$$`.

Implement `stop_owned_children` as TERM, bounded polling, then KILL only for
the exact recorded PIDs. Cleanup removes only a runtime directory matching
`$work_root/v1-drm-runtime.*`; it never recursively removes the artifact
directory. Traps must preserve the original run status:

```sh
finish() {
    finish_status=$?
    trap - EXIT HUP INT TERM
    stop_owned_children "$done_pid" "$probe_pid" "$control_pid" \
        "$old_probe_pid" "$swaybg_pid" "$niri_pid" || finish_status=2
    remove_owned_runtime || finish_status=2
    release_owned_lock || finish_status=2
    exit "$finish_status"
}
trap finish EXIT HUP INT TERM
```

Initialize every PID variable, including `old_probe_pid`, to empty before
installing the trap. After close, retain the old PID for cleanup and wait for
its normal exit; cleanup still handles it if kitty remains alive.

- [x] **Step 5: Make the preflight self-test pass**

Complete `make_test_handoff` using a regular copy of the supplied binary in
the self-test directory, so symlink rejection is exercised independently of
the real candidate. Add tests for a wrong config hash, extra JSON key,
relative artifact path, symlinked binary, and nonempty artifact directory.

Run:

```sh
env NIRI_MATERIAL_WORK_ROOT=/mnt/ssd3/niri-material \
    sh fixtures/v1-drm-smoke.sh --self-test \
    /mnt/ssd3/niri-material/target/debug/niri
sh -n fixtures/v1-drm-smoke.sh
```

Expected: every self-test prints `PASS`; shell syntax exits 0; the self-test
directory and child process are gone.

---

### Task 3: Implement the physical sequence and machine analyzer

**Files:**
- Modify: `niri-experiments/fixtures/v1-drm-smoke.sh`

**Interfaces:**
- Consumes: validated handoff from Task 2 and scene from Task 1.
- Produces: `captures.sha256`, `action-trace.tsv`, state JSON files,
  `niri.log`, client logs, environment JSON, and exact-schema
  `machine-summary.json` with verdict `pass` or `fail`.
- Remap gate: new probe ID plus byte identity between
  `captures/remap-settled.png` and `captures/initial-a.png`.

- [x] **Step 1: Add failing analyzer controls to `--self-test`**

Create a synthetic 320×180 artifact set with ImageMagick and record those
dimensions in its `environment.json`. Use a static tiled background for
`initial-a`, copy it to every return-state file, create a changed rectangle
for `reload-valid`, copy that to `reload-invalid`, and draw distinct
motion/resize/overview/workspace frames. Write state JSON with probe ID 10
initially, zero probes after close, and probe ID 20 after remap.

Then require:

```sh
analyze_artifacts "$st_dir/pass-handoff.json" 320 180
[ "$(jq -r .verdict "$st_dir/pass/machine-summary.json")" = pass ]

cp "$st_dir/pass/captures/remap-settled.png" \
    "$st_dir/remap-settled.saved.png"
magick "$st_dir/pass/captures/remap-settled.png" \
    -fill red -draw 'rectangle 1,1 8,8' \
    "$st_dir/pass/captures/remap-settled.png"
write_test_inventory "$st_dir/pass"
! analyze_artifacts "$st_dir/pass-handoff.json" 320 180
[ "$(jq -r '.gates[] | select(.name == "remap-return") | .status' \
    "$st_dir/pass/machine-summary.json")" = fail ]
mv "$st_dir/remap-settled.saved.png" \
    "$st_dir/pass/captures/remap-settled.png"
write_test_inventory "$st_dir/pass"

printf '%s\n' 'ERROR renderer failed' >>"$st_dir/pass/niri.log"
! analyze_artifacts "$st_dir/pass-handoff.json" 320 180
```

Expected before implementation: FAIL because `analyze_artifacts` is missing.

- [x] **Step 2: Implement compositor startup, output validation, and clients**

Create fresh mode-700 runtime/config/cache directories. Copy the committed KDL
to `runtime-default.kdl` and `runtime-active.kdl`. Start plain niri with an
explicit environment and capture its log:

```sh
env -i HOME="$HOME" PATH="$PATH" \
    XDG_RUNTIME_DIR="$runtime_dir" \
    XDG_CONFIG_HOME="$config_home" \
    XDG_CACHE_HOME="$cache_home" \
    DBUS_SESSION_BUS_ADDRESS="${DBUS_SESSION_BUS_ADDRESS-}" \
    RUST_LOG=niri=debug \
    "$binary" --config "$runtime_active" >"$artifact_dir/niri.log" 2>&1 &
niri_pid=$!
```

Poll for exactly one Wayland socket and one niri IPC socket while proving the
PID remains alive. Query outputs and require this exact predicate:

```jq
keys == ["DP-1"]
and .["DP-1"].logical == {
  x: 0, y: 0, width: 3440, height: 1440,
  scale: 1.0, transform: "Normal"
}
and .["DP-1"].vrr_enabled == false
and (.["DP-1"].current_mode as $i
     | .["DP-1"].modes[$i]
     | .width == 3440 and .height == 1440
       and .refresh_rate == 59999)
```

Copy the committed 40×40 PNG to `grid-tile.png` and start
`swaybg -m tile -i grid-tile.png` on the private Wayland socket. Both output
dimensions are exact multiples of 40, so no generated full-screen wallpaper
is needed.

Before mapping clients, write `environment.json` from `uname`,
`/sys/module/nvidia/version`, `lspci`, logind session properties, the DRM
connector status, and niri's exact output JSON. The analyzer rejects missing
fields rather than filling defaults.

Spawn the control first and probe second so closing/remapping the right-hand
probe restores the original layout. Start both kitty clients with
`--config NONE`, then give both clients
`-o background='#101820'`; give the control `-o background_opacity=1` and the
probe `-o background_opacity=0.10`, matching the proven slice-2 physical
fixture. Both kitty commands must include `-o cursor_blink_interval=0`, hide
the cursor with `\033[?25l`, render only a fixed opaque color marker plus
otherwise static content, and end in `exec sleep 3600`. Poll IPC until exactly
one window of each app ID exists; focus the probe by ID.

- [x] **Step 3: Implement capture, reload, motion, and return helpers**

Add these helpers:

```text
niri_msg ACTION...                 run IPC against the owned socket
record_state TAG                   write outputs/windows/workspaces-TAG.json
capture TAG                        screenshot to captures/TAG.png and wait
capture_pair TAG                   capture TAG-a.png and TAG-b.png
wait_probe_count COUNT TAG         poll exact app-ID count and record state
assert_probe_id EXPECTED TAG       require the one probe ID
load_config PATH TAG               record log byte offsets around reload
capture_burst PREFIX ACTION...     capture fixed 0/50/100/200/400/800 ms burst
```

`capture` uses:

```sh
env NIRI_SOCKET="$niri_socket" "$binary" msg action screenshot-screen \
    --write-to-disk true --show-pointer false --path "$capture_path"
```

Poll until the PNG is nonempty and `magick identify` reports 3440×1440.
Append its relative path and SHA-256 to `captures.sha256`; refuse duplicate
physical paths.

Implement the approved order exactly:

1. settle two seconds; `capture_pair initial`;
2. create `runtime-valid.kdl` by changing only `thickness 20` to
   `thickness 80` and `#dfe8ff` to `#ffc080`; validate, load, retain probe ID,
   settle, capture `reload-valid`;
3. create `runtime-invalid.kdl` from the valid file by changing only
   `offset-x 6` to `offset-x 13`; require standalone validation to fail with
   `offset must not exceed bevel`, load it, retain probe ID, settle, capture
   `reload-invalid`, then restore and capture `reload-restored`;
4. issue `move-column-left` then `move-column-right` three times, capturing
   one leftward burst and `move-settled-a/b` after return;
5. issue `set-column-width 70%`, capture the resize burst, restore
   `set-column-width 40%`, and capture `resize-settled`;
6. record the original probe ID, issue `close-window`, require zero probe
   windows, spawn a new probe, require exactly one distinct ID, settle, and
   capture `remap-settled`;
7. `capture_burst overview-opening toggle-overview`, settle and capture
   `overview-open`, toggle closed, and capture `overview-return`;
8. `focus-workspace-down`, capture a timed burst, settle, then
   `focus-workspace-up`, capture the return burst and `workspace-return`;
9. capture `final-output` and `final-settled`;
10. map an ungraded `v1-drm-done` kitty displaying
    `Sequence complete — inspect the display, then press Mod+Shift+E`, and
    wait for the owned niri process to exit.

Record requested and actual offsets in `action-trace.tsv`. A failed action,
missing capture, changed same-name reload ID, unchanged remap ID, or early
operator exit is integrity failure exit 2.

- [x] **Step 4: Implement exact machine gates**

`analyze_artifacts HANDOFF EXPECTED_WIDTH EXPECTED_HEIGHT` must validate all
paths and hashes before interpreting images. Production `--run` and
`--analyze` always pass `3440 1440` and reject an `environment.json` that
disagrees; only the synthetic self-test passes `320 180`. It then writes every
gate result, continuing after ordinary gate failures so the summary is
complete. Integrity errors exit 2; a complete failed machine matrix exits 1;
all machine gates exit 0.

Use byte comparison for all exact return gates:

```sh
cmp -s captures/initial-a.png captures/initial-b.png
cmp -s captures/reload-valid.png captures/reload-invalid.png
cmp -s captures/initial-a.png captures/reload-restored.png
cmp -s captures/initial-a.png captures/move-settled-a.png
cmp -s captures/move-settled-a.png captures/move-settled-b.png
cmp -s captures/initial-a.png captures/resize-settled.png
cmp -s captures/initial-a.png captures/remap-settled.png
cmp -s captures/initial-a.png captures/overview-return.png
cmp -s captures/initial-a.png captures/workspace-return.png
cmp -s captures/initial-a.png captures/final-settled.png
```

The seventh comparison is the concrete remap-treatment gate: the remapped
probe must have a new ID, and the whole settled scene must be byte-identical
to step 1. No color heuristic is used.

Require `reload-valid` to differ from `initial-a`; every motion/resize burst
must contain at least one frame different from rest. At least one frame in the
overview-opening burst and in each workspace-switch burst must contain both
the fixed opaque probe marker and the expected `#503050` backdrop color; the
0 ms and settled frames need not. Record ImageMagick AE values for all
nonidentity comparisons without inventing a magnitude threshold.

Exclude only the bounded log byte range produced by the deliberate invalid
reload from the general log scan, and separately require that range to contain
`offset must not exceed bevel`. Outside that range, fail on any warning/error
line until an exact line is reviewed and explicitly documented; always fail
on panic, shader compilation/fallback, renderer, DRM, modeset, or page-flip
errors. This is not a regex-wide allowlist.

Write `machine-summary.json` with exact keys `schema`, `verdict`, `pins`,
`environment`, `inventory`, and `gates`. Every gate has exact keys `name`,
`status`, and `evidence`.

- [x] **Step 5: Make analyzer controls and the full self-test pass**

Run:

```sh
env NIRI_MATERIAL_WORK_ROOT=/mnt/ssd3/niri-material \
    sh fixtures/v1-drm-smoke.sh --self-test \
    /mnt/ssd3/niri-material/target/debug/niri
sh -n fixtures/v1-drm-smoke.sh
```

Expected: the passing synthetic artifact returns 0; the mutated remap returns
1 with only `remap-return` failed; the injected renderer error returns 1;
tampered handoffs return 2; all self-test work and child processes are gone.

- [x] **Step 6: Commit the fixture**

```sh
git add fixtures/v1-drm-smoke.sh
git diff --cached --check
git commit -m "test(fixtures): add physical DRM smoke"
```

---

### Task 4: Build and pin the exact candidate, then prepare VT2 handoff

**Files:**
- Create temporary detached worktree:
  `niri-material/.worktrees/v1-drm-candidate`
- Create outside Dropbox: dedicated Cargo target, unique artifact directory,
  `v1-drm-acceptance-handoff.json`, and `run-v1-drm-acceptance.sh`

**Interfaces:**
- Consumes: clean candidate pin and committed evidence fixture.
- Produces: validated release binary, immutable handoff JSON, and a one-command
  VT2 wrapper.

- [x] **Step 1: Create the detached candidate worktree and dedicated target**

From the main material checkout:

```sh
test ! -e .worktrees/v1-drm-candidate
git worktree add --detach .worktrees/v1-drm-candidate 138697be
test -z "$(git -C .worktrees/v1-drm-candidate status --short)"
candidate_target=/mnt/ssd3/niri-material/targets/v1-drm-acceptance
test ! -e "$candidate_target"
env CARGO_TARGET_DIR="$candidate_target" \
    cargo build --release --locked \
    --manifest-path .worktrees/v1-drm-candidate/Cargo.toml
```

Expected binary:
`/mnt/ssd3/niri-material/targets/v1-drm-acceptance/release/niri`.

- [x] **Step 2: Verify the binary and rerun the complete source tests**

```sh
candidate_binary=/mnt/ssd3/niri-material/targets/v1-drm-acceptance/release/niri
test "$("$candidate_binary" --version)" = \
    'niri 26.04 (v26.04-95-g138697be)'
sha256sum "$candidate_binary"
env CARGO_TARGET_DIR="$candidate_target" \
    cargo test --workspace --all-targets --locked \
    --manifest-path .worktrees/v1-drm-candidate/Cargo.toml
```

Expected: the same 293-test baseline is green; record the actual binary hash.

- [x] **Step 3: Run fixture self-tests against the release candidate**

From the evidence worktree:

```sh
env NIRI_MATERIAL_WORK_ROOT=/mnt/ssd3/niri-material \
    sh fixtures/v1-drm-smoke.sh --self-test "$candidate_binary"
```

Expected: all self-tests pass and leave no `v1-drm-self-test.*` directory.

- [x] **Step 4: Prepare one unique run and prove non-VT refusal**

```sh
export NIRI_MATERIAL_WORK_ROOT=/mnt/ssd3/niri-material
artifact_dir=$(mktemp -d "$NIRI_MATERIAL_WORK_ROOT/v1-drm-acceptance.XXXXXX")
handoff=$NIRI_MATERIAL_WORK_ROOT/v1-drm-acceptance-handoff.json
test ! -e "$handoff"
sh fixtures/v1-drm-smoke.sh --prepare \
    "$candidate_binary" "$artifact_dir" "$handoff"

set +e
sh fixtures/v1-drm-smoke.sh --run "$handoff"
refusal_status=$?
set -e
test "$refusal_status" -eq 2
test ! -e "$NIRI_MATERIAL_WORK_ROOT/v1-drm-acceptance.lock"
```

Expected refusal explains that the current session is not active VT2 and
creates no runtime directory, socket, client, or compositor.

- [x] **Step 5: Inspect the generated VT2 wrapper and handoff**

`--prepare` has already written the wrapper from resolved paths. Verify that
it contains only the shebang, `set -eu`, exact work-root export, and one
`exec` of the absolute fixture path with the absolute handoff path:

```sh
test "$(stat -c %a /mnt/ssd3/niri-material/run-v1-drm-acceptance.sh)" = 700
sed -n '1,20p' /mnt/ssd3/niri-material/run-v1-drm-acceptance.sh
jq . /mnt/ssd3/niri-material/v1-drm-acceptance-handoff.json
sha256sum --check "$artifact_dir/prepare.sha256"
```

Do not switch VTs until a reviewer confirms the wrapper, handoff, candidate
hash, clean trees, and successful self-test.

---

### Task 5: Execute the observed physical DRM smoke

**Files:**
- Write only beneath the prepared external artifact directory.

**Interfaces:**
- Consumes: reviewed VT2 wrapper and operator attention for the complete run.
- Produces: raw machine evidence plus nine explicit physical observations
  reported after returning to VT1.

- [x] **Step 1: Record the pre-switch state and give the recovery procedure**

Before asking the operator to switch, record current VT1/session/output facts
and confirm the existing niri session is healthy. State the recovery steps
verbatim:

```text
1. Press Ctrl+Alt+F2 and log in.
2. Run /mnt/ssd3/niri-material/run-v1-drm-acceptance.sh.
3. Watch the whole sequence; do not press the exit bind early.
4. When the final window says the sequence is complete, inspect the display.
5. Press Mod+Shift+E to exit the candidate.
6. Press Ctrl+Alt+F1 to return to the original session.
```

If the candidate display is unusable, skip directly to step 6. Do not kill
the VT1 compositor.

- [x] **Step 2: Run on VT2 and watch every physical gate**

The operator watches specifically for:

1. blank or black frames;
2. untreated flashes on map, reload, move, resize, overview, or workspace
   switch;
3. slab detachment;
4. retained-pixel smear, jump, corruption, or stale content;
5. visible valid reload of refraction/attenuation;
6. unchanged appearance after the invalid reload;
7. jelly during move/resize and exact visual settling;
8. overview/workspace voids, detached bands, or clipped strip-end overhang;
9. responsiveness through exit.

No blanket observation substitutes for these rows.

- [x] **Step 3: Return to VT1 and verify machine completion before asking for observations**

```sh
handoff=/mnt/ssd3/niri-material/v1-drm-acceptance-handoff.json
artifact_dir=$(jq -r .artifact_dir "$handoff")
test -f "$artifact_dir/machine-summary.json"
jq . "$artifact_dir/machine-summary.json"
test ! -e /mnt/ssd3/niri-material/v1-drm-acceptance.lock
test -z "$(find /mnt/ssd3/niri-material -maxdepth 1 \
    -type d -name 'v1-drm-runtime.*' -print -quit)"
(cd "$artifact_dir" && sha256sum --check captures.sha256)
```

If integrity is incomplete or analyzer exit is 2, stop without interpreting
the physical observations. If evidence is valid, ask the operator to report
PASS or FAIL for each numbered row and record their wording verbatim.

- [x] **Step 4: Regrade independently**

From the evidence worktree:

```sh
env NIRI_MATERIAL_WORK_ROOT=/mnt/ssd3/niri-material \
    sh fixtures/v1-drm-smoke.sh --analyze "$handoff"
jq -e '.verdict == "pass" and (.gates | all(.status == "pass"))' \
    "$artifact_dir/machine-summary.json"
```

Expected for acceptance: analyzer exit 0 and every machine gate PASS. Exit 1
is a valid machine failure; retain and report it. Exit 2 is invalid evidence.

---

### Task 6: Record the verdict and reconcile every status surface

**Files:**
- Create: `niri-experiments/docs/results/2026-08-27-v1-drm-acceptance.md`
- Modify on PASS, valid FAIL, or INVALID:
  - `docs/materials/2026-08-22-v1-design.md`
  - `docs/materials/2026-08-24-v1-parity-design.md`
  - `docs/materials/2026-08-25-v1-reference-static-preflight-design.md`
  - `docs/materials/2026-08-24-glass-config-surface-design.md`
  - `docs/materials/plans/2026-08-24-v1-parity.md`
  - `docs/materials/plans/2026-08-26-v1-reference-static-preflight.md`
  - `docs/materials/plans/2026-08-24-glass-config-surface.md`
  - `docs/materials/README.md`
  - `docs/materials/2026-08-27-v1-drm-acceptance-design.md`
  - `docs/materials/plans/2026-08-27-v1-drm-acceptance.md`

**Interfaces:**
- Consumes: independently verified machine summary, capture inventory,
  environment state, complete operator table, and fixture/candidate pins.
- Produces: the sole auditable v1 acceptance verdict and consistent current
  status claims in both repositories.

- [x] **Step 1: Write the result from actual evidence only**

Set the result header to exactly one of `PASS`, `FAIL`, or `INVALID` using the
rules below. The document must contain these sections in order:

```markdown
# Native materials v1 physical DRM acceptance — results

## Verdict
## Pinned candidate and fixtures
## Physical environment
## Procedure
## Machine gates
## Physical observations
## Log review
## Capture inventory
## Cleanup state
## Remaining scope
```

Populate every value from the handoff, machine summary, `captures.sha256`,
system facts, and the operator's nine individual answers. Include the fixture
commit and hashes, candidate source/version/hash, kernel/NVIDIA facts, GPU PCI
ID, seat/VT, connector/model/mode/scale/transform/VRR, action trace, every
machine metric, every warning/error line classification, capture inventory
line count and file hash, and raw artifact path.

PASS requires machine PASS plus nine physical PASS rows. A valid failure names
each failed row and keeps v1 blocked. INVALID records the integrity defect but
does not infer visual correctness.

- [x] **Step 2: Verify and commit the evidence result**

```sh
sha256sum "$artifact_dir/captures.sha256" \
    "$artifact_dir/machine-summary.json" \
    "$artifact_dir/niri.log"
(cd "$artifact_dir" && sha256sum --check captures.sha256)
git diff --check
git add docs/results/2026-08-27-v1-drm-acceptance.md
git diff --cached --check
git commit -m "docs(results): record v1 physical DRM acceptance"
```

The result commit hash becomes the evidence pin used in the material docs.

- [x] **Step 3: Update current material status claims from the verdict**

For PASS, state that physical DRM passed on the pinned RTX 3070/DP-1 system
and native materials v1 is accepted, citing the literal evidence result
commit. For FAIL, name the failed gates and state that v1 remains blocked. For
INVALID, state only that physical DRM remains incomplete.

Update the new design status to implemented with the result commit. Update
this plan status to executed, and check only steps evidenced by the tree and
run artifacts. Do not rewrite historical procedure prose as though it had
always been complete.

- [x] **Step 4: Grep both repositories for propagated drift**

```sh
rg -n 'physical DRM|DRM smoke|v1 acceptance|v1 accepted|remains|pending|blocked|incomplete' \
    docs/materials docs/materials/README.md
rg -n 'physical DRM|DRM smoke|v1 acceptance|v1 accepted|remains|pending|blocked|incomplete' \
    docs/results fixtures
```

Read every hit. Current status surfaces must agree with the actual verdict;
historical instructions may remain when their status header makes chronology
unambiguous.

- [x] **Step 5: Verify and commit the material status change**

```sh
git diff --check
git diff --name-only
test -z "$(git diff --name-only -- . ':(exclude)docs')"
git add docs/materials
git diff --cached --check
git commit -m "docs(materials): record v1 DRM acceptance"
```

---

### Task 7: Review, clean external artifacts, and hand off integration

**Files:**
- Review both committed doc trees and the retained external artifact set.
- Remove only the exact temporary worktree, run, handoff, wrapper, and target
  after review accepts them as no longer needed.

**Interfaces:**
- Consumes: clean feature branches with reviewed evidence and status commits.
- Produces: review-ready branches, no running acceptance state, and no stale
  large artifacts outside Dropbox.

- [x] **Step 1: Run final independent checks before deleting evidence**

```sh
(cd "$artifact_dir" && sha256sum --check captures.sha256)
env NIRI_MATERIAL_WORK_ROOT=/mnt/ssd3/niri-material \
    sh fixtures/v1-drm-smoke.sh --analyze "$handoff"
git status --short
git diff --check
```

Review the machine summary against the result table, all nine operator rows,
the exact log classifications, both repository pins, and every propagated
status claim. Do not clean artifacts until this review is accepted.

- [x] **Step 2: Move exact external artifacts to trash**

After review approval, resolve and print the targets first:

```sh
printf '%s\n' "$artifact_dir" "$handoff" \
    /mnt/ssd3/niri-material/run-v1-drm-acceptance.sh \
    /mnt/ssd3/niri-material/targets/v1-drm-acceptance
```

Require each path to match the explicit names above, then use `gio trash` on
those four targets. Report their total size and that recovery remains possible
until trash is emptied. Verify no `v1-drm-runtime.*`, lock, niri process,
client, or socket remains.

- [x] **Step 3: Remove the detached candidate worktree**

Run from the main material checkout:

```sh
material_repo=$(git rev-parse --show-toplevel)
candidate_worktree=$material_repo/.worktrees/v1-drm-candidate
git -C "$candidate_worktree" status --short
git -C "$material_repo" worktree remove "$candidate_worktree"
git -C "$material_repo" worktree prune
```

Expected: the detached candidate worktree is clean and removed without
`--force`.

- [x] **Step 4: Present branch integration choices**

Keep both feature worktrees until the human chooses integration. Use the
finishing-a-development-branch workflow separately for:

- material branch `v1-drm-acceptance` into `materials-26.04`;
- evidence branch `results/v1-drm-acceptance` into `results/slice3`.

Never merge, push, or delete either feature branch without that explicit
choice.
