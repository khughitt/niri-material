#!/usr/bin/env bash
# material-signals-smoke.sh: nested headless fixture for the material signals
# design (docs/materials/2026-09-02-material-signals-design.md, section 9).
#
# Usage: docs/materials/scripts/material-signals-smoke.sh MODE
#   visual  build, drive one window through set/pulse/clear, screenshot the error flash
#   ipc     event-stream round trip and every rejection case
#   cases   Tracy redraw counts for every steady-state case
#   gpu     Tracy GPU cost of the material draw: this build's default path against
#           the branch base commit on the identical fixture, plus ring and rim orbit
#
# Every resource name is unique per run, every wait is bounded, and a
# leftover nested socket fails the run. Requires: jq, weston, kitty, swaybg, flock, ss,
# ImageMagick (`magick` or `convert`) for the checkerboard backdrop, cmake
# (only if the 0.13.1 Tracy tools must be rebuilt), NIRI_MATERIAL_WORK_ROOT.
set -euo pipefail

MODE=${1:?"usage: $0 visual|ipc|cases|gpu"}
ROOT=$(git rev-parse --show-toplevel)
cd "$ROOT"
RUN=signals-$$-$(date +%s)
EVIDENCE=${NIRI_MATERIAL_WORK_ROOT:?set to the evidence root the roughness smoke used}
WORK=$EVIDENCE/material-signals-$(git rev-parse --short HEAD)/$RUN
RT=$XDG_RUNTIME_DIR/$RUN-rt       # unique, short: nested niri panics on long socket paths
HOST_SEQ=0; UNIT=; HOST_SOCKET=; NIRI_SOCKET=
NIRI_PID=; EVENTS_PID=; CAP_PID=
STEADY_N=; GPU_NS=

pid_running() { local state; state=$(ps -o stat= -p "$1" 2>/dev/null) || return 1; [[ $state != Z* ]]; }

stop_nested() {
    local rc=0
    if [ -n "$NIRI_PID" ]; then
        local pid=$NIRI_PID child_rc=0 fallback=0
        if [ -S "$NIRI_SOCKET" ]; then
            msg action quit --skip-confirmation >/dev/null 2>&1 || true
            for _ in $(seq 50); do pid_running "$pid" || break; sleep 0.1; done
        fi
        if pid_running "$pid"; then fallback=1; kill "$pid" 2>/dev/null || true; fi
        for _ in $(seq 50); do pid_running "$pid" || break; sleep 0.1; done
        if pid_running "$pid"; then kill -KILL "$pid" 2>/dev/null || true; fi
        wait "$pid" 2>/dev/null || child_rc=$?
        NIRI_PID=; NIRI_SOCKET=
        if [ "$fallback" -ne 0 ]; then echo "FAIL: nested niri required forced cleanup" >&2; rc=1
        elif [ "$child_rc" -ne 0 ]; then echo "FAIL: nested niri exited $child_rc" >&2; rc=1; fi
        # niri removes its socket on a clean exit; a leftover one is a failure.
        if ls "$RT"/niri.*.sock >/dev/null 2>&1; then echo "FAIL: nested niri left its socket behind" >&2; rc=1; fi
    fi
    if [ -n "$UNIT" ]; then
        local unit=$UNIT host_socket=$HOST_SOCKET
        if ! timeout 10 systemctl --user stop "$unit" >/dev/null 2>&1; then
            echo "FAIL: could not stop Weston unit $unit" >&2
            rc=1
        fi
        for _ in $(seq 50); do
            if ! systemctl --user is-active --quiet "$unit" 2>/dev/null && [ ! -e "$host_socket" ] && [ ! -L "$host_socket" ]; then break; fi
            sleep 0.1
        done
        if systemctl --user is-active --quiet "$unit" 2>/dev/null; then echo "FAIL: Weston unit $unit still active" >&2; rc=1; fi
        if [ -e "$host_socket" ] || [ -L "$host_socket" ]; then echo "FAIL: Weston socket left behind at $host_socket" >&2; rc=1; fi
        UNIT=; HOST_SOCKET=
    fi
    return "$rc"
}

cleanup() {
    local rc=$?
    stop_nested || rc=1
    [ -n "$EVENTS_PID" ] && { kill "$EVENTS_PID" 2>/dev/null || true; }
    [ -n "$CAP_PID" ] && { kill "$CAP_PID" 2>/dev/null || true; }
    if ls "$RT"/niri.*.sock >/dev/null 2>&1; then
        echo "FAIL: nested niri socket left behind in $RT" >&2
        rc=1
    fi
    if [ -d "$WORK/ref-src" ]; then
        git worktree remove --force "$WORK/ref-src" || { echo "FAIL: could not remove reference worktree $WORK/ref-src" >&2; rc=1; }
    fi
    rm -rf "$RT"
    exit "$rc"
}
# Unique Tracy port so concurrent runs never share the default 8086; the
# client reads TRACY_PORT, the capture tool takes -p.
reserve_tracy_port() {
    local port listeners
    for port in $(seq 20000 39999); do
        exec {TRACY_LOCK_FD}>"$XDG_RUNTIME_DIR/niri-material-tracy-$port.lock"
        if flock -n "$TRACY_LOCK_FD" \
            && listeners=$(ss -H -ltn "sport = :$port") && [ -z "$listeners" ]; then
            TRACY_PORT=$port; export TRACY_PORT
            return
        fi
        exec {TRACY_LOCK_FD}>&-
    done
    echo "FAIL: no free Tracy port in 20000-39999" >&2
    return 1
}
reserve_tracy_port
mkdir -p "$WORK" "$RT"
trap cleanup EXIT

# Cargo's target directory is configured outside the tree here and is
# shared by every worktree, so resolve it rather than assuming ./target, and
# run a snapshot copied into $WORK so a concurrent build cannot replace the
# executable mid-capture. The reference build gets its own target directory.
TARGET=$(cargo metadata --format-version 1 --no-deps | jq -r .target_directory)
case $MODE in
    cases|gpu) cargo build --release --features profile-with-tracy ;;
    *)         cargo build --release ;;
esac
[ -x "$TARGET/release/niri" ] || { echo "FAIL: built binary not found at $TARGET/release/niri" >&2; exit 1; }
cp "$TARGET/release/niri" "$WORK/niri"
NIRI=$WORK/niri
sha256sum "$NIRI" | tee -a "$WORK/SHA256SUMS"

# --- config variants -------------------------------------------------------
# The backdrop is a checkerboard so refraction, distortion, and chromatic
# aberration have detail to act on, and kitty is translucent so the slab is
# visible through the window body (opaque pixels bypass the shader). kitty
# runs with cursor blinking off and no shell, so the only client damage is
# what a case causes.
CHECKER=$WORK/checker.png
if command -v magick >/dev/null; then magick -size 160x90 pattern:checkerboard -scale 800% "$CHECKER"
else convert -size 160x90 pattern:checkerboard -scale 800% "$CHECKER"; fi
KITTY_OPTS='"-o" "cursor_blink_interval=0" "-o" "cursor_stop_blinking_after=0" "-o" "background_opacity=0.6"'
write_config() {   # $1 = path, remaining args = extra KDL lines
    local f=$1; shift
    {
        cat <<EOF
material "tg" { glass {}; }
window-rule { match app-id="^kitty$"; material "tg"; }
spawn-at-startup "swaybg" "-i" "$CHECKER"
spawn-at-startup "kitty" $KITTY_OPTS "--hold" "true"
EOF
        printf '%s\n' "$@"
    } > "$f"
}
# GPU fixtures replace the static kitty with one that repaints ten times a
# second, so both GPU cases measure the same controlled damage and always
# have material draws in the 20 s to 28 s sample window.
write_gpu_config() {   # $1 = path
    {
        cat <<EOF
material "tg" { glass {}; }
window-rule { match app-id="^kitty$"; material "tg"; }
spawn-at-startup "swaybg" "-i" "$CHECKER"
spawn-at-startup "kitty" $KITTY_OPTS "sh" "-c" "while :; do date +%s%N; sleep 0.1; done"
EOF
    } > "$1"
}
write_config "$WORK/base.kdl"
write_gpu_config "$WORK/gpu.kdl"
write_config "$WORK/motion-off.kdl"     'signal { motion "off"; }'
write_config "$WORK/reduced.kdl"        'signal { motion "reduced"; }'
write_config "$WORK/slowdown.kdl"       'animations { slowdown 3; }'
write_config "$WORK/narrow.kdl"         'layout { gaps 0; default-column-width { proportion 0.1; }; }'
write_config "$WORK/attention-none.kdl" 'material "tg2" { glass {}; response "default" { attention "none"; }; }' \
                                         'window-rule { match app-id="^kitty$"; material "tg2"; }'
write_config "$WORK/impulse-none.kdl"   'material "tg2" { glass {}; response "default" { ping "none"; done "none"; error "none"; }; }' \
                                         'window-rule { match app-id="^kitty$"; material "tg2"; }'

# --- host and nested compositor -------------------------------------------
msg() { "$NIRI" msg "$@"; }
kitty_ids() { msg -j windows | jq -r '.[] | select(.app_id=="kitty") | .id'; }
kitty_count() { kitty_ids | wc -l; }
win() { msg -j windows | jq -r --argjson id "$1" ".[] | select(.id==\$id) | $2"; }   # $2 = jq path
assert_eq() { [ "$1" = "$2" ] || { echo "FAIL: $3: got '$1', want '$2'" >&2; exit 1; }; }
expect_fail() { if "$@" >/dev/null 2>&1; then echo "FAIL: expected failure: $*" >&2; exit 1; fi; }

start_nested() {   # $1 = config path; sets NIRI_SOCKET and WID
    HOST_SEQ=$((HOST_SEQ + 1))
    local host=$RUN-host-$BASHPID-$HOST_SEQ
    UNIT=$host-weston; HOST_SOCKET=$XDG_RUNTIME_DIR/$host
    systemd-run --user --unit="$UNIT" --collect \
        weston --backend=headless --renderer=gl --shell=kiosk-shell.so \
        --width=1280 --height=720 --socket="$host"
    for _ in $(seq 100); do [ -S "$HOST_SOCKET" ] && break; sleep 0.1; done
    [ -S "$HOST_SOCKET" ] || { echo "FAIL: Weston socket never appeared at $HOST_SOCKET" >&2; exit 1; }
    ln -s "$HOST_SOCKET" "$RT/$host"
    XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$host "$NIRI" -c "$1" >> "$WORK/niri.log" 2>&1 &
    NIRI_PID=$!
    for _ in $(seq 100); do ls "$RT"/niri.*.sock >/dev/null 2>&1 && break; sleep 0.1; done
    NIRI_SOCKET=$(ls -t "$RT"/niri.*.sock | head -1); export NIRI_SOCKET
    wait_kitty 1
    WID=$(kitty_ids | head -1)
}
wait_kitty() {   # $1 = count
    for _ in $(seq 100); do [ "$(kitty_count)" -ge "$1" ] && return; sleep 0.1; done
    echo "FAIL: only $(kitty_count) kitty windows, wanted $1" >&2; exit 1
}
spawn_kitty_to() {   # $1 = total count wanted
    while [ "$(kitty_count)" -lt "$1" ]; do
        local before; before=$(kitty_count)
        msg action spawn -- kitty -o cursor_blink_interval=0 -o cursor_stop_blinking_after=0 -o background_opacity=0.6 --hold true
        wait_kitty $((before + 1))
    done
}
set_demand() {   # $1 = id, $2 = motion, rest = extra flags
    local id=$1 motion=$2; shift 2
    msg set-window-signal --id "$id" --source demo --accent '#e5a33c' --level demand --motion "$motion" "$@"
}

# --- Tracy 0.13.1 tools ----------------------------------------------------
tools_ready() {
    local retained=$EVIDENCE/material-roughness-b220152d/tools
    TOOLS=$retained
    if sha256sum -c --quiet - <<EOF 2>/dev/null
7b95c9c388b6b689cd87da490b564dd086fa8c9dfd6489424b2401d6ef0c5b0c  $retained/tracy-capture
472e08726cc62ab66ed38f75bd4cbb351f2160c1f742a709a9d5be57c68fb463  $retained/tracy-csvexport
EOF
    then return; fi
    echo "retained Tracy 0.13.1 tools missing or altered; rebuilding" >&2
    git clone --depth 1 --branch v0.13.1 https://github.com/wolfpld/tracy "$WORK/tracy-src"
    cmake -S "$WORK/tracy-src/capture"   -B "$WORK/cap-build" -DCMAKE_BUILD_TYPE=Release
    cmake --build "$WORK/cap-build"
    cmake -S "$WORK/tracy-src/csvexport" -B "$WORK/csv-build" -DCMAKE_BUILD_TYPE=Release
    cmake --build "$WORK/csv-build"
    TOOLS=$WORK/tools; mkdir -p "$TOOLS"
    cp "$WORK/cap-build/tracy-capture" "$WORK/csv-build/tracy-csvexport" "$TOOLS/"
    "$TOOLS/tracy-csvexport" --help >/dev/null
    sha256sum "$TOOLS"/* | tee -a "$WORK/SHA256SUMS"
}
# tracy-capture's -s timer starts only once a client connects and its connect
# loop is unbounded, so the whole capture is wrapped in a timeout.
capture_bg() {   # stdout and stderr go to a per-capture log so numeric substitutions stay clean
    : > "$WORK/$1.capture.log"
    timeout 90 "$TOOLS/tracy-capture" -o "$WORK/$1.tracy" -a 127.0.0.1 -p "$TRACY_PORT" -s 30 \
        > "$WORK/$1.capture.log" 2>&1 & CAP_PID=$!
}
# The -s timer starts when the client connects. Since tracy-capture buffers
# redirected output, wait for its TCP session on this run's locked unique port.
capture_ready() {   # $1 = case
    local _; for _ in $(seq 300); do
        [ -n "$(ss -H -tn state established "( sport = :$TRACY_PORT or dport = :$TRACY_PORT )")" ] && return
        kill -0 "$CAP_PID" 2>/dev/null || break
        sleep 0.1
    done
    echo "FAIL: tracy-capture never reported a connection (see $WORK/$1.capture.log)" >&2; exit 1
}
capture_wait() {
    local rc=0; wait "$CAP_PID" || rc=$?; CAP_PID=
    [ "$rc" -eq 0 ] || { echo "FAIL: tracy-capture exited $rc (no client connected, or timed out)" >&2; exit 1; }
}
# Column lookup by header name, never by position: the CPU (--unwrap) and
# GPU (--gpu) exports lay their columns out differently.
col() {   # $1 = csv, $2 = header name; prints the 1-based column index
    local idx; idx=$(head -1 "$1" | tr ',' '\n' | grep -nx "$2" | cut -d: -f1)
    [ -n "$idx" ] || { echo "FAIL: column '$2' not in $(head -1 "$1")" >&2; exit 1; }
    echo "$idx"
}
export_cpu() { [ -s "$WORK/$1.csv" ] || "$TOOLS/tracy-csvexport" --unwrap "$WORK/$1.tracy" > "$WORK/$1.csv"; }
trace_end() {   # latest timestamp of ANY zone, in ns
    local c; c=$(col "$WORK/$1.csv" ns_since_start)
    awk -F, -v c="$c" 'NR>1 { t=$c+0; if (t>end) end=t } END { printf "%d", end }' "$WORK/$1.csv"
}
# Zones named exactly `Niri::redraw` (a substring filter would also match
# `Niri::redraw_queued_outputs`) with timestamps in [end - $2 s, end - $3 s).
count_window() {   # $1 = case, $2 = seconds-before-end start, $3 = seconds-before-end stop
    export_cpu "$1"
    local end c; end=$(trace_end "$1"); c=$(col "$WORK/$1.csv" ns_since_start)
    awk -F, -v c="$c" -v end="$end" -v a="$2" -v b="$3" \
        'NR>1 && $1=="Niri::redraw" { t=$c+0; if (t>=end-a*1e9 && t<end-b*1e9) n++ } END { printf "%d", n+0 }' "$WORK/$1.csv"
}
count_steady() {   # $1 = case; the final 20 s; appends to rates.txt and prints the count
    local n; n=$(count_window "$1" 20 0)
    printf '%s: %d zones in 20 s = %.1f/s\n' "$1" "$n" "$(awk -v n="$n" 'BEGIN { printf "%.1f", n/20 }')" | tee -a "$WORK/rates.txt" >&2
    echo "$n"
}
# Median exec time of the first 14 `MaterialRenderElement::draw` GPU zones
# between 20 s and 28 s of a trace: the roughness smoke's steady-state
# sample rule applied to the material-specific zone (the generic
# `draw shader` zone also covers border, shadow, and resize shaders). GPU
# zones need the --gpu export; a missing column or a short sample set fails.
gpu_median_ns() {   # $1 = trace path; prints the median in ns. Derived files stay under $WORK.
    local csv=$WORK/$(basename "$1").gpu.csv
    "$TOOLS/tracy-csvexport" --gpu "$1" > "$csv"
    # Tracy 0.13.1's GPU export names these columns "Time from start of
    # program" and "GPU execution time"; `col` fails with the real header if
    # a different tool version disagrees.
    local ct ce; ct=$(col "$csv" "Time from start of program"); ce=$(col "$csv" "GPU execution time")
    awk -F, -v ct="$ct" -v ce="$ce" 'NR>1 && $1=="MaterialRenderElement::draw" && $ct+0>=20e9 && $ct+0<28e9 { print $ce+0 }' "$csv" \
        | head -14 | sort -n | awk '{ a[NR]=$1 } END { if (NR!=14) { print "FAIL: " NR " MaterialRenderElement::draw samples in 20-28 s, need 14" > "/dev/stderr"; exit 1 }
              printf "%d", (a[7]+a[8])/2 }'
}
# Acceptance helpers. Rates are compared with explicit tolerances.
expect_zero() { [ "$2" -eq 0 ] || { echo "FAIL: $1: expected 0 redraws in window, got $2" >&2; exit 1; }; }
expect_about() {   # $1 label, $2 got, $3 want, $4 tolerance fraction
    awk -v g="$2" -v w="$3" -v t="$4" 'BEGIN { exit !(g >= w*(1-t) && g <= w*(1+t)) }' \
        || { echo "FAIL: $1: got $2, want $3 within $(awk -v t="$4" 'BEGIN { printf "%d%%", t*100 }')" >&2; exit 1; }
}

# --- modes -----------------------------------------------------------------
shot() {   # $1 = label; writes a uniquely named screenshot and waits for it
    local f=$WORK/$1-$(date +%s%N).png
    msg action screenshot-screen --write-to-disk true --show-pointer false --path "$f"
    for _ in $(seq 50); do [ -s "$f" ] && break; sleep 0.1; done
    [ -s "$f" ] || { echo "FAIL: screenshot $1 not written" >&2; exit 1; }
    sha256sum "$f" | tee -a "$WORK/SHA256SUMS"
}
mode_visual() {
    start_nested "$WORK/base.kdl"
    # Breathe (4 s period): two shots half a period apart show the rim glint
    # swayed to opposite sides.
    set_demand "$WID" breathe
    sleep 1.0; shot rim-a
    sleep 2.0; shot rim-b
    # Sweep: progress is age / 1.5 s, so 0.5 s in the band is a third of the way.
    msg pulse-window-signal --id "$WID" --source demo --kind done
    sleep 0.5; shot sweep-mid
    sleep 1.2
    msg pulse-window-signal --id "$WID" --source demo --kind error
    sleep 0.05; shot error-flash                # requested just before the 80 ms peak; the shot lands near it
    sleep 1.6
    msg clear-window-signal --id "$WID" --source demo
    sleep 0.5; shot cleared
}

signal_json() { win "$1" .signal; }
# Runs a batch of requests that must all be rejected: no event may be
# appended and the window's folded signal must be byte-identical afterwards.
rejected_batch() {   # $1 = label; commands on stdin, one per line
    local before_events before_state after_events after_state line
    before_events=$(wc -l < "$WORK/events.jsonl"); before_state=$(signal_json "$WID")
    while IFS= read -r line; do [ -n "$line" ] && eval "expect_fail $line"; done
    sleep 0.3
    after_events=$(wc -l < "$WORK/events.jsonl"); after_state=$(signal_json "$WID")
    assert_eq "$after_events" "$before_events" "$1: rejections emitted no event"
    assert_eq "$after_state" "$before_state" "$1: rejections left the signal unchanged"
}
mode_ipc() {
    start_nested "$WORK/base.kdl"
    msg -j event-stream > "$WORK/events.jsonl" & EVENTS_PID=$!
    sleep 0.5
    set_demand "$WID" pulse
    msg pulse-window-signal --id "$WID" --source demo --kind done
    sleep 1.7                                   # done impulse expires -> event with an empty list
    msg clear-window-signal --id "$WID" --source demo   # -> "signal": null
    # Decay must change the fold: Demand -> Quiet.
    msg set-window-signal --id "$WID" --source demo --level demand --ttl-ms 2000 --after-level quiet
    sleep 2.5
    # Rejection batch 1, checked on its own for events and state.
    rejected_batch "batch 1" <<EOF
msg set-window-signal --id 999999 --source x
msg set-window-signal --id "$WID" --source niri
msg pulse-window-signal --id "$WID" --source niri --kind done
msg clear-window-signal --id "$WID" --source niri
msg pulse-window-signal --id "$WID" --source fresh --kind done
msg set-window-signal --id "$WID" --source demo --accent zzz
msg set-window-signal --id "$WID" --source demo --ttl-ms 90000000 --after-level quiet
msg set-window-signal --id "$WID" --source demo --after-level quiet
EOF
    # Fill the bound: demo + s1..s15 = 16 external slots; then s16 must fail.
    local i; for i in $(seq 15); do msg set-window-signal --id "$WID" --source "s$i"; done
    sleep 0.3
    rejected_batch "slot bound" <<EOF
msg set-window-signal --id "$WID" --source s16
EOF
    kill "$EVENTS_PID"; EVENTS_PID=
    # Ordered assertions on the folded signals, in event order.
    jq -c 'select(.WindowSignalChanged) | .WindowSignalChanged.signal' "$WORK/events.jsonl" > "$WORK/signals.jsonl"
    awk '
        /"level":"Demand"/ && !demand { demand=NR }
        /"kind":"Done"/ && demand && !live { live=NR }
        /"impulses":\[\]/ && live && !expired && NR>live { expired=NR }
        /^null$/ && expired && !cleared { cleared=NR }
        /"level":"Quiet"/ && cleared && !decayed { decayed=NR }
        END {
            if (!demand)  { print "FAIL: no Demand event" > "/dev/stderr"; exit 1 }
            if (!live)    { print "FAIL: no event carrying the live Done impulse" > "/dev/stderr"; exit 1 }
            if (!expired) { print "FAIL: no empty-impulse event after the live one" > "/dev/stderr"; exit 1 }
            if (!cleared) { print "FAIL: no null after the clear" > "/dev/stderr"; exit 1 }
            if (!decayed) { print "FAIL: no Quiet decay event after the TTL set" > "/dev/stderr"; exit 1 }
        }' "$WORK/signals.jsonl"
    echo "ipc: OK ($(wc -l < "$WORK/events.jsonl") events recorded)"
}

# run_case NAME CONFIG SETUP [DURING]: fresh nested instance per case so no
# state leaks. SETUP runs before the capture. DURING runs 12 s after the
# capture reports its connection, INSIDE the steady window (the final
# 20 s), so what it causes is measured rather than discarded. Sub-windows
# are counted relative to the trace end with a second of slack for
# readiness detection: burst [end-18.5 s, end-15.5 s), after [end-14 s, end).
run_case() {
    local name=$1 cfg=$2 setup=$3 during=${4:-true}
    start_nested "$cfg"
    $setup
    capture_bg "$name"
    capture_ready "$name"
    sleep 12
    $during
    capture_wait
    stop_nested
}
other_kitty() { kitty_ids | grep -vx "$WID" | head -1; }
setup_quiet_ring() { msg set-window-signal --id "$WID" --source demo --accent '#e5a33c'; assert_eq "$(win "$WID" .signal.level)" Quiet quiet-ring; }
setup_demand() {   # $1 = motion; signaled window unfocused, no until-focus
    spawn_kitty_to 2; OTHER=$(other_kitty)
    set_demand "$WID" "$1"; msg action focus-window --id "$OTHER"
    assert_eq "$(win "$WID" .is_focused)" false "$1 unfocused"
}
setup_demand_focused() {   # until-focus set, then focused: demoted
    spawn_kitty_to 2; OTHER=$(other_kitty)
    set_demand "$WID" pulse --until-focus; msg action focus-window --id "$OTHER"
    msg action focus-window --id "$WID"
    assert_eq "$(win "$WID" .signal.level)" Quiet "until-focus demoted"
}
setup_ten() {   # $1 = motion; narrow columns so all ten are visible on 1280 px
    spawn_kitty_to 10
    local w; for w in $(kitty_ids); do set_demand "$w" "$1"; done
    local workspace view expected=1 x=0 col width
    workspace=$(win "$WID" .workspace_id)
    view=$(msg -j workspaces | jq -r --argjson id "$workspace" '.[] | select(.id==$id) | .scrolling_view_pos')
    while read -r w col width; do
        assert_eq "$col" "$expected" "ten scrolling columns"
        assert_eq "$(win "$w" .signal.motion)" "$2" "ten $1"
        awk -v x="$x" -v view="$view" -v width="$width" 'BEGIN { x-=view; exit !(width>0 && width<=128 && x<1280 && x+width>0) }' \
            || { echo "FAIL: window $w is not narrow and visible (layout-x=$x view=$view width=$width)" >&2; exit 1; }
        x=$(awk -v x="$x" -v width="$width" 'BEGIN { print x+width }')
        expected=$((expected + 1))
    done < <(msg -j windows | jq -r '[.[] | select(.app_id=="kitty")] | sort_by(.layout.pos_in_scrolling_layout[0]) | .[] | [.id, .layout.pos_in_scrolling_layout[0], .layout.tile_size[0]] | @tsv')
    assert_eq "$expected" 11 "ten scrolling windows"
}
setup_inactive_workspace() {
    spawn_kitty_to 2; OTHER=$(other_kitty)
    set_demand "$WID" pulse                       # no until-focus: focusing must not demote
    msg action focus-window --id "$WID"
    msg action move-window-to-workspace-down --focus false
    assert_eq "$(win "$WID" .signal.level)" Demand "still Demand after move"
    [ "$(win "$WID" .workspace_id)" != "$(win "$OTHER" .workspace_id)" ] || { echo "FAIL: same workspace" >&2; exit 1; }
    assert_eq "$(win "$OTHER" .is_focused)" true "focus stayed"
}
setup_hidden_tab() {
    spawn_kitty_to 2; OTHER=$(other_kitty)
    set_demand "$WID" pulse
    msg action focus-window --id "$OTHER"
    msg action consume-or-expel-window-left     # OTHER joins WID's column
    msg action toggle-column-tabbed-display
    msg action focus-window --id "$OTHER"       # OTHER is the shown tab
    assert_eq "$(win "$WID" '.layout.pos_in_scrolling_layout[0]')" "$(win "$OTHER" '.layout.pos_in_scrolling_layout[0]')" "same column"
    assert_eq "$(win "$WID" .is_focused)" false "hidden tab unfocused"
}
setup_offscreen_column() {
    spawn_kitty_to 4                              # four half-width columns; WID is leftmost
    set_demand "$WID" pulse
    msg action focus-column-last
    local col workspace view width
    col=$(win "$WID" '.layout.pos_in_scrolling_layout[0]'); assert_eq "$col" 1 "WID leftmost column"
    workspace=$(win "$WID" .workspace_id)
    view=$(msg -j workspaces | jq -r --argjson id "$workspace" '.[] | select(.id==$id) | .scrolling_view_pos')
    width=$(win "$WID" '.layout.tile_size[0]')
    awk -v view="$view" -v width="$width" 'BEGIN { exit !(width-view <= 0) }' \
        || { echo "FAIL: WID still in view (view=$view width=$width)" >&2; exit 1; }
}
setup_motion_off() { set_demand "$WID" pulse; assert_eq "$(win "$WID" .signal.motion)" Pulse "stored motion"; }
during_pulses() { local k; for k in 1 2 3; do msg pulse-window-signal --id "$WID" --source demo --kind done; done; }
during_one_done() { msg pulse-window-signal --id "$WID" --source demo --kind done; }

# Steady cases: the final 20 s must match the expectation.
steady_zero()  { run_case "$1" "$2" "$3"; expect_zero "$1" "$(count_steady "$1")"; }
steady_about() { run_case "$1" "$2" "$3"; STEADY_N=$(count_steady "$1"); expect_about "$1" "$STEADY_N" "$4" 0.15; }
# Impulse cases with `none` responses: the pulses fire inside the window and
# may cost at most the coalescible request per pulse and per deadline firing
# (two per pulse), with nothing after them.
impulse_none_case() {   # $1 name, $2 cfg, $3 setup
    run_case "$1" "$2" "$3" during_pulses
    local total after; total=$(count_steady "$1"); after=$(count_window "$1" 14 0)
    [ "$total" -le 6 ] || { echo "FAIL: $1: $total redraws for three none-impulses (max 6)" >&2; exit 1; }
    expect_zero "$1 after" "$after"
}
mode_cases() {
    tools_ready
    steady_zero  quiet-ring           "$WORK/base.kdl" setup_quiet_ring
    local pulse_n breathe_n flash_n n
    steady_about demand-pulse "$WORK/base.kdl" "setup_demand pulse" 540; pulse_n=$STEADY_N
    steady_zero  demand-pulse-focused "$WORK/base.kdl" setup_demand_focused
    steady_about demand-breathe "$WORK/base.kdl" "setup_demand breathe" 160; breathe_n=$STEADY_N
    steady_about demand-flash "$WORK/base.kdl" "setup_demand flash" 320; flash_n=$STEADY_N
    steady_about ten-breathe "$WORK/narrow.kdl" "setup_ten breathe Breathe" "$breathe_n"; n=$STEADY_N; expect_about "ten-breathe vs one" "$n" "$breathe_n" 0.10
    steady_about ten-flash "$WORK/narrow.kdl" "setup_ten flash Flash" "$flash_n"; n=$STEADY_N; expect_about "ten-flash vs one" "$n" "$flash_n" 0.10
    steady_zero  inactive-workspace "$WORK/base.kdl" setup_inactive_workspace
    steady_zero  hidden-tab         "$WORK/base.kdl" setup_hidden_tab
    steady_zero  offscreen-column   "$WORK/base.kdl" setup_offscreen_column
    impulse_none_case motion-off   "$WORK/motion-off.kdl"   setup_motion_off
    steady_about reduced-flash "$WORK/reduced.kdl" "setup_demand flash" "$pulse_n"; n=$STEADY_N; expect_about "reduced-flash vs pulse" "$n" "$pulse_n" 0.15
    steady_zero  attention-none "$WORK/attention-none.kdl" "setup_demand pulse"
    impulse_none_case impulse-none "$WORK/impulse-none.kdl" setup_quiet_ring
    # A drawn `done` impulse: a refresh-rate burst for 1.5 s, then nothing.
    run_case done-pulse "$WORK/base.kdl" setup_quiet_ring during_one_done
    local burst after; burst=$(count_window done-pulse 18.5 15.5); after=$(count_window done-pulse 14 0)
    [ "$burst" -ge 30 ] || { echo "FAIL: done-pulse: only $burst redraws in the 3 s burst window (min 30)" >&2; exit 1; }
    expect_zero "done-pulse after" "$after"
    echo "done-pulse: burst $burst, after $after" | tee -a "$WORK/rates.txt" >&2
    steady_about slowdown "$WORK/slowdown.kdl" "setup_demand pulse" "$pulse_n"; n=$STEADY_N; expect_about "slowdown vs pulse" "$n" "$pulse_n" 0.10
    echo "cases: OK, rates in $WORK/rates.txt"
}

# Three captures per case; the reported figure is the median of the three
# per-capture medians, which is what run-to-run noise is judged against.
gpu_case_ns() {   # $1 = name, $2 = cfg, $3 = setup; sets GPU_NS to the median ns over three runs
    local k m; for k in 1 2 3; do
        run_case "$1-$k" "$2" "$3"
        m=$(gpu_median_ns "$WORK/$1-$k.tracy"); echo "$m" >> "$WORK/$1.medians"
    done
    GPU_NS=$(sort -n "$WORK/$1.medians" | sed -n 2p)
}
# The regression reference is this same fixture run on the branch's base
# commit, built with the same features: GPU time is workload dependent, so
# the retained roughness trace (different backdrop, geometry, and damage
# driver) is not comparable and is not used as a gate.
# The base commit has only the generic `draw shader` GPU zone, so the
# reference checkout gets the same profiling-only wrapper Task 8 adds
# (`MaterialRenderElement::draw`) before it is built. The wrapper changes no
# rendering; it only names the zone. Exactly one call site must match.
patch_reference_gpu_zone() {   # $1 = worktree path
    local f=$1/src/render_helpers/material.rs
    local n; n=$(grep -c 'RenderElement::<GlesRenderer>::draw(&inner, frame, src, dst, damage, opaque_regions, cache)' "$f")
    [ "$n" -eq 1 ] || { echo "FAIL: expected one material draw call site in the reference, found $n" >&2; exit 1; }
    perl -0pi -e 's/RenderElement::<GlesRenderer>::draw\(&inner, frame, src, dst, damage, opaque_regions, cache\)/frame.with_gpu_span(smithay::gpu_span_location!("MaterialRenderElement::draw"), |frame| {\n            RenderElement::<GlesRenderer>::draw(&inner, frame, src, dst, damage, opaque_regions, cache)\n        })/' "$f"
    grep -q 'gpu_span_location!("MaterialRenderElement::draw")' "$f" || { echo "FAIL: reference GPU zone patch did not apply" >&2; exit 1; }
}
build_reference() {   # sets REF_NIRI; its own target dir and a snapshot copy
    local base; base=$(git merge-base HEAD materials-26.04)
    git worktree add --detach "$WORK/ref-src" "$base" >/dev/null
    patch_reference_gpu_zone "$WORK/ref-src"
    (cd "$WORK/ref-src" && CARGO_TARGET_DIR=$WORK/ref-target cargo build --release --features profile-with-tracy)
    [ -x "$WORK/ref-target/release/niri" ] || { echo "FAIL: reference binary not built" >&2; exit 1; }
    cp "$WORK/ref-target/release/niri" "$WORK/niri-ref"
    REF_NIRI=$WORK/niri-ref
    sha256sum "$REF_NIRI" | tee -a "$WORK/SHA256SUMS"
    echo "reference binary: $base plus the MaterialRenderElement::draw GPU zone wrapper" | tee -a "$WORK/gpu.txt"
}
# All three GPU runs share one topology: a single focused ticking kitty and
# no signal-driven redraws. Demand at Static motion exercises the ring and
# the rim-orbit shading (the light is parked, the shader path is the same)
# without adding bucket redraws, so the only difference between runs is
# what the material shader computes per damaged frame.
setup_gpu_ring() { set_demand "$WID" static; assert_eq "$(win "$WID" .signal.level)" Demand "gpu ring"; }
mode_gpu() {
    tools_ready
    build_reference
    local ref_ns default_ns ring_ns
    NIRI=$REF_NIRI
    gpu_case_ns gpu-reference "$WORK/gpu.kdl" true; ref_ns=$GPU_NS
    NIRI=$WORK/niri
    gpu_case_ns gpu-default "$WORK/gpu.kdl" true; default_ns=$GPU_NS             # no signal: the default path
    gpu_case_ns gpu-ring-rim "$WORK/gpu.kdl" setup_gpu_ring; ring_ns=$GPU_NS     # ring + rim orbit at Demand, Static
    {
        printf 'base-commit build, default path (median of 3): %.3f ms\n' "$(awk -v n="$ref_ns" 'BEGIN { print n/1e6 }')"
        printf 'this build, default path, no signal (median of 3): %.3f ms\n' "$(awk -v n="$default_ns" 'BEGIN { print n/1e6 }')"
        printf 'this build, ring + rim orbit, demand static (median of 3): %.3f ms\n' "$(awk -v n="$ring_ns" 'BEGIN { print n/1e6 }')"
        printf 'ring + rim orbit delta vs default: %+.1f%%\n' "$(awk -v a="$default_ns" -v b="$ring_ns" 'BEGIN { print (b-a)/a*100 }')"
    } | tee -a "$WORK/gpu.txt"
    # Gate: the default path must not regress against the base commit on the identical fixture.
    awk -v g="$default_ns" -v r="$ref_ns" 'BEGIN { exit !(g <= r*1.10) }' \
        || { echo "FAIL: default path vs base-commit build: got $default_ns, exceeds $ref_ns by more than 10%" >&2; exit 1; }
    git worktree remove --force "$WORK/ref-src"
    echo "gpu: OK"
}

"mode_$MODE"
