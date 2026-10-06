//! Replays a compositor event sequence through the existing event-stream reducer
//! (`niri_ipc::state`) and folds a candidate per-workspace maximum signal level client-side.
//!
//! The question (material-c1330b) is whether a client can derive that summary from the existing
//! snapshots and events without a compositor summary event. "Maximum level" here is a candidate
//! summary only, not an approved accent, motion or impulse fold.
//!
//! The event shapes and their order mirror what the compositor emits (see the brief
//! docs/notes/2026-09-29-signal-model-extensions-brief.md, section "Workspace replay"):
//!
//! - every layout refresh runs `ipc_refresh_workspaces` before `ipc_refresh_windows`
//!   (src/ipc/server.rs `ipc_refresh_layout`), and the compositor applies each event to its own
//!   `EventStreamState` before sending it, so its mirror equals what a client folds;
//! - a workspace change on a known window is a full `WindowOpenedOrChanged` carrying the window's
//!   current folded signal, and no separate `WindowSignalChanged` is sent for it in that refresh;
//! - TTL demotion, clearing and impulse expiry arrive as `WindowSignalChanged`;
//! - a close is a bare `WindowClosed`;
//! - a new event-stream client first receives `replicate()` of the compositor's mirror:
//!   `WorkspacesChanged`, then `WindowsChanged`, then the other parts.

use std::collections::BTreeMap;

use niri_ipc::state::{EventStreamState, EventStreamStatePart as _};
use niri_ipc::{
    Event, Impulse, ImpulseKind, Signal, SignalLevel, SignalMotion, Timestamp, Window,
    WindowLayout, Workspace,
};

use SignalLevel::{Active, Demand, Notice, Quiet};

/// Per-workspace candidate summary: `None` means no window on the workspace carries a signal,
/// which is distinct from `Some(Quiet)`.
type Levels = BTreeMap<u64, Option<SignalLevel>>;

/// Folds the maximum signal level of each known workspace from the reducer's window map.
///
/// The fold is a pure function of the reducer state, so snapshots (`WindowsChanged`) need no
/// special handling. Windows whose `workspace_id` is `None` or names an unknown workspace are not
/// attributed to any workspace; see [`unattributed`].
fn workspace_levels(state: &EventStreamState) -> Levels {
    let mut levels: Levels = state
        .workspaces
        .workspaces
        .keys()
        .map(|id| (*id, None))
        .collect();
    for win in state.windows.windows.values() {
        let Some(level) = win.signal.as_ref().map(|s| s.level) else {
            continue;
        };
        let Some(slot) = win.workspace_id.and_then(|ws| levels.get_mut(&ws)) else {
            continue;
        };
        *slot = (*slot).max(Some(level));
    }
    levels
}

/// Signalled windows that the fold cannot attribute to a known workspace.
fn unattributed(state: &EventStreamState) -> BTreeMap<u64, (Option<u64>, SignalLevel)> {
    state
        .windows
        .windows
        .values()
        .filter_map(|win| {
            let level = win.signal.as_ref()?.level;
            let known = win
                .workspace_id
                .is_some_and(|ws| state.workspaces.workspaces.contains_key(&ws));
            (!known).then_some((win.id, (win.workspace_id, level)))
        })
        .collect()
}

/// The compositor's event-stream mirror plus one connected client.
struct Replay {
    compositor: EventStreamState,
    client: EventStreamState,
}

impl Replay {
    /// Connects a fresh client: it receives `replicate()` of the compositor mirror.
    fn connect(compositor: EventStreamState) -> Self {
        let mut client = EventStreamState::default();
        for event in compositor.replicate() {
            assert!(client.apply(event).is_none());
        }
        Self { compositor, client }
    }

    /// One emitted event: the compositor applies it to its mirror, then the client receives it.
    fn emit(&mut self, event: Event) {
        assert!(self.compositor.apply(event.clone()).is_none());
        assert!(self.client.apply(event).is_none());
    }

    /// Asserts the client's folded levels and that they match the compositor mirror's.
    #[track_caller]
    fn expect(&self, expected: &[(u64, Option<SignalLevel>)]) {
        let expected: Levels = expected.iter().copied().collect();
        assert_eq!(workspace_levels(&self.client), expected);
        assert_eq!(workspace_levels(&self.compositor), expected);
    }
}

fn workspace(id: u64) -> Workspace {
    Workspace {
        id,
        idx: u8::try_from(id).unwrap(),
        name: None,
        output: Some(String::from("DP-1")),
        is_urgent: false,
        is_active: id == 1,
        is_focused: id == 1,
        active_window_id: None,
        scrolling_view_pos: 0.,
    }
}

fn window(id: u64, workspace_id: Option<u64>, signal: Option<Signal>) -> Window {
    Window {
        id,
        title: None,
        app_id: None,
        pid: None,
        workspace_id,
        is_focused: false,
        is_floating: false,
        is_urgent: false,
        layout: WindowLayout {
            pos_in_scrolling_layout: None,
            tile_size: (0., 0.),
            window_size: (0, 0),
            tile_pos_in_workspace_view: None,
            window_offset_in_tile: (0., 0.),
        },
        focus_timestamp: None,
        signal,
    }
}

fn signal(level: SignalLevel, sources: &[&str]) -> Signal {
    Signal {
        level,
        motion: SignalMotion::Static,
        accent: None,
        tag: None,
        sources: sources.iter().map(|s| String::from(*s)).collect(),
        impulses: Vec::new(),
    }
}

fn changed(id: u64, signal: Option<Signal>) -> Event {
    Event::WindowSignalChanged { id, signal }
}

fn moved(id: u64, workspace_id: Option<u64>, signal: Option<Signal>) -> Event {
    Event::WindowOpenedOrChanged {
        window: window(id, workspace_id, signal),
    }
}

#[test]
fn client_folds_workspace_max_signal_through_moves_expiry_clear_close_and_reconnect() {
    // Compositor state before the client connects: windows 10 and 11 on workspace 1, 20 on
    // workspace 2, and 30 alone on workspace 3 with a Demand slot already set.
    let mut compositor = EventStreamState::default();
    compositor.apply(Event::WorkspacesChanged {
        workspaces: vec![workspace(1), workspace(2), workspace(3)],
    });
    compositor.apply(Event::WindowsChanged {
        windows: vec![
            window(10, Some(1), None),
            window(11, Some(1), None),
            window(20, Some(2), None),
            window(30, Some(3), Some(signal(Demand, &["ci"]))),
        ],
    });

    // S0: connect. The initial snapshot already carries window 30's signal.
    let mut r = Replay::connect(compositor);
    r.expect(&[(1, None), (2, None), (3, Some(Demand))]);

    // S1: no signal versus Quiet. A Quiet slot on 20 makes workspace 2 Some(Quiet), while
    // workspace 1 (signal-free windows) stays None.
    r.emit(changed(20, Some(signal(Quiet, &["build"]))));
    r.expect(&[(1, None), (2, Some(Quiet)), (3, Some(Demand))]);

    // S2-S3: competing windows on workspace 1. Notice on 10, then Demand with a TTL (after-level
    // Active) and a live Ping impulse on 11: the maximum wins.
    r.emit(changed(10, Some(signal(Notice, &["mail"]))));
    r.expect(&[(1, Some(Notice)), (2, Some(Quiet)), (3, Some(Demand))]);
    let mut demand_with_impulse = signal(Demand, &["agent"]);
    demand_with_impulse.impulses.push(Impulse {
        source: String::from("agent"),
        kind: ImpulseKind::Ping,
        accent: None,
        at: Timestamp { secs: 10, nanos: 0 },
        expires_at: Timestamp {
            secs: 11,
            nanos: 500_000_000,
        },
    });
    r.emit(changed(11, Some(demand_with_impulse)));
    r.expect(&[(1, Some(Demand)), (2, Some(Quiet)), (3, Some(Demand))]);

    // S4: impulse expiry is a WindowSignalChanged whose level is unchanged.
    r.emit(changed(11, Some(signal(Demand, &["agent"]))));
    r.expect(&[(1, Some(Demand)), (2, Some(Quiet)), (3, Some(Demand))]);

    // S5: window 11 moves to workspace 2. One WindowOpenedOrChanged updates both workspaces.
    r.emit(moved(11, Some(2), Some(signal(Demand, &["agent"]))));
    r.expect(&[(1, Some(Notice)), (2, Some(Demand)), (3, Some(Demand))]);

    // S6: TTL demotion of 11's slot to its after-level (Active), via signal_deadline_fired and the
    // next refresh. The slot, and the window's Some(signal), remain.
    r.emit(changed(11, Some(signal(Active, &["agent"]))));
    r.expect(&[(1, Some(Notice)), (2, Some(Active)), (3, Some(Demand))]);

    // S7: window 30 moves to workspace 1 and its now-empty workspace 3 is removed in the same
    // refresh. Workspaces are refreshed first, so the client briefly holds a Demand window that
    // names a removed workspace: no known workspace shows it.
    r.emit(Event::WorkspacesChanged {
        workspaces: vec![workspace(1), workspace(2)],
    });
    r.expect(&[(1, Some(Notice)), (2, Some(Active))]);
    assert_eq!(
        unattributed(&r.client),
        BTreeMap::from([(30, (Some(3), Demand))])
    );
    r.emit(moved(30, Some(1), Some(signal(Demand, &["ci"]))));
    r.expect(&[(1, Some(Demand)), (2, Some(Active))]);
    assert!(unattributed(&r.client).is_empty());

    // S8: an interactive move (drag) reports the window with workspace_id None until it is
    // dropped, so its Demand is attributed to no workspace while it is held.
    r.emit(moved(30, None, Some(signal(Demand, &["ci"]))));
    r.expect(&[(1, Some(Notice)), (2, Some(Active))]);
    assert_eq!(
        unattributed(&r.client),
        BTreeMap::from([(30, (None, Demand))])
    );
    r.emit(moved(30, Some(2), Some(signal(Demand, &["ci"]))));
    r.expect(&[(1, Some(Notice)), (2, Some(Demand))]);

    // S9: clearing the final source of window 10 sends signal None. Workspace 1 still has
    // windows, but none signalled: None, not Quiet.
    r.emit(changed(10, None));
    r.expect(&[(1, None), (2, Some(Demand))]);

    // S10: closes are bare WindowClosed events; the fold drops the window's level with it.
    r.emit(Event::WindowClosed { id: 30 });
    r.expect(&[(1, None), (2, Some(Active))]);
    r.emit(Event::WindowClosed { id: 11 });
    r.expect(&[(1, None), (2, Some(Quiet))]);

    // S11: the client disconnects and misses two events: 20's last source is cleared and a new
    // window 12 opens on workspace 1 with Notice.
    let stale = workspace_levels(&r.client);
    r.compositor.apply(changed(20, None));
    r.compositor
        .apply(moved(12, Some(1), Some(signal(Notice, &["mail"]))));
    let current: Levels = BTreeMap::from([(1, Some(Notice)), (2, None)]);
    assert_eq!(workspace_levels(&r.compositor), current);
    assert_ne!(stale, current);

    // A reconnecting client gets a fresh snapshot. Between its first event (WorkspacesChanged) and
    // its second (WindowsChanged), a fresh state folds every workspace to None.
    let snapshot = r.compositor.replicate();
    assert!(matches!(snapshot[0], Event::WorkspacesChanged { .. }));
    assert!(matches!(snapshot[1], Event::WindowsChanged { .. }));
    let mut fresh = EventStreamState::default();
    fresh.apply(snapshot[0].clone());
    assert_eq!(
        workspace_levels(&fresh),
        BTreeMap::from([(1, None), (2, None)])
    );
    for event in snapshot[1..].iter().cloned() {
        fresh.apply(event);
    }
    assert_eq!(workspace_levels(&fresh), current);

    // Reusing the old client state also converges: WindowsChanged replaces the whole window map,
    // so the closed-while-away and opened-while-away windows are reconciled.
    for event in r.compositor.replicate() {
        r.client.apply(event);
    }
    r.expect(&[(1, Some(Notice)), (2, None)]);
}
