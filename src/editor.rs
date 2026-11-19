//! The contact editor: every field ContactSolutions keeps, in sections that fold away, with lists
//! that grow (as many phones, addresses, dates... as needed, each with any label) and custom
//! fields for anything else.

use std::collections::HashSet;

use contactsolutions_store::{Address, Contact, CustomField, Email, Im, Labeled, Organization, labels};
use egui::{Align2, Margin, Rect, ScrollArea, Sense, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};

use crate::app::{App, Pane};
use crate::icons::{self, Icon};
use crate::theme::{self, Tokens};
use crate::widgets;

const LABEL_W: f32 = 140.0;
const FIELD_W: f32 = 340.0;

impl App {
    pub fn contact_editor(&mut self, ui: &mut Ui) {
        let t = self.t;
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 8.0, t.surface);
        ui.painter().rect_stroke(rect, 8.0, Stroke::new(1.0, t.border), StrokeKind::Inside);
        let Pane::Edit(c) = &self.pane else { return };
        let mut c: Contact = (**c).clone();
        let is_new = c.id.is_empty() || self.contact(&c.id).is_none();
        let inner = rect.shrink2(vec2(24.0, 18.0));
        let mut u = ui.new_child(egui::UiBuilder::new().max_rect(inner).layout(egui::Layout::top_down(egui::Align::Min)));
        let ui = &mut u;

        // Heading with the avatar and the actions.
        let mut save = false;
        let mut cancel = false;
        let mut to_theme = false;
        let ctx = ui.ctx().clone();
        let photo = self.photo_texture(&ctx, &c);
        ui.horizontal(|ui| {
            let (ar, _) = ui.allocate_exact_size(Vec2::splat(52.0), Sense::hover());
            widgets::contact_avatar(ui.painter(), &c, ar.center(), 25.0, photo);
            ui.add_space(6.0);
            ui.vertical(|ui| {
                ui.add_space(2.0);
                let title = if is_new { "New contact".to_string() } else { format!("Edit {}", c.display_name()) };
                ui.label(egui::RichText::new(title).font(theme::semibold(20.0)).color(t.text));
                ui.label(egui::RichText::new("Saved where MessageCraft and MailCraft can see it. Fill in as much or as little as you like.").font(theme::regular(12.0)).color(t.text_dim));
            });
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            if widgets::primary_button(ui, &t, "Save", !c.is_blank()).on_hover_text("Ctrl+S").clicked() {
                save = true;
            }
            if widgets::secondary_button(ui, &t, "Cancel").clicked() {
                cancel = true;
            }
            if widgets::secondary_button(ui, &t, "Theme…").on_hover_text("Colours, font, background").clicked() {
                to_theme = true;
            }
            if c.is_blank() {
                ui.label(egui::RichText::new("A contact needs a name, a company, an email address or a phone number.").font(theme::regular(12.0)).color(t.text_dim));
            }
        });
        ui.add_space(8.0);

        let all_labels: Vec<String> = self.label_counts().into_iter().map(|(l, _)| l).collect();
        let mut closed = std::mem::take(&mut self.closed_sections);
        let mut photo_action = 0u8;
        ScrollArea::vertical().auto_shrink([false, false]).id_salt(("editor", c.id.clone())).show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 6.0;
            ui.set_max_width(760.0);

            if open(ui, &t, &mut closed, "name", Icon::Person, "Name") {
                ui.horizontal(|ui| {
                    lbl(ui, &t, "Kind");
                    combo(ui, "kind", &mut c.kind, labels::KIND, 160.0);
                });
                text(ui, &t, "Prefix", &mut c.name.prefix, "Dr., Ms., Rev.");
                text(ui, &t, "First name", &mut c.name.given, "");
                text(ui, &t, "Middle name", &mut c.name.middle, "");
                text(ui, &t, "Last name", &mut c.name.family, "");
                text(ui, &t, "Suffix", &mut c.name.suffix, "Jr., III, PhD");
                text(ui, &t, "Nickname", &mut c.name.nickname, "");
                text(ui, &t, "Maiden name", &mut c.name.maiden, "");
                text(ui, &t, "Phonetic first", &mut c.name.phonetic_given, "How it sounds (Yomi)");
                text(ui, &t, "Phonetic middle", &mut c.name.phonetic_middle, "");
                text(ui, &t, "Phonetic last", &mut c.name.phonetic_family, "");
                text(ui, &t, "Initials", &mut c.name.initials, "");
                text(ui, &t, "File as", &mut c.name.file_as, "Smith, Anna");
                text(ui, &t, "Display name", &mut c.name.display, "Leave empty to use the name above");
                ui.horizontal(|ui| {
                    ui.add_space(LABEL_W + 6.0);
                    ui.checkbox(&mut c.favorite, egui::RichText::new("Favorite").font(theme::regular(13.0)));
                });
            }

            if open(ui, &t, &mut closed, "picture", Icon::Image, "Picture") {
                ui.horizontal(|ui| {
                    lbl(ui, &t, "Picture");
                    ui.spacing_mut().item_spacing.x = 8.0;
                    if widgets::secondary_button(ui, &t, if c.photo.is_empty() { "Choose a picture…" } else { "Change picture…" }).clicked() {
                        photo_action = 1;
                    }
                    if !c.photo.is_empty() && widgets::secondary_button(ui, &t, "Remove").clicked() {
                        photo_action = 2;
                    }
                });
                hint(ui, &t, "PNG, JPEG, GIF or WebP. You can also drop a picture on the window.");
                ui.horizontal(|ui| {
                    lbl(ui, &t, "Shape");
                    combo(ui, "avatar_shape", &mut c.theme.avatar_shape, labels::AVATAR_SHAPE, 160.0);
                });
                text(ui, &t, "Emoji", &mut c.theme.emoji, "One emoji to go with them");
            }

            if open(ui, &t, &mut closed, "contact", Icon::Phone, "Phone, email, chat") {
                labeled_list(ui, &t, "Phone", "phone", &mut c.phones, labels::PHONE, "Number");
                email_list(ui, &t, &mut c.emails);
                im_list(ui, &t, &mut c.im);
                labeled_list(ui, &t, "SIP / calling", "sip", &mut c.sip, labels::SIP, "sip address");
            }

            if open(ui, &t, &mut closed, "addresses", Icon::Location, "Addresses") {
                address_list(ui, &t, &mut c.addresses);
            }

            if open(ui, &t, &mut closed, "work", Icon::Building, "Work") {
                org_list(ui, &t, &mut c.organizations);
                text(ui, &t, "Profession", &mut c.occupation, "");
            }

            if open(ui, &t, &mut closed, "web", Icon::Globe, "Web and social") {
                labeled_list(ui, &t, "Website", "web", &mut c.websites, labels::WEBSITE, "https://");
                labeled_list(ui, &t, "Social profile", "social", &mut c.social_profiles, labels::SOCIAL, "Profile link or @name");
            }

            if open(ui, &t, &mut closed, "dates", Icon::Cake, "Dates") {
                text(ui, &t, "Birthday", &mut c.birthday, "YYYY-MM-DD, or --MM-DD without a year");
                text(ui, &t, "Birthplace", &mut c.birthplace, "");
                text(ui, &t, "Anniversary", &mut c.anniversary, "YYYY-MM-DD");
                labeled_list(ui, &t, "Other date", "event", &mut c.events, labels::EVENT, "YYYY-MM-DD");
                text(ui, &t, "Date of death", &mut c.deathdate, "");
                text(ui, &t, "Place of death", &mut c.deathplace, "");
            }

            if open(ui, &t, &mut closed, "relations", Icon::People, "Relationships") {
                labeled_list(ui, &t, "Related person", "relation", &mut c.relations, labels::RELATION, "Their name");
            }

            if open(ui, &t, &mut closed, "personal", Icon::Card, "Personal") {
                ui.horizontal(|ui| {
                    lbl(ui, &t, "Gender");
                    combo(ui, "gender", &mut c.gender, labels::GENDER, 200.0);
                });
                text(ui, &t, "Pronouns", &mut c.pronouns, "she/her, he/him, they/them…");
                list_text(ui, &t, "Languages", &mut c.languages, "English, Español");
                text(ui, &t, "Time zone", &mut c.time_zone, "America/Chicago");
                text(ui, &t, "Map location", &mut c.geo, "latitude, longitude");
                list_text(ui, &t, "Hobbies", &mut c.hobbies, "Separate with commas");
                list_text(ui, &t, "Interests", &mut c.interests, "");
                list_text(ui, &t, "Skills", &mut c.skills, "");
                list_text(ui, &t, "Expertise", &mut c.expertise, "");
            }

            if open(ui, &t, &mut closed, "ids", Icon::Tag, "IDs, places, calendars and keys") {
                labeled_list(ui, &t, "ID number", "extid", &mut c.external_ids, labels::EXTERNAL_ID, "");
                labeled_list(ui, &t, "Desk or building", "loc", &mut c.locations, labels::LOCATION, "");
                labeled_list(ui, &t, "Calendar link", "cal", &mut c.calendar_urls, labels::CALENDAR, "https://");
                labeled_list(ui, &t, "Public key", "key", &mut c.public_keys, labels::KEY, "Key or where to get it");
            }

            if open(ui, &t, &mut closed, "outlook", Icon::Mail, "Outlook details") {
                let o = &mut c.outlook;
                text(ui, &t, "Account", &mut o.account, "");
                text(ui, &t, "Customer ID", &mut o.customer_id, "");
                text(ui, &t, "Government ID", &mut o.government_id, "");
                text(ui, &t, "Organizational ID", &mut o.organizational_id, "");
                text(ui, &t, "Billing information", &mut o.billing_information, "");
                text(ui, &t, "Mileage", &mut o.mileage, "");
                text(ui, &t, "Directory server", &mut o.directory_server, "");
                text(ui, &t, "Computer network", &mut o.computer_network_name, "");
                text(ui, &t, "FTP site", &mut o.ftp_site, "");
                text(ui, &t, "Free/busy address", &mut o.internet_free_busy, "");
                text(ui, &t, "Subject", &mut o.subject, "");
                text(ui, &t, "Keywords", &mut o.keywords, "");
                text(ui, &t, "User 1", &mut o.user1, "");
                text(ui, &t, "User 2", &mut o.user2, "");
                text(ui, &t, "User 3", &mut o.user3, "");
                text(ui, &t, "User 4", &mut o.user4, "");
                ui.horizontal(|ui| {
                    lbl(ui, &t, "Priority");
                    combo(ui, "priority", &mut o.priority, labels::PRIORITY, 140.0);
                    ui.add_space(12.0);
                    ui.label(egui::RichText::new("Sensitivity").font(theme::regular(13.0)).color(t.text_dim));
                    combo(ui, "sensitivity", &mut o.sensitivity, labels::SENSITIVITY, 140.0);
                });
                ui.horizontal(|ui| {
                    ui.add_space(LABEL_W + 6.0);
                    ui.checkbox(&mut o.private, egui::RichText::new("Private").font(theme::regular(13.0)));
                    ui.checkbox(&mut o.journal, egui::RichText::new("Keep a journal").font(theme::regular(13.0)));
                });
                text(ui, &t, "Follow-up flag", &mut o.follow_up_flag, "Call, Follow up, Send email…");
                ui.horizontal(|ui| {
                    lbl(ui, &t, "Follow-up due");
                    ui.add(egui::TextEdit::singleline(&mut o.follow_up_due).hint_text("YYYY-MM-DD").desired_width(160.0).font(theme::regular(13.5)).margin(Margin::symmetric(8, 5)));
                    ui.checkbox(&mut o.follow_up_done, egui::RichText::new("Done").font(theme::regular(13.0)));
                });
            }

            if open(ui, &t, &mut closed, "custom", Icon::Plus, "Your own fields") {
                hint(ui, &t, "Anything else you want to keep: give it a name and a kind.");
                custom_list(ui, &t, &mut c.custom_fields);
            }

            if open(ui, &t, &mut closed, "labels", Icon::Tag, "Labels and notes") {
                label_chips(ui, &t, &mut c.labels, &all_labels);
                ui.horizontal_top(|ui| {
                    lbl(ui, &t, "Notes");
                    ui.add(egui::TextEdit::multiline(&mut c.notes).desired_width(FIELD_W + 120.0).desired_rows(5).font(theme::regular(13.0)).margin(Margin::symmetric(8, 6)));
                });
            }

            if !c.extra.is_empty() && open(ui, &t, &mut closed, "extra", Icon::Info, "Kept from other apps") {
                hint(ui, &t, "Fields another app added. ContactSolutions keeps them as they are.");
                for (k, v) in &c.extra {
                    ui.horizontal(|ui| {
                        lbl(ui, &t, k);
                        ui.label(egui::RichText::new(v.to_string()).font(theme::regular(12.5)).color(t.text));
                    });
                }
            }
            ui.add_space(30.0);
        });
        self.closed_sections = closed;

        // Pictures change files right away; the contact itself is saved with Save.
        match photo_action {
            1 => {
                if let Some(path) = rfd::FileDialog::new().set_title("Choose a picture").add_filter("Pictures", &["png", "jpg", "jpeg", "gif", "webp"]).pick_file() {
                    match std::fs::read(&path) {
                        Ok(bytes) => {
                            let bytes = shrink_picture(&bytes).unwrap_or(bytes);
                            if let Some(store) = &self.store {
                                if let Err(e) = store.set_photo(&mut c, &bytes) {
                                    self.notify(e, true);
                                }
                            }
                            self.forget_textures();
                        }
                        Err(e) => self.notify(e.to_string(), true),
                    }
                }
            }
            2 => {
                c.photo.clear();
                self.forget_textures();
            }
            _ => {}
        }

        self.pane = Pane::Edit(Box::new(c.clone()));
        if to_theme {
            self.pane = Pane::Theme(Box::new(c));
        } else if save {
            self.save_pane();
        } else if cancel {
            self.pane = Pane::Card;
        }
    }
}

/// Big photos are scaled to 512 px and saved as PNG, so every app can load them quickly.
pub fn shrink_picture(bytes: &[u8]) -> Option<Vec<u8>> {
    let img = image::load_from_memory(bytes).ok()?;
    if img.width() <= 512 && img.height() <= 512 {
        return None;
    }
    let img = img.thumbnail(512, 512);
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).ok()?;
    Some(out.into_inner())
}

// --- pieces ------------------------------------------------------------------------------------

/// A section heading that folds its section away when clicked. Returns whether it's open.
fn open(ui: &mut Ui, t: &Tokens, closed: &mut HashSet<&'static str>, key: &'static str, icon: Icon, title: &str) -> bool {
    ui.add_space(6.0);
    let w = ui.available_width();
    let (r, resp) = ui.allocate_exact_size(vec2(w, 34.0), Sense::click());
    let is_open = !closed.contains(key);
    resp.widget_info(|| egui::WidgetInfo::selected(egui::Role::Button, true, is_open, title));
    let p = ui.painter();
    if resp.hovered() {
        p.rect_filled(r, 4.0, t.hover_subtle);
    }
    icons::paint(p, Rect::from_min_size(pos2(r.min.x + 4.0, r.center().y - 7.0), Vec2::splat(14.0)), if is_open { Icon::ChevronDown } else { Icon::ChevronRight }, t.text_dim, t.text_dim);
    icons::paint(p, Rect::from_min_size(pos2(r.min.x + 24.0, r.center().y - 9.0), Vec2::splat(18.0)), icon, t.icon, t.accent);
    p.text(pos2(r.min.x + 50.0, r.center().y), Align2::LEFT_CENTER, title, theme::semibold(14.0), t.text);
    p.line_segment([pos2(r.min.x, r.max.y - 0.5), pos2(r.max.x, r.max.y - 0.5)], Stroke::new(1.0, t.rule));
    if resp.clicked() {
        if is_open {
            closed.insert(key);
        } else {
            closed.remove(key);
        }
    }
    is_open
}

fn lbl(ui: &mut Ui, t: &Tokens, text: &str) {
    widgets::field_label(ui, LABEL_W, 28.0, egui::RichText::new(text).font(theme::regular(13.0)).color(t.text_dim));
}

fn hint(ui: &mut Ui, t: &Tokens, text: &str) {
    ui.horizontal(|ui| {
        ui.add_space(LABEL_W + 6.0);
        ui.add(egui::Label::new(egui::RichText::new(text).font(theme::regular(11.5)).color(t.text_dim)).wrap());
    });
}

fn text(ui: &mut Ui, t: &Tokens, label: &str, value: &mut String, hint_text: &str) {
    ui.horizontal(|ui| {
        lbl(ui, t, label);
        ui.add(egui::TextEdit::singleline(value).hint_text(hint_text).desired_width(FIELD_W).font(theme::regular(13.5)).margin(Margin::symmetric(8, 5)));
    });
}

/// A list of words edited as "a, b, c".
fn list_text(ui: &mut Ui, t: &Tokens, label: &str, value: &mut Vec<String>, hint_text: &str) {
    let id = ui.make_persistent_id(("list_text", label));
    let mut s: String = ui.data(|d| d.get_temp::<String>(id)).unwrap_or_else(|| value.join(", "));
    // Pick up outside changes when the field isn't being typed in.
    let joined = value.join(", ");
    let split = |s: &str| s.split([',', ';']).map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect::<Vec<_>>();
    if split(&s) != *value {
        s = joined;
    }
    ui.horizontal(|ui| {
        lbl(ui, t, label);
        let r = ui.add(egui::TextEdit::singleline(&mut s).hint_text(hint_text).desired_width(FIELD_W).font(theme::regular(13.5)).margin(Margin::symmetric(8, 5)));
        if r.changed() {
            *value = split(&s);
        }
    });
    ui.data_mut(|d| d.insert_temp(id, s));
}

fn combo(ui: &mut Ui, id: impl std::hash::Hash + std::fmt::Debug, value: &mut String, options: &[&str], width: f32) {
    let shown = if value.is_empty() { "—".to_string() } else { cap(value) };
    egui::ComboBox::from_id_salt(id).width(width).selected_text(shown).show_ui(ui, |ui| {
        for o in options {
            ui.selectable_value(value, o.to_string(), if o.is_empty() { "—".to_string() } else { cap(o) });
        }
    });
}

/// A label chooser: the standard labels, or any word typed in.
fn label_pick(ui: &mut Ui, id: impl std::hash::Hash + std::fmt::Debug, value: &mut String, options: &[&str]) {
    let shown = if value.is_empty() { "Label".to_string() } else { cap(value) };
    egui::ComboBox::from_id_salt(id).width(112.0).selected_text(shown).height(320.0).show_ui(ui, |ui| {
        for o in options {
            ui.selectable_value(value, o.to_string(), cap(o));
        }
        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Own label:");
            ui.add(egui::TextEdit::singleline(value).desired_width(120.0));
        });
    });
}

fn remove_button(ui: &mut Ui, t: &Tokens) -> bool {
    widgets::icon_button(ui, t, Icon::Close, 24.0, "Remove").clicked()
}

fn add_link(ui: &mut Ui, t: &Tokens, text: &str) -> bool {
    let mut clicked = false;
    ui.horizontal(|ui| {
        ui.add_space(LABEL_W + 6.0);
        let g = ui.painter().layout_no_wrap(text.to_string(), theme::regular(13.0), t.accent_text);
        let (r, resp) = ui.allocate_exact_size(vec2(g.size().x + 26.0, 24.0), Sense::click());
        resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, text));
        if resp.hovered() {
            ui.painter().rect_filled(r, 4.0, t.hover);
        }
        icons::paint(ui.painter(), Rect::from_min_size(pos2(r.min.x + 4.0, r.center().y - 7.0), Vec2::splat(14.0)), Icon::Plus, t.accent_text, t.accent_text);
        ui.painter().galley(pos2(r.min.x + 22.0, r.center().y - g.size().y / 2.0), g, t.accent_text);
        clicked = resp.clicked();
    });
    clicked
}

fn labeled_list(ui: &mut Ui, t: &Tokens, title: &str, id: &str, items: &mut Vec<Labeled>, options: &[&str], hint_text: &str) {
    let mut remove = None;
    for (i, item) in items.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            lbl(ui, t, if i == 0 { title } else { "" });
            label_pick(ui, (id, i), &mut item.label, options);
            ui.add(egui::TextEdit::singleline(&mut item.value).hint_text(hint_text).desired_width(FIELD_W - 120.0).font(theme::regular(13.5)).margin(Margin::symmetric(8, 5)));
            if remove_button(ui, t) {
                remove = Some(i);
            }
        });
    }
    if let Some(i) = remove {
        items.remove(i);
    }
    if add_link(ui, t, &format!("Add {}", title.to_lowercase())) {
        items.push(Labeled::new(options.first().copied().unwrap_or("other"), ""));
    }
}

fn email_list(ui: &mut Ui, t: &Tokens, items: &mut Vec<Email>) {
    let mut remove = None;
    for (i, e) in items.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            lbl(ui, t, if i == 0 { "Email" } else { "" });
            label_pick(ui, ("email", i), &mut e.label, labels::EMAIL);
            ui.add(egui::TextEdit::singleline(&mut e.address).hint_text("name@example.com").desired_width(FIELD_W - 120.0).font(theme::regular(13.5)).margin(Margin::symmetric(8, 5)));
            if remove_button(ui, t) {
                remove = Some(i);
            }
        });
        if !e.address.is_empty() || !e.display_name.is_empty() {
            ui.horizontal(|ui| {
                lbl(ui, t, "");
                ui.add_space(120.0);
                ui.add(egui::TextEdit::singleline(&mut e.display_name).hint_text("Display as (Outlook)").desired_width(FIELD_W - 120.0).font(theme::regular(12.5)).margin(Margin::symmetric(8, 4)));
            });
        }
    }
    if let Some(i) = remove {
        items.remove(i);
    }
    if add_link(ui, t, "Add email") {
        items.push(Email { label: "home".into(), ..Default::default() });
    }
}

fn im_list(ui: &mut Ui, t: &Tokens, items: &mut Vec<Im>) {
    let mut remove = None;
    for (i, m) in items.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            lbl(ui, t, if i == 0 { "Chat" } else { "" });
            label_pick(ui, ("im", i), &mut m.protocol, labels::IM_PROTOCOL);
            ui.add(egui::TextEdit::singleline(&mut m.address).hint_text("Name or number on that service").desired_width(FIELD_W - 120.0).font(theme::regular(13.5)).margin(Margin::symmetric(8, 5)));
            if remove_button(ui, t) {
                remove = Some(i);
            }
        });
    }
    if let Some(i) = remove {
        items.remove(i);
    }
    if add_link(ui, t, "Add chat") {
        items.push(Im { label: "other".into(), protocol: "Teams".into(), address: String::new() });
    }
}

fn address_list(ui: &mut Ui, t: &Tokens, items: &mut Vec<Address>) {
    let mut remove = None;
    for (i, a) in items.iter_mut().enumerate() {
        if i > 0 {
            ui.add_space(6.0);
        }
        ui.horizontal(|ui| {
            lbl(ui, t, if i == 0 { "Address" } else { "" });
            label_pick(ui, ("addr", i), &mut a.label, labels::ADDRESS);
            ui.checkbox(&mut a.mailing, egui::RichText::new("Mailing address").font(theme::regular(12.5)));
            if remove_button(ui, t) {
                remove = Some(i);
            }
        });
        let small = |ui: &mut Ui, v: &mut String, h: &str, w: f32| {
            ui.add(egui::TextEdit::singleline(v).hint_text(h).desired_width(w).font(theme::regular(13.5)).margin(Margin::symmetric(8, 5)));
        };
        ui.horizontal(|ui| {
            lbl(ui, t, "");
            ui.add(egui::TextEdit::multiline(&mut a.street).hint_text("Street").desired_rows(1).desired_width(FIELD_W).font(theme::regular(13.5)).margin(Margin::symmetric(8, 5)));
        });
        ui.horizontal(|ui| {
            lbl(ui, t, "");
            small(ui, &mut a.extended, "Apartment, suite, floor", FIELD_W * 0.6);
            small(ui, &mut a.po_box, "PO box", FIELD_W * 0.4 - 6.0);
        });
        ui.horizontal(|ui| {
            lbl(ui, t, "");
            small(ui, &mut a.city, "City", FIELD_W * 0.45);
            small(ui, &mut a.region, "State / region", FIELD_W * 0.3);
            small(ui, &mut a.postal_code, "ZIP", FIELD_W * 0.25 - 12.0);
        });
        ui.horizontal(|ui| {
            lbl(ui, t, "");
            small(ui, &mut a.country, "Country", FIELD_W * 0.7);
            small(ui, &mut a.country_code, "Code", FIELD_W * 0.3 - 6.0);
        });
    }
    if let Some(i) = remove {
        items.remove(i);
    }
    if add_link(ui, t, "Add address") {
        items.push(Address { label: "home".into(), ..Default::default() });
    }
}

fn org_list(ui: &mut Ui, t: &Tokens, items: &mut Vec<Organization>) {
    let mut remove = None;
    for (i, o) in items.iter_mut().enumerate() {
        if i > 0 {
            ui.add_space(6.0);
        }
        ui.horizontal(|ui| {
            lbl(ui, t, if i == 0 { "Organization" } else { "" });
            label_pick(ui, ("org", i), &mut o.label, labels::ORGANIZATION);
            ui.checkbox(&mut o.current, egui::RichText::new("Current").font(theme::regular(12.5)));
            if remove_button(ui, t) {
                remove = Some(i);
            }
        });
        let row = |ui: &mut Ui, a: &mut String, ha: &str, b: &mut String, hb: &str| {
            ui.horizontal(|ui| {
                lbl(ui, t, "");
                ui.add(egui::TextEdit::singleline(a).hint_text(ha).desired_width(FIELD_W * 0.5 - 3.0).font(theme::regular(13.5)).margin(Margin::symmetric(8, 5)));
                ui.add(egui::TextEdit::singleline(b).hint_text(hb).desired_width(FIELD_W * 0.5 - 3.0).font(theme::regular(13.5)).margin(Margin::symmetric(8, 5)));
            });
        };
        row(ui, &mut o.company, "Company", &mut o.department, "Department");
        row(ui, &mut o.title, "Job title", &mut o.office, "Office");
        row(ui, &mut o.job_description, "Job description", &mut o.phonetic_company, "Phonetic company");
        row(ui, &mut o.symbol, "Stock symbol", &mut o.cost_center, "Cost center");
        row(ui, &mut o.start_date, "Started (YYYY-MM-DD)", &mut o.end_date, "Ended");
        ui.horizontal(|ui| {
            lbl(ui, t, "");
            ui.add(egui::TextEdit::singleline(&mut o.domain).hint_text("Company website domain").desired_width(FIELD_W).font(theme::regular(13.5)).margin(Margin::symmetric(8, 5)));
        });
    }
    if let Some(i) = remove {
        items.remove(i);
    }
    if add_link(ui, t, "Add organization") {
        items.push(Organization { label: "work".into(), current: true, ..Default::default() });
    }
}

fn custom_list(ui: &mut Ui, t: &Tokens, items: &mut Vec<CustomField>) {
    let mut remove = None;
    for (i, f) in items.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut f.label).hint_text("Field name").desired_width(LABEL_W).font(theme::regular(13.0)).margin(Margin::symmetric(8, 5)));
            combo(ui, ("custom_kind", i), &mut f.kind, labels::CUSTOM_KIND, 80.0);
            if f.kind == "yesno" {
                let mut yes = f.value.eq_ignore_ascii_case("yes") || f.value.eq_ignore_ascii_case("true");
                if ui.checkbox(&mut yes, "Yes").changed() {
                    f.value = if yes { "Yes".into() } else { "No".into() };
                }
            } else {
                let hint_text = match f.kind.as_str() {
                    "date" => "YYYY-MM-DD",
                    "url" => "https://",
                    "email" => "name@example.com",
                    "phone" => "Number",
                    "number" => "0",
                    _ => "Value",
                };
                ui.add(egui::TextEdit::singleline(&mut f.value).hint_text(hint_text).desired_width(FIELD_W - 80.0).font(theme::regular(13.5)).margin(Margin::symmetric(8, 5)));
            }
            if remove_button(ui, t) {
                remove = Some(i);
            }
        });
    }
    if let Some(i) = remove {
        items.remove(i);
    }
    if add_link(ui, t, "Add a field") {
        items.push(CustomField { label: String::new(), value: String::new(), kind: "text".into() });
    }
}

fn label_chips(ui: &mut Ui, t: &Tokens, labels: &mut Vec<String>, all: &[String]) {
    ui.horizontal_wrapped(|ui| {
        lbl(ui, t, "Labels");
        let mut remove = None;
        for (i, l) in labels.iter().enumerate() {
            let g = ui.painter().layout_no_wrap(l.clone(), theme::regular(12.5), t.text);
            let (r, resp) = ui.allocate_exact_size(vec2(g.size().x + 34.0, 26.0), Sense::click());
            resp.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, format!("Remove label {l}")));
            ui.painter().rect_filled(r, 13.0, if resp.hovered() { t.hover } else { t.accent_soft });
            ui.painter().galley(pos2(r.min.x + 10.0, r.center().y - g.size().y / 2.0), g, t.accent_soft_text);
            icons::paint(ui.painter(), Rect::from_center_size(pos2(r.max.x - 13.0, r.center().y), Vec2::splat(10.0)), Icon::Close, t.accent_soft_text, t.accent_soft_text);
            if resp.on_hover_text("Remove").clicked() {
                remove = Some(i);
            }
        }
        if let Some(i) = remove {
            labels.remove(i);
        }
        let id = ui.make_persistent_id("new_label_text");
        let mut s: String = ui.data(|d| d.get_temp(id)).unwrap_or_default();
        egui::ComboBox::from_id_salt("add_label").width(150.0).selected_text("Add a label").show_ui(ui, |ui| {
            let candidates: Vec<String> = all.iter().filter(|l| !labels.iter().any(|x| x.eq_ignore_ascii_case(l))).cloned().collect();
            for l in candidates.iter() {
                if ui.selectable_label(false, l).clicked() {
                    labels.push(l.clone());
                }
            }
            ui.separator();
            ui.horizontal(|ui| {
                let r = ui.add(egui::TextEdit::singleline(&mut s).hint_text("New label").desired_width(130.0));
                if (ui.button("Add").clicked() || (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))) && !s.trim().is_empty() {
                    labels.push(s.trim().to_string());
                    s.clear();
                }
            });
        });
        ui.data_mut(|d| d.insert_temp(id, s));
    });
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}
