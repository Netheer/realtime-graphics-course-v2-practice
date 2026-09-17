@vertex
fn vertexMain(@builtin(vertex_index) vertexIndex: u32)
    -> @builtin(position) vec4f
{
    var positions = array<vec2f, 3>(
        vec2f( 0.0,  0.5),
        vec2f(-0.5, -0.5),
        vec2f( 0.5, -0.5),
    );

    return vec4f(positions[vertexIndex], 0.0, 1.0);
}

@fragment
fn fragmentMain(@builtin(position) position: vec4f) -> @location(0) vec4f {
    let cell_size = 64.0;
    let column = modf(position.x / cell_size).whole;
    let row = modf(position.y / cell_size).whole;
    let pos = modf((column + row) / 2.0).fract;
    if pos > 0 {
        return vec4f(1.0, 0.0, 0.0, 1.0);
    }
    return vec4f(0.0, 0.0, 1.0, 1.0);
}
