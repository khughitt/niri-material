---
id: material-d09741
title: Attribute hidden-window GPU work to clients or material rendering
status: done
priority: 1
size: s
complexity: mid
process: direct
owner: materials-26.04
created: 2026-09-29T21:44:30Z
updated: 2026-10-01T04:39:27Z
started: 2026-09-30T09:13:31Z
completed: 2026-10-01T04:39:26Z
depends: []
parent: material-5d6b2c
tags: [performance]
source: docs/notes/2026-09-29-resource-aware-rendering-brief.md
model: claude-opus-5-5
agent: codex
---

Question: Do hidden or fully occluded material windows still cause compositor material draws, prefilter rebuilds, redraw scheduling or client GPU work beyond the existing visibility gates?
Where to start: docs/notes/2026-09-29-resource-aware-rendering-brief.md; src/layout/monitor.rs::update_render_elements and workspaces_with_render_geo; src/layout/tile.rs::render/tick_deadline; src/niri.rs::fill_xray_elements/send_frame_callbacks/send_frame_callbacks_on_fallback_timer; src/render_helpers/effect_buffer.rs; docs/materials/scripts/material-signals-smoke.sh.
Bound: Trace these paths and compare one fixed workload visible, on an inactive workspace, in a hidden tab, offscreen and behind an opaque covering window, plus an empty-workspace control. Include a second lit output and an overview/transition control so visible work is not misclassified. Start with one end-to-end capture pilot using tools/capture-meta, then the bounded matrix only when the lane passes. Attribute client work separately from compositor redraws, material draws and shared backdrop/prefilter rebuilds. No visibility algorithm, client suspension policy, always-on collector, exhaustive sweep or host launcher change.
Expected result: Record a per-case attribution table and a recommendation: existing gates suffice, or a reproducible gap with the responsible caller and a focused follow-up. Use existing Tracy draw/redraw zones; document any temporary instrumentation and positive draw controls. Host-wide GPU utilization alone is not evidence of compositor rendering. Update this task and the brief.
Ideas it wakes: On completion, run tasks note on material-7afc31 with the finding, in the same commit as this result.

## Notes

- 2026-09-30T09:13:31Z (materials-26.04): started
  provenance: {"harness_session":"claude-code:753832fc-7ed3-4976-8c1c-9feafc79240f","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-09-30T09:19:20Z (materials-26.04): harness survey: reuse glass-optic-smoke-lib.sh + idle-budget check_renderer/await_gpu_rest; client = weston-simple-egl (self-reports fps); nested winit gives one output only, so the second-output control is a layout-level test (add_output) rather than a Tracy case; per-case settles have only passed from a TTY, so the pilot likely parks quiet --needs headless. Note: 2026-09-18 evidence doc says it counted MaterialRenderElement::draw but the script counted Niri::redraw.
- 2026-09-30T09:20:52Z (materials-26.04): code trace (read-only, spot-checked): inactive workspace + hidden tab culled before Tile::render (monitor.rs:1711, scrolling.rs:2974); offscreen columns are NOT view-culled — ScrollingSpace::render walks every column, so Tile::render_inner prepares effect buffers and renders the tile offscreen (tile.rs:1959-1966) though smithay then skips the draw; MaterialRenderElement has no opaque_regions override, so a window under glass (or alpha<1) gets a full material draw + full-rate frame callbacks, and under an opaque window its aurora/attention deadlines still queue 4 Hz NoDamage redraws; every hidden-client commit queues a full output redraw (compositor.rs:318/366/386); prefilter holds only Background layer, never rebuilt by a hidden window (effect_buffer.rs:425). Possible overview under-render: tick deadline uses unzoomed positions. Capture must confirm.
- 2026-09-30T09:40:29Z (materials-26.04): rehearsal (HWA_REHEARSAL=1, busy desktop, no capture record; not evidence), counts in final 20 s, probe weston-simple-egl under roughness-0.3 glass: visible 1199 redraws / 1199 material draws / 60 fps; inactive-workspace 20/0, offscreen 0, client 1.0 fps; hidden-tab 20/0/0, 1.0 fps; offscreen-column 20 redraws, 0 draws but 20 OffscreenBuffer::render (tile body re-rendered per fallback commit), 1.2 fps; covered-opaque (opaque floating kitty) 20/0 draws/20 offscreen, 1.2 fps; covered-alpha (0.5-opacity cover) 1199/1199, client 60 fps — full cost; overview 1196/1197, 60 fps; empty 0/0. Prefilter::downsample 0 everywhere (static wallpaper, cached pyramid). Matches the code trace.
- 2026-09-30T09:41:55Z (materials-26.04): parked (waiting on user, quiet; headless, 20 min): From a TTY with the desktop stopped, in .worktrees/material-d09741: (1) pilot: CAPTURE_TASK=material-d09741 NIRI_MATERIAL_WORK_ROOT=/mnt/ssd3/niri-material OUT=/mnt/ssd3/niri-material/material-d09741/pilot-1 docs/materials/scripts/hidden-window-attribution.sh pilot — phases: preflight 1, tracy build 0-3 (cached from rehearsal), load wait 0-5, 3 cases x ~50 s; must PASS its gates. (2) only then the same with OUT=.../matrix-1 and 'matrix' (8 cases x ~50 s, ~8 min). Rehearsals on the busy desktop passed; the real preflight refused-risk is per-case settles, which have only passed from a TTY. Then agent: run: notes, evidence doc docs/materials/2026-09-30-hidden-window-attribution-evidence.md with the recorded table, update the brief (and correct the 09-18 evidence doc's zone claim), note material-7afc31, close.
  provenance: {"harness_session":"claude-code:753832fc-7ed3-4976-8c1c-9feafc79240f","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-01T03:36:17Z (materials-26.04): resumed
  provenance: {"harness_session":"claude-code:601fb2a6-cb37-47f5-a964-59f3bb9f54af","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-01T03:36:17Z (materials-26.04): run: 8 min (est 10, headless); build 1.7, preflight 0.5, 3 cases 6; passed: pilot from a TTY with the desktop stopped, user timers held; visible 1199 redraws / 1199 material draws / 60.2 fps, inactive-workspace 20 / 0 / 1.0 fps, empty 0 / 0 (pilot-1)
- 2026-10-01T03:45:32Z (materials-26.04): run: 9 min (est 8, headless); build 0, 8 cases 9; passed: matrix-1; hidden-tab 20 redraws / 0 draws / 1.0 fps; offscreen-column 20 / 0 draws / 20 OffscreenBuffer::render / 100 Tile::render / 1.0 fps; covered-opaque 20 / 0 / 20 offscreen / 1.2 fps; covered-alpha 1199 / 1199 / 60.2 fps; overview 1199 / 1200 / 60.2 fps; Prefilter::downsample 0 in every case
- 2026-10-01T04:39:26Z (materials-26.04): done
  provenance: {"harness_session":"claude-code:601fb2a6-cb37-47f5-a964-59f3bb9f54af","harness_session_source":"CLAUDE_CODE_SESSION_ID"}
- 2026-10-01T04:39:27Z (materials-26.04): result: existing gates suffice for material draws, prefilter work and client pacing; residue (undrawn offscreen render for offscreen/opaquely covered tiles, a redraw per hidden commit) handed to material-7afc31 with the unmeasured covered-optic and second-output cases; evidence docs/materials/2026-09-30-hidden-window-attribution-evidence.md; brief updated; 2026-09-18 evidence corrected (it counted Niri::redraw)
