struct LightViewImmediates {
    view: mat4x4f
}
var<immediate> immediates: LightViewImmediates;

@vertex
fn vertexMain(@location(0) position: vec3f) -> @builtin(position) vec4f {
    return immediates.view * vec4f(position, 1.0);
}