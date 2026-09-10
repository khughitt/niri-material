// Optic: noise (render-pipeline.md stage 10). Neutral at amount 0.
// Uses hash12, fineGrain, srgbToLinear, linearToSrgb, linearToOklab and
// oklabToLinear from the prelude.
uniform float mat_noise;
uniform float mat_noise_type;

vec3 noise_post(vec3 color, vec2 fragCoord) {
    if (mat_noise <= 0.0)
        return color;
    vec2 noiseSeed = fragCoord + vec2(47.0, 113.0);
    if (mat_noise_type < 0.5)
        return color + (hash12(noiseSeed) - 0.5) * mat_noise;
    float grain = fineGrain(noiseSeed) * mat_noise;
    if (mat_noise_type < 1.5)
        return color + vec3(grain);
    // Clamp before re-encoding: a lightness pushed past the gamut yields
    // negative linear values, and pow() of a negative is undefined.
    vec3 lab = linearToOklab(srgbToLinear(clamp(color, 0.0, 1.0)));
    lab.x += grain;
    return linearToSrgb(clamp(oklabToLinear(lab), 0.0, 1.0));
}
