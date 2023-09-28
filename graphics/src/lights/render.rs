use crate::{
    AscendingError, GpuRenderer, InstanceBuffer, LightLayout,
    LightRenderPipeline, Lights, LightsVertex, OrderedIndex,
    StaticBufferObject,
};

use wgpu::util::DeviceExt;

pub struct LightRenderer {
    pub buffer: InstanceBuffer<LightsVertex>,
    area_buffer: wgpu::Buffer,
    dir_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl LightRenderer {
    pub fn new(renderer: &mut GpuRenderer) -> Result<Self, AscendingError> {
        let area_buffer = renderer.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Area Light buffer"),
                contents: &[0; 40000], //2000
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            },
        );

        let dir_buffer = renderer.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Directional Light buffer"),
                contents: &[0; 64000], //2000
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            },
        );

        // Create the bind group layout for the camera.
        let layout = renderer.create_layout(LightLayout);

        // Create the bind group.
        let bind_group =
            renderer
                .device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: area_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: dir_buffer.as_entire_binding(),
                        },
                    ],
                    label: Some("lights_bind_group"),
                });

        Ok(Self {
            buffer: InstanceBuffer::new(renderer.gpu_device()),
            dir_buffer,
            area_buffer,
            bind_group,
        })
    }

    pub fn add_buffer_store(
        &mut self,
        renderer: &GpuRenderer,
        index: OrderedIndex,
    ) {
        self.buffer.add_buffer_store(renderer, index);
    }

    pub fn finalize(&mut self, renderer: &mut GpuRenderer) {
        self.buffer.finalize(renderer)
    }

    pub fn lights_update(
        &mut self,
        lights: &mut Lights,
        renderer: &mut GpuRenderer,
    ) {
        let index = lights.update(
            renderer,
            &mut self.area_buffer,
            &mut self.dir_buffer,
        );

        self.add_buffer_store(renderer, index);
    }
}

pub trait RenderLights<'a, 'b>
where
    'b: 'a,
{
    fn render_lights(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b LightRenderer,
    );
}

impl<'a, 'b> RenderLights<'a, 'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn render_lights(
        &mut self,
        renderer: &'b GpuRenderer,
        buffer: &'b LightRenderer,
    ) {
        if buffer.buffer.count() > 0 {
            self.set_bind_group(1, &buffer.bind_group, &[]);
            self.set_vertex_buffer(1, buffer.buffer.instances(None));
            self.set_pipeline(
                renderer.get_pipelines(LightRenderPipeline).unwrap(),
            );

            self.draw_indexed(
                0..StaticBufferObject::index_count(),
                0,
                0..buffer.buffer.count(),
            );
        }
    }
}