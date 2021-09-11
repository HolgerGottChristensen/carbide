use std::num::NonZeroU32;

use wgpu::util::DeviceExt;

/// An command for uploading an individual glyph.
pub struct TextureAtlasCommand<'a> {
    /// The CPU buffer containing the pixel data.
    pub texture_atlas_buffer: &'a [u8],
    /// The GPU image to which the glyphs are cached.
    pub texture_atlas_texture: &'a wgpu::Texture,
    /// The width of the texture.
    pub width: u32,
    /// The height of the texture.
    pub height: u32,
}

impl<'a> TextureAtlasCommand<'a> {
    /// Creates a buffer on the GPU loaded with the updated pixel data.
    ///
    /// Created with `BufferUsage::COPY_SRC`, ready to be copied to the texture.
    ///
    /// TODO: In the future, we should consider re-using the same buffer and writing to it via
    /// `Buffer::map_write_async`. When asking about how to ensure that the write completes before
    /// the following `copy_buffer_to_texture` command, I was advised to just create a new buffer
    /// each time instead for now.
    /// EDIT:
    /// > if you try to map an existing buffer, it will give it to you only after all the gpu use
    /// > of the buffer is over. So you can't do it every frame reasonably
    pub fn create_buffer(&self, device: &wgpu::Device) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("carbide_buffer_init_descriptor"),
            contents: &self.texture_atlas_buffer,
            usage: wgpu::BufferUsages::COPY_SRC,
        })
    }

    /// Create the copy view ready for copying the pixel data to the texture.
    pub fn buffer_copy_view<'b>(&self, buffer: &'b wgpu::Buffer) -> wgpu::ImageCopyBuffer<'b> {
        wgpu::ImageCopyBuffer {
            buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * self.width),
                rows_per_image: NonZeroU32::new(self.height),
            },
        }
    }

    /// Create the texture copy view ready for receiving the pixel data from the buffer.
    pub fn texture_copy_view(&self) -> wgpu::ImageCopyTexture {
        wgpu::ImageCopyTexture {
            texture: &self.texture_atlas_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: Default::default(),
        }
    }

    /// Encode the command for copying the buffer's pixel data to the glyph cache texture.
    pub fn encode(&self, buffer: &wgpu::Buffer, encoder: &mut wgpu::CommandEncoder) {
        let buffer_copy_view = self.buffer_copy_view(&buffer);
        let texture_copy_view = self.texture_copy_view();
        let extent = self.extent();
        encoder.copy_buffer_to_texture(buffer_copy_view, texture_copy_view, extent);
    }

    /// The extent required for the copy command.
    pub fn extent(&self) -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        }
    }

    /// Short-hand for `create_buffer` and `encode` in succession.
    pub fn load_buffer_and_encode(&self, device: &wgpu::Device, e: &mut wgpu::CommandEncoder) {
        let buffer = self.create_buffer(&device);
        self.encode(&buffer, e);
    }
}
