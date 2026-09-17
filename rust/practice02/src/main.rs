use std::fs;
use std::path::Path;
use std::time::Instant;
use wgpu_app::{run, AppConfig, WgpuApp, WgpuState, KeyCode};

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

struct Practice02 {
    pipeline: wgpu::RenderPipeline,
    last_frame_start: Instant,
    time: f32,
    offset_x : f32,
    offset_y : f32,
}

impl WgpuApp for Practice02 {
    fn new(app: &WgpuState) -> Self {
        let shader = load_shader_module(&app.device, Path::new(PROJECT_ROOT).join("shader.wgsl"))
            .expect("failed to load shader");


        let pipeline_descriptor = wgpu::PipelineLayoutDescriptor{
            label: Some("practice02 layout"),
            bind_group_layouts: &[],
            immediate_size: 128,
        };
        let pipeline_layout = app.device.create_pipeline_layout(&pipeline_descriptor);
        let pipeline = app.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertexMain"),
                buffers: &[],
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
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        });

        Self { pipeline, last_frame_start: Instant::now(), time: 0.0, offset_x: 0.0, offset_y: 0.0 }
    }

    fn redraw(&mut self, app: &mut WgpuState) {
        let Some(surface_texture) = app.begin_frame() else {
            return;
        };

        let now = Instant::now();
        let dt = (now - self.last_frame_start).as_secs_f32();
        self.time += dt;
        self.last_frame_start = now;

        let target_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = app.device.create_command_encoder(&Default::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color{r: 0.07, g: 0.21, b: 0.30, a: 1.0}),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            render_pass.set_pipeline(&self.pipeline);
            let scale: f32 = 0.3;
            let angle = self.time * 0.5;
            let c = angle.cos();
            let s = angle.sin();
            // let radius: f32 = 0.4;
            // let phase = self.time;
            // let x = radius * phase.cos();
            // let y = radius * phase.sin();
            let speed = 0.5;

            if app.keydown.contains(&KeyCode::ArrowUp) {
                self.offset_y += speed * dt;
            }
            if app.keydown.contains(&KeyCode::ArrowDown) {
                self.offset_y -= speed * dt;
            }
            if app.keydown.contains(&KeyCode::ArrowLeft) {
                self.offset_x -= speed * dt;
            }
            if app.keydown.contains(&KeyCode::ArrowRight) {
                self.offset_x += speed * dt;
            }
            let x = self.offset_x;
            let y = self.offset_y;
            let transform : [f32; 16] = [
                scale * c, scale * s, 0.0, 0.0,
                -scale * s, scale * c, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                x, y, 0.0, 1.0,
            ];
            let aspect_ratio = app.width() as f32 / app.height() as f32;
            let view : [f32; 16] = [
                1.0 / aspect_ratio, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ];
            render_pass.set_immediates(0, bytemuck::bytes_of(&transform));
            render_pass.set_immediates(64, bytemuck::bytes_of(&view));
            render_pass.draw(0..12, 0..1);
        }

        app.queue.submit(std::iter::once(encoder.finish()));
        app.queue.present(surface_texture);
    }
}

fn main() {
    run::<Practice02>(AppConfig {
        title: "Practice02",
        width: 1280,
        height: 720,
        srgb: false,
    });
}
