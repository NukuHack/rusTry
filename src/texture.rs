﻿
use image::*;
use std::fs;
use std::result::Result::Ok;
use std::path::{Path, PathBuf};
use std::env;
use crate::texture;



pub struct Texture {
    #[allow(unused)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self, ImageError> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, Some(label))
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &DynamicImage,
        label: Option<&str>,
    ) -> Result<Self, ImageError> {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self { texture, view, sampler })
    }


    pub fn load_texture_bytes(
        path: &str
    ) -> Result<Vec<u8>, std::io::Error> {
        std::fs::read(path)
    }
}



pub struct TextureManager {
    pub texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub path: String,
}

impl TextureManager {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        raw_path: &str,
    ) -> Self {
        // Get current directory (handle potential error)
        let current_dir = match env::current_dir() {
            Ok(path) => path,
            Err(e) => {
                eprintln!("Error getting current directory: {}", e);
                panic!("Cannot proceed without current directory");
            }
        };
        println!("Current directory: {:?}", current_dir);
        // Build the full path using PathBuf (platform-agnostic)
        let full_path = current_dir
            .join("resources") // Add "resources" directory
            .join(raw_path);       // Add the final path component
        // Convert PathBuf to string (handle possible invalid UTF-8)
        let path: &str = full_path
            .to_str()
            .expect("Path contains invalid UTF-8 characters");
        // Load texture bytes and handle errors
        let bytes = texture::Texture::load_texture_bytes(&path)
            .expect("Failed to load texture bytes");
        // Create texture with proper error handling
        let texture =
            texture::Texture::from_bytes(device, queue, &bytes, path)
                .expect("Failed to load texture");



        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        Self {
            texture,
            bind_group,
            bind_group_layout,
            path: path.to_string(),
        }
    }
}