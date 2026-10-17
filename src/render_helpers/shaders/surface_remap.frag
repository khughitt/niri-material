#version 100

//_DEFINES_

#if defined(EXTERNAL)
#extension GL_OES_EGL_image_external : require
#endif

precision highp float;
#if defined(EXTERNAL)
uniform samplerExternalOES tex;
#else
uniform sampler2D tex;
#endif

uniform float alpha;
varying vec2 v_coords;

#if defined(DEBUG_FLAGS)
uniform float tint;
#endif

// The shell remap (docs/specs/2026-10-16-noctalia-glass-cohesion-design.md
// §3). Mirrors RemapUniforms::apply in surface_remap.rs.
uniform vec3 remap_anchor0;
uniform vec3 remap_anchor1;
uniform vec3 remap_anchor2;
uniform vec3 remap_offset0;
uniform vec3 remap_offset1;
uniform vec3 remap_offset2;
uniform vec2 remap_radii;

// From material/common.frag, concatenated after this file.
vec3 srgbToLinear(vec3 c);
vec3 linearToSrgb(vec3 c);
vec3 linearToOklab(vec3 c);
vec3 oklabToLinear(vec3 lab);

// Distance from p to the segment a -> b, and the fraction along it.
vec2 segment(vec3 p, vec3 a, vec3 b) {
    vec3 ab = b - a;
    float len2 = dot(ab, ab);
    float t = len2 > 0.0 ? clamp(dot(p - a, ab) / len2, 0.0, 1.0) : 0.0;
    return vec2(length(p - (a + t * ab)), t);
}

vec4 remap(vec4 color) {
    if (color.a <= 0.0)
        return color;
    vec3 straight = clamp(color.rgb / color.a, 0.0, 1.0);
    vec3 lab = linearToOklab(srgbToLinear(straight));
    vec2 s0 = segment(lab, remap_anchor0, remap_anchor1);
    vec2 s1 = segment(lab, remap_anchor1, remap_anchor2);
    float d;
    vec3 offset;
    if (s0.x <= s1.x) {
        d = s0.x;
        offset = mix(remap_offset0, remap_offset1, s0.y);
    } else {
        d = s1.x;
        offset = mix(remap_offset1, remap_offset2, s1.y);
    }
    float w = 1.0 - smoothstep(remap_radii.x, remap_radii.y, d);
    if (w <= 0.0)
        return color;
    vec3 moved = linearToSrgb(clamp(oklabToLinear(lab + w * offset), 0.0, 1.0));
    return vec4(moved * color.a, color.a);
}

void main() {
    vec4 color = texture2D(tex, v_coords);
#if defined(NO_ALPHA)
    color = vec4(color.rgb, 1.0);
#endif

    color = remap(color);

    // Apply final alpha and tint.
    color = color * alpha;

#if defined(DEBUG_FLAGS)
    if (tint == 1.0)
        color = vec4(0.0, 0.2, 0.0, 0.2) + color * 0.8;
#endif

    gl_FragColor = color;
}
