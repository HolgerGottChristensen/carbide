use std::fmt::{Debug, Formatter};
use carbide_core::render::InnerLayer;
use wgpu::{BindGroup, Texture, TextureFormat, TextureView};

/// The colour format of every [`WgpuRenderTarget`].
pub const RENDER_TARGET_FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;

/// A texture that a layer, or an intermediate render pass, is rendered into, with the attachments
/// needed to render into it.
///
/// Created by the 2D renderer, which composites it. Other renderers draw into a layer through it:
/// a [`Layer`](carbide_core::render::Layer) from the wgpu renderer downcasts to this type.
pub struct WgpuRenderTarget {
    texture: Texture,
    view: TextureView,
    bind_group: BindGroup,
    msaa_view: Option<TextureView>,
    depth_view: TextureView,
}

impl WgpuRenderTarget {
    /// Assemble a render target from textures and views made by the renderer that owns it.
    /// `bind_group` is how that renderer samples `view`, and `msaa_view` and `depth_view` must
    /// have the same size as `texture`.
    pub fn from_parts(
        texture: Texture,
        view: TextureView,
        bind_group: BindGroup,
        msaa_view: Option<TextureView>,
        depth_view: TextureView,
    ) -> WgpuRenderTarget {
        WgpuRenderTarget {
            texture,
            view,
            bind_group,
            msaa_view,
            depth_view,
        }
    }

    /// The texture rendered into.
    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    /// A view of [`texture`](Self::texture), to render into or sample from.
    pub fn view(&self) -> &TextureView {
        &self.view
    }

    /// The bind group the owning renderer samples the texture through when compositing it.
    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    /// The multisampled texture rendered into before resolving into [`view`](Self::view), or
    /// `None` when the owning renderer doesn't use multisampling.
    pub fn msaa_view(&self) -> Option<&TextureView> {
        self.msaa_view.as_ref()
    }

    /// The depth/stencil attachment of the owning renderer, the same size as the texture, so a
    /// render pass whose colour attachment is this target can use it.
    pub fn depth_view(&self) -> &TextureView {
        &self.depth_view
    }

    /// The colour format of the texture.
    pub fn texture_format(&self) -> TextureFormat {
        self.texture.format()
    }
}

impl InnerLayer for WgpuRenderTarget {
    fn dimensions(&self) -> (u32, u32) {
        (self.texture.width(), self.texture.height())
    }
}

impl Debug for WgpuRenderTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WgpuRenderTarget")
            .field("dimensions", &self.dimensions())
            .field("format", &self.texture_format())
            .finish_non_exhaustive()
    }
}
