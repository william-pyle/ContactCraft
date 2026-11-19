//! The window's frame in MailCraft's layout (as MessageCraft has it): the title bar with search,
//! the tab strip, the ribbon, the module rail down the left, and the status bar.

use egui::{Align2, Color32, Frame, Margin, Panel, Rect, Sense, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};

use crate::app::{App, Appearance, Backstage, Filter, Pane, RibbonTab};
use crate::icons::{self, Icon};
use crate::theme;
use crate::widgets::{self, icon_button, ribbon_button, ribbon_rule};

pub const TITLE_BAR: f32 = 48.0;
const SEARCH_BOX: f32 = 34.0;
const TAB_H: f32 = 30.0;
const BAND_H: f32 = 48.0;
pub const STATUS_BAR_H: f32 = 30.0;
const RAIL_W: f32 = 48.0;

impl App {
    pub fn draw(&mut self, ui: &mut Ui) {
        let t = self.t;
        Panel::top("title_bar").exact_size(TITLE_BAR).frame(Frame::new().fill(t.title_bar)).show(ui, |ui| self.title_bar(ui));
        if self.in_dialog() {
            self.draw_dialog(ui);
            let ctx = ui.ctx().clone();
            self.dialogs(&ctx);
            self.notice_overlay(&ctx);
            return;
        }
        Panel::bottom("status_bar").exact_size(STATUS_BAR_H).frame(Frame::new().fill(t.status_bar)).show(ui, |ui| self.status_bar(ui));
        Panel::left("rail").exact_size(RAIL_W).resizable(false).show_separator_line(false).frame(Frame::new().fill(t.nav)).show(ui, |ui| self.rail(ui));
        Panel::top("tabs").exact_size(TAB_H).show_separator_line(false).frame(Frame::new().fill(t.tab_strip)).show(ui, |ui| self.tab_strip(ui));

        if let Some(page) = self.backstage {
            egui::CentralPanel::no_frame().show(ui, |ui| self.backstage_view(ui, page));
        } else {
            Panel::top("ribbon")
                .exact_size(BAND_H + 12.0)
                .show_separator_line(false)
                .frame(Frame::new().fill(t.canvas).inner_margin(Margin { left: 4, right: 10, top: 5, bottom: 7 }))
                .show(ui, |ui| self.ribbon(ui));
            if self.settings.label_pane {
                Panel::left("labels")
                    .default_size(230.0)
                    .size_range(170.0..=340.0)
                    .resizable(true)
                    .show_separator_line(false)
                    .frame(Frame::new().fill(t.canvas).inner_margin(Margin { left: 0, right: 6, top: 0, bottom: 8 }))
                    .show(ui, |ui| self.label_pane(ui));
            }
            Panel::left("list")
                .default_size(360.0)
                .size_range(260.0..=620.0)
                .resizable(true)
                .show_separator_line(false)
                .frame(Frame::new().fill(t.canvas).inner_margin(Margin { left: if self.settings.label_pane { 0 } else { 4 }, right: 6, top: 0, bottom: 8 }))
                .show(ui, |ui| self.contact_list(ui));
            egui::CentralPanel::no_frame()
                .frame(Frame::new().fill(t.canvas).inner_margin(Margin { left: 0, right: 10, top: 0, bottom: 8 }))
                .show(ui, |ui| self.right_pane(ui));
        }

        let ctx = ui.ctx().clone();
        self.dialogs(&ctx);
        self.notice_overlay(&ctx);
    }

    fn right_pane(&mut self, ui: &mut Ui) {
        if self.active_group.is_some() {
            self.group_editor(ui);
            return;
        }
        match self.pane.clone() {
            Pane::Card => self.contact_card(ui),
            Pane::Edit(_) => self.contact_editor(ui),
            Pane::Theme(_) => self.theme_editor(ui),
        }
    }

    // -----------------------------------------------------------------------------------------

    fn title_bar(&mut self, ui: &mut Ui) {
        let t = self.t;
        let rect = ui.max_rect();
        let p = ui.painter().clone();
        p.line_segment([pos2(rect.min.x, rect.max.y - 0.5), pos2(rect.max.x, rect.max.y - 0.5)], Stroke::new(1.0, t.border.linear_multiply(0.6)));

        // App mark: the family's blue tile, with a solid white person.
        let mark = Rect::from_min_size(pos2(rect.min.x + 12.0, rect.center().y - 12.0), Vec2::splat(24.0));
        p.rect_filled(mark, 6.0, t.accent);
        icons::paint(&p, mark.shrink(3.5), Icon::PersonSolid, Color32::WHITE, Color32::WHITE);
        let menu = ui.interact(mark.expand(4.0), ui.scope_id().with("system_menu"), Sense::click());
        menu.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, "Window menu (Alt+Space)"));
        if menu.clicked() || menu.secondary_clicked() {
            if let Err(error) = self.window_chrome.system_menu() { self.notify(error, true); }
        }
        menu.on_hover_text("Window menu (Alt+Space)");

        let controls = Rect::from_min_max(pos2(rect.max.x - 126.0, rect.min.y), rect.max);
        self.window_buttons(ui, controls);
        let mut interactive = vec![mark.expand(4.0), controls];
        if let Some(dialog) = &self.dialog {
            p.with_clip_rect(Rect::from_min_max(pos2(mark.max.x + 10.0, rect.min.y), pos2(controls.min.x - 8.0, rect.max.y)))
                .text(pos2(mark.max.x + 14.0, rect.center().y), Align2::LEFT_CENTER, crate::dialog::title(&dialog.request), theme::semibold(13.0), t.text);
            self.window_chrome.set_toolbar(rect, interactive, ui.ctx().pixels_per_point());
            return;
        }

        // Quick access: new contact and import.
        let mut qat = ui.new_child(egui::UiBuilder::new().max_rect(Rect::from_min_size(pos2(mark.max.x + 10.0, rect.center().y - 14.0), vec2(80.0, 28.0))).layout(egui::Layout::left_to_right(egui::Align::Center)));
        qat.spacing_mut().item_spacing.x = 2.0;
        if icon_button(&mut qat, &t, Icon::Plus, 28.0, "New contact (Ctrl+N)").clicked() {
            self.new_contact();
        }
        if icon_button(&mut qat, &t, Icon::Import, 28.0, "Import contacts from a file").clicked() {
            self.pick_import_file();
        }
        interactive.push(qat.min_rect().expand(2.0));

        // Search, centred like MailCraft's.
        let compact = rect.width() < 900.0;
        let sharing_width = if compact { 38.0 } else { 206.0 };
        let search_left = rect.min.x + 140.0;
        let search_right = controls.min.x - sharing_width - 20.0;
        let w = (rect.width() * 0.36).clamp(200.0, 540.0).min((search_right - search_left).max(80.0));
        let center = rect.center().x.clamp(search_left + w / 2.0, search_right - w / 2.0);
        let sr = Rect::from_center_size(pos2(center, rect.center().y), vec2(w, SEARCH_BOX));
        interactive.push(sr);
        let focused = ui.memory(|m| m.focused() == Some(egui::Id::unique("search_box")));
        p.rect_filled(sr, 6.0, t.input);
        p.rect_stroke(sr, 6.0, Stroke::new(1.0, if focused { t.accent } else { t.border_strong }), StrokeKind::Inside);
        icons::paint(&p, Rect::from_min_size(pos2(sr.min.x + 10.0, sr.center().y - 8.0), Vec2::splat(16.0)), Icon::Search, t.text_dim, t.text_dim);
        let show_filter = self.search.is_empty() && w >= 300.0;
        let edit_rect = Rect::from_min_max(pos2(sr.min.x + 34.0, sr.min.y + 2.0), pos2(sr.max.x - if show_filter { 120.0 } else { 34.0 }, sr.max.y - 2.0));
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(edit_rect).layout(egui::Layout::left_to_right(egui::Align::Center)));
        let edit = egui::TextEdit::singleline(&mut self.search)
            .id(egui::Id::unique("search_box"))
            .frame(Frame::NONE)
            .hint_text(egui::RichText::new("Search contacts").color(t.text_dim))
            .font(theme::regular(13.0))
            .text_color(t.text)
            .desired_width(edit_rect.width());
        let resp = child.add(edit);
        if resp.changed() {
            self.backstage = None;
        }
        if self.focus_search {
            resp.request_focus();
            self.focus_search = false;
        }
        if show_filter {
            p.text(pos2(sr.max.x - 12.0, sr.center().y), Align2::RIGHT_CENTER, self.filter_label(), theme::regular(12.0), t.text_dim);
        } else if !self.search.is_empty() {
            let cr = Rect::from_center_size(pos2(sr.max.x - 17.0, sr.center().y), Vec2::splat(22.0));
            let mut c = ui.new_child(egui::UiBuilder::new().max_rect(cr));
            if icon_button(&mut c, &t, Icon::Close, 22.0, "Clear search").clicked() {
                self.search.clear();
            }
        }

        // Right: whether the other Craft apps can see the contacts.
        let right = Rect::from_min_max(pos2(sr.max.x + 12.0, rect.min.y), pos2(controls.min.x - 8.0, rect.max.y));
        let mut r = ui.new_child(egui::UiBuilder::new().max_rect(right).layout(egui::Layout::right_to_left(egui::Align::Center)));
        r.spacing_mut().item_spacing.x = 8.0;
        let (dot, text) = match (&self.store, self.demo) {
            (Some(_), false) => (t.icon_green, "Shared with Craft apps"),
            (Some(_), true) => (t.icon_yellow, "Demo: nothing is saved"),
            (None, _) => (t.icon_red, "Contacts not open"),
        };
        let sharing = if compact { icon_button(&mut r, &t, Icon::Share, 28.0, text) } else { widgets::chip(&mut r, &t, Icon::Share, text, Some(dot)) };
        interactive.push(sharing.rect);
        if sharing.on_hover_text("How MessageCraft and MailCraft use these contacts").clicked() {
            self.backstage = Some(Backstage::Sharing);
        }
        self.window_chrome.set_toolbar(rect, interactive, ui.ctx().pixels_per_point());
    }

    fn window_buttons(&mut self, ui: &mut Ui, controls: Rect) {
        let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        for (index, label) in ["Minimize window", if maximized { "Restore window" } else { "Maximize window" }, "Close window (Alt+F4)"].iter().enumerate() {
            let rect = Rect::from_min_size(pos2(controls.min.x + index as f32 * 42.0, controls.min.y), vec2(42.0, controls.height()));
            let response = ui.interact(rect, ui.scope_id().with(("window_button", index)), Sense::click());
            response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, *label));
            let ink = if index == 2 && response.hovered() { Color32::WHITE } else { self.t.text };
            if response.hovered() || response.has_focus() {
                ui.painter().rect_filled(rect, 0.0, if index == 2 { Color32::from_rgb(196, 43, 59) } else { self.t.hover });
            }
            let glyph = Rect::from_center_size(rect.center(), Vec2::splat(10.0));
            let stroke = Stroke::new(1.0, ink);
            match index {
                0 => { ui.painter().line_segment([pos2(glyph.min.x, glyph.center().y), pos2(glyph.max.x, glyph.center().y)], stroke); }
                1 => {
                    if maximized {
                        let back = glyph.translate(vec2(2.0, -2.0));
                        ui.painter().line_segment([back.left_top(), back.right_top()], stroke);
                        ui.painter().line_segment([back.right_top(), back.right_bottom()], stroke);
                    }
                    ui.painter().rect_stroke(glyph, 0.0, stroke, StrokeKind::Inside);
                }
                _ => {
                    ui.painter().line_segment([glyph.left_top(), glyph.right_bottom()], stroke);
                    ui.painter().line_segment([glyph.right_top(), glyph.left_bottom()], stroke);
                }
            }
            if response.clicked() {
                ui.ctx().send_viewport_cmd(match index {
                    0 => egui::ViewportCommand::Minimized(true),
                    1 => egui::ViewportCommand::Maximized(!maximized),
                    _ => egui::ViewportCommand::Close,
                });
            }
            response.on_hover_text(*label);
        }
    }

    pub fn filter_label(&self) -> String {
        match &self.filter {
            Filter::All => "All contacts".into(),
            Filter::Favorites => "Favorites".into(),
            Filter::Themed => "With a theme".into(),
            Filter::Recent => "Changed this week".into(),
            Filter::Unlabeled => "No label".into(),
            Filter::Label(l) => l.clone(),
            Filter::Group(id) => self.group(id).map(|g| g.name.clone()).unwrap_or_else(|| "Group".into()),
        }
    }

    fn tab_strip(&mut self, ui: &mut Ui) {
        let t = self.t;
        let rect = ui.max_rect();
        let mut x = rect.min.x + 12.0;
        let font = theme::regular(13.333);
        let tabs: [(&str, Option<RibbonTab>); 4] = [("File", None), ("Home", Some(RibbonTab::Home)), ("View", Some(RibbonTab::View)), ("Help", Some(RibbonTab::Help))];
        for (label, tab) in tabs {
            let galley = ui.painter().layout_no_wrap(label.to_string(), font.clone(), t.text);
            let w = galley.size().x + 22.0;
            let r = Rect::from_min_size(pos2(x, rect.min.y), vec2(w, rect.height()));
            let resp = ui.interact(r, egui::Id::unique(("tab", label)), Sense::click());
            let selected = match tab {
                None => self.backstage.is_some(),
                Some(tb) => self.backstage.is_none() && self.ribbon == tb,
            };
            resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, selected, label));
            if resp.hovered() && !selected {
                ui.painter().rect_filled(r.shrink2(vec2(2.0, 3.0)), 4.0, t.hover);
            }
            let col = if selected { t.text } else { t.text_dim };
            let f = if selected { theme::semibold(13.333) } else { font.clone() };
            ui.painter().text(pos2(r.center().x, r.center().y), Align2::CENTER_CENTER, label, f, col);
            if selected {
                let gw = galley.size().x;
                let ul = Rect::from_center_size(pos2(r.center().x, r.max.y - 3.0), vec2(gw, 2.0));
                ui.painter().rect_filled(ul, 1.0, t.accent);
            }
            if resp.clicked() {
                match tab {
                    None => self.backstage = if self.backstage.is_some() { None } else { Some(Backstage::Info) },
                    Some(tb) => {
                        self.backstage = None;
                        self.ribbon = tb;
                    }
                }
            }
            x += w + 2.0;
        }
    }

    fn ribbon(&mut self, ui: &mut Ui) {
        let t = self.t;
        let sheet = ui.max_rect();
        ui.painter().rect_filled(sheet, 8.0, t.ribbon);
        ui.painter().rect_stroke(sheet, 8.0, t.ribbon_edge, StrokeKind::Inside);
        let inner = sheet.shrink2(vec2(8.0, (sheet.height() - 32.0) / 2.0));
        let mut ui = ui.new_child(egui::UiBuilder::new().max_rect(inner).layout(egui::Layout::left_to_right(egui::Align::Center)));
        ui.spacing_mut().item_spacing.x = 2.0;
        let ui = &mut ui;
        let sel = self.selected_contact();
        let has = sel.is_some() && !self.editing();
        match self.ribbon {
            RibbonTab::Home => {
                if self.editing() {
                    if widgets::ribbon_button(ui, &t, Icon::Check, "Save", false, true, false).on_hover_text("Ctrl+S").clicked() {
                        self.save_pane();
                    }
                    if ribbon_button(ui, &t, Icon::Close, "Cancel", false, true, false).clicked() {
                        self.set_pane(Pane::Card);
                    }
                    ribbon_rule(ui, &t);
                    let is_theme = matches!(self.pane, Pane::Theme(_));
                    if ribbon_button(ui, &t, Icon::Edit, "Details", !is_theme, true, false).clicked() {
                        if let Pane::Theme(c) = self.pane.clone() {
                            self.pane = Pane::Edit(c);
                        }
                    }
                    if ribbon_button(ui, &t, Icon::Theme, "Theme", is_theme, true, false).clicked() {
                        if let Pane::Edit(c) = self.pane.clone() {
                            self.pane = Pane::Theme(c);
                        }
                    }
                    return;
                }
                if ribbon_button(ui, &t, Icon::Plus, "New Contact", false, self.store.is_some(), false).clicked() {
                    self.new_contact();
                }
                if ribbon_button(ui, &t, Icon::People, "New Group", false, self.store.is_some(), false).clicked() {
                    self.new_group();
                }
                ribbon_rule(ui, &t);
                if ribbon_button(ui, &t, Icon::Edit, "Edit", false, has, false).clicked() {
                    if let Some(c) = sel.clone() {
                        self.pane = Pane::Edit(Box::new(c));
                    }
                }
                if ribbon_button(ui, &t, Icon::Theme, "Theme", false, has, false).clicked() {
                    if let Some(c) = sel.clone() {
                        self.pane = Pane::Theme(Box::new(c));
                    }
                }
                let fav = sel.as_ref().is_some_and(|c| c.favorite);
                if ribbon_button(ui, &t, Icon::Star, "Favorite", fav, has, false).clicked() {
                    if let Some(c) = &sel {
                        self.toggle_favorite(&c.id);
                    }
                }
                if ribbon_button(ui, &t, Icon::Trash, "Delete", false, has, false).clicked() {
                    self.confirm_delete = sel.as_ref().map(|c| c.id.clone());
                }
                ribbon_rule(ui, &t);
                if ribbon_button(ui, &t, Icon::Clean, "Clean Up", false, !self.contacts.is_empty(), false)
                    .on_hover_text("Tidy names, numbers and addresses, and merge duplicates")
                    .clicked()
                {
                    self.open_cleanup();
                }
                ribbon_rule(ui, &t);
                let email = sel.as_ref().and_then(|c| c.emails.iter().find(|e| !e.address.is_empty())).map(|e| e.address.clone());
                if ribbon_button(ui, &t, Icon::Mail, "Email", false, has && email.is_some(), false).clicked() {
                    if let Some(e) = email {
                        crate::app::open_url(&format!("mailto:{e}"));
                    }
                }
                if ribbon_button(ui, &t, Icon::Copy, "Copy Card", false, has, false).on_hover_text("Copy the name, numbers and addresses as text").clicked() {
                    if let Some(c) = &sel {
                        ui.ctx().copy_text(crate::card::as_text(c));
                        self.notify("Copied.", false);
                    }
                }
                ribbon_rule(ui, &t);
                if ribbon_button(ui, &t, Icon::Import, "Import", false, self.store.is_some(), false).clicked() {
                    self.pick_import_file();
                }
                if ribbon_button(ui, &t, Icon::Export, "Export", false, !self.contacts.is_empty(), false).clicked() {
                    self.backstage = Some(Backstage::Export);
                }
            }
            RibbonTab::View => {
                if ribbon_button(ui, &t, Icon::FolderPane, "Label Pane", self.settings.label_pane, true, false).clicked() {
                    self.settings.label_pane = !self.settings.label_pane;
                    self.save_settings();
                }
                if ribbon_button(ui, &t, Icon::Spacing, "Tighter Spacing", self.settings.tighter, true, false).clicked() {
                    self.settings.tighter = !self.settings.tighter;
                    self.save_settings();
                }
                if ribbon_button(ui, &t, Icon::Person, "Sort by Last Name", self.settings.sort_by_family, true, false).clicked() {
                    self.settings.sort_by_family = !self.settings.sort_by_family;
                    self.save_settings();
                    self.reload();
                }
                ribbon_rule(ui, &t);
                for (a, label) in [(Appearance::System, "System Theme"), (Appearance::Light, "Light"), (Appearance::Dark, "Dark")] {
                    if ribbon_button(ui, &t, Icon::Theme, label, self.settings.appearance == a, true, false).clicked() {
                        self.settings.appearance = a;
                        self.apply_settings(ui.ctx());
                        self.save_settings();
                    }
                }
                ribbon_rule(ui, &t);
                if ribbon_button(ui, &t, Icon::Zoom, &format!("Zoom {}%", self.settings.zoom), false, true, true).clicked() {
                    let presets = [80u16, 90, 100, 110, 125, 150, 175, 200];
                    let i = presets.iter().position(|z| *z == self.settings.zoom).unwrap_or(2);
                    self.settings.zoom = presets[(i + 1) % presets.len()];
                    self.apply_settings(ui.ctx());
                    self.save_settings();
                }
                if ribbon_button(ui, &t, Icon::Undo, "Reset View", false, true, false).clicked() {
                    self.settings = crate::app::Settings { appearance: self.settings.appearance, ..Default::default() };
                    self.apply_settings(ui.ctx());
                    self.save_settings();
                    self.reload();
                }
            }
            RibbonTab::Help => {
                if ribbon_button(ui, &t, Icon::Share, "Sharing with Craft Apps", false, true, false).clicked() {
                    self.backstage = Some(Backstage::Sharing);
                }
                if ribbon_button(ui, &t, Icon::Import, "How to Import", false, true, false).clicked() {
                    self.backstage = Some(Backstage::Import);
                }
                if ribbon_button(ui, &t, Icon::FolderPane, "Open Data Folder", false, self.store.is_some(), false).clicked() {
                    if let Some(s) = &self.store {
                        crate::app::open_path(s.dir());
                    }
                }
                let checking = matches!(self.updater.state, crate::update::UpdateState::Checking);
                if ribbon_button(ui, &t, Icon::Sync, "Check for Updates", checking, true, false).clicked() {
                    let ctx = ui.ctx().clone();
                    self.check_for_updates(&ctx, true);
                }
                if ribbon_button(ui, &t, Icon::Info, "About ContactSolutions", false, true, false).clicked() {
                    self.backstage = Some(Backstage::About);
                }
            }
        }
    }

    fn rail(&mut self, ui: &mut Ui) {
        let t = self.t;
        let rect = ui.max_rect();
        let items = [(Icon::People, "Contacts"), (Icon::Star, "Favorites"), (Icon::Import, "Import and export"), (Icon::Gear, "Options")];
        let mut y = rect.min.y + 4.0 + TAB_H;
        for (i, (icon, tip)) in items.iter().enumerate() {
            let r = Rect::from_min_size(pos2(rect.min.x, y), vec2(RAIL_W, 48.0));
            let resp = ui.interact(r, egui::Id::unique(("rail", i)), Sense::click()).on_hover_text(*tip);
            resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, *tip));
            let active = match i {
                0 => self.backstage.is_none() && self.filter != Filter::Favorites,
                1 => self.backstage.is_none() && self.filter == Filter::Favorites,
                2 => matches!(self.backstage, Some(Backstage::Import) | Some(Backstage::Export)),
                _ => self.backstage == Some(Backstage::Options),
            };
            if resp.hovered() {
                ui.painter().rect_filled(r.shrink(6.0), 6.0, t.hover);
            }
            if active {
                ui.painter().rect_filled(Rect::from_min_size(pos2(r.min.x + 2.0, r.center().y - 10.0), vec2(3.0, 20.0)), 1.5, t.accent);
            }
            let ir = Rect::from_center_size(r.center(), Vec2::splat(22.0));
            let col = if active { t.accent } else { t.icon };
            icons::paint(ui.painter(), ir, *icon, col, if active { t.accent } else { t.icon });
            if resp.clicked() {
                match i {
                    0 => {
                        self.backstage = None;
                        if self.filter == Filter::Favorites {
                            self.filter = Filter::All;
                        }
                    }
                    1 => {
                        self.backstage = None;
                        self.filter = Filter::Favorites;
                    }
                    2 => self.backstage = Some(Backstage::Import),
                    _ => self.backstage = Some(Backstage::Options),
                }
            }
            y += 48.0;
        }
    }

    fn status_bar(&mut self, ui: &mut Ui) {
        let t = self.t;
        let rect = ui.max_rect();
        let p = ui.painter();
        let font = theme::regular(12.0);
        let y = rect.center().y;
        let n = self.contacts.len();
        let shown = self.shown().len();
        let fav = self.contacts.iter().filter(|c| c.favorite).count();
        let left = if shown == n { format!("{n} contacts") } else { format!("{shown} of {n} contacts") };
        let lw = p.text(pos2(rect.min.x + 12.0, y), Align2::LEFT_CENTER, left, font.clone(), t.text_dim).width();
        let fw = p.text(pos2(rect.min.x + 12.0 + lw + 28.0, y), Align2::LEFT_CENTER, format!("Favorites: {fav}"), font.clone(), t.text_dim).width();
        if let Some(badge) = self.update_badge() {
            let x = rect.min.x + 12.0 + lw + 28.0 + fw + 28.0;
            let g = p.layout_no_wrap(badge.clone(), theme::semibold(12.0), t.accent_text);
            let br = Rect::from_min_max(pos2(x - 6.0, rect.min.y + 4.0), pos2(x + g.size().x + 6.0, rect.max.y - 4.0));
            let resp = ui.interact(br, egui::Id::unique("update_badge"), Sense::click()).on_hover_text("Show the new version");
            let p = ui.painter();
            if resp.hovered() {
                p.rect_filled(br, 4.0, t.hover);
            }
            p.text(pos2(x, y), Align2::LEFT_CENTER, badge, theme::semibold(12.0), t.accent_text);
            if resp.clicked() {
                self.show_update_prompt();
            }
        }
        let p = ui.painter();
        let (text, col) = match (&self.store, &self.store_error) {
            (Some(_), _) if self.demo => ("Demo contacts — changes are thrown away when you close".to_string(), t.orange),
            (Some(_), _) if self.unsaved() => ("Not saved yet".to_string(), t.orange),
            (Some(s), _) => (format!("Saved — shared with your Craft apps ({})", s.dir().display()), t.text_dim),
            (None, Some(e)) => (format!("The contacts couldn't be opened: {e}"), t.red),
            (None, None) => ("Not open".into(), t.red),
        };
        p.text(pos2(rect.max.x - 12.0, y), Align2::RIGHT_CENTER, text, font, col);
    }

    fn notice_overlay(&mut self, ctx: &egui::Context) {
        let Some(n) = &self.notice else { return };
        let t = self.t;
        let text = n.text.clone();
        let error = n.error;
        let screen = ctx.content_rect();
        egui::Area::new(egui::Id::unique("notice"))
            .order(egui::Order::Foreground)
            .anchor(Align2::RIGHT_BOTTOM, vec2(-18.0, -(STATUS_BAR_H + 14.0)))
            .show(ctx, |ui| {
                Frame::new()
                    .fill(t.menu)
                    .stroke(Stroke::new(1.0, if error { t.red.linear_multiply(0.7) } else { t.border_strong }))
                    .corner_radius(8)
                    .shadow(egui::Shadow { offset: [0, 4], blur: 16, spread: 0, color: t.shadow_popup })
                    .inner_margin(Margin::symmetric(14, 10))
                    .show(ui, |ui| {
                        ui.set_max_width((screen.width() * 0.4).clamp(260.0, 460.0));
                        ui.horizontal(|ui| {
                            let (r, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
                            icons::paint(ui.painter(), r, if error { Icon::Warning } else { Icon::Info }, if error { t.red } else { t.icon }, if error { t.red } else { t.accent });
                            ui.add(egui::Label::new(egui::RichText::new(text).font(theme::regular(12.5)).color(t.text)).wrap());
                            if icon_button(ui, &t, Icon::Close, 20.0, "Dismiss").clicked() {
                                self.notice = None;
                            }
                        });
                    });
            });
    }

    fn dialogs(&mut self, ctx: &egui::Context) {
        let t = self.t;
        if let Some(id) = self.confirm_delete.clone() {
            let name = self.contact(&id).map(|c| c.display_name()).unwrap_or_default();
            let mut close = false;
            let mut go = false;
            let modal = egui::Modal::new(egui::Id::unique("delete_contact")).frame(dialog_frame(&t)).show(ctx, |ui| {
                ui.set_width(420.0);
                ui.label(egui::RichText::new(format!("Delete {name}?")).font(theme::semibold(17.0)).color(t.text));
                ui.add(
                    egui::Label::new(egui::RichText::new("The contact, its picture and its theme are deleted for every Craft app that uses ContactSolutions's contacts.").font(theme::regular(13.0)).color(t.text_dim))
                        .wrap(),
                );
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if widgets::primary_button(ui, &t, "Delete", true).clicked() {
                        go = true;
                    }
                    if widgets::secondary_button(ui, &t, "Cancel").clicked() {
                        close = true;
                    }
                });
            });
            if go {
                self.confirm_delete = None;
                self.delete_contact(&id);
            } else if close || modal.should_close() {
                self.confirm_delete = None;
            }
        }
        if let Some(id) = self.confirm_group_delete.clone() {
            let name = self.group(&id).map(|g| g.name.clone()).unwrap_or_default();
            let mut go = false;
            let mut close = false;
            let modal = egui::Modal::new(egui::Id::unique("delete_group")).frame(dialog_frame(&t)).show(ctx, |ui| {
                ui.set_width(390.0);
                ui.label(egui::RichText::new(format!("Delete group {name}?")).font(theme::semibold(17.0)).color(t.text));
                ui.label("Contacts and numbers in the group stay saved.");
                ui.horizontal(|ui| {
                    if widgets::primary_button(ui, &t, "Delete group", true).clicked() { go = true; }
                    if widgets::secondary_button(ui, &t, "Cancel").clicked() { close = true; }
                });
            });
            if go {
                self.confirm_group_delete = None;
                self.delete_group(&id);
            } else if close || modal.should_close() { self.confirm_group_delete = None; }
        }
        if let Some(next) = self.confirm_discard.clone() {
            let mut choice = 0;
            let modal = egui::Modal::new(egui::Id::unique("discard")).frame(dialog_frame(&t)).show(ctx, |ui| {
                ui.set_width(420.0);
                ui.label(egui::RichText::new("Save your changes?").font(theme::semibold(17.0)).color(t.text));
                ui.add(egui::Label::new(egui::RichText::new("This contact has changes that aren't saved yet.").font(theme::regular(13.0)).color(t.text_dim)).wrap());
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if widgets::primary_button(ui, &t, "Save", true).clicked() {
                        choice = 1;
                    }
                    if widgets::secondary_button(ui, &t, "Don't save").clicked() {
                        choice = 2;
                    }
                    if widgets::secondary_button(ui, &t, "Keep editing").clicked() {
                        choice = 3;
                    }
                });
            });
            if modal.should_close() && choice == 0 {
                choice = 3;
            }
            match choice {
                1 => {
                    self.confirm_discard = None;
                    self.save_pane();
                    if !self.editing() {
                        self.pane = next;
                        self.finish_pending_group_navigation();
                    }
                }
                2 => {
                    self.confirm_discard = None;
                    self.pane = next;
                    self.finish_pending_group_navigation();
                }
                3 => {
                    self.confirm_discard = None;
                    self.pending_group = None;
                    self.pending_new_group = false;
                }
                _ => {}
            }
        }
        self.import_dialog(ctx);
        self.cleanup_dialog(ctx);
        self.merge_dialog(ctx);
        self.update_dialog(ctx);
        self.new_label_dialog(ctx);
    }
}

pub fn dialog_frame(t: &theme::Tokens) -> Frame {
    Frame::new()
        .fill(t.menu)
        .stroke(Stroke::new(1.0, t.border_strong))
        .corner_radius(8)
        .shadow(egui::Shadow { offset: [0, 8], blur: 28, spread: 0, color: t.shadow_window })
        .inner_margin(Margin::same(20))
}
