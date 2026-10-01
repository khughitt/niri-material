# Filament shift cap: one-core guard and documentation — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** add a rendered regression test that fails if the ring's half-gap cap stops
keeping a second band core off the chamfer, and document the cap as the intended model.

**Architecture:** `src/tests/ring_pair.rs` already renders matched ring-on/off pairs
at a frozen mid-resize instant. Its fixture setup moves into a helper, and its edge
sampling is split out of `profile`. A pure function `one_core` judges one edge
profile and has its own unit tests. A new rendered test runs five glass rows at
`light-ior` 1, 6 and 12 through `one_core`. Documentation follows in a second task.
No shader change.

**Tech Stack:** Rust test code in the `niri` crate (nextest via `just test-one`),
headless surfaceless GLES, Markdown docs, the `tasks` CLI.

**Spec:** [2026-10-01-filament-shift-cap-design.md](../specs/2026-10-01-filament-shift-cap-design.md)
(two review rounds, approved for planning at `87400ac0`).

## Global Constraints

- Work in `.worktrees/material-a85a18` on branch `material-a85a18`.
- No change to `src/render_helpers/shaders/`. The cap-removal edit in Task 1 is a
  one-time mutation check and is reverted before the commit.
- The test config pins `offset-x 6` and `offset-y 6`. With those offsets the face
  edge sits 12 px inside the window edge on the left and top, and flush with it on
  the right and bottom (`face_inset` 12 and 0).
- The fixture keeps flex 0, distortion 0, ripple 0 and chromatic aberration 0.
- The anchor rule: the core `C` is the largest sample whose inward distance exceeds
  `face_inset`. `C` must be positive and within 2 px of `face_inset + ring-gap`.
- The scan rule: walk toward smaller inward distance to the first local minimum, the
  first sample whose next outward neighbour is strictly greater. Every sample beyond
  it must be below `C / 2`.
- The test prints the `light-ior` 1-vs-6 and 6-vs-12 ring-on differences and does not
  assert them.
- Glass rows (name, ior, thickness, bevel, gap): `stock` 1.5/20/12/8,
  `binding` 1.28/31.2/10/2, `ior102` 1.02/80/12/5, `ior124` 1.24/43.3/9/8,
  `ior150` 1.5/80/12/5.
- Gates: `just test-one -p niri ring_pair` while editing, `just test-fast` before each
  commit. The pre-commit hook runs `just check`. If it reports
  `upstream-report ... is stale`, run `just upstream-report` and stage
  `docs/materials/upstream-divergence.md`.
- If a build fails with errors that contradict the source, suspect stale artifacts in
  the shared target dir: `cargo clean -p niri-config -p niri`.

## Review Focus

1. **Left and top edges.** The pilot only measured the right edge. On the left and
   top the core sits at `12 + gap` and the chamfer lies under the transparent client,
   inside the window. Task 1 has a unit case with `face_inset` 12, and the rendered
   test checks all four edges.
2. **Plateaus next to the core.** The capped `binding` profile has an equal pair
   (59, 59) just outside the core. The walk must pass through equal samples, not stop
   on them. Task 1 has a unit case for this.
3. **Ghost brighter than the core.** The anchor must ignore off-face samples. The
   reviewer's case `[82, 60, 20, 83, 50, 0]` is a Task 1 unit case.
4. **Unlit or misplaced core.** An all-zero profile, or one whose brightest face
   sample is far from `face_inset + gap`, must fail rather than pass vacuously.
   Task 1 has both unit cases.
5. **The guard must fail without the cap.** Task 1's mutation step must show
   failures on `binding` and `ior150`. If it does not, the test does not guard the cap.

---

### Task 1: One-core regression test for the ring cap

**Files:**
- Modify: `src/tests/ring_pair.rs`

**Interfaces:**
- Consumes: the existing `Glass`, `Variant`, `BASE`, `config`, `reload`, `render_at`,
  `window_rect`, `dump`, `diff`, `light_map`, `profile` and `matched_pairs` in the
  same file.
- Produces:
  - `const OFFSET: f64`
  - `const STOCK`, `BINDING`, `IOR102`, `IOR124`, `IOR150: Glass`
  - `fn mid_resize_fixture(glass: &Glass) -> (Fixture, ClientId, WlSurface)`
  - `fn edge_samples(map: &[u8], rect: Rectangle<f64, Logical>) -> Vec<(&'static str, Vec<(u8, f64)>)>`
  - `fn one_core(samples: &[(u8, f64)], face_inset: f64, gap: f64) -> Result<(), String>`
  - `#[test] fn ring_cap_keeps_one_core()`
  - `material-22d78f` will reuse `mid_resize_fixture`.

- [ ] **Step 1: Write the failing unit tests for `one_core`**

Append to `src/tests/ring_pair.rs`:

```rust
#[test]
fn one_core_passes_a_capped_halo_step() {
    // dense150 (1.5/80/12, gap 5), capped, right edge: the core at +5.5,
    // then a halo step of 23 on the chamfer, under half the core.
    let s = [
        (40, 7.5),
        (62, 6.5),
        (82, 5.5),
        (81, 4.5),
        (58, 3.5),
        (32, 2.5),
        (18, 1.5),
        (12, 0.5),
        (23, -0.5),
        (14, -1.5),
        (10, -2.5),
        (0, -3.5),
    ];
    assert_eq!(one_core(&s, 0., 5.), Ok(()));
}

#[test]
fn one_core_walks_through_a_plateau() {
    // binding (gap 2), capped, right edge: 59, 59 just outside the core.
    let s = [
        (63, 3.5),
        (83, 2.5),
        (82, 1.5),
        (59, 0.5),
        (59, -0.5),
        (33, -1.5),
        (18, -2.5),
        (0, -3.5),
    ];
    assert_eq!(one_core(&s, 0., 2.), Ok(()));
}

#[test]
fn one_core_rejects_an_uncapped_ghost() {
    // dense150 with the cap removed: 73 on the chamfer after the minimum.
    let s = [
        (62, 6.5),
        (82, 5.5),
        (81, 4.5),
        (58, 3.5),
        (32, 2.5),
        (18, 1.5),
        (12, 0.5),
        (73, -0.5),
        (47, -1.5),
        (0, -2.5),
    ];
    assert!(one_core(&s, 0., 5.).is_err());
}

#[test]
fn one_core_ignores_a_brighter_ghost_off_the_face() {
    // Outward from the core: 82, 60, 20, 83, 50, 0. The 83 lies outside the
    // face, so it cannot become the anchor, and it fails the scan.
    let s = [
        (10, 4.5),
        (40, 3.5),
        (82, 2.5),
        (60, 1.5),
        (20, 0.5),
        (83, -0.5),
        (50, -1.5),
        (0, -2.5),
    ];
    let err = one_core(&s, 0., 2.).unwrap_err();
    assert!(err.contains("second maximum 83"), "{err}");
}

#[test]
fn one_core_anchors_inside_a_left_inset() {
    // Left edge, face 12 px in, gap 2: pixel order runs outward to inward.
    // The chamfer under the window peaks at +8.5 (below half), the core at
    // +14.5.
    let s = [
        (0, 4.5),
        (5, 6.5),
        (30, 8.5),
        (12, 10.5),
        (20, 12.5),
        (83, 14.5),
        (40, 16.5),
    ];
    assert_eq!(one_core(&s, 12., 2.), Ok(()));
    // The same profile with the core's 83 on the chamfer instead: the face
    // core is then 40 at +16.5, which also misses 14 by more than 2 px.
    let moved = [(0, 4.5), (83, 8.5), (12, 10.5), (40, 16.5)];
    assert!(one_core(&moved, 12., 2.).is_err());
}

#[test]
fn one_core_rejects_an_unlit_or_misplaced_core() {
    let dark = [(0, 2.5), (0, 1.5), (0, 0.5), (0, -0.5)];
    assert!(one_core(&dark, 0., 2.).unwrap_err().contains("unlit"));
    let far = [(5, 16.5), (80, 15.5), (5, 14.5), (0, 0.5)];
    assert!(one_core(&far, 0., 5.).unwrap_err().contains("expected near"));
}
```

- [ ] **Step 2: Run them to verify they fail**

Run: `just test-one -p niri one_core`
Expected: compile error, ``cannot find function `one_core` in this scope``.

- [ ] **Step 3: Implement `one_core`**

Add above the `#[test]` functions in `src/tests/ring_pair.rs`:

```rust
/// The cap's guard (spec 2026-10-01-filament-shift-cap-design): one band core
/// per edge. `samples` are (value, inward distance) across one window edge,
/// in any order; the face edge sits `face_inset` px inward. The core is the
/// brightest face sample, positive and within 2 px of `face_inset + gap`.
/// Outward from it, past the first local minimum, every sample stays below
/// half the core.
fn one_core(samples: &[(u8, f64)], face_inset: f64, gap: f64) -> Result<(), String> {
    // Outermost first.
    let mut s = samples.to_vec();
    s.sort_by(|a, b| a.1.total_cmp(&b.1));
    // Of equal maxima, the outermost.
    let (ci, &(core, at)) = s
        .iter()
        .enumerate()
        .filter(|&(_, &(_, d))| d > face_inset)
        .max_by(|a, b| a.1 .0.cmp(&b.1 .0).then(b.0.cmp(&a.0)))
        .ok_or("no samples on the face")?;
    if core == 0 {
        return Err("no core: the face is unlit".to_owned());
    }
    let want = face_inset + gap;
    if (at - want).abs() > 2. {
        return Err(format!("core {core} at {at:+.1}, expected near {want:+.1}"));
    }
    // Outward to the first local minimum: stop where the next outward sample
    // is brighter.
    let mut min = ci;
    while min > 0 && s[min - 1].0 <= s[min].0 {
        min -= 1;
    }
    match s[..min]
        .iter()
        .rev()
        .find(|&&(v, _)| 2 * u16::from(v) >= u16::from(core))
    {
        Some(&(v, d)) => Err(format!(
            "second maximum {v} at {d:+.1} beyond the minimum at {:+.1}; core {core} at {at:+.1}",
            s[min].1
        )),
        None => Ok(()),
    }
}
```

- [ ] **Step 4: Run the unit tests to verify they pass**

Run: `just test-one -p niri one_core`
Expected: 6 passed.

- [ ] **Step 5: Pin the offsets and lift the glass rows to constants**

In `config()`, add two lines to the `glass {{ … }}` block, after `bevel {bevel}`:

```
                offset-x {OFFSET}
                offset-y {OFFSET}
```

Below `const REST`, add:

```rust
/// The glass offsets the config pins (the defaults). The face is the window
/// narrowed by 2 * OFFSET and shifted by OFFSET: its edge sits 2 * OFFSET
/// inside the window on the left and top, flush with it on the right and
/// bottom.
const OFFSET: f64 = 6.;

// Stock geometry: the shared shift stays under the cap (2.28 px vs 4).
const STOCK: Glass = Glass {
    name: "stock",
    ior: 1.5,
    thickness: 20.,
    bevel: 12.,
    gap: 8.,
};
// Live-Prism-like geometry: the shift reaches the cap (1 px) at every
// light-ior.
const BINDING: Glass = Glass {
    name: "binding",
    ior: 1.28,
    thickness: 31.2,
    bevel: 10.,
    gap: 2.,
};
// The cap test's density rows (spec 2026-10-01-filament-shift-cap-design):
// the shared shift reaches the cap from light-ior about 10.1, 5.6 and 0.41.
const IOR102: Glass = Glass {
    name: "ior102",
    ior: 1.02,
    thickness: 80.,
    bevel: 12.,
    gap: 5.,
};
const IOR124: Glass = Glass {
    name: "ior124",
    ior: 1.24,
    thickness: 43.3,
    bevel: 9.,
    gap: 8.,
};
const IOR150: Glass = Glass {
    name: "ior150",
    ior: 1.5,
    thickness: 80.,
    bevel: 12.,
    gap: 5.,
};
```

In `ring_pair_is_reproducible_at_a_frozen_instant`, delete the `stock` and `binding`
locals and their comments, and change the loop to `for glass in [&STOCK, &BINDING] {`.

- [ ] **Step 6: Extract `mid_resize_fixture` and `edge_samples`**

Add the imports:

```rust
use wayland_client::protocol::wl_surface::WlSurface;

use super::client::ClientId;
```

Move the opening of `matched_pairs`, from `let mut f = Fixture::with_config(...)`
through the `f.roundtrip(id);` that follows `window.set_size(W + 200, H)`, into:

```rust
/// A focused shm window whose column is half way through a 200 px wider
/// linear resize, ready to render at `MID`. Every variant renders from this
/// one state.
fn mid_resize_fixture(glass: &Glass) -> (Fixture, ClientId, WlSurface) {
    // … the moved lines, unchanged …
    (f, id, surface)
}
```

and open `matched_pairs` with `let (mut f, id, surface) = mid_resize_fixture(glass);`.

Split `profile`'s sampling out:

```rust
/// Per window edge, the light map on the line through the window centre:
/// (value, inward distance from the edge) at pixel centres within 40 px of
/// it, negative outside the window, in pixel order.
fn edge_samples(
    map: &[u8],
    rect: Rectangle<f64, Logical>,
) -> Vec<(&'static str, Vec<(u8, f64)>)> {
    let cx = (rect.loc.x + rect.size.w / 2.).floor() as i32;
    let cy = (rect.loc.y + rect.size.h / 2.).floor() as i32;
    let (l, t) = (rect.loc.x, rect.loc.y);
    let (r, b) = (l + rect.size.w, t + rect.size.h);
    // (name, horizontal, edge, +1 when inward is increasing pixel index).
    let sides = [
        ("left", true, l, 1.),
        ("right", true, r, -1.),
        ("top", false, t, 1.),
        ("bottom", false, b, -1.),
    ];
    sides
        .into_iter()
        .map(|(name, horizontal, edge, inward)| {
            let limit = i32::from(if horizontal { OUT_W } else { OUT_H });
            let first = edge.floor() as i32;
            let samples = (first - 40..=first + 40)
                .filter(|&i| (0..limit).contains(&i))
                .map(|i| {
                    let (x, y) = if horizontal { (i, cy) } else { (cx, i) };
                    let value = map[y as usize * usize::from(OUT_W) + x as usize];
                    (value, inward * (f64::from(i) + 0.5 - edge))
                })
                .collect();
            (name, samples)
        })
        .collect()
}
```

`profile` keeps its doc comment and becomes:

```rust
fn profile(map: &[u8], rect: Rectangle<f64, Logical>) -> String {
    let mut out = String::new();
    for (name, samples) in edge_samples(map, rect) {
        // … the existing body from `let (peak_idx, &(peak, peak_in)) = samples`
        //   through the `write!`, unchanged …
    }
    out.trim_end_matches("; ").to_owned()
}
```

Run: `just test-one -p niri ring_pair --no-capture`
Expected: all tests pass. The `stock` and `binding` reports match the brief's table:
binding `mid` right `c +1.59`, stock `rest` left `c +20.03`. Pinning the default
offsets changes no pixel.

- [ ] **Step 7: Add the rendered test**

```rust
/// The half-gap cap keeps one band core per edge on dense glass (spec
/// 2026-10-01-filament-shift-cap-design). Also prints, per row, what
/// light-ior still changes with the ring on; where the cap binds that is
/// nothing.
#[test]
fn ring_cap_keeps_one_core() {
    let mut failures = Vec::new();
    for glass in [&STOCK, &BINDING, &IOR102, &IOR124, &IOR150] {
        let (mut f, _, _) = mid_resize_fixture(glass);
        let mut ring_on = Vec::new();
        for light_ior in [1., 6., 12.] {
            let v = Variant { light_ior, ..BASE };
            reload(&mut f, glass, v);
            let on = render_at(&mut f, MID);
            reload(&mut f, glass, Variant { ring: false, ..v });
            let off = render_at(&mut f, MID);
            let tag = format!("cap-light-ior-{light_ior}");
            dump(&format!("{}-{tag}-on", glass.name), &on);
            dump(&format!("{}-{tag}-off", glass.name), &off);
            let map = light_map(&on, &off);
            for (side, samples) in edge_samples(&map, window_rect(&mut f)) {
                let face_inset = if matches!(side, "left" | "top") {
                    2. * OFFSET
                } else {
                    0.
                };
                if let Err(e) = one_core(&samples, face_inset, glass.gap) {
                    failures.push(format!("{} light-ior {light_ior} {side}: {e}", glass.name));
                }
            }
            ring_on.push(on);
        }
        let (p1, m1) = diff(&ring_on[0], &ring_on[1]);
        let (p2, m2) = diff(&ring_on[1], &ring_on[2]);
        eprintln!(
            "{}: ring on, light-ior 1 vs 6: {p1} px, max {m1}; 6 vs 12: {p2} px, max {m2}",
            glass.name
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
```

- [ ] **Step 8: Run it and read the report**

Run: `just test-one -p niri ring_cap_keeps_one_core --no-capture`
Expected: PASS. In the report:
- `binding` and `ior150` show `0 px` for both comparisons;
- `ior124` shows `0 px` for 6 vs 12 and a nonzero 1 vs 6;
- `stock` and `ior102` are nonzero for both.

If the anchor fails on a left or top edge, stop. Print that edge's samples and
compare the core's position with `12 + gap` before changing anything: the spec's
geometry claim is what failed, and the fix belongs in the spec.

- [ ] **Step 9: Mutation check: the test fails without the cap**

```bash
sed -i 's/float cap = 0.5 \* gap;/float cap = 1e6;/' src/render_helpers/shaders/material/main.frag
grep -n 'float cap' src/render_helpers/shaders/material/main.frag   # expect: float cap = 1e6;
just test-one -p niri ring_cap_keeps_one_core --no-capture 2>&1 | grep -E 'light-ior .* (left|right|top|bottom):' | sort | uniq
git checkout src/render_helpers/shaders/material/main.frag
git diff --stat src/render_helpers/shaders/   # expect: empty
```

Expected: FAIL, with failure lines for `binding` and `ior150` at `light-ior` 1 and 6,
at least on the right and bottom edges. Record the result:

```bash
tasks note material-1eab8b "mutation: cap removed -> ring_cap_keeps_one_core fails on <rows/light-ior/edges as printed>; shader reverted"
```

If neither `binding` nor `ior150` fails, the guard does not do its job: stop and
report, do not commit.

- [ ] **Step 10: Gates and commit**

```bash
just test-fast
tasks done material-1eab8b "ring_cap_keeps_one_core guards the half-gap cap on five glass rows; one_core unit-tested; mutation check failed as expected"
tasks check
git add src/tests/ring_pair.rs tasks/
git commit -m "test(material): one band core per edge under the ring cap (material-a85a18)"
```

---

### Task 2: Document the cap and close the decision

**Files:**
- Modify: `docs/materials/material-config.md:80-89`
- Modify: `docs/notes/2026-09-29-glass-measurement-brief.md:61-63` and after the
  `**Cap binding, measured.**` paragraph (line 144)

**Interfaces:**
- Consumes: Task 1's test name `ring_cap_keeps_one_core` and the row names.
- Produces: nothing code depends on.

- [ ] **Step 1: Rewrite the cap paragraph in `material-config.md`**

Replace the paragraph from `Only the ring's shared refracted shift is capped` through
`every value saturate or reduce the knob to chromatic split alone.` with:

```markdown
The ring's shared refracted shift is capped at half `ring-gap`, 4 px at the
default gap of 8. The cap keeps a second copy of the band core off the
chamfer. Without it, dense glass shows one there, with a seam at the face
edge. The cap bounds the shared shift only: the per-channel aberration
offsets ride on top of it uncapped, and aurora uses the same index with no
cap at all.

For an undistorted chamfer normal, the shared shift reaches the cap when
`sin(a - asin(sin(a) / n)) * 0.2 * thickness >= 0.5 * ring-gap`. Here `n`
is the light-path index and `a = atan(min(bevel, thickness) / bevel)` is
the chamfer tilt, 45° whenever `thickness >= bevel`. Beyond that point a
higher `light-ior` no longer moves the ring's landing point on the chamfer.
It still changes the aberration split, pixels whose distorted or rippled
normal yields less than the cap, and aurora.

| glass (`ior`/`thickness`/`bevel`, `ring-gap`) | reaches the cap from `light-ior` |
| --- | --- |
| 1.5/20/12, 8 (stock) | never within 1–12 |
| 1.24/43.3/9, 8 | about 5.6 |
| 1.02/80/12, 5 | about 10.1 |
| 1.28/31.2/10, 2 | every value |
| 1.5/80/12, 5 | every value |

At `ring-gap 2` the cap is 1 px, and on glass as dense as the last two rows
the chamfer's shared shift reaches it at any `light-ior`. There `light-ior`
acts only through the sub-pixel aberration split and, while jelly is
active, through rippled pixels. `ring_cap_keeps_one_core` in
`src/tests/ring_pair.rs` renders these rows and fails if a second core
appears.
```

- [ ] **Step 2: Record the decision in the brief**

In `docs/notes/2026-09-29-glass-measurement-brief.md`, after the paragraph that
begins `**Cap binding, measured.**`, insert:

```markdown
**Cap decision.** `material-a85a18`, 2026-10-01: the hard half-gap cap is
the intended model. Without it, dense glass shows a second copy of the band
core on the chamfer. Where the shared shift reaches the cap, `light-ior` no
longer moves it. The [design](../specs/2026-10-01-filament-shift-cap-design.md)
records the alternatives and why they were rejected. `ring_cap_keeps_one_core`
in `src/tests/ring_pair.rs` guards the cap at ior 1.02, 1.24 (bevel 9), 1.28
and 1.5.
```

Replace *Alternatives* item 3:

```markdown
3. Add production clock controls or redesign the ring cap now. Defer both:
   the existing test seam may suffice, and no current cap-binding visual
   defect has been established by this pass. Since closed: the test seam
   sufficed (`material-0e80c1`), and the cap stays as it is
   (`material-a85a18`, see *Cap decision* under the findings).
```

- [ ] **Step 3: File the Prism idea**

```bash
tasks add "Say what light-ior does where the niri ring cap binds" --project prism --status idea \
  --agent claude-code/claude-opus-5-5 \
  -b "niri-material material-a85a18 kept the ring's half-ring-gap cap on the shared light shift. On the live terminal-glass (ior 1.28, thickness 31.2, bevel 10, ring-gap 2) the chamfer's shared shift reaches the 1 px cap at every light-ior, so light-ior 6 no longer moves it. It still changes the uncapped chromatic-aberration offsets (blue about 0.27 px at light-ior 1, 0.10 at 6), ripple-tilted pixels while jelly is active, and aurora. The palette's light-ior definition could say so. Reference: niri-material docs/materials/material-config.md, the light-ior paragraphs."
tasks note material-a85a18 "prism idea filed: <returned id>"
```

- [ ] **Step 4: Gates, close, and commit**

```bash
just test-fast
tasks done material-7b0318 "material-config documents the cap and its binding table; brief records the decision; prism idea <id> filed"
tasks done material-a85a18 "Half-gap cap kept as the intended model; ring_cap_keeps_one_core guards it on five glass rows; docs and brief updated; prism idea <id>"
tasks check
git add docs/materials/material-config.md docs/notes/2026-09-29-glass-measurement-brief.md tasks/
git commit -m "docs(material): the ring cap is the intended model (material-a85a18)"
```

The pre-commit hook runs the full `just check` for material docs, including the
parameter-table freshness test. The new table sits outside the `params:` markers, so
that test does not read it.
