//! The theme editor: how one person looks in every Craft app (banner colours and pattern,
//! picture shape and emoji, font, text size and colours, background colour or picture, message
//! bubbles, sounds), with a live preview of their card and a conversation.

use contactcraft_store::{Contact, ContactTheme, labels};
use egui::{Align2, Color32, CornerRadius, FontId, Margin, Rect, ScrollArea, Sense, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};

use crate::app::{App, Pane};
use crate::icons::Icon;
use crate::theme::{self, Tokens};
use crate::widgets;

const LABEL_W: f32 = 130.0;

/// Ready-made looks to start from.
const PRESETS: &[(&str, &str, &str, &str, &str)] = &[
    // name, accent, accent2, style, background
    ("Ocean", "#0B6FB8", "#29B6D8", "gradient", "#EEF6FB"),
    ("Sunset", "#E0582F", "#F4B23E", "diagonal", "#FFF6EE"),
    ("Forest", "#2E7D4F", "#9CCB6B", "glow", "#F1F8F0"),
    ("Berry", "#8E2C7A", "#E05C9A", "dots", "#FBF0F7"),
    ("Midnight", "#1E2A5A", "#5A4BB8", "glow", "#14182A"),
    ("Slate", "#3F4B5E", "#8A97AB", "stripes", "#F3F5F8"),
    ("Gold", "#8C6A12", "#E2BE4A", "gradient", "#FFFBEE"),
    ("Plain", "", "", "solid", ""),
];

impl App {
    pub fn theme_editor(&mut self, ui: &mut Ui) {
        let t = self.t;
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 8.0, t.surface);
        ui.painter().rect_stroke(rect, 8.0, Stroke::new(1.0, t.border), StrokeKind::Inside);
        let Pane::Theme(c) = &self.pane else { return };
        let mut c: Contact = (**c).clone();

        let inner = rect.shrink2(vec2(22.0, 18.0));
        let wide = inner.width() >= 860.0;
        let (controls_rect, preview_rect) = if wide {
            let split = inner.min.x + (inner.width() * 0.5).clamp(380.0, 520.0);
            (Rect::from_min_max(inner.min, pos2(split - 12.0, inner.max.y)), Rect::from_min_max(pos2(split + 8.0, inner.min.y), inner.max))
        } else {
            let split = inner.min.y + 330.0;
            (Rect::from_min_max(pos2(inner.min.x, split + 12.0), inner.max), Rect::from_min_max(inner.min, pos2(inner.max.x, split)))
        };

        self.theme_preview(ui, preview_rect, &c);

        let mut u = ui.new_child(egui::UiBuilder::new().max_rect(controls_rect).layout(egui::Layout::top_down(egui::Align::Min)));
        let ui = &mut u;
        ui.label(egui::RichText::new(format!("Theme for {}", if c.display_name().is_empty() { "this contact".to_string() } else { c.display_name() })).font(theme::semibold(19.0)).color(t.text));
        ui.label(egui::RichText::new("MessageCraft and MailCraft show this person with these colours, this font and this background.").font(theme::regular(12.0)).color(t.text_dim));
        ui.add_space(8.0);
        let mut save = false;
        let mut cancel = false;
        let mut details = false;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            if widgets::primary_button(ui, &t, "Save", !c.is_blank()).on_hover_text("Ctrl+S").clicked() {
                save = true;
            }
            if widgets::secondary_button(ui, &t, "Cancel").clicked() {
                cancel = true;
            }
            if widgets::secondary_button(ui, &t, "Details…").clicked() {
                details = true;
            }
            if widgets::secondary_button(ui, &t, "Reset theme").on_hover_text("Back to the plain look").clicked() {
                let photo_shape = c.theme.avatar_shape.clone();
                c.theme = ContactTheme { avatar_shape: photo_shape, ..Default::default() };
            }
        });
        ui.add_space(8.0);

        let mut pick_background = false;
        let mut pick_font = false;
        let mut pick_sound: Option<u8> = None;
        ScrollArea::vertical().auto_shrink([false, false]).id_salt(("theme_editor", c.id.clone())).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 7.0;
            let th = &mut c.theme;

            heading(ui, &t, "Start from");
            ui.horizontal_wrapped(|ui| {
                for (name, a, b, style, bg) in PRESETS {
                    if preset_swatch(ui, &t, name, a, b).clicked() {
                        th.accent = a.to_string();
                        th.accent2 = b.to_string();
                        th.style = style.to_string();
                        th.background = bg.to_string();
                        th.text_color.clear();
                        th.text.clear();
                        if name == &"Plain" {
                            th.background_image.clear();
                        }
                    }
                }
            });

            heading(ui, &t, "Banner");
            color_row(ui, &t, "Main colour", &mut th.accent, "Picked from the name");
            color_row(ui, &t, "Second colour", &mut th.accent2, "A darker main colour");
            ui.horizontal(|ui| {
                lbl(ui, &t, "Pattern");
                combo(ui, "style", &mut th.style, labels::THEME_STYLE, 150.0);
            });
            color_row(ui, &t, "Banner text", &mut th.text, "White or black, whichever reads");

            heading(ui, &t, "Picture");
            ui.horizontal(|ui| {
                lbl(ui, &t, "Shape");
                combo(ui, "shape", &mut th.avatar_shape, labels::AVATAR_SHAPE, 150.0);
            });
            ui.horizontal(|ui| {
                lbl(ui, &t, "Emoji");
                ui.add(egui::TextEdit::singleline(&mut th.emoji).char_limit(16).desired_width(80.0).font(theme::regular(16.0)).margin(Margin::symmetric(8, 4)));
                for e in ["⭐", "❤", "😀", "🎉", "🏠", "💼", "🎓", "⚽", "🎵", "🐶", "🌻", "☕"] {
                    if ui.add(egui::Button::new(egui::RichText::new(e).size(16.0)).frame(false)).clicked() {
                        th.emoji = e.to_string();
                    }
                }
            });

            heading(ui, &t, "Text");
            ui.horizontal(|ui| {
                lbl(ui, &t, "Font");
                let shown = if th.font.is_empty() { "The app's font".to_string() } else { th.font.clone() };
                egui::ComboBox::from_id_salt("font").width(220.0).height(360.0).selected_text(shown).show_ui(ui, |ui| {
                    if ui.selectable_label(th.font.is_empty(), "The app's font").clicked() {
                        th.font.clear();
                        th.font_file.clear();
                    }
                    for (name, file) in labels::WINDOWS_FONTS {
                        if ui.selectable_label(th.font == *name, *name).clicked() {
                            th.font = name.to_string();
                            th.font_file = file.to_string();
                        }
                    }
                    ui.separator();
                    if ui.selectable_label(false, "Another font file…").clicked() {
                        pick_font = true;
                    }
                });
            });
            ui.horizontal(|ui| {
                lbl(ui, &t, "Size");
                let mut auto = th.text_size < 8.0;
                if ui.checkbox(&mut auto, "App's size").changed() {
                    th.text_size = if auto { 0.0 } else { 14.0 };
                }
                if !auto {
                    ui.add(egui::Slider::new(&mut th.text_size, 9.0..=32.0).step_by(0.5).suffix(" pt"));
                }
            });
            ui.horizontal(|ui| {
                lbl(ui, &t, "Style");
                ui.checkbox(&mut th.bold, egui::RichText::new("Bold").strong());
                ui.checkbox(&mut th.italic, egui::RichText::new("Italic").italics());
            });
            color_row(ui, &t, "Text colour", &mut th.text_color, "Readable on the background");
            color_row(ui, &t, "Link colour", &mut th.link_color, "The app's blue");

            heading(ui, &t, "Background");
            color_row(ui, &t, "Colour", &mut th.background, "The app's own");
            ui.horizontal(|ui| {
                lbl(ui, &t, "Picture");
                ui.spacing_mut().item_spacing.x = 8.0;
                if widgets::secondary_button(ui, &t, if th.background_image.is_empty() { "Choose a picture…" } else { "Change…" }).clicked() {
                    pick_background = true;
                }
                if !th.background_image.is_empty() && widgets::secondary_button(ui, &t, "Remove").clicked() {
                    th.background_image.clear();
                }
            });
            if !th.background_image.is_empty() {
                ui.horizontal(|ui| {
                    lbl(ui, &t, "Fit");
                    combo(ui, "fit", &mut th.background_fit, labels::BACKGROUND_FIT, 150.0);
                });
                ui.horizontal(|ui| {
                    lbl(ui, &t, "Strength");
                    let mut v = th.background_opacity as f32;
                    if ui.add(egui::Slider::new(&mut v, 5.0..=100.0).suffix("%")).changed() {
                        th.background_opacity = v.round() as u8;
                    }
                });
                ui.horizontal(|ui| {
                    lbl(ui, &t, "");
                    ui.checkbox(&mut th.background_blur, "Soften the picture so text stays readable");
                });
            }

            heading(ui, &t, "Messages (MessageCraft)");
            color_row(ui, &t, "Their messages", &mut th.bubble_in, "The app's grey");
            color_row(ui, &t, "Their text", &mut th.bubble_in_text, "Readable on it");
            color_row(ui, &t, "Your messages", &mut th.bubble_out, "The app's blue");
            color_row(ui, &t, "Your text", &mut th.bubble_out_text, "Readable on it");
            ui.horizontal(|ui| {
                lbl(ui, &t, "Roundness");
                let mut auto = th.bubble_radius == 0;
                if ui.checkbox(&mut auto, "App's own").changed() {
                    th.bubble_radius = if auto { 0 } else { 14 };
                }
                if !auto {
                    let mut v = th.bubble_radius as f32;
                    if ui.add(egui::Slider::new(&mut v, 1.0..=24.0).suffix(" px")).changed() {
                        th.bubble_radius = v.round() as u8;
                    }
                }
            });

            heading(ui, &t, "Sounds");
            sound_row(ui, &t, "Ringtone", &mut th.ringtone, 1, &mut pick_sound);
            sound_row(ui, &t, "Message sound", &mut th.text_tone, 2, &mut pick_sound);
            ui.add_space(30.0);
        });

        if pick_background {
            if let Some(path) = rfd::FileDialog::new().set_title("Choose a background picture").add_filter("Pictures", &["png", "jpg", "jpeg", "gif", "webp"]).pick_file() {
                match std::fs::read(&path) {
                    Ok(bytes) => {
                        if let Some(store) = &self.store {
                            if let Err(e) = store.set_background(&mut c, &bytes) {
                                self.notify(e, true);
                            }
                        }
                        self.forget_textures();
                    }
                    Err(e) => self.notify(e.to_string(), true),
                }
            }
        }
        if pick_font {
            if let Some(path) = rfd::FileDialog::new().set_title("Choose a font file").add_filter("Fonts", &["ttf", "otf", "ttc"]).pick_file() {
                let ok = std::fs::read(&path).map(|b| theme::looks_like_font(&b)).unwrap_or(false);
                if !ok {
                    self.notify("That file isn't a font ContactCraft can read.", true);
                } else if let Some(store) = &self.store {
                    c.theme.font.clear();
                    if let Err(e) = store.add_font(&mut c, &path) {
                        self.notify(e, true);
                    }
                }
            }
        }
        if let Some(which) = pick_sound {
            if let Some(path) = rfd::FileDialog::new().set_title("Choose a sound").add_filter("Sounds", &["wav", "mp3", "m4a", "wma", "ogg", "flac"]).pick_file() {
                let s = path.display().to_string();
                if which == 1 {
                    c.theme.ringtone = s;
                } else {
                    c.theme.text_tone = s;
                }
            }
        }

        self.pane = Pane::Theme(Box::new(c.clone()));
        if details {
            self.pane = Pane::Edit(Box::new(c));
        } else if save {
            self.save_pane();
        } else if cancel {
            self.pane = Pane::Card;
        }
    }

    /// The person's card and a short conversation, drawn with the theme being edited.
    fn theme_preview(&mut self, ui: &mut Ui, area: Rect, c: &Contact) {
        let t = self.t;
        let has_image = self.store.as_ref().is_some_and(|s| s.background_path(c).is_some());
        let look = self.look(c, has_image);
        let frame = area;
        ui.painter().rect_filled(frame, 10.0, t.surface);
        self.paint_backdrop(ui, frame, c, &look, CornerRadius::same(10));
        let banner = Rect::from_min_size(frame.min, vec2(frame.width(), 96.0));
        self.paint_banner(ui, banner, c, &look, CornerRadius { nw: 10, ne: 10, sw: 0, se: 0 });
        ui.painter().rect_stroke(frame, 10.0, Stroke::new(1.0, t.border_strong), StrokeKind::Inside);
        ui.painter().text(pos2(frame.max.x - 12.0, frame.min.y + 10.0), Align2::RIGHT_TOP, "Preview", theme::medium(11.0), look.banner_ink.gamma_multiply(0.8));

        let ctx = ui.ctx().clone();
        let photo = self.photo_texture(&ctx, c);
        let ac = pos2(frame.min.x + 56.0, banner.max.y);
        let corner = match c.theme.avatar_shape.as_str() {
            "square" => 7.0,
            "rounded" => 16.0,
            _ => 36.0,
        };
        ui.painter().rect_filled(Rect::from_center_size(ac, Vec2::splat(72.0)), corner, look.bg.unwrap_or(t.surface));
        widgets::contact_avatar(ui.painter(), c, ac, 32.0, photo);
        let name = if c.display_name().is_empty() { "Their name".to_string() } else { c.display_name() };
        let fid = |scale: f32| FontId::new(look.size * scale, look.family.clone());
        let style_text = |s: &str, scale: f32, col: Color32| {
            let mut rt = egui::RichText::new(s).font(fid(scale)).color(col);
            if c.theme.bold {
                rt = rt.strong();
            }
            if c.theme.italic {
                rt = rt.italics();
            }
            rt
        };
        widgets::clipped(ui.painter(), pos2(ac.x + 48.0, banner.max.y - 24.0), frame.max.x - ac.x - 70.0, &name, FontId::new((look.size * 1.6).clamp(18.0, 36.0), look.family.clone()), look.banner_ink);
        if !c.theme.emoji.is_empty() {
            ui.painter().text(pos2(frame.max.x - 18.0, banner.min.y + 30.0), Align2::RIGHT_TOP, &c.theme.emoji, theme::regular(28.0), look.banner_ink);
        }
        if self.font_is_missing(c) {
            ui.painter().text(pos2(frame.min.x + 18.0, frame.max.y - 14.0), Align2::LEFT_BOTTOM, "That font isn't on this computer: the app's font is shown.", theme::regular(11.5), t.red);
        }

        let body = Rect::from_min_max(pos2(frame.min.x + 22.0, banner.max.y + 46.0), pos2(frame.max.x - 22.0, frame.max.y - 30.0));
        let mut u = ui.new_child(egui::UiBuilder::new().max_rect(body).layout(egui::Layout::top_down(egui::Align::Min)));
        let ui = &mut u;
        ui.spacing_mut().item_spacing.y = 4.0;
        let phone = c.phones.iter().find(|p| !p.value.is_empty()).map(|p| p.value.clone()).unwrap_or_else(|| "+1 555 010 2030".into());
        let email = c.emails.iter().find(|e| !e.address.is_empty()).map(|e| e.address.clone()).unwrap_or_else(|| "name@example.com".into());
        for (l, v, link) in [("Mobile", phone, true), ("Email", email, true), ("Note", "Text in their font, size and colour.".to_string(), false)] {
            ui.horizontal(|ui| {
                widgets::field_label(ui, 86.0, look.size + 6.0, style_text(l, 0.88, look.dim));
                ui.label(style_text(&v, 1.0, if link { look.link } else { look.text }));
            });
        }
        ui.add_space(10.0);
        // A short conversation, as MessageCraft would show it.
        let bin = theme::hex(&c.theme.bubble_in).unwrap_or(t.bubble_in);
        let bout = theme::hex(&c.theme.bubble_out).unwrap_or(t.bubble_out);
        let tin = theme::hex(&c.theme.bubble_in_text).unwrap_or_else(|| theme::ink_on(bin));
        let tout = theme::hex(&c.theme.bubble_out_text).unwrap_or_else(|| theme::ink_on(bout));
        let radius = if c.theme.bubble_radius == 0 { 14.0 } else { c.theme.bubble_radius as f32 };
        let width = ui.available_width();
        for (text, mine) in [("Hi! Are we still on for Saturday?", false), ("Yes — see you at 10.", true), ("Great, I'll bring coffee ☕", false)] {
            let job = egui::text::LayoutJob::simple(text.to_string(), fid(1.0), if mine { tout } else { tin }, width * 0.62);
            let g = ui.painter().layout_job(job);
            let size = g.size() + vec2(24.0, 14.0);
            let (row, _) = ui.allocate_exact_size(vec2(width, size.y + 4.0), Sense::hover());
            let r = if mine { Rect::from_min_size(pos2(row.max.x - size.x, row.min.y), size) } else { Rect::from_min_size(row.min, size) };
            ui.painter().rect_filled(r, radius, if mine { bout } else { bin });
            ui.painter().galley(pos2(r.min.x + 12.0, r.min.y + 7.0), g, if mine { tout } else { tin });
        }
    }
}

fn heading(ui: &mut Ui, t: &Tokens, text: &str) {
    ui.add_space(8.0);
    ui.label(egui::RichText::new(text).font(theme::semibold(14.0)).color(t.text));
    let w = ui.available_width();
    let (r, _) = ui.allocate_exact_size(vec2(w, 1.0), Sense::hover());
    ui.painter().line_segment([r.left_center(), r.right_center()], Stroke::new(1.0, t.rule));
}

fn lbl(ui: &mut Ui, t: &Tokens, text: &str) {
    widgets::field_label(ui, LABEL_W, 26.0, egui::RichText::new(text).font(theme::regular(13.0)).color(t.text_dim));
}

fn combo(ui: &mut Ui, id: &str, value: &mut String, options: &[&str], width: f32) {
    let mut cap = value.clone();
    if let Some(f) = cap.get_mut(0..1) {
        f.make_ascii_uppercase();
    }
    egui::ComboBox::from_id_salt(id).width(width).selected_text(cap).show_ui(ui, |ui| {
        for o in options {
            let mut l = o.to_string();
            if let Some(f) = l.get_mut(0..1) {
                f.make_ascii_uppercase();
            }
            ui.selectable_value(value, o.to_string(), l);
        }
    });
}

/// A colour that can be left to the app ("auto") or set.
fn color_row(ui: &mut Ui, t: &Tokens, label: &str, value: &mut String, auto_text: &str) {
    ui.horizontal(|ui| {
        lbl(ui, t, label);
        let mut own = theme::hex(value).is_some();
        if ui.checkbox(&mut own, "").on_hover_text("Choose a colour").changed() {
            *value = if own { "#0F68F0".into() } else { String::new() };
        }
        if let Some(mut col) = theme::hex(value) {
            let mut rgb = [col.r(), col.g(), col.b()];
            if egui::color_picker::color_edit_button_srgb(ui, &mut rgb).changed() {
                col = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                *value = theme::to_hex(col);
            }
            let mut text = value.clone();
            if ui.add(egui::TextEdit::singleline(&mut text).desired_width(76.0).font(theme::regular(12.5)).margin(Margin::symmetric(6, 3))).changed() && theme::hex(&text).is_some() {
                *value = text.to_uppercase();
            }
        } else {
            ui.label(egui::RichText::new(auto_text).font(theme::regular(12.5)).color(t.text_dim));
        }
    });
}

fn sound_row(ui: &mut Ui, t: &Tokens, label: &str, value: &mut String, which: u8, pick: &mut Option<u8>) {
    ui.horizontal(|ui| {
        lbl(ui, t, label);
        ui.add(egui::TextEdit::singleline(value).hint_text("The app's sound").desired_width(220.0).font(theme::regular(12.5)).margin(Margin::symmetric(8, 4)));
        if widgets::icon_button(ui, t, Icon::Import, 26.0, "Choose a sound file…").clicked() {
            *pick = Some(which);
        }
    });
}

fn preset_swatch(ui: &mut Ui, t: &Tokens, name: &str, a: &str, b: &str) -> egui::Response {
    let (r, resp) = ui.allocate_exact_size(vec2(78.0, 48.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, format!("{name} theme")));
    let ca = theme::hex(a).unwrap_or(t.pill_quiet);
    let cb = theme::hex(b).unwrap_or(t.pill_quiet);
    let sw = Rect::from_min_size(r.min, vec2(r.width(), 28.0));
    let half = Rect::from_min_max(sw.min, pos2(sw.center().x, sw.max.y));
    ui.painter().rect_filled(sw, 6.0, cb);
    ui.painter().rect_filled(half, CornerRadius { nw: 6, sw: 6, ne: 0, se: 0 }, ca);
    ui.painter().rect_stroke(sw, 6.0, Stroke::new(if resp.hovered() { 2.0 } else { 1.0 }, if resp.hovered() { t.accent } else { t.border }), StrokeKind::Inside);
    ui.painter().text(pos2(r.center().x, sw.max.y + 10.0), Align2::CENTER_CENTER, name, theme::regular(11.5), t.text_dim);
    resp
}
