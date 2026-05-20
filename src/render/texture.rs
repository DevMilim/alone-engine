use std::path::Path;

use wgpu::{BindGroup, Sampler, TexelCopyBufferLayout, TexelCopyTextureInfoBase, TextureView};

use crate::ImageAsset;

#[derive(Debug, Clone, PartialEq)]
pub struct GpuTextureAsset {
    pub view: TextureView,
    pub sampler: Sampler,
    pub bind_group: BindGroup,
}
