# Material backdrop blur: nested verification evidence

**Result:** passed 2026-08-29 from source commit
`eb4f1bd65f6573610aeb9af1a64260c07fba63e3`, release binary SHA-256
`bcb08742699b642f8f194d4db96738dcceb12dabd370b62faab9d919999d0bee`.
The separate DRM default-preserves-v1 acceptance run remains outstanding.

## Retained artifacts

Raw captures, exact KDL inputs, the executed capture script, and logs remain
untracked outside the product tree at:

```text
$NIRI_MATERIAL_WORK_ROOT/material-backdrop-blur-eb4f1bd6
```

`NIRI_MATERIAL_WORK_ROOT` is the parent of Cargo's shared target directory in
this checkout. The retained manifests verify with:

```sh
target_dir=$(cargo metadata --no-deps --format-version 1 | jq -r .target_directory)
export NIRI_MATERIAL_WORK_ROOT=${target_dir%/target}
evidence_dir="$NIRI_MATERIAL_WORK_ROOT/material-backdrop-blur-eb4f1bd6"
cd "$evidence_dir"
sha256sum --check captures.sha256
sha256sum --check inputs.sha256
sha256sum --check logs.sha256
sha256sum captures.sha256 inputs.sha256 logs.sha256
```

The three manifest hashes are:

| Manifest | SHA-256 |
| --- | --- |
| `captures.sha256` | `63b01f98f22d42921e03f3c74d7596be39c355c3877e6afa7d2fea2677ec2b37` |
| `inputs.sha256` | `d6f414f0038bb55f80983ffe808b784b6a546bfe7d1bbf5ac49b02b341afabf9` |
| `logs.sha256` | `9ebd54081c9e44a237931a74f955f07ed892c9e982b9333f43211d327ef64020` |

The capture hashes are:

| State | SHA-256 |
| --- | --- |
| 3 passes | `34a8ba5544aa867cb0c38b27c43b5a0ba74dc1b4c893fff3d6ba7f101ccbf761` |
| 6 passes | `513f710c9d8bf55c5266cc351b98716dcbfa9772fd66f0c8e324866c57d2120d` |
| global `blur { off }` | `b97fe1630d60221b81fcf639bfba64858db26d284e52e7a4183a5850fa06eb35` |
| material opt-out | `b97fe1630d60221b81fcf639bfba64858db26d284e52e7a4183a5850fa06eb35` |
| overview | `399f529bf7637e22492d284cea15ad9792df928cf777e3b9768cedf138238245` |

Every capture is 1280×720. The identical global-off and opt-out hashes prove
that the global override and material default select the same sharp state.

## Measurement

The text-free window ROI is `120x100+650+350`. Recompute its grayscale
statistics with:

```sh
for image in enabled-3 enabled-6 global-off opt-out; do
    printf '%s ' "$image"
    magick "$evidence_dir/${image}.png[120x100+650+350]" -colorspace gray \
        -format 'mean=%[fx:mean] sd=%[fx:standard_deviation] min=%[fx:minima] max=%[fx:maxima]\n' \
        info:
done
```

| State | Mean | Standard deviation | Min | Max |
| --- | ---: | ---: | ---: | ---: |
| 3 passes | 0.184021 | 0.00144691 | 0.182722 | 0.186644 |
| 6 passes | 0.183005 | 0 | 0.183005 | 0.183005 |
| global off | 0.188216 | 0.0401255 | 0.147428 | 0.229781 |
| material opt-out | 0.188216 | 0.0401255 | 0.147428 | 0.229781 |

The captures also show the material still rendering in both blurred states and
the overview retaining the blurred treatment. `niri.log` contains no warning,
error, panic, or `material: error` line.

## Isolation and cleanup

The run used the plan's dedicated 1280×720 headless Weston GL host, never
`niri --session` and never the desktop compositor. After capture, the owned
systemd user unit was inactive and its `backdrop-blur` Wayland socket was
absent. Prism and the daily-driver config were not touched.
