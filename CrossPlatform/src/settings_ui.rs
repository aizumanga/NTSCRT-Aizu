use eframe::egui;
use ntsc_rs::settings::{EnumValue, SettingDescriptor, SettingKind, Settings};

pub fn show_settings<T: Settings>(
    ui: &mut egui::Ui,
    settings: &mut T,
    descriptors: &[SettingDescriptor<T>],
) -> bool {
    let mut changed = false;
    for descriptor in descriptors {
        changed |= show_setting(ui, settings, descriptor);
    }
    changed
}

fn show_setting<T: Settings>(
    ui: &mut egui::Ui,
    settings: &mut T,
    descriptor: &SettingDescriptor<T>,
) -> bool {
    let mut changed = false;
    let response = match &descriptor.kind {
        SettingKind::Enumeration { options, .. } => {
            let selected = settings
                .get_field::<EnumValue>(&descriptor.id)
                .expect("enum descriptor must reference an enum")
                .0;
            let selected_label = options
                .iter()
                .find(|option| option.index == selected)
                .map(|option| option.label)
                .unwrap_or("Unknown");
            egui::ComboBox::new(&descriptor.id, descriptor.label)
                .selected_text(selected_label)
                .show_ui(ui, |ui| {
                    for option in options {
                        let option_response =
                            ui.selectable_label(selected == option.index, option.label);
                        if option_response.clicked() {
                            settings
                                .set_field(&descriptor.id, EnumValue(option.index))
                                .expect("enum descriptor must accept an enum");
                            changed = true;
                        }
                        if let Some(description) = option.description {
                            option_response.on_hover_text(description);
                        }
                    }
                })
                .response
        }
        SettingKind::Percentage { logarithmic, .. } => {
            let mut value = settings
                .get_field::<f32>(&descriptor.id)
                .expect("percentage descriptor must reference f32");
            let response = ui.add(
                egui::Slider::new(&mut value, 0.0..=1.0)
                    .text(descriptor.label)
                    .logarithmic(*logarithmic)
                    .custom_formatter(|value, _| format!("{:.1}%", value * 100.0)),
            );
            if response.changed() {
                settings
                    .set_field(&descriptor.id, value)
                    .expect("percentage descriptor must accept f32");
            }
            response
        }
        SettingKind::IntRange { range, .. } => {
            let mut value = settings
                .get_field::<i32>(&descriptor.id)
                .expect("integer descriptor must reference i32");
            let response =
                ui.add(egui::Slider::new(&mut value, range.clone()).text(descriptor.label));
            if response.changed() {
                settings
                    .set_field(&descriptor.id, value)
                    .expect("integer descriptor must accept i32");
            }
            response
        }
        SettingKind::FloatRange {
            range, logarithmic, ..
        } => {
            let mut value = settings
                .get_field::<f32>(&descriptor.id)
                .expect("float descriptor must reference f32");
            let response = ui.add(
                egui::Slider::new(&mut value, range.clone())
                    .text(descriptor.label)
                    .logarithmic(*logarithmic),
            );
            if response.changed() {
                settings
                    .set_field(&descriptor.id, value)
                    .expect("float descriptor must accept f32");
            }
            response
        }
        SettingKind::Boolean => {
            let mut value = settings
                .get_field::<bool>(&descriptor.id)
                .expect("boolean descriptor must reference bool");
            let response = ui.checkbox(&mut value, descriptor.label);
            if response.changed() {
                settings
                    .set_field(&descriptor.id, value)
                    .expect("boolean descriptor must accept bool");
            }
            response
        }
        SettingKind::Group { children, .. } => {
            let mut enabled = settings
                .get_field::<bool>(&descriptor.id)
                .expect("group descriptor must reference bool");
            let mut header_response = None;
            ui.group(|ui| {
                ui.set_width(ui.available_width());
                let checkbox = ui.checkbox(&mut enabled, descriptor.label);
                if checkbox.changed() {
                    settings
                        .set_field(&descriptor.id, enabled)
                        .expect("group descriptor must accept bool");
                    changed = true;
                }
                header_response = Some(checkbox);
                ui.add_enabled_ui(enabled, |ui| {
                    ui.indent(&descriptor.id, |ui| {
                        changed |= show_settings(ui, settings, children);
                    });
                });
            });
            header_response.expect("group always creates a checkbox")
        }
    };

    let response = if let Some(description) = descriptor.description {
        response.on_hover_text(description)
    } else {
        response
    };
    changed || response.changed()
}
