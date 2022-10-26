use bytemuck::{Pod, Zeroable};
use camera::Projection;
use crevice::std140::{AsStd140, Std140};
use input::FrameTime;
use ultraviolet::Mat4;
use wgpu::util::DeviceExt;

pub(crate) use super::{Layout, LayoutStorage, Renderer};

#[repr(C)]
#[derive(Clone, Copy, Hash, Pod, Zeroable)]
pub struct SystemLayout;

impl Layout for SystemLayout {
    fn create_layout(&self, device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("system_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX
                        | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX
                        | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX
                        | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        })
    }
}

#[derive(AsStd140)]
pub struct CameraUniform {
    view: mint::ColumnMatrix4<f32>,
    proj: mint::ColumnMatrix4<f32>,
    eye: mint::Vector3<f32>,
}

#[derive(AsStd140)]
pub struct TimeUniform {
    //seconds since the start of the program. given by the FrameTime
    seconds: f32,
}

#[derive(AsStd140)]
pub struct TextColoredUniform {
    //which texture array to use for the glyph.
    colored: u32,
}

pub struct System<Controls: camera::controls::Controls> {
    camera: camera::Camera<Controls>,
    camera_buffer: wgpu::Buffer,
    time_buffer: wgpu::Buffer,
    text_colored_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl<Controls> System<Controls>
where
    Controls: camera::controls::Controls,
{
    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    pub fn controls(&self) -> &Controls {
        self.camera.controls()
    }

    pub fn controls_mut(&mut self) -> &mut Controls {
        self.camera.controls_mut()
    }

    pub fn eye(&self) -> [f32; 3] {
        self.camera.eye()
    }

    pub fn new(
        renderer: &Renderer,
        layout_storage: &mut LayoutStorage,
        projection: Projection,
        controls: Controls,
    ) -> Self {
        let mut camera = camera::Camera::new(projection, controls);

        // FIXME: think more about the initial state of the camera.
        // Update the camera.
        camera.update(0.0);

        // Create the camera uniform.
        let proj = camera.projection();
        let view = camera.view();
        let eye: mint::Vector3<f32> = camera.eye().into();

        let camera_info = CameraUniform { view, proj, eye };
        let time_info = TimeUniform { seconds: 0.0 };
        let colored_info = TextColoredUniform { colored: 0 };

        // Create the uniform buffers.
        let camera_buffer = renderer.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("camera buffer"),
                contents: camera_info.as_std140().as_bytes(),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            },
        );

        let time_buffer = renderer.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("time buffer"),
                contents: time_info.as_std140().as_bytes(),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            },
        );

        let text_colored_buffer = renderer.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("text colored buffer"),
                contents: colored_info.as_std140().as_bytes(),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_DST,
            },
        );

        // Create the bind group layout for the camera.
        let layout =
            layout_storage.create_layout(renderer.device(), SystemLayout);

        // Create the bind group.
        let bind_group =
            renderer
                .device()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: camera_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: time_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: text_colored_buffer.as_entire_binding(),
                        },
                    ],
                    label: Some("system_bind_group"),
                });

        Self {
            camera,
            camera_buffer,
            time_buffer,
            text_colored_buffer,
            bind_group,
        }
    }

    pub fn projection(&self) -> mint::ColumnMatrix4<f32> {
        self.camera.projection()
    }

    pub fn set_controls(&mut self, controls: Controls) -> Controls {
        self.camera.set_controls(controls)
    }

    pub fn set_projection(&mut self, projection: Projection) {
        self.camera.set_projection(projection);
    }

    pub fn update(&mut self, renderer: &Renderer, frame_time: &FrameTime) {
        if self.camera.update(frame_time.delta_seconds()) {
            let proj = self.camera.projection();
            let view = self.camera.view();
            let eye: mint::Vector3<f32> = self.camera.eye().into();

            let camera_info = CameraUniform { view, proj, eye };

            renderer.queue().write_buffer(
                &self.camera_buffer,
                0,
                camera_info.as_std140().as_bytes(),
            );
        }

        let time_info = TimeUniform {
            seconds: frame_time.seconds(),
        };

        renderer.queue().write_buffer(
            &self.time_buffer,
            0,
            time_info.as_std140().as_bytes(),
        );
    }

    pub fn set_text_colored(&mut self, renderer: &Renderer, colored: bool) {
        let colored_info = TextColoredUniform {
            colored: colored as u32,
        };

        renderer.queue().write_buffer(
            &self.text_colored_buffer,
            0,
            colored_info.as_std140().as_bytes(),
        );
    }
    
    pub fn view(&self) -> mint::ColumnMatrix4<f32> {
        self.camera.view()
    }
}
