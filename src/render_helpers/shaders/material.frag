precision highp float;

varying vec2 niri_v_coords;
uniform vec2 niri_size;
uniform float niri_scale;
uniform float niri_alpha;

uniform sampler2D niri_tex_win;
uniform sampler2D niri_tex_bg;
uniform vec4 mat_win_rect;
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
uniform float mat_jelly_flex;
uniform float mat_jelly_ripple;
uniform float mat_lip;
uniform vec2 mat_shift;

void main() {
    vec2 win_uv = mat_win_rect.xy + niri_v_coords * mat_win_rect.zw;
    vec4 win = texture2D(niri_tex_win, win_uv);

    vec2 bg_uv = mat_bg_rect.xy + niri_v_coords * mat_bg_rect.zw;
    vec3 bg = texture2D(niri_tex_bg, clamp(bg_uv, 0.0, 1.0)).rgb;

    // Stand-in for the glass terms until slice 2: tint the background
    // toward the attenuation colour, by an amount that scales with
    // thickness over its configured range.
    float strength = clamp(mat_thickness / 200.0, 0.0, 1.0);
    vec3 glassed = mix(bg, bg * mat_attenuation_color.rgb, strength);

    vec4 color = vec4(win.rgb + (1.0 - win.a) * glassed,
                      win.a + (1.0 - win.a) * 1.0);
    gl_FragColor = color * niri_alpha;
}
