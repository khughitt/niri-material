# Material optics: byte-identical migration evidence

**Design:** `../specs/2026-09-10-material-optics-design.md` §5.
**Run:** 2026-09-10, headless Weston (`weston --backend=headless --renderer=gl`), nested niri.

## Binaries

| | Commit | Cargo target directory | Version | sha256 |
| --- | --- | --- | --- | --- |
| before | `22d10019f5988fa28d81a2215a492c7e5c98be87` | `/mnt/ssd3/niri-material/target-before` | `niri 26.04 (22d10019)` | `cfa6268b4eb31ebba46dd19474b0bda73f121c14eb6221cb088834484d0b9585` |
| after | `380e0610a4c7395b8d67b6f526eab1efef86b31c` | `/mnt/ssd3/niri-material/target` | `niri 26.04 (380e0610-modified)` | `67b63e9551920db20caec84329e1a3fe20246feacab929e644aa41ece05df533` |

The after build's `-modified` suffix records the required task claim in
`tasks/material-1930b9.md`; that was the only dirty path at build time. The
rendering source was commit `380e0610a4c7395b8d67b6f526eab1efef86b31c`.

## Captures

Every PNG the two smokes produced was decoded and compared before against
after with `magick compare -metric AE`. PNG container bytes can differ because
ImageMagick writes metadata; the verdict below concerns decoded pixels.

| Smoke | Captures | Zero differing pixels |
| --- | ---: | ---: |
| `glass-noise-saturation-smoke` | 19 | 19 |
| `glass-noise-type-smoke` | 26 | 26 |

All 45 comparisons reported `AE=0 (0)`. The comparison script reported
matching capture counts, `status=0`, and exited 0. Both scripts' own assertions
also held on both binaries: all four smoke runs exited 0.

## Hashes

The complete capture manifests, binary hashes, versions, smoke stdout, and
comparison stdout are archived under `/mnt/ssd3/niri-material/evidence-optics`.
The four capture manifests are:

- `before-glass-noise-saturation-smoke/SHA256SUMS`
- `after-glass-noise-saturation-smoke/SHA256SUMS`
- `before-glass-noise-type-smoke/SHA256SUMS`
- `after-glass-noise-type-smoke/SHA256SUMS`

The smoke transcripts are the corresponding sibling `.log` files, and
`compare.log` contains the decoded-pixel verdict for every capture.
