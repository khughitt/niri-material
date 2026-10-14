# Material cohesion on Noctalia surfaces

2026-10-14. Scope handoff for `material-62347d`; not an approved design.

## Problem

Make Noctalia's bar and panels feel part of the same material as the windows.
The source asks for cohesion, without choosing a surface treatment or renderer
extension. Start with one representative panel; the bar is the second consumer.

## Current behaviour and evidence

- At main checkout `d9914514`, `niri-config/src/layer_rule.rs` exposes
  background effects, opacity, shadows and corner geometry, but no material
  reference. `src/layer/mapped.rs` renders surface pixels, shadows and
  `background_effect::render_for_tile`; it has no `MaterialState` or material
  render element. Window material rules therefore cannot simply select a bar.
- `docs/wiki/Configuration:-Layer-Rules.md` documents blur, noise, saturation,
  namespace matching and separate popup overrides. These are an existing route
  for a first comparison, not evidence that they reproduce the window glass.
- Read-only `niri msg --json layers` on 2026-10-14 reported
  `noctalia-bar-default` on the top layer and `noctalia-wallpaper` on the
  background layer. No Noctalia panel was open; its namespace, surface geometry
  and transparency are unverified.
- The original capture is
  `mindful:thought:d6cae16e5bd867ec390c14ab866e7619`: use material on bar,
  panel and other Noctalia surfaces for cohesion. No overlapping open research
  was found in the local task corpus.

## Constraints

- Keep wallpaper and unrelated shell surfaces outside the first treatment.
- Establish the actual panel's namespace, buffer extent, alpha and background
  painting before choosing a shader footprint; a surface rectangle may contain
  empty space around the visible panel. Compositor effects cannot be assumed
  visible through an opaque client background.
- Preserve client text, input regions, exclusive zones, popup placement,
  output scaling, capture blocking and damage behaviour. Any renderer design
  must follow `docs/materials/render-pipeline.md`.
- Research can inspect source and saved configuration without using the desktop.
  A rendered proof uses a nested compositor (`nested`); opening panels or
  changing the live desktop needs separate host-use authorization. Appearance
  review needs `owner`; settled cost evidence needs `quiet` if later required.

## Alternatives

1. **Existing layer effects plus Noctalia styling (current first experiment).**
   Test one panel with matched tint, transparency, blur and geometry. This is
   the smallest way to find out which cues supply cohesion.
2. **Opt-in material rendering for layer surfaces.** Reuse the window material
   system if the comparison needs its slab, edge or interior optics. This needs
   a reviewed design for surface footprint, client composition and lifecycle.
3. **Client-side imitation of the glass.** Consider only if the required shape
   is unavailable to the compositor; it introduces a separate optical model.

Prefer the bounded comparison before a renderer extension. No visual choice
or claim of adequate cohesion has been made.

## Unanswered questions

- Which panel offers a small, representative target, and what does its surface
  expose? The feasibility follow-up answers from Noctalia source/configuration
  and, if available, a nested instance.
- Do existing effects supply enough cohesion, or are slab optics essential?
  The owner judges a labelled comparison after that follow-up.
- If a material pass is needed, can it use a known visible panel rectangle and
  preserve shell behaviour? Source tracing answers the feasibility; design
  follows only if the first comparison justifies it.

## Proposed decomposition

- Keep `material-62347d` as an idea until feasibility establishes a concrete
  surface and an experiment the owner can judge.
- `material-55380c` is the child goal within the existing material-composition
  lane `material-3aa1f2`.
- `material-e27dbc` identifies one panel and specifies the smallest comparison
  (P2, small, mid complexity, direct, `nested`). `material-62347d` depends on
  it. Its completion records the finding here and uses `--no-reviewed` to
  readmit that idea before the result branch merges.
- File implementation or renderer design only after the finding; do not add
  a general shell-material framework in this scope pass.
