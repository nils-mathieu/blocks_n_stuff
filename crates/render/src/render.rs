use std::mem::size_of_val;

use wgpu::util::DeviceExt;

use crate::data::RenderData;
use crate::{RenderTarget, Renderer};

impl Renderer {
    /// Renders to the provided [`RenderTarget`] using the provided [`RenderData`].
    pub fn render(&mut self, target: RenderTarget, data: RenderData) {
        // The first step is to upload the data that we have to the GPU.
        // This data is expected to change on every frame, so we can't just upload it once and be
        // done with it.
        self.gpu.queue.write_buffer(
            &self.frame_uniforms_buffer,
            0,
            bytemuck::bytes_of(&data.frame_uniforms),
        );

        // The number of chunks visible from the camera is expected to change roughly every frame.
        // Detecting whether chunks have changed is not trivial anyway, so just re-upload all this
        // data is probably the best option.
        // Because this number can change, we need to make sure that the buffer is big enough to
        // store all the data.
        if self.chunk_uniforms_buffer.size() < data.chunk_uniforms.len() as u64 {
            (self.chunk_uniforms_buffer, self.chunk_uniforms_bind_group) =
                super::create_chunks_uniforms_buffer(
                    &self.gpu,
                    &self.chunk_uniforms_layout,
                    data.chunk_uniforms.len() as wgpu::BufferAddress,
                    self.chunk_uniforms_alignment,
                );
        }

        self.gpu.queue.write_buffer(
            &self.chunk_uniforms_buffer,
            0,
            bytemuck::cast_slice(data.chunk_uniforms),
        );

        // Upload the line vertices to the GPU.
        if self.line_vertices.size() < size_of_val(&**data.line_vertices) as u64 {
            self.line_vertices =
                self.gpu
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        contents: bytemuck::cast_slice(data.line_vertices),
                        label: Some("Line Vertices Buffer"),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    });
        } else {
            self.gpu.queue.write_buffer(
                &self.line_vertices,
                0,
                bytemuck::cast_slice(data.line_vertices),
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

        // The quad pipeline is responsible for rendering axis-aligned quads (the faces of voxels).
        // Every time a new instance buffer is drawn, the pipeline must be bound to the correct
        // chunk uniform (using the dynamic offset of the bind group).
        rp.set_pipeline(&self.quad_pipeline);
        for quad_vertices in data.quad_vertices.iter() {
            rp.set_vertex_buffer(0, quad_vertices.vertices);
            rp.set_bind_group(
                1,
                &self.chunk_uniforms_bind_group,
                &[quad_vertices.chunk_index * self.chunk_uniforms_alignment],
            );
            rp.draw(0..4, 0..quad_vertices.len);
        }

        // The line pipeline is responsible for drawing lines in world-space.
        if !data.line_vertices.is_empty() {
            rp.set_pipeline(&self.line_pipeline);
            rp.set_vertex_buffer(
                0,
                self.line_vertices
                    .slice(..size_of_val(&**data.line_vertices) as u64),
            );
            rp.draw(0..data.line_vertices.len() as u32, 0..1)
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
