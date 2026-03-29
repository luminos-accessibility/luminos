// magnify_bilinear.wgsl -- Bilinear magnification with sRGB-correct sampling
//
// Phase 0 default shader. Uses hardware texture filtering (single
// textureSampleLevel call) for maximum performance. Text may appear
// slightly blurry at high zoom levels (10x+). The bicubic variant
// provides sharper results at higher GPU cost.

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

// Full-screen triangle vertex shader (3 vertices, no vertex buffer needed).
//
// Generates a single triangle that covers the entire clip space [-1, 1]
// using only the built-in vertex_index (0, 1, 2). This avoids the need
// for a vertex buffer entirely.
//
// Vertex positions and UVs:
//   index 0: pos (-1, -1), uv (0, 1)  -- bottom-left
//   index 1: pos ( 3, -1), uv (2, 1)  -- far right (clipped)
//   index 2: pos (-1,  3), uv (0,-1)  -- far top (clipped)
//
// After clipping, the visible portion covers the full viewport with
// UV coordinates [0,1] mapping correctly to the source texture.
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let uv = vec2f(
        f32((vertex_index << 1u) & 2u),
        f32(vertex_index & 2u),
    );
    out.position = vec4f(uv * 2.0 - 1.0, 0.0, 1.0);
    // Flip Y for texture coordinates (top-left origin).
    out.uv = vec2f(uv.x, 1.0 - uv.y);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    // Sample the source texture with bilinear interpolation (hardware filtering).
    var color = textureSampleLevel(source_tex, source_sampler, in.uv, 0.0);

    // Channel swizzle for BGRA sources (Windows DXGI).
    if uniforms.is_bgra > 0.5 {
        color = vec4f(color.b, color.g, color.r, color.a);
    }

    return color;
}
