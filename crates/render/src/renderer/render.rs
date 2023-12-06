use crate::data::{RenderData, Ui};
use crate::{RenderTarget, Renderer};

impl Renderer {
    /// Renders to the provided [`RenderTarget`] using the provided [`RenderData`].
    #[profiling::function]
    pub fn render(&mut self, target: RenderTarget, data: &mut RenderData) {
        let res = self.gpu.resources.read();

        self.gpu.queue.write_buffer(
            &res.frame_uniforms_buffer,
            0,
            bytemuck::bytes_of(&data.uniforms),
        );

        // Now that we have upload everything, we can start recording GPU commands.
        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Command Encoder"),
            });

        // ========================================
        // Base Scene
        // ========================================

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
                view: &res.depth_buffer,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        // Set the bind groups that are used by most of the pipelines.
        rp.set_bind_group(0, &res.frame_uniforms_bind_group, &[]);
        rp.set_bind_group(2, &res.texture_atlas_bind_group, &[]);

        self.skybox_pipeline.render(&self.gpu, &mut rp);
        self.quad_pipeline.render(&self.gpu, &mut rp, &data.quads);
        self.line_pipeline.render(&self.gpu, &mut rp, &data.lines);

        drop(rp);

        // ========================================
        // Post Processing
        // ========================================

        let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Post Processing Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                resolve_target: None,
                view: target.view,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        rp.set_bind_group(0, &res.frame_uniforms_bind_group, &[]);
        rp.set_bind_group(1, &res.depth_buffer_bind_group, &[]);

        if data.fog_enabled {
            self.fog_pipeline.render(&self.gpu, &mut rp);
        }

        drop(rp);

        // ========================================
        // UI
        // ========================================

        let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("UI Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                resolve_target: None,
                view: target.view,
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        rp.set_bind_group(0, &res.frame_uniforms_bind_group, &[]);

        for elem in &data.ui {
            match elem {
                Ui::Text(data) => {
                    self.text_pipeline.render(&self.gpu, &mut rp, *data);
                }
                Ui::Sprite { instances, texture } => {
                    self.ui_sprite_pipeline.render(&mut rp, *instances, texture);
                }
            }
        }

        drop(rp);

        // ========================================
        // Submit
        // ========================================

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
