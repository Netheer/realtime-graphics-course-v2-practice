struct Camera {
    viewProjection: mat4x4f,
    position: vec3f,
    inverseViewProjection: mat4x4f,
}

struct VertexOut {
    @builtin(position) clip_position: vec4f,
    @location(0) ndc: vec2f,
}

@group(0) @binding(0) var<uniform> camera: Camera;
@group(0) @binding(1) var linearSampler: sampler;
@group(0) @binding(5) var environmentTexture: texture_2d<f32>;

const PI: f32 = 3.14159265359;

@vertex
fn vertexMain(@builtin(vertex_index) index: u32) -> VertexOut {
    let positions = array<vec2f, 6>(
        vec2f(-1.0, -1.0),
        vec2f(1.0, -1.0),
        vec2f(-1.0, 1.0),
        vec2f(-1.0, 1.0),
        vec2f(1.0, -1.0),
        vec2f(1.0, 1.0)
    );
    let ndc = positions[index];
    return VertexOut(vec4f(ndc, 1.0, 1.0), ndc);
}

@fragment
fn fragmentMain(in: VertexOut) -> @location(0) vec4f {
    let world_position = camera.inverseViewProjection * vec4f(in.ndc, 1.0, 1.0);
    let dir = normalize(world_position.xyz / world_position.w - camera.position);
    let texcoord = vec2f(
        atan2(dir.z, dir.x) / PI * 0.5 + 0.5,
        -atan2(dir.y, length(dir.xz)) / PI + 0.5
    );
    return vec4f(textureSample(environmentTexture, linearSampler, texcoord).rgb, 1.0);
}
