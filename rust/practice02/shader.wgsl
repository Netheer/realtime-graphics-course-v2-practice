struct VertexOut {
    @builtin(position) position: vec4f,
    @location(0) color: vec4f,
}

const R : f32 = 1.0;
const PI : f32 = 3.141592653589793;
const POSITIONS = array<vec2f, 6>(
    vec2f(R, 0.0),
    vec2f(R * cos(60 * PI / 180), R * sin(60 * PI / 180)),
    vec2f(R * cos(120 * PI / 180), R * sin(120 * PI / 180)),
    vec2f(R * cos(180 * PI / 180), R * sin(180 * PI / 180)),
    vec2f(R * cos(240 * PI / 180), R * sin(240 * PI / 180)),
    vec2f(R * cos(300 * PI / 180), R * sin(300 * PI / 180)),
);

const TRIANGLES = array<u32, 12>(
    0u, 1u, 2u,
    0u, 2u, 3u,
    0u, 3u, 4u,
    0u, 4u, 5u,
);

const COLORS = array<vec4f, 6>(
    vec4f(1.00, 0.29, 0.29, 1.0),
    vec4f(0.16, 0.72, 0.79, 1.0),
    vec4f(1.00, 0.84, 0.40, 1.0),
    vec4f(1.00, 0.29, 0.29, 1.0),
    vec4f(0.16, 0.72, 0.79, 1.0),
    vec4f(1.00, 0.29, 0.29, 1.0),
);

struct Immediates {
    transform: mat4x4f,
    view: mat4x4f,
}
var<immediate> immediates: Immediates;

@vertex
fn vertexMain(@builtin(vertex_index) vertexIndex: u32) -> VertexOut {
    let pointIndex = TRIANGLES[vertexIndex];
    let p = POSITIONS[pointIndex];
    let v = vec4f(p.x, p.y, 0.0, 1.0);
    return VertexOut(
        immediates.view * immediates.transform * v,
        COLORS[pointIndex]
    );
}

@fragment
fn fragmentMain(in: VertexOut) -> @location(0) vec4f {
    return in.color;
}
