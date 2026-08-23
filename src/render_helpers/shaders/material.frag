precision highp float;

varying vec2 niri_v_coords;
uniform vec2 niri_size;
uniform float niri_scale;
uniform float niri_alpha;

uniform sampler2D niri_tex_win;
uniform sampler2D niri_tex_bg;
uniform vec4 mat_win_rect;
uniform vec4 mat_bg_rect;

void main() {
    vec2 win_uv = mat_win_rect.xy + niri_v_coords * mat_win_rect.zw;
    vec4 win = texture2D(niri_tex_win, win_uv);

    vec2 bg_uv = mat_bg_rect.xy + niri_v_coords * mat_bg_rect.zw;
    vec3 bg = texture2D(niri_tex_bg, clamp(bg_uv, 0.0, 1.0)).rgb;

    // Stand-in for the glass terms: a cool tint. Premultiplied, slab
    // coverage = 1.0 (element area == window area in slice 0).
    vec3 glassed = bg * vec3(0.86, 0.90, 1.0);

    vec4 color = vec4(win.rgb + (1.0 - win.a) * glassed,
                      win.a + (1.0 - win.a) * 1.0);
    gl_FragColor = color * niri_alpha;
}
