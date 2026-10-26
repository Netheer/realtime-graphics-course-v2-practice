const PI: f32 = 3.14159265;

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
    pointLightPosition: vec3f,
    pointLightIntensity: vec3f,
    pointLightAttenuation: vec3f,
}

struct Immediates {
    model: mat4x4f,
    albedo: vec3f,
    roughness: f32,
    specular: f32,
}

@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(1) @binding(0) var<uniform> lights: LightsUniforms;

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

fn calculateLighting(albedo: vec3f, normal: vec3f, light_direction: vec3f, light_intensity: vec3f,
    camera_direction: vec3f, roughness: f32, specular: f32) -> vec3f {
    let r = clamp(roughness, 0.01, 1.0);
    let power = 1.0 / (r * r) - 1.0;
    let h = normalize(camera_direction + light_direction);
    let q = max(0.0, dot(normal, light_direction));
    let s = specular * (power + 2) / (2 * PI) * pow(max(0.0, dot(normal, h)), power);
    let color_specular = albedo * light_intensity * q * s;
    let color_diffuse = albedo * light_intensity * q;
    let color = color_specular + color_diffuse;
    return color;
}

@fragment
fn fragmentMain(in: VertexOut) -> @location(0) vec4f {
    let normal = normalize(in.normal);
    let light_direction = lights.pointLightPosition - in.world_position;
    let light_distance = length(light_direction);
    let normalized_direction = light_direction / max(light_distance, 0.0001);
    let attenuation = 1 / (lights.pointLightAttenuation.x + lights.pointLightAttenuation.y * light_distance
    + lights.pointLightAttenuation.z * light_distance * light_distance);
    let new_intensity = lights.pointLightIntensity * attenuation;
    let camera_direction = normalize(camera.position - in.world_position);

    let color = immediates.albedo * lights.ambientIntensity
    + calculateLighting(immediates.albedo, normal, lights.lightDirection, lights.lightIntensity, camera_direction, immediates.roughness, immediates.specular)
    + calculateLighting(immediates.albedo, normal, normalized_direction, new_intensity, camera_direction, immediates.roughness, immediates.specular);

    return vec4f(color, 1.0);
}
