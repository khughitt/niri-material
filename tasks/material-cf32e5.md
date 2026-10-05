---
id: material-cf32e5
title: "Noise placement: a site attribute selecting backdrop, glass, or film grain"
status: todo
priority: 2
size: m
complexity: high
created: 2026-10-05T01:45:48Z
updated: 2026-10-05T09:36:17Z
depends: [prism-eef38f, material-a00785]
parent: material-3aa1f2
tags: [noise, rendering, cross-project]
agent: claude-code/claude-fable-5-1
---

First device that can move. Give the glass noise node a site attribute (backdrop | glass | film) and implement the two new placements:

- backdrop: grain the sharp effect-buffer texture before the Kawase blur and the prefilter pyramid (src/render_helpers/effect_buffer.rs), cached with the sharp texture. Per output, like blur passes: one amount, no focus split. Pre-blur grain is low-pass filtered, so heavy roughness erases it; that is expected and the prism design (prism-eef38f) records it as a dominance interaction.
- film: grain at the reserved post hook (render-pipeline.md stage 9), over transmitted light plus ring, aurora, and glint. Glass only; opaque client pixels bypass the shader.
- glass: today's behind placement, byte-identical for configs that name no site.

Settle: default (glass), invalidation of blur and pyramids when backdrop grain changes, unsupported-site error, seed coordinates and grain scale per site, and how future noise layers (material-3fcba2) would select a site, without implementing layers.

Prove the look before the contract: compare today's grain with pre-blur grain on one backdrop at several roughness levels. "Cheaper" is a hypothesis: measure static wallpaper, animated backdrop, and parameter dragging separately; backdrop grain adds a texture generation and invalidates the blur and pyramid caches.

The site vocabulary comes from the pipeline schema designed in prism-eef38f (sites, scope, composition law, coverage); do not start before that design is reviewed.
