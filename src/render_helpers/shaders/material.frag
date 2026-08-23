precision highp float;

varying vec2 niri_v_coords;
uniform vec2 niri_size;
uniform float niri_scale;
uniform float niri_alpha;

uniform sampler2D niri_tex_win;
uniform sampler2D niri_tex_bg;
uniform sampler2D niri_tex_backdrop;
uniform vec4 mat_win_rect;
uniform vec4 mat_geo_rect;
uniform vec4 mat_slab_rect;
uniform vec2 mat_area_size;
uniform float mat_chamfer;
uniform vec4 mat_bg_rect;
uniform vec4 mat_backdrop_rect;
uniform vec4 mat_ws_rect;
uniform vec4 mat_ws_color;
uniform vec4 mat_backdrop_color;
uniform float mat_ior;
uniform float mat_thickness;
uniform vec4 mat_attenuation_color;
uniform float mat_attenuation_distance;
uniform float mat_chromatic_aberration;
uniform float mat_distortion;
uniform float mat_distortion_scale;
uniform float mat_samples;
uniform float mat_anisotropic_blur;
uniform float mat_jelly_ripple;

bool inRect(vec2 v, vec4 rect) {
    return all(greaterThanEqual(v, rect.xy))
        && all(lessThan(v, rect.xy + rect.zw));
}

// Composes the per-target background buffer over the workspace color inside
// the workspace rect, and the backdrop buffer over the backdrop color outside it.
// `v` is element UV. Returns sRGB-encoded rgb; layers are opaque after solid
// colors compose behind them.
vec3 sampleBackground(vec2 v) {
    vec4 c;
    if (inRect(v, mat_ws_rect)) {
        vec2 uv = mat_bg_rect.xy + v * mat_bg_rect.zw;
        c = texture2D(niri_tex_bg, clamp(uv, 0.0, 1.0));
        c = c + mat_ws_color * (1.0 - c.a);
    } else {
        vec2 uv = mat_backdrop_rect.xy + v * mat_backdrop_rect.zw;
        c = texture2D(niri_tex_backdrop, clamp(uv, 0.0, 1.0));
        c = c + mat_backdrop_color * (1.0 - c.a);
    }
    return c.rgb;
}

// The renderer works on sRGB-encoded values; glass optics are computed in
// linear light locally and re-encoded on output (G10 pipeline work stays
// deferred).
vec3 srgbToLinear(vec3 c) {
    vec3 low = c / 12.92;
    vec3 high = pow((c + 0.055) / 1.055, vec3(2.4));
    return mix(high, low, vec3(lessThanEqual(c, vec3(0.04045))));
}

vec3 linearToSrgb(vec3 c) {
    vec3 low = c * 12.92;
    vec3 high = 1.055 * pow(c, vec3(1.0 / 2.4)) - 0.055;
    return mix(high, low, vec3(lessThanEqual(c, vec3(0.0031308))));
}

// Slab constants, matching the legacy slab mesh (depth and corner radius
// in logical px) so the implementations stay visually comparable.
const float SLAB_DEPTH = 12.0;
const float SLAB_RADIUS = 28.0;

float sdRoundedBox(vec2 p, vec2 b, float r) {
    vec2 q = abs(p) - b + vec2(r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2(0.0))) - r;
}

// Outward gradient of sdRoundedBox — the silhouette's edge direction.
vec2 sdRoundedBoxGrad(vec2 p, vec2 b, float r) {
    vec2 q = abs(p) - b + vec2(r);
    vec2 g;
    if (q.x > 0.0 && q.y > 0.0)
        g = q / length(q);
    else if (q.x > q.y)
        g = vec2(1.0, 0.0);
    else
        g = vec2(0.0, 1.0);
    return g * vec2(p.x >= 0.0 ? 1.0 : -1.0, p.y >= 0.0 ? 1.0 : -1.0);
}

// The fragment-faked slab at an element-local logical-px position:
// coverage in [0, 1] and the surface normal (y-down, +z toward the
// viewer). The outer silhouette is the slab back and stays fixed; the
// chamfer's inner edge is the front face and trails the jelly shear
// (later slice-2 work) — the quad-form equivalent of the legacy anchored
// vertex shear. Chamfer normals slope at 45 degrees like the mesh ring.
void slabSurface(vec2 p, out float coverage, out vec3 normal) {
    vec2 slab_min = mat_slab_rect.xy * mat_area_size;
    vec2 slab_size = mat_slab_rect.zw * mat_area_size;
    vec2 half_ext = slab_size * 0.5;
    vec2 center = slab_min + half_ext;

    float r = min(SLAB_RADIUS, min(half_ext.x, half_ext.y));
    // Tiny-slab guard: for windows too small to carry a chamfer the upper
    // bound collapses to 0 (never below — reversed clamp bounds are
    // undefined in GLSL) and the whole slab renders as front face.
    float max_chamfer = max(min(half_ext.x, half_ext.y) - 1.0, 0.0);
    float chamfer = clamp(mat_chamfer, 0.0, max_chamfer);

    float d = sdRoundedBox(p - center, half_ext, r);
    float aa = 1.0 / niri_scale;
    coverage = 1.0 - smoothstep(-aa, 0.0, d);

    // Jelly shear placeholders; the jelly task replaces these locals with
    // the mat_jelly_move / mat_jelly_resize uniforms.
    vec2 jelly_move = vec2(0.0);
    vec2 jelly_resize = vec2(0.0);

    vec2 inner_half = (half_ext - vec2(chamfer))
        * (vec2(1.0) + jelly_resize / (2.0 * half_ext));
    vec2 inner_center = center + jelly_move;
    float ri = max(r - chamfer, 1.0);
    float di = sdRoundedBox(p - inner_center, inner_half, ri);

    if (chamfer > 0.0 && di >= 0.0) {
        vec2 g = sdRoundedBoxGrad(p - inner_center, inner_half, ri);
        float bevel = min(chamfer, SLAB_DEPTH);
        float slope = length(vec2(bevel, chamfer));
        normal = normalize(vec3(g * (bevel / slope), chamfer / slope));
    } else {
        normal = vec3(0.0, 0.0, 1.0);
    }
}

void main() {
    vec2 v = niri_v_coords;
    vec2 p = v * mat_area_size;

    // Window sample: zero outside the texture footprint. Corner clipping
    // already happened when the window elements rendered into the
    // offscreen, so the texture is final window content.
    vec4 win = vec4(0.0);
    if (inRect(v, mat_geo_rect)) {
        vec2 wv = (v - mat_geo_rect.xy) / mat_geo_rect.zw;
        win = texture2D(niri_tex_win, mat_win_rect.xy + wv * mat_win_rect.zw);
    }

    float coverage;
    vec3 surfaceNormal;
    slabSurface(p, coverage, surfaceNormal);

    vec4 glassed = vec4(0.0);
    if (coverage > 0.0) {
        vec3 sampled = srgbToLinear(sampleBackground(v));

        // Beer-Lambert over the view-lengthened slab path: the
        // orthographic incident ray is (0, 0, -1); the structural normal's
        // z is its cosine, so the chamfer tints more strongly than the
        // face. The 0.25 floor bounds near-edge-on facets at four times
        // the configured thickness.
        float surfaceCosine = clamp(surfaceNormal.z, 0.0, 1.0);
        float opticalDistance = mat_thickness / max(surfaceCosine, 0.25);
        vec3 att = pow(clamp(mat_attenuation_color.rgb, vec3(0.001), vec3(1.0)),
                       vec3(opticalDistance / mat_attenuation_distance));
        vec3 transmitted = sampled * att;

        glassed = vec4(linearToSrgb(transmitted), 1.0) * coverage;
    }

    // §2 compositing contract: opaque window pixels pass through
    // untouched; the slab contributes only through window transparency
    // and outside the window. Premultiplied throughout; window-rule
    // opacity arrives once, as niri_alpha.
    vec4 color = win + (1.0 - win.a) * glassed;
    gl_FragColor = color * niri_alpha;
}
