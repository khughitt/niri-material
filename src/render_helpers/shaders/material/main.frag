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
    if (win.a == 1.0) {
        gl_FragColor = win * niri_alpha;
        return;
    }

    float coverage;
    vec3 surfaceNormal;
    float slabDist;
    float innerDist;
    float slabChamfer;
    slabSurface(p, coverage, surfaceNormal, slabDist, innerDist, slabChamfer);

    vec4 glassed = vec4(0.0);
    if (coverage > 0.0) {
        vec3 n = surfaceNormal;

        // Distortion perturbs the normal before the ray is built (drei
        // computes its distortion normal ahead of the sample loop too).
        // Position is element-local; continuity across slabs is not
        // needed since slabs are visually separate.
        if (mat_distortion > 0.0) {
            vec3 dp = vec3(p, 0.0) * (mat_distortion_scale * 0.01);
            n = normalize(n + mat_distortion * vec3(
                snoiseFractal(dp), snoiseFractal(dp.zxy), snoiseFractal(dp.yxz)));
        }

        if (mat_jelly_activity > 0.0 && mat_jelly_ripple > 0.0) {
            vec3 rp = vec3(p, 0.0) * 0.005 + mat_jelly_seed;
            vec3 phase = vec3(mat_jelly_time, -mat_jelly_time, -mat_jelly_time) * 0.5;
            vec3 ripple = vec3(
                snoiseJelly(rp + phase),
                snoiseJelly(rp.zxy + phase),
                0.0);
            n = normalize(n + mat_jelly_activity * mat_jelly_ripple * ripple);
        }

        // Neutral effects make every tap identical: skip the loop and
        // jitter entirely. Config validation keeps both non-negative, so
        // exact zero agrees with the gates.
        vec3 sampled;
        if (mat_anisotropic_blur == 0.0 && mat_chromatic_aberration == 0.0) {
            sampled = tap(v, n, mat_ior, mat_thickness);
        } else {
            float smear = mat_thickness * mat_anisotropic_blur;
            float count = clamp(mat_samples, 1.0, 8.0);
            vec3 acc = vec3(0.0);
            for (int i = 0; i < 8; ++i) {
                float fi = float(i);
                if (fi >= count)
                    break;
                float r1 = hash12(gl_FragCoord.xy + fi * 17.0);
                float r2 = hash12(gl_FragCoord.xy + fi * 71.0 + 3.7);
                float t = mat_thickness + smear * (fi + r1) / count;
                if (mat_chromatic_aberration > 0.0) {
                    // drei's per-sample per-channel ior spread
                    float spread = mat_chromatic_aberration * (fi + r2) / count;
                    acc.r += tap(v, n, mat_ior, t).r;
                    acc.g += tap(v, n, mat_ior * (1.0 + spread), t).g;
                    acc.b += tap(v, n, mat_ior * (1.0 + 2.0 * spread), t).b;
                } else {
                    acc += tap(v, n, mat_ior, t);
                }
            }
            sampled = acc / count;
        }

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

        // Fresnel edge glint: Schlick from the configured IOR on the
        // structural normal. Replaces the legacy
        // environment-probe specular on the chamfer; additive per the §2
        // slab terms. The strength constants are art-directed against the
        // legacy look and validated visually, not physically derived.
        float f0 = (mat_ior - 1.0) / (mat_ior + 1.0);
        f0 = f0 * f0;
        float fresnel = f0 + (1.0 - f0) * pow(1.0 - surfaceCosine, 5.0);
        float facing = 0.0;
        if (length(surfaceNormal.xy) > 0.001)
            facing = max(dot(normalize(surfaceNormal.xy),
                             normalize(mat_sig_light.xy)), 0.0);
        vec3 specular = vec3(fresnel * (0.15 + 0.85 * facing));
        if (mat_sig_accent.w > 0.0 && mat_sig_light.z > 0.0)
            specular = mix(specular, specular * mat_sig_accent.rgb * 2.0,
                           mat_sig_light.z * mat_sig_accent.w);

        vec3 emissive = vec3(0.0);
        bool showAccent = mat_sig_response.x == 1 && mat_sig_accent.w > 0.0;
        bool showFocus = mat_sig_response.z == 1 && mat_sig_focus.x > 0.0;
        if ((showAccent || showFocus) && slabChamfer > 0.0) {
            // Mask at the displayed fragment, never at the refracted point:
            // exactly 0 on the face and everywhere di <= 0, 1 one px into
            // the bevel, 0 everywhere when the rendered chamfer is 0.
            float mask = smoothstep(0.0, 1.0, innerDist);
            if (mask > 0.0) {
                float inset = mat_sig_ring.x;
                float width = mat_sig_ring.y;
                float depth = mat_thickness * 0.6;
                float ior = 1.0 + (mat_ior - 1.0) * mat_light_ior;
                float ca = mat_chromatic_aberration * 0.1;
                vec2 shift0 = lightShift(n, ior, depth);
                float len0 = length(shift0);
                float cap = 0.5 * inset;
                vec2 base = len0 > cap ? shift0 * (cap / len0) : shift0;
                vec2 q0 = p + base;
                vec3 band = vec3(
                    filamentBand(q0, inset, width),
                    filamentBand(q0 + lightShift(n, ior * (1.0 + ca), depth) - shift0,
                                 inset, width),
                    filamentBand(q0 + lightShift(n, ior * (1.0 + 2.0 * ca), depth) - shift0,
                                 inset, width));
                float presence = mat_sig_accent.w;
                float pulse = mat_sig_response.y == 2 ? mat_sig_breath : 0.0;
                float accentGlow = showAccent
                    ? (0.15 + 0.35 * mat_sig_level) * (1.0 + pulse * mat_sig_level)
                    : 0.0;
                float focusGlow = 0.0;
                if (showFocus) {
                    vec2 q = q0 - g_center;
                    float ang = atan(q.y, q.x);
                    float drift = mat_sig_focus.y;
                    // Mirrors signal.rs `travel`: integer multiples of the
                    // drift keep the 2π wrap continuous.
                    float travel = sin(ang * 2.0 + drift) * sin(ang * 3.0 - 2.0 * drift);
                    focusGlow = mat_sig_focus.x * 0.7 * (0.55 + 0.45 * travel);
                }
                float glow = (accentGlow * presence + focusGlow) * (1.0 + 2.0 * mat_jelly_activity);
                vec3 color = showAccent
                    ? mix(mat_sig_ring_color, mat_sig_accent.rgb, presence)
                    : mat_sig_ring_color;
                emissive += color * glow * band * mask * pow(att, vec3(0.2));
            }
        }

        float diag = (p.x + p.y) / (mat_area_size.x + mat_area_size.y);
        for (int k = 0; k < 4; ++k) {
            int sel = k == 0 ? mat_sig_impulse_resp.x : k == 1 ? mat_sig_impulse_resp.y
                    : k == 2 ? mat_sig_impulse_resp.z : mat_sig_impulse_resp.w;
            if (sel != 3)
                continue;
            float env = k == 0 ? mat_sig_impulse_env.x : k == 1 ? mat_sig_impulse_env.y
                      : k == 2 ? mat_sig_impulse_env.z : mat_sig_impulse_env.w;
            float prog = k == 0 ? mat_sig_impulse_prog.x : k == 1 ? mat_sig_impulse_prog.y
                       : k == 2 ? mat_sig_impulse_prog.z : mat_sig_impulse_prog.w;
            vec3 rgb = k == 0 ? mat_sig_impulse_rgb0 : k == 1 ? mat_sig_impulse_rgb1
                     : k == 2 ? mat_sig_impulse_rgb2 : mat_sig_impulse_rgb3;
            float d = (diag - prog) / 0.06;
            emissive += rgb * env * 0.5 * exp(-d * d);
        }

        vec3 glassColor = linearToSrgb(transmitted + specular + emissive);
        // Post hooks, in OPTICS order (render-pipeline.md stages 9 and 10).
        glassColor = saturation_post(glassColor, gl_FragCoord.xy);
        glassColor = noise_post(glassColor, gl_FragCoord.xy);
        glassed = vec4(glassColor, 1.0) * coverage;
    }

    // §2 compositing contract: opaque window pixels pass through
    // untouched; the slab contributes only through window transparency
    // and outside the window. Premultiplied throughout; window-rule
    // opacity arrives once, as niri_alpha.
    vec4 color = win + (1.0 - win.a) * glassed;
    gl_FragColor = color * niri_alpha;
}
