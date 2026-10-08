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

// Uniform cubic B-spline weights of the lattice points i - 1 to i + 2 at
// position t in the cell.
vec4 noiseBspline(float t) {
    float t2 = t * t;
    float t3 = t2 * t;
    float s = 1.0 - t;
    return vec4(s * s * s, 3.0 * t3 - 6.0 * t2 + 4.0, -3.0 * t3 + 3.0 * t2 + 3.0 * t + 1.0, t3) / 6.0;
}

// One layer's signed grain before its amount. White is hash12 - 0.5; fine
// and lightness use fineGrain's high-pass. At scale 1 and below, the
// original per-pixel hash. Above it, lattice values reconstructed with the
// cubic B-spline over the 4x4 points around the pixel, divided by the
// reconstruction's own deviation sqrt(w^T C w), so the grain's deviation is
// the same at every scale and every position in a cell. The weights are
// separable, so w^T C w = sum over offsets d of C(d) x(|dx|) y(|dy|), where
// x(k) sums bx_j bx_(j+k). White values are independent: x0 y0. Fine values
// share hashes and correlate up to two points apart (-1/6, -7/36, 1/24,
// 1/36, 1/72 at (1,0), (1,1), (2,0), (2,1), (2,2)); each norm coefficient is
// that correlation times the number of offsets sharing it. Hermite corners
// went flat at the lattice points and read as a grid of squares at scale 8
// (owner's look, 2026-10-07).
float noiseValue(vec2 fragCoord, float type, float scale, vec2 offset) {
    if (scale <= 1.0)
        return type < 0.5 ? hash12(fragCoord + offset) - 0.5 : fineGrain(fragCoord + offset);
    vec2 p = fragCoord / scale;
    vec2 base = floor(p) + offset;
    vec2 t = fract(p);
    vec4 bx = noiseBspline(t.x);
    vec4 by = noiseBspline(t.y);
    float x0 = dot(bx, bx);
    float y0 = dot(by, by);
    float g[16];
    float norm;
    if (type < 0.5) {
        // Value (x, y) is the lattice point base + (x - 1, y - 1).
        for (int y = 0; y < 4; y++)
            for (int x = 0; x < 4; x++)
                g[y * 4 + x] = hash12(base + vec2(float(x) - 1.0, float(y) - 1.0)) - 0.5;
        norm = x0 * y0;
    } else {
        // The 6x6 block of hashes around the cell: block (x, y) is the
        // lattice point base + (x - 2, y - 2), so value (x, y) is block
        // (x + 1, y + 1) and its 3x3 neighbourhood lies in the block. A
        // value is (9 h - its 3x3 sum) / 8, scaled as fineGrain scales; the
        // 3x3 sums go through row sums of three.
        float h[36];
        for (int y = 0; y < 6; y++)
            for (int x = 0; x < 6; x++)
                h[y * 6 + x] = hash12(base + vec2(float(x) - 2.0, float(y) - 2.0));
        float r[24];
        for (int y = 0; y < 6; y++)
            for (int x = 0; x < 4; x++)
                r[y * 4 + x] = h[y * 6 + x] + h[y * 6 + x + 1] + h[y * 6 + x + 2];
        float k = 0.94280904 / 8.0;
        for (int y = 0; y < 4; y++)
            for (int x = 0; x < 4; x++)
                g[y * 4 + x] = (9.0 * h[(y + 1) * 6 + x + 1]
                                - (r[y * 4 + x] + r[(y + 1) * 4 + x] + r[(y + 2) * 4 + x])) * k;
        float x1 = bx.x * bx.y + bx.y * bx.z + bx.z * bx.w;
        float y1 = by.x * by.y + by.y * by.z + by.z * by.w;
        float x2 = bx.x * bx.z + bx.y * bx.w;
        float y2 = by.x * by.z + by.y * by.w;
        norm = x0 * y0 - 0.33333333 * (x1 * y0 + x0 * y1) - 0.77777778 * x1 * y1
             + 0.08333333 * (x2 * y0 + x0 * y2) + 0.11111111 * (x2 * y1 + x1 * y2)
             + 0.05555556 * x2 * y2;
    }
    vec4 rows = vec4(dot(bx, vec4(g[0], g[1], g[2], g[3])),
                     dot(bx, vec4(g[4], g[5], g[6], g[7])),
                     dot(bx, vec4(g[8], g[9], g[10], g[11])),
                     dot(bx, vec4(g[12], g[13], g[14], g[15])));
    return dot(by, rows) / sqrt(norm);
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
// returned in linear light. A lone slot-0 layer at scale 1 runs the
// pre-layer body verbatim: the layered path computes the same values, but
// the driver compiles its lightness arithmetic differently, which moved one
// pixel by one code against the 4a8b2072 baseline.
vec3 noise_behind(vec3 color, vec2 fragCoord) {
    if (!noiseAt(0.0))
        return color;
    if (mat_noise_site.x == 0.0 && mat_noise_scale.x <= 1.0
        && mat_noise.y <= 0.0 && mat_noise.z <= 0.0 && mat_noise.w <= 0.0) {
        vec3 encoded = linearToSrgb(color);
        vec2 noiseSeed = fragCoord + NOISE_SEED_0;
        if (mat_noise_type.x < 0.5)
            return srgbToLinear(encoded + (hash12(noiseSeed) - 0.5) * mat_noise.x);
        float grain = fineGrain(noiseSeed) * mat_noise.x;
        if (mat_noise_type.x < 1.5)
            return srgbToLinear(encoded + vec3(grain));
        vec3 lab = linearToOklab(srgbToLinear(clamp(encoded, 0.0, 1.0)));
        lab.x += grain;
        return clamp(oklabToLinear(lab), 0.0, 1.0);
    }
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
