mod geometry;
mod scene;

use std::f32::consts::{PI, TAU};
use std::time::Instant;
use glam::{Mat4, Vec3};
use wgpu::util::{DeviceExt, RenderEncoder};
use wgpu_app::{run, AppConfig, WgpuApp};

use geometry::{SphereVertex, generate_icosahedron_positions, generate_icosahedron_indices, transform, generate_colors, random_color};
use scene::{Camera, CelestialBody, ObjectUniforms};
use crate::scene::{BodyKind, BodyMotion, PlanetRing, StarInstance, SunAnimation};

const PROJECT_ROOT: &str = env!("CARGO_MANIFEST_DIR");

struct Homework {
    pipeline: wgpu::RenderPipeline,
    sun_pipeline: wgpu::RenderPipeline,
    star_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    depth_buffer: wgpu::Texture,
    depth_buffer_view: wgpu::TextureView,
    ring_vertex_buffer: wgpu::Buffer,
    ring_index_buffer: wgpu::Buffer,
    ring_index_count: u32,
    bodies: Vec<CelestialBody>,
    rings: Vec<PlanetRing>,
    sun_animation: SunAnimation,
    glow_vertex_buffer: wgpu::Buffer,
    glow_index_buffer: wgpu::Buffer,
    glow_color_buffer: wgpu::Buffer,
    glow_index_count: u32,
    glow_uniform_buffer: wgpu::Buffer,
    glow_bind_group: wgpu::BindGroup,
    glow_pipeline: wgpu::RenderPipeline,
    star_instances_buffer: wgpu::Buffer,
    star_vertex_buffer: wgpu::Buffer,
    star_index_buffer: wgpu::Buffer,
    star_index_count: u32,
    star_instance_count: u32,
    star_uniform_buffer: wgpu::Buffer,
    star_bind_group: wgpu::BindGroup,
    index_count: u32,
    camera: Camera,
    last_frame_start: Instant,
    capture_on_start: bool,
    mouse_captured: bool
}

fn create_depth_buffer(app: &wgpu_app::WgpuState) -> (wgpu::Texture, wgpu::TextureView) {
    let buffer = wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d {
            width: app.width(),
            height: app.height(),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth24Plus,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };
    let texture = app.device.create_texture(&buffer);
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    (texture, view)
}

fn glow_model_matrix(
    camera_position: Vec3,
    sun_position: Vec3,
    sun_radius: f32
) -> Mat4 {
    let direction = camera_position - sun_position;

    let y_axis = direction.normalize();

    let reference = if y_axis.dot(Vec3::Y).abs() > 0.99 {
        Vec3::X
    } else {
        Vec3::Y
    };

    let x_axis = reference.cross(y_axis).normalize();
    let z_axis = x_axis.cross(y_axis);

    let transform = Mat4::from_cols(
        x_axis.extend(0.0),
        y_axis.extend(0.0),
        z_axis.extend(0.0),
        sun_position.extend(1.0),
    );

    transform * Mat4::from_scale(Vec3::splat(sun_radius))
}

impl WgpuApp for Homework {
    fn on_mousemove(&mut self, _app: &mut wgpu_app::WgpuState, dx: f32, dy: f32) {
        if !self.mouse_captured {
            return;
        }
        let sensivity = 0.0005_f32;
        self.camera.horizontal += dx * sensivity;
        self.camera.vertical -= dy * sensivity;
        let limit = std::f32::consts::FRAC_PI_2 - 0.01;
        self.camera.vertical = self.camera.vertical.clamp(-limit, limit);
    }

    fn on_keydown(&mut self, app: &mut wgpu_app::WgpuState, key: wgpu_app::KeyCode) {
        if key == wgpu_app::KeyCode::Escape {
            app.grab_mouse(false);
            self.mouse_captured = false;
        }
    }

    fn on_mousedown(&mut self, app: &mut wgpu_app::WgpuState, button: wgpu_app::MouseButton) {
        if button == wgpu_app::MouseButton::Left {
            app.grab_mouse(true);
            self.mouse_captured = true;
        }
    }

    fn new(app: &wgpu_app::WgpuState) -> Self {
        let subdivisions = 4;
        let mut positions = generate_icosahedron_positions();
        let mut indices = generate_icosahedron_indices(&positions);
        for point in &mut positions {
            *point = point.normalize();
        }
        for _ in 0..subdivisions {
            indices = transform(&mut positions, &indices);
        }

        let mut vertexes : Vec<SphereVertex> = Vec::with_capacity(positions.len());
        for position in positions {
            let vertex = SphereVertex {
                position: position.to_array(),
                normal: position.to_array(),
            };
            vertexes.push(vertex);
        }

        let camera = Camera {
            position: Vec3::new(0.0, 0.0, 300.0),
            horizontal: -std::f32::consts::FRAC_PI_2,
            vertical: 0.0,
        };
        let view = camera.view_matrix();
        let projection = glam::Mat4::perspective_rh(std::f32::consts::PI / 3.0,
                                                    app.width() as f32 / app.height() as f32,
        0.1, 1000.0);
        let projection_view = projection * view;

        let last_frame_start = Instant::now();

        let vertex_buffer = app.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertexes),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = app.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let (depth_buffer, depth_buffer_view) = create_depth_buffer(app);
        let sun_colors : Vec<[f32; 3]> = vec![[1.0, 0.81, 0.28]; vertexes.len()];
        let bind_group_layout = app.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let planet_specs: [(f32, f32, f32); 7] = [
            (24.0, 0.6, 1.0),
            (44.0, 0.8, 1.2),
            (68.0, 0.7, 1.3),
            (96.0, 1.5, 2.5),
            (128.0, 1.3, 2.2),
            (164.0, 1.0, 1.8),
            (204.0, 1.0, 1.8)
        ];

        let sun = CelestialBody::new(app, &bind_group_layout, BodyMotion {
            radius: 8.0,
            orbit_radius: 0.0,
            orbit_angle: 0.0,
            orbit_speed: 0.0,
            spin_angle: 0.0,
            spin_speed: TAU / 50.0,
        }, projection_view, &sun_colors, BodyKind::Sun, None);
        let mut vertex_phases : Vec<[f32; 3]> = Vec::with_capacity(vertexes.len());
        for vertex in &vertexes {
            let [x, y, z] = vertex.position;
            let a = 8.0 * x + 11.0 * y + 5.0 * z;
            let b = -13.0 * x + 7.0 * y + 9.0 * z;
            let c = 17.0 * x + 3.0 * y - 11.0 * z;
            vertex_phases.push([a, b, c]);
        }
        let sun_animation = SunAnimation {
            phase: 0.0,
            colors: sun_colors,
            vertex_phases
        };
        let mut bodies = vec![sun];
        for (orbit_radius, min_radius, max_radius) in planet_specs {
            let radius = rand::random_range(min_radius..max_radius);
            let orbit_angle = rand::random_range(0.0_f32..TAU);
            let period = rand::random_range(10.0_f32..30.0);
            let spin_speed = TAU / period;
            let orbit_speed = 0.35 * (24.0 / orbit_radius).powf(1.5);
            let colors = generate_colors(&vertexes, rand::random_range(5.0..100.0), random_color(), random_color());
            let body_motion = BodyMotion {
                radius,
                orbit_radius,
                orbit_angle,
                orbit_speed,
                spin_angle: 0.0,
                spin_speed,
            };
            let body = CelestialBody::new(
                &app, &bind_group_layout, body_motion, projection_view, &colors, BodyKind::Planet, Some(0)
            );
            let planet_index = bodies.len();
            bodies.push(body);

            let moon_count = rand::random_range(0_usize..=10);
            for j in 0..moon_count {
                let moon_radius = rand::random_range(0.05 * radius..0.12 * radius);
                let moon_orbit_angle = rand::random_range(0.0_f32..TAU);
                let moon_colors = generate_colors(&vertexes, rand::random_range(5.0..100.0), random_color(), random_color());
                let moon_orbit_radius = radius * (3.0 + 3.0 * (j as f32 + 0.5) / moon_count as f32);
                let moon_orbit_speed = 2.0 * PI / 6.0 * (3.0 * radius / moon_orbit_radius).powf(1.5);
                let moon_spin_angle = (PI - moon_orbit_angle).rem_euclid(TAU);
                let moon_spin_speed = -moon_orbit_speed;
                let moon_body_motion = BodyMotion {
                    radius: moon_radius,
                    orbit_radius: moon_orbit_radius,
                    orbit_angle: moon_orbit_angle,
                    orbit_speed: moon_orbit_speed,
                    spin_angle: moon_spin_angle,
                    spin_speed: moon_spin_speed,
                };
                let moon = CelestialBody::new(
                    &app, &bind_group_layout, moon_body_motion, projection_view, &moon_colors, BodyKind::Moon, Some(planet_index)
                );
                bodies.push(moon);
            }
        }

        let (ring_vertices, ring_indices) = geometry::generate_ring(1.0, 1.1, 128);
        let ring_vertex_buffer = app.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&ring_vertices),
            usage: wgpu::BufferUsages::VERTEX
        });
        let ring_index_buffer = app.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&ring_indices),
            usage: wgpu::BufferUsages::INDEX
        });

        let mut rings: Vec<PlanetRing> = Vec::new();
        for (parent_index, body) in bodies.iter().enumerate() {
            if let BodyKind::Planet = body.kind {
                let amount = rand::random_range(0_usize..=3);
                let tilt = rand::random_range(0.3_f32..0.9_f32);
                for j in 0..amount {
                    let scale = body.motion.radius * (1.4 + 0.35 * j as f32);
                    let colors = vec![random_color().to_array(); ring_vertices.len()];

                    let ring = PlanetRing::new(
                        app, &bind_group_layout, parent_index, body.motion.orbit_position(),
                        scale, tilt, projection_view, &colors
                    );
                    rings.push(ring);
                }
            }
        }

        let (glow_vertices, glow_indices) = geometry::generate_ring(0.95, 1.5, 128);
        let mut glow_colors: Vec<[f32; 3]> = Vec::with_capacity(glow_vertices.len());
        for i in 0..glow_vertices.len() {
            if i % 2 == 0 {
                glow_colors.push([0.8, 0.3, 0.05]);
            } else {
                glow_colors.push([0.0, 0.0, 0.0]);
            }
        }
        let glow_vertex_buffer = app.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&glow_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let glow_index_buffer = app.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&glow_indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let glow_color_buffer = app.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&glow_colors),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let glow_uniforms = ObjectUniforms {
            model: glow_model_matrix(camera.position, Vec3::ZERO, bodies[0].motion.radius).to_cols_array_2d(),
            view_projection: projection_view.to_cols_array_2d(),
        };
        let glow_uniform_buffer = app.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&glow_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let glow_bind_group = app.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: glow_uniform_buffer.as_entire_binding(),
            }],
        });

        let stars = StarInstance::generate_stars(2000);
        let star_instances_buffer = app.device.create_buffer_init(&wgpu::util::BufferInitDescriptor{
            label: None,
            contents: bytemuck::cast_slice(&stars),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let stars_positions = generate_icosahedron_positions();
        let stars_indices = generate_icosahedron_indices(&stars_positions);
        let mut star_vertices: Vec<[f32; 3]> = Vec::with_capacity(stars_positions.len());
        for p in &stars_positions {
            star_vertices.push(p.normalize().to_array());
        }
        let star_vertex_buffer = app.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&star_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let star_index_buffer = app.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&stars_indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let star_view = Mat4::look_to_rh(
            Vec3::ZERO,
            camera.forward(),
            Vec3::Y
        );
        let star_uniforms = ObjectUniforms {
            model: Mat4::IDENTITY.to_cols_array_2d(),
            view_projection: (projection * star_view).to_cols_array_2d()
        };
        let star_uniform_buffer = app.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&star_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let star_bind_group = app.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: star_uniform_buffer.as_entire_binding(),
            }],
        });

        let path = format!("{}/shader.wgsl", PROJECT_ROOT);
        let shader_code = std::fs::read_to_string(&path).expect("Не удалось прочитать шейдер");
        let shader_descriptor = wgpu::ShaderModuleDescriptor{ label: Some("Practice01 shader"), source: wgpu::ShaderSource::Wgsl(shader_code.into())};
        let shader_module = app.device.create_shader_module(shader_descriptor);

        let vertex = wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vertexMain"),
            compilation_options: Default::default(),
            buffers: &[Some(wgpu::VertexBufferLayout {
                array_stride: size_of::<SphereVertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 1,
                }],
            }),
            Some(wgpu::VertexBufferLayout {
                array_stride: size_of::<[f32; 3]>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 2,
                }
                ],
            })],
        };
        let color_target = wgpu::ColorTargetState{
            format: app.surface_format(),
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        };
        let targets = [Some(color_target)];
        let fragment = wgpu::FragmentState{
            module: &shader_module,
            entry_point: Some("fragmentMain"),
            compilation_options: Default::default(),
            targets: &targets,
        };
        let pipeline_layout = app.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let mut pipeline_descriptor = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex,
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(fragment),
            multiview_mask: None,
            cache: None,
        };
        let planets_pipeline = app.device.create_render_pipeline(&pipeline_descriptor);

        if let Some(fragment) = pipeline_descriptor.fragment.as_mut() {
            fragment.entry_point = Some("fragmentSun");
        }
        let sun_pipeline = app.device.create_render_pipeline(&pipeline_descriptor);

        let glow_targets = [Some(wgpu::ColorTargetState{
            format: app.surface_format(),
            blend: Some(wgpu::BlendState{
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::Zero,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
            write_mask: wgpu::ColorWrites::ALL,
        })];
        if let Some(fragment) = pipeline_descriptor.fragment.as_mut() {
            fragment.targets = &glow_targets;
        }
        if let Some(depth) = pipeline_descriptor.depth_stencil.as_mut() {
            depth.depth_write_enabled = Some(false);
            depth.depth_compare = Some(wgpu::CompareFunction::LessEqual);
        }
        let glow_pipeline = app.device.create_render_pipeline(&pipeline_descriptor);

        let star_buffer_layouts: [Option<wgpu::VertexBufferLayout>; 2] = [
            Some(wgpu::VertexBufferLayout {
                array_stride: size_of::<[f32; 3]>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                }],
            }),
            Some(wgpu::VertexBufferLayout {
                array_stride: size_of::<StarInstance>() as u64,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &[wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 1,
                }, wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 2,
                }, wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32,
                    offset: 24,
                    shader_location: 3,
                }],
            })
        ];
        if let Some(fragment) = pipeline_descriptor.fragment.as_mut() {
            fragment.entry_point = Some("fragmentStars");
            fragment.targets = &targets;
        }
        pipeline_descriptor.vertex = wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vertexStars"),
            compilation_options: Default::default(),
            buffers: &star_buffer_layouts,
        };
        pipeline_descriptor.depth_stencil = None;
        let star_pipeline = app.device.create_render_pipeline(&pipeline_descriptor);

        Self{ pipeline: planets_pipeline, sun_pipeline, vertex_buffer, index_buffer,
            index_count: indices.len() as u32, depth_buffer, bodies,
            depth_buffer_view, camera, last_frame_start, capture_on_start: true, mouse_captured: false,
            ring_vertex_buffer, ring_index_buffer, ring_index_count: ring_indices.len() as u32, rings,
            sun_animation, star_instances_buffer, star_vertex_buffer, star_index_buffer,
            star_index_count: stars_indices.len() as u32, star_instance_count: stars.len() as u32,
            star_uniform_buffer, star_bind_group, star_pipeline,
            glow_vertex_buffer, glow_index_buffer, glow_color_buffer, glow_index_count: glow_indices.len() as u32,
            glow_uniform_buffer, glow_bind_group, glow_pipeline}
    }

    fn redraw(&mut self, app: &mut wgpu_app::WgpuState) {
        let Some(surface_texture) = app.begin_frame() else {
            return;
        };

        if self.capture_on_start {
            app.grab_mouse(true);
            self.mouse_captured = true;
            self.capture_on_start = false;
        }

        let now = std::time::Instant::now();
        let dt = (now - self.last_frame_start).as_secs_f32();
        self.last_frame_start = now;
        let forward_direction = self.camera.forward();
        let right_direction = forward_direction.cross(Vec3::Y).normalize_or_zero();
        let mut move_direction = Vec3::ZERO;
        let speed = 100.0;
        if app.keydown.contains(&wgpu_app::KeyCode::KeyW) {
            move_direction += forward_direction;
        }
        if app.keydown.contains(&wgpu_app::KeyCode::KeyS) {
            move_direction -= forward_direction;
        }
        if app.keydown.contains(&wgpu_app::KeyCode::KeyA) {
            move_direction -= right_direction;
        }
        if app.keydown.contains(&wgpu_app::KeyCode::KeyD) {
            move_direction += right_direction;
        }
        self.camera.position += move_direction.normalize_or_zero() * speed * dt;

        let view = self.camera.view_matrix();
        let projection = glam::Mat4::perspective_rh(
            std::f32::consts::PI / 3.0,
            app.width() as f32 / app.height() as f32,
            0.1, 1000.0
        );

        let star_view = Mat4::look_to_rh(
            Vec3::ZERO,
            self.camera.forward(),
            Vec3::Y
        );
        let star_uniforms = ObjectUniforms {
            model: Mat4::IDENTITY.to_cols_array_2d(),
            view_projection: (projection * star_view).to_cols_array_2d()
        };
        app.queue.write_buffer(&self.star_uniform_buffer, 0, bytemuck::bytes_of(&star_uniforms));

        let mut world_positions: Vec<Vec3> = Vec::with_capacity(self.bodies.len());
        for body in &mut self.bodies {
            body.motion.update(dt);
            let parent_position = match body.parent {
                None => Vec3::ZERO,
                Some(index) => world_positions[index]
            };
            let world_position = parent_position + body.motion.orbit_position();
            world_positions.push(world_position);
            body.model = Mat4::from_translation(parent_position) * body.motion.model_matrix();
            let uniforms = ObjectUniforms {
                model: body.model.to_cols_array_2d(),
                view_projection: (projection * view).to_cols_array_2d(),
            };
            app.queue.write_buffer(&body.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
        }

        let glow_uniforms = ObjectUniforms {
            model: glow_model_matrix(self.camera.position, world_positions[0], self.bodies[0].motion.radius).to_cols_array_2d(),
            view_projection: (projection * view).to_cols_array_2d(),
        };
        app.queue.write_buffer(&self.glow_uniform_buffer, 0, bytemuck::bytes_of(&glow_uniforms));

        self.sun_animation.update(dt);
        app.queue.write_buffer(&self.bodies[0].color_buffer, 0, bytemuck::cast_slice(&self.sun_animation.colors));

        for ring in &mut self.rings {
            let center = world_positions[ring.parent];
            let model_matrix = ring.model_matrix(center);
            let uniforms = ObjectUniforms {
                model: model_matrix.to_cols_array_2d(),
                view_projection: (projection * view).to_cols_array_2d()
            };
            app.queue.write_buffer(&ring.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
        }

        let target_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let descriptor = wgpu::CommandEncoderDescriptor::default();
        let mut encoder = app.device.create_command_encoder(&descriptor);
        let background = wgpu::Color{ r: 0.0, g: 0.0, b: 0.0, a: 1.0 };

        {
            let star_color_attachment = wgpu::RenderPassColorAttachment {
                view: &target_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(background),
                    store: wgpu::StoreOp::Store,
                },
            };
            let star_settings = wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(star_color_attachment)],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            };
            let mut render_pass = encoder.begin_render_pass(&star_settings);
            render_pass.set_pipeline(&self.star_pipeline);
            render_pass.set_bind_group(0, &self.star_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.star_vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.star_instances_buffer.slice(..));
            render_pass.set_index_buffer(self.star_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(
                0..self.star_index_count,
                0,
                0..self.star_instance_count,
            )
        }

        let color_attachment = wgpu::RenderPassColorAttachment{view: &target_view, depth_slice: None, resolve_target: None, ops: wgpu::Operations{ load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }};
        let attachments = [Some(color_attachment)];

        if self.depth_buffer.width() != app.width() || self.depth_buffer.height() != app.height() {
            let (new_depth_buffer, new_depth_buffer_view) = create_depth_buffer(app);
            self.depth_buffer = new_depth_buffer;
            self.depth_buffer_view = new_depth_buffer_view;
        }

        let settings = wgpu::RenderPassDescriptor { color_attachments: &attachments,
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_buffer_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Discard,
                }),
                stencil_ops: None,
            })
            ,..Default::default() };

        {
            let mut render_pass = encoder.begin_render_pass(&settings);
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            for body in &self.bodies {
                match body.kind {
                    BodyKind::Sun => {
                        render_pass.set_pipeline(&self.sun_pipeline);
                    }
                    BodyKind::Planet | BodyKind::Moon => {
                        render_pass.set_pipeline(&self.pipeline);
                    }
                }
                render_pass.set_bind_group(0, &body.bind_group, &[]);
                render_pass.set_vertex_buffer(1, body.color_buffer.slice(..));
                render_pass.draw_indexed(0..self.index_count, 0, 0..1);
            }

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.ring_vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.ring_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            for ring in &self.rings {
                render_pass.set_bind_group(0, &ring.bind_group, &[]);
                render_pass.set_vertex_buffer(1, ring.color_buffer.slice(..));
                render_pass.draw_indexed(0..self.ring_index_count, 0, 0..1);
            }
            render_pass.set_pipeline(&self.glow_pipeline);
            render_pass.set_bind_group(0, &self.glow_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.glow_vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.glow_color_buffer.slice(..));
            render_pass.set_index_buffer(self.glow_index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..self.glow_index_count, 0, 0..1);
        }
        let command_buffer = encoder.finish();

        app.queue.submit([command_buffer]);

        app.queue.present(surface_texture);
    }
}

fn main() {
    run::<Homework>(AppConfig {
        title: "Homework",
        width: 1980,
        height: 1080,
        srgb: false,
    });
}
