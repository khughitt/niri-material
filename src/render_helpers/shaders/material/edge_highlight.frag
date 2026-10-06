// Optic: edge-highlight (render-pipeline.md stage 6). Neutral at amount 0.
uniform float mat_edge_highlight;
uniform float mat_edge_highlight_alpha;

// D(x) / D(1) for GGX (Trowbridge-Reitz): 1 where the normal meets H.
float ggxPeakRatio(float x, float alpha) {
    float a2 = alpha * alpha;
    float t = x * x * (a2 - 1.0) + 1.0;
    return a2 * a2 / (t * t);
}

// A key-light lobe where the bevel's normal bisects the view and the light
// (spec §3.3): the light sits at 45 degrees' elevation in the signal light's
// direction, so H is 22.5 degrees off vertical. `tilt` fades it in from a
// flat surface, which keeps a rounded face join dark at every roughness.
// Not multiplied by F: it stands for a bright source. bevel.rs mirrors it.
vec3 edge_highlight_specular(vec3 specular, Surface s) {
    if (mat_edge_highlight <= 0.0)
        return specular;
    vec3 l = normalize(vec3(normalize(mat_sig_light.xy), 1.0));
    vec3 h = normalize(l + vec3(0.0, 0.0, 1.0));
    float tilt = smoothstep(0.0, 1.0, (1.0 - s.structural.z) / (1.0 - h.z));
    float lobe = ggxPeakRatio(max(dot(s.perturbed, h), 0.0), mat_edge_highlight_alpha);
    return specular + vec3(mat_edge_highlight * tilt * lobe);
}
