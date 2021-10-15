use crate::graphics::{AnimationBuffer, AnimationRenderPipeline, TextureGroup};

pub trait RenderAnimations<'a, 'b: 'a> {
    fn render_animations(
        &mut self,
        buffer: &'b AnimationBuffer,
        texture_group: &'b TextureGroup,
        pipeline: &'b AnimationRenderPipeline,
    );
}

impl<'a, 'b: 'a> RenderAnimations<'a, 'b> for wgpu::RenderPass<'a> {
    fn render_animations(
        &mut self,
        buffer: &'b AnimationBuffer,
        texture_group: &'b TextureGroup,
        pipeline: &'b AnimationRenderPipeline,
    ) {
        self.set_bind_group(2, &texture_group.bind_group, &[]);
        self.set_vertex_buffer(0, buffer.vertex_buffer.slice(..buffer.vertex_count));
        self.set_index_buffer(
            buffer
                .indice_buffer
                .slice(..(buffer.indice_count * 4) as u64),
            wgpu::IndexFormat::Uint32,
        );
        self.set_pipeline(pipeline.render_pipeline());
        self.draw_indexed(0..buffer.indice_count as u32, 0, 0..1);
    }
}
