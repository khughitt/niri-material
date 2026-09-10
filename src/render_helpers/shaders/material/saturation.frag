// Optic: saturation (render-pipeline.md stage 9). Neutral at 1.
uniform float mat_saturation;

vec3 saturation_post(vec3 color, vec2 fragCoord) {
    if (mat_saturation == 1.0)
        return color;
    const vec3 luma = vec3(0.2126, 0.7152, 0.0722);
    return mix(vec3(dot(color, luma)), color, mat_saturation);
}
