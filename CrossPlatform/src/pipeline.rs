use std::path::Path;

use anyhow::{bail, Result};
use image::{imageops::FilterType, RgbaImage};
use ntsc_rs::{
    settings::standard::NtscEffectFullSettings,
    yiq_fielding::{BlitInfo, DeinterlaceMode, Rgbx, YiqView},
    NtscEffect,
};
use serde::{Deserialize, Serialize};

use crate::crt;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum ResizeFilter {
    Nearest,
    Bilinear,
    Bicubic,
    Lanczos,
    Area,
}

impl ResizeFilter {
    pub const ALL: [Self; 5] = [
        Self::Nearest,
        Self::Bilinear,
        Self::Bicubic,
        Self::Lanczos,
        Self::Area,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Nearest => "Nearest",
            Self::Bilinear => "Bilinear",
            Self::Bicubic => "Bicubic",
            Self::Lanczos => "Lanczos",
            Self::Area => "Area",
        }
    }

    fn image_filter(self) -> FilterType {
        match self {
            Self::Nearest => FilterType::Nearest,
            Self::Bilinear => FilterType::Triangle,
            Self::Bicubic => FilterType::CatmullRom,
            Self::Lanczos => FilterType::Lanczos3,
            Self::Area => FilterType::Gaussian,
        }
    }
}

#[derive(Clone)]
pub struct RenderOptions {
    pub ntsc_enabled: bool,
    pub ntsc_settings: NtscEffectFullSettings,
    pub downscale_width: u32,
    pub resize_filter: ResizeFilter,
    pub output_width: u32,
    pub shader_path: std::path::PathBuf,
}

pub fn render(source: &RgbaImage, options: &RenderOptions) -> Result<RgbaImage> {
    if source.width() == 0 || source.height() == 0 {
        bail!("The source image is empty");
    }

    let mut analog = source.clone();
    if options.ntsc_enabled {
        apply_ntsc(&mut analog, &options.ntsc_settings, 0);
    }

    let downscale_width = options.downscale_width.clamp(16, source.width().max(16));
    let downscale_height = scaled_height(downscale_width, source.width(), source.height());
    let downscaled = image::imageops::resize(
        &analog,
        downscale_width,
        downscale_height,
        options.resize_filter.image_filter(),
    );

    let output_width = options.output_width.max(16);
    let output_height = scaled_height(output_width, source.width(), source.height());
    crt::render(
        &downscaled,
        Path::new(&options.shader_path),
        output_width,
        output_height,
    )
}

fn scaled_height(target_width: u32, source_width: u32, source_height: u32) -> u32 {
    if source_width == 0 {
        return 1;
    }
    ((target_width as f64 * source_height as f64 / source_width as f64).round() as u32).max(1)
}

fn apply_ntsc(image: &mut RgbaImage, settings: &NtscEffectFullSettings, frame: usize) {
    let width = image.width() as usize;
    let height = image.height() as usize;
    let row_bytes = width * 4;
    let alpha: Vec<u8> = image.pixels().map(|pixel| pixel.0[3]).collect();

    let effect: NtscEffect = settings.into();
    let field = effect.use_field.to_yiq_field(frame);
    let mut yiq_buffer = vec![0.0; YiqView::buf_length_for((width, height), field)];
    let mut view = YiqView::from_parts(&mut yiq_buffer, (width, height), field);
    let blit = BlitInfo::from_full_frame(width, height, row_bytes);
    view.set_from_strided_buffer::<Rgbx, u8, _>(image.as_mut(), blit, ());
    effect.apply_effect_to_yiq(&mut view, frame, [1.0, 1.0]);
    view.write_to_strided_buffer::<Rgbx, u8, _>(image.as_mut(), blit, DeinterlaceMode::Bob, ());

    for (pixel, alpha) in image.pixels_mut().zip(alpha) {
        pixel.0[3] = alpha;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaled_height_preserves_common_aspect_ratios() {
        assert_eq!(scaled_height(1280, 1920, 1080), 720);
        assert_eq!(scaled_height(320, 640, 480), 240);
        assert_eq!(scaled_height(1216, 1216, 832), 832);
    }

    #[test]
    fn scaled_height_never_returns_zero() {
        assert_eq!(scaled_height(1, 16, 1), 1);
        assert_eq!(scaled_height(320, 0, 0), 1);
    }
}
