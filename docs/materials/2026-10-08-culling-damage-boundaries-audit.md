# Material tiles: culling and damage boundaries (source audit)

**Status:** source audit, 2026-10-08, at `1d4e39dd`. No renderer change, no
capture. **Task:** `material-82e4bc`, under `material-5d6b2c`.
**Brief:** [resource-aware rendering](../notes/2026-09-29-resource-aware-rendering-brief.md).
**Measured inputs:** [hidden-window attribution](2026-09-30-hidden-window-attribution-evidence.md),
including its 2026-10-06 correction.

Two boundaries, kept apart:

- **A. Pre-render culling.** Before `Tile::render` builds elements: can the
  material preparation (effect-buffer `prepare`, `material.offscreen.render`,
  `material_dynamics`, the fingerprint) and the deadline report be skipped for a
  tile nobody sees?
- **B. Final draw.** After the elements exist: what damage and opaque-region
  facts can `MaterialRenderElement` truthfully expose to smithay's damage
  tracker?

Every claim below names its source. *Unknown* means the source does not settle
it; each row carries the check that would falsify it.

## 1. What runs, in order

`Tile::render` (`src/layout/tile.rs:2305`):

1. Reports `tick_deadline` to the output's `SignalTicks`, only when
   `ctx.signal_ticks` is set (`tile.rs:2316-2322`). That is the `Output` target
   only (`src/niri.rs:4401-4407`); screencast, screen capture and snapshots
   report nothing.
2. Clears the window's `offscreen_data` (`tile.rs:2332`).
3. Under an open or alpha animation, calls `render_inner` at `(0, 0)` into the
   animation's own offscreen (`tile.rs:2334-2385`); otherwise calls
   `render_inner` at `location` (`tile.rs:2391`).

`Tile::render_inner` (`tile.rs:1777`) pushes popups first, directly
(`tile.rs:1836-1843`), then either the resize path (`resize.offscreen.render`,
then `material.offscreen.render` of the resize element, `tile.rs:1913`) or the
normal path (`render_normal`, background and backdrop `prepare`,
`material.offscreen.render`, `tile.rs:2088-2094`). After the offscreen render:
`material_frame`, `material_dynamics` (`tile.rs:2112`), the
`InputFingerprint`, `MaterialState::advance_commit`
(`src/render_helpers/material/mod.rs:684-692`) and `set_offscreen_data`
(`tile.rs:2186`). Border, focus ring, shadow and background-effect elements
follow (`tile.rs:2293`).

Callers of `Tile::render`:

| Path | Gate today | Position passed |
| --- | --- | --- |
| Scrolling, `scrolling.rs:2956-2992` | `visible \|\| alpha_animation` (`:2973-2976`); `visible` is false only for hidden tabs (`:5310-5325`). No view test: offscreen columns render | View-relative, workspace logical (`:2957-2960`) |
| Floating, `floating.rs:1083-1095` | None per tile; the whole space is skipped when floating is hidden (`workspace.rs:1652-1654`) | View-relative |
| Monitor, `monitor.rs:1716-1724` | Workspaces culled against the output (`monitor.rs:1509-1517`); floating renders before scrolling | Elements then rescaled by overview `zoom` and moved by `geo.loc` (`monitor.rs:1695-1706`); horizontal crop unbounded (`monitor.rs:2199-2201`) |
| Interactive move, `layout/mod.rs:4899-4935` | Rendered on `move_.output` only | Output-local before zoom |
| Unmap snapshot, `Tile::render_snapshot` (`tile.rs:2487`) | Always renders: `Output` (twice when xray has blocked-out layers) and `Screencast`, at `(0, 0)`, `signal_ticks: None` | Fake `(0, 0)` |

Render targets: `Output` (`tty.rs:1885-1890`, `winit.rs:228-233`),
`Screencast` (a full output re-render, `screencasting/mod.rs:596-602`) and
`ScreenCapture` (`niri.rs:5512-5521`). All three use the same layout geometry
for the same output. Window casts and window screenshots use `Mapped::render`
(`mapped.rs:559-571`, `niri.rs:5860`), and the window switcher
`render_normal` (`mru.rs:451`): none touch material. A tile is rendered only
by its own monitor (`monitor.rs:1716-1724`), so no tile renders once per
output.

## 2. Boundary A: pre-render culling

### Reveal and bookkeeping after a skip

- **Offscreen content.** `OffscreenBuffer` keeps a persistent texture and an
  inner `OutputDamageTracker` rendered at age 1 (`offscreen.rs:164-169`).
  Surface damage comes from smithay's `DamageBag`, which keeps four commits
  and returns full damage beyond them. The first frame after a skip therefore
  repaints from current surface state, from accumulated or full damage.
- **Material state.** A stale `last_inputs[target]` costs one `bump`
  (`material/mod.rs:684-692`): full element damage, which a reappearing
  element gets anyway. Jelly is computed from residuals each frame
  (`material/mod.rs:227`). The signal frame cache is filled in
  `update_render_elements` (`tile.rs:1144-1147`), not in render.
  `clock.record_optic_render` (`tile.rs:732`) is a maximum across tiles.
- **Frame callbacks and primary scanout.** `update_primary_scanout_output`
  (`niri.rs:4980-5125`) maps a surface presented through `offscreen_data` to
  the material element's id (`niri.rs:5048-5053`). An offscreen-column
  material element never intersects the output, so smithay records no state
  for it and clears the primary output; `send_frame_callbacks` then throttles
  it to the 995 ms fallback (`niri.rs:193`, `:5257-5330`). That matches the
  measured 1.0 fps. Skipping the material work leaves this unchanged: the
  cleared `offscreen_data` (`tile.rs:2332`) reads as not presented, as a
  hidden tab does today.
- **The focus beam.** Once rendered, a beam has no timeout
  (`tile.rs:458-462`); `beam.done` is set only inside `material_dynamics`
  (`tile.rs:773, 791`). If material work is skipped while
  `signal_render_visible` stays true, `are_animations_ongoing` stays true
  (`tile.rs:1065-1069`): a redraw every frame. The cull predicate and
  `signal_render_visible` must be one predicate.

### The visibility predicate is mis-scoped in overview

`tick_deadline` tests `slab_in_view(location, tile_size, bevel, view)`
(`tile.rs:2439-2464`, `render_helpers/signal.rs:319-330`) with `view` the
output rect at the origin (`niri.rs:4404-4406`), while `location` is
view-relative workspace coordinates before the overview's zoom and
`geo.loc`. `signal_render_visible` (`tile.rs:1149-1157`) applies the same band
to the workspace's `view_size` rect (`scrolling.rs:399-410`, `:4143-4145`).
In overview the workspace is drawn at `zoom < 1` with unbounded horizontal
crop, so columns outside the normal view are on screen. By source, a
sustained optic or attention motion on such a column reports no deadline and
holds; the beam stops reporting its animation. Not observed on screen yet:
`material-2e97ff` (filed by this audit) verifies it with an in-process test
and fixes it. During a workspace switch the same mismatch over-reports, which
costs redraws but loses nothing.

**Fixed (`material-2e97ff`, 2026-10-08).** Both predicates now test against
`workspace_screen_view` (`src/layout/monitor.rs`): the output mapped into the
workspace's coordinates through the overview zoom and `geo.loc`, carried to
`Tile::render` on `RenderCtx::signal_ticks` and to
`Tile::update_render_elements` as `screen_view`. The interactive move maps the
output through its own zoom the same way. The view ignores the vertical crop
of overview cards, so it can over-report but never under-report. The two
`overview_column_outside_the_normal_view_*` tests in `src/tests/signal.rs`
failed before the fix.

### Extent a cull must cover

The material element's geometry is `frame.area`: the window texture
footprint merged with the slab (`material/mod.rs:471-501, 817-819`). The slab
is the window rect inflated by `ceil(bevel - max|offset|)` and shifted by the
rounded offset (`material/mod.rs:477-487`), so `slab_in_view`'s tile rect
plus `bevel` bounds it. It does not bound the texture footprint: client
buffer area outside the window geometry (client-side shadows, subsurfaces) is
known only after `encompassing_geo` runs inside the offscreen render
(`offscreen.rs:79`). smithay's `Window::bbox()` gives it before render
(`Mapped` already uses `bbox_with_popups`, `mapped.rs:518`). Also outside the
band: `bob_offset` (added at `tile.rs:1810`), the open animation's expanded
area (`opening_window.rs:75-79`) and the resize animation's
`animated_window_size`. Popups, border, focus ring and shadow are separate
elements and need no cull if only the material work is skipped.

### Coverage

No layout code knows coverage before render: nothing under `src/layout` or
`src/window` reads opaque regions; only smithay does, after elements exist
(`damage/mod.rs:514-533`). Floating tiles render before scrolling tiles
(`monitor.rs:1716` before `:1724`), so cover positions are available in time.
A coverer is opaque over a tile only with: its surface opaque region in output
coordinates minus rounded corners (as `clipped_surface.rs:195-221` computes),
opacity rule 1 (`tile.rs:1792-1800`), no alpha, open or resize animation, no
block-out on the target, and, for the covered side, the full extent above.

New from the shader (section 3): a **material** coverer at `niri_alpha == 1`
is opaque over its slab interior. Its glass samples only the Background-layer
effect buffers, never tiles behind it (`render-pipeline.md` §1).

### Verdicts, boundary A

| Claim | Verdict | Conditions | Falsifying check |
| --- | --- | --- | --- |
| Skip material preparation (effect-buffer `prepare`, `material.offscreen.render`, dynamics, fingerprint, element) for a tile whose material extent is outside the view | **Conditional, safe once its conditions hold** | Overview-correct view (`material-2e97ff`); extent = slab band ∪ `Window::bbox()` ∪ `bob_offset` ∪ open/resize animation areas; never inside `render_snapshot` or the open/alpha animation's `(0, 0)` call; the same predicate as `signal_render_visible`; popups, border, ring and shadow still pushed | In-process: an off-view column with a client commit draws no `material.offscreen.render`; scrolling it into view shows the commit's content on the first frame (pixel compare with a never-culled run); a beam on an off-view tile ends `are_animations_ongoing`; overview shows the column's glass |
| Same skip for a tile under an opaque cover | **Unknown** | All of the above, plus a coverage computation that exists nowhere today | — (needs a design first) |
| Suppress `tick_deadline` for a covered tile | **Conditional** | The coverage proof above; the per-output minimum (`render_helpers/mod.rs:73-75`) tolerates an omitted tile | The capture in section 5 measures what it would save |
| Skipping changes frame callbacks for an offscreen column | **Safe (no change)** | — | Hidden-window matrix rerun: `offscreen-column` client fps stays 1.0 |
| Culling must consider a second output | **Safe (not needed) by source** | A tile renders only on its own monitor; interactive move only on `move_.output` | Two-output lane (`material-1af3c6` host) shows a tile drawn on a monitor that does not own it |

**Landed (`material-7afc31`, 2026-10-08): off-view culling.** On an
`Output` render, `Tile::render` skips the window body (both effect-buffer
`prepare` calls, `material.offscreen.render`, the dynamics, the fingerprint and
the element) when `Tile::material_out_of_view` holds against the screen view
on `RenderCtx::signal_ticks`. The extent is the slab band at `location` and
at the bob offset, plus `LayoutElement::buf_bbox` (smithay `Window::bbox()`,
popups excluded, relative to the window geometry), tested against the view
grown by two physical pixels for the slab's pixel rounding and the unrounded
position `signal_render_visible` uses; a culled body therefore never has its
band in view, which keeps the beam condition. Open, alpha and
resize animations are excluded rather than bounded, and snapshots and other
targets carry no sink, so they never cull. Popups, border, focus ring, shadow
and the background effect are still pushed. In-process checks in
`src/tests/signal.rs`: the off-view column draws no material element on the
output, still draws on a screen capture and in the overview, and after six
culled commits shows the last one on the first revealed frame.
`material-a9a574` adds: a floating window at the screen-edge clamp still
draws; a floating window dragged off the output is culled and redrawn when
dragged back (a tiled drag goes through the alpha offscreen and never
culls); mid workspace switch both workspaces draw while a column outside the
incoming view stays culled; and a client shadow reaching into the view keeps
an otherwise off-view column drawn. The
hidden-window matrix rerun (offscreen-column fps stays 1.0) was not run; the
source claim above is unchanged.

## 3. Boundary B: final-draw damage and opacity

### What the element depends on

- `niri_tex_win` is read once, at the fragment's own UV
  (`shaders/material/main.frag:11`); an opaque window texel returns
  `win * niri_alpha` before any glass work (`main.frag:13-16`).
- The glass term reads the background and backdrop through displaced taps
  (`prelude.frag:411-428`) and reflection past the silhouette
  (`reflection.frag:19`), plus `gl_FragCoord` hashes and uniforms. It never
  reads the window texture.
- The composite is `win + (1 - win.a) * glassed`, times `niri_alpha`
  (`main.frag:235-236`), with `glassed = vec4(glassColor, 1) * coverage`
  (`main.frag:228`) and `coverage = 1 - smoothstep(-1/scale, 0, d)`
  (`prelude.frag:372-373`).

So each output pixel is a function of the window texel at the same place, of
glass inputs independent of the window, and of alpha. And wherever slab
coverage is 1 and `niri_alpha` is 1, the output is opaque, whatever the
client draws.

### Damage today

`MaterialRenderElement` keeps smithay's default `damage_since`: full geometry
whenever the commit differs. `InputFingerprint` (`material/mod.rs:599-624`,
filled at `tile.rs:1974-1987, 2153-2166`) bumps the shared commit on: the
offscreen's commit (`window`), background and backdrop id and commit,
`mapping`, `backdrop_color`, `corner_radius`, `jelly`, `signal` (including
focus and the beam), `glass_signal` and every optic's uniforms. Geometry,
alpha and scale are left to smithay, which damages old and new rects on any
change (`smithay damage/mod.rs:147-152, 554-575`). `frame.geo_rect`,
`slab_rect`, `chamfer` and `win_rect` are neither fingerprinted nor seen by
smithay.

`OffscreenRenderElement` already maps client surface damage into texture
damage (inner tracker, `offscreen.rs:164-181`) and exposes it through
`damage_since` (`offscreen.rs:267-287`). On texture reallocation (growth,
non-unique texture, renderer change) and on scale change it builds a **new**
`DamageBag` under the same buffer `Id` (`offscreen.rs:106-147, 156-162`), so
the commit counter restarts. The material fingerprint compares only that
counter, and a consumer holding an older commit can get partial history.
Growth and scale change also change the element geometry, which smithay
damages fully. Whether the non-unique or renderer-change cases ever collide
with an equal counter is unknown.

### Verdicts, boundary B

| Claim | Verdict | Conditions | Falsifying check |
| --- | --- | --- | --- |
| Forward client-only damage: window damage D maps to D translated by the footprint offset | **Conditional** | A `DamageBag` owned by `MaterialState`: push the offscreen's `damage_since(last window commit)`, translated by `tex_geo.loc - area.loc` and dilated 1 px for LINEAR sampling (`shader_element.rs:392-393`), only when `window` alone changed; full area on any other fingerprint change, on any change of `geo_rect`, `slab_rect`, `chamfer`, `win_rect` or area size, and on an offscreen generation change (needs a generation id; the counter alone is not enough) | Unit: an offscreen reallocation at an equal counter yields full damage. Headless: a 1 px client update on a translucent glass window, partial against forced-full redraw, pixel-identical, including a footprint shift at constant area. Grep: no `niri_tex_win` read other than `main.frag:11` |
| Declare opaque regions: slab interior ∪ client-opaque window area | **Conditional** | `niri_alpha == 1` exactly (`tile.rs:1792-1800`); slab rect eroded ≥ 1 physical px and minus corner boxes of `radius + chamfer + aa` (`prelude.frag:359-360`); glass known to draw (`draw` falls back to the window texture alone when `render_prefiltered` fails, `material/mod.rs:847-875`); client region carried through the offscreen, which drops it today (`offscreen.rs:289-291`); `precise_down` rounding (`shader_element.rs:272-277`) | Render with the region declared over stale content behind; any difference is a false claim. Debug opaque-region toggle (`niri.rs:4441-4443`) at fractional scale and in overview (`Rescale` rounds opaque rects with `to_i32_round`, `smithay element/utils/elements.rs:75-79`). Force the prefiltered fallback: the slab claim must be withdrawn |
| Opaque regions matter for the resize, open or alpha paths | **No** | Resize declares none (`resize.rs:93`) and changes area each frame; open and alpha nest the element inside another offscreen (`tile.rs:2333-2385`) | — |
| A window covered by a glass window keeps full-rate frame callbacks and full material draws today | **Safe as a source claim; unmeasured** | Neither the material nor the offscreen element declares opacity, so smithay never marks the covered element `Skipped` (`damage/mod.rs:514-533`), and primary scanout stays on the output | The optional glass-cover case in section 5 |

## 4. Recommendation

1. **First, `material-2e97ff`:** make `tick_deadline` and
   `signal_render_visible` share one overview-correct view. It is a
   correctness fix whatever happens to culling, and any cull needs the same
   predicate.
2. **Then, offscreen-column culling of the material preparation
   (`material-7afc31`), direct.** Section 2's conditions and checks are its
   specification; no reviewed design is needed. The saving is the measured
   residue: one window-sized offscreen pass per hidden-client commit (about
   1 Hz). It is small, so it ranks below correctness work.
3. **Coverage culling, opaque regions and damage forwarding
   (`material-7f6d0e`) need a reviewed design.** They need a coordinate and
   rounding contract, the fallback rule, an offscreen generation id and a
   coverage computation that does not exist. Measure first: the section 5
   capture establishes whether a covered sustained optic costs anything, and
   its optional glass-cover case whether a window under glass pays full rate.
   Without those numbers there is no case for the design.
4. **Retain current behaviour** for snapshots, the resize, open and alpha
   paths, and every non-`Output` target.

Quantitative cost of any of these belongs to `material-31074f`. Unmeasured and
still unknown: second-output scheduling, and the cost of each claim above in
time or watts.

## 5. Covered sustained-optic capture

Prepared by `material-d21ff0` (fixture extension, no idle host). Run by
`material-46b23d` (`quiet`, headless lane), after the preparation lands.

Extend `docs/materials/scripts/hidden-window-attribution.sh` with an
`optic` mode. The probe is a static kitty (`sleep`, cursor hidden) under the
`hwa-probe` material plus `aurora 0.5 { drift-hz 4; }`, as in
`optic-settling-smoke.sh:196`. It commits nothing, so redraws come from the
optic deadline alone. The input gate is `signal { idle-after-ms 0; }`, the
`gate-off` setting that keeps 4 Hz without input (optic-settling evidence,
*Gate policy*), except in the idle control. Count the existing zones in the
final 20 s.

| Case | Setup | Expected (source) | Gate |
| --- | --- | --- | --- |
| `optic-visible` | Probe in view, `gos-other` focused | about 80 `Niri::redraw`, about 80 material draws | Positive control: draws > 40 |
| `optic-covered-alpha` | 0.5-opacity floating cover | as visible | Translucent control: draws > 40 |
| `optic-covered-idle` | Opaque cover, `idle-after-ms 5000`, no input | 0 redraws in the final 20 s | Idle control: redraws = 0 |
| `optic-covered-opaque` | Opaque floating cover | about 80 redraws, 80 `Tile::render`, 0 material draws | Finding |
| `glass-cover` (optional) | `weston-simple-egl` probe under an opaque cover that carries a glass material at opacity 1 | 60 fps client, about 1200 probe draws | Finding for `material-7f6d0e` |

The pilot is `optic-visible` and `optic-covered-idle`, through the verdict
step, before the matrix. Estimate: build 2 min, about 1 min per case
(30 s trace plus settle), so the pilot takes about 5 min and the matrix about
8 min. `HWA_REHEARSAL=1` exercises setups and analysis on a busy desktop
during preparation.
