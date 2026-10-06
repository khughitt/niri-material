// Helpers shared by the material program and the effect-program grain pass
// (grain.frag): transfer functions, the hash grain, and Oklab. No uniforms;
// concatenated first in both programs.

// The renderer works on sRGB-encoded values; glass optics are computed in
// linear light locally and re-encoded on output (G10 pipeline work stays
// deferred).
vec3 srgbToLinear(vec3 c) {
    vec3 low = c / 12.92;
    // mix evaluates both branches; keep the power defined for signed grain.
    vec3 high = pow(max((c + 0.055) / 1.055, vec3(0.0)), vec3(2.4));
    return mix(high, low, vec3(lessThanEqual(c, vec3(0.04045))));
}

vec3 linearToSrgb(vec3 c) {
    vec3 low = c * 12.92;
    vec3 high = 1.055 * pow(max(c, vec3(0.0)), vec3(1.0 / 2.4)) - 0.055;
    return mix(high, low, vec3(lessThanEqual(c, vec3(0.0031308))));
}

float hash12(vec2 p)
{
    vec3 p3 = fract(vec3(p.xyx) * 0.1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// High-pass hash grain: the fragment's hash minus the mean of its eight
// unit-offset neighbours. A weighted sum of nine independent uniforms is
// bell-shaped and zero-mean, and the subtraction removes the low-frequency
// energy that reads as clumps. Var(centre - mean8) = (1/12)(1 + 1/8), so
// sqrt(8/9) brings the standard deviation back to a single hash's.
float fineGrain(vec2 p) {
    float mean = (hash12(p + vec2(-1.0, -1.0)) + hash12(p + vec2(0.0, -1.0))
                + hash12(p + vec2(1.0, -1.0)) + hash12(p + vec2(-1.0, 0.0))
                + hash12(p + vec2(1.0, 0.0)) + hash12(p + vec2(-1.0, 1.0))
                + hash12(p + vec2(0.0, 1.0)) + hash12(p + vec2(1.0, 1.0))) / 8.0;
    return (hash12(p) - mean) * 0.94280904;
}

// Oklab (Björn Ottosson, public domain), linear sRGB in and out.
vec3 linearToOklab(vec3 c) {
    float l = 0.4122214708 * c.r + 0.5363325363 * c.g + 0.0514459929 * c.b;
    float m = 0.2119034982 * c.r + 0.6806995451 * c.g + 0.1073969566 * c.b;
    float s = 0.0883024619 * c.r + 0.2817188376 * c.g + 0.6299787005 * c.b;
    float l_ = pow(max(l, 0.0), 1.0 / 3.0);
    float m_ = pow(max(m, 0.0), 1.0 / 3.0);
    float s_ = pow(max(s, 0.0), 1.0 / 3.0);
    return vec3(
        0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
        1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
        0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_);
}

vec3 oklabToLinear(vec3 lab) {
    float l_ = lab.x + 0.3963377774 * lab.y + 0.2158037573 * lab.z;
    float m_ = lab.x - 0.1055613458 * lab.y - 0.0638541728 * lab.z;
    float s_ = lab.x - 0.0894841775 * lab.y - 1.2914855480 * lab.z;
    float l = l_ * l_ * l_;
    float m = m_ * m_ * m_;
    float s = s_ * s_ * s_;
    return vec3(
        4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
        -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
        -0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s);
}

