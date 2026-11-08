---
id: material-68bb16
title: "A material renderer outside niri's render path: a portable core and material behind client surfaces"
status: shelved
priority: 2
created: 2026-10-17T22:29:43Z
updated: 2026-10-18T22:47:40Z
depends: []
tags: [material]
source: dots-a417c7
agent: claude-code/claude-opus-5-5
---

Owner's idea (2026-10-17, from dots-a417c7), in their words: "long-term, do you think it would be possible to create a standalone ~wayland / niri material renderer? wrap existing material; separate material rendering from niri, etc. logic, and have niri-material + material-surface? 'material-surface' could just be a toggle-able surface that can be rendered ~'above' a workspace? kitty could then be displayed along side of it / coordinated?"

Assessment at scoping time (niri-material 8cfd964a, not yet reviewed):
- The material is about 5.4k lines of Rust in src/render_helpers/material, tests included (5,360 at 8cfd964a; corrected at scope 2026-10-18 from "about 4.2k"), plus about 1.1k lines of GLSL in src/render_helpers/shaders/material. The shaders are already separate files. Their inputs are textures of the window, the background and the backdrop (`niri_tex_win`, `niri_tex_bg`, `niri_tex_backdrop`), and the elements are built on smithay's GlesRenderer and niri's TtyRenderer.
- A standalone Wayland client cannot do the glass over windows, because a client cannot sample the screen behind its surface. The archived Quickshell prototype (archive/niri-glass) hit exactly this: it lived on the bottom layer and refracted a copy of the wallpaper, because "a third-party surface cannot sample the screen behind it". A client material-surface over a workspace could refract only the wallpaper, or a lagging screen capture that includes itself.
- Feasible split 1, a portable material core: a crate with the shader sources, uniform layout, parameter and signal model, which niri-material's elements consume. It would also serve a headless renderer for sheets and tests. Cost: an interface boundary inside a fork that already rebases about three times a year. Related: the material library lane material-3aa1f2.
- Feasible split 2, compositor-drawn material behind a client's surface: a layer-rule `material` node (alternative 2 of docs/notes/2026-10-14-noctalia-material-brief.md), or later a small protocol through which a client asks for a material behind its surface, as ext-background-effect does for blur. kitty's dropdown would then carry the glass itself, with no second surface to keep in step. A separate toggled material surface stacked under kitty is possible, but two clients would have to agree on geometry and visibility.
- Lean: split 2 first, when layer-surface material is wanted; split 1 once a second consumer (a headless renderer, another host) exists.

Established at scope (2026-10-18, niri-material 27015551):
- No open task overlaps either split. The closest consumer has gone: for Noctalia cohesion the owner chose Noctalia styling over layer-surface material (2026-10-16 verdict on material-55380c), and the dropdowns take a prism-owned look first (dots-a417c7, option (b)).
- The client-sampling limit is confirmed in the archived prototype (archive/niri-glass README and docs/2026-08-05-glass-panes-design.md: a third-party surface cannot sample the real screen behind it).

## Notes

- 2026-10-18T22:47:18Z (materials-26.04): shelved: Wake when a consumer wants material drawn behind a layer or client surface (for example the dropdowns choosing niri-material glass, dots-a417c7's option (c)), or when a second consumer of a portable material core exists (a headless renderer for sheets and tests, another host). Scoped 2026-10-18 as an idea; unshelve returns it to idea for a fresh scope.
- 2026-10-18T22:47:40Z (materials-26.04): scope: shelved; wake: a consumer wants material behind a layer or client surface, or a second consumer of a portable core exists; body's line count corrected (5,360 Rust lines at 8cfd964a, not ~4.2k) and an Established section added (no overlapping open task; Noctalia cohesion chose styling on 2026-10-16, material-55380c; dropdowns take prism's look first, dots-a417c7); checked at niri-material 27015551
