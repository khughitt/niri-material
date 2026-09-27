# niri-material

An unofficial fork of [niri](https://github.com/niri-wm/niri), the scrollable-tiling
Wayland compositor, that adds native **materials**: windows rendered as physical
glass slabs that refract, tint, and light up the desktop behind them.

This is a personal fork, not affiliated with or endorsed by the niri project. It
tracks niri releases (currently **v26.04**) and is permanent: the material system
is not headed upstream. Everything else is stock niri, so niri's own
[documentation](https://niri-wm.github.io/niri/) applies unchanged; its README is
[`README.md`](../README.md) at the repository root.

## What the fork adds

- **Glass material.** Refraction through a bevelled slab, attenuation tint,
  chromatic aberration, distortion, anisotropic and backdrop blur, roughness,
  noise, saturation, iridescence, and a drifting aurora field. All of it is
  configured per material in KDL and assigned to windows through window rules.
- **Pane motion.** The slab flexes and ripples with niri's move, scroll, and
  resize springs, and settles back to rest exactly.
- **Ring of light.** A focused material window shows a ring of light under its
  glass face, with a beam running around it on focus. Turn niri's gradient
  `focus-ring` off for material windows so it does not draw a second ring.
- **Window signals.** `niri msg set-window-signal` / `pulse-window-signal` let a
  program mark a window (accent, attention, ping, done, error). The material
  answers with a ring pulse, ripple, flash, or sweep.
- **Workspace IPC.** The workspace's scrolling view position is exposed over IPC.
  These are two carried commits from upstream PR
  [#4147](https://github.com/niri-wm/niri/pull/4147).

```kdl
material "frost" {
    glass {
        thickness 20
        bevel 12
        attenuation-color "#dfe8ff"
        roughness 0.08
    }
}

window-rule {
    match app-id="^kitty$"
    material "frost"
    geometry-corner-radius 16
}
```

Two presets ship in `/usr/share/niri/materials/` (`aurora`, `rainbow`), used with
`include "/usr/share/niri/materials/aurora.kdl"`.

## Documentation

- [Material configuration](../docs/materials/material-config.md): every parameter,
  presets, signal responses, and window-rule validation.
- [Render pipeline](../docs/materials/render-pipeline.md): the pass order the
  material draws in.
- [Window signal IPC](../docs/wiki/IPC.md#window-signals).
- [Upstream divergence](../docs/materials/upstream-divergence.md): every upstream
  file the fork touches, and how it is rebased onto a new niri release.

To tune materials live from a panel, with saved looks and per-wallpaper
adjustments, see [prism](https://github.com/khughitt/prism).

## Install

Arch Linux: `packaging/arch/PKGBUILD` builds the pinned release. It provides and
conflicts with `niri`.

```sh
git clone https://github.com/khughitt/niri-material.git
cd niri-material/packaging/arch
makepkg -si
```

Elsewhere, build it as you would niri (see niri's
[Getting Started](https://niri-wm.github.io/niri/Getting-Started.html)):
`cargo build --release`, and install `resources/materials/*.kdl` to
`/usr/share/niri/materials/` for the presets.

Branches: `materials-26.04` is the default and the one to build.
`patched-26.04` is niri v26.04 plus the carried IPC commits.

## Issues

Report material bugs here, not to niri. If a bug also happens on stock niri,
report it [upstream](https://github.com/niri-wm/niri/issues).

## License

GPL-3.0-or-later, as niri is; see [`LICENSE`](../LICENSE). niri is by Ivan
Molodetskikh and its contributors. The material system and the other
modifications listed in the upstream divergence document are by Keith Hughitt,
2026.
