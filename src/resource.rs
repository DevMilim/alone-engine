use std::{collections::HashMap, marker::PhantomData, path::Path};

use wgpu::{TexelCopyBufferLayout, TexelCopyTextureInfoBase};

use crate::GpuTextureAsset;

pub struct AssetCache<T> {
    assets: HashMap<usize, T>,
    path_map: HashMap<String, usize>,
    next_id: usize,
}

impl<T> AssetCache<T> {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
            path_map: HashMap::new(),
            next_id: 0,
        }
    }
    fn current_id(&mut self) -> Handler<T> {
        self.next_id += 1;
        Handler::new(self.next_id)
    }
    pub fn get_id(&self, path: &str) -> Option<usize> {
        self.path_map.get(path).copied()
    }
    pub fn get(&self, id: Handler<T>) -> Option<&T> {
        self.assets.get(&id.id)
    }
    pub fn get_mut(&mut self, id: Handler<T>) -> Option<&mut T> {
        self.assets.get_mut(&id.id)
    }
    pub fn insert(&mut self, path: &str, asset: T) -> Handler<T> {
        let id = self.current_id();
        self.assets.insert(id.id, asset);
        self.path_map.insert(path.to_string(), id.id);
        id
    }
    pub fn clear(&mut self) {
        self.assets.clear();
        self.path_map.clear();
        self.next_id = 0;
    }
}

impl<T> Default for AssetCache<T> {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct Handler<T> {
    pub id: usize,
    _phantom: PhantomData<T>,
}

impl<T> Clone for Handler<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            _phantom: self._phantom.clone(),
        }
    }
}

impl<T> Copy for Handler<T> {}

impl<T> Handler<T> {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }
}

pub struct Resources {
    pub textures: AssetCache<ImageAsset>,
}

impl Resources {
    pub fn new() -> Self {
        Self {
            textures: AssetCache::new(),
        }
    }
    pub fn clear(&mut self) {
        self.textures.clear();
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageAsset {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub gpu_image: Option<GpuTextureAsset>,
}
impl ImageAsset {
    pub fn load_from_file(path: &str) -> Self {
        let img = image::open(Path::new(path)).expect("Falha ao carregar textura");
        let rgba = img.to_rgba8();

        let dimensions = rgba.dimensions();

        Self {
            width: dimensions.0,
            height: dimensions.1,
            pixels: rgba.to_vec(),
            gpu_image: None,
        }
    }
    pub fn load_from_image_asset(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_bind_group: &wgpu::BindGroupLayout,
    ) {
        let size = wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            TexelCopyTextureInfoBase {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.pixels,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * self.width),
                rows_per_image: Some(self.height),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = device.create_sampler(&wgpu::wgt::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture_bind_group"),
            layout: texture_bind_group,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        let gpu_image = GpuTextureAsset {
            view,
            sampler,
            bind_group,
        };
        self.gpu_image = Some(gpu_image);
    }
}
