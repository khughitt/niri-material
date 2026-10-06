// Optic: noise. Three placements selected by mat_noise_site
// (0 glass, 1 backdrop, 2 film; design 2026-10-05-noise-placement-design.md):
// noise_behind at the glass site (render-pipeline.md stage 3b), noise_source
// in the effect-program grain pass at the backdrop site, noise_post at the
// film site (stage 9). Neutral at amount 0. Uses hash12, fineGrain,
// srgbToLinear, linearToSrgb, linearToOklab and oklabToLinear from
// common.frag.
uniform float mat_noise;
uniform float mat_noise_type;
uniform float mat_noise_site;

// Glass: the averaged linear backdrop, encoded for the grain, decoded on
// return. The body is the pre-site original, byte for byte, behind one gate.
vec3 noise_behind(vec3 color, vec2 fragCoord) {
    if (mat_noise <= 0.0 || mat_noise_site != 0.0)
        return color;
    vec3 encoded = linearToSrgb(color);
    vec2 noiseSeed = fragCoord + vec2(47.0, 113.0);
    if (mat_noise_type < 0.5)
        return srgbToLinear(encoded + (hash12(noiseSeed) - 0.5) * mat_noise);
    float grain = fineGrain(noiseSeed) * mat_noise;
    if (mat_noise_type < 1.5)
        return srgbToLinear(encoded + vec3(grain));
    // Retain lightness grain's gamut clamp, returning its linear result.
    vec3 lab = linearToOklab(srgbToLinear(clamp(encoded, 0.0, 1.0)));
    lab.x += grain;
    return clamp(oklabToLinear(lab), 0.0, 1.0);
}

// The signed scalar grain for the white and fine kinds, scaled by amount;
// the lightness kind uses the fine value through noiseLightness.
float noiseGrain(vec2 seed) {
    return (mat_noise_type < 0.5 ? hash12(seed) - 0.5 : fineGrain(seed)) * mat_noise;
}

// Lightness grain on an encoded colour: Oklab L of the clamped colour moves
// by `grain`; the result is clamped and re-encoded.
vec3 noiseLightness(vec3 encoded, float grain) {
    vec3 lab = linearToOklab(srgbToLinear(clamp(encoded, 0.0, 1.0)));
    lab.x += grain;
    return linearToSrgb(clamp(oklabToLinear(lab), 0.0, 1.0));
}

// Grain on an encoded colour, every kind: white and fine add in encoding
// (signed, unclamped); lightness goes through noiseLightness.
vec3 noiseEncoded(vec3 encoded, vec2 seed) {
    float grain = noiseGrain(seed);
    if (mat_noise_type < 1.5)
        return encoded + vec3(grain);
    return noiseLightness(encoded, grain);
}

// Film: the encoded glass after ring, aurora, glint and sweeps, before the
// coverage multiply. Same seed offset as the glass site.
vec3 noise_post(vec3 encoded, vec2 fragCoord) {
    if (mat_noise <= 0.0 || mat_noise_site != 2.0)
        return encoded;
    return noiseEncoded(encoded, fragCoord + vec2(47.0, 113.0));
}

// Backdrop: one premultiplied texel of the effect buffer, in the effect
// program. The amount and type arrive from the agreed backdrop grain, not
// from a material, and mat_noise_site is not consulted. A transparent texel
// is left alone; the grain lands on the straight colour and is clamped,
// because the result is stored in an 8-bit premultiplied texture.
vec4 noise_source(vec4 texel, vec2 fragCoord) {
    if (mat_noise <= 0.0 || texel.a <= 0.0)
        return texel;
    vec3 straight = texel.rgb / texel.a;
    vec3 grained = noiseEncoded(straight, fragCoord + vec2(47.0, 113.0));
    return vec4(clamp(grained, 0.0, 1.0) * texel.a, texel.a);
}
