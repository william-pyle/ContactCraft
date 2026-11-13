//! The contact card: the theme's banner and background, the picture, and every detail the
//! contact holds, in sections. Also the theme painting the theme editor's preview shares.

use contactcraft_store::{Contact, ContactTheme};
use egui::{Align2, Color32, CornerRadius, FontFamily, FontId, Rect, ScrollArea, Sense, Stroke, StrokeKind, Ui, Vec2, pos2, vec2};

use crate::app::{App, Pane};
use crate::icons::{self, Icon};
use crate::theme;
use crate::widgets;

/// The colours and type a contact's theme gives their card.
#[derive(Clone)]
pub struct Look {
    pub accent: Color32,
    pub accent2: Color32,
    pub banner_ink: Color32,
    /// The page behind the details: the theme's colour, or None for the app's surface.
    pub bg: Option<Color32>,
    pub text: Color32,
    pub dim: Color32,
    pub link: Color32,
    pub family: FontFamily,
    pub size: f32,
}

impl Look {
    pub fn font(&self, scale: f32) -> FontId {
        FontId::new(self.size * scale, self.family.clone())
    }
}

impl App {
    pub fn look(&mut self, c: &Contact, has_image: bool) -> Look {
        let t = self.t;
        let th = &c.theme;
        let accent = widgets::contact_color(c);
        let accent2 = theme::hex(&th.accent2).unwrap_or_else(|| accent.linear_multiply(0.55).to_opaque());
        let banner_ink = theme::hex(&th.text).unwrap_or_else(|| theme::ink_on(mix(accent, accent2, 0.5)));
        let bg = theme::hex(&th.background);
        let base_text = match bg {
            Some(b) => theme::ink_on(b),
            None if has_image => theme::ink_on(t.surface),
            None => t.text,
        };
        let text = theme::hex(&th.text_color).unwrap_or(base_text);
        let dim = if theme::hex(&th.text_color).is_some() || bg.is_some() { text.gamma_multiply(0.72) } else { t.text_dim };
        let link = theme::hex(&th.link_color).unwrap_or(if bg.is_some() && theme::luminance(bg.unwrap()) < 120.0 { Color32::from_rgb(0x9C, 0xC8, 0xFF) } else { t.accent_text });
        let family = self.theme_family(c).unwrap_or(FontFamily::Proportional);
        let size = if th.text_size >= 8.0 { th.text_size.min(40.0) } else { 13.5 };
        Look { accent, accent2, banner_ink, bg, text, dim, link, family, size }
    }

    /// The banner as a picture (gradient, stripes, dots...), cached per theme.
    pub fn banner_texture(&mut self, ctx: &egui::Context, th: &ContactTheme, accent: Color32, accent2: Color32) -> Option<egui::TextureId> {
        let key = format!("banner|{}|{:?}|{:?}", th.style, accent, accent2);
        if let Some(t) = self.textures.get(&key) {
            return t.as_ref().map(|t| t.id());
        }
        let img = banner_image(&th.style, accent, accent2);
        let h = ctx.load_texture(key.clone(), img, egui::TextureOptions::LINEAR);
        let id = h.id();
        self.textures.insert(key, Some(h));
        Some(id)
    }

    pub fn paint_banner(&mut self, ui: &Ui, rect: Rect, c: &Contact, look: &Look, corner: CornerRadius) {
        let ctx = ui.ctx().clone();
        if c.theme.style == "solid" {
            ui.painter().rect_filled(rect, corner, look.accent);
            return;
        }
        if let Some(tex) = self.banner_texture(&ctx, &c.theme, look.accent, look.accent2) {
            ui.painter().add(egui::epaint::RectShape::filled(rect, corner, Color32::WHITE).with_texture(tex, Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0))));
        }
    }

    /// The theme's background colour and picture, filling `rect`.
    pub fn paint_backdrop(&mut self, ui: &Ui, rect: Rect, c: &Contact, look: &Look, corner: CornerRadius) {
        let ctx = ui.ctx().clone();
        if let Some(bg) = look.bg {
            ui.painter().rect_filled(rect, corner, bg);
        }
        let Some(path) = self.store.as_ref().and_then(|s| s.background_path(c)) else { return };
        let fit = c.theme.background_fit.as_str();
        let Some(tex) = self.texture(&ctx, &path, c.theme.background_blur, fit == "tile") else { return };
        let Some(size) = self.texture_size(&ctx, tex) else { return };
        let alpha = (c.theme.background_opacity.min(100) as f32 / 100.0 * 255.0) as u8;
        let tint = Color32::from_white_alpha(alpha);
        let full = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
        let p = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
        let shape = |r: Rect, uv: Rect| egui::epaint::RectShape::filled(r, corner, tint).with_texture(tex, uv);
        match fit {
            "stretch" => {
                p.add(shape(rect, full));
            }
            "tile" => {
                let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(rect.width() / size.x.max(1.0), rect.height() / size.y.max(1.0)));
                p.add(shape(rect, uv));
            }
            "contain" | "center" => {
                let s = if fit == "center" { 1.0f32.min(rect.width() / size.x).min(rect.height() / size.y) } else { (rect.width() / size.x).min(rect.height() / size.y) };
                let r = Rect::from_center_size(rect.center(), size * s);
                p.add(egui::epaint::RectShape::filled(r, 0.0, tint).with_texture(tex, full));
            }
            _ => {
                // cover: fill the whole area, cropping the picture's overflow.
                let ra = rect.width() / rect.height().max(1.0);
                let ia = size.x / size.y.max(1.0);
                let uv = if ia > ra {
                    let w = ra / ia;
                    Rect::from_min_max(pos2((1.0 - w) / 2.0, 0.0), pos2((1.0 + w) / 2.0, 1.0))
                } else {
                    let h = ia / ra;
                    Rect::from_min_max(pos2(0.0, (1.0 - h) / 2.0), pos2(1.0, (1.0 + h) / 2.0))
                };
                p.add(shape(rect, uv));
            }
        }
    }

    pub fn contact_card(&mut self, ui: &mut Ui) {
        let t = self.t;
        let rect = ui.max_rect();
        ui.painter().rect_filled(rect, 8.0, t.surface);
        let Some(c) = self.selected_contact() else {
            ui.painter().rect_stroke(rect, 8.0, Stroke::new(1.0, t.border), StrokeKind::Inside);
            let ce = rect.center();
            icons::paint(ui.painter(), Rect::from_center_size(pos2(ce.x, ce.y - 40.0), Vec2::splat(56.0)), Icon::Card, t.text_disabled, t.accent);
            let msg = if self.contacts.is_empty() { "Your contacts will show here" } else { "Select a contact to see it" };
            ui.painter().text(pos2(ce.x, ce.y + 8.0), Align2::CENTER_TOP, msg, theme::medium(15.0), t.text);
            ui.painter().text(pos2(ce.x, ce.y + 32.0), Align2::CENTER_TOP, "MessageCraft and MailCraft see the same contacts, pictures and themes.", theme::regular(12.5), t.text_dim);
            return;
        };
        let has_image = self.store.as_ref().is_some_and(|s| s.background_path(&c).is_some());
        let look = self.look(&c, has_image);
        self.paint_backdrop(ui, rect, &c, &look, CornerRadius::same(8));

        // Banner, picture, name.
        let banner = Rect::from_min_size(rect.min, vec2(rect.width(), 128.0));
        self.paint_banner(ui, banner, &c, &look, CornerRadius { nw: 8, ne: 8, sw: 0, se: 0 });
        ui.painter().rect_stroke(rect, 8.0, Stroke::new(1.0, t.border), StrokeKind::Inside);
        let ctx = ui.ctx().clone();
        let photo = self.photo_texture(&ctx, &c);
        let ac = pos2(rect.min.x + 72.0, banner.max.y);
        let ring = look.bg.unwrap_or(t.surface);
        let rr = Rect::from_center_size(ac, Vec2::splat(96.0));
        let corner = match c.theme.avatar_shape.as_str() {
            "square" => 8.0,
            "rounded" => 22.0,
            _ => 48.0,
        };
        ui.painter().rect_filled(rr, corner, ring);
        widgets::contact_avatar(ui.painter(), &c, ac, 44.0, photo);
        let name = c.display_name();
        let name_font = FontId::new((look.size * 1.75).clamp(20.0, 40.0), look.family.clone());
        let mut nx = rr.max.x + 16.0;
        if !c.theme.emoji.is_empty() {
            ui.painter().text(pos2(rect.max.x - 24.0, banner.min.y + 22.0), Align2::RIGHT_TOP, &c.theme.emoji, theme::regular(34.0), look.banner_ink);
        }
        let name_w = (rect.max.x - nx - 90.0).max(60.0);
        widgets::clipped(ui.painter(), pos2(nx, banner.max.y - 30.0), name_w, &name, name_font, look.banner_ink);
        let role = [c.job_title(), c.company()].iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join(" · ");
        let mut sub = role.clone();
        if !c.name.nickname.is_empty() && !name.contains(&c.name.nickname) {
            sub = if sub.is_empty() { format!("“{}”", c.name.nickname) } else { format!("“{}”  ·  {sub}", c.name.nickname) };
        }
        if !c.pronouns.is_empty() {
            sub = if sub.is_empty() { c.pronouns.clone() } else { format!("{sub}  ·  {}", c.pronouns) };
        }
        widgets::clipped(ui.painter(), pos2(nx, banner.max.y - 10.0), name_w, &sub, look.font(0.92), look.banner_ink.gamma_multiply(0.85));
        if c.favorite {
            icons::paint(ui.painter(), Rect::from_min_size(pos2(rr.max.x - 26.0, rr.min.y + 2.0), Vec2::splat(22.0)), Icon::Star, t.icon_yellow, t.icon_yellow);
        }

        // Actions, under the banner.
        let bar = Rect::from_min_max(pos2(rr.max.x + 12.0, banner.max.y + 10.0), pos2(rect.max.x - 20.0, banner.max.y + 44.0));
        let mut bu = ui.new_child(egui::UiBuilder::new().max_rect(bar).layout(egui::Layout::left_to_right(egui::Align::Center)));
        bu.spacing_mut().item_spacing.x = 8.0;
        let mut action = None;
        if widgets::primary_button(&mut bu, &t, "Edit", true).clicked() {
            action = Some(1);
        }
        if widgets::secondary_button(&mut bu, &t, "Theme").clicked() {
            action = Some(2);
        }
        let email = c.emails.iter().find(|e| !e.address.is_empty()).map(|e| e.address.clone());
        if let Some(e) = &email {
            if widgets::secondary_button(&mut bu, &t, "Email").on_hover_text(e).clicked() {
                crate::app::open_url(&format!("mailto:{e}"));
            }
        }
        if widgets::icon_button(&mut bu, &t, Icon::Star, 30.0, if c.favorite { "Remove from favorites" } else { "Add to favorites" }).clicked() {
            action = Some(3);
        }
        if widgets::icon_button(&mut bu, &t, Icon::Trash, 30.0, "Delete").clicked() {
            action = Some(4);
        }
        nx = rect.min.x + 28.0;

        // The details.
        let body = Rect::from_min_max(pos2(nx, banner.max.y + 62.0), pos2(rect.max.x - 12.0, rect.max.y - 8.0));
        let mut u = ui.new_child(egui::UiBuilder::new().max_rect(body).layout(egui::Layout::top_down(egui::Align::Min)));
        let mut copied = None;
        ScrollArea::vertical().auto_shrink([false, false]).id_salt(("card", c.id.clone())).show(&mut u, |ui| {
            ui.set_max_width((body.width() - 16.0).min(760.0));
            details(ui, &t, &c, &look, &mut copied);
            ui.add_space(16.0);
        });
        if let Some(text) = copied {
            ui.ctx().copy_text(text.clone());
            self.notify(format!("Copied {text}"), false);
        }
        match action {
            Some(1) => self.pane = Pane::Edit(Box::new(c.clone())),
            Some(2) => self.pane = Pane::Theme(Box::new(c.clone())),
            Some(3) => self.toggle_favorite(&c.id),
            Some(4) => self.confirm_delete = Some(c.id.clone()),
            _ => {}
        }
    }
}

fn mix(a: Color32, b: Color32, k: f32) -> Color32 {
    let m = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * k).round() as u8;
    Color32::from_rgb(m(a.r(), b.r()), m(a.g(), b.g()), m(a.b(), b.b()))
}

/// The banner pattern as a 480×160 picture.
pub fn banner_image(style: &str, a: Color32, b: Color32) -> egui::ColorImage {
    let (w, h) = (480usize, 160usize);
    let mut px = vec![Color32::BLACK; w * h];
    for y in 0..h {
        for x in 0..w {
            let fx = x as f32 / w as f32;
            let fy = y as f32 / h as f32;
            let c = match style {
                "diagonal" => mix(a, b, ((fx * 0.75 + fy * 0.25) * 1.0).clamp(0.0, 1.0)),
                "stripes" => {
                    let band = ((x as f32 + y as f32) / 22.0).floor() as i32;
                    if band % 2 == 0 { a } else { mix(a, b, 0.65) }
                }
                "dots" => {
                    let (cx, cy) = ((x % 24) as f32 - 12.0, (y % 24) as f32 - 12.0);
                    let base = mix(a, b, fx * 0.4);
                    if cx * cx + cy * cy < 20.0 { mix(base, b, 0.85) } else { base }
                }
                "glow" => {
                    let d1 = ((fx - 0.8).powi(2) * 3.0 + (fy - 0.2).powi(2)).sqrt();
                    let d2 = ((fx - 0.15).powi(2) * 3.0 + (fy - 1.0).powi(2)).sqrt();
                    let g = (1.0 - d1 * 1.6).max(0.0).max((1.0 - d2 * 1.8).max(0.0));
                    mix(a, b, g.clamp(0.0, 1.0))
                }
                _ => mix(a, b, fx),
            };
            px[y * w + x] = c;
        }
    }
    egui::ColorImage::new([w, h], px)
}

fn cap(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => "Other".into(),
    }
}

/// A date as words: "April 12, 1985 (40)" or "April 12".
pub fn nice_date(d: &str) -> String {
    const MONTHS: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
    let parts: Vec<&str> = d.trim_start_matches("--").split('-').collect();
    let (y, m, day) = match parts.as_slice() {
        [m, dd] if d.starts_with("--") => (None, m.parse::<usize>().ok(), dd.parse::<u32>().ok()),
        [y, m, dd] => (y.parse::<i32>().ok(), m.parse::<usize>().ok(), dd.parse::<u32>().ok()),
        _ => return d.to_string(),
    };
    let (Some(m), Some(day)) = (m, day) else { return d.to_string() };
    let Some(month) = MONTHS.get(m.wrapping_sub(1)) else { return d.to_string() };
    match y {
        Some(y) => {
            use chrono::Datelike;
            let now = chrono::Local::now().date_naive();
            let mut age = now.year() - y;
            if (now.month() as usize, now.day()) < (m, day) {
                age -= 1;
            }
            if (0..130).contains(&age) { format!("{month} {day}, {y}  ({age})") } else { format!("{month} {day}, {y}") }
        }
        None => format!("{month} {day}"),
    }
}

fn details(ui: &mut Ui, t: &theme::Tokens, c: &Contact, look: &Look, copied: &mut Option<String>) {
    ui.spacing_mut().item_spacing.y = 5.0;
    let label_w = (look.size * 9.5).clamp(110.0, 190.0);
    let section = |ui: &mut Ui, icon: Icon, title: &str| {
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
            icons::paint(ui.painter(), r, icon, look.dim, look.accent);
            ui.label(egui::RichText::new(title).font(FontId::new(look.size * 1.04, look.family.clone())).strong().color(look.text));
        });
        let w = ui.available_width();
        let (r, _) = ui.allocate_exact_size(vec2(w, 1.0), Sense::hover());
        ui.painter().line_segment([r.left_center(), r.right_center()], Stroke::new(1.0, if look.bg.is_some() { look.dim.gamma_multiply(0.4) } else { t.rule }));
    };
    // label | value; `link`: 1 copy, 2 mailto, 3 web.
    let field = |ui: &mut Ui, label: &str, value: &str, link: u8, copied: &mut Option<String>| {
        if value.trim().is_empty() {
            return;
        }
        ui.horizontal_top(|ui| {
            widgets::field_label(ui, label_w, look.size + 6.0, egui::RichText::new(cap(label)).font(look.font(0.9)).color(look.dim));
            let rt = egui::RichText::new(value).font(look.font(1.0));
            match link {
                0 => {
                    ui.add(egui::Label::new(rt.color(look.text)).wrap());
                }
                _ => {
                    let resp = ui.add(egui::Label::new(rt.color(look.link)).sense(Sense::click()).wrap());
                    let tip = match link {
                        2 => "Write an email",
                        3 => "Open in your browser",
                        _ => "Copy",
                    };
                    let resp = resp.on_hover_text(tip).on_hover_cursor(egui::CursorIcon::PointingHand);
                    if resp.clicked() {
                        match link {
                            2 => crate::app::open_url(&format!("mailto:{}", value.trim())),
                            3 => {
                                let v = value.trim();
                                let url = if v.contains("://") { v.to_string() } else { format!("https://{v}") };
                                if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("ftp://") {
                                    crate::app::open_url(&url);
                                }
                            }
                            _ => *copied = Some(value.trim().to_string()),
                        }
                    }
                }
            }
        });
    };

    // Contact
    let has_contact = !c.phones.is_empty() || !c.emails.is_empty() || !c.im.is_empty() || !c.sip.is_empty();
    if has_contact {
        section(ui, Icon::Phone, "Contact");
        for p in &c.phones {
            field(ui, &p.label, &p.value, 1, copied);
        }
        for e in &c.emails {
            let v = if e.display_name.is_empty() { e.address.clone() } else { format!("{}  <{}>", e.display_name, e.address) };
            if e.display_name.is_empty() {
                field(ui, &format!("{} email", e.label), &v, 2, copied);
            } else {
                field(ui, &format!("{} email", e.label), &e.address, 2, copied);
            }
        }
        for i in &c.im {
            field(ui, &if i.protocol.is_empty() { i.label.clone() } else { i.protocol.clone() }, &i.address, 1, copied);
        }
        for s in &c.sip {
            field(ui, &format!("SIP {}", s.label), &s.value, 1, copied);
        }
    }
    if c.addresses.iter().any(|a| !a.is_empty()) {
        section(ui, Icon::Location, "Addresses");
        for a in c.addresses.iter().filter(|a| !a.is_empty()) {
            let mut label = cap(&a.label);
            if a.mailing {
                label.push_str(" (mailing)");
            }
            field(ui, &label, &a.lines().join("\n"), 1, copied);
        }
    }
    if !c.websites.is_empty() || !c.social_profiles.is_empty() {
        section(ui, Icon::Globe, "Web and social");
        for w in &c.websites {
            field(ui, &w.label, &w.value, 3, copied);
        }
        for s in &c.social_profiles {
            field(ui, &s.label, &s.value, 3, copied);
        }
    }
    if !c.organizations.is_empty() || !c.occupation.is_empty() {
        section(ui, Icon::Building, "Work");
        for o in &c.organizations {
            let mut head = o.company.clone();
            if !o.department.is_empty() {
                head = if head.is_empty() { o.department.clone() } else { format!("{head} — {}", o.department) };
            }
            field(ui, if o.label.is_empty() { "work" } else { &o.label }, &head, 0, copied);
            field(ui, "job title", &o.title, 0, copied);
            field(ui, "office", &o.office, 0, copied);
            field(ui, "job description", &o.job_description, 0, copied);
            field(ui, "phonetic company", &o.phonetic_company, 0, copied);
            field(ui, "stock symbol", &o.symbol, 0, copied);
            field(ui, "cost center", &o.cost_center, 0, copied);
            field(ui, "domain", &o.domain, 3, copied);
            if !o.start_date.is_empty() || !o.end_date.is_empty() {
                let span = format!("{} – {}", if o.start_date.is_empty() { "?" } else { &o.start_date }, if o.end_date.is_empty() { if o.current { "now" } else { "?" } } else { &o.end_date });
                field(ui, "dates", &span, 0, copied);
            }
        }
        field(ui, "profession", &c.occupation, 0, copied);
    }
    let has_dates = !c.birthday.is_empty() || !c.anniversary.is_empty() || !c.events.is_empty() || !c.birthplace.is_empty() || !c.deathdate.is_empty();
    if has_dates {
        section(ui, Icon::Cake, "Dates");
        if !c.birthday.is_empty() {
            field(ui, "birthday", &nice_date(&c.birthday), 0, copied);
        }
        field(ui, "birthplace", &c.birthplace, 0, copied);
        if !c.anniversary.is_empty() {
            field(ui, "anniversary", &nice_date(&c.anniversary), 0, copied);
        }
        for e in &c.events {
            field(ui, &e.label, &nice_date(&e.value), 0, copied);
        }
        if !c.deathdate.is_empty() {
            field(ui, "died", &nice_date(&c.deathdate), 0, copied);
        }
        field(ui, "place of death", &c.deathplace, 0, copied);
    }
    if !c.relations.is_empty() {
        section(ui, Icon::People, "Relationships");
        for r in &c.relations {
            field(ui, &r.label, &r.value, 0, copied);
        }
    }
    let n = &c.name;
    let name_bits: Vec<(&str, &String)> = vec![
        ("prefix", &n.prefix),
        ("first", &n.given),
        ("middle", &n.middle),
        ("last", &n.family),
        ("suffix", &n.suffix),
        ("nickname", &n.nickname),
        ("maiden name", &n.maiden),
        ("phonetic first", &n.phonetic_given),
        ("phonetic middle", &n.phonetic_middle),
        ("phonetic last", &n.phonetic_family),
        ("initials", &n.initials),
        ("file as", &n.file_as),
    ];
    if name_bits.iter().filter(|(_, v)| !v.is_empty()).count() > 2 || !n.phonetic_given.is_empty() || !n.file_as.is_empty() {
        section(ui, Icon::Card, "Name");
        for (l, v) in name_bits {
            field(ui, l, v, 0, copied);
        }
    }
    let personal = !c.gender.is_empty() || !c.languages.is_empty() || !c.hobbies.is_empty() || !c.interests.is_empty() || !c.skills.is_empty() || !c.expertise.is_empty() || !c.time_zone.is_empty() || !c.geo.is_empty() || (c.kind != "individual" && !c.kind.is_empty());
    if personal {
        section(ui, Icon::Person, "Personal");
        if c.kind != "individual" {
            field(ui, "kind", &cap(&c.kind), 0, copied);
        }
        field(ui, "gender", &cap_or_empty(&c.gender), 0, copied);
        field(ui, "pronouns", &c.pronouns, 0, copied);
        field(ui, "languages", &c.languages.join(", "), 0, copied);
        field(ui, "time zone", &c.time_zone, 0, copied);
        field(ui, "location (lat, lon)", &c.geo, 1, copied);
        field(ui, "hobbies", &c.hobbies.join(", "), 0, copied);
        field(ui, "interests", &c.interests.join(", "), 0, copied);
        field(ui, "skills", &c.skills.join(", "), 0, copied);
        field(ui, "expertise", &c.expertise.join(", "), 0, copied);
    }
    if !c.external_ids.is_empty() || !c.locations.is_empty() || !c.calendar_urls.is_empty() || !c.public_keys.is_empty() {
        section(ui, Icon::Tag, "IDs, places and calendars");
        for x in &c.external_ids {
            field(ui, &format!("{} ID", x.label), &x.value, 1, copied);
        }
        for x in &c.locations {
            field(ui, &x.label, &x.value, 0, copied);
        }
        for x in &c.calendar_urls {
            field(ui, &format!("{} calendar", x.label), &x.value, 3, copied);
        }
        for x in &c.public_keys {
            field(ui, &format!("{} key", x.label), &x.value, 1, copied);
        }
    }
    let o = &c.outlook;
    let outlook: Vec<(&str, String)> = vec![
        ("account", o.account.clone()),
        ("customer ID", o.customer_id.clone()),
        ("government ID", o.government_id.clone()),
        ("organizational ID", o.organizational_id.clone()),
        ("billing information", o.billing_information.clone()),
        ("mileage", o.mileage.clone()),
        ("directory server", o.directory_server.clone()),
        ("computer network", o.computer_network_name.clone()),
        ("FTP site", o.ftp_site.clone()),
        ("free/busy address", o.internet_free_busy.clone()),
        ("subject", o.subject.clone()),
        ("keywords", o.keywords.clone()),
        ("user 1", o.user1.clone()),
        ("user 2", o.user2.clone()),
        ("user 3", o.user3.clone()),
        ("user 4", o.user4.clone()),
        ("priority", if o.priority == "normal" { String::new() } else { cap_or_empty(&o.priority) }),
        ("sensitivity", if o.sensitivity == "normal" { String::new() } else { cap_or_empty(&o.sensitivity) }),
        ("private", if o.private { "Yes".into() } else { String::new() }),
        ("follow up", if o.follow_up_flag.is_empty() { String::new() } else { format!("{}{}{}", o.follow_up_flag, if o.follow_up_due.is_empty() { String::new() } else { format!(" — due {}", o.follow_up_due) }, if o.follow_up_done { " (done)" } else { "" }) }),
    ];
    if outlook.iter().any(|(_, v)| !v.is_empty()) {
        section(ui, Icon::Mail, "Outlook details");
        for (l, v) in &outlook {
            field(ui, l, v, 0, copied);
        }
    }
    if !c.custom_fields.is_empty() {
        section(ui, Icon::Plus, "More fields");
        for f in &c.custom_fields {
            let link = match f.kind.as_str() {
                "url" => 3,
                "email" => 2,
                "phone" => 1,
                _ => 0,
            };
            field(ui, &f.label, &f.value, link, copied);
        }
    }
    if !c.labels.is_empty() {
        section(ui, Icon::Tag, "Labels");
        ui.horizontal_wrapped(|ui| {
            for l in &c.labels {
                let g = ui.painter().layout_no_wrap(l.clone(), look.font(0.9), look.text);
                let (r, _) = ui.allocate_exact_size(vec2(g.size().x + 20.0, g.size().y + 8.0), Sense::hover());
                ui.painter().rect_filled(r, CornerRadius::same(12), look.accent.gamma_multiply(0.18));
                ui.painter().rect_stroke(r, 12.0, Stroke::new(1.0, look.accent.gamma_multiply(0.55)), StrokeKind::Inside);
                ui.painter().galley(pos2(r.min.x + 10.0, r.min.y + 4.0), g, look.text);
            }
        });
    }
    if !c.notes.trim().is_empty() {
        section(ui, Icon::Edit, "Notes");
        ui.add(egui::Label::new(egui::RichText::new(&c.notes).font(look.font(1.0)).color(look.text)).wrap());
    }
    if !c.extra.is_empty() {
        section(ui, Icon::Info, "From other apps");
        for (k, v) in &c.extra {
            let text = match v {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            field(ui, k, &text, 1, copied);
        }
    }
    ui.add_space(14.0);
    let stamp = |s: &str| chrono::DateTime::parse_from_rfc3339(s).map(|d| d.with_timezone(&chrono::Local).format("%b %e, %Y %l:%M %p").to_string()).unwrap_or_default();
    ui.label(egui::RichText::new(format!("Added {}  ·  Changed {}", stamp(&c.created), stamp(&c.updated))).font(theme::regular(11.5)).color(look.dim));
}

fn cap_or_empty(s: &str) -> String {
    if s.is_empty() { String::new() } else { cap(s) }
}

/// The contact as plain text, for the clipboard.
pub fn as_text(c: &Contact) -> String {
    let mut out = vec![c.display_name()];
    let role = [c.job_title(), c.company()].iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join(", ");
    if !role.is_empty() {
        out.push(role);
    }
    for p in &c.phones {
        out.push(format!("{}: {}", cap(&p.label), p.value));
    }
    for e in &c.emails {
        out.push(format!("{} email: {}", cap(&e.label), e.address));
    }
    for a in &c.addresses {
        if !a.is_empty() {
            out.push(format!("{}:\n  {}", cap(&a.label), a.lines().join("\n  ")));
        }
    }
    for w in &c.websites {
        out.push(w.value.clone());
    }
    if !c.birthday.is_empty() {
        out.push(format!("Birthday: {}", nice_date(&c.birthday)));
    }
    out.join("\n")
}
