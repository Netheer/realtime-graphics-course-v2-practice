struct ObjectUniforms {
    model: mat4x4f,
    view_projection: mat4x4f
}

struct StarOutput {
    @builtin(position) position: vec4f,
    @location(0) color: vec3f
}

@vertex
fn vertexStars(@location(0) position: vec3f, @location(1) direction: vec3f, @location(2) color: vec3f, @location(3) size: f32) -> StarOutput {
    var output: StarOutput;

    let p = direction + position * size;
    output.position = uniforms.view_projection * vec4f(p, 1);
    output.color = color;
    return output;
}

@fragment
fn fragmentStars(@location(0) color: vec3f) -> @location(0) vec4f {
    return vec4f(color, 1.0);
}

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) color: vec3f,
    @location(1) world_position: vec3f,
    @location(2) world_normal: vec3f
}

@group(0) @binding(0) var<uniform> uniforms: ObjectUniforms;

@vertex
fn vertexMain(@location(0) position: vec3f, @location(1) normal: vec3f, @location(2) color: vec3f)
    -> VertexOutput
{
    var output: VertexOutput;

    let world_position = uniforms.model * vec4f(position, 1.0);
    output.world_position = world_position.xyz;

    output.position = uniforms.view_projection * world_position;
    output.color = color;
    output.world_normal = (uniforms.model * vec4f(normal, 0.0)).xyz;

    return output;
}

@fragment
fn fragmentMain(input: VertexOutput, @builtin(front_facing) front_facing: bool) -> @location(0) vec4f {
    var normal = normalize(input.world_normal);
    if !front_facing {
        normal = -normal;
    }
    let light_direction = -normalize(input.world_position);
    let diffuse = max(0.0, dot(normal, light_direction));
    let rgb = input.color * (0.05 + diffuse);

    return vec4f(rgb, 1.0);
}

@fragment
fn fragmentSun(input: VertexOutput) -> @location(0) vec4f {
    return vec4f(input.color, 1.0);
}
