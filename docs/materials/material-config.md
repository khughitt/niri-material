# Material configuration

`material` definitions name a visual treatment. v1 provides one type,
`glass`. Every parameter below is visually active in the built-in shader.

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
        bevel 12
        offset-x 6
        offset-y 6
        attenuation-color "#dfe8ff"
        roughness 0.08
    }
}

window-rule {
    match app-id="org.example.App"
    material "frost"
    geometry-corner-radius 16
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
| `distortion` `scale=` | float | 0.5 | 0–2 | — |
| `anisotropic-blur` | float | 0 | 0–1 | — |
| `roughness` | float | 0 | 0–1 | — |
| `backdrop-blur` | bool | false | true / false | — |
| `jelly-flex` | float | 0.004 | 0–0.02 | — |
| `jelly-ripple` | float | 0.06 | 0–0.5 | — |
| `bevel` | float | 12 | 0–128 | logical px |
| `offset-x` / `offset-y` | float | 6 | −64–64 | logical px |

`jelly-flex` and `jelly-ripple` use thousandths only in their internal
representation; their configuration values and ranges above are unchanged.

`backdrop-blur` makes the glass refract the blurred backdrop rather than the
sharp one, which is what produces a frosted appearance: blur and refraction
compose into one image instead of being drawn as two. It is a switch, not a
strength — the amount of blur comes from the global `blur` block's `passes` and
`offset`, shared with every other blur consumer. Setting `blur { off }`
disables it along with all other blur, regardless of this parameter.

It is unrelated to `anisotropic-blur`, which smears the refraction itself along
one axis and does not soften the backdrop.

`roughness` progressively softens detail refracted through the glass. It first
uses `backdrop-blur` to choose the sharp or globally blurred source, then
filters that source; setting `blur { off }` disables the blurred-source switch
but does not disable roughness on the sharp source. The native default is `0`,
which preserves the v1 appearance and performs no prefilter allocation or
downsample work. Non-zero roughness lazily builds a damage-aware texture
pyramid for the selected per-target source and reuses it until that source
changes.

The slab uses `thickness` as its depth. Its frame is the window rectangle
inflated by `bevel - max(abs(offset-x), abs(offset-y))`, then translated by
the offsets. Its inner corners follow the window's effective
`geometry-corner-radius`; the outer corners add the drawn chamfer.

## Signal responses

Each material can map a window signal to glass effects with named `response`
blocks. A material with no response blocks gets this built-in `default`:

| Parameter | Values | Default |
| --- | --- | --- |
| `accent` | `ring`, `none` | `ring` |
| `attention` | `rim-orbit`, `ring-pulse`, `none` | `rim-orbit` |
| `ping` | `ripple`, `flash`, `sweep`, `none` | `ripple` |
| `done` | `ripple`, `flash`, `sweep`, `none` | `sweep` |
| `error` | `ripple`, `flash`, `sweep`, `none` | `flash` |
| `ring-inset` | 0–128 logical px | 6 logical px |
| `ring-width` | 0–128 logical px | 2 logical px |

When any `response` block is present, one must be named `default`. Named
responses inherit omitted fields from that block; the `default` block itself
inherits omitted fields from the built-in defaults.

```kdl
material "terminal-glass" {
    glass {
        bevel 16
    }

    response "default" {
        accent "ring"
        attention "rim-orbit"
        ping "ripple"
        done "sweep"
        error "flash"
        ring-inset 6
        ring-width 2
    }

    response "loud" {
        attention "ring-pulse"
    }
}

window-rule {
    match app-id="^kitty$" signal-source="^familiar$" signal-tag="^cats/"
    material "terminal-glass" response="loud"
}
```

`response=` selects a named response on the referenced material; omitting it
selects `default`. `signal-source` and `signal-tag` are regex matches:
`signal-source` matches any source slot (including niri's native `niri`
slot), while `signal-tag` matches the window's folded signal tag. A window
with no signal matches neither.

`ring-inset` is measured inward from the slab's outer edge. Every resolved
response must satisfy `ring-inset + ring-width <= bevel`, otherwise the
configuration is rejected. The ring is visible through the slab's exterior
band and translucent window pixels; an opaque window shows a full ring only
when `bevel >= 2 * max(|offset-x|, |offset-y|) + ring-inset + ring-width`.

## Signal motion and animation

The top-level signal policy defaults to `full`:

```kdl
signal {
    motion "reduced" // full | reduced | off
}

animations {
    material-signal {
        duration-ms 400
        curve "ease-out-cubic"
    }
}
```

`reduced` changes sustained `flash` to `pulse`, `pulse` to `breathe`, and an
impulse `flash` response to `sweep`. `off` makes sustained motion static and
drops impulse effects, while retaining the static accent and level
crossfade. `material-signal` is the baseline signal crossfade; it defaults to
400 ms with `ease-out-cubic` and follows the normal animation configuration,
including `animations { off }`.

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
- an offset wider than the bevel: `offset must not exceed bevel`;
- response blocks without `default`: `material <name>: missing response
  "default"`;
- duplicate response names: `duplicate response: <name>`;
- a ring outside the bevel: `ring-inset + ring-width must not exceed bevel`;
- an unknown window-rule reference: `unknown material: <name>`.
- an unknown response reference: `material <name>: unknown response:
  <response>`.

Includes participate in the same validation, including duplicate names and
unknown references.

On reload, editing parameters of the same resolved material name updates the
existing material state in place. Changing the resolved name swaps that state.
An invalid configuration is rejected as a whole and niri continues using the
previous configuration.
