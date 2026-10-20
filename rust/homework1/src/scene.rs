use std::f32::consts::TAU;
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

pub(super) struct Camera {
    pub(super) position: Vec3,
    pub(super) horizontal: f32,
    pub(super) vertical: f32,
}

impl Camera {
    pub(super) fn forward(&self) -> Vec3 {
        let direction = Vec3::new(self.vertical.cos() * self.horizontal.cos(), self.vertical.sin(), self.vertical.cos() * self.horizontal.sin());
        direction
    }
    pub(super) fn view_matrix(&self) -> glam::Mat4 {
        glam::Mat4::look_to_rh(self.position, self.forward(), Vec3::Y)
    }
}

pub(super) struct CelestialBody {
    pub(super) model: Mat4,
    pub(super) color_buffer: wgpu::Buffer,
    pub(super) uniform_buffer: wgpu::Buffer,
    pub(super) bind_group: wgpu::BindGroup,
    pub(super) kind: BodyKind,
    pub(super) motion: BodyMotion,
    pub(super) parent: Option<usize>
}

impl CelestialBody {
    pub(super) fn new(
        app: &wgpu_app::WgpuState,
        layout: &wgpu::BindGroupLayout,
        motion: BodyMotion,
        projection_view: Mat4,
        colors: &[[f32; 3]],
        kind: BodyKind,
        parent: Option<usize>
    ) -> Self {
        let model = motion.model_matrix();

        let uniforms = ObjectUniforms {
            model: model.to_cols_array_2d(),
            view_projection: projection_view.to_cols_array_2d(),
        };
        let uniform_buffer = app.device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: None,
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let color_buffer = app.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&colors),
            usage: match kind {
                BodyKind::Sun => wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                BodyKind::Planet | BodyKind::Moon => wgpu::BufferUsages::VERTEX
            }
        });
        let bind_group = app.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        CelestialBody { model, uniform_buffer, color_buffer, bind_group, kind, motion, parent }
    }
}

pub(super) enum BodyKind {
    Sun,
    Planet,
    Moon
}

pub(super) struct BodyMotion {
    pub(super) radius: f32,
    pub(super) orbit_radius: f32,
    pub(super) orbit_angle: f32,
    pub(super) orbit_speed: f32,
    pub(super) spin_angle: f32,
    pub(super) spin_speed: f32
}

impl BodyMotion {
    pub(super) fn model_matrix(&self) -> Mat4 {
        let position_at_orbit = self.orbit_position();
        let size_matrix = Mat4::from_scale(Vec3::splat(self.radius));
        let rotation_matrix = Mat4::from_rotation_y(self.spin_angle);
        let orbit_position_matrix = Mat4::from_translation(position_at_orbit);
        let model = orbit_position_matrix * rotation_matrix * size_matrix;

        model
    }

    pub(super) fn update(&mut self, dt: f32) {
        self.orbit_angle = (self.orbit_angle + self.orbit_speed * dt).rem_euclid(std::f32::consts::TAU);
        self.spin_angle = (self.spin_angle + self.spin_speed * dt).rem_euclid(std::f32::consts::TAU);
    }

    pub(super) fn orbit_position(&self) -> Vec3 {
        let pos = Vec3::new(self.orbit_radius * self.orbit_angle.cos(), 0.0, self.orbit_radius * self.orbit_angle.sin());
        pos
    }
}

pub(super) struct PlanetRing {
    pub(super) parent: usize,
    pub(super) scale: f32,
    pub(super) tilt: f32,
    pub(super) color_buffer: wgpu::Buffer,
    pub(super) uniform_buffer: wgpu::Buffer,
    pub(super) bind_group: wgpu::BindGroup,
}

impl PlanetRing {
    pub(super) fn model_matrix(&self, parent_position: Vec3) -> Mat4 {
        Self::calculate_model_matrix(parent_position, self.scale, self.tilt)
    }

    pub(super) fn calculate_model_matrix(parent_position: Vec3, scale: f32, tilt: f32) -> Mat4 {
        let scaled = Mat4::from_scale(Vec3::splat(scale));
        let rotated = Mat4::from_rotation_x(tilt);
        let translated = Mat4::from_translation(parent_position);

        translated * rotated * scaled
    }

    pub(super) fn new(
        app: &wgpu_app::WgpuState,
        layout: &wgpu::BindGroupLayout,
        parent: usize,
        parent_position: Vec3,
        scale: f32,
        tilt: f32,
        view_projection: Mat4,
        colors: &[[f32; 3]],
    ) -> Self {
        let model_matrix = Self::calculate_model_matrix(parent_position, scale, tilt);
        let uniforms = ObjectUniforms {
            model: model_matrix.to_cols_array_2d(),
            view_projection: view_projection.to_cols_array_2d(),
        };
        let uniform_buffer = app.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let color_buffer = app.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(colors),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let bind_group = app.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            parent,
            scale,
            tilt,
            color_buffer,
            uniform_buffer,
            bind_group,
        }
    }
}

pub(super) struct SunAnimation {
    pub(super) phase: f32,
    pub(super) vertex_phases: Vec<[f32; 3]>,
    pub(super) colors: Vec<[f32; 3]>
}

impl SunAnimation {
    pub(super) fn update(&mut self, dt: f32) {
        self.phase = (self.phase + 0.8 * dt).rem_euclid(TAU);
        let orange = Vec3::new(1.0, 0.25, 0.02);
        let yellow = Vec3::new(1.0, 0.9, 0.25);
        for i in 0..self.colors.len() {
            let [a, b, c] = self.vertex_phases[i];
            let c1 = (a + self.phase + 1.5 * (b - 2.0 * self.phase).sin()).sin();
            let c2 = (c + 3.0 * self.phase + (a - self.phase).sin()).sin();
            let t = 0.5 + 0.5 * (0.65 * c1 + 0.35 * c2);
            let color = orange.lerp(yellow, t);
            self.colors[i] = color.to_array();
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct ObjectUniforms {
    pub(super) model: [[f32; 4]; 4],
    pub(super) view_projection: [[f32; 4]; 4]
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct StarInstance {
    pub(super) direction: [f32; 3],
    pub(super) color: [f32; 3],
    pub(super) size: f32
}

impl StarInstance {
    pub(super) fn generate_stars(count: usize) -> Vec<StarInstance> {
        let mut stars: Vec<StarInstance> = Vec::new();
        for _ in 0..count {
            let h = rand::random_range(-1.0..1.0);
            let angle = rand::random_range(0.0..TAU);
            let r = (1.0f32 - h * h).sqrt();
            let direction = Vec3::new(r * angle.cos(), h, r * angle.sin()).to_array();
            let color = Vec3::new(0.65, 0.8, 1.0).lerp(Vec3::new(1.0, 0.85, 0.65), rand::random_range(0.0..1.0)).to_array();
            let size = rand::random_range(0.0015_f32..0.004_f32);
            let star = StarInstance { direction, color, size };
            stars.push(star);
        }
        stars
    }
}

