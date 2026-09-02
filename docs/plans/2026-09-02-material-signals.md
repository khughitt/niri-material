# Material Signals Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give every niri window a material-agnostic signal (identity accent, level, motion, impulses) that external sources write over IPC, the compositor animates, and glass renders as ring, rim orbit, sweep, flash, and ripple responses.

**Architecture:** A per-window `WindowSignals` store on `Mapped` folds one slot per source into one signal. The tile turns that into an `EffectiveSignal` (policy and response applied), a pure solver produces a `SignalFrame`, a glass helper derives shader inputs, and `MaterialRenderElement` uploads them. Transient motion rides niri's animation loop; sustained motion arms one clock-aligned timer per output from the render pass. Three IPC requests with a result channel write slots; window rules select a named `response` block per material.

**Tech Stack:** Rust, Smithay, calloop timers, knuffel KDL config, GLES2 fragment shader, `niri msg` CLI (clap), existing `Fixture` test harness.

**Spec:** `docs/materials/2026-09-02-material-signals-design.md` (sections are referenced as §N below). Task record: `material-a54d89`.

## Global Constraints

- Time: every signal timestamp, deadline, and oscillator phase uses the unadjusted monotonic clock (`get_monotonic_time()` / `Clock::now_unadjusted`) as `Duration`. Only the crossfade uses `Clock::now` (§1 Time).
- Bounds: 16 external slots per window, 64-byte source names, 256-byte tags, 4 live impulses (fifth evicts oldest), `ttl_ms <= 86_400_000` (§1 Bounds).
- Reserved source name: `niri` (§1, §2).
- Impulse lifetime 1.5 s: 80 ms linear attack, exponential decay with 350 ms time constant, `progress = age / 1.5 s` (§3).
- Oscillators: Breathe 4 s sine, Pulse 1.2 s sine, Flash 0.5 s square with 50 ms soft edges; Breathe and Pulse phase includes the per-window seed, Flash phase is global (§3).
- Buckets: Breathe and Pulse at `period / 32`, Flash edges sampled at four points per 50 ms edge (§4).
- Fingerprint quantization: level and accent channels 1/256, breath 1/32, impulse envelope and progress 1/128 (§4).
- JSON convention: enum variants keep Rust names in JSON; the CLI accepts kebab-case through `clap::ValueEnum` (§2).
- Config errors are whole-config errors; IPC errors are error replies with state unchanged (§8).
- Commit messages use conventional commits with the scopes already in use: `feat(ipc)`, `feat(config)`, `feat(render)`, `feat(material)`, `docs(material)`. No AI attribution trailers.
- Build and test with `cargo test -p niri-ipc`, `cargo test -p niri-config`, and `cargo test --bin niri <one filter>` from the worktree root. `cargo test` accepts a single positional filter; run the whole crate when a step names several tests.
- Task lifecycle: every `### Task N` heading has a task record linked with `plan:` and `step:`, chained so only the next one is ever `ready`. Before starting a task run `tasks start <its id>`; its final commit runs `tasks done <its id> "<result>"` and stages `tasks/`. The parent record `material-a54d89` depends on Task 12 and is closed only in Task 12's final step, after Task 12's own record.

| Task | Record | | Task | Record |
|---|---|---|---|---|
| 1 | `material-b0e938` | | 7 | `material-f19f8f` |
| 2 | `material-eb7fe7` | | 8 | `material-0e32cc` |
| 3 | `material-f810f4` | | 9 | `material-9d06b4` |
| 4 | `material-5875b2` | | 10 | `material-4a64bb` |
| 5 | `material-25247f` | | 11 | `material-2ecd18` |
| 6 | `material-844eea` | | 12 | `material-b43616` |

---

## File structure

| File | Responsibility |
|---|---|
| `niri-ipc/src/lib.rs` | Wire enums, `Signal`, `Impulse`, three `Request` variants, `Event::WindowSignalChanged`, `Window.signal` |
| `niri-ipc/src/state.rs` | Apply `WindowSignalChanged` to the cached `WindowsState` |
| `niri-config/src/window_rule.rs` | `Match.signal_source`, `Match.signal_tag` |
| `niri-config/src/material.rs` | `MaterialRef` node with `response=`, `Response`, `ResolvedResponse`, glass selector ids, validation |
| `niri-config/src/signal.rs` (new) | `Signal` top-level block with `motion` policy |
| `niri-config/src/animations.rs` | `MaterialSignalAnim` entry |
| `niri-config/src/lib.rs` | Register the `signal` block and validate response references |
| `src/window/signal.rs` (new) | `WindowSignals` store: slots, impulses, fold, decay, bounds, errors. Pure. |
| `src/window/mapped.rs` | Owns `WindowSignals`; native slot from urgency; focus demotion; rule recompute |
| `src/window/mod.rs` | `ResolvedRules.material` becomes `Option<MaterialRef>`; `window_matches` evaluates signal matches |
| `src/layout/mod.rs` | `LayoutElement::signal()`, `Options.signal` |
| `src/niri.rs` | Mutation entry points returning `Result`, per-window deadline timer reconciled from `State::refresh`, unmap cancellation helper, per-output bucket timer, `SignalTicks` injection and view rect in `Niri::render` |
| `src/handlers/compositor.rs`, `src/handlers/xdg_shell.rs` | Call the cancellation helper on unmap |
| `src/ipc/server.rs` | Request handling with result channel, `to_ipc_signal`, diff emitting `WindowSignalChanged` |
| `src/cli.rs`, `src/ipc/client.rs` | `niri msg set-window-signal`, `pulse-window-signal`, `clear-window-signal` |
| `src/render_helpers/signal.rs` (new) | `EffectiveSignal`, `effective()`, `solve()`, `next_boundary()`, `SignalFrame`, `SignalFingerprint`. Pure. |
| `src/render_helpers/material.rs` | `glass_signal_inputs`, `SignalUniforms`, upload |
| `src/render_helpers/shaders/mod.rs` | Register the new uniforms |
| `src/render_helpers/shaders/material.frag` | Ring, rim orbit, ring pulse, sweep |
| `src/render_helpers/mod.rs` | `RenderCtx.signal_ticks` |
| `src/layout/tile.rs` | Crossfade animation, effective signal, solve, fingerprint, transitions term, tick reporting |
| `src/tests/signal.rs` (new) | Fixture tests for urgency, matching, IPC entry points |
| `docs/materials/material-config.md`, `docs/materials/README.md`, design doc | Documentation updates (§10) |

---

### Task 1: IPC wire types and event-state reduction

**Files:**
- Modify: `niri-ipc/src/lib.rs` (`Event` at 1588, `Window` at ~1300, types after `PickedColor` at 181; the `Request` variants are added in Task 6 together with their server arms, because `process()` in `src/ipc/server.rs:271` matches `Request` exhaustively)
- Modify: `niri-ipc/src/state.rs` (`WindowsState::apply` at line 175)
- Modify: `src/ipc/server.rs` (`make_ipc_window` at 513: `signal: None` until Task 6)
- Modify: `src/ipc/client.rs` (the exhaustive non-JSON `Event` match at ~430)
- Test: `niri-ipc/src/state.rs` (`#[cfg(test)]` module at the bottom; create it if absent)

**Interfaces:**
- Produces: `SignalLevel { Quiet, Active, Notice, Demand }`, `SignalMotion { Static, Breathe, Pulse, Flash }`, `ImpulseKind { Ping, Done, Error }`, `Signal`, `Impulse`, `Event::WindowSignalChanged { id, signal }`, `Window.signal: Option<Signal>`. (The three `Request` variants are Task 6.)

- [ ] **Step 1: Write the failing state test**

Append to `niri-ipc/src/state.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Impulse, ImpulseKind, Signal, SignalLevel, SignalMotion, Timestamp, Window, WindowLayout};

    fn window(id: u64) -> Window {
        Window {
            id,
            title: None,
            app_id: None,
            pid: None,
            workspace_id: None,
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
            signal: None,
        }
    }

    #[test]
    fn window_signal_changed_updates_cached_window() {
        let mut state = WindowsState::default();
        state.apply(Event::WindowsChanged { windows: vec![window(7)] });

        let signal = Signal {
            level: SignalLevel::Demand,
            motion: SignalMotion::Pulse,
            accent: Some(String::from("#e5a33c")),
            tag: None,
            sources: vec![String::from("familiar")],
            impulses: vec![Impulse {
                source: String::from("familiar"),
                kind: ImpulseKind::Done,
                accent: None,
                at: Timestamp { secs: 1, nanos: 0 },
                expires_at: Timestamp { secs: 2, nanos: 500_000_000 },
            }],
        };
        state.apply(Event::WindowSignalChanged { id: 7, signal: Some(signal.clone()) });
        assert_eq!(state.windows[&7].signal, Some(signal));

        state.apply(Event::WindowSignalChanged { id: 7, signal: None });
        assert_eq!(state.windows[&7].signal, None);
    }
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test -p niri-ipc window_signal_changed_updates_cached_window`
Expected: compile error, `Signal` and `signal` field not found.

- [ ] **Step 3: Add the wire types**

In `niri-ipc/src/lib.rs`, after `PickedColor` (line ~185):

```rust
/// Attention level of a window signal.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub enum SignalLevel {
    /// Nothing to report.
    Quiet,
    /// Something is happening; no attention needed.
    Active,
    /// Worth a glance.
    Notice,
    /// Needs the user.
    Demand,
}

/// Sustained motion of a window signal.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub enum SignalMotion {
    /// No sustained motion.
    Static,
    /// Slow 4 s oscillation.
    Breathe,
    /// 1.2 s oscillation.
    Pulse,
    /// 0.5 s square wave.
    Flash,
}

/// Kind of a transient signal impulse.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub enum ImpulseKind {
    /// Generic attention tap.
    Ping,
    /// Something finished.
    Done,
    /// Something failed.
    Error,
}

/// A live transient impulse on a window signal.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Impulse {
    /// Source slot this impulse belongs to.
    pub source: String,
    /// Kind of the impulse.
    pub kind: ImpulseKind,
    /// Accent color as `#rrggbb`, if the impulse carries one.
    pub accent: Option<String>,
    /// When the impulse was raised (unadjusted monotonic clock).
    pub at: Timestamp,
    /// When the impulse expires (unadjusted monotonic clock).
    pub expires_at: Timestamp,
}

/// The folded signal of a window.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Signal {
    /// Highest level across sources.
    pub level: SignalLevel,
    /// Motion of the source that won on level.
    pub motion: SignalMotion,
    /// Identity accent as `#rrggbb`, if any source has one.
    pub accent: Option<String>,
    /// Free-form tag, if any source has one.
    pub tag: Option<String>,
    /// Every contributing source name, in slot order.
    pub sources: Vec<String>,
    /// Live impulses, oldest first.
    pub impulses: Vec<Impulse>,
}
```

Add to `Window` after `focus_timestamp`:

```rust
    /// Folded window signal, if any source has written one.
    pub signal: Option<Signal>,
```

Add to `Event` after `WindowUrgencyChanged`:

```rust
    /// The folded signal of a window changed.
    WindowSignalChanged {
        /// Id of the window.
        id: u64,
        /// New folded signal, or `None` when the last slot was cleared.
        signal: Option<Signal>,
    },
```

- [ ] **Step 4: Reduce the event in `WindowsState::apply`**

In `niri-ipc/src/state.rs`, after the `WindowUrgencyChanged` arm:

```rust
            Event::WindowSignalChanged { id, signal } => {
                if let Some(win) = self.windows.get_mut(&id) {
                    win.signal = signal;
                }
            }
```

Fix every other construction of `Window` in the crate (there may be none besides the test).

- [ ] **Step 5: Run the test and the crate**

Run: `cargo test -p niri-ipc`
Expected: PASS. Then `cargo build` from the workspace root. Two compositor sites break and are fixed in this task: `make_ipc_window` in `src/ipc/server.rs:513` gets `signal: None` (Task 6 replaces it with the real fold), and the exhaustive non-JSON event printer in `src/ipc/client.rs:430` gets its final arm now, next to `WindowUrgencyChanged`:

```rust
                    Event::WindowSignalChanged { id, signal } => match signal {
                        Some(signal) => println!("Window {id}: signal changed to {signal:?}"),
                        None => println!("Window {id}: signal cleared"),
                    },
```

- [ ] **Step 6: Commit**

```bash
tasks done material-b0e938 "feat(ipc): add window signal types and event"
git add niri-ipc/src/lib.rs niri-ipc/src/state.rs src/ipc/server.rs src/ipc/client.rs tasks
git commit -m "feat(ipc): add window signal types and event"
```

---

### Task 2: Config: signal matches, `signal { motion }`, and the `material-signal` animation

**Files:**
- Modify: `niri-config/src/window_rule.rs` (`Match` struct)
- Create: `niri-config/src/signal.rs`
- Modify: `niri-config/src/animations.rs` (`Animations`, `AnimationsPart`, merge list, add `MaterialSignalAnim`)
- Modify: `niri-config/src/lib.rs` (`Config` struct at line 13, `ConfigPart` match at line ~200, module list, `pub use`)
- Test: `niri-config/src/lib.rs` tests module (existing `parse_files` helper at line ~740)

**Interfaces:**
- Produces: `Match.signal_source: Option<RegexEq>`, `Match.signal_tag: Option<RegexEq>`, `niri_config::SignalMotionPolicy { Full, Reduced, Off }`, `Config.signal: niri_config::Signal { motion: SignalMotionPolicy }`, `Animations.material_signal: MaterialSignalAnim(pub Animation)` with default easing 400 ms `EaseOutCubic`.

- [ ] **Step 1: Write the failing config tests**

In `niri-config/src/lib.rs` tests module:

```rust
    #[test]
    fn signal_block_and_matches_parse() {
        let parsed = parse_files(&[(
            "config.kdl",
            r##"
            signal { motion "reduced" }

            animations {
                material-signal { duration-ms 250; curve "ease-out-quad" }
            }

            window-rule {
                match signal-source="^familiar$" signal-tag="^cats/"
                opacity 0.9
            }
            "##,
        )])
        .unwrap();

        assert_eq!(parsed.signal.motion, crate::SignalMotionPolicy::Reduced);
        assert_eq!(
            parsed.animations.material_signal.0.kind,
            crate::animations::Kind::Easing(crate::animations::EasingParams {
                duration_ms: 250,
                curve: crate::animations::Curve::EaseOutQuad,
            })
        );
        let m = &parsed.window_rules[0].matches[0];
        assert!(m.signal_source.as_ref().unwrap().0.is_match("familiar"));
        assert!(m.signal_tag.as_ref().unwrap().0.is_match("cats/ginger"));
    }

    #[test]
    fn signal_motion_defaults_to_full() {
        let parsed = parse_files(&[("config.kdl", "")]).unwrap();
        assert_eq!(parsed.signal.motion, crate::SignalMotionPolicy::Full);
        assert_eq!(
            parsed.animations.material_signal.0.kind,
            crate::animations::Kind::Easing(crate::animations::EasingParams {
                duration_ms: 400,
                curve: crate::animations::Curve::EaseOutCubic,
            })
        );
    }

    #[test]
    fn signal_motion_rejects_unknown_value() {
        let err = parse_files_err(&[("config.kdl", r#"signal { motion "loud" }"#)]);
        assert!(err.contains("unknown"), "{err}");
    }
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test -p niri-config signal_`
Expected: compile errors for `signal`, `material_signal`, `signal_source`.

- [ ] **Step 3: Add the match fields**

In `niri-config/src/window_rule.rs`, inside `Match` after `is_urgent`:

```rust
    #[knuffel(property, str)]
    pub signal_source: Option<RegexEq>,
    #[knuffel(property, str)]
    pub signal_tag: Option<RegexEq>,
```

- [ ] **Step 4: Add the `signal` block**

Create `niri-config/src/signal.rs`:

```rust
use knuffel::errors::DecodeError;

use crate::utils::MergeWith;

/// Global motion policy for material signals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SignalMotionPolicy {
    #[default]
    Full,
    Reduced,
    Off,
}

impl std::str::FromStr for SignalMotionPolicy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "full" => Ok(Self::Full),
            "reduced" => Ok(Self::Reduced),
            "off" => Ok(Self::Off),
            _ => Err(format!("unknown signal motion policy: {s}")),
        }
    }
}

/// The top-level `signal { }` block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Signal {
    pub motion: SignalMotionPolicy,
}

#[derive(knuffel::Decode, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SignalPart {
    #[knuffel(child, unwrap(argument, str))]
    pub motion: Option<SignalMotionPolicy>,
}

impl MergeWith<SignalPart> for Signal {
    fn merge_with(&mut self, part: &SignalPart) {
        if let Some(motion) = part.motion {
            self.motion = motion;
        }
    }
}
```

`#[knuffel(child, unwrap(argument, str))]` parses through `FromStr`; the `Err(String)` becomes the knuffel error text, which is what the third test asserts on. If the crate's `MergeWith` trait has a different signature, follow `Blur`/`BlurPart` in `appearance.rs` exactly.

In `niri-config/src/lib.rs`: add `mod signal;`, `pub use crate::signal::{Signal, SignalMotionPolicy, SignalPart};`, the field `pub signal: Signal,` in `Config` (after `blur`), `signal: Default::default()` in `Config::default()`, and `"signal" => m_merge!(signal),` in the `ConfigPart` match after `"blur"`.

- [ ] **Step 5: Add the animation entry**

In `niri-config/src/animations.rs`, next to `RecentWindowsCloseAnim`:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialSignalAnim(pub Animation);

impl Default for MaterialSignalAnim {
    fn default() -> Self {
        Self(Animation {
            off: false,
            kind: Kind::Easing(EasingParams {
                duration_ms: 400,
                curve: Curve::EaseOutCubic,
            }),
        })
    }
}
```

Copy the `knuffel::Decode` impl that `RecentWindowsCloseAnim` uses (it wraps a plain `Animation` node) for `MaterialSignalAnim`. Add `pub material_signal: MaterialSignalAnim` to `Animations`, `material_signal: Default::default()` to its `Default`, `#[knuffel(child)] pub material_signal: Option<MaterialSignalAnim>` to `AnimationsPart`, and `material_signal` to the `merge_clone!` list.

- [ ] **Step 6: Run the tests**

Run: `cargo test -p niri-config`
Expected: all PASS, including the three new ones. If the full-config `parse` test at line ~1274 compares a whole `Config`, add `signal: Signal::default()` and the new animation to its expected value.

- [ ] **Step 7: Commit**

```bash
tasks done material-eb7fe7 "feat(config): add signal matches, motion policy, and material-signal animation"
git add niri-config/src tasks
git commit -m "feat(config): add signal matches, motion policy, and material-signal animation"
```

---

### Task 3: Config: response blocks and `response=` references

**Files:**
- Modify: `niri-config/src/material.rs` (`MaterialRef` at line 22, `MaterialRefs`, `validate_material_refs` at 102, `Material` at 141, `ResolvedMaterial` at 200, `Material::resolve` and `validate` at 250)
- Modify: `niri-config/src/window_rule.rs` line 62 (`material` becomes a node child)
- Modify: `niri-config/src/lib.rs` (`validate_material_refs` call at line 518)
- Test: `niri-config/src/lib.rs` tests module

**Interfaces:**
- Produces:
  - `MaterialRef { pub name: String, pub response: Option<String> }` decoded from `material "name" response="loud"`.
  - `Response` (parsed block), `ResolvedResponse { accent: AccentResponse, attention: AttentionResponse, ping: ImpulseResponse, done: ImpulseResponse, error: ImpulseResponse, ring_inset: f64, ring_width: f64 }`.
  - `AccentResponse { None = 0, Ring = 1 }`, `AttentionResponse { None = 0, RimOrbit = 1, RingPulse = 2 }`, `ImpulseResponse { None = 0, Ripple = 1, Flash = 2, Sweep = 3 }` (all `#[repr(u8)]`, `Copy`).
  - `ResolvedResponse::attention_is_none(&self) -> bool`, `ResolvedResponse::impulse_selector(&self, kind: niri_ipc::ImpulseKind, policy: SignalMotionPolicy) -> Option<u8>` (returns `None` for `ImpulseResponse::None` or policy `Off`; maps `Flash` to `Sweep` under `Reduced`).
  - `ResolvedMaterial { name, glass, responses: Vec<(String, ResolvedResponse)> }` and `ResolvedMaterial::response(&self, name: Option<&str>) -> ResolvedResponse` (default when `None`).
- Consumes: `niri_ipc::ImpulseKind` (Task 1), `SignalMotionPolicy` (Task 2). Add `niri-ipc` as a dependency of `niri-config` only if it is not already one; check `niri-config/Cargo.toml` first. If adding it is undesirable, define a local `ImpulseKind` in `niri-config` with a `From<niri_ipc::ImpulseKind>` impl in the compositor crate.

- [ ] **Step 1: Write the failing tests**

```rust
    #[test]
    fn response_blocks_resolve_with_inheritance() {
        let parsed = parse_files(&[(
            "config.kdl",
            r##"
            material "tg" {
                glass { bevel 12; offset-x 0; offset-y 0 }
                response "default" {
                    accent "ring"
                    attention "rim-orbit"
                    ping "ripple"
                    done "sweep"
                    error "flash"
                    ring-inset 6
                    ring-width 2
                }
                response "loud" {
                    attention "ring-pulse"
                }
            }
            window-rule {
                match app-id="^kitty$"
                material "tg" response="loud"
            }
            "##,
        )])
        .unwrap();

        let m = parsed.materials[0].resolve();
        let loud = m.response(Some("loud"));
        assert_eq!(loud.attention, crate::AttentionResponse::RingPulse);
        assert_eq!(loud.accent, crate::AccentResponse::Ring);
        assert_eq!(loud.done, crate::ImpulseResponse::Sweep);
        assert_eq!(loud.ring_inset, 6.);
        let r = parsed.window_rules[0].material.as_ref().unwrap();
        assert_eq!(r.name, "tg");
        assert_eq!(r.response.as_deref(), Some("loud"));
    }

    #[test]
    fn material_without_response_block_gets_builtin_default() {
        let parsed = parse_files(&[("config.kdl", r#"material "tg" { glass {} }"#)]).unwrap();
        let d = parsed.materials[0].resolve().response(None);
        assert_eq!(d.accent, crate::AccentResponse::Ring);
        assert_eq!(d.attention, crate::AttentionResponse::RimOrbit);
        assert_eq!(d.ping, crate::ImpulseResponse::Ripple);
        assert_eq!(d.done, crate::ImpulseResponse::Sweep);
        assert_eq!(d.error, crate::ImpulseResponse::Flash);
        assert_eq!((d.ring_inset, d.ring_width), (6., 2.));
    }

    #[test]
    fn response_block_without_default_is_an_error() {
        let err = parse_files_err(&[(
            "config.kdl",
            r#"material "tg" { glass {}; response "loud" { attention "ring-pulse" } }"#,
        )]);
        assert!(err.contains("missing response \"default\""), "{err}");
    }

    #[test]
    fn duplicate_response_name_is_an_error() {
        let err = parse_files_err(&[(
            "config.kdl",
            r#"material "tg" { glass {}; response "default" {}; response "default" {} }"#,
        )]);
        assert!(err.contains("duplicate response: default"), "{err}");
    }

    #[test]
    fn ring_must_fit_in_bevel() {
        let err = parse_files_err(&[(
            "config.kdl",
            r#"material "tg" { glass { bevel 4 }; response "default" { ring-inset 3; ring-width 2 } }"#,
        )]);
        assert!(err.contains("ring-inset + ring-width must not exceed bevel"), "{err}");
    }

    #[test]
    fn unknown_response_reference_is_an_error() {
        let err = parse_files_err(&[(
            "config.kdl",
            r#"
            material "tg" { glass {} }
            window-rule { material "tg" response="loud" }
            "#,
        )]);
        assert!(err.contains("unknown response: loud"), "{err}");
    }

    #[test]
    fn material_reference_rejects_malformed_nodes() {
        for (bad, needle) in [
            (r#"material "tg" "extra""#, "unexpected argument"),
            (r#"material "tg" { glass {} }"#, "unexpected node"),
            (r#"(typed)material "tg""#, "no type name expected"),
            (r#"material (typed)"tg""#, "type name"),
            (r#"material "tg" response=(typed)"default""#, "type name"),
            (r#"material "tg" bogus="x""#, "unexpected property"),
        ] {
            let err = parse_files_err(&[(
                "config.kdl",
                &format!("material \"tg\" {{ glass {{}} }}\nwindow-rule {{ {bad} }}"),
            )]);
            assert!(err.contains(needle), "{bad}: {err}");
        }
    }

    #[test]
    fn impulse_selector_follows_policy() {
        use crate::{ImpulseResponse, ResolvedResponse, SignalMotionPolicy};
        let r = ResolvedResponse::default();
        assert_eq!(
            r.impulse_selector(niri_ipc::ImpulseKind::Error, SignalMotionPolicy::Full),
            Some(ImpulseResponse::Flash as u8)
        );
        assert_eq!(
            r.impulse_selector(niri_ipc::ImpulseKind::Error, SignalMotionPolicy::Reduced),
            Some(ImpulseResponse::Sweep as u8)
        );
        assert_eq!(
            r.impulse_selector(niri_ipc::ImpulseKind::Error, SignalMotionPolicy::Off),
            None
        );
        let mut none = r;
        none.done = ImpulseResponse::None;
        assert_eq!(none.impulse_selector(niri_ipc::ImpulseKind::Done, SignalMotionPolicy::Full), None);
    }
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test -p niri-config`
Expected: compile errors.

- [ ] **Step 3: Add the response types**

In `niri-config/src/material.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum AccentResponse {
    None = 0,
    #[default]
    Ring = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum AttentionResponse {
    None = 0,
    #[default]
    RimOrbit = 1,
    RingPulse = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ImpulseResponse {
    #[default]
    None = 0,
    Ripple = 1,
    Flash = 2,
    Sweep = 3,
}

macro_rules! response_from_str {
    ($ty:ident, $($s:literal => $v:ident),+ $(,)?) => {
        impl std::str::FromStr for $ty {
            type Err = String;
            fn from_str(s: &str) -> Result<Self, String> {
                match s {
                    $($s => Ok(Self::$v),)+
                    _ => Err(format!(concat!("unknown ", stringify!($ty), " value: {}"), s)),
                }
            }
        }
    };
}
response_from_str!(AccentResponse, "none" => None, "ring" => Ring);
response_from_str!(AttentionResponse, "none" => None, "rim-orbit" => RimOrbit, "ring-pulse" => RingPulse);
response_from_str!(ImpulseResponse, "none" => None, "ripple" => Ripple, "flash" => Flash, "sweep" => Sweep);

/// A `response "name" { ... }` block inside a material definition.
#[derive(knuffel::Decode, Debug, Clone, PartialEq)]
pub struct Response {
    #[knuffel(argument)]
    pub name: String,
    #[knuffel(child, unwrap(argument, str))]
    pub accent: Option<AccentResponse>,
    #[knuffel(child, unwrap(argument, str))]
    pub attention: Option<AttentionResponse>,
    #[knuffel(child, unwrap(argument, str))]
    pub ping: Option<ImpulseResponse>,
    #[knuffel(child, unwrap(argument, str))]
    pub done: Option<ImpulseResponse>,
    #[knuffel(child, unwrap(argument, str))]
    pub error: Option<ImpulseResponse>,
    #[knuffel(child, unwrap(argument))]
    pub ring_inset: Option<FloatOrInt<0, 128>>,
    #[knuffel(child, unwrap(argument))]
    pub ring_width: Option<FloatOrInt<0, 128>>,
}

/// A fully resolved response block. `Default` is the spec's built-in
/// `default` block.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ResolvedResponse {
    pub accent: AccentResponse,
    pub attention: AttentionResponse,
    pub ping: ImpulseResponse,
    pub done: ImpulseResponse,
    pub error: ImpulseResponse,
    pub ring_inset: f64,
    pub ring_width: f64,
}

impl Default for ResolvedResponse {
    fn default() -> Self {
        Self {
            accent: AccentResponse::Ring,
            attention: AttentionResponse::RimOrbit,
            ping: ImpulseResponse::Ripple,
            done: ImpulseResponse::Sweep,
            error: ImpulseResponse::Flash,
            ring_inset: 6.,
            ring_width: 2.,
        }
    }
}

impl ResolvedResponse {
    fn with_overrides(base: Self, r: &Response) -> Self {
        Self {
            accent: r.accent.unwrap_or(base.accent),
            attention: r.attention.unwrap_or(base.attention),
            ping: r.ping.unwrap_or(base.ping),
            done: r.done.unwrap_or(base.done),
            error: r.error.unwrap_or(base.error),
            ring_inset: r.ring_inset.map_or(base.ring_inset, |x| x.0),
            ring_width: r.ring_width.map_or(base.ring_width, |x| x.0),
        }
    }

    pub fn attention_is_none(&self) -> bool {
        self.attention == AttentionResponse::None
    }

    /// Resolves an impulse kind to an opaque glass selector under the policy.
    pub fn impulse_selector(
        &self,
        kind: niri_ipc::ImpulseKind,
        policy: crate::SignalMotionPolicy,
    ) -> Option<u8> {
        use crate::SignalMotionPolicy as P;
        if policy == P::Off {
            return None;
        }
        let response = match kind {
            niri_ipc::ImpulseKind::Ping => self.ping,
            niri_ipc::ImpulseKind::Done => self.done,
            niri_ipc::ImpulseKind::Error => self.error,
        };
        let response = match (policy, response) {
            (P::Reduced, ImpulseResponse::Flash) => ImpulseResponse::Sweep,
            (_, r) => r,
        };
        match response {
            ImpulseResponse::None => None,
            r => Some(r as u8),
        }
    }
}
```

Change `Material` to carry `#[knuffel(children(name = "response"))] pub responses: Vec<Response>,` and `ResolvedMaterial` to add `pub responses: Vec<(String, ResolvedResponse)>` plus:

```rust
impl ResolvedMaterial {
    /// The named response, or `default`. Names are validated at parse time.
    pub fn response(&self, name: Option<&str>) -> ResolvedResponse {
        let name = name.unwrap_or("default");
        self.responses
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, r)| *r)
            .expect("response names are validated at parse time")
    }
}
```

In `Material::resolve`, build `responses`: if `self.responses` is empty, `vec![("default", ResolvedResponse::default())]`; otherwise resolve `default` first with `with_overrides(ResolvedResponse::default(), block)`, then every other block with `with_overrides(default_resolved, block)`.

Extend `Material::validate` after the offset rule:

```rust
        if !self.responses.is_empty() && !self.responses.iter().any(|r| r.name == "default") {
            return Err(format!("material {}: missing response \"default\"", self.name));
        }
        let mut seen = std::collections::HashSet::new();
        for r in &self.responses {
            if !seen.insert(r.name.as_str()) {
                return Err(format!("duplicate response: {}", r.name));
            }
        }
        let resolved = self.resolve();
        for (_, r) in &resolved.responses {
            if r.ring_inset + r.ring_width > bevel {
                return Err(String::from("ring-inset + ring-width must not exceed bevel"));
            }
        }
```

- [ ] **Step 4: Make `MaterialRef` a node with `response=`**

Replace the `DecodeScalar` impl with a `knuffel::Decode` impl for a node that has one string argument and an optional `response` property. Keep the ref recording: `MaterialRefs.root` becomes `Vec<(Spanned<Literal, S>, Option<String>)>` and `included` becomes `Vec<(String, Option<String>, String)>`.

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialRef {
    pub name: String,
    pub response: Option<String>,
}

impl<S: knuffel::traits::ErrorSpan> knuffel::Decode<S> for MaterialRef {
    fn decode_node(
        node: &knuffel::ast::SpannedNode<S>,
        ctx: &mut knuffel::decode::Context<S>,
    ) -> Result<Self, DecodeError<S>> {
        if let Some(type_name) = &node.type_name {
            ctx.emit_error(DecodeError::unexpected(type_name, "type name", "no type name expected for this node"));
        }
        let Some(arg) = node.arguments.first() else {
            return Err(DecodeError::missing(node, "material name argument"));
        };
        // `DecodeScalar::decode` runs `type_check`, which rejects a typed
        // scalar such as `(typed)"tg"`, before decoding the string.
        let name: String = knuffel::traits::DecodeScalar::decode(arg, ctx)?;
        if let Some(extra) = node.arguments.get(1) {
            ctx.emit_error(DecodeError::unexpected(&extra.literal, "argument", "unexpected argument"));
        }
        for child in node.children() {
            ctx.emit_error(DecodeError::unexpected(
                child,
                "node",
                format!("unexpected node `{}`", child.node_name.escape_default()),
            ));
        }
        let mut response: Option<String> = None;
        for (key, val) in &node.properties {
            match &***key {
                "response" => {
                    response = Some(knuffel::traits::DecodeScalar::decode(val, ctx)?);
                }
                other => {
                    return Err(DecodeError::unexpected(key, "property", format!("unexpected property `{other}`")));
                }
            }
        }
        // Record for post-include validation, exactly as before but with the response.
        let refs = ctx.get::<Rc<RefCell<MaterialRefs<S>>>>().expect("material refs must be set in the parse context").clone();
        let recursion = ctx.get::<crate::Recursion>().expect("recursion must be set in the parse context").0;
        let file = ctx.get::<crate::FileName>().expect("file name must be set in the parse context").0.clone();
        if recursion == 0 {
            refs.borrow_mut().root.push((arg.literal.clone(), response.clone()));
        } else {
            refs.borrow_mut().included.push((name.clone(), response.clone(), file));
        }
        Ok(Self { name, response })
    }
}
```

In `window_rule.rs` change the attribute on `material` to `#[knuffel(child)]`. Check how the existing tests write `material "frost"` inside window rules; the node form is identical for the user, only the decode changes.

Extend `validate_material_refs` so that, when the material is known and `response` is `Some(r)`, it checks `materials.iter().find(|m| m.name == name).map(|m| m.responses.is_empty() && r == "default" || m.responses.iter().any(|b| b.name == r))` and emits `format!("material {name}: unknown response: {r}")` otherwise.

- [ ] **Step 5: Fix every consumer of `MaterialRef(pub String)`**

`cargo build` and follow the errors: `src/window/mod.rs:103` (`ResolvedRules.material: Option<String>` becomes `Option<MaterialRef>`), the rule-resolution code that copies it, `src/layout/tile.rs:378` (`resolve_material(self.window.rules().material.as_ref(), &self.options)`), and any test in `src/tests/material.rs` that reads `.material` (compare `.as_ref().map(|m| m.name.as_str())`).

Update `resolve_material` in `src/layout/tile.rs`:

```rust
fn resolve_material(r: Option<&MaterialRef>, options: &Options) -> Option<ResolvedMaterial> {
    let r = r?;
    let mut material = options.materials.get(&r.name).cloned()?;
    material.glass.backdrop_blur = backdrop_blur_enabled(&material.glass, &options.blur);
    // Collapse to the one selected response so MaterialState carries exactly what it renders.
    let selected = material.response(r.response.as_deref());
    material.responses = vec![(String::from("default"), selected)];
    Some(material)
}
```

- [ ] **Step 6: Make a response-only reload reach the retained state**

`apply_resolved` in `src/render_helpers/material.rs:498` copies only changed glass fields when the name is unchanged, so editing a response under the same material name would leave `MaterialState` on the old response. Change the same-name arm to:

```rust
        (Some(state), Some(resolved)) if state.material.name == resolved.name => {
            let changed = state.material.glass != resolved.glass
                || state.material.responses != resolved.responses;
            if changed {
                state.material.glass = resolved.glass;
                state.material.responses = resolved.responses.clone();
                state.bump();
            }
            false
        }
```

Add a fixture test to `src/tests/material.rs`, next to the existing reload tests:

```rust
#[test]
fn response_only_reload_updates_retained_material() {
    let base = |attention: &str| {
        config(&format!(
            r##"
            material "tg" {{
                glass {{}}
                response "default" {{ attention "{attention}" }}
            }}
            window-rule {{ material "tg" }}
            "##
        ))
    };
    let mut f = Fixture::with_config(base("rim-orbit"));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_window(&mut f, id, "a");

    f.niri_state().reload_config(Ok(base("ring-pulse")));
    f.niri_state().refresh_and_flush_clients();

    let response = f
        .niri()
        .layout
        .workspaces()
        .flat_map(|ws| ws.tiles())
        .find_map(|tile| tile.material().map(|m| m.material().response(None)))
        .unwrap();
    assert_eq!(response.attention, niri_config::AttentionResponse::RingPulse);
}
```

`State::reload_config(&mut self, config: Result<Config, ()>)` is at `src/niri.rs:1421`. Add a `Tile::material(&self) -> Option<&MaterialState>` accessor if one does not exist.

- [ ] **Step 7: Run the tests**

Run: `cargo test -p niri-config` then `cargo test --bin niri material`
Expected: all PASS.

- [ ] **Step 8: Commit**

```bash
tasks done material-f810f4 "feat(config): add material response blocks and response references"
git add niri-config/src src/window src/layout/tile.rs src/render_helpers/material.rs src/tests tasks
git commit -m "feat(config): add material response blocks and response references"
```

---

### Task 4: The pure signal store

**Files:**
- Create: `src/window/signal.rs`
- Modify: `src/window/mod.rs` (add `pub mod signal;`)
- Test: inline `#[cfg(test)]` module in `src/window/signal.rs`

**Interfaces:**
- Produces (all in `crate::window::signal`):
  - Constants `MAX_EXTERNAL_SLOTS = 16`, `MAX_SOURCE_LEN = 64`, `MAX_TAG_LEN = 256`, `MAX_IMPULSES = 4`, `MAX_TTL = Duration::from_secs(86_400)`, `IMPULSE_LIFETIME = Duration::from_millis(1500)`, `NATIVE_SOURCE = "niri"`.
  - `Slot`, `Expiry`, `Impulse`, `SetSlot`, `SignalError`, `Folded`, `WindowSignals`.
  - `WindowSignals::set(&mut self, source: &str, set: SetSlot, now: Duration) -> Result<(), SignalError>`
  - `WindowSignals::pulse(&mut self, source: &str, kind: ImpulseKind, accent: Option<Color>, now: Duration) -> Result<(), SignalError>`
  - `WindowSignals::clear(&mut self, source: &str) -> Result<(), SignalError>`
  - `WindowSignals::set_native_urgent(&mut self, urgent: bool, now: Duration) -> bool` (returns whether anything changed)
  - `WindowSignals::on_focus(&mut self) -> bool`
  - `WindowSignals::advance(&mut self, now: Duration) -> bool` (applies expiries, prunes impulses)
  - `WindowSignals::next_deadline(&self) -> Option<Duration>`
  - `WindowSignals::fold(&self, now: Duration) -> Option<Folded>` (time-aware: applies any elapsed expiry and drops expired impulses in the result without mutating; `advance` is what mutates)
  - `parse_accent(s: &str) -> Result<Color, SignalError>` and `accent_hex(c: Color) -> String` (normalized `#rrggbb`).
- Consumes: `niri_ipc::{SignalLevel, SignalMotion, ImpulseKind}`, `niri_config::Color`.

- [ ] **Step 1: Write the failing tests**

Create `src/window/signal.rs` with the tests first:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use niri_ipc::{ImpulseKind, SignalLevel as L, SignalMotion as M};

    fn ms(v: u64) -> Duration {
        Duration::from_millis(v)
    }

    fn set(level: L, motion: M) -> SetSlot {
        SetSlot { accent: None, level, motion, tag: None, expiry: None, until_focus: false }
    }

    #[test]
    fn level_is_max_and_motion_follows_winner() {
        let mut s = WindowSignals::default();
        s.set("a", SetSlot { motion: M::Breathe, ..set(L::Active, M::Breathe) }, ms(1)).unwrap();
        s.set("b", set(L::Demand, M::Flash), ms(2)).unwrap();
        let f = s.fold(ms(2)).unwrap();
        assert_eq!(f.level, L::Demand);
        assert_eq!(f.motion, M::Flash);
        assert_eq!(f.sources, vec!["a", "b"]);
    }

    #[test]
    fn ties_go_to_latest_written() {
        let mut s = WindowSignals::default();
        s.set("a", set(L::Notice, M::Breathe), ms(1)).unwrap();
        s.set("b", set(L::Notice, M::Pulse), ms(2)).unwrap();
        assert_eq!(s.fold(ms(2)).unwrap().motion, M::Pulse);
    }

    #[test]
    fn accent_and_tag_select_independently() {
        let mut s = WindowSignals::default();
        let accent = parse_accent("#e5a33c").unwrap();
        s.set("a", SetSlot { accent: Some(accent), ..set(L::Quiet, M::Static) }, ms(1)).unwrap();
        s.set("b", SetSlot { tag: Some(String::from("t")), ..set(L::Demand, M::Pulse) }, ms(2)).unwrap();
        let f = s.fold(ms(2)).unwrap();
        assert_eq!(f.accent, Some(accent));
        assert_eq!(f.tag.as_deref(), Some("t"));
    }

    #[test]
    fn native_slot_exists_iff_urgent_and_pings_on_rise() {
        let mut s = WindowSignals::default();
        assert!(s.set_native_urgent(true, ms(5)));
        let f = s.fold(ms(5)).unwrap();
        assert_eq!((f.level, f.motion), (L::Demand, M::Pulse));
        assert_eq!(f.impulses.len(), 1);
        assert_eq!(f.impulses[0].kind, ImpulseKind::Ping);
        assert!(s.set_native_urgent(false, ms(6)));
        assert!(s.fold(ms(6)).is_none());
    }

    #[test]
    fn external_requests_cannot_touch_the_native_slot() {
        let mut s = WindowSignals::default();
        s.set_native_urgent(true, ms(0));
        assert_eq!(s.pulse("niri", ImpulseKind::Done, None, ms(1)), Err(SignalError::ReservedSource));
        assert_eq!(s.clear("niri"), Err(SignalError::ReservedSource));
        assert_eq!(s.fold(ms(1)).unwrap().sources, vec!["niri"]);
    }

    #[test]
    fn reserved_source_and_bounds_are_rejected_without_mutation() {
        let mut s = WindowSignals::default();
        assert_eq!(s.set("niri", set(L::Quiet, M::Static), ms(1)), Err(SignalError::ReservedSource));
        let long = "x".repeat(MAX_SOURCE_LEN + 1);
        assert_eq!(s.set(&long, set(L::Quiet, M::Static), ms(1)), Err(SignalError::SourceTooLong));
        let tag = Some("t".repeat(MAX_TAG_LEN + 1));
        assert_eq!(s.set("a", SetSlot { tag, ..set(L::Quiet, M::Static) }, ms(1)), Err(SignalError::TagTooLong));
        for i in 0..MAX_EXTERNAL_SLOTS {
            s.set(&format!("s{i}"), set(L::Quiet, M::Static), ms(i as u64)).unwrap();
        }
        assert_eq!(s.set("one-more", set(L::Quiet, M::Static), ms(99)), Err(SignalError::TooManySlots));
        assert_eq!(s.fold(ms(99)).unwrap().sources.len(), MAX_EXTERNAL_SLOTS);
    }

    #[test]
    fn ttl_is_capped_and_after_requires_ttl() {
        let mut s = WindowSignals::default();
        let too_long = Some(Expiry { at: MAX_TTL + ms(1) + ms(1), after_level: L::Quiet, after_motion: M::Static });
        assert_eq!(
            SetSlot::validate_ttl(Some(MAX_TTL + ms(1)), Some(L::Quiet), None),
            Err(SignalError::TtlTooLong)
        );
        assert_eq!(SetSlot::validate_ttl(None, Some(L::Quiet), None), Err(SignalError::AfterWithoutTtl));
        assert!(SetSlot::validate_ttl(Some(ms(10)), None, None).is_err());
        let _ = too_long;
        assert!(s.set("a", set(L::Quiet, M::Static), ms(1)).is_ok());
    }

    #[test]
    fn slot_decays_to_after_and_keeps_accent() {
        let mut s = WindowSignals::default();
        let accent = parse_accent("#112233").unwrap();
        let expiry = Some(Expiry { at: ms(1000), after_level: L::Quiet, after_motion: M::Breathe });
        s.set("a", SetSlot { accent: Some(accent), expiry, ..set(L::Demand, M::Pulse) }, ms(0)).unwrap();
        assert_eq!(s.next_deadline(), Some(ms(1000)));
        // Time-aware fold sees the decay before any timer runs `advance`.
        assert_eq!(s.fold(ms(999)).unwrap().level, L::Demand);
        assert_eq!(s.fold(ms(1000)).unwrap().level, L::Quiet);
        assert!(!s.advance(ms(999)));
        assert!(s.advance(ms(1000)));
        let f = s.fold(ms(1000)).unwrap();
        assert_eq!((f.level, f.motion, f.accent), (L::Quiet, M::Breathe, Some(accent)));
        assert_eq!(s.next_deadline(), None);
    }

    #[test]
    fn expiry_then_focus_demotes_once_in_either_order() {
        let expiry = Some(Expiry { at: ms(1000), after_level: L::Notice, after_motion: M::Breathe });
        // Expiry first, then focus: focus is a no-op.
        let mut s = WindowSignals::default();
        s.set("a", SetSlot { until_focus: true, expiry, ..set(L::Demand, M::Pulse) }, ms(0)).unwrap();
        assert!(s.advance(ms(1000)));
        assert!(!s.on_focus());
        let f = s.fold(ms(1000)).unwrap();
        assert_eq!((f.level, f.motion), (L::Notice, M::Breathe));
        // Focus first, then the old deadline: expiry is gone, nothing changes.
        let mut s = WindowSignals::default();
        s.set("a", SetSlot { until_focus: true, expiry, ..set(L::Demand, M::Pulse) }, ms(0)).unwrap();
        assert!(s.on_focus());
        assert_eq!(s.next_deadline(), None);
        assert!(!s.advance(ms(1000)));
        let f = s.fold(ms(1000)).unwrap();
        assert_eq!((f.level, f.motion), (L::Notice, M::Breathe));
    }

    #[test]
    fn until_focus_demotes_only_asking_slots() {
        let mut s = WindowSignals::default();
        s.set("a", SetSlot { until_focus: true, ..set(L::Demand, M::Pulse) }, ms(0)).unwrap();
        s.set("b", set(L::Notice, M::Breathe), ms(1)).unwrap();
        assert!(s.on_focus());
        let f = s.fold(ms(2)).unwrap();
        assert_eq!((f.level, f.motion), (L::Notice, M::Breathe));
    }

    #[test]
    fn pulse_requires_slot_and_fifth_evicts_oldest() {
        let mut s = WindowSignals::default();
        assert_eq!(s.pulse("a", ImpulseKind::Done, None, ms(0)), Err(SignalError::UnknownSource));
        s.set("a", set(L::Quiet, M::Static), ms(0)).unwrap();
        for i in 0..5u64 {
            s.pulse("a", ImpulseKind::Done, None, ms(i)).unwrap();
        }
        let f = s.fold(ms(4)).unwrap();
        assert_eq!(f.impulses.len(), MAX_IMPULSES);
        assert_eq!(f.impulses[0].at, ms(1));
        assert_eq!(s.next_deadline(), Some(ms(1) + IMPULSE_LIFETIME));
        assert_eq!(s.fold(ms(1) + IMPULSE_LIFETIME).unwrap().impulses.len(), 3);
        assert!(s.advance(ms(1) + IMPULSE_LIFETIME));
        assert_eq!(s.fold(ms(1) + IMPULSE_LIFETIME).unwrap().impulses.len(), 3);
    }

    #[test]
    fn clear_drops_slot_and_its_impulses() {
        let mut s = WindowSignals::default();
        s.set("a", set(L::Quiet, M::Static), ms(0)).unwrap();
        s.pulse("a", ImpulseKind::Ping, None, ms(0)).unwrap();
        assert_eq!(s.clear("b"), Err(SignalError::UnknownSource));
        s.clear("a").unwrap();
        assert!(s.fold(ms(0)).is_none());
    }

    #[test]
    fn accent_parses_and_normalizes() {
        assert_eq!(accent_hex(parse_accent("#E5A33Cff").unwrap()), "#e5a33c");
        assert_eq!(parse_accent("nope"), Err(SignalError::BadColor));
    }
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test --bin niri window::signal`
Expected: compile errors.

- [ ] **Step 3: Implement the store**

Above the tests in `src/window/signal.rs`:

```rust
//! Per-window signal store (design §1). Pure: no timers, no renderer.

use std::collections::VecDeque;
use std::time::Duration;

use niri_config::Color;
use niri_ipc::{ImpulseKind, SignalLevel, SignalMotion};

pub const MAX_EXTERNAL_SLOTS: usize = 16;
pub const MAX_SOURCE_LEN: usize = 64;
pub const MAX_TAG_LEN: usize = 256;
pub const MAX_IMPULSES: usize = 4;
pub const MAX_TTL: Duration = Duration::from_secs(86_400);
pub const IMPULSE_LIFETIME: Duration = Duration::from_millis(1500);
pub const NATIVE_SOURCE: &str = "niri";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Expiry {
    pub at: Duration,
    pub after_level: SignalLevel,
    pub after_motion: SignalMotion,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Slot {
    pub accent: Option<Color>,
    pub level: SignalLevel,
    pub motion: SignalMotion,
    pub tag: Option<String>,
    pub expires: Option<Expiry>,
    pub until_focus: bool,
    pub written_at: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Impulse {
    pub source: String,
    pub kind: ImpulseKind,
    pub accent: Option<Color>,
    pub at: Duration,
}

impl Impulse {
    pub fn expires_at(&self) -> Duration {
        self.at + IMPULSE_LIFETIME
    }
}

/// Validated input for `set`.
#[derive(Debug, Clone, PartialEq)]
pub struct SetSlot {
    pub accent: Option<Color>,
    pub level: SignalLevel,
    pub motion: SignalMotion,
    pub tag: Option<String>,
    pub expiry: Option<Expiry>,
    pub until_focus: bool,
}

impl SetSlot {
    /// Turns the IPC ttl/after flags into an `Expiry` relative to `now`.
    pub fn expiry_from_ttl(
        ttl: Option<Duration>,
        after_level: Option<SignalLevel>,
        after_motion: Option<SignalMotion>,
        now: Duration,
    ) -> Result<Option<Expiry>, SignalError> {
        Self::validate_ttl(ttl, after_level, after_motion)?;
        Ok(ttl.map(|ttl| Expiry {
            at: now + ttl,
            after_level: after_level.unwrap_or(SignalLevel::Quiet),
            after_motion: after_motion.unwrap_or(SignalMotion::Static),
        }))
    }

    pub fn validate_ttl(
        ttl: Option<Duration>,
        after_level: Option<SignalLevel>,
        after_motion: Option<SignalMotion>,
    ) -> Result<(), SignalError> {
        match ttl {
            Some(ttl) if ttl > MAX_TTL => Err(SignalError::TtlTooLong),
            Some(_) if after_level.is_none() && after_motion.is_none() => Err(SignalError::TtlWithoutAfter),
            None if after_level.is_some() || after_motion.is_some() => Err(SignalError::AfterWithoutTtl),
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalError {
    ReservedSource,
    SourceTooLong,
    TagTooLong,
    TooManySlots,
    UnknownSource,
    TtlTooLong,
    TtlWithoutAfter,
    AfterWithoutTtl,
    BadColor,
}

impl std::fmt::Display for SignalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::ReservedSource => "source name \"niri\" is reserved",
            Self::SourceTooLong => "source name longer than 64 bytes",
            Self::TagTooLong => "tag longer than 256 bytes",
            Self::TooManySlots => "window already has 16 signal sources",
            Self::UnknownSource => "no signal slot for this source on this window",
            Self::TtlTooLong => "ttl-ms exceeds 86400000",
            Self::TtlWithoutAfter => "ttl-ms requires after-level or after-motion",
            Self::AfterWithoutTtl => "after-level and after-motion require ttl-ms",
            Self::BadColor => "accent must be #rrggbb or #rrggbbaa",
        };
        f.write_str(s)
    }
}

/// The fold result (design §1).
#[derive(Debug, Clone, PartialEq)]
pub struct Folded {
    pub level: SignalLevel,
    pub motion: SignalMotion,
    pub accent: Option<Color>,
    pub tag: Option<String>,
    pub sources: Vec<String>,
    pub impulses: Vec<Impulse>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct WindowSignals {
    /// Kept sorted by (written_at, name).
    slots: Vec<(String, Slot)>,
    impulses: VecDeque<Impulse>,
}

pub fn parse_accent(s: &str) -> Result<Color, SignalError> {
    let hex = s.strip_prefix('#').ok_or(SignalError::BadColor)?;
    if !(hex.len() == 6 || hex.len() == 8) || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(SignalError::BadColor);
    }
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| SignalError::BadColor);
    Ok(Color::from_rgba8_unpremul(byte(0)?, byte(2)?, byte(4)?, 0xff))
}

pub fn accent_hex(c: Color) -> String {
    let q = |v: f32| (v.clamp(0., 1.) * 255.).round() as u8;
    format!("#{:02x}{:02x}{:02x}", q(c.r), q(c.g), q(c.b))
}

impl WindowSignals {
    fn sort(&mut self) {
        self.slots.sort_by(|(an, a), (bn, b)| a.written_at.cmp(&b.written_at).then_with(|| an.cmp(bn)));
    }

    fn slot_mut(&mut self, source: &str) -> Option<&mut Slot> {
        self.slots.iter_mut().find(|(n, _)| n == source).map(|(_, s)| s)
    }

    fn validate_external(source: &str, tag: Option<&str>) -> Result<(), SignalError> {
        if source == NATIVE_SOURCE {
            return Err(SignalError::ReservedSource);
        }
        if source.len() > MAX_SOURCE_LEN {
            return Err(SignalError::SourceTooLong);
        }
        if tag.is_some_and(|t| t.len() > MAX_TAG_LEN) {
            return Err(SignalError::TagTooLong);
        }
        Ok(())
    }

    pub fn set(&mut self, source: &str, set: SetSlot, now: Duration) -> Result<(), SignalError> {
        Self::validate_external(source, set.tag.as_deref())?;
        let exists = self.slots.iter().any(|(n, _)| n == source);
        let external = self.slots.iter().filter(|(n, _)| n != NATIVE_SOURCE).count();
        if !exists && external >= MAX_EXTERNAL_SLOTS {
            return Err(SignalError::TooManySlots);
        }
        let slot = Slot {
            accent: set.accent,
            level: set.level,
            motion: set.motion,
            tag: set.tag,
            expires: set.expiry,
            until_focus: set.until_focus,
            written_at: now,
        };
        match self.slot_mut(source) {
            Some(existing) => *existing = slot,
            None => self.slots.push((source.to_owned(), slot)),
        }
        self.sort();
        Ok(())
    }

    pub fn pulse(
        &mut self,
        source: &str,
        kind: ImpulseKind,
        accent: Option<Color>,
        now: Duration,
    ) -> Result<(), SignalError> {
        Self::validate_external(source, None)?;
        if !self.slots.iter().any(|(n, _)| n == source) {
            return Err(SignalError::UnknownSource);
        }
        self.push_impulse(Impulse { source: source.to_owned(), kind, accent, at: now });
        Ok(())
    }

    fn push_impulse(&mut self, impulse: Impulse) {
        if self.impulses.len() >= MAX_IMPULSES {
            self.impulses.pop_front();
        }
        self.impulses.push_back(impulse);
    }

    pub fn clear(&mut self, source: &str) -> Result<(), SignalError> {
        Self::validate_external(source, None)?;
        let before = self.slots.len();
        self.slots.retain(|(n, _)| n != source);
        if self.slots.len() == before {
            return Err(SignalError::UnknownSource);
        }
        self.impulses.retain(|i| i.source != source);
        Ok(())
    }

    /// Native slot exists iff urgent (design §1).
    pub fn set_native_urgent(&mut self, urgent: bool, now: Duration) -> bool {
        let exists = self.slots.iter().any(|(n, _)| n == NATIVE_SOURCE);
        match (urgent, exists) {
            (true, false) => {
                self.slots.push((
                    NATIVE_SOURCE.to_owned(),
                    Slot {
                        accent: None,
                        level: SignalLevel::Demand,
                        motion: SignalMotion::Pulse,
                        tag: None,
                        expires: None,
                        until_focus: false,
                        written_at: now,
                    },
                ));
                self.sort();
                self.push_impulse(Impulse {
                    source: NATIVE_SOURCE.to_owned(),
                    kind: ImpulseKind::Ping,
                    accent: None,
                    at: now,
                });
                true
            }
            (false, true) => {
                self.slots.retain(|(n, _)| n != NATIVE_SOURCE);
                self.impulses.retain(|i| i.source != NATIVE_SOURCE);
                true
            }
            _ => false,
        }
    }

    pub fn on_focus(&mut self) -> bool {
        let mut changed = false;
        for (_, slot) in &mut self.slots {
            if slot.until_focus {
                let (level, motion) = slot
                    .expires
                    .map_or((SignalLevel::Quiet, SignalMotion::Static), |e| (e.after_level, e.after_motion));
                changed |= slot.level != level || slot.motion != motion || slot.expires.is_some();
                slot.level = level;
                slot.motion = motion;
                slot.expires = None;
                slot.until_focus = false;
            }
        }
        changed
    }

    pub fn advance(&mut self, now: Duration) -> bool {
        let mut changed = false;
        for (_, slot) in &mut self.slots {
            if let Some(e) = slot.expires {
                if now >= e.at {
                    slot.level = e.after_level;
                    slot.motion = e.after_motion;
                    slot.expires = None;
                    // The after pair is the final state; a later focus must
                    // not demote it a second time to Quiet/Static.
                    slot.until_focus = false;
                    changed = true;
                }
            }
        }
        let before = self.impulses.len();
        self.impulses.retain(|i| now < i.expires_at());
        changed |= self.impulses.len() != before;
        changed
    }

    pub fn next_deadline(&self) -> Option<Duration> {
        let slots = self.slots.iter().filter_map(|(_, s)| s.expires.map(|e| e.at));
        let impulses = self.impulses.iter().map(Impulse::expires_at);
        slots.chain(impulses).min()
    }

    /// Time-aware fold: a slot whose expiry has passed reads as its `after`
    /// pair even if no timer has run `advance` yet, and expired impulses are
    /// omitted. Reads therefore never depend on timer delivery.
    pub fn fold(&self, now: Duration) -> Option<Folded> {
        if self.slots.is_empty() {
            return None;
        }
        let effective = |s: &Slot| match s.expires {
            Some(e) if now >= e.at => (e.after_level, e.after_motion),
            _ => (s.level, s.motion),
        };
        let max_level = self.slots.iter().map(|(_, s)| effective(s).0).max()?;
        // Latest written among the tied slots wins; slots are sorted ascending.
        let (_, winner) = self.slots.iter().rev().find(|(_, s)| effective(s).0 == max_level)?;
        let accent = winner
            .accent
            .or_else(|| self.slots.iter().find_map(|(_, s)| s.accent));
        let tag = winner
            .tag
            .clone()
            .or_else(|| self.slots.iter().find_map(|(_, s)| s.tag.clone()));
        Some(Folded {
            level: max_level,
            motion: effective(winner).1,
            accent,
            tag,
            sources: self.slots.iter().map(|(n, _)| n.clone()).collect(),
            impulses: self.impulses.iter().filter(|i| now < i.expires_at()).cloned().collect(),
        })
    }
}
```

Add `pub mod signal;` to `src/window/mod.rs`.

- [ ] **Step 4: Run the tests**

Run: `cargo test --bin niri window::signal`
Expected: all PASS. If `Color` does not implement `Eq`, keep the `PartialEq` derives as written and compare with `assert_eq!` (it only needs `PartialEq`).

- [ ] **Step 5: Commit**

```bash
tasks done material-5875b2 "feat(render): add the per-window signal store"
git add src/window/signal.rs src/window/mod.rs tasks
git commit -m "feat(render): add the per-window signal store"
```

---

### Task 5: Attach signals to windows, urgency, focus, and rule matching

**Files:**
- Modify: `src/window/mapped.rs` (struct at ~89, `new` at ~288, `set_is_focused` at 390, `set_urgent` at 601)
- Modify: `src/window/mod.rs` (`window_matches` at ~403)
- Modify: `src/layout/mod.rs` (`LayoutElement` trait at ~248, `Options` at ~394, options construction at ~656)
- Modify: `src/layout/tile.rs` (`refresh_material`; called from wherever rule recomputation already triggers it)
- Create: `src/tests/signal.rs`; register `mod signal;` in `src/tests/mod.rs`

**Interfaces:**
- Produces: `Mapped::signals(&self) -> &WindowSignals`, `Mapped::signals_mut(&mut self) -> &mut WindowSignals` (callers must call `Mapped::signal_changed(&mut self)` afterwards, which sets `need_to_recompute_rules` and `signal_deadline_dirty`), `Mapped::take_signal_deadline_dirty(&mut self) -> bool`, `LayoutElement::signal(&self, now: Duration) -> Option<Folded>`, `Options.signal: niri_config::Signal`, `Layout::find_window_mut_by(&mut self, pred: impl FnMut(&W) -> bool) -> Option<&mut W>` covering the interactively moved window.
- Consumes: `WindowSignals` (Task 4), `Match.signal_source/tag` (Task 2).

- [ ] **Step 1: Write the failing fixture test**

`src/tests/signal.rs`:

```rust
use niri_config::Config;

use super::client::ClientId;
use super::*;
use crate::utils::get_monotonic_time;
use crate::utils::with_toplevel_role;

fn config(text: &str) -> Config {
    Config::parse_mem(text).unwrap()
}

fn open_window(f: &mut Fixture, id: ClientId, title: &str) -> wayland_client::protocol::wl_surface::WlSurface {
    let window = f.client(id).create_window();
    let surface = window.surface.clone();
    window.set_title(title);
    window.commit();
    f.roundtrip(id);
    let window = f.client(id).window(&surface);
    window.attach_new_buffer();
    window.ack_last_and_commit();
    f.double_roundtrip(id);
    surface
}

fn material_of(f: &mut Fixture, title: &str) -> Option<String> {
    f.niri()
        .layout
        .windows()
        .find(|(_, m)| with_toplevel_role(m.toplevel(), |r| r.title.as_deref() == Some(title)))
        .and_then(|(_, m)| m.rules().material.as_ref().map(|r| r.name.clone()))
}

#[test]
fn urgency_creates_native_slot_and_matches_signal_source() {
    let mut f = Fixture::with_config(config(
        r##"
        material "calm" { glass {} }
        material "alarm" { glass {} }
        window-rule { material "calm" }
        window-rule {
            match signal-source="^niri$"
            material "alarm"
        }
        "##,
    ));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_window(&mut f, id, "a");
    open_window(&mut f, id, "b"); // "b" takes focus, so "a" can become urgent
    assert_eq!(material_of(&mut f, "a").as_deref(), Some("calm"));

    let now = get_monotonic_time();
    {
        let niri = f.niri();
        let (_, mapped) = niri
            .layout
            .windows_mut()
            .find(|(_, m)| with_toplevel_role(m.toplevel(), |r| r.title.as_deref() == Some("a")))
            .unwrap();
        mapped.set_urgent(true);
        assert!(mapped.signals().fold(now).is_some());
    }
    f.niri_state().refresh_and_flush_clients();
    assert_eq!(material_of(&mut f, "a").as_deref(), Some("alarm"));
}
```

`State::refresh_and_flush_clients` (`src/niri.rs:754`) runs `State::refresh`, which calls `refresh_window_rules`, so the rule with the signal match re-resolves.

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test --bin niri urgency_creates_native_slot`
Expected: compile error, `signals` not found.

- [ ] **Step 3: Store signals on `Mapped`**

In `src/window/mapped.rs`, add the field and accessors:

```rust
    /// Per-window signal store (design §1).
    signals: crate::window::signal::WindowSignals,
    /// Set by every signal mutation; `Niri::refresh_signal_deadlines` takes
    /// it and re-arms the window's deadline timer.
    signal_deadline_dirty: bool,
```

Initialize with `signals: Default::default()` in `Mapped::new`. Add:

```rust
    pub fn signals(&self) -> &crate::window::signal::WindowSignals {
        &self.signals
    }

    pub fn signals_mut(&mut self) -> &mut crate::window::signal::WindowSignals {
        &mut self.signals
    }

    /// Call after any signal mutation so rules with signal matches re-resolve
    /// and the deadline timer is reconciled on the next refresh.
    pub fn signal_changed(&mut self) {
        self.need_to_recompute_rules = true;
        self.signal_deadline_dirty = true;
    }

    pub fn take_signal_deadline_dirty(&mut self) -> bool {
        std::mem::take(&mut self.signal_deadline_dirty)
    }
```

In `set_urgent`, after computing `changed`, add:

```rust
        if changed {
            let now = crate::utils::get_monotonic_time();
            if self.signals.set_native_urgent(urgent, now) {
                self.signal_deadline_dirty = true;
            }
        }
```

In `set_is_focused`, after `self.is_urgent = false;`:

```rust
        let now = crate::utils::get_monotonic_time();
        let mut changed = self.signals.set_native_urgent(false, now);
        if is_focused {
            changed |= self.signals.on_focus();
        }
        self.signal_deadline_dirty |= changed;
```

`need_to_recompute_rules = true` is already set on both paths.

- [ ] **Step 4: Match on signal source and tag**

In `src/window/mod.rs::window_matches`, after the `is_urgent` block:

```rust
    if m.signal_source.is_some() || m.signal_tag.is_some() {
        let Some(folded) = window.signal(crate::utils::get_monotonic_time()) else {
            return false;
        };
        if let Some(re) = &m.signal_source {
            if !folded.sources.iter().any(|s| re.0.is_match(s)) {
                return false;
            }
        }
        if let Some(re) = &m.signal_tag {
            match &folded.tag {
                Some(tag) if re.0.is_match(tag) => {}
                _ => return false,
            }
        }
    }
```

`WindowRef` is the type passed to `window_matches`; add `fn signal(&self, now: Duration) -> Option<Folded>` to whatever trait or impl gives it `is_urgent()` (see `src/window/mod.rs:187`), delegating to `mapped.signals().fold(now)`.

- [ ] **Step 5: Expose the fold to the layout**

In `src/layout/mod.rs`, in `LayoutElement` after `fn is_urgent(&self) -> bool;`:

```rust
    /// The folded window signal at `now`, if any source has written one.
    fn signal(&self, now: Duration) -> Option<crate::window::signal::Folded>;
```

Implement for `Mapped` (`self.signals.fold(now)`) and for the test window type in `src/layout/mod.rs` tests (`None`). Add `pub signal: niri_config::Signal` to `Options` and `signal: config.signal` where options are built from config (line ~666).

Add a lookup on `Layout` next to `with_windows_mut` (`src/layout/mod.rs:1681`) that covers the interactively moved window, which lives outside every workspace while a drag is in progress:

```rust
    /// Finds a window by predicate, including one under interactive move,
    /// which lives outside every workspace while the drag is in progress.
    /// Predicate-based because `LayoutElement::Id` is `Window` for `Mapped`
    /// while callers hold a `MappedId` or an IPC id.
    pub fn find_window_mut_by(&mut self, mut pred: impl FnMut(&W) -> bool) -> Option<&mut W> {
        if let Some(InteractiveMoveState::Moving(move_)) = &mut self.interactive_move {
            if pred(move_.tile.window()) {
                return Some(move_.tile.window_mut());
            }
        }
        self.workspaces_mut()
            .find_map(|ws| ws.windows_mut().find(|w| pred(w)))
    }
```

Add a layout unit test in `src/layout/mod.rs` tests: open a window through the existing test harness, begin an interactive move on it with `interactive_move_begin` (line 3806) and one `interactive_move_update` so it enters `Moving`, then with `let id = window.id().clone();` assert `find_window_mut_by(|w| w.id() == &id).is_some()` while `workspaces_mut().flat_map(|ws| ws.windows_mut())` no longer yields it.

- [ ] **Step 6: Run the test**

Run: `cargo test --bin niri signal`
Expected: PASS for the fixture test and the store tests.

- [ ] **Step 7: Commit**

```bash
tasks done material-25247f "feat(render): attach signals to windows and match on source and tag"
git add src/window src/layout src/tests tasks
git commit -m "feat(render): attach signals to windows and match on source and tag"
```

---

### Task 6: Mutation entry points, deadline timers, IPC server, and CLI

**Files:**
- Modify: `niri-ipc/src/lib.rs` (the three `Request` variants after `PickColor`)
- Modify: `src/niri.rs` (`Niri` struct fields, new `impl Niri` methods, `State::refresh` at 793; timer precedent at line 1285)
- Modify: `src/handlers/compositor.rs` (~297) and `src/handlers/xdg_shell.rs` (~862): unmap cancellation calls
- Modify: `src/ipc/server.rs` (`Request::PickColor` arm at ~372, `make_ipc_window` at 513, window diff at ~760)
- Modify: `src/cli.rs` (`Msg` at 63), `src/ipc/client.rs` (request mapping at ~38, response handling at ~300)
- Test: `src/tests/signal.rs`

**Interfaces:**
- Produces: `Niri::set_window_signal(&mut self, req: SetWindowSignalArgs) -> Result<(), String>`, `Niri::pulse_window_signal(&mut self, id: u64, source: &str, kind: ImpulseKind, accent: Option<&str>) -> Result<(), String>`, `Niri::clear_window_signal(&mut self, id: u64, source: &str) -> Result<(), String>`, `Niri::refresh_signal_deadlines(&mut self)` (called from `State::refresh` right after `refresh_window_rules`), `Niri::rearm_signal_deadline(&mut self, id: MappedId)`, `to_ipc_signal(folded: &Folded) -> niri_ipc::Signal`.
- Consumes: Tasks 1, 4, 5.

- [ ] **Step 1: Write the failing tests**

Append to `src/tests/signal.rs`:

```rust
fn window_id(f: &mut Fixture, title: &str) -> u64 {
    f.niri()
        .layout
        .windows()
        .find(|(_, m)| with_toplevel_role(m.toplevel(), |r| r.title.as_deref() == Some(title)))
        .map(|(_, m)| m.id().get())
        .unwrap()
}

#[test]
fn ipc_entry_points_validate_and_mutate() {
    use crate::niri::SetWindowSignalArgs;
    use niri_ipc::{ImpulseKind, SignalLevel, SignalMotion};

    let mut f = Fixture::with_config(config(""));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_window(&mut f, id, "a");
    let wid = window_id(&mut f, "a");

    let args = |source: &str| SetWindowSignalArgs {
        id: wid,
        source: source.to_owned(),
        accent: Some(String::from("#e5a33c")),
        level: SignalLevel::Demand,
        motion: SignalMotion::Pulse,
        tag: Some(String::from("cats/ginger")),
        ttl_ms: Some(30_000),
        after_level: Some(SignalLevel::Quiet),
        after_motion: None,
        until_focus: true,
    };

    let niri = f.niri();
    assert!(niri.set_window_signal(SetWindowSignalArgs { id: 999, ..args("familiar") }).is_err());
    assert!(niri.set_window_signal(args("niri")).is_err());
    assert!(niri.pulse_window_signal(wid, "niri", ImpulseKind::Done, None).is_err());
    assert!(niri.clear_window_signal(wid, "niri").is_err());
    assert!(niri.pulse_window_signal(wid, "familiar", ImpulseKind::Done, None).is_err());
    niri.set_window_signal(args("familiar")).unwrap();
    assert!(niri.pulse_window_signal(wid, "familiar", ImpulseKind::Done, Some("zzz")).is_err());
    niri.pulse_window_signal(wid, "familiar", ImpulseKind::Done, None).unwrap();

    let now = get_monotonic_time();
    let (_, mapped) = niri.layout.windows().find(|(_, m)| m.id().get() == wid).unwrap();
    let signal = crate::ipc::server::to_ipc_signal(&mapped.signals().fold(now).unwrap());
    assert_eq!(signal.level, SignalLevel::Demand);
    assert_eq!(signal.accent.as_deref(), Some("#e5a33c"));
    assert_eq!(signal.sources, vec!["familiar"]);
    assert_eq!(signal.impulses.len(), 1);
    assert_eq!(signal.impulses[0].kind, ImpulseKind::Done);
    assert_eq!(
        signal.impulses[0].expires_at.secs * 1_000_000_000 + u64::from(signal.impulses[0].expires_at.nanos),
        signal.impulses[0].at.secs * 1_000_000_000 + u64::from(signal.impulses[0].at.nanos) + 1_500_000_000
    );

    niri.clear_window_signal(wid, "familiar").unwrap();
    assert!(niri.clear_window_signal(wid, "familiar").is_err());
    let (_, mapped) = niri.layout.windows().find(|(_, m)| m.id().get() == wid).unwrap();
    assert!(mapped.signals().fold(now).is_none());
}

#[test]
fn refresh_reconciles_deadline_timer_after_focus() {
    use crate::niri::SetWindowSignalArgs;
    use niri_ipc::{SignalLevel, SignalMotion};

    let mut f = Fixture::with_config(config(""));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_window(&mut f, id, "a");
    open_window(&mut f, id, "b");
    let wid = window_id(&mut f, "a");
    f.niri()
        .set_window_signal(SetWindowSignalArgs {
            id: wid, source: String::from("t"), accent: None,
            level: SignalLevel::Demand, motion: SignalMotion::Pulse, tag: None,
            ttl_ms: Some(5000), after_level: Some(SignalLevel::Quiet), after_motion: None,
            until_focus: true,
        })
        .unwrap();
    f.niri_state().refresh_and_flush_clients();
    let mapped_id = f.niri().layout.windows().find(|(_, m)| m.id().get() == wid).map(|(_, m)| m.id()).unwrap();
    assert!(f.niri().signal_deadlines.contains_key(&mapped_id), "ttl armed a deadline");

    // Focusing "a" demotes the until-focus slot and cancels its expiry.
    // `Mapped::is_focused` is written only by `State::update_keyboard_focus`
    // (see the idiom in src/tests/material.rs), so move focus, update it,
    // then refresh so `refresh_signal_deadlines` runs.
    f.niri().layout.focus_left();
    f.niri_state().update_keyboard_focus();
    f.niri_state().refresh_and_flush_clients();
    assert!(!f.niri().signal_deadlines.contains_key(&mapped_id), "focus reconciled the timer away");
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test --bin niri ipc_entry_points`
Expected: compile errors.

- [ ] **Step 3: Add the entry points and deadline timer on `Niri`**

In `src/niri.rs`, add to the `Niri` struct:

```rust
    /// One deadline timer per window with pending slot or impulse expiry (design §1).
    pub signal_deadlines: HashMap<MappedId, RegistrationToken>,
```

Initialize with `HashMap::new()`. Add the argument struct and methods:

```rust
#[derive(Debug, Clone)]
pub struct SetWindowSignalArgs {
    pub id: u64,
    pub source: String,
    pub accent: Option<String>,
    pub level: niri_ipc::SignalLevel,
    pub motion: niri_ipc::SignalMotion,
    pub tag: Option<String>,
    pub ttl_ms: Option<u32>,
    pub after_level: Option<niri_ipc::SignalLevel>,
    pub after_motion: Option<niri_ipc::SignalMotion>,
    pub until_focus: bool,
}

impl Niri {
    /// Finds a mapped window by IPC id, including one under interactive move.
    fn mapped_by_ipc_id(&mut self, id: u64) -> Result<&mut Mapped, String> {
        self.layout
            .find_window_mut_by(|m| m.id().get() == id)
            .ok_or_else(|| format!("no window with id {id}"))
    }

    /// Mutations only mark the window; the next `State::refresh` reconciles
    /// timers. One coalescible redraw request is queued per output so the
    /// tile re-evaluates its effective signal (`RedrawState::queue_redraw`
    /// folds it into an already queued frame). Whether anything then
    /// animates is decided by the tile, so a mutation never causes
    /// recurring redraws.
    fn after_signal_mutation(&mut self) {
        self.queue_redraw_all();
    }

    /// Re-arms the deadline timer of every window whose signals changed since
    /// the last refresh. Covers IPC writes, urgency, focus demotion, and
    /// timer-driven decay through one path.
    pub fn refresh_signal_deadlines(&mut self) {
        let mut dirty = Vec::new();
        self.layout.with_windows_mut(|mapped, _| {
            if mapped.take_signal_deadline_dirty() {
                dirty.push(mapped.id());
            }
        });
        for id in dirty {
            self.rearm_signal_deadline(id);
        }
    }

    pub fn set_window_signal(&mut self, a: SetWindowSignalArgs) -> Result<(), String> {
        use crate::window::signal::{parse_accent, SetSlot};
        let now = get_monotonic_time();
        let accent = a.accent.as_deref().map(parse_accent).transpose().map_err(|e| e.to_string())?;
        let ttl = a.ttl_ms.map(|ms| Duration::from_millis(u64::from(ms)));
        let expiry = SetSlot::expiry_from_ttl(ttl, a.after_level, a.after_motion, now)
            .map_err(|e| e.to_string())?;
        let set = SetSlot {
            accent,
            level: a.level,
            motion: a.motion,
            tag: a.tag.clone(),
            expiry,
            until_focus: a.until_focus,
        };
        let mapped = self.mapped_by_ipc_id(a.id)?;
        mapped.signals_mut().set(&a.source, set, now).map_err(|e| e.to_string())?;
        mapped.signal_changed();
        self.after_signal_mutation();
        Ok(())
    }

    pub fn pulse_window_signal(
        &mut self,
        id: u64,
        source: &str,
        kind: niri_ipc::ImpulseKind,
        accent: Option<&str>,
    ) -> Result<(), String> {
        use crate::window::signal::parse_accent;
        let now = get_monotonic_time();
        let accent = accent.map(parse_accent).transpose().map_err(|e| e.to_string())?;
        let mapped = self.mapped_by_ipc_id(id)?;
        mapped.signals_mut().pulse(source, kind, accent, now).map_err(|e| e.to_string())?;
        mapped.signal_changed();
        self.after_signal_mutation();
        Ok(())
    }

    pub fn clear_window_signal(&mut self, id: u64, source: &str) -> Result<(), String> {
        let mapped = self.mapped_by_ipc_id(id)?;
        mapped.signals_mut().clear(source).map_err(|e| e.to_string())?;
        mapped.signal_changed();
        self.after_signal_mutation();
        Ok(())
    }

    /// Replaces the window's deadline timer with one for its next expiry, or removes it.
    pub fn rearm_signal_deadline(&mut self, id: MappedId) {
        if let Some(token) = self.signal_deadlines.remove(&id) {
            self.event_loop.remove(token);
        }
        let deadline = self
            .layout
            .find_window_mut_by(|m| m.id() == id)
            .and_then(|w| w.signals().next_deadline());
        let Some(deadline) = deadline else { return };
        let delay = deadline.saturating_sub(get_monotonic_time());
        let timer = Timer::from_duration(delay);
        let token = self
            .event_loop
            .insert_source(timer, move |_, _, state| {
                state.niri.signal_deadline_fired(id);
                TimeoutAction::Drop
            })
            .unwrap();
        self.signal_deadlines.insert(id, token);
    }

    fn signal_deadline_fired(&mut self, id: MappedId) {
        self.signal_deadlines.remove(&id);
        let now = get_monotonic_time();
        // `find_window_mut_by` covers a window under interactive move, so a
        // deadline that fires mid-drag is never lost.
        let Some(w) = self.layout.find_window_mut_by(|m| m.id() == id) else { return };
        let changed = w.signals_mut().advance(now);
        // Always mark dirty so the refresh re-arms for the next deadline.
        w.signal_changed();
        if changed {
            self.queue_redraw_all();
        }
    }
}
```

Call `self.niri.refresh_signal_deadlines()` in `State::refresh` (`src/niri.rs:793`) immediately after `self.niri.refresh_window_rules()`. Add one cancellation helper on `Niri`:

```rust
    /// Drops a window's deadline timer; called from every unmap path.
    pub fn cancel_signal_deadline(&mut self, id: MappedId) {
        if let Some(token) = self.signal_deadlines.remove(&id) {
            self.event_loop.remove(token);
        }
    }
```

and call `self.niri.cancel_signal_deadline(id);` at both unmap sites, right before `self.niri.layout.remove_window(&window, transaction.clone())`: `src/handlers/compositor.rs:297` and `src/handlers/xdg_shell.rs:862`. The xdg-activation path and the urgent actions need no extra call: `set_urgent` sets the dirty flag and the refresh re-arms.

Append two fixture tests to `src/tests/signal.rs`:

```rust
#[test]
fn rewrite_replaces_the_deadline_and_close_cancels_it() {
    use crate::niri::SetWindowSignalArgs;
    use niri_ipc::{SignalLevel, SignalMotion};

    let mut f = Fixture::with_config(config(""));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    let surface = open_window(&mut f, id, "a");
    let wid = window_id(&mut f, "a");
    let args = |ttl: u32| SetWindowSignalArgs {
        id: wid, source: String::from("t"), accent: None,
        level: SignalLevel::Notice, motion: SignalMotion::Static, tag: None,
        ttl_ms: Some(ttl), after_level: Some(SignalLevel::Quiet), after_motion: None,
        until_focus: false,
    };
    f.niri().set_window_signal(args(5000)).unwrap();
    f.niri_state().refresh_and_flush_clients();
    f.niri().set_window_signal(args(9000)).unwrap();
    f.niri_state().refresh_and_flush_clients();
    assert_eq!(f.niri().signal_deadlines.len(), 1, "rewrite replaced, not duplicated");

    // Unmap by committing a null buffer, the idiom used in src/tests/floating.rs.
    let window = f.client(id).window(&surface);
    window.attach_null();
    window.commit();
    f.double_roundtrip(id);
    f.niri_state().refresh_and_flush_clients();
    assert!(f.niri().signal_deadlines.is_empty(), "unmap cancelled the timer");
}
```

- [ ] **Step 4: IPC requests, server conversion, request handling, event diff**

In `niri-ipc/src/lib.rs`, add to `Request` after `PickColor`:

```rust
    /// Set a source's signal on a window.
    SetWindowSignal {
        /// Id of the window.
        id: u64,
        /// Source name; `niri` is reserved.
        source: String,
        /// Accent color as `#rrggbb` or `#rrggbbaa`.
        accent: Option<String>,
        /// Level.
        level: SignalLevel,
        /// Sustained motion.
        motion: SignalMotion,
        /// Free-form tag for window-rule matching.
        tag: Option<String>,
        /// Time until the slot decays to `after_level` / `after_motion`, in ms.
        ttl_ms: Option<u32>,
        /// Level after decay.
        after_level: Option<SignalLevel>,
        /// Motion after decay.
        after_motion: Option<SignalMotion>,
        /// Whether focusing the window demotes the slot to its `after` pair.
        until_focus: bool,
    },
    /// Raise a transient impulse on a window for an existing source.
    PulseWindowSignal {
        /// Id of the window.
        id: u64,
        /// Source name; must already have a slot.
        source: String,
        /// Kind of impulse.
        kind: ImpulseKind,
        /// Accent color override as `#rrggbb` or `#rrggbbaa`.
        accent: Option<String>,
    },
    /// Remove a source's signal slot from a window.
    ClearWindowSignal {
        /// Id of the window.
        id: u64,
        /// Source name.
        source: String,
    },
```

In `src/ipc/server.rs`:

```rust
pub fn to_ipc_signal(folded: &crate::window::signal::Folded) -> niri_ipc::Signal {
    use crate::window::signal::accent_hex;
    niri_ipc::Signal {
        level: folded.level,
        motion: folded.motion,
        accent: folded.accent.map(accent_hex),
        tag: folded.tag.clone(),
        sources: folded.sources.clone(),
        impulses: folded
            .impulses
            .iter()
            .map(|i| niri_ipc::Impulse {
                source: i.source.clone(),
                kind: i.kind,
                accent: i.accent.map(accent_hex),
                at: Timestamp::from(i.at),
                expires_at: Timestamp::from(i.expires_at()),
            })
            .collect(),
    }
}
```

In `make_ipc_window` set `signal: mapped.signals().fold(get_monotonic_time()).as_ref().map(to_ipc_signal)`.

In the diff loop after the urgency check:

```rust
            let signal = mapped.signals().fold(get_monotonic_time()).as_ref().map(to_ipc_signal);
            if signal != ipc_win.signal {
                events.push(Event::WindowSignalChanged { id, signal });
            }
```

Add the three request arms next to `PickColor`, following its channel pattern:

```rust
        Request::SetWindowSignal { id, source, accent, level, motion, tag, ttl_ms, after_level, after_motion, until_focus } => {
            let (tx, rx) = async_channel::bounded(1);
            let args = crate::niri::SetWindowSignalArgs { id, source, accent, level, motion, tag, ttl_ms, after_level, after_motion, until_focus };
            ctx.event_loop.insert_idle(move |state| {
                let _ = tx.send_blocking(state.niri.set_window_signal(args));
            });
            rx.recv().await.map_err(|_| String::from("error setting window signal"))??;
            Response::Handled
        }
        Request::PulseWindowSignal { id, source, kind, accent } => {
            let (tx, rx) = async_channel::bounded(1);
            ctx.event_loop.insert_idle(move |state| {
                let _ = tx.send_blocking(state.niri.pulse_window_signal(id, &source, kind, accent.as_deref()));
            });
            rx.recv().await.map_err(|_| String::from("error pulsing window signal"))??;
            Response::Handled
        }
        Request::ClearWindowSignal { id, source } => {
            let (tx, rx) = async_channel::bounded(1);
            ctx.event_loop.insert_idle(move |state| {
                let _ = tx.send_blocking(state.niri.clear_window_signal(id, &source));
            });
            rx.recv().await.map_err(|_| String::from("error clearing window signal"))??;
            Response::Handled
        }
```

The handler returns `Result<Response, String>`; the double `?` turns both the channel failure and the compositor's `Err(String)` into the error reply.

- [ ] **Step 5: CLI**

In `src/cli.rs` `Msg`, after `PickColor`:

```rust
    /// Set a source's signal on a window.
    SetWindowSignal {
        #[arg(long)]
        id: u64,
        #[arg(long)]
        source: String,
        #[arg(long)]
        accent: Option<String>,
        #[arg(long, value_enum, default_value_t = niri_ipc::SignalLevel::Quiet)]
        level: niri_ipc::SignalLevel,
        #[arg(long, value_enum, default_value_t = niri_ipc::SignalMotion::Static)]
        motion: niri_ipc::SignalMotion,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long)]
        ttl_ms: Option<u32>,
        #[arg(long, value_enum)]
        after_level: Option<niri_ipc::SignalLevel>,
        #[arg(long, value_enum)]
        after_motion: Option<niri_ipc::SignalMotion>,
        #[arg(long)]
        until_focus: bool,
    },
    /// Raise a transient impulse on a window for an existing signal source.
    PulseWindowSignal {
        #[arg(long)]
        id: u64,
        #[arg(long)]
        source: String,
        #[arg(long, value_enum)]
        kind: niri_ipc::ImpulseKind,
        #[arg(long)]
        accent: Option<String>,
    },
    /// Remove a source's signal slot from a window.
    ClearWindowSignal {
        #[arg(long)]
        id: u64,
        #[arg(long)]
        source: String,
    },
```

In `src/ipc/client.rs`, map each to the matching `Request` variant (clone fields), and handle the response like `Msg::Action` does (expect `Response::Handled`, print nothing).

- [ ] **Step 6: Run the tests and a CLI check**

Run: `cargo test --bin niri signal` (PASS) and `cargo run --bin niri -- msg set-window-signal --help` (prints the flags, `--level` shows `quiet, active, notice, demand`).

- [ ] **Step 7: Commit**

```bash
tasks done material-844eea "feat(ipc): add window signal requests, event, timers, and CLI"
git add niri-ipc/src/lib.rs src/niri.rs src/ipc src/cli.rs src/handlers/compositor.rs src/handlers/xdg_shell.rs src/tests/signal.rs tasks
git commit -m "feat(ipc): add window signal requests, event, timers, and CLI"
```

---

### Task 7: Effective signal, solver, and fingerprint

**Files:**
- Create: `src/render_helpers/signal.rs`
- Modify: `src/render_helpers/mod.rs` (add `pub mod signal;`)
- Test: inline `#[cfg(test)]` in `src/render_helpers/signal.rs`

**Interfaces:**
- Produces:
  - `EffectiveSignal { accent: Option<Color>, level: SignalLevel, motion: SignalMotion, impulses: Vec<EffectiveImpulse> }`, `EffectiveImpulse { selector: u8, accent: Option<Color>, at: Duration }`.
  - `effective(folded: &Folded, policy: SignalMotionPolicy, response: &ResolvedResponse) -> EffectiveSignal`.
  - `effective.is_sustained() -> bool` (`motion != Static`), `effective.has_live_impulses(now) -> bool`.
  - `SignalFrame { accent: Option<[f32; 3]>, level: f32, breath: f32, impulses: [ImpulseFrame; 4] }`, `ImpulseFrame { selector: u8, envelope: f32, progress: f32, accent: Option<[f32; 3]> }` (`Default` = selector 0).
  - `solve(effective: &EffectiveSignal, now: Duration, seed: f32, level: f32, accent: Option<[f32; 3]>) -> SignalFrame`.
  - `breath(motion, now, seed) -> f32`, `next_boundary(motion, now) -> Option<Duration>`, `envelope(age) -> f32`, `progress(age) -> f32`, `level_value(SignalLevel) -> f32`, `color_linear(Color) -> [f32; 3]`.
  - `SignalFingerprint::quantize(frame: &SignalFrame) -> SignalFingerprint` (`Default` = at rest).
- Consumes: `Folded` (Task 4), `ResolvedResponse` and `SignalMotionPolicy` (Tasks 2, 3).

- [ ] **Step 1: Write the failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use niri_config::{ResolvedResponse, SignalMotionPolicy as P};
    use niri_ipc::{ImpulseKind, SignalLevel as L, SignalMotion as M};
    use crate::window::signal::{Folded, Impulse};

    fn ms(v: u64) -> Duration { Duration::from_millis(v) }

    fn folded(motion: M, impulses: Vec<Impulse>) -> Folded {
        Folded { level: L::Demand, motion, accent: None, tag: None, sources: vec!["a".into()], impulses }
    }

    fn impulse(kind: ImpulseKind, at: Duration) -> Impulse {
        Impulse { source: "a".into(), kind, accent: None, at }
    }

    #[test]
    fn policy_off_makes_motion_static_and_drops_impulses() {
        let r = ResolvedResponse::default();
        let e = effective(&folded(M::Pulse, vec![impulse(ImpulseKind::Done, ms(0))]), P::Off, &r);
        assert_eq!(e.motion, M::Static);
        assert!(e.impulses.is_empty());
    }

    #[test]
    fn reduced_maps_flash_to_pulse_and_pulse_to_breathe() {
        let r = ResolvedResponse::default();
        assert_eq!(effective(&folded(M::Flash, vec![]), P::Reduced, &r).motion, M::Pulse);
        assert_eq!(effective(&folded(M::Pulse, vec![]), P::Reduced, &r).motion, M::Breathe);
    }

    #[test]
    fn attention_none_makes_motion_static() {
        let mut r = ResolvedResponse::default();
        r.attention = niri_config::AttentionResponse::None;
        assert_eq!(effective(&folded(M::Pulse, vec![]), P::Full, &r).motion, M::Static);
    }

    #[test]
    fn none_response_drops_impulse() {
        let mut r = ResolvedResponse::default();
        r.done = niri_config::ImpulseResponse::None;
        let e = effective(&folded(M::Static, vec![impulse(ImpulseKind::Done, ms(0)), impulse(ImpulseKind::Error, ms(1))]), P::Full, &r);
        assert_eq!(e.impulses.len(), 1);
        assert_eq!(e.impulses[0].selector, niri_config::ImpulseResponse::Flash as u8);
    }

    #[test]
    fn envelope_and_progress_shapes() {
        assert_eq!(envelope(ms(0)), 0.);
        assert!((envelope(ms(80)) - 1.).abs() < 1e-6);
        assert!(envelope(ms(430)) < envelope(ms(80)));
        assert_eq!(envelope(ms(1500)), 0.);
        assert_eq!(progress(ms(0)), 0.);
        assert!((progress(ms(750)) - 0.5).abs() < 1e-6);
        assert!(progress(ms(1500)) >= 1.);
    }

    #[test]
    fn breath_is_zero_when_static_and_bounded_otherwise() {
        assert_eq!(breath(M::Static, ms(123), 0.3), 0.);
        for t in (0..4000).step_by(50) {
            let b = breath(M::Breathe, ms(t), 0.3);
            assert!((0. ..=1.).contains(&b));
        }
    }

    #[test]
    fn flash_ignores_seed_and_breathe_uses_it() {
        assert_eq!(breath(M::Flash, ms(120), 0.), breath(M::Flash, ms(120), 0.7));
        assert_ne!(breath(M::Breathe, ms(500), 0.), breath(M::Breathe, ms(500), 0.7));
    }

    #[test]
    fn next_boundary_is_clock_aligned() {
        // Breathe period 4000 ms / 32 = 125 ms buckets, aligned to the absolute clock.
        assert_eq!(next_boundary(M::Breathe, ms(0)), Some(ms(125)));
        assert_eq!(next_boundary(M::Breathe, ms(130)), Some(ms(250)));
        assert_eq!(next_boundary(M::Pulse, ms(0)), Some(Duration::from_micros(37_500)));
        assert_eq!(next_boundary(M::Static, ms(10)), None);
        // Flash: 500 ms period, edges at 0 and 250, exactly 4 sample instants
        // per 50 ms edge, nothing on the plateaus: 16 boundaries per second.
        assert_eq!(next_boundary(M::Flash, ms(0)), Some(Duration::from_micros(12_500)));
        assert_eq!(next_boundary(M::Flash, ms(60)), Some(Duration::from_micros(262_500)));
        assert_eq!(next_boundary(M::Flash, ms(300)), Some(Duration::from_micros(512_500)));
        let mut t = Duration::ZERO;
        let mut seq = Vec::new();
        while let Some(n) = next_boundary(M::Flash, t) {
            if n >= Duration::from_secs(1) {
                break;
            }
            seq.push(n);
            t = n;
        }
        let expected: Vec<Duration> = [0u64, 250, 500, 750]
            .iter()
            .flat_map(|edge| (1..=4).map(move |k| Duration::from_micros(edge * 1000 + 12_500 * k)))
            .collect();
        assert_eq!(seq, expected);
    }

    #[test]
    fn oscillator_is_exact_after_days_of_uptime() {
        let days = Duration::from_secs(9 * 86_400);
        for t in [ms(0), ms(37), ms(613), ms(1199)] {
            let a = breath(M::Pulse, t, 0.3);
            let b = breath(M::Pulse, days + t, 0.3);
            assert!((a - b).abs() < 1e-6, "{t:?}: {a} vs {b}");
            assert_eq!(next_boundary(M::Pulse, days + t).map(|n| n - days), next_boundary(M::Pulse, t));
        }
    }

    #[test]
    fn fingerprint_is_default_at_rest_and_tracks_progress_and_selector() {
        let e = EffectiveSignal { accent: None, level: L::Quiet, motion: M::Static, impulses: vec![] };
        let f = solve(&e, ms(5000), 0.1, 0., None);
        assert_eq!(SignalFingerprint::quantize(&f), SignalFingerprint::default());

        let mut e2 = e.clone();
        e2.impulses.push(EffectiveImpulse { selector: 3, accent: None, at: ms(0) });
        // Two moments on the flat tail where the envelope bucket is the same but progress moved.
        let a = SignalFingerprint::quantize(&solve(&e2, ms(1400), 0.1, 0., None));
        let b = SignalFingerprint::quantize(&solve(&e2, ms(1420), 0.1, 0., None));
        assert_ne!(a, b);

        let mut e3 = e2.clone();
        e3.impulses[0].selector = 2;
        let c = SignalFingerprint::quantize(&solve(&e3, ms(1400), 0.1, 0., None));
        assert_ne!(a, c);
    }
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test --bin niri render_helpers::signal`
Expected: compile errors.

- [ ] **Step 3: Implement the module**

```rust
//! Effective signal and the pure envelope solver (design §3, §4).

use std::f32::consts::TAU;
use std::time::Duration;

use niri_config::{Color, ResolvedResponse, SignalMotionPolicy};
use niri_ipc::{SignalLevel, SignalMotion};

use crate::window::signal::{Folded, IMPULSE_LIFETIME};

pub const BREATHE_PERIOD: Duration = Duration::from_millis(4000);
pub const PULSE_PERIOD: Duration = Duration::from_millis(1200);
pub const FLASH_PERIOD: Duration = Duration::from_millis(500);
pub const FLASH_EDGE: Duration = Duration::from_millis(50);
const BUCKETS_PER_PERIOD: u32 = 32;
const FLASH_EDGE_SAMPLES: u32 = 4;
const ATTACK: Duration = Duration::from_millis(80);
const DECAY_TAU_SECS: f32 = 0.35;

#[derive(Debug, Clone, PartialEq)]
pub struct EffectiveImpulse {
    pub selector: u8,
    pub accent: Option<Color>,
    pub at: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EffectiveSignal {
    pub accent: Option<Color>,
    pub level: SignalLevel,
    pub motion: SignalMotion,
    pub impulses: Vec<EffectiveImpulse>,
}

impl EffectiveSignal {
    pub fn is_sustained(&self) -> bool {
        self.motion != SignalMotion::Static
    }

    pub fn has_live_impulses(&self, now: Duration) -> bool {
        self.impulses.iter().any(|i| now < i.at + IMPULSE_LIFETIME)
    }
}

/// Stage 1: apply the global policy and the response block (design §3).
pub fn effective(folded: &Folded, policy: SignalMotionPolicy, response: &ResolvedResponse) -> EffectiveSignal {
    let motion = match (policy, folded.motion) {
        (SignalMotionPolicy::Off, _) => SignalMotion::Static,
        (SignalMotionPolicy::Reduced, SignalMotion::Flash) => SignalMotion::Pulse,
        (SignalMotionPolicy::Reduced, SignalMotion::Pulse) => SignalMotion::Breathe,
        (_, m) => m,
    };
    let motion = if response.attention_is_none() { SignalMotion::Static } else { motion };
    let impulses = folded
        .impulses
        .iter()
        .filter_map(|i| {
            let selector = response.impulse_selector(i.kind, policy)?;
            Some(EffectiveImpulse { selector, accent: i.accent, at: i.at })
        })
        .collect();
    EffectiveSignal { accent: folded.accent, level: folded.level, motion, impulses }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ImpulseFrame {
    pub selector: u8,
    pub envelope: f32,
    pub progress: f32,
    pub accent: Option<[f32; 3]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SignalFrame {
    pub accent: Option<[f32; 3]>,
    pub level: f32,
    pub breath: f32,
    pub impulses: [ImpulseFrame; 4],
}

pub fn level_value(level: SignalLevel) -> f32 {
    match level {
        SignalLevel::Quiet => 0.,
        SignalLevel::Active => 1. / 3.,
        SignalLevel::Notice => 2. / 3.,
        SignalLevel::Demand => 1.,
    }
}

/// sRGB config color to linear RGB, matching the shader's `srgbToLinear`.
pub fn color_linear(c: Color) -> [f32; 3] {
    let lin = |v: f32| if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) };
    [lin(c.r), lin(c.g), lin(c.b)]
}

pub fn envelope(age: Duration) -> f32 {
    if age >= IMPULSE_LIFETIME {
        return 0.;
    }
    if age < ATTACK {
        return age.as_secs_f32() / ATTACK.as_secs_f32();
    }
    (-(age - ATTACK).as_secs_f32() / DECAY_TAU_SECS).exp()
}

pub fn progress(age: Duration) -> f32 {
    (age.as_secs_f32() / IMPULSE_LIFETIME.as_secs_f32()).min(1.)
}

fn period(motion: SignalMotion) -> Option<Duration> {
    match motion {
        SignalMotion::Static => None,
        SignalMotion::Breathe => Some(BREATHE_PERIOD),
        SignalMotion::Pulse => Some(PULSE_PERIOD),
        SignalMotion::Flash => Some(FLASH_PERIOD),
    }
}

/// Oscillator value in [0, 1]. Breathe and Pulse are quantized to their
/// bucket so the value is constant between boundaries; Flash is a square
/// with 50 ms soft edges and a global phase.
/// Sub-period time as `f32`, taken with integer arithmetic first so the
/// value stays exact after days of uptime (a raw `as_secs_f32` loses the
/// millisecond digits past a few days).
fn in_period(now: Duration, period: Duration) -> f32 {
    let nanos = now.as_nanos() % period.as_nanos();
    Duration::from_nanos(nanos as u64).as_secs_f32()
}

pub fn breath(motion: SignalMotion, now: Duration, seed: f32) -> f32 {
    let Some(period) = period(motion) else { return 0. };
    let p = period.as_secs_f32();
    let t = in_period(now, period);
    if motion == SignalMotion::Flash {
        let half = p / 2.;
        let e = FLASH_EDGE.as_secs_f32();
        let rising = (t / e).clamp(0., 1.);
        let falling = 1. - ((t - half) / e).clamp(0., 1.);
        return if t < half { rising } else { falling };
    }
    let bucket = p / BUCKETS_PER_PERIOD as f32;
    let t = (t / bucket).floor() * bucket;
    let phase = (t / p + seed) * TAU;
    0.5 - 0.5 * phase.cos()
}

/// Next absolute-clock instant at which `breath` changes (design §4).
pub fn next_boundary(motion: SignalMotion, now: Duration) -> Option<Duration> {
    let period = period(motion)?;
    if motion == SignalMotion::Flash {
        // Exactly four sample instants per edge: edge + 12.5, 25, 37.5, 50 ms.
        // The edge start itself is not a boundary: the value there equals
        // the preceding plateau. Search this period and the next.
        let half = period / 2;
        let sample = FLASH_EDGE / FLASH_EDGE_SAMPLES;
        let t = Duration::from_nanos((now.as_nanos() % period.as_nanos()) as u64);
        let base = now - t;
        let mut candidates = Vec::new();
        for start in [base, base + period] {
            for edge in [Duration::ZERO, half] {
                for k in 1..=FLASH_EDGE_SAMPLES {
                    candidates.push(start + edge + sample * k);
                }
            }
        }
        return candidates.into_iter().filter(|c| *c > now).min();
    }
    let bucket = period / BUCKETS_PER_PERIOD;
    let n = now.as_nanos() / bucket.as_nanos() + 1;
    Some(Duration::from_nanos((n * bucket.as_nanos()) as u64))
}

/// Stage 2: pure solve (design §3). `level` and `accent` are the
/// crossfaded values the tile already computed.
pub fn solve(e: &EffectiveSignal, now: Duration, seed: f32, level: f32, accent: Option<[f32; 3]>) -> SignalFrame {
    let mut impulses = [ImpulseFrame::default(); 4];
    let live = e.impulses.iter().filter(|i| now < i.at + IMPULSE_LIFETIME);
    for (slot, i) in impulses.iter_mut().zip(live) {
        let age = now.saturating_sub(i.at);
        *slot = ImpulseFrame {
            selector: i.selector,
            envelope: envelope(age),
            progress: progress(age),
            accent: i.accent.map(color_linear),
        };
    }
    SignalFrame { accent, level, breath: breath(e.motion, now, seed), impulses }
}

/// Quantized frame for damage tracking (design §4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignalFingerprint {
    level_q: i32,
    accent_q: [i32; 3],
    breath_q: i32,
    impulses_q: [(u8, i32, i32, [i32; 3]); 4],
}

impl Default for SignalFingerprint {
    /// At rest: no accent (-1 sentinel), zero level and breath, no impulses.
    fn default() -> Self {
        Self {
            level_q: 0,
            accent_q: [-1; 3],
            breath_q: 0,
            impulses_q: [(0, 0, 0, [-1; 3]); 4],
        }
    }
}

impl SignalFingerprint {
    pub fn quantize(f: &SignalFrame) -> Self {
        let q256 = |v: f32| (v * 256.).round() as i32;
        let q128 = |v: f32| (v * 128.).round() as i32;
        let color = |c: Option<[f32; 3]>| c.map_or([-1; 3], |c| c.map(q256));
        let mut impulses_q = [(0u8, 0i32, 0i32, [-1i32; 3]); 4];
        for (dst, i) in impulses_q.iter_mut().zip(f.impulses.iter()) {
            *dst = if i.selector == 0 {
                (0, 0, 0, [-1; 3])
            } else {
                (i.selector, q128(i.envelope), q128(i.progress), color(i.accent))
            };
        }
        Self {
            level_q: q256(f.level),
            accent_q: color(f.accent),
            breath_q: (f.breath * 32.).round() as i32,
            impulses_q,
        }
    }
}
```

The 1/32 quantization of `breath` in the fingerprint follows the bucket count so a Breathe or Pulse window changes fingerprint exactly when its bucket changes.

- [ ] **Step 4: Run the tests**

Run: `cargo test --bin niri render_helpers::signal`
Expected: PASS. If `next_boundary_is_clock_aligned` differs by a nanosecond from integer division, compare with a 1 µs tolerance rather than changing the algorithm.

- [ ] **Step 5: Commit**

```bash
tasks done material-f19f8f "feat(render): add the signal solver and fingerprint"
git add src/render_helpers/signal.rs src/render_helpers/mod.rs tasks
git commit -m "feat(render): add the signal solver and fingerprint"
```

---

### Task 8: Glass helper, uniforms, and shader registry

**Files:**
- Modify: `src/render_helpers/material.rs` (`JellyUniforms` region ~190, `InputFingerprint` at 375, `MaterialState::element` at 454, `MaterialRenderElement` at 523, `draw` uniforms at ~640)
- Modify: `src/render_helpers/shaders/mod.rs` (uniform list at ~158)
- Modify: `src/render_helpers/shaders/material.frag` (uniform declarations at top only)
- Test: `src/render_helpers/material.rs` tests module (existing, see `tap_count_follows_the_strongest_multi_tap_effect` at ~1273)

**Interfaces:**
- Produces:
  - `GlassSignalInputs { activity_add: f32, chromatic_aberration: f64, distortion: f64, samples: u8, impulses: [ImpulseFrame; 4] }`.
  - `glass_signal_inputs(frame: &SignalFrame, glass: &ResolvedGlass) -> GlassSignalInputs`.
  - `SignalUniforms { accent: [f32; 4], level: f32, breath: f32, light: [f32; 3], impulse_env: [f32; 4], impulse_prog: [f32; 4], impulse_rgb: [[f32; 3]; 4], impulse_resp: [i32; 4], response: [i32; 2], ring: [f32; 2] }` with `SignalUniforms::quiet(response: &ResolvedResponse) -> Self`.
  - `GlassSignalFingerprint { activity_add_q: i32, chromatic_q: i32, distortion_q: i32, samples: u8 }` with `GlassSignalFingerprint::quantize(&GlassSignalInputs)` (1/1024 steps) and `Default` matching `GlassSignalInputs::quiet` of default glass only through `quantize`, never assumed.
  - `InputFingerprint` gains exactly two fields: `pub signal: SignalFingerprint` and `pub glass_signal: GlassSignalFingerprint`.
  - `MaterialState::element` gains exactly two parameters, inserted after `jelly: JellyUniforms`: `signal: SignalUniforms, glass_signal: GlassSignalInputs`. `MaterialRenderElement` stores both. `draw` uploads `mat_chromatic_aberration`, `mat_distortion`, and `mat_samples` from `glass_signal`, not from `glass`.
  - `SignalUniforms::from_frame(frame, g, response)` sets `light` to the quiet `[-1, -1, 0]` unless `response.attention == AttentionResponse::RimOrbit`.
- Consumes: Task 7 types, `ResolvedResponse` (Task 3).

- [ ] **Step 1: Write the failing tests**

In the tests module of `src/render_helpers/material.rs`:

```rust
    #[test]
    fn glass_signal_inputs_sums_ripples_and_maxes_flash() {
        use crate::render_helpers::signal::{ImpulseFrame, SignalFrame};
        use niri_config::ImpulseResponse as R;
        let mut frame = SignalFrame { accent: None, level: 0., breath: 0., impulses: Default::default() };
        frame.impulses[0] = ImpulseFrame { selector: R::Ripple as u8, envelope: 0.7, progress: 0.1, accent: None };
        frame.impulses[1] = ImpulseFrame { selector: R::Ripple as u8, envelope: 0.6, progress: 0.2, accent: None };
        frame.impulses[2] = ImpulseFrame { selector: R::Flash as u8, envelope: 0.4, progress: 0.3, accent: None };
        frame.impulses[3] = ImpulseFrame { selector: R::Flash as u8, envelope: 0.8, progress: 0.4, accent: None };
        let glass = ResolvedGlass::default();
        let g = glass_signal_inputs(&frame, &glass);
        assert!((g.activity_add - 0.999).abs() < 1e-6, "ripples clamp to [0, 1)");
        assert!((g.chromatic_aberration - 0.4).abs() < 1e-6, "0 + 0.5 * max(0.4, 0.8)");
        assert!((g.distortion - 0.2).abs() < 1e-6);
        assert_eq!(g.samples, tap_count(0., 0.4));
        assert!(g.impulses.iter().all(|i| i.selector == R::None as u8), "consumed slots become none");
    }

    #[test]
    fn rim_light_only_moves_under_rim_orbit() {
        use crate::render_helpers::signal::SignalFrame;
        use niri_config::{AttentionResponse, ResolvedResponse};
        let frame = SignalFrame { accent: Some([1., 0.5, 0.]), level: 1., breath: 1., impulses: Default::default() };
        let g = glass_signal_inputs(&frame, &ResolvedGlass::default());
        let mut r = ResolvedResponse::default();
        for (attention, moves) in [
            (AttentionResponse::RimOrbit, true),
            (AttentionResponse::RingPulse, false),
            (AttentionResponse::None, false),
        ] {
            r.attention = attention;
            let u = SignalUniforms::from_frame(&frame, &g, &r);
            let quiet = SignalUniforms::quiet(&r).light;
            assert_eq!(u.light != quiet, moves, "{attention:?}");
        }
    }

    #[test]
    fn glass_signal_inputs_leaves_sweep_alone() {
        use crate::render_helpers::signal::{ImpulseFrame, SignalFrame};
        use niri_config::ImpulseResponse as R;
        let mut frame = SignalFrame { accent: None, level: 0., breath: 0., impulses: Default::default() };
        frame.impulses[0] = ImpulseFrame { selector: R::Sweep as u8, envelope: 0.5, progress: 0.5, accent: None };
        let g = glass_signal_inputs(&frame, &ResolvedGlass::default());
        assert_eq!(g.impulses[0].selector, R::Sweep as u8);
        assert_eq!(g.activity_add, 0.);
        assert_eq!(g.samples, 1);
    }
```

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test --bin niri glass_signal_inputs`
Expected: compile errors.

- [ ] **Step 3: Implement the helper and uniform struct**

In `src/render_helpers/material.rs`:

```rust
use super::signal::{ImpulseFrame, SignalFingerprint, SignalFrame};

/// Glass-specific interpretation of a `SignalFrame` (design §6). The one
/// place glass selectors are read.
#[derive(Debug, Clone, PartialEq)]
pub struct GlassSignalInputs {
    pub activity_add: f32,
    pub chromatic_aberration: f64,
    pub distortion: f64,
    pub samples: u8,
    pub impulses: [ImpulseFrame; 4],
}

pub fn glass_signal_inputs(frame: &SignalFrame, glass: &ResolvedGlass) -> GlassSignalInputs {
    use niri_config::ImpulseResponse as R;
    let mut activity_add = 0.;
    let mut flash = 0f32;
    let mut impulses = frame.impulses;
    for i in &mut impulses {
        match i.selector {
            s if s == R::Ripple as u8 => {
                activity_add += i.envelope;
                *i = ImpulseFrame::default();
            }
            s if s == R::Flash as u8 => {
                flash = flash.max(i.envelope);
                *i = ImpulseFrame::default();
            }
            _ => {}
        }
    }
    let chromatic_aberration = (glass.chromatic_aberration + 0.5 * f64::from(flash)).min(1.);
    let distortion = (glass.distortion + 0.25 * f64::from(flash)).min(1.);
    GlassSignalInputs {
        activity_add: activity_add.min(0.999),
        chromatic_aberration,
        distortion,
        samples: tap_count(glass.anisotropic_blur, chromatic_aberration),
        impulses,
    }
}

/// Uniform values for the signal responses.
#[derive(Debug, Clone, PartialEq)]
pub struct SignalUniforms {
    pub accent: [f32; 4],
    pub level: f32,
    pub breath: f32,
    pub light: [f32; 3],
    pub impulse_env: [f32; 4],
    pub impulse_prog: [f32; 4],
    pub impulse_rgb: [[f32; 3]; 4],
    pub impulse_resp: [i32; 4],
    pub response: [i32; 2],
    pub ring: [f32; 2],
}

impl SignalUniforms {
    /// A window with no signal: bit-identical to today's output.
    pub fn quiet(response: &niri_config::ResolvedResponse) -> Self {
        Self {
            accent: [0.; 4],
            level: 0.,
            breath: 0.,
            light: [-1., -1., 0.],
            impulse_env: [0.; 4],
            impulse_prog: [0.; 4],
            impulse_rgb: [[0.; 3]; 4],
            impulse_resp: [0; 4],
            response: [response.accent as i32, response.attention as i32],
            ring: [response.ring_inset as f32, response.ring_width as f32],
        }
    }

    /// Builds the uniforms from a frame after glass interpretation.
    pub fn from_frame(frame: &SignalFrame, g: &GlassSignalInputs, response: &niri_config::ResolvedResponse) -> Self {
        // Rim orbit only: sway the glint light around top-left by up to
        // level * pi/2, driven by breath so its cost is covered by the breath
        // buckets. Every other attention response keeps today's fixed light.
        let light = if response.attention == niri_config::AttentionResponse::RimOrbit {
            let base = -3. * std::f32::consts::FRAC_PI_4;
            let sway = frame.level * std::f32::consts::FRAC_PI_2 * (2. * frame.breath - 1.) / 2.;
            let angle = base + sway;
            [angle.cos(), angle.sin(), frame.level]
        } else {
            [-1., -1., 0.]
        };
        let mut rgb = [[0.; 3]; 4];
        let mut env = [0.; 4];
        let mut prog = [0.; 4];
        let mut resp = [0; 4];
        for (k, i) in g.impulses.iter().enumerate() {
            env[k] = i.envelope;
            prog[k] = i.progress;
            resp[k] = i32::from(i.selector);
            rgb[k] = i.accent.or(frame.accent).unwrap_or([1., 1., 1.]);
        }
        Self {
            accent: frame.accent.map_or([0.; 4], |c| [c[0], c[1], c[2], 1.]),
            level: frame.level,
            breath: frame.breath,
            light,
            impulse_env: env,
            impulse_prog: prog,
            impulse_rgb: rgb,
            impulse_resp: resp,
            response: [response.accent as i32, response.attention as i32],
            ring: [response.ring_inset as f32, response.ring_width as f32],
        }
    }
}
```

Add the glass fingerprint next to `JellyFingerprint`:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GlassSignalFingerprint {
    activity_add_q: i32,
    chromatic_q: i32,
    distortion_q: i32,
    samples: u8,
}

impl GlassSignalFingerprint {
    pub fn quantize(g: &GlassSignalInputs) -> Self {
        Self {
            activity_add_q: (g.activity_add * 1024.).round() as i32,
            chromatic_q: (g.chromatic_aberration * 1024.).round() as i32,
            distortion_q: (g.distortion * 1024.).round() as i32,
            samples: g.samples,
        }
    }
}
```

Add `GlassSignalInputs::quiet(glass: &ResolvedGlass) -> Self` (activity 0, base aberration and distortion, `tap_count` of the base values, default impulses). Extend `InputFingerprint` with `pub signal: SignalFingerprint` and `pub glass_signal: GlassSignalFingerprint`. Add `signal: SignalUniforms` and `glass_signal: GlassSignalInputs` fields to `MaterialRenderElement`, and the two parameters to `MaterialState::element` after `jelly`:

```rust
    pub fn element(
        &self,
        frame: MaterialFrame,
        mapping: BackgroundMapping,
        jelly: JellyUniforms,
        signal: SignalUniforms,
        glass_signal: GlassSignalInputs,
        scale: f64,
        alpha: f32,
        target: RenderTarget,
        inputs: InputFingerprint,
        win_rect: [f32; 4],
        win_src: Rectangle<f64, Buffer>,
        win_texture: GlesTexture,
        bg: Rc<RefCell<EffectBuffer>>,
        backdrop: Rc<RefCell<EffectBuffer>>,
        backdrop_color: [f32; 4],
    ) -> MaterialRenderElement
```

In `draw`, upload `mat_chromatic_aberration` from `self.glass_signal.chromatic_aberration as f32`, `mat_distortion` from `self.glass_signal.distortion as f32`, and `mat_samples` from `f32::from(self.glass_signal.samples)`, and add:

```rust
            Uniform::new("mat_sig_accent", self.signal.accent),
            Uniform::new("mat_sig_level", self.signal.level),
            Uniform::new("mat_sig_breath", self.signal.breath),
            Uniform::new("mat_sig_light", self.signal.light),
            Uniform::new("mat_sig_impulse_env", self.signal.impulse_env),
            Uniform::new("mat_sig_impulse_prog", self.signal.impulse_prog),
            Uniform::new("mat_sig_impulse_rgb0", self.signal.impulse_rgb[0]),
            Uniform::new("mat_sig_impulse_rgb1", self.signal.impulse_rgb[1]),
            Uniform::new("mat_sig_impulse_rgb2", self.signal.impulse_rgb[2]),
            Uniform::new("mat_sig_impulse_rgb3", self.signal.impulse_rgb[3]),
            Uniform::new("mat_sig_impulse_resp", UniformValue::_4i(r[0], r[1], r[2], r[3])),
            Uniform::new("mat_sig_response", UniformValue::_2i(self.signal.response[0], self.signal.response[1])),
            Uniform::new("mat_sig_ring", self.signal.ring),
```

with `let r = self.signal.impulse_resp;` and `use smithay::backend::renderer::gles::UniformValue;`. Also add `mat_jelly_activity` as `(jelly.activity + activity_add).min(0.999)`; the tile does that sum before building `JellyUniforms` (Task 9), so `draw` needs no change for it.

Register every new uniform in `src/render_helpers/shaders/mod.rs` next to `mat_jelly_ripple`:

```rust
                UniformName::new("mat_sig_accent", UniformType::_4f),
                UniformName::new("mat_sig_level", UniformType::_1f),
                UniformName::new("mat_sig_breath", UniformType::_1f),
                UniformName::new("mat_sig_light", UniformType::_3f),
                UniformName::new("mat_sig_impulse_env", UniformType::_4f),
                UniformName::new("mat_sig_impulse_prog", UniformType::_4f),
                UniformName::new("mat_sig_impulse_rgb0", UniformType::_3f),
                UniformName::new("mat_sig_impulse_rgb1", UniformType::_3f),
                UniformName::new("mat_sig_impulse_rgb2", UniformType::_3f),
                UniformName::new("mat_sig_impulse_rgb3", UniformType::_3f),
                UniformName::new("mat_sig_impulse_resp", UniformType::_4i),
                UniformName::new("mat_sig_response", UniformType::_2i),
                UniformName::new("mat_sig_ring", UniformType::_2f),
```

Declare them at the top of `material.frag`:

```glsl
uniform vec4 mat_sig_accent;
uniform float mat_sig_level;
uniform float mat_sig_breath;
uniform vec3 mat_sig_light;
uniform vec4 mat_sig_impulse_env;
uniform vec4 mat_sig_impulse_prog;
uniform vec3 mat_sig_impulse_rgb0;
uniform vec3 mat_sig_impulse_rgb1;
uniform vec3 mat_sig_impulse_rgb2;
uniform vec3 mat_sig_impulse_rgb3;
uniform ivec4 mat_sig_impulse_resp;
uniform ivec2 mat_sig_response;
uniform vec2 mat_sig_ring;
```

Unused uniforms may be optimized out by the GL compiler; smithay's `ShaderProgram::compile` tolerates a missing location for declared names (check `shader_element.rs` for how it handles `glGetUniformLocation` returning -1; if it errors, reference each uniform trivially in `main` until Task 10 uses them).

Every existing call to `MaterialState::element` in `src/layout/tile.rs` passes `SignalUniforms::quiet(&response)` and `GlassSignalInputs::quiet(glass)` for now, with `InputFingerprint { signal: SignalFingerprint::default(), glass_signal: GlassSignalFingerprint::quantize(&quiet_inputs), .. }`. Task 9 wires the real values.

- [ ] **Step 4: Run the tests and the material suite**

Run: `cargo test --bin niri material`
Expected: PASS, including the existing material tests, since quiet uniforms leave output unchanged.

- [ ] **Step 5: Commit**

```bash
tasks done material-0e32cc "feat(render): add glass signal inputs and uniforms"
git add src/render_helpers src/layout/tile.rs tasks
git commit -m "feat(render): add glass signal inputs and uniforms"
```

---

### Task 9: Tile wiring: crossfade, effective signal, fingerprint, transitions, and bucket timers

**Files:**
- Modify: `src/layout/tile.rs` (fields at ~90-125, `new` at 286, `are_transitions_ongoing` at 569, both material sites at ~1321 and ~1503, `render` at 1697)
- Modify: `src/render_helpers/mod.rs` (`RenderCtx` at 58)
- Modify: `src/render_helpers/signal.rs` (`slab_in_view`, `tick_deadline`)
- Modify: `src/niri.rs` (`OutputState` at 447, `Niri::render` for injection and the view rect, `redraw` at 4578)
- Test: `src/tests/signal.rs`, `src/render_helpers/signal.rs`

**Interfaces:**
- Produces:
  - `SignalTicks { pub next: Cell<Option<Duration>>, pub view: Cell<Rectangle<f64, Logical>> }` in `src/render_helpers/mod.rs` with `SignalTicks::report(&self, deadline: Duration)` (keeps the minimum) and `SignalTicks::reset(&self)`.
  - `RenderCtx.signal_ticks: Option<Rc<SignalTicks>>` (an `Rc`, so `Niri::render` can inject it from `&self` without lifetime coupling; `r()` clones it).
  - `OutputState.signal_ticks: Rc<SignalTicks>`, `OutputState.signal_timer: Option<RegistrationToken>`.
  - `slab_in_view(location, size, bevel, view) -> bool` and `tick_deadline(eff, in_view, now) -> Option<Duration>` in `src/render_helpers/signal.rs` (pure; unit-tested); `Tile::signal_tick_deadline(&self, location, view, now) -> Option<Duration>` composes them.
  - `pub fn Niri::arm_signal_timer(&mut self, output: &Output)` (public so the fixture test can call it).
  - `Tile::signal_crossfade: Option<SignalCrossfade>` (private).
- Consumes: Tasks 5, 7, 8.

- [ ] **Step 1: Write the failing tests**

Append to `src/tests/signal.rs`:

```rust
#[test]
fn crossfade_and_live_impulses_are_transitions_but_sustained_motion_is_not() {
    use crate::niri::SetWindowSignalArgs;
    use niri_ipc::{ImpulseKind, SignalLevel, SignalMotion};
    use std::time::Duration;

    let mut f = Fixture::with_config(config(r#"material "tg" { glass {} }  window-rule { material "tg" }"#));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_window(&mut f, id, "a");
    let wid = window_id(&mut f, "a");

    // Baseline: settle the open animation first, or it masks every assertion below.
    f.niri_complete_animations();
    f.niri().layout.update_render_elements(None);
    assert!(!f.niri().layout.are_animations_ongoing(None), "settled window has no transition");

    f.niri()
        .set_window_signal(SetWindowSignalArgs {
            id: wid,
            source: String::from("t"),
            accent: Some(String::from("#e5a33c")),
            level: SignalLevel::Demand,
            motion: SignalMotion::Pulse,
            tag: None,
            ttl_ms: None,
            after_level: None,
            after_motion: None,
            until_focus: false,
        })
        .unwrap();
    // The crossfade is created by update_render_elements, which the headless
    // fixture does not run on its own.
    f.niri_state().refresh_and_flush_clients();
    f.niri().layout.update_render_elements(None);
    assert!(f.niri().layout.are_animations_ongoing(None), "crossfade is a transition");

    f.niri_complete_animations(); // drops the finished crossfade in Tile::advance_animations
    f.niri().layout.update_render_elements(None);
    assert!(!f.niri().layout.are_animations_ongoing(None), "sustained Pulse is not a transition");

    f.niri().pulse_window_signal(wid, "t", ImpulseKind::Done, None).unwrap();
    f.niri_state().refresh_and_flush_clients();
    f.niri().layout.update_render_elements(None);
    assert!(f.niri().layout.are_animations_ongoing(None), "live impulse is a transition");

    // Expire the impulse by advancing the frozen unadjusted clock past 1.5 s.
    let later = f.niri().clock.now_unadjusted() + Duration::from_secs(2);
    f.niri().clock.set_unadjusted(later);
    f.niri().layout.update_render_elements(None);
    assert!(!f.niri().layout.are_animations_ongoing(None), "expired impulse is not a transition");
    f.niri().clock.clear();
}

#[test]
fn unsignaled_material_window_has_no_transition() {
    let mut f = Fixture::with_config(config(r#"material "tg" { glass {} }  window-rule { material "tg" }"#));
    f.add_output(1, (1920, 1080));
    let id = f.add_client();
    open_window(&mut f, id, "a");
    f.niri_complete_animations(); // finish the open animation
    f.niri().layout.update_render_elements(None);
    assert!(!f.niri().layout.are_animations_ongoing(None), "no crossfade starts for a quiet window");
}

#[test]
fn arm_signal_timer_follows_the_accumulator() {
    let mut f = Fixture::with_config(config(""));
    f.add_output(1, (1920, 1080));
    let output = f.niri_output(1);

    f.niri().arm_signal_timer(&output);
    assert!(f.niri().output_state[&output].signal_timer.is_none());

    let deadline = get_monotonic_time() + std::time::Duration::from_millis(50);
    f.niri().output_state[&output].signal_ticks.report(deadline);
    f.niri().arm_signal_timer(&output);
    assert!(f.niri().output_state[&output].signal_timer.is_some());

    f.niri().output_state[&output].signal_ticks.reset();
    f.niri().arm_signal_timer(&output);
    assert!(f.niri().output_state[&output].signal_timer.is_none(), "reset removes the timer");
}
```

The redraw contract these tests and the Task 12 smoke measure: each accepted mutation and each deadline-timer firing queues one coalescible redraw request per output (an already queued frame absorbs it, and one firing may prune several expiries at once), and everything else is decided by the tile. "No recurring redraws" for a case therefore means no bucket timer and no transitions term, which the smoke measures as zero `Niri::redraw` zones in a steady-state window that starts after the last event.

The headless fixture never runs a real render pass, so tick *reporting* is covered by pure helpers in `src/render_helpers/signal.rs` (the tile only composes them) and timer *arming* by the test above; the end-to-end rate is measured in the Task 12 smoke. There is no tile test constructor with a material in `src/layout/tile.rs` tests, which is why the logic lives in the pure module.

Add to `src/render_helpers/signal.rs`:

```rust
/// Whether a tile's slab band is in view. The band is the tile rect inflated
/// by `bevel` on every side: a superset of the exact slab, so a visible
/// band never freezes when the tile rect alone leaves view.
pub fn slab_in_view(
    location: Point<f64, Logical>,
    size: Size<f64, Logical>,
    bevel: f64,
    view: Rectangle<f64, Logical>,
) -> bool {
    let slab = Rectangle::new(
        location - Point::from((bevel, bevel)),
        Size::from((size.w + 2. * bevel, size.h + 2. * bevel)),
    );
    slab.overlaps(view)
}

/// Next redraw deadline for a tile: only for sustained motion that is in view.
pub fn tick_deadline(eff: &EffectiveSignal, in_view: bool, now: Duration) -> Option<Duration> {
    if !eff.is_sustained() || !in_view {
        return None;
    }
    next_boundary(eff.motion, now)
}
```

with `use smithay::utils::{Logical, Point, Rectangle, Size};`, and these tests in its module:

```rust
    #[test]
    fn slab_in_view_uses_the_bevel_band() {
        let view = Rectangle::new(Point::from((0., 0.)), Size::from((1920., 1080.)));
        let size = Size::from((400., 300.));
        assert!(slab_in_view(Point::from((10., 10.)), size, 12., view));
        // Tile rect fully left of the view, band still overlapping.
        assert!(slab_in_view(Point::from((-400. + 6., 10.)), size, 12., view));
        // Band entirely out of view.
        assert!(!slab_in_view(Point::from((-400. - 20., 10.)), size, 12., view));
    }

    #[test]
    fn tick_deadline_requires_sustained_motion_in_view() {
        let sustained = EffectiveSignal { accent: None, level: L::Demand, motion: M::Pulse, impulses: vec![] };
        assert!(tick_deadline(&sustained, true, ms(100)).is_some());
        assert!(tick_deadline(&sustained, false, ms(100)).is_none());
        let quiet = EffectiveSignal { motion: M::Static, ..sustained };
        assert!(tick_deadline(&quiet, true, ms(100)).is_none());
    }
```

`niri_complete_animations` exists on the fixture (line 85).

- [ ] **Step 2: Run them to verify they fail**

Run: `cargo test --bin niri signal`
Expected: first assertion fails (no crossfade yet) or compile error on `signal_timer`.

- [ ] **Step 3: Add `SignalTicks` to the render context**

In `src/render_helpers/mod.rs`:

```rust
/// Per-output accumulator for the earliest sustained-signal bucket boundary
/// among tiles that actually rendered (design §4).
#[derive(Debug, Default)]
pub struct SignalTicks {
    pub next: Cell<Option<Duration>>,
    pub view: Cell<Rectangle<f64, Logical>>,
}

impl SignalTicks {
    pub fn reset(&self) {
        self.next.set(None);
    }

    pub fn report(&self, deadline: Duration) {
        let next = self.next.get().map_or(deadline, |n| n.min(deadline));
        self.next.set(Some(next));
    }
}

pub struct RenderCtx<'a, R> {
    pub renderer: &'a mut R,
    pub target: RenderTarget,
    pub xray: Option<&'a Xray>,
    pub signal_ticks: Option<Rc<SignalTicks>>,
}
```

Update `RenderCtx::r()` (clone the `Rc`) and every `RenderCtx { .. }` construction site (`grep -rn "RenderCtx {" src/`, including `src/backend/tty.rs:1884` and `src/backend/winit.rs:227`) to carry `signal_ticks: None`. The backends never set it; injection is centralized in `Niri::render` (Step 6).

- [ ] **Step 4: Crossfade and per-frame wiring in `Tile`**

In `src/layout/tile.rs` add:

```rust
struct SignalCrossfade {
    anim: Animation,
    level_from: f32,
    level_to: f32,
    accent_from: Option<[f32; 3]>,
    accent_to: Option<[f32; 3]>,
}

impl SignalCrossfade {
    fn current(&self) -> (f32, Option<[f32; 3]>) {
        let t = self.anim.clamped_value() as f32;
        let level = self.level_from + (self.level_to - self.level_from) * t;
        let accent = match (self.accent_from, self.accent_to) {
            (Some(a), Some(b)) => Some([0, 1, 2].map(|i| a[i] + (b[i] - a[i]) * t)),
            (None, Some(b)) => Some(b.map(|v| v * t)),
            (Some(a), None) => Some(a.map(|v| v * (1. - t))),
            (None, None) => None,
        };
        (level, accent)
    }
}
```

Fields on `Tile`: `signal_crossfade: Option<SignalCrossfade>`, `signal_target: Option<(f32, Option<[f32; 3]>)>` (last target, to detect changes), `signal_frame_cache: RefCell<Option<(EffectiveSignal, f32, Option<[f32; 3]>)>>`. Initialize `signal_crossfade` to `None`, `signal_target` to `Some((0., None))` so an unsignaled window never starts a zero-to-zero crossfade, and the cache to `None`.

Add a method the render sites call:

```rust
    /// Stage 1 plus crossfade bookkeeping. Returns the effective signal and
    /// the crossfaded (level, accent) for this frame, or `None` when the
    /// window has no signal and no crossfade is running.
    fn signal_for_frame(&mut self, response: &ResolvedResponse) -> Option<(EffectiveSignal, f32, Option<[f32; 3]>)> {
        use crate::render_helpers::signal::{color_linear, effective, level_value, EffectiveSignal};
        let folded = self.window.signal(self.clock.now_unadjusted());
        let eff = folded
            .as_ref()
            .map(|f| effective(f, self.options.signal.motion, response))
            .unwrap_or(EffectiveSignal { accent: None, level: niri_ipc::SignalLevel::Quiet, motion: niri_ipc::SignalMotion::Static, impulses: vec![] });
        let target = (level_value(eff.level), eff.accent.map(color_linear));
        // `signal_target` starts as Some((0., None)), so a quiet window never
        // starts a zero-to-zero crossfade.
        if self.signal_target != Some(target) {
            // A finished crossfade is removed by `advance_animations`, so
            // "current" is either the running crossfade or the last target.
            let (from_level, from_accent) = self
                .signal_crossfade
                .as_ref()
                .map(SignalCrossfade::current)
                .or(self.signal_target)
                .unwrap_or((0., None));
            let config = self.options.animations.material_signal.0;
            self.signal_crossfade = Some(SignalCrossfade {
                anim: Animation::new(self.clock.clone(), 0., 1., 0., config),
                level_from: from_level,
                level_to: target.0,
                accent_from: from_accent,
                accent_to: target.1,
            });
            self.signal_target = Some(target);
        }
        if folded.is_none() && self.signal_crossfade.is_none() {
            return None;
        }
        let (level, accent) = self.signal_crossfade.as_ref().map_or(target, SignalCrossfade::current);
        Some((eff, level, accent))
    }
```

Add the crossfade to `Tile::advance_animations` (`src/layout/tile.rs:534`), following the other animations there, so a completed crossfade is dropped instead of reported as ongoing forever:

```rust
        if let Some(crossfade) = &self.signal_crossfade {
            if crossfade.anim.is_done() {
                self.signal_crossfade = None;
            }
        }
```

The fixture's `niri_complete_animations` sets the clock to complete instantly only for the duration of one `advance_animations` call, which is exactly when this removal runs.

Because `render_inner` takes `&self`, `Tile::update_render_elements` (which is `&mut self` and already runs before every render) calls `signal_for_frame` and stores the result in `signal_frame_cache`.

At both material render sites, after `jelly_state`:

```rust
                                let response = material.material().response(None);
                                let now = self.clock.now_unadjusted();
                                let sig = self.signal_frame_cache.borrow().clone();
                                let (frame_sig, glass_sig, sig_uniforms) = match &sig {
                                    Some((eff, level, accent)) => {
                                        let seed = material.jelly_seed()[0];
                                        let frame = solve(eff, now, seed, *level, *accent);
                                        let g = glass_signal_inputs(&frame, glass);
                                        let u = SignalUniforms::from_frame(&frame, &g, &response);
                                        (SignalFingerprint::quantize(&frame), g, u)
                                    }
                                    None => (SignalFingerprint::default(), GlassSignalInputs::quiet(glass), SignalUniforms::quiet(&response)),
                                };
                                let activity = (jelly.activity + glass_sig.activity_add).min(0.999);
```

Add `GlassSignalInputs::quiet(glass)` (activity 0, base aberration and distortion, `tap_count` of base, default impulses) and `MaterialState::jelly_seed(&self) -> [f32; 3]` accessor. Feed `activity` into `JellyUniforms` and `JellyFingerprint::quantize` (pass a `JellyState` copy with the summed activity), put `frame_sig` and the quantized glass inputs into `InputFingerprint`, and pass `sig_uniforms` plus the effective aberration, distortion, and samples to `material.element(..)`.

Report sustained ticks in `Tile::render`, at the top, before rendering, through a pure method so it can be unit-tested:

```rust
    /// Next bucket boundary this tile needs a redraw for, if its effective
    /// motion is sustained and its slab band is in view. The slab is the
    /// tile rect inflated by `bevel` on every side, a superset of the exact
    /// slab so a visible band never freezes when the tile rect leaves view.
    pub fn signal_tick_deadline(
        &self,
        location: Point<f64, Logical>,
        view: Rectangle<f64, Logical>,
        now: Duration,
    ) -> Option<Duration> {
        use crate::render_helpers::signal::{slab_in_view, tick_deadline};
        let cache = self.signal_frame_cache.borrow();
        let (eff, _, _) = cache.as_ref()?;
        let bevel = self.material.as_ref()?.material().glass.bevel;
        let in_view = slab_in_view(location, self.tile_size(), bevel, view);
        tick_deadline(eff, in_view, now)
    }
```

and at the top of `Tile::render`:

```rust
        if let Some(ticks) = &ctx.signal_ticks {
            if let Some(b) = self.signal_tick_deadline(location, ticks.view.get(), self.clock.now_unadjusted()) {
                ticks.report(b);
            }
        }
```

Extend `are_transitions_ongoing`:

```rust
            || self.signal_crossfade.as_ref().is_some_and(|c| !c.anim.is_done())
            || self
                .signal_frame_cache
                .borrow()
                .as_ref()
                .is_some_and(|(eff, _, _)| eff.has_live_impulses(self.clock.now_unadjusted()))
```

- [ ] **Step 5: Set the view rect once per output pass**

`Niri::render` renders the interactively moved tile (`render_interactive_move_for_output`, `src/niri.rs:4361`) before any workspace, so the view must be set before that, not inside scrolling or floating rendering. In `Niri::render`, at the same point injection happens (Step 6), set `ticks.view` to the output's logical rect at the origin: `Rectangle::new(Point::from((0., 0.)), output_size(output).to_f64())` using the existing logical-size helper for outputs in `src/niri.rs`. Tile `location`s handed to `Tile::render` by scrolling, floating, and the interactive move are all in that output-view space (scrolling applies `view_off` before calling `tile.render`), so the overlap test in Step 4 is consistent on every path. During a workspace switch animation a tile of the outgoing workspace may report while sliding out; that over-reports for the duration of an animation that already runs at refresh rate, and is accepted.

- [ ] **Step 6: Per-output timer in `niri.rs`**

Add to `OutputState`:

```rust
    /// Earliest sustained-signal bucket boundary reported during the last render (design §4).
    pub signal_ticks: Rc<crate::render_helpers::SignalTicks>,
    /// Timer armed for that boundary, if any.
    pub signal_timer: Option<RegistrationToken>,
```

Output render contexts are built by the backends (`src/backend/tty.rs:1884`, `src/backend/winit.rs:227`), which then call `niri.render_to_vec`, which calls `Niri::render` (`src/niri.rs`, the `pub fn render<R: NiriRenderer>(&self, mut ctx: RenderCtx<R>, output, ..)` entry). Centralize injection there: at the top of `Niri::render`, when `ctx.target == RenderTarget::Output` and `ctx.signal_ticks.is_none()`, look up `self.output_state.get(output)`, call `state.signal_ticks.reset()`, and set `ctx.signal_ticks = Some(state.signal_ticks.clone())`. Every downstream reborrow carries it. Arming happens after the pass:

```rust
    pub fn arm_signal_timer(&mut self, output: &Output) {
        let state = self.output_state.get_mut(output).unwrap();
        if let Some(token) = state.signal_timer.take() {
            self.event_loop.remove(token);
        }
        let Some(deadline) = state.signal_ticks.next.get() else { return };
        let delay = deadline.saturating_sub(get_monotonic_time());
        let output = output.clone();
        let token = self
            .event_loop
            .insert_source(Timer::from_duration(delay), move |_, _, state| {
                if let Some(os) = state.niri.output_state.get_mut(&output) {
                    os.signal_timer = None;
                }
                state.niri.queue_redraw(&output);
                TimeoutAction::Drop
            })
            .unwrap();
        self.output_state.get_mut(&output).unwrap().signal_timer = Some(token);
    }
```

Call `self.arm_signal_timer(output)` in `Niri::redraw` immediately after `res = backend.render(self, output, target_presentation_time);`, and only when `res != RenderResult::Skipped`, so a skipped frame never arms from a stale accumulator. Remove the token when the output is removed (grep `output_state.remove`).

- [ ] **Step 7: Run the tests**

Run: `cargo test --bin niri signal` and `cargo test --bin niri material`
Expected: PASS.

- [ ] **Step 8: Commit**

```bash
tasks done material-9d06b4 "feat(render): drive material signals from tiles with crossfade and bucket timers"
git add src/layout src/render_helpers src/niri.rs src/tests/signal.rs tasks
git commit -m "feat(render): drive material signals from tiles with crossfade and bucket timers"
```

---

### Task 10: Glass shader responses

**Files:**
- Modify: `src/render_helpers/shaders/material.frag` (Fresnel block at ~394-408, `slabSurface` for the outer SDF value)

**Interfaces:**
- Consumes the uniforms from Task 8. Selector ids: accent `1 = ring`; attention `1 = rim-orbit`, `2 = ring-pulse`; impulse `3 = sweep` (ripple and flash never reach the shader).

- [ ] **Step 1: Expose the slab's outer signed distance**

`slabSurface` already computes the outer rounded-box SDF for `coverage`. Add an `out float outerDist` parameter returning that signed distance (negative inside), and capture it in `main` as `float slabDist;`.

- [ ] **Step 2: Replace the fixed light direction**

In the Fresnel block, replace `normalize(vec2(-1.0, -1.0))` with `normalize(mat_sig_light.xy)`. With `SignalUniforms::quiet` the vector is `(-1, -1, 0)`, so the output is bit-identical to today. Tint the specular:

```glsl
        vec3 specular = vec3(fresnel * (0.15 + 0.85 * facing));
        if (mat_sig_accent.w > 0.0 && mat_sig_light.z > 0.0)
            specular = mix(specular, specular * mat_sig_accent.rgb * 2.0, mat_sig_light.z);
```

- [ ] **Step 3: Ring and ring pulse**

After `transmitted` and `specular` are computed, before `glassed`:

```glsl
        vec3 emissive = vec3(0.0);
        if (mat_sig_response.x == 1 && mat_sig_accent.w > 0.0) {
            float inset = mat_sig_ring.x;
            float width = mat_sig_ring.y;
            float depth = -slabDist;             // distance inward from the slab edge, logical px
            float band = smoothstep(inset - 0.5, inset + 0.5, depth)
                       * (1.0 - smoothstep(inset + width - 0.5, inset + width + 0.5, depth));
            float glow = 0.15 + 0.35 * mat_sig_level;
            if (mat_sig_response.y == 2)
                glow *= 1.0 + mat_sig_breath * mat_sig_level;
            emissive += mat_sig_accent.rgb * glow * band;
        }
```

- [ ] **Step 4: Sweep**

```glsl
        float diag = (p.x + p.y) / (mat_area_size.x + mat_area_size.y);
        for (int k = 0; k < 4; ++k) {
            int sel = k == 0 ? mat_sig_impulse_resp.x : k == 1 ? mat_sig_impulse_resp.y
                    : k == 2 ? mat_sig_impulse_resp.z : mat_sig_impulse_resp.w;
            if (sel != 3)
                continue;
            float env = k == 0 ? mat_sig_impulse_env.x : k == 1 ? mat_sig_impulse_env.y
                      : k == 2 ? mat_sig_impulse_env.z : mat_sig_impulse_env.w;
            float prog = k == 0 ? mat_sig_impulse_prog.x : k == 1 ? mat_sig_impulse_prog.y
                       : k == 2 ? mat_sig_impulse_prog.z : mat_sig_impulse_prog.w;
            vec3 rgb = k == 0 ? mat_sig_impulse_rgb0 : k == 1 ? mat_sig_impulse_rgb1
                     : k == 2 ? mat_sig_impulse_rgb2 : mat_sig_impulse_rgb3;
            float d = (diag - prog) / 0.06;
            emissive += rgb * env * 0.5 * exp(-d * d);
        }

        glassed = vec4(linearToSrgb(transmitted + specular + emissive), 1.0) * coverage;
```

GLES2 requires constant loop bounds and no dynamic vector indexing, which is why the selects are written out.

- [ ] **Step 5: Verify compile and visuals on the nested instance**

Live nested runs never target the desktop session: the nested winit window throttles frame callbacks when unfocused, so a headless Weston host is used. The procedure below is the one Task 12 reuses; every command is run from the worktree root.

```bash
#!/usr/bin/env bash
# Nested signals fixture. Run from the worktree root. Every step polls with a
# bound; nothing sleeps for a fixed guess, and failures are not masked.
set -euo pipefail
cargo build --release
NIRI=$PWD/target/release/niri
WORK=${NIRI_MATERIAL_WORK_ROOT:?set to the evidence root the roughness smoke used}/material-signals-$(git rev-parse --short HEAD)
mkdir -p "$WORK"
cat > "$WORK/config.kdl" <<'EOF'
material "tg" { glass {} }
window-rule { match app-id="^kitty$"; material "tg" }
spawn-at-startup "kitty"
EOF

# Headless host (1280x720, kiosk shell); wait for its socket.
systemd-run --user --unit=signals-headless-weston --collect \
  weston --backend=headless --renderer=gl --shell=kiosk-shell.so \
  --width=1280 --height=720 --socket=signals-headless
for _ in $(seq 50); do [ -S "$XDG_RUNTIME_DIR/signals-headless" ] && break; sleep 0.1; done
[ -S "$XDG_RUNTIME_DIR/signals-headless" ] || { echo "weston socket never appeared" >&2; exit 1; }

# Fresh short runtime dir for the nested compositor (long paths panic on SUN_LEN).
RT=$XDG_RUNTIME_DIR/sig; rm -rf "$RT"; mkdir -p "$RT"
ln -s "$XDG_RUNTIME_DIR/signals-headless" "$RT/signals-headless"

cleanup() {
  kill "${NIRI_PID:-}" 2>/dev/null || true
  wait "${NIRI_PID:-}" 2>/dev/null || true
  systemctl --user stop signals-headless-weston || true
  if ls "$RT"/niri.*.sock >/dev/null 2>&1; then echo "niri socket left behind in $RT" >&2; exit 1; fi
}
trap cleanup EXIT

XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=signals-headless \
  "$NIRI" -c "$WORK/config.kdl" > "$WORK/niri.log" 2>&1 &
NIRI_PID=$!
for _ in $(seq 100); do ls "$RT"/niri.*.sock >/dev/null 2>&1 && break; sleep 0.1; done
export NIRI_SOCKET=$(ls -t "$RT"/niri.*.sock | head -1)   # every niri msg below targets this instance
msg() { "$NIRI" msg "$@"; }
for _ in $(seq 100); do
  WID=$(msg -j windows | jq -r '.[] | select(.app_id=="kitty") | .id' | head -1)
  [ -n "$WID" ] && break; sleep 0.1
done
[ -n "${WID:-}" ] || { echo "kitty window never mapped" >&2; exit 1; }

# Drive the signal; the screenshot is taken 100 ms after the error pulse, at
# the impulse's 80 ms peak.
msg set-window-signal --id "$WID" --source demo --accent '#e5a33c' --level demand --motion pulse
msg pulse-window-signal --id "$WID" --source demo --kind done
sleep 1.6
msg pulse-window-signal --id "$WID" --source demo --kind error
sleep 0.1
SHOT=$WORK/error-flash.png
msg action screenshot-screen --write-to-disk true --show-pointer false --path "$SHOT"
for _ in $(seq 50); do [ -s "$SHOT" ] && break; sleep 0.1; done   # write is asynchronous
[ -s "$SHOT" ] || { echo "screenshot not written" >&2; exit 1; }
msg clear-window-signal --id "$WID" --source demo
sha256sum "$SHOT" | tee -a "$WORK/SHA256SUMS"
```

Expected: an amber ring on the visible slab band, the rim glint swaying, one diagonal sweep, one chromatic jolt captured in `error-flash.png`, then the default look again. Take further screenshots by repeating the `screenshot-screen` line with a new `--path` at each state. The evidence directory and `SHA256SUMS` feed the Task 12 record.

- [ ] **Step 6: Commit**

```bash
tasks done material-4a64bb "feat(material): render ring, rim orbit, and sweep signal responses"
git add src/render_helpers/shaders/material.frag tasks
git commit -m "feat(material): render ring, rim orbit, and sweep signal responses"
```

---

### Task 11: Documentation and task closure

**Files:**
- Modify: `docs/materials/material-config.md`, `docs/materials/README.md`, `docs/materials/2026-09-02-material-signals-design.md` (status header), `docs/wiki/IPC.md`

- [ ] **Step 1: Configuration reference**

Add to `material-config.md`: the `response "name" { }` block with the glass vocabulary and defaults, `ring-inset`/`ring-width` and the rule `ring-inset + ring-width <= bevel` plus the full-ring condition `bevel >= 2 * max(|offset|) + ring-inset + ring-width`, `material "name" response="loud"` on window rules, `signal-source` and `signal-tag` matches, the top-level `signal { motion "full" | "reduced" | "off" }` block, and `animations { material-signal { } }`. Follow the file's existing table style.

- [ ] **Step 2: IPC reference**

In `docs/wiki/IPC.md`, document the three requests with every flag, the `WindowSignalChanged` event, the `signal` field on `Window`, the JSON shape (enum variants in Rust spelling), bounds, and the error cases, in the same style as the existing event-stream section.

- [ ] **Step 3: Design doc alignment**

The design doc was amended before implementation (commit `6c67b026`) to match this plan: one deadline timer per window, time-aware fold, refresh-driven reconciliation, bevel-inflated visibility, and rim light only under `rim-orbit`. Here only update its status header to "implemented on `feat/material-signals` through `<commit>`; verification pending" (Task 12 finalizes it), and add the README ledger line (`../plans/2026-09-02-material-signals.md`, following the roughness entry). If implementation deviated from the design anywhere else, amend the design in this step and say so in the commit.

- [ ] **Step 4: Commit**

```bash
tasks done material-2ecd18 "docs(material): document material signals configuration and IPC"
git add docs tasks
git commit -m "docs(material): document material signals configuration and IPC"
```

---

### Task 12: Verification evidence

**Files:**
- Create: `docs/materials/2026-09-XX-material-signals-smoke.md` (date of the run)
- Modify: design doc status header, `docs/materials/README.md`

- [ ] **Step 1: Unit and fixture suites**

Run: `cargo test -p niri-ipc && cargo test -p niri-config && cargo test --bin niri`
Expected: all PASS. Record the counts.

- [ ] **Step 2: Nested IPC round trip**

Start the nested instance with the Task 10 Step 5 script up to the `msg()` definition (put the driving commands below in place of its signal section; the `trap` cleanup still applies). Record the event stream in the background: `msg -j event-stream > "$WORK/events.jsonl" & EVENTS_PID=$!`. Then set, pulse, clear as in Task 10 Step 5, then `msg set-window-signal --id "$WID" --source demo --ttl-ms 2000 --after-level quiet`, `sleep 2.5`, and `kill "$EVENTS_PID"`. Expected in `events.jsonl`: `WindowSignalChanged` on every write, on impulse expiry about 1.5 s after each pulse, on decay at 2 s, and `"signal":null` after the final clear; verify with `jq -c 'select(.WindowSignalChanged) | .WindowSignalChanged.signal' "$WORK/events.jsonl"`. Also run each rejection and assert a non-zero exit plus no new line in `events.jsonl`: unknown id (`--id 999999`), `--source niri` on set, pulse, and clear, pulse before set (`--source fresh`), `--accent zzz`, `--ttl-ms 90000000 --after-level quiet`, `--after-level quiet` without `--ttl-ms`, and a 17th source (`for i in $(seq 17); do msg set-window-signal --id "$WID" --source "s$i"; done`, expecting the last to fail).

- [ ] **Step 3: GLES smoke with redraw counts**

**Tracy tooling.** This tree's `tracy-client-sys 0.28.0` embeds Tracy 0.13.1 (protocol 76); the installed 0.14 tools cannot read its traces, as the roughness smoke records. Use the protocol-matched tools the roughness run built, verifying them against that record's SHA-256 table:

```bash
TOOLS=${NIRI_MATERIAL_WORK_ROOT:?}/material-roughness-b220152d/tools
sha256sum "$TOOLS/tracy-capture" "$TOOLS/tracy-csvexport"   # must match the roughness smoke table
```

If they are absent, rebuild them from the pinned release before continuing:

```bash
git clone --depth 1 --branch v0.13.1 https://github.com/wolfpld/tracy "$WORK/tracy-src"
cmake -S "$WORK/tracy-src/capture"   -B "$WORK/tracy-capture-build"   -DCMAKE_BUILD_TYPE=Release && cmake --build "$WORK/tracy-capture-build"
cmake -S "$WORK/tracy-src/csvexport" -B "$WORK/tracy-csvexport-build" -DCMAKE_BUILD_TYPE=Release && cmake --build "$WORK/tracy-csvexport-build"
TOOLS=$WORK/tools; mkdir -p "$TOOLS"; cp "$WORK"/tracy-capture-build/tracy-capture "$WORK"/tracy-csvexport-build/tracy-csvexport "$TOOLS"/
```

Build the profiling binary with `cargo build --release --features profile-with-tracy` and run the Task 10 Step 5 script with `NIRI=$PWD/target/release/niri` from that build. For each case: apply the case setup, start a 30 s capture, and count `Niri::redraw` zones in the steady-state window, defined as the final 20 s of the trace, so the first 10 s absorb setup events and the trace's retained earlier history is excluded by construction:

```bash
count_steady() {  # $1 = case name; prints Niri::redraw zones in the last 20 s and the rate per second
  local CASE=$1 CAP="$WORK/$1.tracy"
  "$TOOLS/tracy-capture" -o "$CAP" -a 127.0.0.1 -s 30
  "$TOOLS/tracy-csvexport" --unwrap -f "Niri::redraw" "$CAP" > "$WORK/$CASE.csv"
  # --unwrap emits one row per zone with ns_since_start in column 4; without it the rows are aggregates.
  awk -F, 'NR>1 { t=$4+0; if (t>max) max=t; ts[NR]=t }
           END { lo=max-20e9; n=0; for (k in ts) if (ts[k]>=lo) n++; printf "%s: %d zones in 20 s = %.1f/s\n", "'"$CASE"'", n, n/20 }' "$WORK/$CASE.csv" | tee -a "$WORK/rates.txt"
}
```

**Case setup.** All commands go through `msg` against `$NIRI_SOCKET`. Helper to open more kitty windows and wait for them: `spawn_kitty() { msg action spawn -- kitty; for _ in $(seq 100); do [ "$(msg -j windows | jq '[.[] | select(.app_id=="kitty")] | length')" -ge "$1" ] && return; sleep 0.1; done; echo "kitty $1 never mapped" >&2; exit 1; }`. Config variants are separate files applied with `msg action load-config-file "$WORK/<variant>.kdl"`; each variant is the base config plus the lines named. After every setup, assert the state with the `jq` line given, then call `count_steady <case>`.

| Case | Setup and state assertion |
|---|---|
| quiet-ring | `msg set-window-signal --id "$WID" --source demo --accent '#e5a33c'`; assert `msg -j windows \| jq '.[] \| select(.id=='"$WID"') \| .signal.level' == "Quiet"` |
| demand-pulse | `spawn_kitty 2`; `OTHER=$(msg -j windows \| jq '.[] \| select(.app_id=="kitty" and .id!='"$WID"') \| .id')`; `msg set-window-signal --id "$WID" --source demo --level demand --motion pulse --until-focus`; `msg action focus-window --id "$OTHER"`; assert `.is_focused==false` for `$WID` |
| demand-pulse-focused | continue from demand-pulse: `msg action focus-window --id "$WID"`; assert `.signal.level=="Quiet"` for `$WID` (until-focus demoted it) |
| demand-breathe / demand-flash | as demand-pulse with `--motion breathe` / `--motion flash` |
| ten-breathe / ten-flash | `spawn_kitty 10`; `for w in $(msg -j windows \| jq '.[] \| select(.app_id=="kitty") \| .id'); do msg set-window-signal --id "$w" --source demo --level demand --motion breathe; done` (or `flash`); focus a non-kitty-free state is not needed since all ten are visible or tabbed; assert ten windows report `.signal.motion=="Breathe"` |
| inactive-workspace | from demand-pulse: `msg action focus-window --id "$WID"; msg action move-window-to-workspace-down --focus false`; assert `$WID`'s `.workspace_id` differs from `$OTHER`'s and `$OTHER` `.is_focused==true` |
| hidden-tab | from demand-pulse: `msg action focus-window --id "$OTHER"; msg action consume-or-expel-window-left; msg action toggle-column-tabbed-display; msg action focus-window --id "$OTHER"` so both share one tabbed column and `$OTHER` is the shown tab; assert both have equal `.layout.pos_in_scrolling_layout[0]` and `$WID` `.is_focused==false` |
| offscreen-column | from demand-pulse with `$WID` in the leftmost column: `spawn_kitty 3; spawn_kitty 4;` then `msg action focus-column-last`; with default 0.5-proportion columns on a 1280 px view the leftmost column is scrolled out; assert `$WID` `.layout.tile_pos_in_workspace_view[0] + .layout.tile_size[0] <= 0` |
| motion-off | `load-config-file` variant with `signal { motion "off" }`; set demand-pulse on `$WID` and two `pulse ... --kind done`; assert `.signal.motion=="Pulse"` (stored) while the capture shows no recurrence |
| reduced-flash | variant with `signal { motion "reduced" }`; set `--motion flash`; expect the Pulse rate |
| attention-none | variant whose `response "default" { attention "none" }`; set demand-pulse |
| impulse-none | variant whose `response "default" { ping "none"; done "none"; error "none" }`; set the slot, then pulse `done` three times inside the first 10 s |
| done-pulse | set the slot; pulse `done` once inside the first 10 s |
| slowdown | variant with `animations { slowdown 3 }`; repeat demand-pulse |

The contract from Task 9: one coalescible redraw request per output per mutation and per deadline firing; steady state is what the 20 s window measures.

| Case | Expected `Niri::redraw` zones in the final 20 s |
|---|---|
| quiet-ring | 0 |
| demand-pulse | about 540 (27/s) |
| demand-pulse-focused | 0 |
| demand-breathe | about 160 (8/s) |
| demand-flash | about 320 (16/s) |
| ten-breathe, ten-flash | within 10% of the single-window case |
| inactive-workspace, hidden-tab, offscreen-column | 0 |
| motion-off | 0 |
| reduced-flash | about 540 (the Pulse rate) |
| attention-none | 0 |
| impulse-none | 0 (the per-pulse and per-expiry requests all fall in the first 10 s) |
| done-pulse | 0 in the final 20 s; additionally, in the first 10 s, a burst lasting about 1.5 s after the pulse |
| error flash on default glass | visibly jolts in the Task 10 screenshot |
| slowdown | rates equal to demand-pulse |

Record per-frame GPU `draw shader` cost of ring plus rim orbit against the roughness baseline the same way the roughness smoke did (matched steady-state samples, median, delta). Stop the instance and the Weston unit after the last case.

- [ ] **Step 4: Physical DRM check**

Install the Arch package per `docs/materials/2026-08-28-v1-daily-driver-rollout-design.md`, leave a `Demand + Pulse` window unfocused for ten minutes, and confirm no regressions in the daily-driver session. Record the build hash.

- [ ] **Step 5: Close out**

Write the smoke doc, set the design doc status to implemented and verified with the commit and evidence link, add the README ledger lines, then:

```bash
tasks done material-b43616 "Verification evidence recorded in docs/materials/<smoke>.md"
tasks done material-a54d89 "Material signals landed on feat/material-signals; evidence in docs/materials/<smoke>.md"
git add docs tasks
git commit -m "docs(material): record material signals acceptance"
```

---

## Self-review notes

- Spec coverage: §1 Tasks 4 and 5; §2 Tasks 1 and 6; §3 Task 7; §4 Tasks 7 and 9; §5 Tasks 2 and 3; §6 Tasks 8 and 10; §7 the file table; §8 Tasks 3, 4, and 6; §9 Tasks 6, 9, and 12; §10 Task 11.
- Deliberate simplifications recorded for the docs task: one deadline timer per window reconciled from `State::refresh` (§1), time-aware fold (§1), bevel-inflated visibility rect (§4), and breath-driven rim sway under `rim-orbit` only (§6). All keep the spec's cost bounds.
- Types are consistent across tasks: `Folded`, `EffectiveSignal`, `SignalFrame`, `ImpulseFrame`, `GlassSignalInputs`, `SignalUniforms`, `SignalFingerprint`, `SignalTicks`, `SetWindowSignalArgs`, `to_ipc_signal`.
