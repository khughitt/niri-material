# Embedded textures and material composition: scoping brief

Scope pass: 2026-10-06. Handoff for `material-3aa1f2`, not an approved design.

## Problem

Give glass a useful textured reading background and reuse shared assets where
there is a real consumer. This pass covers embedded textures (`material-05c901`),
two-material composition (`material-197db1`) and asset consumption
(`material-599548`). Their shared source `thought:3f94e656b70f4e5585c1cb60c166e4da`
was retrieved successfully; its stone/paper, shared-asset and composition
suggestions agree with the preserved task bodies.

## Current behaviour and evidence

- `niri-config/src/material/mod.rs::ResolvedMaterial` selects one glass
  definition with named responses. `resources/materials/aurora.kdl` already
  combines glass with Aurora; `material-f0fc7b` completed those optics.
  The ice cracks/preset task `material-bb3fe5` remains open. Adding Aurora
  does not itself require combining two independent base materials.
- `prelude.frag::sampleBackground/tap` refracts the composed background and
  backdrop textures. `MaterialRenderElement::draw` binds the client texture
  and the two prefilter levels of each background buffer. No material-local
  imported image is part of that binding or the resolved glass definition.
  A textured wallpaper is already sampled, but does not establish embedded
  material depth or window-local coordinates.
- The [pipeline](../materials/render-pipeline.md) distinguishes source textures,
  normals, transmitted color, interior light and encoded film. `pipeline.rs`
  describes their different carriers/composition laws and marks every site
  non-orderable. `adding-an-optic.md` defines `within` as additive attenuated
  light; that is not automatically a stone/paper image layer.
- Noise sites landed (`1d2777aa`); noise layers `material-3fcba2` remain in
  progress. Their concrete grain behavior does not establish a common stack
  for textures, base-material blending and profiles. Prism's rack goal
  `prism-a03862` is still open and its task describes lowering to native fields.
- Related task records already retain registry coordination `ops-c8b158`,
  catalog work `ops-1b5a9c` and provider investigation `forge-e0743c`.
  The provider remains an idea; no shared API is verified here.
  `mind6-b4d838` has its own texture/skybox requirements; shared identity
  vocabulary `mind6-5c0193` is shelved.

## Constraints

Preserve opaque client pixels, linear-light/alpha boundaries, neutral output,
cache invalidation and idle/motion behavior. Do not change accepted text or
ring appearance as a side effect. If the desired layer needs optical treatment
of opaque content, reuse `material-d257d9`'s content-stage design. Keep existing
sources and foreign goals intact. The registry hub already depends on
`material-599548`; making this consumer depend on that hub would create a cycle.
Any later prerequisite names the actual provider contract, checked for cycles.
Reuse `forge-e0743c` instead of filing another substrate investigation.

## Alternatives

1. **Current lean: one reading-texture proof first.** Compare existing refracted
   background sampling with the smallest genuinely material-local texture
   proposal, then report resource requirements to existing registry work.
2. Build a native embedded image layer immediately. Coordinates, placement,
   blending, loading and reload lifetime still require decisions and a reviewed
   visual proof; current hooks alone do not settle them.
3. Build a general material mixer, recursive layer stack or registry client now.
   Shelve generic composition and retain registry integration as an unresolved
   consumer handoff until a concrete visual outcome and provider contract exist.

## Unanswered questions

- Is a refracted background texture sufficient, or must the reading layer follow
  the window at a distinct slab depth? `material-56f91a` traces feasibility and
  proposes a bounded frozen-pixel proof; the owner later judges its look.
- Which decoded image, color/alpha, sizing, addressing and cache/reload contract
  does that proof need? The same task supplies the minimum consumer requirements;
  existing catalog/provider work settles the shared service or store.
- What wanted second-base-material result requires layer, blend or convolution
  beyond one glass definition plus optics? A concrete consumer and owner-reviewed
  outcome must answer before `material-197db1` returns to active review.

## Proposed decomposition

- `material-05c901`: **briefed**, awaiting a concrete embedded-texture proof.
- `material-599548`: **briefed**, reusing existing registry/catalog/provider
  work and awaiting this consumer's requirements; no new registry task or API.
- `material-197db1`: **shelved** until a concrete second material/texture
  consumer needs an owner-approved composition that existing mechanisms cannot express.

### Texture feasibility

- `material-56f91a`: P2, small, mid complexity, direct research. Trace the current
  texture path, recommend one bounded proof and record resource requirements.
  No production optic, asset loader, shared identity vocabulary or live capture.
  Completion updates this brief and adds findings to `material-05c901` and
  `material-599548` in the same commit. Existing lane and foreign task ownership remain intact.
