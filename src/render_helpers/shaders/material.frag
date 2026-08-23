precision highp float;

varying vec2 niri_v_coords;
uniform vec2 niri_size;
uniform float niri_scale;
uniform float niri_alpha;

uniform sampler2D niri_tex_win;
uniform sampler2D niri_tex_bg;
uniform vec4 mat_win_rect;
uniform vec4 mat_geo_rect;
uniform vec4 mat_slab_rect;
uniform vec2 mat_area_size;
uniform float mat_chamfer;
uniform vec4 mat_bg_rect;
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
        vec2 bg_uv = mat_bg_rect.xy + v * mat_bg_rect.zw;
        vec3 bg = texture2D(niri_tex_bg, clamp(bg_uv, 0.0, 1.0)).rgb;
        float strength = clamp(mat_thickness / 200.0, 0.0, 1.0);
        glassed = vec4(mix(bg, bg * mat_attenuation_color.rgb, strength), 1.0);
    }

    vec4 color = win + (1.0 - win.a) * glassed;
    gl_FragColor = color * niri_alpha;
}
