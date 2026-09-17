use wgpu_app::{run, AppConfig, WgpuApp, WgpuState};

const PROJECT_ROOT: &str = env!("CARGO_MANIFEST_DIR");

struct Practice01 {
    pipeline: wgpu::RenderPipeline,
}

impl WgpuApp for Practice01 {
    fn new(app: &WgpuState) -> Self {
        let path = format!("{}/shader.wgsl", PROJECT_ROOT);
        let shader_code = std::fs::read_to_string(&path).expect("Не удалось прочитать шейдер");
        let shader_descriptor = wgpu::ShaderModuleDescriptor{ label: Some("Practice01 shader"), source: wgpu::ShaderSource::Wgsl(shader_code.into())};
        let shader_module = app.device.create_shader_module(shader_descriptor);

        let vertex = wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vertexMain"),
            compilation_options: Default::default(),
            buffers: &[],
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
        let pipeline_descriptor = wgpu::RenderPipelineDescriptor {
            label: Some("Practice01 pipeline"),
            layout: None,
            vertex,
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(fragment),
            multiview_mask: None,
            cache: None,
        };

        let pipeline = app.device.create_render_pipeline(&pipeline_descriptor);
        Self{ pipeline }
    }

    fn redraw(&mut self, app: &mut WgpuState) {
        let Some(surface_texture) = app.begin_frame() else {
            return;
        };

        let target_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let descriptor = wgpu::CommandEncoderDescriptor::default();
        let mut encoder = app.device.create_command_encoder(&descriptor);
        let background = wgpu::Color{ r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
        let color_attachment = wgpu::RenderPassColorAttachment{view: &target_view, depth_slice: None, resolve_target: None, ops: wgpu::Operations{ load: wgpu::LoadOp::Clear(background), store: wgpu::StoreOp::Store }};
        let attachments = [Some(color_attachment)];
        let settings = wgpu::RenderPassDescriptor { color_attachments: &attachments, ..Default::default() };
        {
            let mut render_pass = encoder.begin_render_pass(&settings);
            render_pass.set_pipeline(&self.pipeline);
            render_pass.draw(0..3, 0..1);
        }
        let command_buffer = encoder.finish();

        app.queue.submit([command_buffer]);

        app.queue.present(surface_texture);
    }
}

fn main() {
    run::<Practice01>(AppConfig {
        title: "Practice01",
        width: 1280,
        height: 720,
        srgb: false,
    });
}
