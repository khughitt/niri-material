#!/usr/bin/env bash
# Noise placement look simulation (material-cf32e5, design §7.1): offline,
# ImageMagick only, no renderer. For one backdrop image it compares grain
# added after a Kawase-like blur chain and a roughness pyramid level (what
# the glass site does) with grain added before them (what the backdrop
# site does), over blur passes p in 0..3 and pyramid levels L in 0..2, for
# white and fine grain. It writes one contact sheet per grain kind and a
# table of the grain's remaining standard deviation per cell, relative to
# the glass site at p = 0, L = 0.
#
# The chain is a model, not the shader: p half-size box downsamples then p
# bilinear (triangle) upsamples stand in for dual Kawase with offset 3; a
# pyramid level L is a 2^L box downsample and a bilinear upsample. The
# backdrop is synthesized unless BACKDROP names an image, so the script
# needs no host file.
#
# Env: OUT (artifact dir, required), AMOUNT (grain amount, default 0.3),
# BACKDROP (optional image), SEED (default 11), SIZE (default 512).
# Requires: ImageMagick 7 (`magick`).
set -eu
OUT=${OUT:?artifact directory}
AMOUNT=${AMOUNT:-0.3}
SEED=${SEED:-11}
SIZE=${SIZE:-512}
mkdir -p "$OUT/cells"
command -v magick >/dev/null || { echo "magick (ImageMagick 7) is required" >&2; exit 1; }

bd="$OUT/backdrop.png"
if [ -n "${BACKDROP:-}" ]; then
    magick "$BACKDROP" -resize "${SIZE}x${SIZE}^" -gravity center -extent "${SIZE}x${SIZE}" -alpha off -depth 16 "$bd"
else
    # Gradients, a textured region and hard edges: the three things grain
    # reads differently against.
    magick -size "${SIZE}x${SIZE}" gradient:'#3a4a6a-#c9b89a' \
        \( -size "$((SIZE / 2))x$((SIZE / 2))" -seed "$SEED" plasma:fractal -blur 0x1 \) \
        -gravity southeast -composite \
        -fill '#f2efe6' -draw "rectangle $((SIZE / 8)),$((SIZE / 8)) $((SIZE * 3 / 8)),$((SIZE / 4))" \
        -fill '#1a1c22' -draw "rectangle $((SIZE / 8)),$((SIZE * 5 / 16)) $((SIZE * 3 / 8)),$((SIZE * 7 / 16))" \
        -fill '#b8463a' -draw "circle $((SIZE * 3 / 4)),$((SIZE / 4)) $((SIZE * 3 / 4)),$((SIZE * 3 / 8))" \
        -alpha off -depth 16 "$bd"
fi

# Uniform noise in [0, 1], one channel, seeded.
noise="$OUT/noise.png"
magick -size "${SIZE}x${SIZE}" -seed "$SEED" xc:black -colorspace gray -type grayscale -fx 'rand()' -alpha off -depth 16 "$noise"
# Fine grain as the prelude's fineGrain: (hash - mean of the 8 neighbours)
# * 0.9428, centred on 0.5 here. mean8 = (9 * mean9 - centre) / 8, so
# fine = 1.125 * (centre - mean9) * 0.9428.
fine="$OUT/noise-fine.png"
k=1.0607
magick "$noise" \( +clone -statistic Mean 3x3 \) \
    -compose Mathematics -define compose:args="0,-$k,$k,0.5" -composite -alpha off -depth 16 "$fine"

# grain(img, noiseimg): img + (noise - 0.5) * AMOUNT, signed, unclamped
# until written.
half=$(magick xc: -format '%[fx:0.5*'"$AMOUNT"']' info:)
grain() {
    magick "$1" "$2" -alpha off -compose Mathematics -define compose:args="0,$AMOUNT,1,-$half" -composite -alpha off -depth 16 "$3"
}
# chain(img, p): p box halvings then p triangle doublings, back to SIZE.
chain() {
    local in=$1 p=$2 out=$3 i args=()
    for ((i = 0; i < p; i++)); do args+=(-filter Box -resize 50%); done
    for ((i = 0; i < p; i++)); do args+=(-filter Triangle -resize 200%); done
    magick "$in" -alpha off "${args[@]}" -resize "${SIZE}x${SIZE}!" -depth 16 "$out"
}
# level(img, L): one 2^L box downsample and a triangle upsample.
level() {
    local in=$1 L=$2 out=$3 pct
    if [ "$L" -eq 0 ]; then cp "$in" "$out"; return; fi
    pct=$(magick xc: -format '%[fx:100/pow(2,'"$L"')]' info:)
    magick "$in" -alpha off -filter Box -resize "$pct%" -filter Triangle -resize "${SIZE}x${SIZE}!" -depth 16 "$out"
}
# sd(a, b): standard deviation of the signed difference a - b.
sd() {
    magick "$1" "$2" -alpha off -compose Mathematics -define compose:args="0,1,-1,0.5" -composite \
        -alpha off -colorspace gray -format '%[fx:standard_deviation]' info:
}

table="$OUT/table.tsv"
printf 'kind\tp\tL\tglass_sd\tbackdrop_sd\tglass_rel\tbackdrop_rel\n' > "$table"
for kind in white fine; do
    case $kind in white) n=$noise ;; fine) n=$fine ;; esac
    grain "$bd" "$n" "$OUT/cells/grained-$kind.png"
    ref=""
    tiles=()
    for p in 0 1 2 3; do
        chain "$bd" "$p" "$OUT/cells/clean-p$p.png"
        chain "$OUT/cells/grained-$kind.png" "$p" "$OUT/cells/pre-$kind-p$p.png"
        for L in 0 1 2; do
            clean="$OUT/cells/clean-p$p-L$L.png"
            level "$OUT/cells/clean-p$p.png" "$L" "$clean"
            g="$OUT/cells/glass-$kind-p$p-L$L.png"
            grain "$clean" "$n" "$g"
            b="$OUT/cells/backdrop-$kind-p$p-L$L.png"
            level "$OUT/cells/pre-$kind-p$p.png" "$L" "$b"
            gs=$(sd "$g" "$clean"); bs=$(sd "$b" "$clean")
            [ -n "$ref" ] || ref=$gs
            gr=$(magick xc: -format "%[fx:$gs/$ref]" info:)
            br=$(magick xc: -format "%[fx:$bs/$ref]" info:)
            printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$kind" "$p" "$L" "$gs" "$bs" "$gr" "$br" >> "$table"
            # 1:1 crops of the textured quadrant, scaled 2x, for the sheet.
            for site in glass backdrop; do
                src=$g; [ $site = backdrop ] && src=$b
                magick "$src" -gravity southeast -crop 160x160+16+16 +repage -scale 200% \
                    -gravity north -background '#202020' -fill '#e0e0e0' -pointsize 14 \
                    -splice 0x18 -annotate +0+2 "$site p=$p L=$L" "$OUT/cells/tile-$kind-$site-p$p-L$L.png"
                tiles+=("$OUT/cells/tile-$kind-$site-p$p-L$L.png")
            done
        done
    done
    magick montage "${tiles[@]}" -tile 6x4 -geometry +4+4 -background '#101010' "$OUT/sheet-$kind.png"
done
column -t -s $'\t' "$table"
echo "sheets: $OUT/sheet-white.png $OUT/sheet-fine.png"
