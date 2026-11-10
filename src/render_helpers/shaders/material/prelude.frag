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
uniform float mat_bevel_profile;
#ifdef MATERIAL_VIEW
// The rigid tilt (docs/materials/render-pipeline.md#under-a-swing-the-rigid-tilt): 1 while a swing
// runs. mat_view carries cos θ − 1, sin θ, λ = w / (κE) and the wall spill gain; mat_view_dir is d;
// mat_view_quad maps the drawn quad's UV to the resting area's UV (offset, scale); mat_view_seed is
// framebuffer px per logical px, row-major. At the identity pose: 1, (0, 0, 0, gain), d,
// (0, 0, 1, 1). Only this view variant declares them; it is bound only while a swing runs.
uniform int mat_view_on;
uniform vec4 mat_view;
uniform vec2 mat_view_dir;
uniform vec4 mat_view_quad;
uniform vec4 mat_view_seed;
#endif

// Slab geometry published by slabSurface for the focus filament: the outer
// silhouette and the face (the chamfer's inner edge, jelly included), which
// the ring beam runs inside.
vec2 g_center;
vec2 g_half;
vec4 g_outer_r;
vec2 g_face_center;
vec2 g_face_half;
vec4 g_face_r;

// Glass edge constants: bevel.rs holds the same values under the same names,
// and a test there checks these lines (docs/materials/glass-optics.md#bevel-profile).
const float BEVEL_SLOPE_CAP = 20.0;
const float BEVEL_OUTER_SOFTEN = 0.5;
const float BEVEL_RIDGE_EPS = 0.0001;
const float BEVEL_PROFILE_EPS = 0.000001;
const float BEVEL_LSP_SMALL = 0.001;
const float BEVEL_TANH_CLAMP = 10.0;
const float RAY_PATH_FLOOR = 0.25;
const float TAP_MIN_Z = 0.05;

// What a specular hook sees of the surface at this fragment
// (docs/materials/glass-optics.md#specular-hook).
struct Surface {
    vec2 p;            // element-local logical px
    vec2 v;            // element UV
    vec3 structural;   // the slab's normal
    vec3 perturbed;    // after distortion and jelly ripple
    float cosine;      // structural.z, clamped to [0, 1]
    float fresnel;     // Schlick at the structural normal
    float across;      // u: 0 at the face edge, 1 at the silhouette, 0 on the face
    vec2 acrossDir;    // normalize(grad u), outward across the bevel
    float outerDist;   // signed distance to the silhouette, negative inside
#ifdef MATERIAL_VIEW
    vec3 view;         // toward the viewer: +z at the identity pose
#endif
};

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

// log(softplus(t)), without overflow for large t or loss for very negative t.
float lsp(float t) {
    if (t > 0.0)
        return log(t + log(1.0 + exp(-t)));
    float x = exp(t);
    float ratio = x < BEVEL_LSP_SMALL ? 1.0 - 0.5 * x : log(1.0 + x) / x;
    return t + log(ratio);
}

// tanh with a clamped argument: GLSL ES 1.00 has no tanh.
float tanhs(float t) {
    float e = exp(2.0 * clamp(t, -BEVEL_TANH_CLAMP, BEVEL_TANH_CLAMP));
    return (e - 1.0) / (e + 1.0);
}

// The outer box's softened outward gradient: softplus for each positive part,
// tanhs for each sign, normalized in the log domain. vec2(0.0) on the ridge
// between parallel sides, where the caller uses the inner gradient.
vec2 softOuterGrad(vec2 p, vec2 b, vec4 radii) {
    float r = cornerRadius(p, radii);
    vec2 q = abs(p) - b + vec2(r);
    vec2 l = vec2(lsp(q.x / BEVEL_OUTER_SOFTEN), lsp(q.y / BEVEL_OUTER_SOFTEN));
    float m = max(l.x, l.y);
    vec2 c = vec2(exp(l.x - m) * tanhs(p.x / BEVEL_OUTER_SOFTEN),
                  exp(l.y - m) * tanhs(p.y / BEVEL_OUTER_SOFTEN));
    float len = length(c);
    return len < BEVEL_RIDGE_EPS ? vec2(0.0) : c / len;
}

// f(u) = 1 - (1 - u^k)^(1/k): 0 at the face edge, 1 at the silhouette.
float bevelProfile(float u, float k) {
    return 1.0 - pow(max(1.0 - pow(u, k), 0.0), 1.0 / k);
}

// f'(u), both bases kept off zero: GLSL leaves pow(0, y <= 0) undefined.
float bevelProfileSlope(float u, float k) {
    float a = clamp(u, BEVEL_PROFILE_EPS, 1.0);
    float b = clamp(1.0 - pow(u, k), BEVEL_PROFILE_EPS, 1.0);
    return pow(a, k - 1.0) * pow(b, 1.0 / k - 1.0);
}
#ifdef MATERIAL_VIEW

// The slab's frame, independent of the fragment: the silhouette and the face that trails the
// jelly, published for the filament and the traced solid. The view variant's slabSurface
// starts with it; the rest program's keeps the base's inline copy.
void slabFrame(out float chamfer) {
    vec2 slab_min = mat_slab_rect.xy * mat_area_size;
    vec2 slab_size = mat_slab_rect.zw * mat_area_size;
    vec2 half_ext = slab_size * 0.5;
    vec2 center = slab_min + half_ext;

    // Tiny-slab guard: for windows too small to carry a chamfer the upper
    // bound collapses to 0 (never below — reversed clamp bounds are
    // undefined in GLSL) and the whole slab renders as front face.
    float max_chamfer = max(min(half_ext.x, half_ext.y) - 1.0, 0.0);
    chamfer = clamp(mat_chamfer, 0.0, max_chamfer);

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
    // normal below depends on — it takes its slope from the height field
    // between the inner and outer boundaries, so a separately fitted outer
    // ring would change that run, and tilt the normal, at exactly the corners
    // it narrowed.
    vec2 radius_half = min(inner_half, half_ext - vec2(chamfer));
    vec4 inner_r = fitRadii(mat_corner_radius, radius_half);
    vec4 outer_r = inner_r + vec4(chamfer);

    g_center = center;
    g_half = half_ext;
    g_outer_r = outer_r;
    g_face_center = inner_center;
    g_face_half = inner_half;
    g_face_r = inner_r;
}
#endif

// The fragment-faked slab at an element-local logical-px position:
// coverage in [0, 1], the structural normal (y-down, +z toward the viewer),
// the local height, and the across-bevel coordinate u with its direction.
// The outer silhouette is the slab back and stays fixed; the face trails
// the jelly. The bevel is a height field measured against both boundaries
// (docs/materials/glass-optics.md#bevel-profile); bevel.rs mirrors it.
void slabSurface(vec2 p, out float coverage, out vec3 normal, out float outerDist,
                 out float innerDist, out float chamferOut, out float height,
                 out float across, out vec2 acrossDir) {
#ifdef MATERIAL_VIEW
    float chamfer;
    slabFrame(chamfer);
    chamferOut = chamfer;
    vec2 center = g_center;
    vec2 half_ext = g_half;
    vec2 inner_center = g_face_center;
    vec2 inner_half = g_face_half;
    vec4 inner_r = g_face_r;
    vec4 outer_r = g_outer_r;
#else
    // The base's frame, inline: slabFrame is its twin in the view variant. Change both.
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
    // normal below depends on — it takes its slope from the height field
    // between the inner and outer boundaries, so a separately fitted outer
    // ring would change that run, and tilt the normal, at exactly the corners
    // it narrowed.
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
#endif

    float d = sdRoundedBox(p - center, half_ext, outer_r);
    outerDist = d;
    float aa = 1.0 / niri_scale;
    coverage = 1.0 - smoothstep(-aa, 0.0, d);

    float di = sdRoundedBox(p - inner_center, inner_half, inner_r);
    innerDist = di;

    height = mat_thickness;
    across = 0.0;
    acrossDir = vec2(0.0);
    normal = vec3(0.0, 0.0, 1.0);
    if (chamfer > 0.0 && di >= 0.0) {
        vec2 gIn = sdRoundedBoxGrad(p - inner_center, inner_half, inner_r);
        vec2 gOut = softOuterGrad(p - center, half_ext, outer_r);
        if (gOut == vec2(0.0))
            gOut = gIn;
        float w = di - d;
        vec2 gradU;
        if (w < aa) {
            // Narrower than a physical pixel: rim.
            across = 1.0;
            gradU = gIn / max(w, aa);
        } else {
            across = clamp(di / w, 0.0, 1.0);
            gradU = (-d * gIn + di * gOut) / (w * w);
        }
        float rise = min(chamfer, mat_thickness);
        float k = mat_bevel_profile;
        height = mat_thickness - rise * bevelProfile(across, k);
        vec2 slope = rise * bevelProfileSlope(across, k) * gradU;
        float s = length(slope);
        if (s > BEVEL_SLOPE_CAP)
            slope *= BEVEL_SLOPE_CAP / s;
        normal = normalize(vec3(slope, 1.0));
        acrossDir = length(gradU) > 0.0 ? normalize(gradU) : gIn;
    }
}
#ifdef MATERIAL_VIEW

// The view path's state for this fragment (docs/materials/motion.md#tilted-slab-trace). main sets
// it while mat_view_on is 1. At the identity pose it keeps these initial values, the rest optics'
// own, and no stage reads it outside a branch on g_view_on.
bool g_view_on = false;
vec3 g_ray = vec3(0.0, 0.0, -1.0);   // slab-local incident direction, unit
vec3 g_view = vec3(0.0, 0.0, 1.0);   // toward the viewer: -g_ray
vec3 g_r = vec3(0.0, 0.0, -1.0);     // the incident direction, unnormalised
vec3 g_h0 = vec3(0.0);               // the window-plane hit, z = 0
vec2 g_h0_shift = vec2(0.0);         // g_h0.xy minus the screen point
vec3 g_rho = vec3(0.0, 0.0, -1.0);   // g_r / -g_r.z: the step per px of depth
float g_tau0 = 0.0;                  // the ray parameter at g_h0
bool g_plane_ok = true;              // the ray points down
bool g_decal_ok = true;              // and its plane hit lies inside the near clip

// view/mod.rs holds the same value under the same name.
const float VIEW_NEAR_CLIP = 0.75;

// (R⁻¹ − I) v and (R − I) v: the turn by ∓θ in the (d, z) plane less the identity, exactly
// zero at θ = 0. view/mod.rs mirrors both.
vec3 viewUnrotateDelta(vec3 v) {
    float a = dot(v.xy, mat_view_dir);
    float da = a * mat_view.x + v.z * mat_view.y;
    float dz = -a * mat_view.y + v.z * mat_view.x;
    return vec3(mat_view_dir * da, dz);
}

vec3 viewRotateDelta(vec3 v) {
    float a = dot(v.xy, mat_view_dir);
    float da = a * mat_view.x - v.z * mat_view.y;
    float dz = a * mat_view.y + v.z * mat_view.x;
    return vec3(mat_view_dir * da, dz);
}

// toScreen(q) − q.xy for a slab-local point: into the world, then onto z = 0
// along its camera ray. Exactly zero at the identity.
vec2 viewScreenShift(vec3 q) {
    vec3 dq = viewRotateDelta(q - vec3(g_center, 0.0));
    vec2 exy = q.xy + dq.xy;
    float zeta = (q.z + dq.z) * mat_view.z;
    return dq.xy + (exy - g_center) * (zeta / (1.0 - zeta));
}

// The camera ray through screen point p, slab-local, and its window-plane hit. slabFrame must
// have run.
void viewBegin(vec2 p) {
    vec2 rel = p - g_center;
    vec3 oShift = viewUnrotateDelta(vec3(rel, 0.0));
    vec3 dir = vec3(rel * mat_view.z, -1.0);
    g_r = dir + viewUnrotateDelta(dir);
    g_ray = normalize(g_r);
    g_view = -g_ray;
    g_plane_ok = g_r.z < 0.0;
    g_tau0 = g_plane_ok ? -oShift.z / g_r.z : 0.0;
    g_h0_shift = oShift.xy + g_r.xy * g_tau0;
    g_h0 = vec3(p + g_h0_shift, 0.0);
    g_rho = g_plane_ok ? g_r / -g_r.z : vec3(0.0, 0.0, -1.0);
    g_decal_ok = g_plane_ok
        && viewRotateDelta(g_h0 - vec3(g_center, 0.0)).z * mat_view.z <= VIEW_NEAR_CLIP;
}

// The traced slab (docs/materials/motion.md#tilted-slab-trace). view/solid.rs mirrors these line
// for line under the same names.
const float VIEW_GOLDEN = 0.618033988749895;
const int VIEW_GOLDEN_STEPS = 28;
const int VIEW_BISECT_STEPS = 24;

// The trace's state. Each starts at the value viewHit and viewTrace reset it to, which is what
// a fragment off the view path keeps: g_wall is read outside a branch on g_view_on.
float g_rise = 0.0;               // min(chamfer, thickness): the bevel's height
float g_chamfer = 0.0;            // the chamfer drawn
bool g_wall = false;              // shade this fragment as the wall
bool g_face_hit = false;          // the window-plane hit is on the flat face: it is shaded there
bool g_face_near = false;         // and lies near the outline, so its coverage is the outline's
float g_hit_z = 0.0;              // the shading point's depth
float g_view_coverage = 0.0;      // the outline coverage
float g_psi_min = 1e30;           // Ψmin over the segment, where, on which plane, in which quadrant
float g_min_t = 0.0;
vec3 g_min_plane = vec3(0.0, 0.0, 1.0);
vec2 g_min_sel = vec2(0.0);
float g_trace_t = 0.0;            // the entry on a hit, the minimiser on a miss
bool g_trace_hit = false;

// σ for corner radius r: today's box distance about the slab's centre.
float viewSigmaR(vec2 xy, float r) {
    return sdRoundedBox(xy - g_center, g_half, vec4(r));
}

// σ with the radius of the quadrant `sel` (relative to the centre) lies in.
float viewSigma(vec2 xy, vec2 sel) {
    return viewSigmaR(xy, cornerRadius(sel, g_outer_r));
}

// ∇σ in the same quadrant.
vec2 viewSigmaGrad(vec2 xy, vec2 sel) {
    float r = cornerRadius(sel, g_outer_r);
    return sdRoundedBoxGrad(xy - g_center, g_half, vec4(r));
}

// Ψ: <= 0 inside; X inverts bevelProfile. Without a rise Ψ is σ.
float viewPsi(vec3 q, vec2 sel) {
    float sigma = viewSigma(q.xy, sel);
    if (g_rise <= 0.0)
        return sigma;
    float y = -q.z / g_rise;
    float x = y >= 1.0
        ? 1.0
        : pow(max(1.0 - pow(max(1.0 - y, 0.0), mat_bevel_profile), 0.0), 1.0 / mat_bevel_profile);
    return sigma + g_chamfer * (1.0 - x);
}

// ∂Ψ/∂z = c X'(y) / rise, both bases kept off zero as bevelProfileSlope keeps them.
float viewPsiDz(vec3 q) {
    if (g_rise <= 0.0)
        return 0.0;
    float y = -q.z / g_rise;
    if (y >= 1.0)
        return 0.0;
    float k = mat_bevel_profile;
    float b = clamp(1.0 - y, BEVEL_PROFILE_EPS, 1.0);
    float a = clamp(1.0 - pow(b, k), BEVEL_PROFILE_EPS, 1.0);
    return g_chamfer * pow(a, 1.0 / k - 1.0) * pow(b, k - 1.0) / g_rise;
}

// The point t of the way down the segment: z = -t * thickness.
vec3 viewPoint(float t) {
    return g_h0 + g_rho * (t * mat_thickness);
}

// The minimum of Ψ on [a, b], its ends included; a tie goes to an end, the start first.
// `end` is 0 for the interior, 1 at a, 2 at b.
float viewMinimise(float a, float b, vec2 sel, out float psi, out float end) {
    float lo = a;
    float hi = b;
    float c = hi - VIEW_GOLDEN * (hi - lo);
    float d = lo + VIEW_GOLDEN * (hi - lo);
    float fc = viewPsi(viewPoint(c), sel);
    float fd = viewPsi(viewPoint(d), sel);
    for (int i = 0; i < VIEW_GOLDEN_STEPS; ++i) {
        if (fc < fd) {
            hi = d;
            d = c;
            fd = fc;
            c = hi - VIEW_GOLDEN * (hi - lo);
            fc = viewPsi(viewPoint(c), sel);
        } else {
            lo = c;
            c = d;
            fc = fd;
            d = lo + VIEW_GOLDEN * (hi - lo);
            fd = viewPsi(viewPoint(d), sel);
        }
    }
    float t = fc < fd ? c : d;
    psi = fc < fd ? fc : fd;
    end = 0.0;
    float fa = viewPsi(viewPoint(a), sel);
    float fb = viewPsi(viewPoint(b), sel);
    if (fb <= psi) {
        t = b;
        psi = fb;
        end = 2.0;
    }
    if (fa <= psi) {
        t = a;
        psi = fa;
        end = 1.0;
    }
    return t;
}

// Brackets the ray's entry: the segment split where it crosses the
// centre lines, each piece minimised in its own quadrant, the first piece that reaches 0
// bisected. Then top or wall: below the rim and outside at the rim depth is the wall, Ψ there
// taken in the quadrant of the piece that holds the rim depth (the earlier one on a seam).
void viewTrace() {
    float T = mat_thickness;
    float c1 = 2.0;
    float c2 = 2.0;
    vec3 n1 = vec3(1.0, 0.0, 0.0);
    vec3 n2 = vec3(0.0, 1.0, 0.0);
    if (T > 0.0 && g_rho.x != 0.0) {
        float at = (g_center.x - g_h0.x) / (T * g_rho.x);
        if (at > 0.0 && at < 1.0)
            c1 = at;
    }
    if (T > 0.0 && g_rho.y != 0.0) {
        float at = (g_center.y - g_h0.y) / (T * g_rho.y);
        if (at > 0.0 && at < 1.0)
            c2 = at;
    }
    if (c2 < c1) {
        float s = c1;
        c1 = c2;
        c2 = s;
        vec3 m = n1;
        n1 = n2;
        n2 = m;
    }
    float e1 = min(c1, 1.0);
    float e2 = min(c2, 1.0);
    vec3 m1 = c1 < 1.0 ? n1 : vec3(0.0, 0.0, 1.0);
    vec3 m2 = c2 < 1.0 ? n2 : vec3(0.0, 0.0, 1.0);
    g_psi_min = 1e30;
    g_min_t = 0.0;
    g_min_plane = vec3(0.0, 0.0, 1.0);
    g_min_sel = vec2(0.0);
    g_trace_hit = false;
    g_trace_t = 0.0;
    float rim = T > 0.0 ? min(g_rise / T, 1.0) : 0.0;
    // The last non-empty piece ends at 1 >= rim, so the loop always sets it.
    vec2 rimSel = vec2(0.0);
    bool rimFound = false;
    for (int i = 0; i < 3; ++i) {
        float a = i == 0 ? 0.0 : i == 1 ? e1 : e2;
        float b = i == 0 ? e1 : i == 1 ? e2 : 1.0;
        vec3 na = i == 0 ? vec3(0.0, 0.0, 1.0) : i == 1 ? m1 : m2;
        vec3 nb = i == 0 ? m1 : i == 1 ? m2 : vec3(0.0, 0.0, 1.0);
        if (b <= a)
            continue;
        vec2 sel = viewPoint(0.5 * (a + b)).xy - g_center;
        if (!rimFound && rim <= b) {
            rimFound = true;
            rimSel = sel;
        }
        float psi;
        float end;
        float t = viewMinimise(a, b, sel, psi, end);
        if (psi < g_psi_min) {
            g_psi_min = psi;
            g_min_t = t;
            g_min_sel = sel;
            g_min_plane = end == 1.0 ? na : end == 2.0 ? nb : vec3(0.0, 0.0, 1.0);
        }
        if (!g_trace_hit && psi <= 0.0) {
            g_trace_hit = true;
            if (viewPsi(viewPoint(a), sel) <= 0.0) {
                g_trace_t = a;
            } else {
                float lo = a;
                float hi = t;
                for (int j = 0; j < VIEW_BISECT_STEPS; ++j) {
                    float mid = 0.5 * (lo + hi);
                    if (viewPsi(viewPoint(mid), sel) <= 0.0)
                        hi = mid;
                    else
                        lo = mid;
                }
                g_trace_t = hi;
            }
        }
    }
    if (!g_trace_hit)
        g_trace_t = g_min_t;
    g_wall = g_trace_t > rim && viewPsi(viewPoint(rim), rimSel) > 0.0;
}

// The outline distance m: Ψmin over the norm of its screen gradient at the minimiser,
// held to its plane, as a correction to Ψmin that is exactly zero at the identity.
float viewOutline() {
    vec3 q = viewPoint(g_min_t);
    vec2 g0 = viewSigmaGrad(q.xy, g_min_sel);
    vec3 gq = vec3(g0, viewPsiDz(q));
    vec3 n = g_min_plane;
    float tau = g_tau0 + g_min_t * mat_thickness / -g_r.z;
    float scale = 1.0 + tau * mat_view.z;
    vec3 ux = vec3(1.0, 0.0, 0.0) + viewUnrotateDelta(vec3(1.0, 0.0, 0.0));
    vec3 uy = vec3(0.0, 1.0, 0.0) + viewUnrotateDelta(vec3(0.0, 1.0, 0.0));
    vec3 jx = scale * (ux - g_r * (dot(n, ux) / dot(n, g_r)));
    vec3 jy = scale * (uy - g_r * (dot(n, uy) / dot(n, g_r)));
    vec2 g = vec2(dot(gq, jx), dot(gq, jy));
    float len = length(g);
    if (len == 0.0)
        return g_psi_min;
    return g_psi_min + g_psi_min * (length(g0) - len) / len;
}

// The shading point's shift from the screen point viewBegin took: the
// window-plane hit on the flat face; else, past a sure miss, the traced entry, or on a miss the
// minimiser. Sets the coverage and whether to shade the wall. slabFrame and viewBegin must have
// run, and main sets g_chamfer and g_rise.
vec2 viewHit() {
    g_wall = false;
    g_face_hit = false;
    g_face_near = false;
    g_view_coverage = 0.0;
    g_hit_z = 0.0;
    if (!g_plane_ok)
        return vec2(0.0);
    float aa = 1.0 / niri_scale;
    if (sdRoundedBox(g_h0.xy - g_face_center, g_face_half, g_face_r) < 0.0) {
        g_face_hit = true;
        // Far from the outline the coverage is 1 either way. Within 4 physical px of the
        // face plane's cross-section (sigma + chamfer), the projected outline can be nearer
        // than the slab-local one: with bevel 0 the face reaches the silhouette, and a
        // slab-local coverage errs past the side bound there (plan review round 1). The
        // plane-to-screen map shrinks no distance by more than 2x over the configured
        // ranges, so 4 px slab-local is at least 2 px on screen.
        if (viewSigma(g_h0.xy, g_h0.xy - g_center) + g_chamfer > -4.0 * aa) {
            g_face_near = true;
            viewTrace();
            g_wall = false;
            g_view_coverage = 1.0 - smoothstep(-aa, 0.0, viewOutline());
        }
        return g_h0_shift;
    }
    // A sure miss: coverage exactly 0, no bracketing.
    float rMin = min(min(g_outer_r.x, g_outer_r.y), min(g_outer_r.z, g_outer_r.w));
    if (viewSigmaR(g_h0.xy, rMin) - mat_thickness * length(g_rho.xy) > 0.0)
        return g_h0_shift;
    viewTrace();
    g_view_coverage = 1.0 - smoothstep(-aa, 0.0, viewOutline());
    g_hit_z = -g_trace_t * mat_thickness;
    return g_h0_shift + g_rho.xy * (g_trace_t * mat_thickness);
}
#endif

// The refracted ray's length to the backdrop plane under local height h
// (docs/materials/glass-optics.md#slab-and-ray-model). On the face, t = (0, 0, -1) and L = h.
float rayPath(vec3 t, float h) {
    return h / max(-t.z, RAY_PATH_FLOOR);
}

// Distortion and ripple can tip a tap's normal past horizontal; lift it into
// the structural cap so every refracted ray points down.
vec3 liftTapNormal(vec3 n) {
    return normalize(vec3(n.xy, max(n.z, TAP_MIN_Z)));
}

// One refraction tap: bend the orthographic ray at the lifted normal, follow
// it to the backdrop plane under height h, and sample the composed background
// there. Offsets are logical px mapped through the element-UV frame; the
// element and the background buffers share the y-down orientation.
vec3 tap(vec2 v, vec3 n, float ior, float h) {
    vec3 refr = refract(vec3(0.0, 0.0, -1.0), liftTapNormal(n), 1.0 / ior);
#ifdef MATERIAL_VIEW
    if (g_view_on)
        refr = refract(g_ray, liftTapNormal(n), 1.0 / ior);
#endif
    vec2 vv = v + (refr.xy * rayPath(refr, h)) / mat_area_size;
#ifdef MATERIAL_VIEW
    // Under a tilt the exit point on the back plane goes through toScreen.
    if (g_view_on)
        vv += viewScreenShift(vec3(v * mat_area_size + refr.xy * rayPath(refr, h), -mat_thickness))
            / mat_area_size;
#endif
    return srgbToLinear(sampleBackground(vv));
}

// Interior light follows the perturbed normal through the light-path index
// and lands `depth` px into the slab; this is that in-plane displacement.
vec2 lightShift(vec3 n, float ior, float depth) {
#ifdef MATERIAL_VIEW
    if (g_view_on)
        return refract(g_ray, n, 1.0 / ior).xy * depth;
#endif
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
