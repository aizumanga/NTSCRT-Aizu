use std::{
    io::{Cursor, Write},
    path::Path,
    sync::Arc,
};

use anyhow::{anyhow, bail, Context, Result};
use image::RgbaImage;
use librashader::{
    presets::ShaderFeatures,
    runtime::{
        wgpu::{FilterChain, FilterChainOptions, WgpuOutputView},
        Viewport,
    },
};
use parking_lot::Mutex;
use wgpu29::{
    Buffer, BufferAddress, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Device,
    ErrorFilter, Extent3d, Instance, Queue, TexelCopyBufferInfo, TexelCopyBufferLayout,
    TexelCopyTextureInfo, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureViewDescriptor,
};

struct BufferDimensions {
    height: usize,
    unpadded_bytes_per_row: usize,
    padded_bytes_per_row: usize,
}

impl BufferDimensions {
    fn new(width: usize, height: usize) -> Self {
        let unpadded_bytes_per_row = width * 4;
        let align = wgpu29::COPY_BYTES_PER_ROW_ALIGNMENT as usize;
        let padding = (align - unpadded_bytes_per_row % align) % align;
        Self {
            height,
            unpadded_bytes_per_row,
            padded_bytes_per_row: unpadded_bytes_per_row + padding,
        }
    }
}

pub fn render(
    input: &RgbaImage,
    preset_path: &Path,
    output_width: u32,
    output_height: u32,
) -> Result<RgbaImage> {
    pollster::block_on(render_async(
        input,
        preset_path,
        output_width,
        output_height,
    ))
}

async fn render_async(
    input: &RgbaImage,
    preset_path: &Path,
    output_width: u32,
    output_height: u32,
) -> Result<RgbaImage> {
    if output_width == 0 || output_height == 0 {
        bail!("Output dimensions must be greater than zero");
    }

    let instance = Instance::new(wgpu29::InstanceDescriptor::new_without_display_handle_from_env());
    let adapter = instance
        .request_adapter(&wgpu29::RequestAdapterOptions {
            power_preference: wgpu29::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .context("No compatible graphics adapter was found")?;

    let required_features = wgpu29::Features::ADDRESS_MODE_CLAMP_TO_BORDER
        | wgpu29::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
        | wgpu29::Features::FLOAT32_FILTERABLE;
    let missing_features = required_features & !adapter.features();
    if !missing_features.is_empty() {
        bail!(
            "The graphics adapter does not support features required by the CRT shaders: \
             {missing_features:?}"
        );
    }

    let (device, queue) = adapter
        .request_device(&wgpu29::DeviceDescriptor {
            label: Some("NTSCRT shader device"),
            required_features,
            required_limits: wgpu29::Limits::default(),
            memory_hints: Default::default(),
            ..Default::default()
        })
        .await
        .context("Could not create a graphics device")?;

    render_with_device(
        &device,
        &queue,
        input,
        preset_path,
        output_width,
        output_height,
    )
    .await
}

async fn render_with_device(
    device: &Device,
    queue: &Queue,
    input: &RgbaImage,
    preset_path: &Path,
    output_width: u32,
    output_height: u32,
) -> Result<RgbaImage> {
    let input_size = Extent3d {
        width: input.width(),
        height: input.height(),
        depth_or_array_layers: 1,
    };
    let input_texture = device.create_texture(&TextureDescriptor {
        label: Some("NTSCRT input"),
        size: input_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        view_formats: &[TextureFormat::Rgba8Unorm],
    });
    queue.write_texture(
        TexelCopyTextureInfo {
            texture: &input_texture,
            mip_level: 0,
            origin: wgpu29::Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        input.as_raw(),
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(input.width() * 4),
            rows_per_image: Some(input.height()),
        },
        input_size,
    );

    let load_error_scope = device.push_error_scope(ErrorFilter::Validation);
    let chain_result = FilterChain::load_from_path(
        preset_path,
        ShaderFeatures::NONE,
        device,
        queue,
        Some(&FilterChainOptions {
            force_no_mipmaps: false,
            enable_cache: false,
            adapter_info: None,
        }),
    );
    if let Some(error) = load_error_scope.pop().await {
        bail!(
            "Could not load CRT preset {}: {error}",
            preset_path.display()
        );
    }
    let mut chain = chain_result
        .with_context(|| format!("Could not load CRT preset {}", preset_path.display()))?;

    let output_size = Extent3d {
        width: output_width,
        height: output_height,
        depth_or_array_layers: 1,
    };
    let output_texture = device.create_texture(&TextureDescriptor {
        label: Some("NTSCRT output"),
        size: output_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Rgba8Unorm,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        view_formats: &[TextureFormat::Rgba8Unorm],
    });
    let output_view = output_texture.create_view(&TextureViewDescriptor::default());
    let output =
        WgpuOutputView::new_from_raw(&output_view, output_size.into(), TextureFormat::Rgba8Unorm);
    let viewport = Viewport::new_render_target_sized_origin(output, None)
        .context("Could not create the CRT output viewport")?;

    let dimensions = BufferDimensions::new(output_width as usize, output_height as usize);
    let output_buffer = Arc::new(device.create_buffer(&BufferDescriptor {
        label: Some("NTSCRT readback"),
        size: (dimensions.padded_bytes_per_row * dimensions.height) as BufferAddress,
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
    }));

    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
        label: Some("NTSCRT render"),
    });
    let frame_error_scope = device.push_error_scope(ErrorFilter::Validation);
    let frame_result = chain.frame(&input_texture, &viewport, &mut encoder, 0, None);
    if let Some(error) = frame_error_scope.pop().await {
        bail!("CRT shader rendering failed: {error}");
    }
    frame_result.context("CRT shader rendering failed")?;
    encoder.copy_texture_to_buffer(
        output_texture.as_image_copy(),
        TexelCopyBufferInfo {
            buffer: &output_buffer,
            layout: TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(dimensions.padded_bytes_per_row as u32),
                rows_per_image: Some(output_height),
            },
        },
        output_size,
    );

    let submission = queue.submit([encoder.finish()]);
    device
        .poll(wgpu29::PollType::Wait {
            submission_index: Some(submission),
            timeout: None,
        })
        .context("Waiting for the CRT shader failed")?;

    read_buffer(
        device,
        output_buffer,
        dimensions,
        output_width,
        output_height,
    )
}

fn read_buffer(
    device: &Device,
    output_buffer: Arc<Buffer>,
    dimensions: BufferDimensions,
    width: u32,
    height: u32,
) -> Result<RgbaImage> {
    let pixels = Arc::new(Mutex::new(Vec::new()));
    let pixels_for_callback = Arc::clone(&pixels);
    let buffer_for_callback = Arc::clone(&output_buffer);

    output_buffer
        .slice(..)
        .map_async(wgpu29::MapMode::Read, move |result| {
            if result.is_ok() {
                let mapped = buffer_for_callback.slice(..).get_mapped_range();
                let mut output = pixels_for_callback.lock();
                output.reserve(dimensions.unpadded_bytes_per_row * dimensions.height);
                let mut cursor = Cursor::new(&mut *output);
                for row in mapped.chunks(dimensions.padded_bytes_per_row) {
                    cursor
                        .write_all(&row[..dimensions.unpadded_bytes_per_row])
                        .expect("writing to Vec cannot fail");
                }
                drop(mapped);
                buffer_for_callback.unmap();
            }
        });

    device
        .poll(wgpu29::PollType::Wait {
            submission_index: None,
            timeout: None,
        })
        .context("Reading the rendered image from the GPU failed")?;

    let bytes = pixels.lock().clone();
    if bytes.is_empty() {
        return Err(anyhow!("The GPU returned an empty rendered image"));
    }
    RgbaImage::from_raw(width, height, bytes)
        .ok_or_else(|| anyhow!("The GPU returned an invalid rendered image"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::SHADERS;
    use image::Rgba;

    #[test]
    #[ignore = "requires a graphics adapter and compiles every CRT preset"]
    fn renders_every_bundled_crt_preset() {
        let shader_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../Vendor/slang-shaders");
        let input = RgbaImage::from_pixel(64, 48, Rgba([90, 160, 220, 255]));

        for (name, relative_path) in SHADERS {
            let output = render(&input, &shader_root.join(relative_path), 320, 240)
                .unwrap_or_else(|error| panic!("{name} failed to render: {error:#}"));
            assert_eq!(
                output.dimensions(),
                (320, 240),
                "{name} returned the wrong dimensions"
            );
        }
    }
}
