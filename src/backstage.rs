//! The File view (MailCraft's "backstage"): information, importing and exporting, how the other
//! Craft apps share the contacts, options and about.

use egui::{Align2, Rect, ScrollArea, Sense, Stroke, Ui, Vec2, pos2, vec2};

use crate::app::{App, Appearance, Backstage};
use crate::icons::{self, Icon};
use crate::theme;
use crate::widgets;

impl App {
    pub fn backstage_view(&mut self, ui: &mut Ui, page: Backstage) {
        let t = self.t;
        let rect = ui.max_rect();
        let col = Rect::from_min_size(rect.min, vec2(220.0, rect.height()));
        ui.painter().rect_filled(col, 0.0, t.backstage_column);
        ui.painter().line_segment([pos2(col.max.x, col.min.y), pos2(col.max.x, col.max.y)], Stroke::new(1.0, t.border));
        let mut y = col.min.y + 12.0;
        let back = Rect::from_min_size(pos2(col.min.x + 8.0, y), vec2(col.width() - 16.0, 36.0));
        let resp = ui.interact(back, egui::Id::unique("bs_back"), Sense::click());
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, "Back"));
        if resp.hovered() {
            ui.painter().rect_filled(back, 4.0, t.hover);
        }
        icons::paint(ui.painter(), Rect::from_min_size(pos2(back.min.x + 8.0, back.center().y - 9.0), Vec2::splat(18.0)), Icon::ChevronLeft, t.icon, t.icon);
        ui.painter().text(pos2(back.min.x + 34.0, back.center().y), Align2::LEFT_CENTER, "Back", theme::regular(14.0), t.text);
        if resp.clicked() {
            self.backstage = None;
            return;
        }
        y += 50.0;
        let pages = [
            (Backstage::Info, Icon::Info, "Info"),
            (Backstage::Import, Icon::Import, "Import"),
            (Backstage::Export, Icon::Export, "Export"),
            (Backstage::Sharing, Icon::Share, "Craft apps"),
            (Backstage::Options, Icon::Gear, "Options"),
            (Backstage::About, Icon::Card, "About"),
        ];
        for (p, icon, label) in pages {
            let r = Rect::from_min_size(pos2(col.min.x + 8.0, y), vec2(col.width() - 16.0, 38.0));
            let resp = ui.interact(r, egui::Id::unique(("bs", label)), Sense::click());
            resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, p == page, label));
            if p == page {
                ui.painter().rect_filled(r, 4.0, t.checked);
                ui.painter().rect_filled(Rect::from_min_size(pos2(r.min.x, r.min.y + 8.0), vec2(3.0, r.height() - 16.0)), 1.5, t.accent);
            } else if resp.hovered() {
                ui.painter().rect_filled(r, 4.0, t.hover);
            }
            icons::paint(ui.painter(), Rect::from_min_size(pos2(r.min.x + 10.0, r.center().y - 9.0), Vec2::splat(18.0)), icon, t.icon, t.accent);
            ui.painter().text(pos2(r.min.x + 38.0, r.center().y), Align2::LEFT_CENTER, label, if p == page { theme::semibold(14.0) } else { theme::regular(14.0) }, t.text);
            if resp.clicked() {
                self.backstage = Some(p);
            }
            y += 40.0;
        }

        let content = Rect::from_min_max(pos2(col.max.x + 40.0, rect.min.y + 24.0), pos2(rect.max.x - 30.0, rect.max.y - 10.0));
        let mut u = ui.new_child(egui::UiBuilder::new().max_rect(content).layout(egui::Layout::top_down(egui::Align::Min)));
        ScrollArea::vertical().auto_shrink([false, false]).id_salt(("bs_page", label_of(page))).show(&mut u, |ui| {
            ui.set_max_width(760.0);
            ui.spacing_mut().item_spacing.y = 8.0;
            match page {
                Backstage::Info => self.page_info(ui),
                Backstage::Import => self.page_import(ui),
                Backstage::Export => self.page_export(ui),
                Backstage::Sharing => self.page_sharing(ui),
                Backstage::Options => self.page_options(ui),
                Backstage::About => self.page_about(ui),
            }
        });
    }

    fn page_info(&mut self, ui: &mut Ui) {
        let t = self.t;
        title(ui, &t, "Info");
        let n = self.contacts.len();
        let photos = self.contacts.iter().filter(|c| !c.photo.is_empty()).count();
        let themed = self.contacts.iter().filter(|c| !c.theme.is_default()).count();
        let labels = self.label_counts().len();
        para(ui, &t, &format!("{n} contacts  ·  {photos} with a picture  ·  {themed} with a theme  ·  {labels} labels"));
        ui.add_space(6.0);
        if let Some(s) = &self.store {
            let dir = s.dir().to_path_buf();
            sub(ui, &t, "Where your contacts are kept");
            para(ui, &t, &format!("{}\n\ncontacts.db holds the contacts; photos, backgrounds and fonts hold their pictures and theme files. Back up this folder to keep everything.", dir.display()));
            ui.horizontal(|ui| {
                if widgets::secondary_button(ui, &t, "Open the folder").clicked() {
                    crate::app::open_path(&dir);
                }
                if widgets::secondary_button(ui, &t, "Read again").on_hover_text("Reload every contact from the store").clicked() {
                    self.forget_textures();
                    self.reload();
                    self.notify("Contacts read again.", false);
                }
            });
        } else if let Some(e) = &self.store_error {
            para(ui, &t, &format!("The contact store couldn't be opened: {e}"));
        }
        if self.demo {
            ui.add_space(6.0);
            para(ui, &t, "This is the demo: made-up contacts in a temporary folder, removed when you close ContactCraft.");
        }
    }

    fn page_import(&mut self, ui: &mut Ui) {
        let t = self.t;
        title(ui, &t, "Import contacts");
        para(ui, &t, "Bring contacts in from Google Contacts, Outlook, an iPhone or any program that saves vCard or CSV files. Every field they carry is kept; columns ContactCraft has no place for become your own fields.");
        if widgets::primary_button(ui, &t, "Choose a file to import…", self.store.is_some()).clicked() {
            self.pick_import_file();
        }
        para(ui, &t, "You can also drop the file onto the ContactCraft window.");
        ui.add_space(8.0);
        sub(ui, &t, "From Google Contacts");
        steps(ui, &t, &[
            "Open contacts.google.com in your browser and sign in.",
            "Select the contacts to bring (or none, for all of them), then choose Export from the menu (the ⋮ button, or Export on the left).",
            "Pick vCard (keeps pictures) or Google CSV, then Export. The file goes to your Downloads folder.",
            "Back here, click \"Choose a file to import…\" and pick that file.",
        ]);
        sub(ui, &t, "From Outlook");
        steps(ui, &t, &[
            "Classic Outlook: File › Open & Export › Import/Export › Export to a file › Comma Separated Values, then choose Contacts.",
            "New Outlook or Outlook.com: People › Manage contacts › Export contacts, then Export.",
            "Import the .csv file here. Outlook's own fields (account, billing information, user fields and the rest) are kept.",
        ]);
        sub(ui, &t, "From an iPhone or Mac");
        steps(ui, &t, &["In Contacts, select the contacts, then File › Export › Export vCard (or share them as a vCard file).", "Import the .vcf file here; pictures come along."]);
    }

    fn page_export(&mut self, ui: &mut Ui) {
        let t = self.t;
        title(ui, &t, "Export contacts");
        para(ui, &t, &format!("Save your contacts to a file, to import into Google Contacts, Outlook or another computer. The list shows {} of {} contacts right now.", self.shown().len(), self.contacts.len()));
        let rows = [
            ("vcf", "vCard (.vcf)", "For Google Contacts, Outlook, iPhone and almost everything else. Keeps pictures, and every ContactCraft field and theme for another ContactCraft."),
            ("csv", "Google CSV (.csv)", "The spreadsheet file Google Contacts imports. No pictures or themes."),
            ("json", "ContactCraft file (.json)", "Everything, exactly as ContactCraft keeps it (pictures stay in the data folder)."),
        ];
        for (kind, name, what) in rows {
            ui.add_space(6.0);
            sub(ui, &t, name);
            para(ui, &t, what);
            ui.horizontal(|ui| {
                if widgets::primary_button(ui, &t, "Export all", !self.contacts.is_empty()).clicked() {
                    self.export(kind, false);
                }
                if widgets::secondary_button(ui, &t, "Export the contacts shown").clicked() {
                    self.export(kind, true);
                }
            });
        }
    }

    fn page_sharing(&mut self, ui: &mut Ui) {
        let t = self.t;
        title(ui, &t, "Sharing with Craft apps");
        para(ui, &t, "ContactCraft keeps your contacts where every Craft app on this computer can read and change them, whether ContactCraft is open or not. MessageCraft can use their names, pictures and themes for your conversations, and MailCraft can do the same for email.");
        sub(ui, &t, "How it works");
        steps(ui, &t, &[
            "The contacts are one database file in your own Windows profile, with the pictures beside it. Only programs you run can open it.",
            "Several programs can use it at once: a change one app makes shows in the others within a couple of seconds.",
            "Each contact carries its theme (colours, font, background, message colours), so it looks the same everywhere.",
            "Programs other than ContactCraft use a small shared library (contactcraft-store), described in the guide that comes with ContactCraft.",
        ]);
        if let Some(s) = &self.store {
            let dir = s.dir().to_path_buf();
            sub(ui, &t, "Where");
            para(ui, &t, &dir.display().to_string());
            ui.horizontal(|ui| {
                if widgets::secondary_button(ui, &t, "Open the folder").clicked() {
                    crate::app::open_path(&dir);
                }
                if widgets::secondary_button(ui, &t, "Copy the location").clicked() {
                    ui.ctx().copy_text(dir.display().to_string());
                    self.notify("Copied.", false);
                }
            });
        }
        sub(ui, &t, "Craft apps on this computer");
        for (name, exe) in [("MessageCraft", "MessageCraft.exe"), ("MailCraft", "mailcraft.exe")] {
            let found = ["ProgramFiles", "LOCALAPPDATA"].iter().filter_map(|v| std::env::var_os(v)).map(std::path::PathBuf::from).any(|b| b.join(name).join(exe).is_file() || b.join("Programs").join(name).join(exe).is_file());
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
                icons::paint(ui.painter(), r, if found { Icon::Check } else { Icon::Info }, t.icon, if found { t.green } else { t.text_dim });
                ui.label(egui::RichText::new(format!("{name}: {}", if found { "installed" } else { "not found" })).font(theme::regular(13.0)).color(t.text));
            });
        }
    }

    fn page_options(&mut self, ui: &mut Ui) {
        let t = self.t;
        title(ui, &t, "Options");
        sub(ui, &t, "Appearance");
        ui.horizontal(|ui| {
            for (a, label) in [(Appearance::System, "Follow Windows"), (Appearance::Light, "Light"), (Appearance::Dark, "Dark")] {
                if ui.radio(self.settings.appearance == a, label).clicked() {
                    self.settings.appearance = a;
                    self.apply_settings(ui.ctx());
                    self.save_settings();
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Zoom").font(theme::regular(13.0)).color(t.text));
            let mut z = self.settings.zoom as f32;
            if ui.add(egui::Slider::new(&mut z, 80.0..=200.0).step_by(5.0).suffix("%")).changed() {
                self.settings.zoom = z as u16;
                self.apply_settings(ui.ctx());
                self.save_settings();
            }
        });
        sub(ui, &t, "List");
        if ui.checkbox(&mut self.settings.label_pane, "Show the label pane").changed() {
            self.save_settings();
        }
        if ui.checkbox(&mut self.settings.tighter, "Tighter spacing (one line per contact)").changed() {
            self.save_settings();
        }
        if ui.checkbox(&mut self.settings.sort_by_family, "Sort by last name").changed() {
            self.save_settings();
            self.reload();
        }
        sub(ui, &t, "Updates");
        if ui.checkbox(&mut self.settings.check_updates, "Check for a new version when ContactCraft starts").changed() {
            self.save_settings();
        }
        para(ui, &t, "ContactCraft asks GitHub for its latest release; nothing about you or your contacts is sent.");
        if widgets::secondary_button(ui, &t, "Check now").clicked() {
            let ctx = ui.ctx().clone();
            self.check_for_updates(&ctx, true);
        }
    }

    fn page_about(&mut self, ui: &mut Ui) {
        let t = self.t;
        title(ui, &t, "About ContactCraft");
        para(ui, &t, &format!("Version {}", env!("CARGO_PKG_VERSION")));
        ui.horizontal(|ui| {
            if widgets::secondary_button(ui, &t, "Check for updates").clicked() {
                let ctx = ui.ctx().clone();
                self.check_for_updates(&ctx, true);
            }
            if widgets::secondary_button(ui, &t, "Releases on GitHub").clicked() {
                crate::app::open_url(crate::update::RELEASES_PAGE);
            }
        });
        para(ui, &t, "Your contacts on Windows, with every detail Google Contacts, Outlook and vCard can hold, a picture and a theme for each person, shared with every Craft app.");
        para(ui, &t, "Its look follows MailCraft's design, as MessageCraft does. Inter typeface: SIL Open Font License 1.1.");
        para(ui, &t, "Keyboard: Ctrl+N new contact, Ctrl+F search, Ctrl+S save, Enter edit, Delete delete, ↑ ↓ move, Esc close.");
    }
}

fn label_of(p: Backstage) -> &'static str {
    match p {
        Backstage::Info => "info",
        Backstage::Import => "import",
        Backstage::Export => "export",
        Backstage::Sharing => "sharing",
        Backstage::Options => "options",
        Backstage::About => "about",
    }
}

fn title(ui: &mut Ui, t: &theme::Tokens, s: &str) {
    ui.label(egui::RichText::new(s).font(theme::semibold(26.0)).color(t.text));
    ui.add_space(4.0);
}

fn sub(ui: &mut Ui, t: &theme::Tokens, s: &str) {
    ui.add_space(6.0);
    ui.label(egui::RichText::new(s).font(theme::semibold(15.0)).color(t.text));
}

fn para(ui: &mut Ui, t: &theme::Tokens, s: &str) {
    ui.add(egui::Label::new(egui::RichText::new(s).font(theme::regular(13.0)).color(t.text_dim)).wrap());
}

fn steps(ui: &mut Ui, t: &theme::Tokens, items: &[&str]) {
    for (i, s) in items.iter().enumerate() {
        ui.horizontal_top(|ui| {
            let (r, _) = ui.allocate_exact_size(Vec2::splat(22.0), Sense::hover());
            ui.painter().circle_filled(r.center(), 10.0, t.accent_soft);
            ui.painter().text(r.center(), Align2::CENTER_CENTER, (i + 1).to_string(), theme::semibold(11.5), t.accent_soft_text);
            ui.add(egui::Label::new(egui::RichText::new(*s).font(theme::regular(13.0)).color(t.text)).wrap());
        });
    }
}
