use std::mem::size_of_val;

use wgpu::util::DeviceExt;

use crate::data::RenderData;
use crate::{RenderTarget, Renderer};

impl Renderer {
    /// Renders to the provided [`RenderTarget`] using the provided [`RenderData`].
    pub fn render(&mut self, target: RenderTarget, data: &mut RenderData) {
        // The first step is to upload the data that we have to the GPU.
        // This data is expected to change on every frame, so we can't just upload it once and be
        // done with it.
        self.gpu.queue.write_buffer(
            &self.frame_uniforms_buffer,
            0,
            bytemuck::bytes_of(&data.frame_uniforms),
        );

        let prepared_quad = self.quad_pipeline.prepare(&self.gpu, &data.quads);

        // Upload the line instances to the GPU.
        if self.line_instances.size() < size_of_val(&data.line_instances) as u64 {
            self.line_instances =
                self.gpu
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        contents: bytemuck::cast_slice(&data.line_instances),
                        label: Some("Line Vertices Buffer"),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    });
        } else {
            self.gpu.queue.write_buffer(
                &self.line_instances,
                0,
                bytemuck::cast_slice(&data.line_instances),
            );
        }

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
                view: &self.depth_buffer,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        // Set the bind groups that are used by most of the pipelines.
        rp.set_bind_group(0, &self.frame_uniforms_bind_group, &[]);
        rp.set_bind_group(2, &self.texture_atlas_bind_group, &[]);

        // The skybox pipeline is responsible for rendering the skybox.
        // This is the first thing that we render.
        rp.set_pipeline(&self.skybox_pipeline);
        rp.draw(0..4, 0..1);

        self.quad_pipeline.render(&mut rp, prepared_quad);

        // The line pipeline is responsible for drawing lines in world-space.
        if !data.line_instances.is_empty() {
            rp.set_pipeline(&self.line_pipeline);
            rp.set_vertex_buffer(
                0,
                self.line_instances
                    .slice(..size_of_val(&data.line_instances) as u64),
            );
            rp.draw(0..4, 0..data.line_instances.len() as u32);
        }

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
