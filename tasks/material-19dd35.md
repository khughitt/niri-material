---
id: material-19dd35
title: "Ice: cracks optic and the ice preset, tuned headless"
status: todo
priority: 2
size: m
complexity: mid
process: direct
needs: [owner]
created: 2026-10-15T18:39:46Z
updated: 2026-10-15T18:39:46Z
depends: []
parent: material-bb3fe5
tags: [material, rendering]
agent: claude-code/claude-opus-5-5
spec: docs/specs/2026-09-10-material-optics-design.md
---

Why: the first half of material-bb3fe5. Spec docs/specs/2026-09-10-material-optics-design.md §7.1 (cracks, :364-385) and §8 (ice preset, :454-522) still stand; its Status line lists them open.

Established (checked 2026-10-15 at 2dd3b9fa; do not re-derive):
- Author the optic by docs/materials/adding-an-optic.md, not the spec's §2 list. edge-highlight (13b54584) is the current three-file pattern: niri-config/src/material/optics/, src/render_helpers/material/optics/, shaders/material/<name>.frag, plus the lib.rs re-export, a stage in niri-config/src/material/pipeline.rs, pipeline.json and the parameter table regenerated with MATERIAL_DOCS_UPDATE=1, a material-config.md section and a render-pipeline.md row. Tests enforce hook order and the schema file.
- The specular hook is now vec3 cracks_specular(vec3 specular, Surface s) (glass-edge-optics §3.4); the spec's hook table at :115 is stale.
- No optic uses the normal hook yet: cracks adds the first call site in main.frag after the ripple (~:52) and an optic("cracks","normal") stage after ripple. Surface.perturbed then carries the crack bend into reflection and edge-highlight; declare that Interaction in pipeline.rs.
- No cellular noise exists (hash12, snoise, noiseValue only; no hash22): the F1/F2 search is new code, and costs about 18 hash12 calls, not the spec's nine. Seed from mat_jelly_seed.
- Packaging: PKGBUILD installs resources/materials/*.kdl by glob (no change); add ice.kdl to both Cargo.toml asset lists (rpm ~:176, deb ~:192) and name it in every_preset_parses_alone_and_through_an_absolute_include (niri-config/src/lib.rs ~:797).
- The §8 preset values predate the height-field bevel and Fresnel, so expect retuning. Noise layers now allow up to four noise nodes with scale=; frost grain may use them.

Choices left to the implementer: whether cracks_specular recomputes the mask from s.p or shares it; single versus layered frost noise.

Done: the optic is neutral at amount 0 and its effect is measured in a frozen-pixel test (as src/tests/glass_edge.rs does, no host); ice.kdl parses and installs; docs are updated, including the spec's Status line and stale hook table. The preset is tuned with docs/materials/scripts/glass-parameter-sweep.sh (add cracks to its key list; BACKDROP=aperiodic-s20260929 from 433fb253 shows refraction breaking along the cracks), and the owner judges a self-explaining sheet. No quiet host needed.
