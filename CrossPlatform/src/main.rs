mod crt;
mod paths;
mod pipeline;
mod settings_ui;

use std::{
    fs,
    path::PathBuf,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use anyhow::{Context, Result};
use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use image::RgbaImage;
use ntsc_rs::settings::{standard::NtscEffectFullSettings, SettingsList};
use serde::{Deserialize, Serialize};

use crate::{
    paths::{find_shader_root, SHADERS},
    pipeline::{RenderOptions, ResizeFilter},
};

#[derive(Deserialize, Serialize)]
struct ProjectPreset {
    version: u32,
    ntsc_enabled: bool,
    ntsc_settings: serde_json::Value,
    downscale_width: u32,
    resize_filter: ResizeFilter,
    output_width: u32,
    shader_index: usize,
}

struct RenderMessage {
    generation: u64,
    result: Result<RgbaImage>,
}

struct NtscrtApp {
    source_path: Option<PathBuf>,
    source: Option<RgbaImage>,
    source_texture: Option<TextureHandle>,
    output: Option<RgbaImage>,
    output_texture: Option<TextureHandle>,
    shader_root: Option<PathBuf>,
    shader_index: usize,
    ntsc_enabled: bool,
    ntsc_settings: NtscEffectFullSettings,
    settings_list: SettingsList<NtscEffectFullSettings>,
    downscale_width: u32,
    resize_filter: ResizeFilter,
    output_width: u32,
    dirty: bool,
    busy: bool,
    generation: u64,
    status: String,
    sender: Sender<RenderMessage>,
    receiver: Receiver<RenderMessage>,
}

impl NtscrtApp {
    fn new(_creation_context: &eframe::CreationContext<'_>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let shader_root = find_shader_root();
        let status = if shader_root.is_some() {
            "Open an image to begin.".to_owned()
        } else {
            "CRT shaders were not found. Set NTSCRT_SHADER_DIR or place slang-shaders beside the executable.".to_owned()
        };
        Self {
            source_path: None,
            source: None,
            source_texture: None,
            output: None,
            output_texture: None,
            shader_root,
            shader_index: 1,
            ntsc_enabled: true,
            ntsc_settings: NtscEffectFullSettings::default(),
            settings_list: SettingsList::new(),
            downscale_width: 320,
            resize_filter: ResizeFilter::Area,
            output_width: 1280,
            dirty: false,
            busy: false,
            generation: 0,
            status,
            sender,
            receiver,
        }
    }

    fn open_image(&mut self, context: &egui::Context) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter(
                "Images",
                &["png", "jpg", "jpeg", "webp", "bmp", "tga", "gif"],
            )
            .pick_file()
        else {
            return;
        };
        match image::open(&path)
            .with_context(|| format!("Could not open {}", path.display()))
            .map(|image| image.to_rgba8())
        {
            Ok(image) => {
                self.source_texture = Some(load_texture(
                    context,
                    "source-preview",
                    &image,
                    TextureOptions::LINEAR,
                ));
                self.source = Some(image);
                self.source_path = Some(path);
                self.output = None;
                self.output_texture = None;
                self.dirty = true;
                self.status = "Image loaded. Press Render preview.".to_owned();
            }
            Err(error) => self.status = format!("{error:#}"),
        }
    }

    fn save_output(&mut self) {
        let Some(output) = &self.output else {
            self.status = "Render an image before exporting.".to_owned();
            return;
        };
        let default_name = self
            .source_path
            .as_ref()
            .and_then(|path| path.file_stem())
            .and_then(|name| name.to_str())
            .map(|name| format!("{name}-ntscrt.png"))
            .unwrap_or_else(|| "ntscrt-output.png".to_owned());
        let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG image", &["png"])
            .set_file_name(default_name)
            .save_file()
        else {
            return;
        };
        match output.save(&path) {
            Ok(()) => self.status = format!("Saved {}", path.display()),
            Err(error) => self.status = format!("Could not save {}: {error}", path.display()),
        }
    }

    fn save_preset(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("NTSCRT preset", &["json"])
            .set_file_name("ntscrt-preset.json")
            .save_file()
        else {
            return;
        };
        let result = (|| -> Result<()> {
            let ntsc_json = self
                .settings_list
                .to_json_string(&self.ntsc_settings)
                .context("Could not serialize ntsc-rs settings")?;
            let preset = ProjectPreset {
                version: 1,
                ntsc_enabled: self.ntsc_enabled,
                ntsc_settings: serde_json::from_str(&ntsc_json)?,
                downscale_width: self.downscale_width,
                resize_filter: self.resize_filter,
                output_width: self.output_width,
                shader_index: self.shader_index,
            };
            fs::write(&path, serde_json::to_vec_pretty(&preset)?)
                .with_context(|| format!("Could not write {}", path.display()))
        })();
        self.status = match result {
            Ok(()) => format!("Saved preset {}", path.display()),
            Err(error) => format!("{error:#}"),
        };
    }

    fn load_preset(&mut self) {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("NTSCRT preset", &["json"])
            .pick_file()
        else {
            return;
        };
        let result = (|| -> Result<ProjectPreset> {
            let bytes =
                fs::read(&path).with_context(|| format!("Could not read {}", path.display()))?;
            let preset: ProjectPreset = serde_json::from_slice(&bytes)
                .with_context(|| format!("Invalid preset {}", path.display()))?;
            if preset.version != 1 {
                anyhow::bail!("Unsupported preset version {}", preset.version);
            }
            Ok(preset)
        })();
        match result {
            Ok(preset) => {
                match self
                    .settings_list
                    .from_json(&preset.ntsc_settings.to_string())
                {
                    Ok(settings) => {
                        self.ntsc_settings = settings;
                        self.ntsc_enabled = preset.ntsc_enabled;
                        self.downscale_width = preset.downscale_width.max(16);
                        self.resize_filter = preset.resize_filter;
                        self.output_width = preset.output_width.max(16);
                        self.shader_index = preset.shader_index.min(SHADERS.len() - 1);
                        self.dirty = self.source.is_some();
                        self.status = format!("Loaded preset {}", path.display());
                    }
                    Err(error) => self.status = format!("Invalid ntsc-rs settings: {error}"),
                }
            }
            Err(error) => self.status = format!("{error:#}"),
        }
    }

    fn start_render(&mut self, context: &egui::Context) {
        if self.busy {
            return;
        }
        let Some(source) = self.source.clone() else {
            self.status = "Open an image first.".to_owned();
            return;
        };
        let Some(shader_root) = &self.shader_root else {
            self.status = "CRT shaders were not found. Set NTSCRT_SHADER_DIR.".to_owned();
            return;
        };
        let shader_path = shader_root.join(SHADERS[self.shader_index].1);
        if !shader_path.is_file() {
            self.status = format!("Shader preset not found: {}", shader_path.display());
            return;
        }

        self.generation += 1;
        let generation = self.generation;
        let options = RenderOptions {
            ntsc_enabled: self.ntsc_enabled,
            ntsc_settings: self.ntsc_settings.clone(),
            downscale_width: self.downscale_width,
            resize_filter: self.resize_filter,
            output_width: self.output_width,
            shader_path,
        };
        let sender = self.sender.clone();
        let context = context.clone();
        self.busy = true;
        self.status = "Rendering NTSC and CRT pipeline…".to_owned();
        thread::spawn(move || {
            let result = pipeline::render(&source, &options);
            let _ = sender.send(RenderMessage { generation, result });
            context.request_repaint();
        });
    }

    fn poll_render(&mut self, context: &egui::Context) {
        while let Ok(message) = self.receiver.try_recv() {
            if message.generation != self.generation {
                continue;
            }
            self.busy = false;
            match message.result {
                Ok(image) => {
                    self.output_texture = Some(load_texture(
                        context,
                        "output-preview",
                        &image,
                        TextureOptions::LINEAR,
                    ));
                    self.output = Some(image);
                    self.dirty = false;
                    self.status = "Preview rendered.".to_owned();
                }
                Err(error) => self.status = format!("{error:#}"),
            }
        }
    }

    fn show_toolbar(&mut self, context: &egui::Context) {
        egui::TopBottomPanel::top("toolbar").show(context, |ui| {
            ui.horizontal_wrapped(|ui| {
                if ui.button("Open image").clicked() {
                    self.open_image(context);
                }
                if ui
                    .add_enabled(self.output.is_some(), egui::Button::new("Export PNG"))
                    .clicked()
                {
                    self.save_output();
                }
                ui.separator();
                if ui.button("Load preset").clicked() {
                    self.load_preset();
                }
                if ui.button("Save preset").clicked() {
                    self.save_preset();
                }
                ui.separator();
                let label = if self.busy {
                    "Rendering…"
                } else if self.dirty {
                    "Render preview *"
                } else {
                    "Render preview"
                };
                if ui
                    .add_enabled(
                        self.source.is_some() && !self.busy,
                        egui::Button::new(label),
                    )
                    .clicked()
                {
                    self.start_render(context);
                }
            });
        });
    }

    fn show_sidebar(&mut self, context: &egui::Context) {
        egui::SidePanel::left("settings")
            .default_width(330.0)
            .min_width(280.0)
            .show(context, |ui| {
                ui.heading("NTSCRT");
                ui.small("Windows/Linux image frontend");
                ui.separator();

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.heading("Pipeline");
                    let mut changed = ui
                        .add(
                            egui::Slider::new(&mut self.downscale_width, 64..=1920)
                                .text("Retro width"),
                        )
                        .changed();
                    changed |= ui
                        .add(
                            egui::Slider::new(&mut self.output_width, 320..=3840)
                                .text("Output width"),
                        )
                        .changed();
                    egui::ComboBox::from_label("Resize")
                        .selected_text(self.resize_filter.label())
                        .show_ui(ui, |ui| {
                            for filter in ResizeFilter::ALL {
                                changed |= ui
                                    .selectable_value(
                                        &mut self.resize_filter,
                                        filter,
                                        filter.label(),
                                    )
                                    .changed();
                            }
                        });
                    egui::ComboBox::from_label("CRT shader")
                        .selected_text(SHADERS[self.shader_index].0)
                        .show_ui(ui, |ui| {
                            for (index, (label, _)) in SHADERS.iter().enumerate() {
                                changed |= ui
                                    .selectable_value(&mut self.shader_index, index, *label)
                                    .changed();
                            }
                        });

                    ui.separator();
                    ui.heading("VHS / NTSC");
                    changed |= ui
                        .checkbox(&mut self.ntsc_enabled, "Enable ntsc-rs")
                        .changed();
                    ui.add_enabled_ui(self.ntsc_enabled, |ui| {
                        changed |= settings_ui::show_settings(
                            ui,
                            &mut self.ntsc_settings,
                            &self.settings_list.setting_descriptors,
                        );
                    });
                    self.dirty |= changed && self.source.is_some();
                });
            });
    }

    fn show_preview(&mut self, context: &egui::Context) {
        egui::CentralPanel::default().show(context, |ui| {
            if self.source_texture.is_none() {
                ui.centered_and_justified(|ui| {
                    ui.label("Open a PNG, JPEG, WebP, BMP, TGA, or GIF image.");
                });
                return;
            }

            ui.columns(2, |columns| {
                show_texture(&mut columns[0], "Source", self.source_texture.as_ref());
                show_texture(&mut columns[1], "Processed", self.output_texture.as_ref());
            });
        });

        egui::TopBottomPanel::bottom("status").show(context, |ui| {
            ui.horizontal_wrapped(|ui| {
                if self.busy {
                    ui.spinner();
                }
                ui.label(&self.status);
            });
        });
    }
}

impl eframe::App for NtscrtApp {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_render(context);
        self.show_toolbar(context);
        self.show_sidebar(context);
        self.show_preview(context);
    }
}

fn load_texture(
    context: &egui::Context,
    name: &str,
    image: &RgbaImage,
    options: TextureOptions,
) -> TextureHandle {
    let color_image = ColorImage::from_rgba_unmultiplied(
        [image.width() as usize, image.height() as usize],
        image.as_raw(),
    );
    context.load_texture(name, color_image, options)
}

fn show_texture(ui: &mut egui::Ui, label: &str, texture: Option<&TextureHandle>) {
    ui.vertical_centered(|ui| {
        ui.strong(label);
        ui.add_space(4.0);
        if let Some(texture) = texture {
            let available = ui.available_size();
            let texture_size = texture.size_vec2();
            let scale = (available.x / texture_size.x)
                .min(available.y / texture_size.y)
                .min(1.0);
            ui.add(
                egui::Image::new(texture)
                    .fit_to_exact_size((texture_size * scale).max(egui::vec2(1.0, 1.0))),
            );
        } else {
            ui.label("Render preview to see the result.");
        }
    });
}

fn main() -> eframe::Result {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../../Assets/icon-source.png"))
        .unwrap_or_default();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("NTSCRT")
            .with_inner_size([1280.0, 760.0])
            .with_min_inner_size([860.0, 560.0])
            .with_icon(icon),
        ..Default::default()
    };
    eframe::run_native(
        "NTSCRT",
        options,
        Box::new(|creation_context| Ok(Box::new(NtscrtApp::new(creation_context)))),
    )
}
