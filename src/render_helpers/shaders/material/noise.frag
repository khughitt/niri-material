// Optic: noise. Up to four layers (design 2026-10-06-noise-layers-design.md),
// one per vec4 component: amount, type (0 white, 1 fine, 2 lightness), site
// (0 glass, 1 backdrop, 2 film; design 2026-10-05-noise-placement-design.md)
// and scale (grain cell in physical pixels; 1 is per-pixel grain).
// noise_behind applies the glass layers (render-pipeline.md stage 3b),
// noise_source the backdrop layers in the effect-program grain pass,
// noise_post the film layers (stage 9); each in slot order. Neutral at
// amount 0. Uses hash12, fineGrain, srgbToLinear, linearToSrgb,
// linearToOklab and oklabToLinear from common.frag.
uniform vec4 mat_noise;
uniform vec4 mat_noise_type;
uniform vec4 mat_noise_site;
uniform vec4 mat_noise_scale;

// Per-slot seed offsets; slot 0 is the single-node original. Glass and film
// layers use their material slot, backdrop layers their place in the
// backdrop list (the effect program receives only those).
const vec2 NOISE_SEED_0 = vec2(47.0, 113.0);
const vec2 NOISE_SEED_1 = vec2(1301.0, 2659.0);
const vec2 NOISE_SEED_2 = vec2(3709.0, 977.0);
const vec2 NOISE_SEED_3 = vec2(2381.0, 3917.0);

// One layer's signed grain before its amount. White is hash12 - 0.5; fine
// and lightness use fineGrain's high-pass. At scale 1 and below, the
// original per-pixel hash. Above it, Hermite-interpolated lattice values
// divided by the interpolation's own deviation sqrt(w^T C w), so the
// grain's deviation is the same at every scale and every position in a
// cell. White corners are independent (C = I). Fine corners share hashes:
// adjacent ones correlate at -1/6 and diagonal ones at -7/36, which the
// norm subtracts as twice those (0.33333333, 0.38888889).
float noiseValue(vec2 fragCoord, float type, float scale, vec2 offset) {
    if (scale <= 1.0)
        return type < 0.5 ? hash12(fragCoord + offset) - 0.5 : fineGrain(fragCoord + offset);
    vec2 p = fragCoord / scale;
    vec2 base = floor(p) + offset;
    vec2 u = fract(p);
    u = u * u * (3.0 - 2.0 * u);
    float w00 = (1.0 - u.x) * (1.0 - u.y);
    float w10 = u.x * (1.0 - u.y);
    float w01 = (1.0 - u.x) * u.y;
    float w11 = u.x * u.y;
    float norm = w00 * w00 + w10 * w10 + w01 * w01 + w11 * w11;
    float g00;
    float g10;
    float g01;
    float g11;
    if (type < 0.5) {
        g00 = hash12(base) - 0.5;
        g10 = hash12(base + vec2(1.0, 0.0)) - 0.5;
        g01 = hash12(base + vec2(0.0, 1.0)) - 0.5;
        g11 = hash12(base + vec2(1.0, 1.0)) - 0.5;
    } else {
        // The 4x4 block of hashes around the cell: block (x, y) is the
        // lattice point base + (x - 1, y - 1), so the corners are block
        // cells 5, 6, 9 and 10, and each corner's 3x3 neighbourhood lies in
        // the block. A corner's fine value is (9 h - its 3x3 sum) / 8,
        // scaled as fineGrain scales.
        float h[16];
        for (int y = 0; y < 4; y++)
            for (int x = 0; x < 4; x++)
                h[y * 4 + x] = hash12(base + vec2(float(x) - 1.0, float(y) - 1.0));
        float k = 0.94280904 / 8.0;
        g00 = (9.0 * h[5] - (h[0] + h[1] + h[2] + h[4] + h[5] + h[6] + h[8] + h[9] + h[10])) * k;
        g10 = (9.0 * h[6] - (h[1] + h[2] + h[3] + h[5] + h[6] + h[7] + h[9] + h[10] + h[11])) * k;
        g01 = (9.0 * h[9] - (h[4] + h[5] + h[6] + h[8] + h[9] + h[10] + h[12] + h[13] + h[14])) * k;
        g11 = (9.0 * h[10] - (h[5] + h[6] + h[7] + h[9] + h[10] + h[11] + h[13] + h[14] + h[15])) * k;
        norm -= 0.33333333 * (w00 * w10 + w00 * w01 + w10 * w11 + w01 * w11)
              + 0.38888889 * (w00 * w11 + w10 * w01);
    }
    return (w00 * g00 + w10 * g10 + w01 * g01 + w11 * g11) / sqrt(norm);
}

// Lightness grain on an encoded colour: Oklab L of the clamped colour moves
// by `grain`; the result is clamped and re-encoded.
vec3 noiseLightness(vec3 encoded, float grain) {
    vec3 lab = linearToOklab(srgbToLinear(clamp(encoded, 0.0, 1.0)));
    lab.x += grain;
    return linearToSrgb(clamp(oklabToLinear(lab), 0.0, 1.0));
}

// A layer's grain on an encoded colour: white and fine add in encoding
// (signed, unclamped); lightness goes through noiseLightness.
vec3 noiseApply(vec3 encoded, float grain, float type) {
    if (type < 1.5)
        return encoded + vec3(grain);
    return noiseLightness(encoded, grain);
}

bool noiseAt(float site) {
    return (mat_noise.x > 0.0 && mat_noise_site.x == site)
        || (mat_noise.y > 0.0 && mat_noise_site.y == site)
        || (mat_noise.z > 0.0 && mat_noise_site.z == site)
        || (mat_noise.w > 0.0 && mat_noise_site.w == site);
}

// One glass layer on the running value: encoded unless isLinear, in which
// case it holds a lightness layer's clamped linear result, re-encoded here
// only because another glass layer follows.
void noiseBehindLayer(inout vec3 v, inout bool isLinear, vec2 fragCoord,
                      float amount, float type, float site, float scale, vec2 offset) {
    if (amount <= 0.0 || site != 0.0)
        return;
    if (isLinear) {
        v = linearToSrgb(v);
        isLinear = false;
    }
    float grain = noiseValue(fragCoord, type, scale, offset) * amount;
    if (type < 1.5) {
        v = v + vec3(grain);
        return;
    }
    // Retain lightness grain's gamut clamp, keeping its linear result.
    vec3 lab = linearToOklab(srgbToLinear(clamp(v, 0.0, 1.0)));
    lab.x += grain;
    v = clamp(oklabToLinear(lab), 0.0, 1.0);
    isLinear = true;
}

// Glass: the averaged linear backdrop, encoded once for the glass layers,
// returned in linear light. One active layer is the pre-layer arithmetic.
vec3 noise_behind(vec3 color, vec2 fragCoord) {
    if (!noiseAt(0.0))
        return color;
    vec3 v = linearToSrgb(color);
    bool isLinear = false;
    noiseBehindLayer(v, isLinear, fragCoord, mat_noise.x, mat_noise_type.x, mat_noise_site.x, mat_noise_scale.x, NOISE_SEED_0);
    noiseBehindLayer(v, isLinear, fragCoord, mat_noise.y, mat_noise_type.y, mat_noise_site.y, mat_noise_scale.y, NOISE_SEED_1);
    noiseBehindLayer(v, isLinear, fragCoord, mat_noise.z, mat_noise_type.z, mat_noise_site.z, mat_noise_scale.z, NOISE_SEED_2);
    noiseBehindLayer(v, isLinear, fragCoord, mat_noise.w, mat_noise_type.w, mat_noise_site.w, mat_noise_scale.w, NOISE_SEED_3);
    return isLinear ? v : srgbToLinear(v);
}

vec3 noisePostLayer(vec3 encoded, vec2 fragCoord, float amount, float type, float site,
                    float scale, vec2 offset) {
    if (amount <= 0.0 || site != 2.0)
        return encoded;
    return noiseApply(encoded, noiseValue(fragCoord, type, scale, offset) * amount, type);
}

// Film: the encoded glass after ring, aurora, glint and sweeps, before the
// coverage multiply.
vec3 noise_post(vec3 encoded, vec2 fragCoord) {
    encoded = noisePostLayer(encoded, fragCoord, mat_noise.x, mat_noise_type.x, mat_noise_site.x, mat_noise_scale.x, NOISE_SEED_0);
    encoded = noisePostLayer(encoded, fragCoord, mat_noise.y, mat_noise_type.y, mat_noise_site.y, mat_noise_scale.y, NOISE_SEED_1);
    encoded = noisePostLayer(encoded, fragCoord, mat_noise.z, mat_noise_type.z, mat_noise_site.z, mat_noise_scale.z, NOISE_SEED_2);
    encoded = noisePostLayer(encoded, fragCoord, mat_noise.w, mat_noise_type.w, mat_noise_site.w, mat_noise_scale.w, NOISE_SEED_3);
    return encoded;
}

vec3 noiseSourceLayer(vec3 straight, vec2 fragCoord, float amount, float type, float scale,
                      vec2 offset) {
    if (amount <= 0.0)
        return straight;
    return noiseApply(straight, noiseValue(fragCoord, type, scale, offset) * amount, type);
}

// Backdrop: one premultiplied texel of the effect buffer, in the effect
// program. The slots hold the agreed backdrop layers in backdrop-list order,
// not a material's slots, and mat_noise_site is not consulted. A transparent
// texel is left alone; the grain lands on the straight colour and is clamped
// once, because the result is stored in an 8-bit premultiplied texture.
vec4 noise_source(vec4 texel, vec2 fragCoord) {
    if (texel.a <= 0.0
        || (mat_noise.x <= 0.0 && mat_noise.y <= 0.0 && mat_noise.z <= 0.0 && mat_noise.w <= 0.0))
        return texel;
    vec3 straight = texel.rgb / texel.a;
    straight = noiseSourceLayer(straight, fragCoord, mat_noise.x, mat_noise_type.x, mat_noise_scale.x, NOISE_SEED_0);
    straight = noiseSourceLayer(straight, fragCoord, mat_noise.y, mat_noise_type.y, mat_noise_scale.y, NOISE_SEED_1);
    straight = noiseSourceLayer(straight, fragCoord, mat_noise.z, mat_noise_type.z, mat_noise_scale.z, NOISE_SEED_2);
    straight = noiseSourceLayer(straight, fragCoord, mat_noise.w, mat_noise_type.w, mat_noise_scale.w, NOISE_SEED_3);
    return vec4(clamp(straight, 0.0, 1.0) * texel.a, texel.a);
}
