struct VertexIn {
    @location(0) position: vec3f,
    @location(1) normal: vec3f,
    @location(2) texcoord: vec2f,
}

struct VertexOut {
    @builtin(position) clip_position: vec4f,
    @location(0) world_position: vec3f,
    @location(1) normal: vec3f,
}

struct CameraUniforms {
    viewProjection: mat4x4f,
    position: vec3f,
}

struct LightsUniforms {
    ambientIntensity: vec3f,
    lightDirection: vec3f,
    lightIntensity: vec3f,
    shadowProjection: mat4x4f
}

struct Immediates {
    model: mat4x4f,
    albedo: vec3f,
    roughness: f32,
    specularFactor: f32,
}

@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(1) @binding(0) var<uniform> lights: LightsUniforms;
@group(1) @binding(1) var shadow_map: texture_depth_2d;
@group(1) @binding(2) var shadow_map_sampler: sampler;
@group(1) @binding(3) var pcf_sampler: sampler_comparison;

var<immediate> immediates: Immediates;

@vertex
fn vertexMain(in: VertexIn) -> VertexOut {
    let world_position = immediates.model * vec4f(in.position, 1.0);
    return VertexOut(
        camera.viewProjection * world_position,
        world_position.xyz,
        normalize((immediates.model * vec4f(in.normal, 0.0)).xyz)
    );
}

const PI = 3.1415926535;

fn computeLight(normal: vec3f, lightDirection: vec3f, lightIntensity: vec3f, viewDirection: vec3f) -> vec3f {
    let halfway = normalize(lightDirection + viewDirection);
    let power = 1.0 / max(0.0001, immediates.roughness * immediates.roughness) - 1.0;
    return max(0.0, dot(normal, lightDirection)) * (1.0 + immediates.specularFactor * (power + 2.0) / (2.0 * PI) * pow(max(0.0, dot(halfway, normal)), power)) * lightIntensity;
}

@fragment
fn fragmentMain(in: VertexOut) -> @location(0) vec4f {
    let normal = normalize(in.normal);

    var position = lights.shadowProjection * vec4f(in.world_position, 1.0);
    let u = 0.5 + 0.5 * position.x;
    let v = 0.5 - 0.5 * position.y;
    var multiplier = 1.0;
    let sampled_visibility = textureSampleCompare(shadow_map, pcf_sampler, vec2f(u, v), position.z - 0.001);

    let viewDirection = normalize(camera.position - in.world_position);
    let halfway = normalize(lights.lightDirection + viewDirection);
    let specularPower = 1.0 / max(0.0001, immediates.roughness * immediates.roughness) - 1.0;
    let cosine = max(0.0, dot(normal, lights.lightDirection));

    if (-1.0 <= position.x && position.x <= 1.0 && -1.0 <= position.y && position.y <= 1.0 && 0.0 <= position.z && position.z <= 1.0) {
        multiplier = sampled_visibility;
    }

    let color_ambient = immediates.albedo * lights.ambientIntensity;
    let color_diffuse = immediates.albedo * lights.lightIntensity * cosine;
    let color_specular = lights.lightIntensity * immediates.specularFactor * (specularPower + 2.0) / (2.0 * PI) * pow(max(0.0, dot(halfway, normal)), specularPower);
    let color = color_ambient + multiplier * (color_diffuse + color_specular);

    return vec4f(color, 1.0);
}
