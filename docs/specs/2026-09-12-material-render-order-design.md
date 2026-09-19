# Material render order by depth

**Status:** reviewed for planning on 2026-09-12, including transmitted
chamfer grain without a mask. The
[implementation plan](../plans/2026-09-12-material-render-order.md) is approved
for inline execution. Behind hooks and signed transfer helpers are committed
at `e33aa968`; candidate pixel and signal checks have passed. Strict
GPU cost passed on 2026-09-18 in a retained TTY matrix: nine P8-only settled
sub-runs, with all three case medians at 0.020 ms and +0.0% candidate
differences. The within implementation, its linked GLES pixel matrix, and its
strict within-cost matrix have passed and Task 2 is committed at `e79b226b`. The older ring pixel
matrix also passed under its documented outer-settle, pixel-only limitation.
The older aurora visual, cadence, and strict cost-only evidence passed, as did
the final software gates. The final whole-branch review found only the scoped
documentation reconciliation recorded in the parent closeout; neither child
commit is merged or deployed. The old-build baseline records the required additive-light failure,
with a user-authorized GPU quietness waiver for baseline and candidate pixels
only; timing remained strict. Source baseline: `8e3d890d`; reviewed spec
revision: `522a09fe`. **Amended and approved 2026-09-17:** the within section
gains ring scatter, face placement and a motion deferral after a default-glass
capture showed the ring reading as a plated bezel. The user selected SDD
execution.

**Goal:** `material-5b3107`. Children: `material-f8b6e9` (behind: noise and
saturation), `material-92edaf` (within: ring and aurora).

## Approaches

**A. Depth-ordered hooks — recommended.** Keep all work in the material
program. Transform the averaged linear backdrop at `behind`, before full
attenuation. Add interior light at `within`, evaluated at its own refracted
landing point and attenuated over the remaining path. Keep surface light
unattenuated, then encode and reserve `post` for film effects.

**B. Interior image in `tap` — rejected.** Composing ring and aurora in
`sampleBackground` repeats their evaluation through the sample loop (up to
eight taps, with additional per-channel sampling under aberration). A uses
the same perturbed normal at a single interior evaluation, retaining the
ring's own per-channel band evaluations. This preserves landing-point
refraction, but does not reproduce B's multi-tap smear of interior light.

**C. Masks only — rejected.** Multiplying noise by one minus the bevel mask
and leaving the ring unchanged does not establish the requested depth order.

## Historical baseline order and implementation evidence

At source baseline `8e3d890d`, the
[pipeline](../materials/render-pipeline.md) matched the then-current
`src/render_helpers/shaders/material/main.frag`:

```text
opaque window bypass → slab geometry → perturbed normal → averaged taps
→ full-path attenuation → Fresnel / iridescence / accent
→ masked ring + aurora, each weighted by att^0.2 → surface sweeps
→ sRGB encode → saturation → noise → coverage → window composition
```

The ring still uses `lightShift` at `0.6 * thickness`, with the shared shift capped
at half `ring-inset`. Chromatic offsets are added after that cap. Its
display-fragment mask is zero on the face. `aurora_emissive` currently
receives the normal but evaluates its field at the unrefracted position.
Both already multiply their light by `pow(att, vec3(0.2))`.

`NoiseOptic::values` and `SaturationOptic::values` confirm that omitted
amounts inherit from the global blur block only while backdrop blur is
effective. Explicit material values override this rule. Grain type never
inherits. These policies stay unchanged.

## Target order

Depth fraction `d` runs from the back of the slab (`0`) to the viewer-facing
surface (`1`). The remaining fraction toward the viewer is `1-d`.

| Depth | Stage | Operation |
| --- | --- | --- |
| Geometry | `normal` | Perturb the normal before sampling, as today. |
| Behind | taps → `behind` → full attenuation | Average linear backdrop taps; apply saturation then noise once; multiply by `att`. |
| Within | `within` | Add ring and aurora evaluated at their refracted interior landing points, each weighted by `att^(1-d)`. Both use `d = 0.8`. |
| Surface | `specular`, `emissive` | Fresnel, iridescence and accent retain their order; sweeps remain additive surface light with no attenuation. |
| Film | encode → `post` | Encode the sum to sRGB; no current optic remains at `post`. |
| Composition | coverage → window | Preserve premultiplication, opaque bypass and the single application of window-rule opacity. |

In linear light, before encoding:

```text
glass = behind(averaged_sample) * att
      + ring_at_landing_point * att^0.2
      + aurora_at_landing_point * att^0.2
      + specular + surface_emissive
```

This changes hook positions inside one draw, not output render-pass order.
Shared sharp/blurred textures, prefilter pyramids, cache keys, uploads and
damage ownership remain as they are. The shared-texture cache argument in
the [2026-09-02 design](2026-09-02-material-noise-saturation-design.md)
still applies; its choice of finished-glass postprocessing is superseded
only when the behind child lands.

### Behind: saturation and noise

Rename `saturation_post` and `noise_post` to `saturation_behind` and
`noise_behind`. Their boundary is linear colour; internally encode to sRGB,
apply the existing formula, and decode the result. Preserve saturation
before noise, luma coefficients, noise seed and offset, all three grain
formulas, and the `lightness` Oklab operation. Neutral branches return the
original linear input before any round trip.

The formulas retain their existing colour spaces; moving them before tint
and additive light intentionally changes the final image. Saturation zero
now desaturates the sampled backdrop, not coloured tint or emitted light.
Grain remains seeded by `gl_FragCoord.xy`: applying it after averaged taps
does not make its pattern refract or become anisotropically smeared.

The current transfer helpers evaluate `pow` branches even for values that
select the linear branch. White/fine grain and saturation can produce
negative channels. The implementation must make those evaluations defined
without silently clamping away signed grain: bound only the unused power
branch's base and preserve the signed linear branch. Retain the existing
explicit gamut clamp for Oklab lightness noise. Check dark, bright and
out-of-gamut intermediate values, not only mid-grey.

### Within: ring and aurora

Ring stays inline in `main.frag`. Remove its display-fragment bevel mask
and the associated mask branch; retain the response selectors and the
existing zero-chamfer guard. Keep `light-ior`, per-channel aberration,
filament shape, half-inset cap, focus drift, accent response and jelly gain.

Use `lightShift` with the perturbed normal and the remaining geometric path
`thickness * (1-d)`. At `d = 0.8`, this changes the ring's current lookup
path from `0.6 * thickness` to `0.2 * thickness`. The cap still applies only
to the shared shift; chromatic offsets remain outside it. The visible change
is small: at the stock default glass the uncapped shift at `0.6` was about
6.9 px against the 2.5 px cap, and at `0.2` it is about 2.3 px, just under
the cap, so this example's shared shift changes by roughly 0.2 px. The cap
still limits sufficiently dense settings. At fixed normal and index, it now
requires three times the previous thickness to bind; it does not bind at
every previously capped setting. Per-channel offsets also shrink with the
shorter path.

Rename `aurora_emissive` to `aurora_within` and evaluate its existing field
at a refracted landing point using the same remaining path and existing
light-path index `1 + (ior - 1) * light-ior`. Aurora needs neither a
silhouette cap nor new per-channel field evaluations. Preserve its field
scale, phase, colours, intensity and cadence policy.

**Amendment, 2026-09-17.** A capture of the default glass showed the ring
reading as a plated bezel: a uniform band with a hard inner edge, unchanged
by roughness or tint. Mask removal answers the hard edge. Two further
changes make the ring behave as light inside the volume rather than paint
on the chamfer; the third records what this child does not do.

*Scatter.* Frosted glass diffuses interior light as it diffuses the
backdrop. Upload `mat_scatter = roughness * clamp(ior * 2 - 2, 0, 1)`, the
normalised prefilter selection `effect_buffer.rs` already computes, and
widen both Gaussians of `filamentBand` with it, conserving each term's
integral so scattered light spreads without brightening:

```text
w_c  = width + 6 * scatter
w_h  = 9 + 18 * scatter
band = (width / w_c) * exp(-2 ((d - inset) / w_c)^2)
     + 0.3 * (9 / w_h) * exp(-2 ((d - inset - 2) / w_h)^2)
```

At `scatter 0` this is the existing band exactly. The widening constants are
art-directed, not derived from the pyramid: its top level averages the
whole source, which no filament width should match. The per-channel
aberration bands use the same widths. `filamentBand` takes the scatter as
an argument so the prelude stays free of a new uniform read.

*Face placement.* Drop the `ring-inset + ring-width <= bevel` validation in
`niri-config`, both error sites and the test that expects them. Its
rationale, keeping text untouched, is served by the opaque bypass, not by
the constraint; on a translucent face the ring showing through the window
is the embedded look this child exists to produce. `ring-inset` keeps its
0–128 range and `ring-width` stays positive. The half-inset cap still
applies to the shared shift, so a larger inset also permits a larger shift.
The defaults (`ring-inset 5`, `ring-width 2.6`) do not change here; whether
the stock ring should move onto the face is a tuning judgment for the
captures, filed as a follow-up if so.

*Motion.* This child changes no motion. The focus drift stays as it is, and
the request for subtle movement is judged by `material-0e130e`, which owns
the one-shot focus pulse and the visibility/activity gating of sustained
attention motion; that task is scoped against the ring this child lands,
not the masked one. Motion captures here record reach and deltas under the
existing drift only.

Each within contribution owns its fixed depth and applies attenuation
exactly once; `main` adds the returned linear contribution to `within`.
The ring follows that same rule inline. No configurable depth or generic
layer object is needed. `emissive` means surface light and never applies
attenuation, even if an existing hook signature carries shared context.

The `att^0.2` factor is preserved exactly. Whole-image brightness is not an
identity guarantee: removing the ring mask and moving either landing point
changes which radiance is evaluated. Dense-glass comparisons must separate
the unchanged attenuation law from those spatial changes.

No new oscillator, cadence source or motion parameter is proposed; the
subtle-movement request is deferred to `material-0e130e` as stated above.

### Hook recipe

Add two GLSL hook names to the existing recipe; keep the Rust `Optic`
contract and registration mechanism.

| Hook | Signature | Neutral |
| --- | --- | --- |
| `behind` | `vec3 <name>_behind(vec3 color, vec2 fragCoord)` | Return the linear input unchanged. |
| `within` | `vec3 <name>_within(vec2 p, vec3 n, vec3 att, float innerDist)` | Return zero linear light; an active optic includes its remaining-path attenuation. |

Existing `normal`, `specular`, `emissive` and `post` signatures remain.
Document the surface-only meaning of `emissive` and the encoded-colour
boundary of `post`. Keep calls ordered within each hook. Since both
registries describe render order, update renderer `OPTICS` and config
`ORDER` together as the children move stages; retain their existing
agreement test. This is internal ordering, not a KDL grammar change.

## Acceptance and review corrections

These corrections were accepted for planning. They avoid promising
properties the selected equations do not provide.

### Noise

The requested gate, "bevel-band grain drops to zero", cannot hold generally
under A. A transmitting bevel still contains `behind(sample) * att`; `att`
need not be zero and is exactly one for white attenuation colour. Moving
noise protects the additive glint, ring, aurora and sweeps from being
postprocessed; it does not eliminate grain in transmitted bevel content.

**Recommended gate:** in deterministic linear-light comparisons, toggling
noise changes only transmission. Isolate additive-light contributions with
paired lights-on/lights-off captures at both noise settings; their
difference must remain unchanged within capture quantization tolerance.
Report the full bevel's grain measure separately. Exact zero over the whole
bevel would require a mask or another rendering change, outside A as
proposed; do not silently add one.

Reuse the existing grain smoke's signed on/off residual standard deviation.
At default amount `0.02`, compare a flat text-free face against the previous
build using a pinned uniform midtone fixture with white attenuation colour
and additive lights disabled. Require the standard deviation to match
within 10%, and report actual delta. This isolates preservation of the grain
formula. Also capture the dense-glass fixture and report face and bevel
measures: preservation of its previous final-image amplitude is not implied
by moving grain before attenuation. Pin seeds, geometry and animation state.

### Ring

Replace every rest-state face-zero assertion with a reach measurement from
the actual slab silhouette; opaque text retains byte identity through the
existing bypass. Motion captures record interior reach and pixel deltas,
with timing and alignment limits stated, rather than gate them.

Retain the proposed `ring-inset + 2 + 18 + cap` logical-pixel radius as a
reported reference contour, where `cap = 0.5 * ring-inset`. It is not a
universal zero-change bound: `filamentBand` has Gaussian tails, allows a
configurable core wider than the fixed halo, and adds uncapped chromatic
offsets. At two halo widths its halo term is still `0.3 * exp(-8)`.

**Recommended gate:** pin the rest fixture and a visible-difference
threshold of one 8-bit channel code; derive its reach bound from both
Gaussian widths at the fixture's scatter, maximum enabled gain, output
encoding and maximum per-channel displacement. Two fixtures join the
matrix for the amendment: a roughness fixture (`roughness 0.5` on the
pinned glass, so `scatter 0.5`) whose bound uses the widened widths, and a
face fixture (`ring-inset 20` under `bevel 12`), which the old validation
refuses, whose ring must change face pixels inside the structural chamfer
under a translucent client. The unnormalised gain stays in the derivation;
integral conservation only lowers the peaks, so the bound remains
conservative. Record maximum delta and changed-pixel count
beyond the proposed reference contour as well. The implementation plan must
write down that fixture and calculated bound before capturing. An exact
finite-support gate would require truncating the Gaussian or adding a
spatial cutoff, neither proposed here.

### Shared checks

- Neutral noise/saturation and disabled ring/aurora preserve the baseline's
  decoded pixels; active optics compile in the real GLES program without
  material fallback warnings or non-finite colours.
- Opaque application pixels remain byte-identical in all on/off pairs.
- Existing inheritance, explicit overrides, blur-off behavior, response
  selectors and aurora motion-policy/cadence checks remain covered.
- Test aurora's refracted landing point and ring reach at rest and under
  motion, including aberration and dense glass. Record the preserved
  attenuation factor separately from changed spatial brightness.
- Run `just test` and `just check` for implementation. Reuse existing smoke
  scripts and evidence conventions; no new capture framework. Record frame
  cost for the changed shaders, without claiming per-tap equivalence.

## Children and documentation

Execute the approved implementation plan sequentially.
The goal remains open through implementation and evidence.

1. **`material-f8b6e9`: behind hook: noise and saturation.** Move the two
   functions and call sites, handle defined colour conversion, update the
   grain/saturation smoke assertions and record face/bevel evidence.
2. **`material-92edaf`: within hook: ring and aurora.** Move interior light
   ahead of surface terms, remove the ring mask, reconcile lookup depth,
   scatter the band with roughness, drop the inset-within-bevel
   validation, refract aurora, replace rest face-zero assertions and
   record motion under the existing drift. Motion policy is
   `material-0e130e`'s.

Each child updates `render-pipeline.md`, `adding-an-optic.md`, relevant
`material-config.md` text and the optics spec status in its landing change.
The behind child marks the older noise/saturation composition order as
superseded; the within child does the same for ring confinement and aurora
placement claims. Search user-facing docs for the same claims, including
"four hooks", finished-glass saturation/noise, ring face confinement and
the `ring-inset + ring-width <= bevel` rule.
Keep old capture evidence historical, with links to its replacement.

This spec does not relabel current pipeline documentation as implemented.
Parameter names, bounds, KDL, Prism, `postprocess.frag`, texture ownership
and the optic recipe's overall shape remain unchanged. No compatibility
layer, shared-texture variant or configurable pass graph is introduced.
