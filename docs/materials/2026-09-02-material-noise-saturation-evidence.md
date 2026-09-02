# Glass noise and saturation: verification evidence

**Result:** native implementation passed automated and nested Weston GLES
acceptance on 2026-09-02. Prism contract commit `bcedf4cd` was fast-forwarded
to its requested `main` checkout and passed its full suite.

## Pinned revisions and tools

- Native source: `7c702e5851e508fda4fadf122b4e9ce7c48e68f9`
- Implementation binary SHA-256:
  `460453b002b08f3dc332b7240fb904e5eb9f5c3fb2b5a280a56584e0b2de9698`
- Pre-change binary SHA-256:
  `8edc614ce17217ba55af3f89be0e83437c708bcbe32820df34f60a5b8ad8792d`
- Prism base: `c3c459d7ece853730a1380410539a1ed26924d16`
- Verified Prism commit: `bcedf4cdbfe5b92e5e9d40ea69be4fdc57446254`
- Portable Prism patch SHA-256:
  `6907d63e50887f9b159973308dabfb64222c507d98037c664a94368fd1ff007f`
- Prism bundle SHA-256:
  `d5ec18b700f4166423e8f42c9b382394f98889bb65b9a446eaea0dfe14ff5a9e`
- Weston 15.0.1, OpenGL ES 3.2 Mesa 26.2.1, GLSL ES 3.20, Mesa llvmpipe
  (LLVM 22.1.8), ImageMagick 7.1.2-30, and kitty 0.48.2.

The binary reported `v26.04-167-g7c702e58-modified` because Task 2's task
record was already marked doing. `git diff 7c702e58 -- src` was empty; the
binary hash above pins the executable actually captured.

## Retained artifacts

Set `CAD932_ARTIFACT_DIR` to the retained
`material-cad932-acceptance.XXXXXX` directory outside the product tree. It
contains the nine PNGs and KDL files, `run-acceptance.sh`,
`verify-metrics.sh`, binary/config/capture manifests, Weston and niri logs,
window/output JSON, and PID/socket cleanup records.

| Artifact | SHA-256 |
| --- | --- |
| capture script | `1840bdcb9ceeaddfc617ccdd8419e0a77418f3e698cb98e8105329dbf5b15349` |
| metric script | `d21f00d02eed89297b8b8a51a725080b0492ff4585facf6ac614b0293d6e7278` |
| config validation log | `d7959cb6f108ec75cd0fa02543f7166a518c3e4b60567a5e8f2fcd42341bde3b` |
| metric output | `a5cbe943efa1922073ac68ef22585ce5a4ec3b5fbe775250865c31d542a5bb0d` |
| Weston log | `8f52210e30c52cd3c7a339b3db97dfecc8a238a3ee80b3a85c03c9a042b4a394` |
| niri log | `51194f6d3ce2e3465e4054c90fb9eaed229402e4842d42398570f27d354ee748` |

The wallpaper was the pinned 1280×720 red, green, and blue thirds image with
SHA-256
`fa81fd9535e1a4873b7c36b759d564418b12a341d13f4cdcaf08687d67a5370a`.

## Exact runtime configuration

Every row used this complete material fixture. The matrix below changed only
the named values and client opacity. `blur-off.kdl` additionally placed `off`
inside the global `blur` block.

```kdl
prefer-no-csd

layout {
    gaps 40
    background-color "transparent"
    default-column-width { proportion 0.6; }
    focus-ring { off; }
    border { off; }
    shadow { off; }
}

animations { off; }
hotkey-overlay { skip-at-startup; }
config-notification { disable-failed; }

blur {
    passes 1
    offset 8
    noise 0
    saturation 1
}

material "cad932-probe" {
    glass {
        ior 1.5
        thickness 20
        attenuation-color "#dfe8ff"
        attenuation-distance 60
        chromatic-aberration 0
        distortion 0 scale=0.5
        anisotropic-blur 0
        roughness 0
        backdrop-blur true
        jelly-flex 0
        jelly-ripple 0
        bevel 12
        offset-x 6
        offset-y 6
    }
}

window-rule {
    match app-id="^material-cad932-probe$"
    material "cad932-probe"
    geometry-corner-radius 0
    background-effect {
        blur false
        noise 0
        saturation 1
    }
}
```

| Capture | Binary | backdrop blur | global off | noise | saturation | opacity |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| `default-off-before` | pre-change | false | no | 0 | 1 | 0 |
| `default-off-after` | implementation | false | no | 0 | 1 | 0 |
| `saturation-0` | implementation | true | no | 0 | 0 | 0 |
| `noise-0` | implementation | true | no | 0 | 1 | 0 |
| `noise-008` | implementation | true | no | 0.08 | 1 | 0 |
| `opaque-neutral` | implementation | true | no | 0 | 1 | 1 |
| `opaque-active` | implementation | true | no | 0.08 | 0 | 1 |
| `blur-off` | implementation | true | yes | 0.08 | 0 | 0 |
| `material-opt-out` | implementation | false | no | 0.08 | 0 | 0 |

All nine KDL files validated with their capture binary and contained neither
`xray true` nor a bare `blur true` child. Their SHA-256 values are retained in
`configs.sha256`.

The blank kitty command fixed the app ID, title, cursor state, and content;
only `background_opacity` changed between transparent and opaque clients.
Niri reported exactly one probe with a 704×640 window. Pixel differencing
located the 716×652 material footprint at `+40,+40`, including its six-pixel
frame on each side.

## Results

| Gate | Result |
| --- | ---: |
| pre-change vs implementation default-off AE | 0 |
| saturation red vs green channel AE | 0 |
| saturation red vs blue channel AE | 0 |
| noise ROI RMSE | 1439.59 normalized 0.0219667 |
| neutral noise ROI grayscale standard deviation | 0 |
| active noise ROI grayscale standard deviation | 0.0221145 |
| opaque neutral vs active ROI AE | 0 |
| global-off vs material-opt-out AE | 0 |
| material shader/fallback/panic log hits | 0 |

The capture SHA-256 values are:

| Capture | SHA-256 |
| --- | --- |
| `default-off-before` | `7ca2b4e15848456f8b15ce104a2aaf04d7b99ac46384029a684ff66aef8734d8` |
| `default-off-after` | `7ca2b4e15848456f8b15ce104a2aaf04d7b99ac46384029a684ff66aef8734d8` |
| `saturation-0` | `44b7cd4fb89da4fce7e96482542c8cc14018cc9001fae730b58141715195da08` |
| `noise-0` | `60a912caed1d5ddac7a8b9fe09c30ec6affaa5cdc0ada15f5cd5a821ad19216f` |
| `noise-008` | `b87d7b37e9b150f6c9632bea8eeef5fb3e3919d2548404fb238854460c842b55` |
| `opaque-neutral` | `86dc19da219841dcd4b95068277729621d6aff18afe96d8f87617b28f0f5265a` |
| `opaque-active` | `9f3a71ae8a5aa5ecc4b599dee8990b85cc2123d0c8b8d67c3c15f95687c22ab5` |
| `blur-off` | `7ca2b4e15848456f8b15ce104a2aaf04d7b99ac46384029a684ff66aef8734d8` |
| `material-opt-out` | `7ca2b4e15848456f8b15ce104a2aaf04d7b99ac46384029a684ff66aef8734d8` |

## Automated verification and isolation

Native verification passed 254 niri tests, 51 niri-config tests, one wiki
parse test, and three niri-ipc tests. Both focused regression tests passed,
the material shader validated, changed Rust files passed `rustfmt --check`,
and `tasks check` reported no errors or warnings.

Prism's focused description suite passed 11 tests and its full suite passed
142 tests. Its niri renderer, manifest, and exact-KDL tests were unchanged.

The run used one directly owned Weston headless GL host and never used
`niri --session`. Baseline and implementation niri sessions were separate;
all matched implementation rows shared one compositor process, output,
wallpaper, position, and three-second settle interval. Cleanup proved every
recorded Weston, niri, swaybg, and kitty PID was absent and the private host
socket and lock file no longer existed.

## Prism delivery

The requested Prism checkout fast-forwarded from
`c3c459d7ece853730a1380410539a1ed26924d16` to
`bcedf4cdbfe5b92e5e9d40ea69be4fdc57446254`. Its final status was clean,
the commit was an ancestor of `main`, and the focused 11-test contract suite
and full 142-test suite both passed. The retained bundle and patch remain
inspection artifacts only.
