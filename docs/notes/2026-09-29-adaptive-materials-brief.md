# Adaptive materials scoping brief

## Problem

Make useful combinations of glass parameters easier to find, compose and reuse.
This pass covers parameter structure (`material-0c7eed`), compositional profiles
(`material-764d8c`) and learned render order (`material-e2f01a`), which share an
adaptive-material source and tag. Goal: `material-0f225e`. This is a scoping
handoff, not an approved implementation design.

## Current behaviour and evidence

- `niri-config/src/material/params.rs::ParamSpec` records parser bounds,
  defaults, units and read/write helpers; its tests check parser agreement.
  It has no learned interaction, preference or influence model. The shipped
  metadata task is `material-cc6d07`.
- `Material::resolve` in `niri-config/src/material/mod.rs` resolves one glass
  definition and named response overrides over a default response.
  `src/layout/tile.rs::resolve_material` selects one definition and response.
  Separately, `src/window/signal.rs::fold` selects signal level/motion and
  accent/tag, and `shaders/material/main.frag` mixes the accent into ring light.
  `material-930c55` records owner-confirmed Familiar hues on 2026-09-24.
  These are existing composition mechanisms, not a general profile algebra.
- `src/render_helpers/material/optics/mod.rs::OPTICS` is a static registry;
  `src/render_helpers/shaders/material/main.frag` explicitly calls saturation,
  noise, aurora and iridescence at different physical stages. Changing registry
  order alone does not reorder those calls. Commit `e33aa968` moved backdrop
  colour optics before attenuation; `d5b4188a` integrated the depth-ordered
  rendering work. The [pipeline](../materials/render-pipeline.md) records the
  resulting semantics.
- The [optics design](../specs/2026-09-10-material-optics-design.md) describes
  Prism profiles as full value snapshots and a device rack in shader order.
  Current Prism implementations and the referenced cross-project tasks were
  not inspected in this checkout. A rack is not evidence of runtime reorder.

The original Mindful thought `a476e6bcd1fd4297b70824758235d821` could not be
retrieved locally. Original task bodies, sources and notes remain intact.

## Constraints

Preserve optical depth, linear-light/encoded-light boundaries, opaque-client
bypass, neutral behavior, and idle/motion gates. Parser bounds establish legal
inputs, not visually good regions. Cost sensitivity, displacement and visual
preference are different quantities: `material-31074f` measures cost and
interactions; `material-bb8480` calibrates displacement, not aesthetic quality.
Reuse these open investigations and the focus-swap study `material-8e3b73`;
their existing goals and sources remain unchanged. New model/API behavior
requires reviewed design and plan documents after a concrete gap is established.

## Alternatives

1. **Current lean: retain fixed optics, named definitions/responses and signal
   composition.** Use existing interaction and focus-swap evidence to identify
   a specific missing behavior before adding metadata or composition operators.
2. Add a small empirical relationship table or one explicit profile operator
   after the findings establish its consumer, meaning and correctness check.
   Store measured evidence separately from parser facts unless a design shows
   why they should share a representation.
3. Build a general profile algebra and optimize effect permutations now.
   Defer: legal order constraints, preference data and the objective remain
   unresolved; more dynamic range alone has not been established as better.

## Unanswered questions

- Which joint parameter effects are repeatable, and which matter visually?
  `material-31074f` can answer the measured cost part; owner-rated examples and
  an explicit consumer are still needed before claiming preferred subspaces.
- What desired base/Familiar/focus combination cannot the current mechanisms
  express? `material-8e3b73` supplied the state/parameter inventory and visual
  baseline; the owner kept the hard cut (2026-10-02). Focus can select a
  definition (hard cut, fresh state), a named response (in place, ring and
  signal values only) or signal accents (folded and crossfaded independently),
  so focus-dependent glass needs two definitions today. This clarifies the
  question without settling generic override, blend or constrain semantics.
- Which orders could legally vary, and what objective justifies learning them?
  A future renderer design and reproducible comparison must answer. Verify
  `prism-a03862` and `prism-542904` then; their old references do not prove that
  the native renderer supports reordering today.

## Proposed decomposition

- `material-0f225e` groups the three previously unparented ideas; P2, medium,
  high complexity, planned. No duplicate research or design task is filed.
- `material-0c7eed`: **briefed**; reuse `material-31074f`. Its completion note
  must wake this idea alongside its existing waiting ideas and update this brief.
- `material-764d8c`: **briefed**; reuse `material-8e3b73`, now complete: its
  composition finding is recorded above and noted on this idea. No additional capture lane
  is required; remaining profile questions stay explicit.
- `material-e2f01a`: **shelved** until a renderer-supported constrained reorder
  mechanism exists and a reproducible visual or cost objective justifies
  comparing legal orders. Verify the Prism prerequisites before unshelving.

The two reused investigations retain their existing parents and scope. No new
capture, benchmark or runtime change was made during this pass.
