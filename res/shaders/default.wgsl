// Vertex Shader
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct InstanceInput {
    @location(5) model_0: vec4<f32>,
    @location(6) model_1: vec4<f32>,
    @location(7) model_2: vec4<f32>,
    @location(8) model_3: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct GlobalBufferContext {
    camera: CameraBufferContext,
}

struct CameraBufferContext {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> global_context: GlobalBufferContext;
@group(0) @binding(1) var trilinear_sampler: sampler;
@group(0) @binding(2) var point_sampler: sampler;

@vertex
fn vs_main(vert: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;
    let model = mat4x4<f32>(
        instance.model_0,
        instance.model_1,
        instance.model_2,
        instance.model_3,
    );
    out.clip_position = global_context.camera.view_proj * model * vec4<f32>(vert.position, 1.0);
    out.tex_coords = vert.tex_coords;
    return out;
}

// Fragment Shader
@group(1) @binding(0) var albedo_texture: texture_2d<f32>;
//@group(1) @binding(1) var universal_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.tex_coords, 0.0, 1.0);
    // return textureSample(albedo_texture, universal_sampler, in.tex_coords);
}