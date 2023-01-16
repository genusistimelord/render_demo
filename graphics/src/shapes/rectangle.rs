use crate::{
    Allocation, AscendingError, AtlasGroup, OtherError, RectVertex, Texture,
};
use cosmic_text::Color;
use image::{self, ImageBuffer};
use std::cmp;

#[derive(Default)]
pub struct Rect {
    pub position: [u32; 3],
    pub size: [u32; 2],
    pub border_width: u32,
    pub container: Option<Allocation>,
    pub container_uv: [u16; 4],
    pub border: Option<Allocation>,
    pub border_uv: [u16; 4],
    pub radius: f32,
    pub bytes: Vec<u8>,
    /// if anything got updated we need to update the buffers too.
    pub changed: bool,
}

impl Default for Image {
    fn default() -> Self {
        Self {
            pos: [0; 3],
            size: [0; 2],
            border_width: 0,
            container: None,
            container_uv: [0; 4],
            border: None,
            border_uv: [0; 4],
            radius: 0,
            bytes: Vec::new(),
            changed: true,
        }
    }
}

impl Rect {
    fn add_color(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &mut AtlasGroup,
        color: Color,
    ) -> Option<Allocation> {
        let col =
            format!("{},{},{},{}", color.r(), color.g(), color.b(), color.a());
        let mut image: ImageBuffer<image::Rgba<u8>, Vec<u8>> =
            ImageBuffer::new(1, 1);
        let pixel = image.get_pixel_mut(0, 0);
        *pixel = image::Rgba([color.r(), color.g(), color.b(), color.a()]);
        atlas.upload(col, image.as_raw(), 1, 1, device, queue)
    }

    pub fn set_color(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &mut AtlasGroup,
        color: Color,
    ) -> &mut Self {
        self.container = Self::add_color(device, queue, atlas, color);
        self.changed = true;
        self
    }

    pub fn set_border_color(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &mut AtlasGroup,
        color: Color,
    ) -> &mut Self {
        self.border = Self::add_color(device, queue, atlas, color);
        self.changed = true;
        self
    }

    pub fn set_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &mut AtlasGroup,
        path: String,
    ) -> Result<&mut Self, AscendingError> {
        let allocation = Texture::from_file(path)?
            .group_upload(atlas, device, queue)
            .ok_or_else(|| OtherError::new("failed to upload image"))?;
        self.container = Some(allocation);
        self.changed = true;
        Ok(self)
    }

    pub fn set_border_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        atlas: &mut AtlasGroup,
        path: String,
    ) -> Result<&mut Self, AscendingError> {
        let allocation = Texture::from_file(path)?
            .group_upload(atlas, device, queue)
            .ok_or_else(|| OtherError::new("failed to upload image"))?;
        self.border = Some(allocation);
        self.changed = true;
        Ok(self)
    }

    //Set the Rendering Offset of the container.
    pub fn set_container_uv(
        &mut self,
        x: u16,
        y: u16,
        w: u16,
        h: u16,
    ) -> &mut Self {
        self.container_uv = [x, y, w, h];
        self.changed = true;
        self
    }

    //Set the Rendering Offset of the border.
    pub fn set_border_uv(
        &mut self,
        x: u16,
        y: u16,
        w: u16,
        h: u16,
    ) -> &mut Self {
        self.border_uv = [x, y, w, h];
        self.changed = true;
        self
    }

    pub fn create_quad(&mut self) {
        let mut rects = Vec::new();

        let containter_tex = match self.container {
            Some(allocation) => allocation,
            None => return,
        };

        let border_tex = match self.border {
            Some(allocation) => allocation,
            None => return,
        };

        let (u, v, width, height) = containter_tex.rect();
        let container_data = [
            self.container_uv[0].saturating_add(u as u16),
            self.container_uv[1].saturating_add(v as u16),
            cmp::min(self.container_uv[2], width as u16),
            cmp::min(self.container_uv[3], height as u16),
        ];

        let (u, v, width, height) = border_tex.rect();
        let border_data = [
            self.border_uv[0].saturating_add(u as u16),
            self.border_uv[1].saturating_add(v as u16),
            cmp::min(self.border_uv[2], width as u16),
            cmp::min(self.border_uv[3], height as u16),
        ];

        let buffer = RectVertex {
            position: [
                self.position[0] as f32,
                self.position[1] as f32,
                self.position[2] as f32,
            ],
            size: [self.size[0] as f32, self.size[1] as f32],
            border_width: self.border_width as f32,
            radius: self.radius,
            container_data,
            border_data,
            layer: containter_tex.layer as u32,
            border_layer: border_tex.layer as u32,
        };

        rects.push(buffer);

        self.bytes = bytemuck::cast_slice(&rects).to_vec();
    }

    /// used to check and update the ShapeVertex array.
    pub fn update(&mut self) -> bool {
        // if points added or any data changed recalculate paths.
        if self.changed {
            self.create_quad();

            self.changed = false;
            true
        } else {
            false
        }
    }
}
