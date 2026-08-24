# Glass config surface: design

**Status:** drafted 2026-08-24; implementation plan drafted at
`docs/materials/plans/2026-08-24-glass-config-surface.md`. Execution pending,
and gated behind the v1 parity pass.
**Parent design:** `docs/materials/2026-08-22-v1-design.md`
**Gate recorded in:** `docs/materials/2026-08-24-v1-parity-design.md`,
“Config surface review”

## Goal

Settle the `material { glass { ... } }` parameter set before v1 acceptance
makes it a released compatibility surface. The current set was inherited
wholesale from the Quickshell prototype, which was built to test the idea
rather than to propose a long-term API. Four of its parameters describe the
prototype's internals rather than what a user is choosing.

This design changes names, units, and which knobs exist. It does not change
the optics: every shader path other than slab geometry and tap count is
untouched.

It does change default behaviour, in two ways, both corrections.

Making the corner radius follow the window means a default niri window —
`geometry-corner-radius 0` — gets inner radius 0 and outer radius `chamfer`,
where the deleted constants gave 16 and 28. The glass now matches the window
it sits under.

Clamping the chamfer compositor-side changes jelly on windows too small to
carry the configured band. The shader always drew such a band clamped; what
was not clamped was the Rust-side copy feeding the jelly flex limit. On an
8×8 window at the defaults the effective band is 9, not 12, so the flex limit
becomes 2.25 rather than 3 — the amplitude now follows the band actually
drawn. Static appearance there is unchanged, since the shader was already
clamping.

Every other default is preserved, including the `thickness` unification,
which is neutral for the reason given under “Appearance neutrality”.

## Resolved surface

```kdl
material "frost" {
    glass {
        ior 1.5
        thickness 20
        attenuation-color "#dfe8ff"
        attenuation-distance 60
        chromatic-aberration 0
        distortion 0 scale=0.5
        anisotropic-blur 0
        jelly-flex 0.004
        jelly-ripple 0.06
        bevel 12
        offset-x 6
        offset-y 6
    }
}
```

Twelve KDL nodes carrying thirteen configurable values, from fourteen of
each — the counts differ now because `scale` is a property of `distortion`
rather than a node of its own. Removed: `lip`, `shift-x`, `shift-y`,
`samples`, `distortion-scale`. Added: `bevel`, `offset-x`, `offset-y`, and
`distortion`'s `scale` property. The slab's corner radius follows the window's
`geometry-corner-radius` and has no glass-side parameter at all.

| Parameter | Type | Default | Range | Unit |
| --- | --- | --- | --- | --- |
| `ior` | float | 1.5 | 1.0–3.0 | — |
| `thickness` | float | 20 | 0–200 | logical px |
| `attenuation-color` | color | `#dfe8ff` | any color | — |
| `attenuation-distance` | float | 60 | > 0 through 65535 | logical px |
| `chromatic-aberration` | float | 0 | 0–1 | — |
| `distortion` | float | 0 | 0–1 | — |
| `distortion` `scale=` | float | 0.5 | 0–2 | — |
| `anisotropic-blur` | float | 0 | 0–1 | — |
| `jelly-flex` | float | 0.004 | 0–0.02 | — |
| `jelly-ripple` | float | 0.06 | 0–0.5 | — |
| `bevel` | float | 12 | 0–128 | logical px |
| `offset-x` / `offset-y` | float | 6 | −64–64 | logical px |

## Decisions

### `thickness` is one length with one meaning

The shader carried two thicknesses. An optical one — the `thickness`
parameter, driving refraction displacement, the Beer-Lambert path, and the
anisotropic smear width — and a geometric one, the fixed `SLAB_DEPTH` of 12
logical px driving the bevel normal. A parameter named for a length that is
not the length of the thing is a defect the API would carry forever.

`thickness` becomes both. `SLAB_DEPTH` is deleted from the shader and from
`src/render_helpers/material.rs`. The bevel angle becomes a consequence of
`thickness` and `bevel` rather than of a constant.

Rejected: splitting into a unitless `refraction` strength plus a separate
`depth`. It preserves a thin-slab-with-strong-refraction look that nothing
needs, at the cost of a third knob and of `refraction` no longer being a
physical quantity. Also rejected: renaming only and keeping the constant,
which labels the incoherence instead of removing it.

### The visible band is the parameter

`lip` inflated the slab past the window on every side and `shift-x`/`shift-y`
translated it, with the visible bevel band derived as
`lip + max(abs(shift-x), abs(shift-y))`. That derivation encodes real taste —
it guarantees the bevel spans the whole widest band, so the glass hugs the
window instead of reading as a detached flat strip past a gap — but it means
the quantity a user can point at is the one they cannot set.

The relation inverts. `bevel` is the visible band, set directly, and the
uniform inflation is derived:

```
inflate = bevel - max(abs(offset-x), abs(offset-y))
slab    = window rect inflated by `inflate`, translated by (offset-x, offset-y)
chamfer = min(bevel, max(half the smaller slab side - 1, 0))
```

The chamfer is the bevel wherever the bevel fits, which is every window big
enough to carry the band. The clamp is the tiny-slab guard, and it is applied
once, compositor-side, so the value handed to the shader is the band actually
drawn — the jelly flex limit is derived from the same number and would
otherwise disagree with it on small windows.

The invariant is preserved by construction rather than by convention, and no
look is lost: `(lip, shift)` maps to `(lip + max(abs(shift)), shift)` and back
by the identity above, so every configuration expressible today has exactly
one counterpart. Sub-pixel values are the one place the mapping is not
value-identical: the old code snapped `lip` up to the physical grid and then
reused that rounded number as the chamfer, so `lip 5.3` at scale 2 reported a
5.5 band. The inflation is still snapped — element geometry and damage must
land on pixels — but the chamfer is now the `bevel` unrounded, since a band
width has no reason to be pixel-aligned. (Unrounded, not unconditional: the
tiny-slab clamp above still applies.) `bevel`'s range is 0–128 rather than `lip`'s 0–64 precisely
so that it is: the widest band reachable today combines both maxima. The new
domain is a strict superset — it also admits uniform bands wider than 64,
which the old parameterization could not express.

Rejected: keeping `lip`/`shift` and adding an independent `bevel`, which lets
a user set a bevel narrower than the band (detached strip) or wider (bevel
running under the window) — it exposes the knob by discarding the taste.
Also rejected: per-edge extension, which expresses looks the current model
cannot at the cost of four or five knobs and turns the pedestal look into
four edits.

### The corner radius follows the window

`SLAB_RADIUS` was fixed at 28 logical px. It is the slab's *outer* radius; the
inner flat face, which is where the window actually sits, was derived as
`outer - chamfer` = 16. So the constant encoded “a 16 px window radius plus a
12 px bevel” — a guess about the window, made by code that has the window.

The derivation inverts to match the geometry inversion. The inner face takes
the window's radius; the outer follows:

```
inner = the tile's effective geometry-corner-radius, per corner
outer = inner + chamfer
```

`chamfer`, not `bevel`: on a window too small to carry the configured band the
two differ, and the outer ring follows the band that is drawn.

Each set is *fitted* to the box it is drawn on, by the same CSS
corner-overlap rule the window itself uses — one proportional reduction across
all four corners from the tightest adjacent-pair sum, never a per-corner
clamp, which would shrink a large radius whose neighbour is small. This
matters because neither box is the window: with the default offset a 100 px
window yields a 112 px slab and an 88 px inner face. Fitting the inner face
is enough, because the outer ring inherits non-overlap from it — adjacent
outer radii sum to `inner_a + inner_b + 2 * bevel` against an outer edge of
`inner_edge + 2 * bevel`.

There is no glass-side `corner-radius` parameter: an override is a way to
reintroduce the mismatch this removes.

The value fed to the shader is the radius the window is actually rendered
with, which is the rule value through two transforms, both already applied in
`src/layout/tile.rs`. First `scaled_by(1. - expanded_progress)`: the tile
squares the window's corners as it expands into maximize or fullscreen, so the
rule value alone would leave rounded glass around a square window. Then
`fit_to`, the CSS corner-overlap rule — one proportional reduction across all
four corners from adjacent-pair sums, not a per-corner clamp — fitted against
the geometry of the branch doing the rendering, which differs between the
normal and resize paths.

The shader then fits again, because neither box it draws is the window: the
inner face is the window narrowed by twice the offset, and jelly can scale it
further. It fits the inner radius against whichever of the two boxes binds
harder — the inner face, or the slab less one chamfer — which keeps
`outer = inner + chamfer` exactly true. That exactness is load-bearing: the
bevel normal builds its slope from `chamfer` as the horizontal run, so fitting
the outer ring separately would tilt the normal at precisely the corners it
narrowed. Repeated proportional fitting is safe in the only direction that
matters, since no fit ever enlarges a radius.

That radius is therefore a shader input that changes without any `glass`
parameter changing: during an expand animation, and on a reload that edits
only `geometry-corner-radius`. `apply_resolved` bumps the material commit only
when the `glass` block differs, and the damage tracker cannot see a uniform,
so the effective radius joins `InputFingerprint` alongside the background,
backdrop, and jelly inputs. Without that, the element's pixels stay stale
until unrelated damage happens to arrive.

A 16 px window radius reproduces the deleted constants exactly (inner 16,
outer 28). A default niri window has `geometry-corner-radius 0`, so default
glass is square where it meets the window and rounded at the bevel's outer
lip — which is what a chamfer around a square window looks like. Users who
want the prototype's rounded look set `geometry-corner-radius` on the same
window rule that assigns the material; `material-config.md`'s example does so.

Rejected: an explicit `corner-radius` parameter, which leaves the original
mismatch in place by default. Also rejected: inheriting but collapsing the
four corners to one scalar, which is wrong for asymmetric radii and cannot be
fixed later without another surface change.

### `samples` is derived, not configured

`samples` was a GPU-cost dial in an appearance block, and it was inert unless
`anisotropic-blur` or `chromatic-aberration` was nonzero — the reason the
parity capture matrix needs a dependency edge for it. It leaves the config.

The count is computed compositor-side, so the uniform stays and only the
parameter goes:

```
strength = max(anisotropic-blur, chromatic-aberration)
taps     = strength > 0 ? clamp(ceil(8 * strength), 2, 8) : 1
```

`strength = 0` keeps the existing single-tap fast path unchanged. `0.5` gives
4, reproducing the prototype's default at the strength it shipped. `1.0`
gives 8, the old maximum. The shader's bounded loop is unchanged. Computing
this in Rust rather than in GLSL keeps it unit-testable.

This self-regulates: lowering an effect lowers its cost, so a user on weak
hardware has a lever without a separate dial.

Rejected: a compositor-level quality setting, which is a reasonable shape but
opens a new top-level config surface in a design scoped to the glass block.
Also rejected: keeping `samples` and documenting it, which leaves the
inert-until-activated case in the surface.

### `distortion`'s scale rides its amplitude

`distortion-scale` did nothing while `distortion` was 0, so
`distortion-scale 1.5` alone was valid config with no effect. Making it a
property of the node it depends on gives the scale a syntactic owner:

```kdl
distortion 0.5 scale=1.5
```

This matches niri's own idiom for the shape — `spring damping-ratio=1.0
stiffness=800 epsilon=0.0001`, `variable-refresh-rate on-demand=true`.
`ResolvedGlass` keeps `distortion` and `distortion_scale` as separate fields;
only the written form changes, so the uniform path does not move.

This does not make inert state unrepresentable — `distortion 0 scale=1.5` is
still accepted and still does nothing, and the default row in “Resolved
surface” is exactly that case. What it removes is the orphan: a scale can no
longer be written without naming the amplitude it belongs to, so the reader of
a config always sees the two together.

Rejected: a `distortion { amount; scale }` child block, which groups them just
as well but is the heaviest shape in an otherwise flat block. Also rejected:
leaving them as siblings, which is the smell being removed elsewhere here.

## Validation

One cross-parameter rule, which the current surface does not have:

```
max(abs(offset-x), abs(offset-y)) <= bevel
```

Violating it makes `inflate` negative — glass narrower than the window on one
side, which has no meaning. The whole configuration is rejected with
`offset must not exceed bevel`.

`Material::resolve` cannot host this: it is infallible and runs after the
config is already accepted. The check is a `Material::validate` called
immediately after `Material::decode_node` in the `"material"` arm of
`niri-config/src/lib.rs`, emitting through `ctx.emit_error` exactly as the
neighbouring `duplicate material` check does. Unlike `validate_material_refs`,
which must wait until every include has merged because a reference may name a
later definition, this rule is per-definition and needs no deferral — but
includes decode through the same arm, so it must be tested through one.

`offset-x` and `offset-y` carry `shift-x`/`shift-y`'s −64–64 unchanged.
`bevel` is 0–128, not `lip`'s 0–64, so that every band width reachable under
the old parameterization stays reachable — see “The visible band is the
parameter”. No second constraint caps the derived inflation: `bevel 128` with
zero offset is a 128 px uniform band, which the old surface could not express
and which nothing needs to forbid.

## Breakage

This is a clean break with no compatibility layer. `lip`, `shift-x`,
`shift-y`, `samples`, and `distortion-scale` become unknown nodes, which
knuffel already rejects, so an old configuration fails validation with a
pointed error and niri keeps running its previous configuration. That is the
existing fail-early behaviour for any unknown key and needs no new code.

The material system has not shipped in a release, so no deployed
configuration is affected.

## Appearance neutrality

Unifying `thickness` with the slab depth changes no pixel at the defaults, and
this is worth stating precisely because it is easy to assume otherwise.

`SLAB_DEPTH` reaches the rendered image through exactly two expressions:
`min(chamfer, SLAB_DEPTH)` in the shader's bevel normal, and
`0.25 * SLAB_DEPTH.min(chamfer)` in the jelly flex clamp
(`src/layout/tile.rs`). Both are the bevel depth. The default chamfer is
`lip + max(abs(shift))` = 12, which equals the old constant, so both
expressions evaluate to 12 before the change and to `min(12, 20)` = 12 after
it. The 45-degree bevel and the jelly clamp are unchanged **for any window
large enough to carry the band** — see the clamp's effect on small windows
above, which is a separate change and not this one.

Away from the defaults the two disagree in both directions, because the old
depth is `min(bevel, 12)` and the new one is `min(bevel, thickness)`. They
agree only while `bevel <= min(12, thickness)`, or when `thickness` is exactly
12. So `bevel 15` with the default `thickness 20` already differs — 12 before,
15 after — well below the `bevel > 20` threshold an earlier draft of this
document claimed; and `bevel 12` with `thickness 5` differs the other way, 12
before and 5 after. Both directions are the correction: the bevel can now be
as deep as the slab and no deeper, instead of being capped at a constant
unrelated to either.

`SLAB_CORNER_RADIUS` in `src/render_helpers/material.rs` is unused outside its
own definition — the shader carries a separate copy — so deleting the Rust
constant has no effect at all.

## Testing

**Config.** Each new name parses and resolves. `lip`, `shift-x`, `shift-y`,
`samples`, and `distortion-scale` are rejected as unknown nodes.
`max(abs(offset)) > bevel` is rejected with the stated message, both in a root
file and in an include, since both decode through the same arm.
`distortion 0.5 scale=1.5` parses, and `scale` without an amplitude is a parse
error rather than an accepted no-op.

**Geometry.** `material_frame_inflates_by_lip_and_shift` and
`material_frame_shift_slides_the_slab` become bevel/offset equivalents. One
new test pins the appearance-neutrality claim above: the default parameters
produce the same frame rect and chamfer as the pre-change constants. That
claim belongs in a test rather than only in this document.

**Bevel depth.** Both directions of the boundary, since the frame test above
covers only the default: `bevel 15` with `thickness 20` gives 15 where the old
constant gave 12, and `bevel 12` with `thickness 5` gives 5 where it gave 12.
`bevel 12` with `thickness 20` gives 12, unchanged.

**Damage.** Changing the effective corner radius while every `glass` parameter
stays equal advances the material element's commit — the `InputFingerprint`
case for the radius, mirroring the existing background and jelly cases.

**Tap count.** A unit test at strength 0, 0.125, 0.5, and 1.0, asserting 1, 2,
4, and 8 taps.

**Rendered appearance is neither gated nor seen.** `niri-visual-tests` is an
interactive GTK viewer, not a snapshot suite: CI runs `cargo test --all
--exclude niri-visual-tests` and its own job only runs `cargo build --package
niri-visual-tests`, so a case there asserts nothing.

It also cannot show *this*. The viewer builds its `RenderCtx` with
`xray: None`, and `Tile::render_inner` gates the material on
`self.material.is_some() && ctx.xray.is_some()`, so a case added there renders
a plain tile with no glass at all. Displaying a material would mean
constructing an `Xray` with background and backdrop buffers — more than
inspection scaffolding earns. **No visual cases are added**, and no step of
this work looks at a rendered material.

What is covered is the plumbing: that the uniform carries the tile's rendered
radius, in `CornerRadius` order, scaled by `1 - expanded_progress`, and that
changing it advances the element's commit. The shader's quadrant selection and
the rendered radii themselves are checked by nothing and seen by no one in
this repository. Closing that means a headless capture harness in the style of
the parity pass, which is out of scope here; until then it is a known and
stated gap, not an oversight.

## Implementation surface

| File | Change |
| --- | --- |
| `niri-config/src/material.rs` | `Glass` and `ResolvedGlass` fields, defaults, `resolve`, and `Material::validate` for the offset/bevel rule |
| `niri-config/src/lib.rs` | Call `Material::validate` in the `"material"` arm, beside the duplicate-name check; config tests |
| `src/render_helpers/material.rs` | Delete both slab constants; derive the tap count; clamp the chamfer; carry the corner radius in the uniforms and in `InputFingerprint`; geometry, tap-count and damage tests inline |
| `src/render_helpers/shaders/material.frag` | Per-corner `sdRoundedBox` and gradient; `fitRadii`; invert the radius derivation; `thickness` for the bevel depth |
| `src/render_helpers/shaders/mod.rs` | `mat_corner_radius` uniform |
| `src/layout/tile.rs` | Jelly clamp uses `thickness`; feed each branch's fitted radius (`geometry_corner_radius().scaled_by(1. - expanded_progress).fit_to(..)`) |
| `docs/materials/material-config.md` | Parameter table, radius inheritance, the offset rule; drop the provisional caveat |
| `docs/materials/2026-08-22-v1-design.md` | §4 parameter table, which the config module cites as this surface's specification |
| `docs/materials/2026-08-24-v1-parity-design.md` | Mark “Config surface review” resolved with the implementing commit |

The corner-radius vec4 uses niri's `CornerRadius` order — top-left, top-right,
bottom-right, bottom-left — reusing the existing `From<CornerRadius> for
[f32; 4]`.

## Sequencing

This lands **after** the v1 parity pass, as a single change.

The parity pass compares native against the frozen prototype, whose surface is
the current one. Landing this first would break the row mapping: `samples`
would have no native counterpart, and the `lip`, `shift-x`, and `shift-y` rows
would need remapping to `bevel` and `offset`. Running parity first keeps its
evidence a clean statement about the optics port and leaves a calibrated
instrument undisturbed.

Splitting this into two slices was also rejected. The four changes are
entangled — `bevel` is what makes `thickness`-as-depth meaningful, and the
radius derivation needs `bevel` for the outer edge — so a split touches
`slabSurface` twice.

Neither this design nor the parity pass satisfies the physical DRM gate or v1
acceptance.
