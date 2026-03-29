// magnify_bicubic.wgsl -- Bicubic Catmull-Rom magnification with sRGB-correct sampling
//
// Higher-quality shader using 4x4 tap pattern (16 texture lookups per pixel).
// Produces sharper text and edges at high zoom levels (10x-20x) compared to
// bilinear. Uses Catmull-Rom spline weights (a = -0.5) for optimal sharpness
// without ringing artifacts.

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
};

struct MagnifyUniforms {
    viewport_size: vec2f,
    source_size: vec2f,
    is_bgra: f32,
    _pad: f32,
    _pad2: vec2f,
};

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> uniforms: MagnifyUniforms;

// Full-screen triangle vertex shader (shared with bilinear variant).
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let uv = vec2f(
        f32((vertex_index << 1u) & 2u),
        f32(vertex_index & 2u),
    );
    out.position = vec4f(uv * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2f(uv.x, 1.0 - uv.y);
    return out;
}

// Cubic interpolation weight function (Catmull-Rom spline, a = -0.5).
//
// Standard Catmull-Rom weights:
//   |x| <= 1:  1.5*|x|^3 - 2.5*|x|^2 + 1
//   1 < |x| < 2: -0.5*|x|^3 + 2.5*|x|^2 - 4*|x| + 2
//   |x| >= 2:  0
fn cubic_weight(x: f32) -> f32 {
    let ax = abs(x);
    if ax <= 1.0 {
        return 1.5 * ax * ax * ax - 2.5 * ax * ax + 1.0;
    }
    if ax < 2.0 {
        return -0.5 * ax * ax * ax + 2.5 * ax * ax - 4.0 * ax + 2.0;
    }
    return 0.0;
}

// Bicubic interpolation: 4x4 tap pattern (16 texture lookups per pixel).
//
// For each output pixel, samples a 4x4 grid of source texels centered
// on the corresponding source position. The contribution of each texel
// is weighted by the product of horizontal and vertical Catmull-Rom
// weights based on the fractional distance from the sample center.
fn sample_bicubic(tex: texture_2d<f32>, samp: sampler, uv: vec2f, tex_size: vec2f) -> vec4f {
    let pixel = uv * tex_size - 0.5;
    let pixel_floor = floor(pixel);
    let frac = pixel - pixel_floor;

    var result = vec4f(0.0);
    var weight_sum = 0.0;

    for (var j = -1; j <= 2; j++) {
        for (var i = -1; i <= 2; i++) {
            let offset = vec2f(f32(i), f32(j));
            let sample_pos = (pixel_floor + offset + 0.5) / tex_size;
            let w = cubic_weight(frac.x - f32(i)) * cubic_weight(frac.y - f32(j));
            result += textureSampleLevel(tex, samp, sample_pos, 0.0) * w;
            weight_sum += w;
        }
    }

    return result / weight_sum;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    // Sample the source texture with bicubic interpolation.
    var color = sample_bicubic(source_tex, source_sampler, in.uv, uniforms.source_size);

    // Channel swizzle for BGRA sources (Windows DXGI).
    if uniforms.is_bgra > 0.5 {
        color = vec4f(color.b, color.g, color.r, color.a);
    }

    return color;
}
