# Merge review: 418b0130 (materials-26.04 into material-abc08c)

- Verdict: **accept**
- Reviewed revision: 418b0130eddf8e421a34bfb001c63533bb16d224 (first parent 45281a91, second parent ecf2e4f6 = materials-26.04; merge base ba604426fef89ab5cb84a25a872efa3d2129350f)
- Reviewer: claude-code/claude-opus-5-5
- Scope: only what the merge could have broken; earlier task and final branch reviews were not repeated. No tests were run (the full `just test` run is in progress separately); `python3 tools/upstream-report --check` was run and exited 0.

## Findings

No Critical or Important findings.

### Minor 1: rest-cost figure predates main's grain fold (optional, informational)

`docs/materials/performance.md:209` gives the swing's cost "against 0.66 ms at rest". That rest figure was measured on the branch before main's `63567202 perf(material): fold scaled grain rows into one accumulator (material-03fdf3)` came in. That commit lowers the material program's register count (its noise.frag comment says the array form cost +0.034 ms per material draw). After the merge, both programs carry the folded grain, so the rest figure, and probably the 1.47 ms swing figure, overstate current cost by a few hundredths of a ms. The row links the evidence doc that carries the measured revision, so the text is revision-bound evidence rather than wrong. No action needed for merge; it could be noted the next time the cost A/B is rerun.

## 1. Textual conflict resolution

- `docs/materials/upstream-divergence.md`: the generated section was regenerated rather than hand-merged. `python3 tools/upstream-report --check` exits 0 on the merge tree. The totals are consistent: class B 57 = main's 56 + the branch's one new seam path; class C 49 = main's 49. Per-file counts combine both sides (for example, `niri-config/src/lib.rs` +2133 = main's +2091 + the branch's +42).
- `src/tests/client.rs`: the resolution keeps:
  - the branch's `State::create_subsurface`;
  - main's `create_layer_popup`, `Popup` and `add_colored_subsurface`;
  - main's `XdgPopup`, `XdgPositioner` and `WlRegion` Dispatch impls.

  Exactly one copy remains of each of the following: the `subcompositor` field (line 63), its initializer (205), the registry bind (783–785), and the `Dispatch<WlSubcompositor, ()>` and `Dispatch<WlSubsurface, ()>` impls (1125, 1138). Both helpers create the subsurface with `()` user data, which matches the one remaining impl. `git diff materials-26.04 418b0130 -- src/tests/client.rs` holds only the branch's additions (`create_subsurface`, `attach_new_shm_pixels` x2, `attach_shm_pixels`) and the reordering of the impls. Nothing from main was dropped.
- `src/render_helpers/material/view/solid.rs`: rustfmt only rewrapped a doc comment. No code changed.

## 2. Semantic interactions in files both sides changed

Hunk-level check: for each of the 14 files both sides changed, `diff(base→materials-26.04)` equals `diff(45281a91→merge)`, and `diff(base→45281a91)` equals `diff(materials-26.04→merge)`, both compared as sorted +/- line sets. The only exceptions are `upstream-divergence.md` (regenerated) and `client.rs` (the subcompositor deduplication above). Every hunk from each side therefore reached the merge intact. The behavioural questions follow.

- **Shaders, noise.frag (main's `noiseRow` fold).** The branch's `#ifdef MATERIAL_VIEW` glide (`p = (fragCoord + g_seed_glide) / scale`) sits before `base = floor(p) + offset`, which main's rewrite still uses unchanged. The folded accumulator therefore sees the gliding lattice in the view variant, and the rest program keeps `fragCoord / scale`. The `scale <= 1` branch (whole-pixel seed) is unchanged on both sides. The branch's `variant()` preprocessor model accepts the file (only the MATERIAL_VIEW forms), and render-pipeline.md's "grain above scale 1 adds the fractional remainder back" still holds.
- **Shaders, mod.rs (main's `surface_remap` program).** `surface_remap` is a `compile_custom_texture_shader` program (smithay's `//_DEFINES_` variants) that includes `material/common.frag`, which now carries the branch's `#ifdef MATERIAL_VIEW` `g_seed_glide` declaration. MATERIAL_VIEW is never defined there, so the block drops out and the remap's text is the same as on main. `the_surface_remap_shader_compiles_on_the_test_renderer` (src/tests/shell_hue.rs:259) will confirm this in the running full suite. Main adds no material uniform. `material_view_uniform_names()` builds on `material_uniform_names()`, so any later addition would reach the view variant automatically. Main changes no `ProgramType` or material program variant.
- **render_helpers/material/mod.rs.** Main only removed the stale e88df7 known-limitation comment and reordered imports, so nothing interacts with the branch's view-uniform plumbing.
- **Layout and the frost-drag fix (f1e91880).** A material tile under interactive move now has no alpha animation and draws directly instead of through the alpha offscreen. The branch's interactions:
  - Swing start already excludes an interactively moved tile: `focused` is None while `Moving`, and `Monitor::tile_mut` only searches workspaces.
  - A swing already running when a drag starts now renders through the direct path. That path applies the footprint crop and culling (`material_out_of_view` returns `!footprint.overlaps(view)` while posed), so the footprint is honoured. Before the merge, the same frame went through the alpha path's shifted view crop, which the branch also supports.
  - `cull_body` can now apply to a dragged material tile (it was suppressed under alpha). It uses the same swing-aware `material_out_of_view`, so it cannot cull a footprint that is in view.
  - The alpha path stays reachable for material tiles through scrolling.rs's `animate_alpha` (lines 930, 934, 1093, 5204). The branch's `the_view_crop_follows_the_tile_into_an_offscreen` drives `animate_alpha` directly, so it still tests a live path. render-pipeline.md's mention of "the alpha path's view crop" is still accurate.
- **Layout, monitor and shell hue.** Main's `Monitor::workspace_blend` and the shell-hue resolver (`Niri::update_shell_hue`) only read workspace or active-window ids and signal accents. They are independent of `focus_origin::observe`, `tile_centre_on_output` and `tile_mut`. No new `LayoutElement` implementor arrived from main, so the branch's `background_effect_visible` trait method covers every implementor (Mapped, TestWindow, and the visual-tests window).
- **Config (lib.rs, material/mod.rs, pipeline.rs).** Main's lib.rs adds `validate_surface_remaps` and the oklab and face modules. pipeline.rs and the other config files only reorder imports. The response struct, `response_fields()`, `ResolvedResponse::resolve` and the `STAGES` slab entry (`VIEW_RESPONSES`) are untouched by main, so the three view keys still parse, inherit from `default` and reach the slab stage. The surface-remap validation reads only the glass (`material.resolve().glass`) and rejects a `response=`, so the view keys cannot leak into the remap's face match. `resources/materials/pipeline.json` changed only on the branch, and main's pipeline.rs edit is formatting, so the generated-JSON staleness test is unaffected.
- **Tests (mod.rs, ring_look.rs, layout/tests.rs).** These are union merges: `frost_drag`, `shell_hue` and `view_tilt` are all declared, and the `view_swing` module is kept. Main's ring_look.rs change only rewraps a comment and reorders imports. Main dropped the `MergeWith` import from layout/tests.rs, and the branch's view_swing tests do not use it (the tree compiles).
- **Docs.** render-pipeline.md combines main's surface-remap paragraph and three-row effects table with the branch's step-3 footprint sentence and *Under a swing* section, with no contradiction: the remap is a layer-surface texture program outside the material stages. main did not touch material-config.md, and its README change (frost-drag resolution) does not conflict with anything the branch documents.

## Not checked

- Runtime results: the full suite run in progress covers the GPU-compiled programs, including the remap shader with common.frag's new block and the view variant with the folded grain.
- Pixel equivalence of main's grain fold under the view variant beyond reading the code: the branch's grain-sensitive view tests in the full run cover it.
