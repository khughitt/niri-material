---
id: material-f72d5c
title: "Edge light: an attenuation-tinted rim term that lights the bevel from the scene"
status: todo
priority: 1
size: m
complexity: mid
process: planned
created: 2026-10-09T23:29:33Z
updated: 2026-10-09T23:32:49Z
depends: [material-ac926f]
parent: material-6062fd
tags: [rendering, material]
source: material-ad1780
agent: claude-code/claude-fable-5-1
---

Stage 4 attenuates along h / -t.z. That is physically right: a ray entering the bevel bends toward the outward-tilted normal, so it travels inward under the face and exits the back, and at the live active look (thickness 31.2, bevel 12, #4c563a over 9 px) the rim is still 19 px thick, so the whole band transmits ~2e-4 and the height-field profile cannot show. What real dark glass has and this model lacks is piped light: backdrop light gathered inside the slab by internal reflection and emitted at the edge, which is why smoked glass has a luminous rim over a dark face. Design: an edge-light term on the bevel, in the specular stage beside the reflection optic (or as the reflection optic's rim weight): the scene sampled just beyond the silhouette (as reflection does), weighted by a rim profile that peaks at the silhouette (e.g. u^2) instead of raw Schlick (f0 1.5 % at ior 1.28 lights one pixel), and tinted by attenuation-color over the path back to the face edge ((1 - u) * bevel / attenuation-distance), so the rim fades from scene-coloured at the silhouette into the dark slab at the face edge. Neutral at amount 0; face and outside untouched. Amend docs/specs/2026-09-30-glass-edge-optics §3.2/§3.3 and render-pipeline.md stage 6; declare in the pipeline schema; bevel.rs mirrors the profile with a test pinning face identity and the rim brighter than the face edge. Evidence: frozen dumps before/after (face identical, outside identical, bevel gradient toward the silhouette) and an owner-judged sheet on the live looks over a flat and a patterned backdrop. Proceed only if the spike's sheet reads as one slab.

## Notes

- 2026-10-09T23:32:48Z (materials-26.04): 2026-10-09 correction before the spike: the side-exit path idea was wrong physics (the refracted ray bends inward, never out the side wall); retitled and rebodied as an edge-light term. The spike (material-ac926f) probes this form.
