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

void main() {
    vec2 v = niri_v_coords;

    // Window sample: zero outside the texture footprint.
    vec4 win = vec4(0.0);
    if (inRect(v, mat_geo_rect)) {
        vec2 wv = (v - mat_geo_rect.xy) / mat_geo_rect.zw;
        win = texture2D(niri_tex_win, mat_win_rect.xy + wv * mat_win_rect.zw);
    }

    // Stand-in for the glass terms until the slab shading lands: tint the
    // background toward the attenuation colour across the slab rect.
    vec4 glassed = vec4(0.0);
    if (inRect(v, mat_slab_rect)) {
        vec3 bg = sampleBackground(v);
        float strength = clamp(mat_thickness / 200.0, 0.0, 1.0);
        glassed = vec4(mix(bg, bg * mat_attenuation_color.rgb, strength), 1.0);
    }

    vec4 color = win + (1.0 - win.a) * glassed;
    gl_FragColor = color * niri_alpha;
}
