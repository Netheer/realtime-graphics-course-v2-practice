use std::collections::HashMap;
use std::f32::consts::{TAU};
use glam::Vec3;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct SphereVertex {
    pub(super) position : [f32; 3],
    pub(super) normal : [f32; 3],
}

pub(super) fn generate_icosahedron_positions() -> Vec<Vec3> {
    let phi = (1.0_f32 + 5.0_f32.sqrt()) / 2.0;
    let mut positions = Vec::with_capacity(12);
    for i in [-1.0_f32, 1.0_f32] {
        for j in [-1.0_f32, 1.0_f32] {
            positions.push(Vec3::new(0.0, i, j * phi));
            positions.push(Vec3::new(j * phi, 0.0, i));
            positions.push(Vec3::new(i, j * phi, 0.0));
        }
    }

    positions
}

pub(super) fn generate_icosahedron_indices(positions: &[Vec3]) -> Vec<u32> {
    let mut indices = Vec::with_capacity(60);
    for i in 0..positions.len() {
        for j in (i + 1)..positions.len() {
            for k in (j + 1)..positions.len() {
                let ij = (positions[i] - positions[j]).length_squared();
                let jk = (positions[j] - positions[k]).length_squared();
                let ik = (positions[i] - positions[k]).length_squared();

                if (ij - 4.0).abs() < 1e-5_f32 && (jk - 4.0).abs() < 1e-5_f32 && (ik - 4.0).abs() < 1e-5_f32 {
                    let u = positions[j] - positions[i];
                    let v = positions[k] - positions[i];
                    let normal = u.cross(v);
                    if positions[i].dot(normal) > 0.0_f32 {
                        indices.extend_from_slice(&[i as u32, j as u32, k as u32]);
                    } else {
                        indices.extend_from_slice(&[i as u32, k as u32, j as u32]);
                    }
                }
            }
        }
    }

    indices
}

fn midpoint_index(
    a: u32,
    b: u32,
    positions : &mut Vec<Vec3>,
    cache: &mut HashMap<(u32, u32), u32>
) -> u32 {
    let key = (a.min(b), a.max(b));
    if let Some(&index) = cache.get(&key) {
        return index;
    }
    let coords = ((positions[a as usize] + positions[b as usize]) / 2.0).normalize();
    let index = positions.len() as u32;
    positions.push(coords);
    cache.insert(key, index);
    index
}

pub(super) fn transform(
    positions: &mut Vec<Vec3>,
    indices: &[u32]
) -> Vec<u32> {
    let mut connections : HashMap<(u32, u32), u32> = HashMap::new();
    let mut indexes : Vec<u32> = Vec::with_capacity(indices.len() * 4);
    for triangle in indices.chunks_exact(3) {
        let a = triangle[0];
        let b = triangle[1];
        let c = triangle[2];
        let ab = midpoint_index(a, b, positions, &mut connections);
        let bc = midpoint_index(b, c, positions, &mut connections);
        let ac = midpoint_index(a, c, positions, &mut connections);

        indexes.push(a);
        indexes.push(ab);
        indexes.push(ac);
        indexes.push(ab);
        indexes.push(b);
        indexes.push(bc);
        indexes.push(ac);
        indexes.push(bc);
        indexes.push(c);
        indexes.push(ab);
        indexes.push(bc);
        indexes.push(ac);
    }

    indexes
}

pub(super) fn generate_colors(vertices: &[SphereVertex], c: f32, color1: Vec3, color2: Vec3) -> Vec<[f32; 3]> {
    let mut vertex_colors: Vec<[f32; 3]> = Vec::new();
    for vertex in vertices {
        let t = ((c * vertex.position[1]).sin() + 1.0) / 2.0;
        let color = ((1.0 - t) * color1 + t * color2).to_array();
        vertex_colors.push(color);
    }

    vertex_colors
}

pub(super) fn random_color() -> Vec3 {
    let red = rand::random_range(0.1_f32..1.0_f32);
    let green = rand::random_range(0.1_f32..1.0_f32);
    let blue = rand::random_range(0.1_f32..1.0_f32);
    let color = Vec3::new(red, green, blue);
    color
}

pub(super) fn generate_ring(
    inner_radius: f32,
    outer_radius: f32,
    segments: u32) -> (Vec<SphereVertex>, Vec<u32>) {
    let mut vertexes: Vec<SphereVertex> = Vec::with_capacity((segments * 2) as usize);
    let mut indices: Vec<u32> = Vec::new();
    for i in 0..segments {
        let alpha = TAU * i as f32 / segments as f32;
        let inner = SphereVertex {
            position: [inner_radius * alpha.cos(), 0.0, inner_radius * alpha.sin()],
            normal: [0.0, 1.0, 0.0],
        };
        let outer = SphereVertex {
            position: [outer_radius * alpha.cos(), 0.0, outer_radius * alpha.sin()],
            normal: [0.0, 1.0, 0.0],
        };
        vertexes.push(inner);
        vertexes.push(outer);

        let next = (i + 1) % segments;
        indices.extend_from_slice(&[i * 2u32, next * 2u32, i * 2u32 + 1u32]);
        indices.extend_from_slice(&[next * 2u32, next * 2u32 + 1u32, i * 2u32 + 1u32]);
    }

    (vertexes, indices)
}

