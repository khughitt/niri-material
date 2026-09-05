# Glass noise and saturation parameters: implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `glass { noise; saturation }` become optional material parameters whose written values always render and whose omission keeps today's inheritance from the global `blur` block.

**Architecture:** The config crate gains two `Option` fields on `Glass` and `ResolvedGlass`. `resolve_material` in the layout becomes the single place that turns "written or inherited" into the `(noise, saturation)` pair the renderer already consumes. The shader, damage tracking and uniforms do not change.

**Tech Stack:** Rust (niri fork), knuffel KDL config, `just` front door (`just test-fast`, `just check`, `just test`), Weston headless GL + kitty + ImageMagick for the nested smoke.

**Spec:** `docs/specs/2026-09-05-material-glass-noise-saturation-params-design.md`

**Task:** `material-1293e8` (native half of Prism goal `prism-63dd45`)

## Global Constraints

- Ranges are exactly `noise` 0–1 and `saturation` 0–3; out-of-range values are parse errors.
- A written value applies regardless of `backdrop-blur` and regardless of `blur { off }`.
- Omitted values resolve to `None` in `ResolvedGlass`; the layout applies `?? (effective ? global : neutral)` with neutral `(0, 1)`.
- Test entry points are `just test-fast` (inner loop: the packages changed against HEAD and their dependents), `just check` and `just test`. Never run bare `cargo test` for any step in this plan; the front door records timing and enforces suite selection.
- Live nested runs use a headless Weston unit, never the desktop session.
- Work happens in the worktree `.worktrees/glass-noise-saturation` on branch `glass-noise-saturation`.
- Never edit `tasks/*.md` by hand; use the `tasks` CLI. `tasks check` runs inside `just check`.
- Each `### Task N` heading has a child task under `material-1293e8`. Its first step is `tasks start <child>`; its commit step runs `tasks done <child>` and stages `tasks/` alongside the code, so every commit carries its own task record. Task 5 closes its own child before it closes `material-1293e8`.
- Commit messages are conventional commits. No attribution trailer of any kind.

| Task | Child |
| --- | --- |
| 1 | `material-9f91c9` |
| 2 | `material-93131b` |
| 3 | `material-f5ccb3` |
| 4 | `material-4af245` |
| 5 | `material-4df34c` |

---

### Task 1: Config grammar

**Files:**
- Modify: `niri-config/src/material.rs:377-409` (`Glass`), `:433-451` (`ResolvedGlass`), `:453-473` (`Default`), `:507-530` (`resolve`)
- Test: `niri-config/src/lib.rs:1092-1190`

**Interfaces:**
- Produces: `niri_config::ResolvedGlass { noise: Option<f64>, saturation: Option<f64>, .. }`. Task 2 reads these two fields.

- [ ] **Step 0: Start the child task**

Run: `tasks start material-9f91c9`

- [ ] **Step 1: Write the failing tests**

In `niri-config/src/lib.rs`, inside the existing `mod tests` next to `backdrop_blur_parses_and_resolves`, add:

```rust
    #[test]
    fn glass_noise_and_saturation_parse_as_written() {
        let parsed = do_parse(
            r##"
            material "frost" {
                glass {
                    noise 0.02
                    saturation 0.85
                }
            }
            "##,
        );
        let glass = parsed.materials[0].resolve().glass;
        assert_eq!(glass.noise, Some(0.02));
        assert_eq!(glass.saturation, Some(0.85));
    }

    #[test]
    fn glass_noise_and_saturation_resolve_independently() {
        let noise_only = do_parse(r##"material "frost" { glass { noise 0.5; }; }"##);
        let glass = noise_only.materials[0].resolve().glass;
        assert_eq!(glass.noise, Some(0.5));
        assert_eq!(glass.saturation, None);

        let saturation_only = do_parse(r##"material "frost" { glass { saturation 0; }; }"##);
        let glass = saturation_only.materials[0].resolve().glass;
        assert_eq!(glass.noise, None);
        assert_eq!(glass.saturation, Some(0.));
    }

    #[test]
    fn glass_noise_and_saturation_default_to_inherit() {
        let d = ResolvedGlass::default();
        assert_eq!(d.noise, None);
        assert_eq!(d.saturation, None);
    }

    #[test]
    fn glass_noise_rejects_values_outside_zero_and_one() {
        for value in ["-0.01", "1.01"] {
            let err = do_parse_err(&format!(
                "material \"frost\" {{ glass {{ noise {value}; }}; }}\n"
            ));
            assert!(err.contains("value must be between 0 and 1"), "{err}");
        }
    }

    #[test]
    fn glass_saturation_rejects_values_outside_zero_and_three() {
        for value in ["-0.01", "3.01"] {
            let err = do_parse_err(&format!(
                "material \"frost\" {{ glass {{ saturation {value}; }}; }}\n"
            ));
            assert!(err.contains("value must be between 0 and 3"), "{err}");
        }
    }
```

Also extend the `ResolvedGlass { .. }` literal in `material_parses_full_glass_block` (around line 1120) with two lines so it keeps compiling once the struct grows:

```rust
                offset_y: 4.,
                noise: None,
                saturation: None,
```

- [ ] **Step 2: Run the inner loop to verify it fails**

Run: `just test-fast`
Expected: the selection is `niri-config` and its dependents; compilation of the `niri-config` test target fails with `no field noise on type ResolvedGlass`.

- [ ] **Step 3: Add the fields**

In `niri-config/src/material.rs`, `Glass` (after `roughness`):

```rust
    #[knuffel(child, unwrap(argument))]
    pub noise: Option<FloatOrInt<0, 1>>,
    #[knuffel(child, unwrap(argument))]
    pub saturation: Option<FloatOrInt<0, 3>>,
```

`ResolvedGlass` (after `offset_y`):

```rust
    /// Post-optics noise amplitude. `None` means inherit: the global `blur`
    /// block's value while backdrop blur is effective, neutral otherwise. A
    /// written value applies regardless of either switch. The layout applies
    /// the rule, since only it knows the global block.
    pub noise: Option<f64>,
    /// Post-optics saturation factor, with the same inheritance rule as
    /// `noise`.
    pub saturation: Option<f64>,
```

`Default for ResolvedGlass` (after `offset_y: 6.,`):

```rust
            noise: None,
            saturation: None,
```

`Material::resolve`, inside the `ResolvedGlass { .. }` literal (after `offset_y`):

```rust
                noise: g.noise.map(|x| x.0),
                saturation: g.saturation.map(|x| x.0),
```

- [ ] **Step 4: Run the inner loop to verify it passes**

Run: `just test-fast`
Expected: exit 0; the nextest summary lists `glass_noise_and_saturation_parse_as_written`, `glass_noise_and_saturation_resolve_independently`, `glass_noise_and_saturation_default_to_inherit`, `glass_noise_rejects_values_outside_zero_and_one`, `glass_saturation_rejects_values_outside_zero_and_three` and the pre-existing `material_parses_full_glass_block` as PASS.

- [ ] **Step 5: Fix any other `ResolvedGlass` literal**

Run: `rg -n 'ResolvedGlass \{' src niri-config/src`
Expected: only `material.rs` (struct, `Default`, `resolve`), `lib.rs:1120` (fixed in Step 1), and `src/layout/tile.rs:2199`, which uses `..Default::default()` and needs nothing. If anything else appears, add `noise: None, saturation: None` or `..Default::default()` and rerun `just test-fast`.

- [ ] **Step 6: Check, close the child, commit**

Run: `cargo fmt --all && just check`
Expected: exit 0 (clippy warnings pre-exist on this branch; none from the changed files).

```bash
tasks done material-9f91c9 "Glass and ResolvedGlass carry optional noise and saturation; parse and range tests pass"
git add niri-config/src/material.rs niri-config/src/lib.rs tasks/
git commit -m "feat(config): optional glass noise and saturation parameters

glass { noise; saturation } join the material grammar as Option fields on
Glass and ResolvedGlass. None means inherit; the layout applies the rule."
```

---

### Task 2: Independent resolution in the layout

**Files:**
- Modify: `src/layout/tile.rs:186-206` (`resolve_material`), `:2192-2216` (test helper)
- Test: `src/layout/tile.rs` `mod tests`

**Interfaces:**
- Consumes: `ResolvedGlass::noise`, `ResolvedGlass::saturation` from Task 1.
- Produces: unchanged `MaterialRenderConfig { material, noise: f32, saturation: f32 }`.

- [ ] **Step 0: Start the child task**

Run: `tasks start material-93131b`

- [ ] **Step 1: Generalise the test helper and write the failing matrix test**

Replace the `options_with` helper at the top of `mod tests` in `src/layout/tile.rs` with a two-argument builder and keep `options_with` as a thin wrapper so the existing tests do not move:

```rust
    fn options_for(glass: niri_config::ResolvedGlass, blur: niri_config::Blur) -> Options {
        let material = niri_config::ResolvedMaterial {
            name: String::from("frost"),
            glass,
            responses: vec![(
                String::from("default"),
                niri_config::ResolvedResponse::default(),
            )],
        };
        Options {
            materials: Rc::new(HashMap::from([(String::from("frost"), material)])),
            blur,
            ..Default::default()
        }
    }

    fn options_with(name: &str, backdrop_blur: bool, blur_off: bool) -> Options {
        assert_eq!(name, "frost", "the fixture defines one material");
        options_for(
            niri_config::ResolvedGlass {
                backdrop_blur,
                ..Default::default()
            },
            niri_config::Blur {
                off: blur_off,
                ..Default::default()
            },
        )
    }
```

Then add, after `material_postprocess_follows_effective_backdrop_blur`:

```rust
    #[test]
    fn written_noise_and_saturation_resolve_independently_of_each_other_and_of_blur() {
        let reference = MaterialRef {
            name: String::from("frost"),
            response: None,
        };
        // Non-neutral globals so "inherited" and "neutral" are distinguishable
        // from "written", and so an explicit neutral value is visibly a choice.
        let global = niri_config::Blur {
            noise: 0.02,
            saturation: 1.5,
            ..Default::default()
        };
        let global_off = niri_config::Blur {
            off: true,
            ..global
        };
        // (written noise, written saturation, backdrop-blur, blur block, expected pair)
        for (noise, saturation, backdrop_blur, blur, expected) in [
            // both written: the pair survives every switch
            (Some(0.3), Some(0.5), true, global, (0.3, 0.5)),
            (Some(0.3), Some(0.5), false, global, (0.3, 0.5)),
            (Some(0.3), Some(0.5), true, global_off, (0.3, 0.5)),
            // both omitted: today's inheritance
            (None, None, true, global, (0.02, 1.5)),
            (None, None, false, global, (0., 1.)),
            (None, None, true, global_off, (0., 1.)),
            // one written, one omitted: each side decided on its own
            (Some(0.3), None, true, global, (0.3, 1.5)),
            (Some(0.3), None, false, global, (0.3, 1.)),
            (None, Some(0.5), true, global, (0.02, 0.5)),
            (None, Some(0.5), false, global, (0., 0.5)),
            // explicit neutral beats non-neutral globals
            (Some(0.), Some(1.), true, global, (0., 1.)),
        ] {
            let options = options_for(
                niri_config::ResolvedGlass {
                    noise,
                    saturation,
                    backdrop_blur,
                    ..Default::default()
                },
                blur,
            );
            let resolved = resolve_material(Some(&reference), &options).unwrap();
            assert_eq!(
                (resolved.noise, resolved.saturation),
                expected,
                "noise {noise:?} saturation {saturation:?} backdrop {backdrop_blur} off {}",
                blur.off
            );
        }
    }
```

- [ ] **Step 2: Run the inner loop to verify it fails**

Run: `just test-fast`
Expected: `written_noise_and_saturation_resolve_independently_of_each_other_and_of_blur` FAILS on the first "both written" row with `left: (0.02, 1.5)`, `right: (0.3, 0.5)`, because the current code ignores written values. Every other test in the selection passes.

- [ ] **Step 3: Apply the rule in `resolve_material`**

Replace the `let (noise, saturation) = if backdrop_blur { .. } else { .. };` block in `resolve_material` with:

```rust
    // A written value is a material optic and renders as written. Only an
    // omitted value inherits, and only while backdrop blur is effective;
    // otherwise it is neutral. Each parameter decides on its own.
    let inherited = |global: f64, neutral: f64| if backdrop_blur { global } else { neutral };
    let noise = material
        .glass
        .noise
        .unwrap_or_else(|| inherited(options.blur.noise, 0.)) as f32;
    let saturation = material
        .glass
        .saturation
        .unwrap_or_else(|| inherited(options.blur.saturation, 1.)) as f32;
```

Append to the doc comment above `resolve_material`, after the sentence about the global `blur { off }` switch:

```rust
/// The same goes for `noise` and `saturation`: a written glass value wins,
/// an omitted one inherits the global value only while backdrop blur is
/// effective, and the renderer receives one final pair.
```

- [ ] **Step 4: Run the inner loop to verify it passes**

Run: `just test-fast`
Expected: exit 0. The summary lists as PASS the new matrix test and the pre-existing layout tests it must not regress: `backdrop_blur_survives_resolution_when_global_blur_is_on`, `backdrop_blur_is_off_when_the_global_switch_is_off`, `backdrop_blur_stays_off_when_the_material_opts_out`, `material_postprocess_follows_effective_backdrop_blur`, `a_name_the_config_does_not_define_resolves_to_nothing`, and in `render_helpers::material` `postprocess_change_advances_the_commit_in_place`.

- [ ] **Step 5: Check, close the child, commit**

Run: `cargo fmt --all && just check`
Expected: exit 0.

```bash
tasks done material-93131b "resolve_material prefers written noise and saturation; the 11-row matrix passes"
git add src/layout/tile.rs tasks/
git commit -m "feat(layout): resolve written glass noise and saturation

A written glass value renders regardless of backdrop-blur and blur { off };
an omitted one inherits the global value only while backdrop blur is
effective. Each parameter decides independently."
```

---

### Task 3: Documentation

**Files:**
- Modify: `docs/materials/material-config.md:35-67`
- Modify: `docs/specs/2026-09-02-material-noise-saturation-design.md:1-6`
- Modify: `docs/materials/2026-08-29-material-backdrop-blur-design.md:348-354`
- Modify: `docs/materials/README.md:26-28`

- [ ] **Step 0: Start the child task**

Run: `tasks start material-f5ccb3`

- [ ] **Step 1: Material grammar reference**

In `docs/materials/material-config.md`, add two table rows after the `roughness` row:

```markdown
| `noise` | float | inherit | 0–1 | — |
| `saturation` | float | inherit | 0–3 | — |
```

Replace the paragraph beginning "When `backdrop-blur` is effective, glass also inherits `noise` and `saturation`..." with:

```markdown
`noise` and `saturation` are applied after the glass optics: saturation
first, then screen-space noise. A written value is a material optic and
applies regardless of `backdrop-blur` and of `blur { off }`. An omitted
value inherits the global `blur` block's `noise` or `saturation` while
`backdrop-blur` is effective and is neutral otherwise (`noise 0`,
`saturation 1`); each parameter decides on its own, so `blur { off }` and
material opt-out neutralise only inherited values. Per-window
`background-effect` overrides remain independent and do not alter the
material.
```

- [ ] **Step 2: Predecessor spec status**

In `docs/specs/2026-09-02-material-noise-saturation-design.md`, append to the `**Status:**` paragraph:

```markdown
Superseded in part on 2026-09-05: the "no glass-specific parameters" decision
gave way to optional `glass { noise; saturation }` in
[`2026-09-05-material-glass-noise-saturation-params-design.md`](2026-09-05-material-glass-noise-saturation-params-design.md).
The composition order and inheritance for omitted values are unchanged.
```

- [ ] **Step 3: Backdrop-blur design pointer**

In `docs/materials/2026-08-29-material-backdrop-blur-design.md`, at the end of the first paragraph of "## Follow-up: noise and saturation" (after "...without restoring the redundant background-effect pass."), add:

```markdown
Since 2026-09-05 a material may also write its own `noise` and `saturation`
(`../specs/2026-09-05-material-glass-noise-saturation-params-design.md`);
inheritance is the omitted-value behaviour.
```

- [ ] **Step 4: README index**

In `docs/materials/README.md`, after the `2026-09-02-material-noise-saturation-evidence.md` entry add:

```markdown
- `../specs/2026-09-05-material-glass-noise-saturation-params-design.md`: optional per-material `noise` and `saturation` design; written values always apply.
```

(The evidence and script entries are added in Task 4 when they exist.)

- [ ] **Step 5: Grep for the stale claim**

Run: `rg -n 'inherits .noise. and .saturation.|supplies blur strength, saturation, and noise|makes both values neutral' docs README.md wiki 2>/dev/null`
Expected: no hit outside the historical spec and evidence documents (`2026-09-02-*`), which describe their own commit and stay as written.

- [ ] **Step 6: Check, close the child, commit**

Run: `just check`
Expected: exit 0 (a docs-only change; `test-affected` would select nothing).

```bash
tasks done material-f5ccb3 "material-config, predecessor spec, backdrop-blur design and README describe written noise and saturation"
git add docs/materials/material-config.md docs/specs/2026-09-02-material-noise-saturation-design.md docs/materials/2026-08-29-material-backdrop-blur-design.md docs/materials/README.md tasks/
git commit -m "docs(materials): document written glass noise and saturation"
```

---

### Task 4: Nested GLES smoke and evidence

**Files:**
- Create: `docs/materials/scripts/glass-noise-saturation-smoke.sh`
- Create: `docs/materials/2026-09-05-material-glass-noise-saturation-params-evidence.md`
- Modify: `docs/materials/README.md` (two index entries)

**Interfaces:**
- Consumes: a debug `niri` built from this branch and one built from the merge base `7185ee51`.

- [ ] **Step 0: Start the child task**

Run: `tasks start material-4af245`

- [ ] **Step 1: Build both binaries**

```bash
W=/mnt/ssd/Dropbox/niri-material/.worktrees/glass-noise-saturation
cargo build --manifest-path "$W/Cargo.toml" --target-dir /mnt/ssd3/tmp/material-1293e8-impl-target
git -C /mnt/ssd/Dropbox/niri-material worktree add /mnt/ssd3/tmp/material-1293e8-base 7185ee51
cargo build --manifest-path /mnt/ssd3/tmp/material-1293e8-base/Cargo.toml --target-dir /mnt/ssd3/tmp/material-1293e8-base-target
```

Expected: both `debug/niri` binaries exist. (Building, unlike testing, has no front-door recipe; `cargo build` is the right tool here.)

- [ ] **Step 2: Write the smoke script**

Create `docs/materials/scripts/glass-noise-saturation-smoke.sh` and `chmod 755` it:

```bash
#!/usr/bin/env bash
# Glass noise and saturation parameters smoke (material-1293e8): capture a
# blank transparent kitty over a glass material on a headless Weston host and
# assert that written noise/saturation render with backdrop-blur off and with
# blur { off }, and that omitted values are byte-identical to the pre-change
# binary. Exit 0 means every assertion held; any failure exits non-zero with
# a FAIL line and the trap preserves that status.
#
# Env: IMPL (implementation niri), BASE (pre-change niri), OUT (artifact dir).
# Requires: weston, kitty, swaybg, jq, rg, ImageMagick.
set -eu
IMPL=${IMPL:?implementation niri binary}
BASE=${BASE:?pre-change niri binary}
OUT=${OUT:?artifact directory}
mkdir -p "$OUT"
# One short, unique runtime dir per run: nested niri panics on long socket
# paths, and concurrent runs must never share or delete each other's sockets.
RT=$(mktemp -d "$XDG_RUNTIME_DIR/gns.XXXXXX")
RUN=$(basename "$RT")
HOST=$RUN-host; UNIT=$RUN-weston; NIRI_PID=
WALL=$OUT/color-bars.png
magick -size 425x720 xc:'rgb(255,32,32)' -size 427x720 xc:'rgb(32,255,32)' -size 428x720 xc:'rgb(32,32,255)' +append "$WALL"

fail() { echo "FAIL: $*" >&2; exit 1; }
cleanup() {
    local rc=$?
    if [ -n "$NIRI_PID" ]; then kill "$NIRI_PID" 2>/dev/null || true; wait "$NIRI_PID" 2>/dev/null || true; fi
    systemctl --user stop "$UNIT" 2>/dev/null || true
    rm -rf "$RT"
    if [ -S "$XDG_RUNTIME_DIR/$HOST" ]; then echo "WARN: weston socket $HOST still present" >&2; fi
    exit "$rc"
}
trap cleanup EXIT

write_config() {   # $1 path, $2 glass extra lines, $3 blur extra lines
    cat > "$1" <<KDL
prefer-no-csd
layout {
    gaps 40
    background-color "transparent"
    default-column-width { proportion 0.6; }
    focus-ring { off; }
    border { off; }
    shadow { off; }
}
animations { off; }
hotkey-overlay { skip-at-startup; }
config-notification { disable-failed; }
spawn-at-startup "swaybg" "-m" "fill" "-i" "$WALL"
blur {
    passes 1
    offset 8
    noise 0.08
    saturation 1.5
    $3
}
material "gns-probe" {
    glass {
        ior 1.5
        thickness 20
        attenuation-color "#dfe8ff"
        attenuation-distance 60
        chromatic-aberration 0
        distortion 0 scale=0.5
        anisotropic-blur 0
        roughness 0
        backdrop-blur false
        jelly-flex 0
        jelly-ripple 0
        bevel 12
        offset-x 6
        offset-y 6
        $2
    }
}
window-rule {
    match app-id="^gns-probe$"
    material "gns-probe"
    geometry-corner-radius 0
    background-effect {
        blur false
        noise 0
        saturation 1
    }
}
KDL
}

start_nested() {   # $1 niri binary, $2 config
    systemd-run --user --unit="$UNIT" --collect weston --backend=headless --renderer=gl \
        --shell=kiosk-shell.so --width=1280 --height=720 --socket="$HOST" >/dev/null 2>&1
    for _ in $(seq 100); do [ -S "$XDG_RUNTIME_DIR/$HOST" ] && break; sleep 0.1; done
    [ -S "$XDG_RUNTIME_DIR/$HOST" ] || fail "no Weston socket"
    ln -sf "$XDG_RUNTIME_DIR/$HOST" "$RT/$HOST"
    "$1" validate -c "$2" || fail "config $2 does not validate with $1"
    XDG_RUNTIME_DIR=$RT WAYLAND_DISPLAY=$HOST "$1" -c "$2" >> "$OUT/niri.log" 2>&1 &
    NIRI_PID=$!
    for _ in $(seq 100); do ls "$RT"/niri.*.sock >/dev/null 2>&1 && break; sleep 0.1; done
    NIRI_SOCKET=$(ls -t "$RT"/niri.*.sock | head -1) || fail "no niri socket"
    export NIRI_SOCKET
    sleep 1
}
stop_nested() {
    kill "$NIRI_PID" 2>/dev/null || true; wait "$NIRI_PID" 2>/dev/null || true; NIRI_PID=
    systemctl --user stop "$UNIT" 2>/dev/null || true
    rm -f "$RT"/niri.*.sock "$RT/$HOST"; sleep 0.5
}
msg() { "$1" msg "${@:2}"; }
probe_count() { msg "$1" -j windows | jq -r '.[] | select(.app_id=="gns-probe") | .id' | wc -l; }

capture() {   # $1 name, $2 niri binary, $3 glass extra, $4 blur extra
    local kdl=$OUT/$1.kdl png=$OUT/$1.png
    write_config "$kdl" "$3" "$4"
    start_nested "$2" "$kdl"
    msg "$2" action spawn -- kitty --config NONE --class gns-probe -o background_opacity=0 \
        -o cursor_blink_interval=0 sh -c 'printf "\033[?25l"; exec sleep 600'
    for _ in $(seq 100); do [ "$(probe_count "$2")" -ge 1 ] && break; sleep 0.1; done
    [ "$(probe_count "$2")" -eq 1 ] || fail "expected exactly one probe window for $1"
    sleep 2
    msg "$2" action screenshot-screen --write-to-disk true --show-pointer false --path "$png"
    for _ in $(seq 50); do [ -s "$png" ] && break; sleep 0.1; done
    [ -s "$png" ] || fail "capture $1 not written"
    magick "$png" -crop 200x400+100+160 +repage "$OUT/$1-roi.png"
    stop_nested
}

# ImageMagick exits 1 for a non-identical pair and prints the metric on stderr;
# these helpers return the number and leave the judgement to the assertions.
ae() { magick compare -metric AE "$1" "$2" null: 2>&1 >/dev/null || true; }
rmse() { magick compare -metric RMSE "$1" "$2" null: 2>&1 >/dev/null | awk '{print $1}' || true; }
sd() { magick "$1" -colorspace Gray -format '%[fx:standard_deviation]' info:; }
assert_zero() { [ "$2" = "0" ] || fail "$1 expected 0, got $2"; }
assert_positive() { awk -v v="$2" 'BEGIN { exit !(v > 0) }' || fail "$1 expected > 0, got $2"; }
assert_greater() { awk -v a="$2" -v b="$3" 'BEGIN { exit !(a > b) }' || fail "$1 expected $2 > $3"; }

sha256sum "$IMPL" "$BASE" > "$OUT/binaries.sha256"
"$BASE" --version > "$OUT/base.version"; "$IMPL" --version > "$OUT/impl.version"

capture omitted-before   "$BASE" "" ""
capture omitted-after    "$IMPL" "" ""
capture written-neutral  "$IMPL" $'noise 0\n        saturation 1' ""
capture written-sat-0    "$IMPL" $'noise 0\n        saturation 0' ""
capture written-noise-05 "$IMPL" $'noise 0.5\n        saturation 1' ""
capture written-blur-off "$IMPL" $'noise 0.5\n        saturation 0' "off"

omitted_identity_ae=$(ae "$OUT/omitted-before.png" "$OUT/omitted-after.png")
written_neutral_vs_omitted_ae=$(ae "$OUT/written-neutral.png" "$OUT/omitted-after.png")
for ch in R G B; do magick "$OUT/written-sat-0-roi.png" -channel $ch -separate "$OUT/sat0-$ch.png"; done
sat0_rg_ae=$(ae "$OUT/sat0-R.png" "$OUT/sat0-G.png")
sat0_rb_ae=$(ae "$OUT/sat0-R.png" "$OUT/sat0-B.png")
noise_roi_rmse=$(rmse "$OUT/written-neutral-roi.png" "$OUT/written-noise-05-roi.png")
neutral_sd=$(sd "$OUT/written-neutral-roi.png")
noise05_sd=$(sd "$OUT/written-noise-05-roi.png")
for ch in R G B; do magick "$OUT/written-blur-off-roi.png" -channel $ch -separate "$OUT/bluroff-$ch.png"; done
bluroff_rg_ae=$(ae "$OUT/bluroff-R.png" "$OUT/bluroff-G.png")
bluroff_vs_omitted_ae=$(ae "$OUT/written-blur-off.png" "$OUT/omitted-after.png")

for v in omitted_identity_ae written_neutral_vs_omitted_ae sat0_rg_ae sat0_rb_ae noise_roi_rmse neutral_sd noise05_sd bluroff_rg_ae bluroff_vs_omitted_ae; do
    printf '%s=%s\n' "$v" "${!v}"
done | tee "$OUT/metrics.txt"

assert_zero omitted_identity_ae "$omitted_identity_ae"                    # omitted values: byte-identical to the pre-change binary
assert_zero written_neutral_vs_omitted_ae "$written_neutral_vs_omitted_ae"  # explicit neutral beats non-neutral globals, backdrop-blur off
assert_zero sat0_rg_ae "$sat0_rg_ae"                                      # written saturation 0 renders grayscale with backdrop-blur off
assert_zero sat0_rb_ae "$sat0_rb_ae"
assert_positive noise_roi_rmse "$noise_roi_rmse"                          # written noise changes the ROI
assert_greater noise_sd "$noise05_sd" "$neutral_sd"                       # and raises its variance
assert_zero bluroff_rg_ae "$bluroff_rg_ae"                                # written values survive blur { off }
assert_positive bluroff_vs_omitted_ae "$bluroff_vs_omitted_ae"

sha256sum "$OUT"/*.png "$OUT"/*.kdl >> "$OUT/SHA256SUMS"
if rg -n 'material.*(error|fallback)|error compiling material shader|panic' "$OUT/niri.log"; then
    fail "material error, fallback or panic in niri.log"
fi
echo "PASS: artifacts in $OUT"
```

- [ ] **Step 3: Run it**

```bash
OUT=$(mktemp -d /mnt/ssd3/tmp/material-1293e8-smoke.XXXXXX)
IMPL=/mnt/ssd3/tmp/material-1293e8-impl-target/debug/niri \
BASE=/mnt/ssd3/tmp/material-1293e8-base-target/debug/niri \
OUT=$OUT bash docs/materials/scripts/glass-noise-saturation-smoke.sh; echo "exit=$?"
cat "$OUT/metrics.txt"
```

Expected: the script prints `PASS: artifacts in ...` and `exit=0`. The assertions it enforces are:

| Assertion | Meaning |
| --- | --- |
| `omitted_identity_ae` is 0 | omitted values are byte-identical to the pre-change binary |
| `written_neutral_vs_omitted_ae` is 0 | explicit `noise 0; saturation 1` overrides the non-neutral globals and renders with backdrop-blur off |
| `sat0_rg_ae`, `sat0_rb_ae` are 0 | written `saturation 0` renders grayscale with backdrop-blur off |
| `noise_roi_rmse` > 0 and `noise05_sd` > `neutral_sd` | written noise changes the ROI and raises its variance |
| `bluroff_rg_ae` is 0 and `bluroff_vs_omitted_ae` > 0 | written values survive `blur { off }` |
| log gate | no material error, fallback or panic |

A FAIL line means stop and debug with superpowers:systematic-debugging; do not soften an assertion.

- [ ] **Step 4: Write the evidence document**

Create `docs/materials/2026-09-05-material-glass-noise-saturation-params-evidence.md` with this shape, filled from the run:

```markdown
# Glass noise and saturation parameters: verification evidence

**Result:** PASS or FAIL, date, host (Weston version, GLES renderer string from
`$OUT/niri.log`).

## Pinned revisions

- Implementation source commit and binary SHA-256 (from `binaries.sha256`).
- Pre-change source commit `7185ee51` and binary SHA-256.
- `just test` and `just check` results on the implementation commit.

## Procedure

`docs/materials/scripts/glass-noise-saturation-smoke.sh` with the six
captures it defines; the runtime KDL fixture is the one the script writes,
with `blur { noise 0.08; saturation 1.5 }` as the deliberately non-neutral
global block.

## Metrics

Paste `metrics.txt` and the script's final PASS line.

## Retained artifacts

Path of `$OUT` and the contents of `SHA256SUMS`.

## Cleanup

`systemctl --user list-units 'gns.*'` empty; no `gns.*` directory under
`$XDG_RUNTIME_DIR`; the base worktree at `/mnt/ssd3/tmp/material-1293e8-base`
removed with `git worktree remove`.
```

- [ ] **Step 5: Index and clean up**

Add to `docs/materials/README.md` after the spec entry from Task 3:

```markdown
- `2026-09-05-material-glass-noise-saturation-params-evidence.md`: passing nested GLES evidence that written noise and saturation render with backdrop blur off and under `blur { off }`.
- `scripts/glass-noise-saturation-smoke.sh`: headless harness for those captures.
```

Then: `git -C /mnt/ssd/Dropbox/niri-material worktree remove /mnt/ssd3/tmp/material-1293e8-base`, and confirm `systemctl --user list-units 'gns.*'` and `ls -d "$XDG_RUNTIME_DIR"/gns.*` show nothing.

- [ ] **Step 6: Check, close the child, commit**

Run: `just check`
Expected: exit 0.

```bash
tasks done material-4af245 "headless GLES smoke passes: written values render with backdrop-blur off and blur { off }; omitted values byte-identical"
git add docs/materials/scripts/glass-noise-saturation-smoke.sh docs/materials/2026-09-05-material-glass-noise-saturation-params-evidence.md docs/materials/README.md tasks/
git commit -m "test(materials): nested GLES smoke for written glass noise and saturation"
```

---

### Task 5: Close out

**Files:**
- Modify: `docs/specs/2026-09-05-material-glass-noise-saturation-params-design.md:3-8`
- Modify: `tasks/material-4df34c.md`, `tasks/material-1293e8.md` via the CLI only

- [ ] **Step 0: Start the child task**

Run: `tasks start material-4df34c`

- [ ] **Step 1: Full gates**

Run: `just check && just test`
Expected: both exit 0. Record the trailing summary lines in the evidence document's "Pinned revisions" section if Task 4 left them as placeholders.

- [ ] **Step 2: Spec status**

Replace the `**Status:**` paragraph of `docs/specs/2026-09-05-material-glass-noise-saturation-params-design.md` with:

```markdown
**Status:** implemented on `glass-noise-saturation` (native piece
`material-1293e8`), `just check` and `just test` passing, nested GLES evidence
in
[`2026-09-05-material-glass-noise-saturation-params-evidence.md`](../materials/2026-09-05-material-glass-noise-saturation-params-evidence.md).
Hub goal `prism-63dd45`; the Prism piece `prism-d0d4cb` follows once this
build is installed.
```

- [ ] **Step 3: Close the child, then the piece, in one commit**

```bash
tasks done material-4df34c "gates pass; spec status records the landing"
tasks done material-1293e8 "glass { noise; saturation } land as optional parameters; written values always apply; nested GLES evidence passes"
tasks check
git add docs/specs/2026-09-05-material-glass-noise-saturation-params-design.md docs/materials/2026-09-05-material-glass-noise-saturation-params-evidence.md tasks/
git commit -m "chore: land the glass noise and saturation parameters piece"
```

`tasks done material-1293e8` refuses while any child is open; if it does, a child's commit step was skipped. Close that child first rather than forcing.

- [ ] **Step 4: Hand off**

Use superpowers:finishing-a-development-branch to merge `glass-noise-saturation` into `materials-26.04`. Then rebuild and install the package (`packaging/arch`) so the Prism piece can validate its fragment; that install is the precondition recorded on `prism-d0d4cb`.
