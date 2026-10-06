// Effect program: the backdrop grain pass (noise site=backdrop). grain_source()
// in shaders/mod.rs assembles the effect header, common.frag and the noise
// optic's GLSL ahead of this main; blur.vert supplies v_coords.
void main() {
    gl_FragColor = noise_source(texture2D(tex, v_coords), gl_FragCoord.xy);
}
