# Adding an optic

An optic is one stage of the glass pipeline: its KDL node, resolved values,
uniforms, GLSL, and documentation. This recipe uses `noise` as the worked
example. See `../specs/2026-09-10-material-optics-design.md` for the design.

## 1. Config: `niri-config/src/material/optics/<name>.rs`

Define the knuffel node, a resolved struct with `Default`, `resolve`, and the
parameter metadata. `noise` carries an amount and a `type=` property:

```rust
#[derive(knuffel::Decode, Debug, Clone, Copy, PartialEq)]
pub struct Noise {
    #[knuffel(argument)]
    pub amount: FloatOrInt<0, 1>,
    #[knuffel(property(name = "type"), str)]
    pub kind: Option<NoiseType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ResolvedNoise {
    pub amount: Option<f64>,
    pub kind: NoiseType,
}

pub fn resolve(node: Option<Noise>) -> ResolvedNoise {
    ResolvedNoise {
        amount: node.map(|n| n.amount.0),
        kind: node.and_then(|n| n.kind).unwrap_or_default(),
    }
}

pub fn params() -> Vec<ParamSpec> {
    vec![ParamSpec {
        node: "noise",
        kind: ParamKind::inherit::<FloatOrInt<0, 1>>(),
        write: |v| format!("noise {v}"),
        read: Some(|g| g.noise.amount),
    }]
}
```

`ParamKind::float::<T>(default, unit)` and `ParamKind::inherit::<T>()` take
their range from the node field's `Bounded` type. `read` returns the resolved
scalar. `material_parameter_specs_match_the_parser` writes each scalar bound
and an out-of-range value through `write`; the `node` string is the remaining
hand-maintained fact, so spell it as the table should show it (`"noise type="`).

Add `validate(&self) -> Result<(), String>` only for a real cross-parameter
rule, and call it from `Material::validate`.

Register the optic in `niri-config/src/material/`:

- `optics/mod.rs`: export the module, append its name to `ORDER`, and extend
  `params()` with its specs.
- `mod.rs`: add the optional node to `Glass`, the resolved value to
  `ResolvedGlass` and its default, and call `resolve` from `Material::resolve`.
- `lib.rs`: re-export its public types.

Keep parse, default, and validation-error tests beside the material tests in
`niri-config/src/lib.rs`.

## 2. GLSL: `src/render_helpers/shaders/material/<name>.frag`

Prefix new uniforms with `mat_<name>_`; migrated `saturation` and `noise` keep
their older names. Define one function for each hook the optic uses:

| Hook | Signature | Neutral return |
| --- | --- | --- |
| `normal` | `vec3 <name>_normal(vec3 n, vec2 p)` | `n` |
| `specular` | `vec3 <name>_specular(vec3 specular, vec3 surfaceNormal, float surfaceCosine)` | `specular` |
| `emissive` | `vec3 <name>_emissive(vec2 p, vec3 n, vec3 att, float innerDist)` | `vec3(0.0)` |
| `post` | `vec3 <name>_post(vec3 color, vec2 fragCoord)` | `color` |

The file is concatenated after `prelude.frag`, which provides `snoise`,
`snoiseFractal`, `hash12`, `fineGrain`, the colour conversions,
`sdRoundedBox`, the slab globals, `mat_jelly_seed`, `mat_thickness`, and
`mat_ior`. Put the neutral check first. Neutral is an explicit resolved value;
omission may instead inherit a non-neutral global blur value, as `noise` and
`saturation` do while backdrop blur is effective.

## 3. Renderer: `src/render_helpers/material/optics/<name>.rs`

Implement `Optic` on a marker type:

```rust
pub struct NoiseOptic;

impl Optic for NoiseOptic {
    const NAME: &'static str = "noise";
    const GLSL: &'static str = include_str!("../../shaders/material/noise.frag");
    const UNIFORMS: &'static [(&'static str, UniformType)] = &[
        ("mat_noise", UniformType::_1f),
        ("mat_noise_type", UniformType::_1f),
    ];

    fn values(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Vec<Uniform<'static>> {
        // One Uniform per UNIFORMS entry, in the same order.
    }

    // Override only when values change without a configuration change.
    // fn next_change(glass: &ResolvedGlass, ctx: &OpticFrame<'_>) -> Option<Duration>
}
```

`OpticFrame` supplies the clock, motion policy, animations switch, effective
backdrop-blur gate, global `blur` block, and window seed. Uniform values join
the frame fingerprint, and `next_change` joins the tile redraw deadline.
Test `values` for each input that affects it in the same file.

Finally, export the module and append `OpticEntry::of::<<name>::<Name>Optic>()`
to `OPTICS` at its render position. `ORDER` in niri-config must match; a test
pins them. Add one call per used hook to `shaders/material/main.frag`, in
`OPTICS` order.

## 4. Documentation and proof

Regenerate the parameter table:

```sh
MATERIAL_DOCS_UPDATE=1 python3 tools/tt test-fast -- cargo test -p niri-config material_parameter_table_matches_the_docs
```

Add an `## Optics` subsection to `material-config.md` stating the stage,
effect, explicit neutral, and any omission/inheritance rule. Update the stage
in `render-pipeline.md` and its Prism mapping when one exists.

Run `just test`. Then use a smoke under `docs/materials/scripts/` to prove the
neutral render, the intended effect, and frame cost. Compare decoded pixels
and record the command's non-zero-on-difference exit status in an evidence doc.
