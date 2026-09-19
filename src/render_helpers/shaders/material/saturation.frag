// Optic: saturation (render-pipeline.md stage 3a, behind). Neutral at 1.
uniform float mat_saturation;

vec3 saturation_behind(vec3 color, vec2 fragCoord) {
    if (mat_saturation == 1.0)
        return color;
    vec3 encoded = linearToSrgb(color);
    const vec3 luma = vec3(0.2126, 0.7152, 0.0722);
    return srgbToLinear(mix(vec3(dot(encoded, luma)), encoded, mat_saturation));
}
