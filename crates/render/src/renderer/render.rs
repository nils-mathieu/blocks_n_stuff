use crate::data::RenderData;
use crate::{RenderTarget, Renderer};

impl Renderer {
    /// Renders to the provided [`RenderTarget`] using the provided [`RenderData`].
    pub fn render(&mut self, target: RenderTarget, data: &mut RenderData) {
        self.gpu.queue.write_buffer(
            &self.resources.frame_uniforms_buffer,
            0,
            bytemuck::bytes_of(&data.frame),
        );

        // Now that we have upload everything, we can start recording GPU commands.
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Command Encoder"),
            });

        let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Main Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
                resolve_target: None,
                view: target.view,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
                view: &self.resources.depth_buffer,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        // Set the bind groups that are used by most of the pipelines.
        rp.set_bind_group(0, &self.resources.frame_uniforms_bind_group, &[]);
        rp.set_bind_group(2, &self.resources.texture_atlas_bind_group, &[]);

        self.skybox_pipeline.render(&self.gpu, &mut rp);
        self.quad_pipeline.render(&self.gpu, &mut rp, &data.quads);
        self.line_pipeline.render(&self.gpu, &mut rp, &data.lines);

        // Now that everything is recorded, we can submit the commands to the GPU.
        drop(rp);

        let iter = self
            .gpu
            .iter_temp_command_encoders()
            .map(|e| {
                std::mem::replace(
                    &mut *e.lock(),
                    self.gpu
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Temporary Command Encoder"),
                        }),
                )
                .finish()
            })
            .chain(std::iter::once(encoder.finish()));

        self.gpu.queue.submit(iter);
    }
}
