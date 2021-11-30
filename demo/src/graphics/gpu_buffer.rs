pub(crate) use crate::graphics::SpriteVertex;
use std::marker::PhantomData;
use wgpu::util::DeviceExt;

pub struct BufferPass {
    pub vertices: Vec<u8>,
    pub indices: Vec<u8>,
}

pub trait BufferLayout {
    fn attributes() -> Vec<wgpu::VertexAttribute>;
    fn default_buffer() -> BufferPass;
    fn index_stride() -> usize;
    fn vertex_stride() -> usize;
    fn with_capacity(capacity: usize) -> BufferPass;
}

pub struct GpuBuffer<K: BufferLayout> {
    pub vertex_buffer: wgpu::Buffer,
    vertex_count: usize,
    vertex_max: usize,
    pub index_buffer: wgpu::Buffer,
    index_count: usize,
    index_max: usize,
    phantom_data: PhantomData<K>,
}

impl<K: BufferLayout> GpuBuffer<K> {
    fn create_buffer(device: &wgpu::Device, buffers: BufferPass) -> Self {
        GpuBuffer {
            vertex_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: &buffers.vertices,
                usage: wgpu::BufferUsages::VERTEX
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
            }),
            vertex_count: 0,
            vertex_max: buffers.vertices.len(),
            index_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Indices Buffer"),
                contents: &buffers.indices,
                usage: wgpu::BufferUsages::INDEX,
            }), // set to 0 as we set this as we add sprites.
            index_count: (buffers.indices.len() / K::index_stride()),
            index_max: buffers.indices.len(),
            phantom_data: PhantomData,
        }
    }

    /// Gets the indice count which is Max/stride.
    pub fn index_count(&self) -> usize {
        self.index_count
    }

    /// Gets the indice maxium value.
    pub fn index_max(&self) -> usize {
        self.index_max
    }

    pub fn indices(&self, bounds: Option<(u64, u64)>) -> wgpu::BufferSlice {
        let range = if let Some(bounds) = bounds {
            bounds.0..bounds.1
        } else {
            0..(self.index_count * K::index_stride()) as u64
        };

        self.index_buffer.slice(range)
    }

    /// creates a new pre initlized VertexBuffer with a default size.
    /// default size is based on the initial BufferPass::vertices length.
    pub fn new(device: &wgpu::Device) -> Self {
        Self::create_buffer(device, K::default_buffer())
    }

    /// Set the Indices based on how many Vertex's Exist / indices stride.
    pub fn set_index_count(&mut self, count: usize) {
        self.index_count = count;
    }

    pub fn set_indices_from(&mut self, queue: &wgpu::Queue, bytes: &[u8]) {
        let size = bytes.len();

        if size >= self.index_max {
            return;
        }

        self.index_count = size / K::index_stride();
        queue.write_buffer(&self.index_buffer, 0, bytes);
    }

    /// Set the New buffer array to the VertexBuffer.
    /// Sets the vertex_count based on array length / struct stride.
    pub fn set_vertices_from(&mut self, queue: &wgpu::Queue, bytes: &[u8]) {
        let size = bytes.len();

        if size >= self.vertex_max {
            return;
        }

        self.vertex_count = size / K::vertex_stride();
        queue.write_buffer(&self.vertex_buffer, 0, bytes);
    }

    /// Gets the Vertex elements count.
    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }

    /// Gets Vertex buffers max size in bytes.
    pub fn vertex_max(&self) -> usize {
        self.vertex_max
    }

    /// Gets Vertex buffers struct stride.
    pub fn vertex_stride(&self) -> usize {
        K::vertex_stride()
    }

    pub fn vertices(&self, bounds: Option<(u64, u64)>) -> wgpu::BufferSlice {
        let range = if let Some(bounds) = bounds {
            bounds.0..bounds.1
        } else {
            0..self.vertex_count as u64
        };

        self.vertex_buffer.slice(range)
    }
    /// creates a new pre initlized VertexBuffer with a entity count.
    /// size created BufferPass::vertices length.
    pub fn with_capacity(device: &wgpu::Device, capacity: usize) -> Self {
        Self::create_buffer(device, K::with_capacity(capacity))
    }
}