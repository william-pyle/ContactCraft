//! The label pane (like MailCraft's folder pane) and the contact list.

use egui::{Align2, Rect, ScrollArea, Sense, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};

use crate::app::{App, Filter};
use crate::icons::{self, Icon};
use crate::theme;
use crate::widgets;

impl App {
    pub fn label_pane(&mut self, ui: &mut Ui) {
        let t = self.t;
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 8.0, t.panel);
        ui.painter().rect_stroke(rect, 8.0, Stroke::new(1.0, t.border), StrokeKind::Inside);
        let body = rect.shrink2(vec2(6.0, 8.0));
        let mut u = ui.new_child(egui::UiBuilder::new().max_rect(body).layout(egui::Layout::top_down(egui::Align::Min)));
        let ui = &mut u;
        ui.spacing_mut().item_spacing.y = 1.0;
        let n = |f: &Filter| self.contacts.iter().filter(|c| self.matches_filter(c, f)).count();
        let fixed = [
            (Filter::All, Icon::People, "All contacts"),
            (Filter::Favorites, Icon::Star, "Favorites"),
            (Filter::Recent, Icon::Sync, "Changed this week"),
            (Filter::Themed, Icon::Theme, "With a theme"),
            (Filter::Unlabeled, Icon::Tag, "No label"),
        ];
        let counts: Vec<usize> = fixed.iter().map(|(f, _, _)| n(f)).collect();
        let mut pick = None;
        ScrollArea::vertical().auto_shrink([false, false]).id_salt("labels").show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            heading(ui, &t, "CONTACTCRAFT");
            for ((f, icon, label), count) in fixed.iter().zip(counts) {
                if row(ui, &t, *icon, label, count, self.filter == *f, *f == Filter::All).clicked() {
                    pick = Some(f.clone());
                }
            }
            ui.add_space(10.0);
            heading(ui, &t, "LABELS");
            let labels = self.label_counts();
            if labels.is_empty() {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space(14.0);
                    ui.add(egui::Label::new(egui::RichText::new("Labels you give contacts show here.").font(theme::regular(12.0)).color(t.text_dim)).wrap());
                });
            }
            for (l, count) in labels {
                let f = Filter::Label(l.clone());
                if row(ui, &t, Icon::Tag, &l, count, self.filter == f, false).clicked() {
                    pick = Some(f);
                }
            }
            ui.add_space(4.0);
            let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 30.0), Sense::click());
            resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, "New label"));
            if resp.hovered() {
                ui.painter().rect_filled(r.shrink2(vec2(2.0, 1.0)), 4.0, t.hover_subtle);
            }
            icons::paint(ui.painter(), Rect::from_min_size(pos2(r.min.x + 12.0, r.center().y - 8.0), Vec2::splat(16.0)), Icon::Plus, t.accent_text, t.accent_text);
            ui.painter().text(pos2(r.min.x + 38.0, r.center().y), Align2::LEFT_CENTER, "New label", theme::regular(13.0), t.accent_text);
            if resp.clicked() {
                self.new_label = Some(String::new());
            }
            ui.add_space(10.0);
            heading(ui, &t, "GROUPS");
            let groups: Vec<(String, String, usize)> = self.groups.iter().map(|g| (g.id.clone(), g.name.clone(), g.members.len())).collect();
            for (id, name, count) in groups {
                if row(ui, &t, Icon::People, &name, count, self.filter == Filter::Group(id.clone()), false).clicked() {
                    pick = Some(Filter::Group(id));
                }
            }
            if ui.button("+ New group").clicked() {
                self.new_group();
            }
        });
        if let Some(f) = pick {
            if let Filter::Group(id) = &f {
                self.select_group(id);
            } else {
                self.active_group = None;
                self.filter = f;
            }
        }
    }

    pub fn contact_list(&mut self, ui: &mut Ui) {
        let t = self.t;
        let rect = ui.max_rect();
        ui.expand_to_include_rect(rect);
        ui.painter().rect_filled(rect, 8.0, t.surface);
        ui.painter().rect_stroke(rect, 8.0, Stroke::new(1.0, t.border), StrokeKind::Inside);

        let header = Rect::from_min_size(rect.min, vec2(rect.width(), 46.0));
        let title = self.filter_label();
        let g = ui.painter().text(pos2(header.min.x + 18.0, header.center().y), Align2::LEFT_CENTER, &title, theme::regular(15.0), t.accent_text);
        ui.painter().rect_filled(Rect::from_min_size(pos2(header.min.x + 18.0, header.center().y + 12.0), vec2(g.width(), 2.0)), 1.0, t.accent);
        let shown_n = self.shown().len();
        ui.painter().text(pos2(header.max.x - 16.0, header.center().y), Align2::RIGHT_CENTER, format!("{shown_n}"), theme::regular(12.0), t.text_dim);

        let body = Rect::from_min_max(pos2(rect.min.x + 4.0, header.max.y), pos2(rect.max.x - 2.0, rect.max.y - 4.0));
        let mut u = ui.new_child(egui::UiBuilder::new().max_rect(body).layout(egui::Layout::top_down(egui::Align::Min)));
        let ui = &mut u;
        if self.store.is_none() {
            ui.add_space(30.0);
            ui.vertical_centered(|ui| ui.label(egui::RichText::new("The contacts couldn't be opened.").font(theme::regular(13.0)).color(t.text_dim)));
            return;
        }
        let shown: Vec<contactcraft_store::Contact> = self.shown().into_iter().cloned().collect();
        if shown.is_empty() {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                let msg = if !self.search.is_empty() {
                    "No contacts match your search."
                } else if matches!(self.filter, Filter::Group(_)) {
                    "No saved contacts in this group."
                } else if self.contacts.is_empty() {
                    "No contacts yet."
                } else {
                    "No contacts here."
                };
                ui.label(egui::RichText::new(msg).font(theme::medium(14.0)).color(t.text));
                if self.contacts.is_empty() {
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new("Add one, or bring yours in from Google or Outlook.").font(theme::regular(12.5)).color(t.text_dim));
                    ui.add_space(12.0);
                    if widgets::primary_button(ui, &t, "Import contacts", true).clicked() {
                        self.pick_import_file();
                    }
                    ui.add_space(6.0);
                    if widgets::secondary_button(ui, &t, "New contact").clicked() {
                        self.new_contact();
                    }
                }
            });
            return;
        }
        let row_h = if self.settings.tighter { 44.0 } else { 56.0 };
        let mut clicked = None;
        let mut fav = None;
        let editing_id = match &self.pane {
            crate::app::Pane::Edit(c) | crate::app::Pane::Theme(c) => Some(c.id.clone()),
            _ => None,
        };
        let ctx = ui.ctx().clone();
        ScrollArea::vertical().auto_shrink([false, false]).id_salt("contact_list").show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            let w = ui.available_width();
            let mut last_letter = None;
            for c in &shown {
                let label = c.display_name();
                let sort = if self.settings.sort_by_family && !c.name.family.is_empty() { c.name.family.clone() } else { c.sort_key() };
                let letter = sort.chars().next().map(|ch| ch.to_uppercase().next().unwrap_or(ch)).filter(|ch| ch.is_alphabetic()).unwrap_or('#');
                if last_letter != Some(letter) && self.search.is_empty() {
                    let (gr, _) = ui.allocate_exact_size(vec2(w, 25.0), Sense::hover());
                    ui.painter().text(pos2(gr.min.x + 14.0, gr.center().y + 2.0), Align2::LEFT_CENTER, letter.to_string(), theme::medium(12.0), t.group_label);
                    last_letter = Some(letter);
                }
                let (r, resp) = ui.allocate_exact_size(vec2(w, row_h), Sense::click());
                let sel = self.selected.as_deref() == Some(c.id.as_str()) || editing_id.as_deref() == Some(c.id.as_str());
                resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, sel, label.clone()));
                if resp.clicked() {
                    clicked = Some(c.id.clone());
                }
                if resp.double_clicked() {
                    clicked = Some(c.id.clone());
                }
                if !ui.is_rect_visible(r) {
                    continue;
                }
                let row = Rect::from_min_max(pos2(r.min.x + 2.0, r.min.y + 1.0), pos2(r.max.x - 6.0, r.max.y - 1.0));
                if sel {
                    ui.painter().rect_filled(row, 4.0, t.selection);
                    ui.painter().rect_filled(Rect::from_min_size(pos2(row.min.x, row.min.y + 6.0), vec2(3.0, row.height() - 12.0)), 1.5, t.selection_edge);
                } else if resp.hovered() {
                    ui.painter().rect_filled(row, 4.0, t.hover_subtle);
                }
                let photo = self.photo_texture(&ctx, c);
                let p = ui.painter();
                let radius = if self.settings.tighter { 13.0 } else { 16.0 };
                widgets::contact_avatar(p, c, pos2(row.min.x + 26.0, row.center().y), radius, photo);
                // A theme shows as a thin stripe of its colours by the avatar.
                if let Some(a) = theme::hex(&c.theme.accent) {
                    let a2 = theme::hex(&c.theme.accent2).unwrap_or(a);
                    let s = Rect::from_min_size(pos2(row.min.x + 8.0, row.center().y - radius), vec2(2.5, radius));
                    p.rect_filled(s, 1.0, a);
                    p.rect_filled(s.translate(vec2(0.0, radius)), 1.0, a2);
                }
                let left = row.min.x + 50.0;
                let right_room = 34.0;
                let mut name = label.clone();
                if !c.theme.emoji.is_empty() {
                    name = format!("{name}  {}", c.theme.emoji);
                }
                if self.settings.tighter {
                    widgets::clipped(p, pos2(left, row.center().y), row.max.x - left - right_room, &name, theme::regular(13.5), t.text);
                } else {
                    widgets::clipped(p, pos2(left, row.min.y + 19.0), row.max.x - left - right_room, &name, theme::regular(14.0), t.text);
                    let sub = sub_line(c);
                    widgets::clipped(p, pos2(left, row.min.y + 38.0), row.max.x - left - right_room, &sub, theme::regular(11.5), t.text_dim);
                }
                // Favorite star: always for favorites, on hover for the rest.
                let star = Rect::from_center_size(pos2(row.max.x - 16.0, row.center().y), Vec2::splat(22.0));
                let hover_star = ui.rect_contains_pointer(star);
                if c.favorite || resp.hovered() || hover_star {
                    let col = if c.favorite { t.icon_yellow } else if hover_star { t.icon } else { t.text_disabled };
                    if hover_star {
                        ui.painter().rect_filled(star, 4.0, t.hover);
                    }
                    icons::paint(ui.painter(), star.shrink(4.0), Icon::Star, col, col);
                    if hover_star && resp.clicked() {
                        fav = Some(c.id.clone());
                        clicked = None;
                    }
                }
            }
            ui.add_space(8.0);
        });
        if let Some(id) = fav {
            self.toggle_favorite(&id);
        } else if let Some(id) = clicked {
            self.select(&id);
        }
    }

    pub fn new_label_dialog(&mut self, ctx: &egui::Context) {
        let t = self.t;
        let Some(mut name) = self.new_label.clone() else { return };
        let mut done = false;
        let mut close = false;
        let modal = egui::Modal::new(egui::Id::unique("new_label")).frame(crate::chrome::dialog_frame(&t)).show(ctx, |ui| {
            ui.set_width(380.0);
            ui.label(egui::RichText::new("New label").font(theme::semibold(17.0)).color(t.text));
            let sel = self.selected_contact().map(|c| c.display_name());
            ui.label(
                egui::RichText::new(match &sel {
                    Some(n) => format!("The label is given to {n}; give it to others from their Edit page."),
                    None => "Give it to contacts from their Edit page.".to_string(),
                })
                .font(theme::regular(12.5))
                .color(t.text_dim),
            );
            ui.add_space(6.0);
            let r = ui.add(egui::TextEdit::singleline(&mut name).hint_text("Label name").desired_width(340.0).margin(egui::Margin::symmetric(8, 6)));
            r.request_focus();
            if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                done = true;
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if widgets::primary_button(ui, &t, "Create", !name.trim().is_empty()).clicked() {
                    done = true;
                }
                if widgets::secondary_button(ui, &t, "Cancel").clicked() {
                    close = true;
                }
            });
        });
        if done && !name.trim().is_empty() {
            self.new_label = None;
            let label = name.trim().to_string();
            if let Some(mut c) = self.selected_contact() {
                if !c.labels.iter().any(|l| l.eq_ignore_ascii_case(&label)) {
                    c.labels.push(label.clone());
                    self.save_contact(c);
                }
            }
            self.filter = Filter::Label(label);
        } else if close || modal.should_close() {
            self.new_label = None;
        } else {
            self.new_label = Some(name);
        }
    }
}

/// The second line of a list row: job and company, else a phone number or an email address.
fn sub_line(c: &contactcraft_store::Contact) -> String {
    let role = [c.job_title(), c.company()].iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join(", ");
    if !role.is_empty() && role != c.display_name() {
        return role;
    }
    c.phones
        .iter()
        .find(|p| !p.value.is_empty())
        .map(|p| p.value.clone())
        .or_else(|| c.emails.iter().find(|e| !e.address.is_empty()).map(|e| e.address.clone()))
        .unwrap_or_default()
}

fn heading(ui: &mut Ui, t: &theme::Tokens, text: &str) {
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 26.0), Sense::hover());
    ui.painter().text(pos2(r.min.x + 12.0, r.center().y + 2.0), Align2::LEFT_CENTER, text, theme::semibold(10.5), t.group_label);
}

fn row(ui: &mut Ui, t: &theme::Tokens, icon: Icon, label: &str, count: usize, selected: bool, strong: bool) -> egui::Response {
    let (r, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, selected, label));
    let p = ui.painter();
    let row = r.shrink2(vec2(2.0, 1.0));
    if selected {
        p.rect_filled(row, 4.0, t.selection);
        p.rect_filled(Rect::from_min_size(pos2(row.min.x, row.min.y + 7.0), vec2(3.0, row.height() - 14.0)), 1.5, t.selection_edge);
    } else if resp.hovered() {
        p.rect_filled(row, 4.0, t.hover_subtle);
    }
    let ink = if selected { t.accent } else { t.icon };
    icons::paint(p, Rect::from_min_size(pos2(row.min.x + 12.0, row.center().y - 8.0), Vec2::splat(16.0)), icon, ink, if selected { t.accent } else { t.text_dim });
    let font = if selected { theme::semibold(13.0) } else { theme::regular(13.0) };
    widgets::clipped(p, pos2(row.min.x + 38.0, row.center().y), row.width() - 80.0, label, font, t.text);
    if count > 0 {
        widgets::count_pill(p, t, pos2(row.max.x - 8.0, row.center().y), count, strong);
    }
    resp
}
