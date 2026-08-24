# Material configuration

`material` definitions name a visual treatment. v1 provides one type,
`glass`. Every parameter below is visually active in the built-in shader.

This surface is provisional. The parameter set was inherited from the
Quickshell prototype and is reviewed before v1 acceptance — names, units, and
which knobs are user-facing may change. See “Config surface review” in
`docs/materials/2026-08-24-v1-parity-design.md`.
`jelly-flex` and `jelly-ripple` consume niri's native move/scrolling and resize
animation residuals and return to their exact rest state when those springs
settle.

Inside the selected workspace, glass composes the workspace background over
the workspace color and retains the backdrop behind any remaining transparency.
Outside it, glass samples the backdrop directly.

```kdl
material "frost" {
    glass {
        thickness 20
        attenuation-color "#dfe8ff"
    }
}

window-rule {
    match app-id="org.example.App"
    material "frost"
}
```

All `glass` parameters are optional. Colors use niri's normal color syntax;
lengths are logical pixels.

| Parameter | Type | Default | Range | Unit |
| --- | --- | --- | --- | --- |
| `ior` | float | 1.5 | 1.0–3.0 | — |
| `thickness` | float | 20 | 0–200 | logical px |
| `attenuation-color` | color | `#dfe8ff` | any color | — |
| `attenuation-distance` | float | 60 | > 0 through 65535 | logical px |
| `chromatic-aberration` | float | 0 | 0–1 | — |
| `distortion` | float | 0 | 0–1 | — |
| `distortion-scale` | float | 0.5 | 0–2 | — |
| `samples` | integer | 4 | 1–8 | — |
| `anisotropic-blur` | float | 0 | 0–1 | — |
| `jelly-flex` | float | 0.004 | 0–0.02 | — |
| `jelly-ripple` | float | 0.06 | 0–0.5 | — |
| `lip` | float | 6 | 0–64 | logical px |
| `shift-x` / `shift-y` | float | 6 | −64–64 | logical px |

`jelly-flex` and `jelly-ripple` use thousandths only in their internal
representation; their configuration values and ranges above are unchanged.

The v1 slab geometry uses fixed 12 px depth and 28 px corner radius. Its
chamfer width is `lip + max(abs(shift-x), abs(shift-y))`; these constants are
not configurable in v1.

## Window rules and validation

`material "name"` is a scalar window-rule field. When several matching rules
set it, the last matching rule wins.

The whole configuration is rejected with these validation errors:

- duplicate material names: `duplicate material: <name>`;
- a definition without exactly one `glass` block: `missing node \`glass\`` or
  `duplicate node \`glass\`, single node expected`;
- a numeric value outside its parameter range: `value must be between <min> and
  <max>` (or, for `attenuation-distance`, `value must be greater than 0 and at
  most 65535`);
- an unknown window-rule reference: `unknown material: <name>`.

Includes participate in the same validation, including duplicate names and
unknown references.

On reload, editing parameters of the same resolved material name updates the
existing material state in place. Changing the resolved name swaps that state.
An invalid configuration is rejected as a whole and niri continues using the
previous configuration.
