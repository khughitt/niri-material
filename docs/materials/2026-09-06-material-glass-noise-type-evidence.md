# Glass noise type: verification evidence

**Result:** PASS, 2026-09-07, headless Weston 15.0.1. All six captures
completed without renderer errors, fallbacks, or panics in `niri.log`.

## Pinned revisions

- Implementation source commit `098bcdcab2c16a1c92a69e6a07f8fe52cbe8637c`,
  binary `/mnt/ssd3/tmp/material-6e7352-impl-098bcdca/niri`, SHA-256
  `b1785771fa36073c57961e8d528afe11565cc51d8936eda094d8a14c176df9ab`.
- Pre-change source commit `7bb470ca`, binary
  `/mnt/ssd3/tmp/material-6e7352-baseline/niri`, SHA-256
  `d5cf857793023e04b32b4cf7b4a4e33a65223af1fdcb89899ac69609fc70ed70`.

## Metrics

| Metric | Value |
| --- | ---: |
| `zero_determinism_ae` | 0 |
| `untyped_vs_white_ae` | 0 |
| `zero_sd` | 0 |
| `white_sd` | 0.144286 |
| `white_full_sd` | 0.144286 |
| `white_down_sd` | 0.0361264 |
| `white_lowfreq_ratio` | 0.0626907 |
| `white_top_band` | 0.203262 |
| `white_below_median` | 0.498963 |
| `white_beyond_white` | 0 |
| `white_ab_rmse` | 101.042 |
| `fine_sd` | 0.144513 |
| `fine_full_sd` | 0.144513 |
| `fine_down_sd` | 0.0180378 |
| `fine_lowfreq_ratio` | 0.0155795 |
| `fine_top_band` | 0.1336 |
| `fine_below_median` | 0.529763 |
| `fine_beyond_white` | 0.0492125 |
| `fine_ab_rmse` | 101.255 |
| `lightness_sd` | 0.165799 |
| `lightness_full_sd` | 0.165799 |
| `lightness_down_sd` | 0.0207432 |
| `lightness_lowfreq_ratio` | 0.0156526 |
| `lightness_top_band` | 0.158075 |
| `lightness_below_median` | 0.463375 |
| `lightness_beyond_white` | 0.114388 |
| `lightness_ab_rmse` | 54.7217 |

The two amount-zero sessions were byte-identical, and omission matched
explicit `type="white"` byte for byte. Each type raised the ROI standard
deviation above the zero capture. Fine's low-frequency variance ratio was
0.0155795 against white's 0.0626907; its top-band share was 0.1336 against
0.203262, and its below-median share was 0.529763 against 0.498963.
Lightness's Oklab a/b RMSE was 54.7217 Q16 (0.000835 normalized), below
white's 101.042 (0.001542 normalized) and fine's 101.255. It was also below
the one-8-bit-code Q16 bound of 257. Fine's recorded share beyond white's
bound was 0.0492125.

The implementation's untyped capture also had AE 0 against the pre-change
warm-mid `gnt-probe` capture, directly confirming default-white compatibility
across the pinned revisions.

The executable KDL uses quoted string values (`type="white"`, `"fine"`, and
`"lightness"`), as required by the parser. The plan and specification examples
were corrected from bare identifiers to match that grammar.

## Reading the figures

The variance ratios match the expected approximately 0.0144 versus 0.0625
closely and show that fine removes most of white's low-frequency energy. The
band shares show the expected thinner extremes, with small differences from
the estimates caused by 8-bit capture quantization and threshold clipping;
fine's 4.9% beyond-white share is expected. Lightness reduces measured chroma
movement substantially within the one-code tolerance; it does not prove exact
preservation. Gamut clipping affected 0.0002375 of the lightness ROI (19 of
80,000 pixels), and conversion quantization also keeps the RMSE above zero.

Artifacts are retained at
`/mnt/ssd3/tmp/material-6e7352-smoke-20260907-1`. The capture run command
was:

```bash
IMPL=/mnt/ssd3/tmp/material-6e7352-impl-098bcdca/niri \
OUT=/mnt/ssd3/tmp/material-6e7352-smoke-20260907-1 \
docs/materials/scripts/glass-noise-type-smoke.sh
```

The later one-code assertion and clipping limitation were checked against
those unchanged artifacts with:

```bash
one_code=$(magick xc: -format '%[fx:quantumrange/255]' info:)
awk -v rmse=54.7217 -v bound="$one_code" 'BEGIN { exit !(rmse < bound) }'
magick /mnt/ssd3/tmp/material-6e7352-smoke-20260907-1/lightness-roi.png \
  -fx 'min(r,min(g,b)) <= 0 || max(r,max(g,b)) >= 1 ? 1 : 0' \
  -format '%[fx:mean]' info:
```

## Desktop acceptance (pending)

Build and install a package containing the merged implementation before
restarting the graphical niri session. The shader is embedded in the binary;
a configuration reload cannot update an already running compositor. Check
`niri msg version` after logging back in to identify the running compositor,
rather than relying only on `niri --version` (the executable on disk).
The package recipe's source commit and build-version pin must name the new
revision when preparing that package.

Use a temporary, manually maintained test material copied from the unfocused
terminal material, assigned to one test terminal. Avoid editing Prism's
generated material file: regeneration can overwrite the experiment.

1. Keep the terminal unfocused, on the same monitor, at the same position
   over a wallpaper with broad, moderately saturated mid-tones. Keep its
   transparency and every glass parameter except the noise type fixed.
2. Compare `noise 0.02 type="white"`, `noise 0.02 type="fine"`, and
   `noise 0.02 type="lightness"`, one at a time. Saved configuration changes
   reload live; there is no restart between types. Repeat at the amount
   normally used on the desktop if different.
3. White should retain the familiar look. Fine should look less clumpy.
   Compare fine and lightness repeatedly: does lightness preserve colored
   areas more convincingly, and is that difference useful at normal grain
   strength? Judge ordinary viewing distance as well as close inspection.
4. Check dark and bright areas for objectionable clipping, and move or
   refocus the window to check for unexpected flicker or rendering errors.
   Stronger noise can help diagnose the difference, but does not replace
   acceptance at the normal amount.
5. Record the running revision, monitor/scale, tested amount, and verdict
   here, then remove the temporary test configuration. If lightness is
   indistinguishable from fine at normal strength, the design calls for
   dropping it before Prism exposes the selector.
