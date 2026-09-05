# Glass noise and saturation parameters: verification evidence

**Result:** PASS, 2026-09-05, headless Weston 15.0.1 (`weston --version` →
`weston 15.0.1`). The nested niri binaries run with the `winit` backend
against that Weston socket (`niri::backend::winit: bound legacy EGL to
wl_display` in `niri.log`); no GLES renderer/vendor/version string appears in
`niri.log` because niri's default log filter is
`niri=debug,smithay::backend::renderer::gles=error` (`src/main.rs`), which
suppresses the `info!`-level "GL Renderer" / "GL Version" / "GL Vendor" lines
that smithay's GLES backend emits at initialization. No renderer errors,
fallbacks, or panics appear in the log for any of the six captures.

## Pinned revisions

- Implementation source commit `ff2b922d` (HEAD at run time was `e94f9e42`,
  Task 3's docs-only commit on top of it), binary
  `/mnt/ssd3/tmp/material-1293e8-impl-target/debug/niri`, SHA-256
  `9f2b2bab223e279028dda2be489ab4db295b665147513d8a6f30acf94ead5c9a`.
- Pre-change source commit `7185ee51`, binary
  `/mnt/ssd3/tmp/material-1293e8-base-target/debug/niri`, SHA-256
  `5c7b1e18165cd65cc738dc0e06dc8855e1b936e4e80d6a2c33cef8c7e5b673c1`.

Both confirmed with `sha256sum` before use; both matched their expected
hashes exactly, so neither binary was rebuilt. Version strings recorded by
the script:

```
$ cat base.version
niri 26.04 (7185ee51)
$ cat impl.version
niri 26.04 (ff2b922d-modified)
```

`just check` and `just test`, run by Task 5 on commit `acf3fe73` (HEAD, the
finished piece including this task's own smoke test):

- `just check` (format, clippy, tooling tests, `tasks check`): exit 0.
- `just test` (full workspace suite, `cargo test`): exit 0. Final summary
  line: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0
  filtered out; finished in 0.06s` (niri_ipc doc-tests, the last binary run).
  Every binary in the run passed: `niri` lib 303, `niri` bin 0, `niri_config`
  lib 68, `niri_config` `wiki-parses` 1, `niri_ipc` lib 4, doc-tests `niri` 0,
  `niri_config` 0, `niri_ipc` 1 — 377 passed, 0 failed overall.

## Procedure

`docs/materials/scripts/glass-noise-saturation-smoke.sh` with the six
captures it defines; the runtime KDL fixture is the one the script writes,
with `blur { noise 0.08; saturation 1.5 }` as the deliberately non-neutral
global block.

A first run (`$OUT` = `/mnt/ssd3/tmp/material-1293e8-smoke.SEJ6MJ`) FAILed on
`sat0_rg_ae` (got `26352.9`, expected `0`) because the ROI crop step wrote a
PNG that ImageMagick auto-encoded as single-channel Grayscale (the
`saturation 0` capture is genuinely R=G=B), which made the script's later
`-channel G/B -separate` step read back a phantom all-zero image instead of
real channel data; the fix — writing the ROI crop with
`-define png:color-type=2` to force a true RGB encoding regardless of
content — was applied to both the script and the plan's Step 2 block, and
this document records the clean re-run below.

## Metrics

```
$ IMPL=/mnt/ssd3/tmp/material-1293e8-impl-target/debug/niri \
  BASE=/mnt/ssd3/tmp/material-1293e8-base-target/debug/niri \
  OUT=/mnt/ssd3/tmp/material-1293e8-smoke.fdX0DY \
  bash docs/materials/scripts/glass-noise-saturation-smoke.sh
...
omitted_identity_ae=0
written_neutral_vs_omitted_ae=0
sat0_rg_ae=0
sat0_rb_ae=0
noise_roi_rmse=8054.6
neutral_sd=0
noise05_sd=0.119614
bluroff_rg_ae=0
bluroff_vs_omitted_ae=141205
PASS: artifacts in /mnt/ssd3/tmp/material-1293e8-smoke.fdX0DY
```

All eight assertions held:

| Assertion | Value | Meaning |
| --- | --- | --- |
| `omitted_identity_ae` = 0 | 0 | omitted values are byte-identical to the pre-change binary |
| `written_neutral_vs_omitted_ae` = 0 | 0 | explicit `noise 0; saturation 1` overrides the non-neutral globals and renders with backdrop-blur off |
| `sat0_rg_ae`, `sat0_rb_ae` = 0 | 0, 0 | written `saturation 0` renders grayscale with backdrop-blur off |
| `noise_roi_rmse` > 0 | 8054.6 | written noise changes the ROI |
| `noise05_sd` > `neutral_sd` | 0.119614 > 0 | and raises its variance |
| `bluroff_rg_ae` = 0 | 0 | written values survive `blur { off }` |
| `bluroff_vs_omitted_ae` > 0 | 141205 | (same) |
| log gate | pass | no material error, fallback or panic in `niri.log` |

## Retained artifacts

`$OUT` = `/mnt/ssd3/tmp/material-1293e8-smoke.fdX0DY`, containing
`binaries.sha256`, `base.version`, `impl.version`, `niri.log`, `run.log`,
`metrics.txt`, `color-bars.png`, the six captures' `.kdl`/`.png`/`-roi.png`
files, the derived `sat0-{R,G,B}.png` and `bluroff-{R,G,B}.png` channel
images, and `SHA256SUMS`:

```
6dde04f50f7812a0d0b72e6ce9bf198384fb90134f57cb5eeffe1c7796182436  bluroff-B.png
6dde04f50f7812a0d0b72e6ce9bf198384fb90134f57cb5eeffe1c7796182436  bluroff-G.png
6dde04f50f7812a0d0b72e6ce9bf198384fb90134f57cb5eeffe1c7796182436  bluroff-R.png
fb6ae1f62093fff92010926d3d64e00dd6b4f010e08419b872689c211165ffdf  color-bars.png
e860144c6467e2a9b2a13a03911178c36742afa43d1a95e4e20ef7c2fe37871f  omitted-after.png
7771a425f748c9acbdd8f187ce98dcd0ae91ce7f459316466b4cd34b75ea20d7  omitted-after-roi.png
e860144c6467e2a9b2a13a03911178c36742afa43d1a95e4e20ef7c2fe37871f  omitted-before.png
a9bf4c7a806e5a344256c1e46d3a7da2f8dd7f213b6ce512f977f887e4a0edab  omitted-before-roi.png
a92d691fe67c821558263d02a87ee8ff0363182c9fe90eab57b9c0bf8278cc05  sat0-B.png
a92d691fe67c821558263d02a87ee8ff0363182c9fe90eab57b9c0bf8278cc05  sat0-G.png
a92d691fe67c821558263d02a87ee8ff0363182c9fe90eab57b9c0bf8278cc05  sat0-R.png
90f99d3ad6e0b3589a5135d146d70b2eee61a33f7e64d705e1ecb4fc7b043d2b  written-blur-off.png
76fe6ac2abf9138ba43bcda59c2398d252c692041f820d6f44d6a4d7ae26c971  written-blur-off-roi.png
e860144c6467e2a9b2a13a03911178c36742afa43d1a95e4e20ef7c2fe37871f  written-neutral.png
b0a887bbe84a7893458f3d793c4e0506b8bb40d01599c679c54f27cf6060b21d  written-neutral-roi.png
6d0540756f61151f08e3957706240a0f23adaff62be48b8cf0fff63c618c2738  written-noise-05.png
62cd15be1f5eee9fdf085cb2f42c7c8bad09e2ed9c7f4680307d8c7617a7b4e4  written-noise-05-roi.png
68321c0bf62490d4bd30ee5081fa5b904167488c2988bc6558137be5c1d1a658  written-sat-0.png
a4ba2c8705272a15b074250bfd9b774ebba2e1713e0d5b9cc6ca0b9cff14a732  written-sat-0-roi.png
73b629839320d5e60578a9d00e5fb9d7f3c361dbcace4f18c92727c4cc23d727  omitted-after.kdl
73b629839320d5e60578a9d00e5fb9d7f3c361dbcace4f18c92727c4cc23d727  omitted-before.kdl
5b87db4c7c6cf47a9eeb2e0ca8874fdb7da4f5026244e07e14cbcff01610ed59  written-blur-off.kdl
3afe4b136fb5332d9a38c1d21c4c4528cfdd70f1131d5be01642ab6da8887ef2  written-neutral.kdl
8ce76c172c12a0d161ede3f3da3ebdeebb8d5dfe8dae52ec161c3b577c753550  written-noise-05.kdl
379e9bad582a64fecbc51b0ec3b2f4dc2cb6325fd38d77de3a7bf732dcbe7eb2  written-sat-0.kdl
```

(`omitted-before.png`/`omitted-after.png` and their `.kdl` files share hashes
pairwise, confirming `omitted_identity_ae = 0` at the file level, not just in
the AE metric; `written-neutral.png` shares the `omitted-after.png` hash for
the same reason, confirming `written_neutral_vs_omitted_ae = 0` at the file
level too.)

## Cleanup

```
$ systemctl --user list-units 'gns.*'
  UNIT LOAD ACTIVE SUB DESCRIPTION
0 loaded units listed.
$ ls -d "$XDG_RUNTIME_DIR"/gns.*
ls: cannot access '/run/user/1000/gns.*': No such file or directory
$ git -C /mnt/ssd/Dropbox/niri-material worktree remove /mnt/ssd3/tmp/material-1293e8-base
```

All three checks confirmed clean after the run and after removing the
pre-change worktree.
