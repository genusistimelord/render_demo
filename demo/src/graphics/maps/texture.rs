pub struct MapTextures {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    unused_ids: Vec<u32>,
}

impl MapTextures {
    pub fn new(device: &wgpu::Device, count: u32) -> Self {
        let extent = wgpu::Extent3d {
            width: 32,
            height: 256,
            depth_or_array_layers: count,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Map array Texture"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba32Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::COPY_SRC,
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Map array Texture"),
            format: Some(wgpu::TextureFormat::Rgba32Uint),
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: std::num::NonZeroU32::new(1),
            base_array_layer: 0,
            array_layer_count: std::num::NonZeroU32::new(count),
        });

        let unused_ids = (0..count).collect();

        Self {
            texture,
            texture_view,
            unused_ids,
        }
    }

    pub fn get_unused_id(&mut self) -> Option<u32> {
        self.unused_ids.pop()
    }

    pub fn mark_id_unused(&mut self, id: u32) {
        self.unused_ids.push(id);
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.texture_view
    }

    pub fn update(&mut self, queue: &wgpu::Queue, id: u32, data: &[u32]) {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: id },
                aspect: wgpu::TextureAspect::All,
            },
            &bytemuck::cast_slice(data).to_vec(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(512),
                rows_per_image: std::num::NonZeroU32::new(256),
            },
            wgpu::Extent3d {
                width: 32,
                height: 256,
                depth_or_array_layers: 1,
            },
        );
    }
}