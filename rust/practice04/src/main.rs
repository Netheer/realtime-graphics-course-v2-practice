use std::fs;
use std::path::Path;
use std::time::Instant;
use glam::Mat4;
use wgpu_app::{run, load_obj, ObjVertex, ObjMesh, AppConfig, WgpuApp, WgpuState, KeyCode};

const PROJECT_ROOT: &str = env!("CARGO_MANIFEST_DIR");

fn load_shader_module(
    device: &wgpu::Device,
    path: impl AsRef<Path>,
) -> std::io::Result<wgpu::ShaderModule> {
    let source = fs::read_to_string(path)?;
    Ok(device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(source.into()),
    }))
}

fn depth_attachment(app: &WgpuState) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = app.device.create_texture(&wgpu::TextureDescriptor {
        label: None,
        size: wgpu::Extent3d { width: app.width(), height: app.height(), depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth24Plus,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: None,
        format: Some(wgpu::TextureFormat::Depth24Plus),
        dimension: Some(wgpu::TextureViewDimension::D2),
        usage: Some(wgpu::TextureUsages::RENDER_ATTACHMENT),
        aspect: wgpu::TextureAspect::DepthOnly,
        base_mip_level: 0,
        mip_level_count: Some(1),
        base_array_layer: 0,
        array_layer_count: Some(1),
    });
    return (texture, view);
}

struct Practice04 {
    pipeline: wgpu::RenderPipeline,
    last_frame_start: Instant,
    time: f32,
    bunny: ObjMesh,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    depth_buffer: wgpu::Texture,
    depth_buffer_view: wgpu::TextureView,
    bunny_x: f32,
    bunny_y: f32,
    bunny2_x: f32,
    bunny2_y: f32,
    bunny3_x: f32,
    bunny3_y: f32,
}

impl WgpuApp for Practice04 {
    fn new(app: &WgpuState) -> Self {
        let shader = load_shader_module(&app.device, Path::new(PROJECT_ROOT).join("shader.wgsl"))
            .expect("failed to load shader");

        let pipeline_layout = app.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            immediate_size: 128,
            ..Default::default()
        });

        let pipeline = app.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertexMain"),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<ObjVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: std::mem::offset_of!(ObjVertex, position) as u64,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: std::mem::offset_of!(ObjVertex, normal) as u64,
                            shader_location: 1,
                        }
                    ]
                })],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragmentMain"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: app.surface_format(),
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState{
                format: wgpu::TextureFormat::Depth24Plus,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Less),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });

        let bunny = load_obj(Path::new(PROJECT_ROOT).join("bunny.obj")).unwrap();
        let bunny_x = 0.0;
        let bunny_y = 0.0;
        let bunny2_x = 0.7;
        let bunny2_y = 0.3;
        let bunny3_x = -0.4;
        let bunny3_y = -0.8;

        let vertex_buffer_size = bunny.vertices.len() * std::mem::size_of::<ObjVertex>();
        let vertex_buffer = app.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: vertex_buffer_size as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        app.queue.write_buffer(&vertex_buffer, 0, bytemuck::cast_slice(&bunny.vertices));

        let index_buffer_size = bunny.indices.len() * std::mem::size_of::<u32>();
        let index_buffer = app.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: index_buffer_size as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        app.queue.write_buffer(&index_buffer, 0, bytemuck::cast_slice(&bunny.indices));

        let (depth_buffer, depth_buffer_view) = depth_attachment(app);

        Self { pipeline, last_frame_start: Instant::now(), time: 0.0, bunny, vertex_buffer, index_buffer, depth_buffer, depth_buffer_view, bunny_x, bunny_y, bunny2_x, bunny2_y, bunny3_x, bunny3_y }
    }

    fn redraw(&mut self, app: &mut WgpuState) {
        let Some(surface_texture) = app.begin_frame() else {
            return;
        };

        let now = Instant::now();
        let dt = (now - self.last_frame_start).as_secs_f32();
        self.time += dt;
        self.last_frame_start = now;
        let angle = self.time;
        let s = angle.sin();
        let c = angle.cos();
        let scale = 0.5;

        let speed = 1.5;
        if app.keydown.contains(&KeyCode::ArrowLeft) {
            self.bunny_x -= speed * dt;
        }
        if app.keydown.contains(&KeyCode::ArrowRight) {
            self.bunny_x += speed * dt;
        }
        if app.keydown.contains(&KeyCode::ArrowUp) {
            self.bunny_y += speed * dt;
        }
        if app.keydown.contains(&KeyCode::ArrowDown) {
            self.bunny_y -= speed * dt;
        }

        if app.keydown.contains(&KeyCode::KeyA) {
            self.bunny2_x -= speed * dt;
        }
        if app.keydown.contains(&KeyCode::KeyD) {
            self.bunny2_x += speed * dt;
        }
        if app.keydown.contains(&KeyCode::KeyW) {
            self.bunny2_y += speed * dt;
        }
        if app.keydown.contains(&KeyCode::KeyS) {
            self.bunny2_y -= speed * dt;
        }

        let model_matrix = Mat4::from_cols_array_2d(&[
            [scale * c, 0.0, -scale * s, 0.0],
            [0.0, scale, 0.0, 0.0],
            [scale * s, 0.0, scale * c, 0.0],
            [self.bunny_x, self.bunny_y, 0.0, 1.0],
        ]);

        let bunny2_matrix = Mat4::from_cols_array_2d(&[
            [scale * c, 0.0, scale * s, 0.0],
            [0.0, scale, 0.0, 0.0],
            [-scale * s, 0.0, scale * c, 0.0],
            [self.bunny2_x, self.bunny2_y, -1.0, 1.0]
        ]);
        let turnX_matrix = Mat4::from_cols_array_2d(&[
            [0.7, 0.0, 0.0, 0.0],
            [0.0, c * 0.7, s * 0.7, 0.0],
            [0.0, -s * 0.7, c * 0.7, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);
        let turnZ_matrix = Mat4::from_cols_array_2d(&[
            [c, -s, 0.0, 0.0],
            [s, c, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]);

        let bunny3_matrix = Mat4::from_cols_array_2d(&[
            [scale, 0.0, 0.0, 0.0],
            [0.0, scale * c, scale * s, 0.0],
            [0.0, -scale * s, scale * c, 0.0],
            [self.bunny3_x, self.bunny3_y, -5.0, 1.0]
        ]);

        let z_shift = 1.5;
        let view_matrix = Mat4::from_cols_array_2d(&[
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, -z_shift, 1.0],
        ]);

        let near = 0.1;
        let far = 100.0;
        let ratio = app.width() as f32 / app.height() as f32;
        let top = near / ratio;
        let x_scale = 1.0;
        let y_scale = near / top;
        let depth_scale = far / (near - far);
        let depth_offset = near * far / (near - far);
        let projection_matrix = Mat4::from_cols_array_2d(&[
            [x_scale, 0.0, 0.0, 0.0],
            [0.0, y_scale, 0.0, 0.0],
            [0.0, 0.0, depth_scale, -1.0],
            [0.0, 0.0, depth_offset, 0.0],
        ]);

        let view_projection_matrix = projection_matrix * view_matrix;

        let target_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        if (self.depth_buffer.width() != app.width() || self.depth_buffer.height() != app.height()) {
            let (new_depth_buffer, new_depth_buffer_view) = depth_attachment(app);
            self.depth_buffer = new_depth_buffer;
            self.depth_buffer_view = new_depth_buffer_view;
        }

        let mut encoder = app.device.create_command_encoder(&Default::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color{r: 0.01, g: 0.02, b: 0.03, a: 1.0}),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_buffer_view,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Discard }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_immediates(64, bytemuck::bytes_of(&view_projection_matrix));
            render_pass.set_immediates(0, bytemuck::bytes_of(&model_matrix));
            render_pass.draw_indexed(0..self.bunny.indices.len() as u32, 0, 0..1);
            render_pass.set_immediates(0, bytemuck::bytes_of(&(bunny2_matrix * turnX_matrix * turnZ_matrix)));
            render_pass.draw_indexed(0..self.bunny.indices.len() as u32, 0, 0..1);
            render_pass.set_immediates(0, bytemuck::bytes_of(&bunny3_matrix));
            render_pass.draw_indexed(0..self.bunny.indices.len() as u32, 0, 0..1);
        }

        app.queue.submit(std::iter::once(encoder.finish()));
        app.queue.present(surface_texture);
    }
}

fn main() {
    run::<Practice04>(AppConfig {
        title: "Practice04",
        width: 1280,
        height: 720,
        srgb: true,
    });
}
