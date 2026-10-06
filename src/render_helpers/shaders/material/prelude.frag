// Material prelude: core uniforms, shared globals, slab and refraction taps.
// common.frag precedes it; shaders/mod.rs emits precision first, then each
// optic in OPTICS order and main.frag. The compile path prepends #version.

const float PI = 3.14159265358979;
const float HALF_PI = 1.57079632679490;

varying vec2 niri_v_coords;
uniform vec2 niri_size;
uniform float niri_scale;
uniform float niri_alpha;

uniform sampler2D niri_tex_win;
uniform sampler2D niri_tex_bg;
uniform sampler2D niri_tex_bg_high;
uniform sampler2D niri_tex_backdrop;
uniform sampler2D niri_tex_backdrop_high;
uniform vec4 mat_win_rect;
uniform vec4 mat_geo_rect;
uniform vec4 mat_slab_rect;
uniform vec2 mat_area_size;
uniform float mat_chamfer;
uniform vec4 mat_corner_radius;
uniform vec2 mat_jelly_move;
uniform vec2 mat_jelly_resize;
uniform float mat_jelly_activity;
uniform float mat_jelly_time;
uniform vec3 mat_jelly_seed;
uniform vec4 mat_bg_rect;
uniform vec4 mat_backdrop_rect;
uniform vec4 mat_ws_rect;
uniform vec4 mat_ws_color;
uniform vec4 mat_backdrop_color;
uniform float mat_bg_prefilter_mix;
uniform float mat_backdrop_prefilter_mix;
uniform float mat_ior;
uniform float mat_scatter;
uniform float mat_thickness;
uniform vec4 mat_attenuation_color;
uniform float mat_attenuation_distance;
uniform float mat_chromatic_aberration;
uniform float mat_distortion;
uniform float mat_distortion_scale;
uniform float mat_samples;
uniform float mat_anisotropic_blur;
uniform float mat_jelly_ripple;
uniform vec4 mat_sig_accent;
uniform float mat_sig_level;
uniform float mat_sig_breath;
uniform vec3 mat_sig_light;
uniform vec4 mat_sig_impulse_env;
uniform vec4 mat_sig_impulse_prog;
uniform vec3 mat_sig_impulse_rgb0;
uniform vec3 mat_sig_impulse_rgb1;
uniform vec3 mat_sig_impulse_rgb2;
uniform vec3 mat_sig_impulse_rgb3;
uniform ivec4 mat_sig_impulse_resp;
uniform ivec3 mat_sig_response;
uniform vec4 mat_sig_ring;
uniform vec4 mat_sig_focus;
uniform vec3 mat_sig_ring_color;
uniform float mat_sig_ring_accent;
uniform float mat_light_ior;

// Slab geometry published by slabSurface for the focus filament: the outer
// silhouette and the face (the chamfer's inner edge, jelly included), which
// the ring beam runs inside.
vec2 g_center;
vec2 g_half;
vec4 g_outer_r;
vec2 g_face_center;
vec2 g_face_half;
vec4 g_face_r;

bool inRect(vec2 v, vec4 rect) {
    return all(greaterThanEqual(v, rect.xy))
        && all(lessThan(v, rect.xy + rect.zw));
}

vec4 samplePrefilter(
    sampler2D low_tex,
    sampler2D high_tex,
    float amount,
    vec2 uv
) {
    vec4 low = texture2D(low_tex, clamp(uv, 0.0, 1.0));
    if (amount == 0.0)
        return low;
    return mix(low, texture2D(high_tex, clamp(uv, 0.0, 1.0)), amount);
}

vec4 sampleBackdrop(vec2 v) {
    vec2 uv = mat_backdrop_rect.xy + v * mat_backdrop_rect.zw;
    vec4 c = samplePrefilter(
        niri_tex_backdrop,
        niri_tex_backdrop_high,
        mat_backdrop_prefilter_mix,
        uv
    );
    return c + mat_backdrop_color * (1.0 - c.a);
}

// Composes the per-target background buffer over the workspace color, then
// over the backdrop when that result remains translucent. Outside the selected
// workspace, samples the backdrop directly. `v` is element UV.
vec3 sampleBackground(vec2 v) {
    vec4 c;
    if (inRect(v, mat_ws_rect)) {
        vec2 uv = mat_bg_rect.xy + v * mat_bg_rect.zw;
        c = samplePrefilter(
            niri_tex_bg,
            niri_tex_bg_high,
            mat_bg_prefilter_mix,
            uv
        );
        c = c + mat_ws_color * (1.0 - c.a);
        if (c.a < 1.0)
            c = c + sampleBackdrop(v) * (1.0 - c.a);
    } else {
        c = sampleBackdrop(v);
    }
    return c.rgb;
}

// ---- simplex noise (Ashima Arts / Stefan Gustavson, MIT) ----
// Source: https://github.com/stegu/webgl-noise/blob/master/src/noise3D.glsl
// License: https://raw.githubusercontent.com/stegu/webgl-noise/master/LICENSE
// Copyright (C) 2011 by Ashima Arts (Simplex noise)
// Copyright (C) 2011-2016 by Stefan Gustavson (Classic noise and others)
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.
vec4 permute(vec4 x) { return mod(((x * 34.0) + 1.0) * x, 289.0); }
vec4 taylorInvSqrt(vec4 r) { return 1.79284291400159 - 0.85373472095314 * r; }
float snoise(vec3 v)
{
    const vec2 C = vec2(1.0 / 6.0, 1.0 / 3.0);
    const vec4 D = vec4(0.0, 0.5, 1.0, 2.0);
    vec3 i  = floor(v + dot(v, C.yyy));
    vec3 x0 = v - i + dot(i, C.xxx);
    vec3 g = step(x0.yzx, x0.xyz);
    vec3 l = 1.0 - g;
    vec3 i1 = min(g.xyz, l.zxy);
    vec3 i2 = max(g.xyz, l.zxy);
    vec3 x1 = x0 - i1 + 1.0 * C.xxx;
    vec3 x2 = x0 - i2 + 2.0 * C.xxx;
    vec3 x3 = x0 - 1.0 + 3.0 * C.xxx;
    i = mod(i, 289.0);
    vec4 p = permute(permute(permute(
                 i.z + vec4(0.0, i1.z, i2.z, 1.0))
               + i.y + vec4(0.0, i1.y, i2.y, 1.0))
               + i.x + vec4(0.0, i1.x, i2.x, 1.0));
    float n_ = 1.0 / 7.0;
    vec3 ns = n_ * D.wyz - D.xzx;
    vec4 j = p - 49.0 * floor(p * ns.z * ns.z);
    vec4 x_ = floor(j * ns.z);
    vec4 y_ = floor(j - 7.0 * x_);
    vec4 x = x_ * ns.x + ns.yyyy;
    vec4 y = y_ * ns.x + ns.yyyy;
    vec4 h = 1.0 - abs(x) - abs(y);
    vec4 b0 = vec4(x.xy, y.xy);
    vec4 b1 = vec4(x.zw, y.zw);
    vec4 s0 = floor(b0) * 2.0 + 1.0;
    vec4 s1 = floor(b1) * 2.0 + 1.0;
    vec4 sh = -step(h, vec4(0.0));
    vec4 a0 = b0.xzyw + s0.xzyw * sh.xxyy;
    vec4 a1 = b1.xzyw + s1.xzyw * sh.zzww;
    vec3 p0 = vec3(a0.xy, h.x);
    vec3 p1 = vec3(a0.zw, h.y);
    vec3 p2 = vec3(a1.xy, h.z);
    vec3 p3 = vec3(a1.zw, h.w);
    vec4 norm = taylorInvSqrt(vec4(dot(p0, p0), dot(p1, p1),
                                   dot(p2, p2), dot(p3, p3)));
    p0 *= norm.x; p1 *= norm.y; p2 *= norm.z; p3 *= norm.w;
    vec4 m = max(0.6 - vec4(dot(x0, x0), dot(x1, x1),
                            dot(x2, x2), dot(x3, x3)), 0.0);
    m = m * m;
    return 42.0 * dot(m * m, vec4(dot(p0, x0), dot(p1, x1),
                                  dot(p2, x2), dot(p3, x3)));
}
// drei's 4-octave fractal (same weights)
float snoiseFractal(vec3 m)
{
    return 0.5333333 * snoise(m) + 0.2666667 * snoise(2.0 * m)
         + 0.1333333 * snoise(4.0 * m) + 0.0666667 * snoise(8.0 * m);
}

// Two octaves are sufficient for the short-lived jelly ripple and keep its
// active-frame cost bounded across full-column panes.
float snoiseJelly(vec3 m)
{
    return 0.6666667 * snoise(m) + 0.3333333 * snoise(2.0 * m);
}

// Selects this quadrant's radius from a CornerRadius-ordered vec4
// (top-left, top-right, bottom-right, bottom-left).
float cornerRadius(vec2 p, vec4 r) {
    float top = p.x < 0.0 ? r.x : r.y;
    float bottom = p.x < 0.0 ? r.w : r.z;
    return p.y < 0.0 ? top : bottom;
}

// CSS corner-overlap fitting, mirroring CornerRadius::fit_to: one
// proportional reduction across all four corners, taken from the tightest
// adjacent-pair sum. Radii are (top-left, top-right, bottom-right,
// bottom-left); `half_ext` is half the box being fitted to.
vec4 fitRadii(vec4 r, vec2 half_ext) {
    vec2 edge = 2.0 * half_ext;
    float k = min(min(edge.x / max(r.x + r.y, 1e-6),
                      edge.x / max(r.w + r.z, 1e-6)),
                  min(edge.y / max(r.x + r.w, 1e-6),
                      edge.y / max(r.y + r.z, 1e-6)));
    return r * min(1.0, k);
}

float sdRoundedBox(vec2 p, vec2 b, vec4 radii) {
    float r = cornerRadius(p, radii);
    vec2 q = abs(p) - b + vec2(r);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2(0.0))) - r;
}

// Outward gradient of sdRoundedBox — the silhouette's edge direction.
vec2 sdRoundedBoxGrad(vec2 p, vec2 b, vec4 radii) {
    float r = cornerRadius(p, radii);
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
void slabSurface(vec2 p, out float coverage, out vec3 normal, out float outerDist,
                 out float innerDist, out float chamferOut) {
    vec2 slab_min = mat_slab_rect.xy * mat_area_size;
    vec2 slab_size = mat_slab_rect.zw * mat_area_size;
    vec2 half_ext = slab_size * 0.5;
    vec2 center = slab_min + half_ext;

    // Tiny-slab guard: for windows too small to carry a chamfer the upper
    // bound collapses to 0 (never below — reversed clamp bounds are
    // undefined in GLSL) and the whole slab renders as front face.
    float max_chamfer = max(min(half_ext.x, half_ext.y) - 1.0, 0.0);
    float chamfer = clamp(mat_chamfer, 0.0, max_chamfer);

    vec2 inner_half = (half_ext - vec2(chamfer))
        * (vec2(1.0) + mat_jelly_resize / max(2.0 * half_ext, vec2(1.0)));
    vec2 inner_center = center + mat_jelly_move;

    // `mat_corner_radius` is fitted to the *window*, but neither SDF box is
    // the window: the inner face is the window narrowed on both axes by
    // 2 * max(|offset-x|, |offset-y|), then translated by the offset
    // (defaults, 100 px window: slab 112, inner face 88). Refit to the box
    // actually being drawn. The reduction is proportional across all four
    // corners, never per-corner clamping, which would shrink a large radius
    // whose neighbour is small.
    // Fit against whichever box binds harder. Jelly scales `inner_half`
    // while `half_ext` stays fixed, so a positive resize can leave the inner
    // face wider than (outer - 2 * chamfer): at the 112/88 geometry a
    // full-flex resize reaches an inner edge near 90.4, and 90.4 + 2 * 12
    // overflows the 112 px outer edge. Constraining the *inner* radius by
    // both keeps `outer = inner + chamfer` exactly true, which the bevel
    // normal below depends on — it builds its slope from `chamfer` as the
    // horizontal run, so a separately fitted outer ring would tilt the
    // normal at exactly the corners it narrowed.
    vec2 radius_half = min(inner_half, half_ext - vec2(chamfer));
    vec4 inner_r = fitRadii(mat_corner_radius, radius_half);
    vec4 outer_r = inner_r + vec4(chamfer);

    g_center = center;
    g_half = half_ext;
    g_outer_r = outer_r;
    g_face_center = inner_center;
    g_face_half = inner_half;
    g_face_r = inner_r;
    chamferOut = chamfer;

    float d = sdRoundedBox(p - center, half_ext, outer_r);
    outerDist = d;
    float aa = 1.0 / niri_scale;
    coverage = 1.0 - smoothstep(-aa, 0.0, d);

    float di = sdRoundedBox(p - inner_center, inner_half, inner_r);
    innerDist = di;

    if (chamfer > 0.0 && di >= 0.0) {
        vec2 g = sdRoundedBoxGrad(p - inner_center, inner_half, inner_r);
        float bevel = min(chamfer, mat_thickness);
        float slope = length(vec2(bevel, chamfer));
        normal = normalize(vec3(g * (bevel / slope), chamfer / slope));
    } else {
        normal = vec3(0.0, 0.0, 1.0);
    }
}

// One refraction tap: bend the orthographic ray at the surface normal and
// sample the composed background where the displaced ray lands. Offsets
// are logical px mapped through the element-UV frame; the element and the
// background buffers share the y-down orientation, so no axis flip.
vec3 tap(vec2 v, vec3 n, float ior, float thickness) {
    vec3 refr = refract(vec3(0.0, 0.0, -1.0), n, 1.0 / ior);
    vec2 vv = v + (refr.xy * thickness) / mat_area_size;
    return srgbToLinear(sampleBackground(vv));
}

// Interior light follows the perturbed normal through the light-path index
// and lands `depth` px into the slab; this is that in-plane displacement.
vec2 lightShift(vec3 n, float ior, float depth) {
    return refract(vec3(0.0, 0.0, -1.0), n, 1.0 / ior).xy * depth;
}

// The band at a landing point `q`: a Gaussian of its distance inward from
// the face edge around `gap`, plus a soft halo bleeding into the glass. The
// caller caps the shared part of the light shift at half the gap before
// landing here, so dense glass cannot carry the core out past the face.
// The per-channel aberration offsets ride on top of the capped shift, so
// the chromatic split survives the cap.
float filamentBand(vec2 q, float gap, float width, float scatter) {
    float d = -sdRoundedBox(q - g_face_center, g_face_half, g_face_r);
    float coreWidth = width + 6.0 * scatter;
    float haloWidth = 9.0 + 18.0 * scatter;
    float core = (d - gap) / coreWidth;
    float halo = (d - gap - 2.0) / haloWidth;
    return (width / coreWidth) * exp(-2.0 * core * core)
         + 0.3 * (9.0 / haloWidth) * exp(-2.0 * halo * halo);
}

// Arc position along the beam line (the face box shrunk by `gap`), clockwise
// from the end of the top-left arc, and the line's perimeter. A projection:
// valid for any q, chamfer points included. Mirrors ring.rs `arc_position`.
float arcPosition(vec2 q, float gap, out float perimeter) {
    vec2 h = g_face_half - vec2(gap);
    vec4 r = max(g_face_r - vec4(gap), vec4(0.0));     // TL, TR, BR, BL
    perimeter = 4.0 * (h.x + h.y) - (2.0 - HALF_PI) * (r.x + r.y + r.z + r.w);
    vec2 d = q - g_face_center;
    float top = 2.0 * h.x - r.x - r.y;
    float right = 2.0 * h.y - r.y - r.z;
    float bottom = 2.0 * h.x - r.z - r.w;
    float left = 2.0 * h.y - r.w - r.x;
    float afterTop = top + r.y * HALF_PI;
    float afterRight = afterTop + right + r.z * HALF_PI;
    float afterBottom = afterRight + bottom + r.w * HALF_PI;
    if (d.x > h.x - r.y && d.y < -h.y + r.y) {
        float a = clamp(atan(d.y + h.y - r.y, d.x - h.x + r.y), -HALF_PI, 0.0);
        return top + r.y * (a + HALF_PI);
    }
    if (d.x > h.x - r.z && d.y > h.y - r.z) {
        float a = clamp(atan(d.y - h.y + r.z, d.x - h.x + r.z), 0.0, HALF_PI);
        return afterTop + right + r.z * a;
    }
    if (d.x < -h.x + r.w && d.y > h.y - r.w) {
        float a = clamp(atan(d.y - h.y + r.w, d.x + h.x - r.w), HALF_PI, PI);
        return afterRight + bottom + r.w * (a - HALF_PI);
    }
    if (d.x < -h.x + r.x && d.y < -h.y + r.x) {
        float a = atan(d.y + h.y - r.x, d.x + h.x - r.x);
        if (a > 0.0) a -= 2.0 * PI;
        a = clamp(a, -PI, -HALF_PI);
        return afterBottom + left + r.x * (a + PI);
    }
    vec2 toEdge = h - abs(d);
    if (toEdge.y <= toEdge.x) {
        if (d.y < 0.0) return clamp(d.x + h.x - r.x, 0.0, top);
        return afterRight + clamp(h.x - r.z - d.x, 0.0, bottom);
    }
    if (d.x > 0.0) return afterTop + clamp(d.y + h.y - r.y, 0.0, right);
    // the left edge starts where the bottom-left arc ends, at y = h − r.w
    return afterBottom + clamp(h.y - r.w - d.y, 0.0, left);
}
