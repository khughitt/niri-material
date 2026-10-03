// Optic: iridescence (render-pipeline.md stage 6). Neutral at amount 0.
uniform float mat_iridescence;

// A thin-film hue from the view angle: the glint runs through the cosine
// palette two and a half times from face-on to edge-on, so the chamfer
// carries a rainbow band and the flat face sits at the palette's start.
// Runs before the signal accent mix, so an accent still tints the result.
vec3 iridescence_specular(vec3 specular, Surface s) {
    if (mat_iridescence <= 0.0)
        return specular;
    const float TAU = 6.28318530718;
    float hue = fract(2.5 * (1.0 - s.cosine));
    vec3 palette = 0.5 + 0.5 * cos(TAU * (hue + vec3(0.0, 1.0 / 3.0, 2.0 / 3.0)));
    return mix(specular, specular * palette * 2.0, mat_iridescence);
}
