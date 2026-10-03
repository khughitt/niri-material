// Optic: reflection (render-pipeline.md stage 6). Neutral at amount 0.
uniform float mat_reflection;

// What a pane edge reflects is the room around it; the nearest stand-in is
// the scene just beyond the silhouette, sampled outward along the
// across-bevel direction bent by the perturbation, one thickness further.
// Untinted: reflection happens at the surface, before light enters the
// glass (spec §3.2). Zero on the face, where the direction is undefined.
vec3 reflection_specular(vec3 specular, Surface s) {
    if (mat_reflection <= 0.0)
        return specular;
    float wr = mat_reflection * smoothstep(0.0, 0.1, s.across);
    if (wr <= 0.0)
        return specular;
    vec2 bent = s.acrossDir + s.perturbed.xy - s.structural.xy;
    float lb = length(bent);
    vec2 dir = lb > 0.0001 ? bent / lb : s.acrossDir;
    float reach = -s.outerDist + mat_thickness;
    vec3 scene = srgbToLinear(sampleBackground(s.v + dir * reach / mat_area_size));
    return specular + s.fresnel * wr * scene;
}
