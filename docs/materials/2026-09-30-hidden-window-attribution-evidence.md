# Hidden-window GPU attribution: capture evidence

**Status:** captured 2026-10-01 from a TTY with the desktop stopped; pilot and
matrix passed every gate. **Task:** `material-d09741`, under `material-5d6b2c`.
**Brief:** [resource-aware rendering](../notes/2026-09-29-resource-aware-rendering-brief.md).

Question: does a material window that nobody can see still cost compositor
redraws, material draws, shared prefilter rebuilds or client GPU work beyond
the existing visibility gates?

Answer: the existing gates hold the expensive parts. In every hidden case the
material draw count is zero, the prefilter is never rebuilt, and the client
drops from 60 fps to the 1 Hz fallback-timer cadence, so a hidden client's
GPU work falls by a factor of about 60. Two small residues remain. Each hidden
client commit queues a full output redraw (20 in 20 s). For an offscreen
column or a window under an opaque cover, `Tile::render` also re-renders the
probe's offscreen buffer on each of those commits, though nothing draws it.
A window under a translucent cover is visible and pays full cost, as it
should.

## Fixture and host

`docs/materials/scripts/hidden-window-attribution.sh` (commit `6829d4c9`,
`pilot` then `matrix`). Each case is a fresh nested niri under a headless
Weston host (`--renderer=gl`, kiosk shell, 1280x720), traced by Tracy 0.13.1
for 30 s; zones are counted in the final 20 s. The workload is
`weston-simple-egl`, which paces on frame callbacks and reports its own frame
rate, under a pinned glass material (`ior 1.5`, `thickness 20`, `bevel 12`,
`roughness 0.3` so the shared roughness pyramid participates, `focus "none"`,
`ring-beam-speed 0`). A static kitty (`gos-other`) holds focus in every case.
The capture protocol (`tools/capture-meta`) ran preflight, identity, a settle
before each case and release.

| Item | Value |
| --- | --- |
| Host | RTX 3070, driver 615.71.09, kernel 7.2.2, TTY session, desktop stopped, user timers held |
| Binary | `cargo build --release --features profile-with-tracy` at `6829d4c9`, SHA-256 `3788326824002059b3c7602899dc68ba64c6cc9f03a9651d49ead9412f634926` |
| Pilot | `$NIRI_MATERIAL_WORK_ROOT/material-d09741/pilot-1`, 8 min (build 1.7) |
| Matrix | `$NIRI_MATERIAL_WORK_ROOT/material-d09741/matrix-1`, 9 min; `results.tsv` SHA-256 `c6ad19f9f8e0d3d190aa21156affc655f992e4cafe5adb36aa0d9e3f5eb8f159` |
| Settles | all eight `settled`: load1 0.32–0.90, CPU 0.9–1.1 %, GPU P8 at 10.85–10.94 W |

Command, from `.worktrees/material-d09741`:

```bash
CAPTURE_TASK=material-d09741 NIRI_MATERIAL_WORK_ROOT=$NIRI_MATERIAL_WORK_ROOT \
    OUT=$NIRI_MATERIAL_WORK_ROOT/material-d09741/matrix-1 \
    docs/materials/scripts/hidden-window-attribution.sh matrix
```

## Matrix, 2026-10-01

Counts of each Tracy zone in the final 20 s; `client fps` is the median of the
probe's last three 5 s reports. `fallback` is
`Niri::send_frame_callbacks_on_fallback_timer`; `frame cb` is
`Niri::send_frame_callbacks`.

| Case | `Niri::redraw` | `Tile::render` | `OffscreenBuffer::render` | commits | frame cb | fallback | `MaterialRenderElement::draw` (GPU) | `Prefilter::downsample` (GPU) | client fps |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `visible` (positive control) | 1199 | 2400 | 1200 | 1200 | 1200 | 19 | 1200 | 0 | 60.2 |
| `overview` (positive control) | 1199 | 2400 | 1200 | 1200 | 1200 | 20 | 1200 | 0 | 60.2 |
| `covered-alpha` (0.5-opacity cover) | 1199 | 3600 | 1200 | 1200 | 1200 | 20 | 1199 | 0 | 60.2 |
| `inactive-workspace` | 20 | 20 | 0 | 20 | 20 | 20 | 0 | 0 | 1.0 |
| `hidden-tab` | 20 | 20 | 0 | 20 | 20 | 20 | 0 | 0 | 1.0 |
| `offscreen-column` | 20 | 100 | 20 | 20 | 20 | 20 | 0 | 0 | 1.0 |
| `covered-opaque` (opaque floating cover) | 20 | 60 | 20 | 20 | 20 | 20 | 0 | 0 | 1.2 |
| `empty` (no probe) | 0 | 0 | 0 | 0 | 0 | 20 | 0 | 0 | — |

The pilot (`visible`, `inactive-workspace`, `empty`) recorded the same counts
within one event. The gated expectations: positive material draws and more
than 20 fps for both controls; zero material draws and zero offscreen renders
for `inactive-workspace`; zero draws for `empty`. The other cases are
findings.

## Attribution

- **Material draws.** Zero in every hidden case, positive in both controls.
  The inactive workspace and the hidden tab are culled before
  `Tile::render` (`src/layout/monitor.rs`, `src/layout/scrolling.rs`). An
  offscreen column and an opaquely covered tile still reach
  `Tile::render`, but smithay's damage tracking skips the draw.
- **Client work.** A hidden probe falls from 60.2 fps to 1.0–1.2 fps:
  `send_frame_callbacks` uses primary-scanout visibility, so the hidden
  surface is serviced only by the 1 Hz fallback timer. This is the existing
  gate; the client's own GPU work scales with it.
- **Compositor redraws.** Each hidden-client commit queues an output redraw
  (`src/handlers/compositor.rs`), so a hidden 1 Hz client costs 20 redraws in
  20 s. These redraws draw no material. The empty workspace confirms the
  fallback timer itself queues nothing.
- **Offscreen buffer.** In `offscreen-column` and `covered-opaque`,
  `Tile::render_inner` prepares the effect buffers and re-renders the probe's
  window into its `OffscreenBuffer` on every such redraw
  (`src/layout/tile.rs`, the `material.offscreen.render` call), although
  the result is never drawn: `ScrollingSpace::render` walks every column,
  and `MaterialRenderElement` declares no opaque region that would let the
  cover cull the tile. At 1 Hz this is one window-sized offscreen pass a
  second per hidden material tile.
- **Shared prefilter.** `Prefilter::downsample` is zero everywhere,
  including the controls: the backdrop pyramid holds only the static
  Background layer and is cached, so no window, hidden or visible, rebuilds it.
- **Translucent cover.** `covered-alpha` costs as much as `visible` (60 fps,
  1199 draws, and 3600 tile renders for three tiles): the probe shows through
  the 0.5-opacity cover, so this is correct visible work, not a gap.

## Recommendation

The existing gates suffice for material draws, shared prefilter work and
client frame pacing; the "hidden kitty GPU activity" this was opened for is
not compositor material rendering, which is zero while hidden. A hidden
client still drives its own 1 Hz work.

The reproducible residue is the offscreen-buffer render for a tile that
reaches `Tile::render` but cannot be seen (an offscreen column, or a tile
under an opaque window), at the hidden client's 1 Hz commit rate. It is small,
and fixing it is a culling change in the tile render path, so it belongs with
`material-7afc31` rather than a fix here. Two parts are not measured:

- A sustained optic (Aurora, attention) on a tile under an opaque cover. The
  code trace indicates its deadline keeps queuing redraws at the optic's
  cadence, because `Tile::tick_deadline` rejects only out-of-view tiles, not
  covered ones. The probe here had no sustained optic. Sustained-optic
  settling (`material-f86183`) reduces this after input idle, but not while
  the user is active on another window.
- A second lit output: winit nests one output, so this rests on the
  per-output render path, not on a capture.

## Correction to the 2026-09-18 evidence

[2026-09-18-ring-focus-motion-evidence.md](2026-09-18-ring-focus-motion-evidence.md)
stated that its counts were `MaterialRenderElement::draw` zones. The signals
smoke counts `Niri::redraw` zones (exact name) there; that document is now
corrected. Its zeros for hidden workspaces, tabs and offscreen columns are
zero redraws, which also implies zero material draws for those cases.

## Correction, 2026-10-06

The *Offscreen buffer* bullet under Attribution says `MaterialRenderElement`
declares no opaque region that would let the cover cull the tile. That is not
the mechanism. `Tile::render_inner` renders the window into its
`OffscreenBuffer` while it builds the tile's render elements, before smithay's
damage tracker runs its occlusion pass, so no opaque region on any element
can skip that render. An opaque region on the material element would only let
elements behind it be skipped. The recommendation stands: culling the tile
before `render_inner` (`material-7afc31`) is the fix. The measurements are
unaffected.
