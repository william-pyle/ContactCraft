//! The group editor. Groups have their own shared theme and can include contacts or bare numbers.

use contactcraft_store::{Contact, GroupMember, labels};
use egui::{Color32, CornerRadius, Rect, ScrollArea, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

use crate::app::App;
use crate::{theme, widgets};

impl App {
    pub fn group_editor(&mut self, ui: &mut Ui) {
        let t = self.t;
        let dialog_mode = self.in_dialog();
        let mut group = if dialog_mode {
            let Some(group) = self.dialog.as_ref().and_then(|d| d.group.clone()) else { return };
            group
        } else {
            let Some(id) = self.active_group.as_ref() else { return };
            let Some(group) = self.group(id).cloned() else {
                self.active_group = None;
                return;
            };
            group
        };
        let id = group.id.clone();
        let mut changed = false;
        let mut pick_banner = false;
        let mut pick_background = false;
        let mut clear_pending_banner = false;
        let mut clear_pending_background = false;
        let mut delete = false;
        let mut save = false;
        let mut cancel = false;
        let contacts: Vec<(String, String)> = self.contacts.iter().map(|c| (c.id.clone(), c.display_name())).collect();

        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 8.0, t.surface);
        ui.painter().rect_stroke(rect, 8.0, Stroke::new(1.0, t.border), StrokeKind::Inside);
        let mut u = ui.new_child(egui::UiBuilder::new().max_rect(rect.shrink2(vec2(22.0, 18.0))).layout(egui::Layout::top_down(egui::Align::Min)));
        if dialog_mode {
            u.horizontal(|ui| {
                ui.label(egui::RichText::new(if id.is_empty() { "New contact group" } else { "Edit contact group" }).font(theme::semibold(20.0)).color(t.text));
                ui.add_space(14.0);
                save = widgets::primary_button(ui, &t, "Save", !self.group_name.trim().is_empty()).clicked();
                cancel = widgets::secondary_button(ui, &t, "Cancel").clicked();
            });
            u.add_space(8.0);
        }
        ScrollArea::vertical().auto_shrink([false, false]).id_salt("group_editor").show(&mut u, |ui| {
            ui.spacing_mut().item_spacing.y = 8.0;
            ui.label(egui::RichText::new("Contact group").font(theme::semibold(20.0)).color(t.text));
            ui.label(egui::RichText::new("Groups are shared with other applications. Members can be saved contacts or phone numbers on their own.").font(theme::regular(12.5)).color(t.text_dim));
            ui.horizontal(|ui| {
                ui.label("Name");
                let r = ui.add(egui::TextEdit::singleline(&mut self.group_name).desired_width(260.0));
                if r.changed() && !self.group_name.trim().is_empty() {
                    group.name = self.group_name.trim().to_string();
                    changed = true;
                }
            });
            if self.group_name.trim().is_empty() {
                ui.colored_label(t.red, "Give the group a name.");
            }

            let mut sample = Contact::new();
            sample.id = group.id.clone();
            sample.name.display = group.name.clone();
            sample.theme = group.theme.clone();
            let has_background = self.store.as_ref().is_some_and(|s| s.background_path(&sample).is_some());
            let look = self.look(&sample, has_background);
            let (preview, _) = ui.allocate_exact_size(vec2(ui.available_width().min(560.0), 146.0), Sense::hover());
            ui.painter().rect_filled(preview, 8.0, t.surface);
            self.paint_backdrop(ui, preview, &sample, &look, CornerRadius::same(8));
            let banner = Rect::from_min_size(preview.min, vec2(preview.width(), 92.0));
            self.paint_banner(ui, banner, &sample, &look, CornerRadius::same(8));
            ui.painter().text(pos2(banner.min.x + 18.0, banner.center().y), egui::Align2::LEFT_CENTER, &group.name, theme::semibold(23.0), look.banner_ink);
            ui.painter().text(pos2(preview.min.x + 18.0, banner.max.y + 28.0), egui::Align2::LEFT_CENTER, "Group theme preview", theme::regular(14.0), look.text);

            ui.add_space(4.0);
            ui.label(egui::RichText::new(format!("Members ({})", group.members.len())).font(theme::semibold(16.0)).color(t.text));
            if group.members.is_empty() {
                ui.label(egui::RichText::new("No members yet.").color(t.text_dim));
            }
            let mut remove = None;
            for (index, member) in group.members.iter().enumerate() {
                ui.horizontal(|ui| {
                    let text = match member {
                        GroupMember::Contact { id } => contacts.iter().find(|(key, _)| key == id).map(|(_, name)| name.clone()).unwrap_or_else(|| format!("Missing contact ({id})")),
                        GroupMember::Number { number } => format!("{number}  (number only)"),
                    };
                    ui.label(text);
                    if ui.small_button("Remove").clicked() { remove = Some(index); }
                });
            }
            if let Some(index) = remove {
                group.members.remove(index);
                changed = true;
            }
            egui::ComboBox::from_id_salt("group_add_contact").selected_text("Add contact…").width(260.0).show_ui(ui, |ui| {
                for (contact_id, name) in &contacts {
                    if !group.members.contains(&GroupMember::Contact { id: contact_id.clone() }) && ui.selectable_label(false, name).clicked() {
                        group.add_contact(contact_id.clone());
                        changed = true;
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.group_number).hint_text("Phone number").desired_width(220.0));
                if widgets::secondary_button(ui, &t, "Add number").clicked() {
                    if !self.group_number.trim().is_empty() {
                        group.add_number(std::mem::take(&mut self.group_number));
                        changed = true;
                    }
                }
            });

            ui.add_space(8.0);
            ui.label(egui::RichText::new("Group theme").font(theme::semibold(16.0)).color(t.text));
            ui.label(egui::RichText::new("App Default uses the standard theme. A preset or custom values give this group its own look.").font(theme::regular(12.5)).color(t.text_dim));
            ui.horizontal_wrapped(|ui| {
                for (name, accent, second, style, background) in [
                    ("Ocean", "#0B6FB8", "#29B6D8", "gradient", "#EEF6FB"),
                    ("Sunset", "#E0582F", "#F4B23E", "diagonal", "#FFF6EE"),
                    ("Forest", "#2E7D4F", "#9CCB6B", "glow", "#F1F8F0"),
                    ("Berry", "#8E2C7A", "#E05C9A", "dots", "#FBF0F7"),
                    ("Midnight", "#1E2A5A", "#5A4BB8", "glow", "#14182A"),
                    ("App Default", "", "", "gradient", ""),
                ] {
                    if ui.button(name).clicked() {
                        group.theme.accent = accent.into();
                        group.theme.accent2 = second.into();
                        group.theme.style = style.into();
                        group.theme.banner_image.clear();
                        clear_pending_banner = true;
                        group.theme.background = background.into();
                        group.theme.background_image.clear();
                        clear_pending_background = true;
                        if name == "App Default" { group.theme = Default::default(); }
                        changed = true;
                    }
                }
            });
            color_field(ui, "Main colour", &mut group.theme.accent, &mut changed);
            color_field(ui, "Second colour", &mut group.theme.accent2, &mut changed);
            color_field(ui, "Banner text", &mut group.theme.text, &mut changed);
            ui.horizontal(|ui| {
                ui.label("Pattern");
                egui::ComboBox::from_id_salt("group_pattern").selected_text(&group.theme.style).show_ui(ui, |ui| {
                    for style in labels::THEME_STYLE {
                        changed |= ui.selectable_value(&mut group.theme.style, (*style).into(), *style).changed();
                    }
                });
            });
            ui.horizontal(|ui| {
                ui.label("Picture shape");
                egui::ComboBox::from_id_salt("group_picture_shape").selected_text(&group.theme.avatar_shape).show_ui(ui, |ui| {
                    for shape in labels::AVATAR_SHAPE {
                        changed |= ui.selectable_value(&mut group.theme.avatar_shape, (*shape).into(), *shape).changed();
                    }
                });
            });
            ui.horizontal(|ui| {
                ui.label("Banner picture");
                if ui.button(if group.theme.banner_image.is_empty() { "Choose…" } else { "Change…" }).clicked() { pick_banner = true; }
                if (!group.theme.banner_image.is_empty() || self.dialog.as_ref().is_some_and(|d| d.pending_banner.is_some())) && ui.button("Remove").clicked() { group.theme.banner_image.clear(); clear_pending_banner = true; changed = true; }
                if self.dialog.as_ref().is_some_and(|d| d.pending_banner.is_some()) { ui.label("Selected; shown after Save"); }
            });
            color_field(ui, "Background", &mut group.theme.background, &mut changed);
            ui.horizontal(|ui| {
                ui.label("Background picture");
                if ui.button(if group.theme.background_image.is_empty() { "Choose…" } else { "Change…" }).clicked() { pick_background = true; }
                if (!group.theme.background_image.is_empty() || self.dialog.as_ref().is_some_and(|d| d.pending_background.is_some())) && ui.button("Remove").clicked() { group.theme.background_image.clear(); clear_pending_background = true; changed = true; }
                if self.dialog.as_ref().is_some_and(|d| d.pending_background.is_some()) { ui.label("Selected; shown after Save"); }
            });
            if !group.theme.background_image.is_empty() {
                ui.horizontal(|ui| {
                    ui.label("Picture fit");
                    egui::ComboBox::from_id_salt("group_background_fit").selected_text(&group.theme.background_fit).show_ui(ui, |ui| {
                        for fit in labels::BACKGROUND_FIT {
                            changed |= ui.selectable_value(&mut group.theme.background_fit, (*fit).into(), *fit).changed();
                        }
                    });
                });
                let mut opacity = group.theme.background_opacity as f32;
                if ui.add(egui::Slider::new(&mut opacity, 5.0..=100.0).text("Picture strength").suffix("%")).changed() {
                    group.theme.background_opacity = opacity.round() as u8;
                    changed = true;
                }
                changed |= ui.checkbox(&mut group.theme.background_blur, "Soften background picture").changed();
            }
            ui.horizontal(|ui| {
                ui.label("Font");
                egui::ComboBox::from_id_salt("group_font").selected_text(if group.theme.font.is_empty() { "App Default" } else { &group.theme.font }).show_ui(ui, |ui| {
                    if ui.selectable_label(group.theme.font.is_empty(), "App Default").clicked() {
                        group.theme.font.clear(); group.theme.font_file.clear(); changed = true;
                    }
                    for (name, file) in labels::WINDOWS_FONTS {
                        if ui.selectable_label(group.theme.font == *name, *name).clicked() {
                            group.theme.font = (*name).into(); group.theme.font_file = (*file).into(); changed = true;
                        }
                    }
                });
            });
            let mut text_size = group.theme.text_size;
            if ui.add(egui::Slider::new(&mut text_size, 0.0..=32.0).text("Text size (0 = App Default)")).changed() {
                group.theme.text_size = text_size; changed = true;
            }
            changed |= ui.checkbox(&mut group.theme.bold, "Bold text").changed();
            changed |= ui.checkbox(&mut group.theme.italic, "Italic text").changed();
            color_field(ui, "Text colour", &mut group.theme.text_color, &mut changed);
            color_field(ui, "Link colour", &mut group.theme.link_color, &mut changed);
            color_field(ui, "Their messages", &mut group.theme.bubble_in, &mut changed);
            color_field(ui, "Their text", &mut group.theme.bubble_in_text, &mut changed);
            color_field(ui, "Your messages", &mut group.theme.bubble_out, &mut changed);
            color_field(ui, "Your text", &mut group.theme.bubble_out_text, &mut changed);
            let mut roundness = group.theme.bubble_radius as f32;
            if ui.add(egui::Slider::new(&mut roundness, 0.0..=24.0).text("Message roundness (0 = App Default)")).changed() {
                group.theme.bubble_radius = roundness.round() as u8; changed = true;
            }
            ui.horizontal(|ui| {
                ui.label("Emoji");
                changed |= ui.add(egui::TextEdit::singleline(&mut group.theme.emoji).desired_width(100.0)).changed();
            });
            ui.horizontal(|ui| {
                ui.label("Ringtone");
                changed |= ui.add(egui::TextEdit::singleline(&mut group.theme.ringtone).hint_text("App Default").desired_width(260.0)).changed();
            });
            ui.horizontal(|ui| {
                ui.label("Message sound");
                changed |= ui.add(egui::TextEdit::singleline(&mut group.theme.text_tone).hint_text("App Default").desired_width(260.0)).changed();
            });
            ui.add_space(10.0);
            if !dialog_mode && widgets::secondary_button(ui, &t, "Delete group").clicked() { delete = true; }
        });

        if pick_banner || pick_background {
            let title = if pick_banner { "Choose a banner picture" } else { "Choose a background picture" };
            if let Some(path) = rfd::FileDialog::new().set_title(title).add_filter("Pictures", &["png", "jpg", "jpeg", "gif", "webp"]).pick_file() {
                match std::fs::read(&path) {
                    Ok(bytes) => {
                        if dialog_mode {
                            if let Some(d) = self.dialog.as_mut() {
                                if pick_banner { d.pending_banner = Some(bytes); }
                                else { d.pending_background = Some(bytes); }
                            }
                        } else if let Some(store) = &self.store {
                            let result = if pick_banner {
                                store.set_banner(&mut group.theme, &group.id, &bytes)
                            } else {
                                store.set_group_background(&mut group, &bytes)
                            };
                            match result { Ok(()) => { changed = true; self.forget_textures(); }, Err(e) => self.notify(e, true) }
                        }
                    }
                    Err(e) => self.notify(e.to_string(), true),
                }
            }
        }
        if dialog_mode {
            if let Some(d) = self.dialog.as_mut() {
                if clear_pending_banner { d.pending_banner = None; }
                if clear_pending_background { d.pending_background = None; }
                d.group = Some(group);
            }
            let ctx = ui.ctx().clone();
            if save { self.save_group_dialog(&ctx); }
            else if cancel { self.finish_dialog(&ctx, "cancelled", None, None); }
        } else if changed { self.save_group(group); }
        if delete { self.confirm_group_delete = Some(id); }
    }
}

fn color_field(ui: &mut Ui, label: &str, value: &mut String, changed: &mut bool) {
    ui.horizontal(|ui| {
        ui.label(label);
        let mut own = theme::hex(value).is_some();
        if ui.checkbox(&mut own, "").on_hover_text("Custom colour").changed() {
            *value = if own { "#0F68F0".into() } else { String::new() };
            *changed = true;
        }
        if let Some(col) = theme::hex(value) {
            let mut rgb = [col.r(), col.g(), col.b()];
            if egui::color_picker::color_edit_button_srgb(ui, &mut rgb).changed() {
                *value = theme::to_hex(Color32::from_rgb(rgb[0], rgb[1], rgb[2]));
                *changed = true;
            }
        } else { ui.label("App Default"); }
    });
}
