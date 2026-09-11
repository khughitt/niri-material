// Optic: aurora (render-pipeline.md stage 6). Neutral at amount 0.
// Uses snoise, snoiseJelly, and mat_jelly_seed from the prelude.
uniform float mat_aurora;
uniform float mat_aurora_phase;
uniform vec3 mat_aurora_color_a;
uniform vec3 mat_aurora_color_b;

// One noise unit is 250 px at this scale.
const float AURORA_SCALE = 0.004;
// The lookup point traces a circle in noise space over one lap of the
// phase, so the loop closes without a seam. The radius sets how far the
// field travels per lap: 2.0 is about 12.6 noise units per 600 s lap,
// about 5 px per second.
const float AURORA_LOOP_RADIUS = 2.0;

vec3 aurora_emissive(vec2 p, vec3 n, vec3 att, float innerDist) {
    if (mat_aurora <= 0.0)
        return vec3(0.0);
    vec3 q = vec3(p * AURORA_SCALE, 0.0) + mat_jelly_seed
           + vec3(cos(mat_aurora_phase), 0.0, sin(mat_aurora_phase)) * AURORA_LOOP_RADIUS;
    // Two octaves for the field, one coarser octave for its brightness.
    float field = 0.5 + 0.5 * snoiseJelly(q);
    float coarse = 0.5 + 0.5 * snoise(q * 0.5);
    vec3 color = mix(mat_aurora_color_a, mat_aurora_color_b, field);
    // att ^ 0.2 is the ring's factor: the field sits inside the glass
    // rather than on it.
    return mat_aurora * 0.35 * color * (0.5 + 0.5 * coarse) * pow(att, vec3(0.2));
}
