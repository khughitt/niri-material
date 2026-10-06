# docs/materials/scripts/vt-lib.sh
# vt-lib.sh: verified VT switching and restoration for the dedicated capture
# lane (docs/specs/2026-10-02-real-tty-settling-lane-design.md §4). Sourced;
# the caller defines fail(). libseat's logind backend releases the seat on
# exit without switching back, so the caller's exit handler must call
# vt_restore. Tests override VT_ACTIVE_FILE, VT_CHVT and VT_LOGINCTL.
VT_ACTIVE_FILE=${VT_ACTIVE_FILE:-/sys/class/tty/tty0/active}
VT_HOME=
vt_active() { local name; name=$(cat "$VT_ACTIVE_FILE"); echo "${name#tty}"; }
# chvt blocks until the VT activates (kbd's chvt waits on VT_WAITACTIVE), so
# the command itself is bounded: 2 s, then KILL 1 s later. The KILL may orphan
# chvt under sudo; the bound plus the readback in vt_switch is the guarantee.
vt_chvt() {
    if [ -n "${VT_CHVT:-}" ]; then timeout -k 1 2 "$VT_CHVT" "$1"
    else timeout -k 1 2 sudo -n /usr/bin/chvt "$1"; fi
}
vt_switch() {   # N: 0 when VT N is active within 2 s of a bounded chvt
    vt_chvt "$1" || return 1
    local _
    for _ in $(seq 20); do [ "$(vt_active)" = "$1" ] && return 0; sleep 0.1; done
    return 1
}
vt_record_home() {
    VT_HOME=$(vt_active)
    [[ $VT_HOME =~ ^[0-9]+$ ]] || fail "cannot read the active VT from $VT_ACTIVE_FILE"
}
vt_spare() {   # the lowest VT in 2..12 that is not home and has no logind session
    local login=${VT_LOGINCTL:-loginctl} sessions used= session vt n
    sessions=$("$login" list-sessions --no-legend) || {
        echo "vt_spare: loginctl list-sessions failed" >&2
        return 1
    }
    while read -r session _; do
        [ -n "$session" ] || continue
        vt=$("$login" show-session "$session" -p VTNr --value) || {
            echo "vt_spare: loginctl show-session $session failed" >&2
            return 1
        }
        used+="$vt"$'\n'
    done <<< "$sessions"
    for n in $(seq 2 12); do
        [ "$n" = "$VT_HOME" ] && continue
        grep -qx "$n" <<< "$used" || { echo "$n"; return 0; }
    done
    return 1
}
vt_active_or_null() {   # the active VT number, or the word null when unreadable
    local active
    if active=$(vt_active 2>/dev/null) && [[ $active =~ ^[0-9]+$ ]]; then echo "$active"
    else echo null; fi
}
vt_restore() {   # OUT_JSON: put VT_HOME back; 1 only when that failed
    local out=$1 from attempt observed unreadable=
    [ -n "$VT_HOME" ] || return 0
    from=$(vt_active_or_null)
    [ "$from" = null ] && unreadable=" (active VT was unreadable before restoring)"
    if [ "$from" = "$VT_HOME" ]; then
        printf '{"outcome": "not-needed", "home": %s}\n' "$VT_HOME" > "$out"
        return 0
    fi
    for attempt in 1 2 3; do
        if vt_switch "$VT_HOME"; then
            printf '{"outcome": "restored", "home": %s, "from": %s, "attempts": %s}\n' \
                "$VT_HOME" "$from" "$attempt" > "$out"
            echo "VT restored to $VT_HOME from ${from/null/unreadable}" >&2
            return 0
        fi
    done
    observed=$(vt_active_or_null)
    printf '{"outcome": "failed", "home": %s, "observed": %s}\n' "$VT_HOME" "$observed" > "$out"
    echo "FAIL: VT restoration failed: home $VT_HOME, active ${observed/null/unreadable}$unreadable" >&2
    return 1
}
