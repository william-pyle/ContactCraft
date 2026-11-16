//! The window's state: the contacts (read from the shared store), what is selected and being
//! edited, settings, pictures and fonts, and the loop that keeps it all current.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use contactcraft_store::{Contact, ContactGroup, GroupMember, Store};
use egui::{ColorImage, TextureHandle};
use serde::{Deserialize, Serialize};

use crate::theme::Tokens;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Appearance {
    #[default]
    System,
    Light,
    Dark,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub appearance: Appearance,
    pub zoom: u16,
    pub label_pane: bool,
    pub tighter: bool,
    /// Sort by family name ("Smith, Anna") instead of the displayed name.
    pub sort_by_family: bool,
    /// Look for a new version on GitHub when ContactCraft starts.
    pub check_updates: bool,
    /// A version the person chose to skip.
    pub skip_version: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { appearance: Appearance::System, zoom: 100, label_pane: true, tighter: false, sort_by_family: false, check_updates: true, skip_version: String::new() }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RibbonTab {
    Home,
    View,
    Help,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backstage {
    Info,
    Import,
    Export,
    Sharing,
    Options,
    About,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Filter {
    All,
    Favorites,
    Themed,
    Recent,
    Unlabeled,
    Label(String),
    Group(String),
}

/// What the right-hand pane shows.
#[derive(Clone, Debug, PartialEq)]
pub enum Pane {
    Card,
    Edit(Box<Contact>),
    Theme(Box<Contact>),
}

pub struct Notice {
    pub text: String,
    pub error: bool,
    pub until: Instant,
}

/// Contacts read from a file, waiting for the person to say "Import".
pub struct ImportPreview {
    pub file: PathBuf,
    pub items: Vec<(Contact, Option<Vec<u8>>)>,
    pub groups: Vec<ContactGroup>,
    pub skip_duplicates: bool,
    pub label: String,
}

pub struct App {
    pub t: Tokens,
    pub store: Option<Store>,
    pub store_error: Option<String>,
    pub contacts: Vec<Contact>,
    pub groups: Vec<ContactGroup>,
    pub active_group: Option<String>,
    pub group_name: String,
    pub group_number: String,
    pub pending_group: Option<String>,
    pub pending_new_group: bool,
    pub revision: i64,
    last_poll: Instant,
    pub selected: Option<String>,
    pub filter: Filter,
    pub search: String,
    pub pane: Pane,
    /// Editor sections the person closed.
    pub closed_sections: HashSet<&'static str>,
    pub confirm_delete: Option<String>,
    pub confirm_group_delete: Option<String>,
    pub confirm_discard: Option<Pane>,
    pub ribbon: RibbonTab,
    pub backstage: Option<Backstage>,
    pub settings: Settings,
    pub notice: Option<Notice>,
    pub focus_search: bool,
    pub import: Option<ImportPreview>,
    pub new_label: Option<String>,
    pub cleanup_dialog: Option<crate::cleanup_ui::CleanupDialog>,
    pub merge: Option<crate::cleanup_ui::MergeSession>,
    pub updater: crate::update::Updater,
    /// Started by another app for one contact (see dialog.rs).
    pub dialog: Option<crate::dialog::DialogState>,
    pub textures: HashMap<String, Option<TextureHandle>>,
    /// Font files registered with egui (usable from the frame after they're added).
    fonts_ready: HashSet<String>,
    fonts_wanted: HashSet<String>,
    fonts_bad: HashSet<String>,
    last_title: String,
    pub demo: bool,
    /// The app's fonts, to which theme fonts are added.
    font_defs: egui::FontDefinitions,
}

const SETTINGS_FILE: &str = "contactcraft-ui.json";

pub fn settings_dir() -> PathBuf {
    let d = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(".")).join("ContactCraft");
    let _ = std::fs::create_dir_all(&d);
    d
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let font_defs = crate::theme::install_fonts(&cc.egui_ctx);
        crate::theme::apply(&cc.egui_ctx);
        let args: Vec<String> = std::env::args().collect();
        let demo = args.iter().any(|a| a == "--demo");
        let mut settings: Settings = std::fs::read(settings_dir().join(SETTINGS_FILE)).ok().and_then(|b| serde_json::from_slice(&b).ok()).unwrap_or_default();
        if args.iter().any(|a| a == "--light") {
            settings.appearance = Appearance::Light;
        } else if args.iter().any(|a| a == "--dark") {
            settings.appearance = Appearance::Dark;
        }
        // The demo works on a private copy in the temp folder, so nothing real changes.
        let opened = if demo {
            let dir = std::env::temp_dir().join(format!("contactcraft-demo-{}", std::process::id()));
            Store::open(&dir)
        } else {
            Store::open_default()
        };
        let (store, store_error) = match opened {
            Ok(s) => (Some(s), None),
            Err(e) => (None, Some(e)),
        };
        let mut app = App {
            t: Tokens::light(),
            store,
            store_error,
            contacts: Vec::new(),
            groups: Vec::new(),
            active_group: None,
            group_name: String::new(),
            group_number: String::new(),
            pending_group: None,
            pending_new_group: false,
            revision: -1,
            last_poll: Instant::now(),
            selected: None,
            filter: Filter::All,
            search: String::new(),
            pane: Pane::Card,
            closed_sections: HashSet::new(),
            confirm_delete: None,
            confirm_group_delete: None,
            confirm_discard: None,
            ribbon: RibbonTab::Home,
            backstage: None,
            settings,
            notice: None,
            focus_search: false,
            import: None,
            new_label: None,
            cleanup_dialog: None,
            merge: None,
            updater: crate::update::Updater::default(),
            dialog: None,
            textures: HashMap::new(),
            fonts_ready: HashSet::new(),
            fonts_wanted: HashSet::new(),
            fonts_bad: HashSet::new(),
            last_title: String::new(),
            demo,
            font_defs,
        };
        if demo {
            crate::demo::fill(&mut app);
        }
        app.reload();
        app.selected = app.contacts.first().map(|c| c.id.clone());
        let dialog_mode = crate::dialog::parse(&args).is_some();
        // Open straight onto a page (handy for the demo and for screenshots).
        if let Some(c) = app.selected_contact() {
            if args.iter().any(|a| a == "--edit") {
                app.pane = Pane::Edit(Box::new(c));
            } else if args.iter().any(|a| a == "--theme") {
                app.pane = Pane::Theme(Box::new(c));
            }
        }
        if demo && args.iter().any(|a| a == "--cleanup") {
            app.open_cleanup();
        }
        if demo && args.iter().any(|a| a == "--cleanup-run") {
            app.run_cleanup(crate::cleanup_ui::CleanupDialog::default());
        }
        if args.iter().any(|a| a == "--file") {
            app.backstage = Some(Backstage::Sharing);
        }
        app.apply_settings(&cc.egui_ctx);
        // Let other Craft apps find this program through the store.
        if !demo {
            if let (Some(store), Ok(exe)) = (&app.store, std::env::current_exe()) {
                let _ = store.set_meta("contactcraft.exe", Some(&exe.display().to_string()));
            }
        }
        if !demo && !dialog_mode && app.settings.check_updates && !args.iter().any(|a| a == "--no-update-check") {
            app.check_for_updates(&cc.egui_ctx, false);
        }
        app
    }

    pub fn apply_settings(&self, ctx: &egui::Context) {
        ctx.set_theme(match self.settings.appearance {
            Appearance::System => egui::ThemePreference::System,
            Appearance::Light => egui::ThemePreference::Light,
            Appearance::Dark => egui::ThemePreference::Dark,
        });
        ctx.set_zoom_factor(self.settings.zoom.clamp(80, 200) as f32 / 100.0);
    }

    pub fn save_settings(&self) {
        if let Ok(b) = serde_json::to_vec_pretty(&self.settings) {
            let _ = std::fs::write(settings_dir().join(SETTINGS_FILE), b);
        }
    }

    pub fn notify(&mut self, text: impl Into<String>, error: bool) {
        self.notice = Some(Notice { text: text.into(), error, until: Instant::now() + Duration::from_secs(if error { 9 } else { 4 }) });
    }

    /// Read contacts and groups from the store again.
    pub fn reload(&mut self) {
        let Some(store) = &self.store else { return };
        let revision = store.revision();
        let contacts = store.list();
        let groups = store.list_groups();
        match contacts {
            Ok(mut list) => {
                if self.settings.sort_by_family {
                    list.sort_by_key(|c| {
                        let n = &c.name;
                        if n.family.is_empty() { c.sort_key() } else { format!("{} {}", n.family, n.given).to_lowercase() }
                    });
                }
                self.contacts = list;
                self.revision = revision;
            }
            Err(e) => self.notify(format!("Couldn't read the contacts: {e}"), true),
        }
        match groups {
            Ok(groups) => self.groups = groups,
            Err(e) => self.notify(format!("Couldn't read the groups: {e}"), true),
        }
        if self.active_group.as_ref().is_some_and(|id| self.group(id).is_none()) {
            self.active_group = None;
            self.filter = Filter::All;
        }
        if let Some(id) = &self.selected {
            if !self.contacts.iter().any(|c| &c.id == id) {
                self.selected = None;
            }
        }
    }

    /// Pick up contact and group changes other applications made to the store.
    fn poll_store(&mut self) {
        if self.last_poll.elapsed() < Duration::from_millis(1500) {
            return;
        }
        self.last_poll = Instant::now();
        let rev = self.store.as_ref().map(|s| s.revision()).unwrap_or(self.revision);
        if rev != self.revision {
            self.reload();
        }
    }

    pub fn contact(&self, id: &str) -> Option<&Contact> {
        self.contacts.iter().find(|c| c.id == id)
    }

    pub fn selected_contact(&self) -> Option<Contact> {
        self.selected.as_deref().and_then(|id| self.contact(id)).cloned()
    }

    pub fn group(&self, id: &str) -> Option<&ContactGroup> {
        self.groups.iter().find(|g| g.id == id)
    }

    pub fn new_group(&mut self) {
        if self.unsaved() {
            self.pending_new_group = true;
            self.pending_group = None;
            self.confirm_discard = Some(Pane::Card);
            return;
        }
        let Some(store) = &self.store else { return };
        let mut group = ContactGroup::new("New group");
        match store.save_group(&mut group) {
            Ok(()) => {
                let id = group.id.clone();
                self.reload();
                self.select_group(&id);
            }
            Err(e) => self.notify(format!("Couldn't create group: {e}"), true),
        }
    }

    pub fn select_group(&mut self, id: &str) {
        if self.unsaved() {
            self.pending_group = Some(id.to_string());
            self.pending_new_group = false;
            self.confirm_discard = Some(Pane::Card);
            return;
        }
        self.group_name = self.group(id).map(|g| g.name.clone()).unwrap_or_default();
        self.group_number.clear();
        self.active_group = Some(id.to_string());
        self.selected = None;
        self.filter = Filter::Group(id.to_string());
        self.pane = Pane::Card;
    }

    pub fn finish_pending_group_navigation(&mut self) {
        if self.pending_new_group {
            self.pending_new_group = false;
            self.new_group();
        } else if let Some(id) = self.pending_group.take() {
            self.select_group(&id);
        }
    }

    pub fn save_group(&mut self, mut group: ContactGroup) {
        let Some(store) = &self.store else { return };
        match store.save_group(&mut group) {
            Ok(()) => self.reload(),
            Err(e) => self.notify(format!("Couldn't save group: {e}"), true),
        }
    }

    pub fn delete_group(&mut self, id: &str) {
        let Some(store) = &self.store else { return };
        match store.delete_group(id) {
            Ok(()) => {
                self.active_group = None;
                self.filter = Filter::All;
                self.reload();
                self.notify("Group deleted.", false);
            }
            Err(e) => self.notify(format!("Couldn't delete group: {e}"), true),
        }
    }

    /// Save a contact to the shared store and show it.
    pub fn save_contact(&mut self, mut c: Contact) -> bool {
        let Some(store) = &self.store else {
            self.notify("The contact store isn't open.", true);
            return false;
        };
        c.labels.retain(|l| !l.trim().is_empty());
        match store.save(&mut c) {
            Ok(()) => {
                let id = c.id.clone();
                if let Filter::Group(group_id) = &self.filter {
                    if let Some(mut group) = self.group(group_id).cloned() {
                        group.add_contact(id.clone());
                        if let Err(e) = store.save_group(&mut group) {
                            self.notify(format!("Contact saved, but couldn't add it to the group: {e}"), true);
                        }
                    }
                }
                self.reload();
                self.selected = Some(id);
                true
            }
            Err(e) => {
                self.notify(format!("Couldn't save: {e}"), true);
                false
            }
        }
    }

    pub fn delete_contact(&mut self, id: &str) {
        if let Some(store) = &self.store {
            match store.delete(id) {
                Ok(()) => {
                    let name = self.contact(id).map(|c| c.display_name()).unwrap_or_default();
                    let next = self.neighbour(id);
                    self.reload();
                    self.selected = next;
                    self.pane = Pane::Card;
                    self.notify(format!("Deleted {name}."), false);
                }
                Err(e) => self.notify(format!("Couldn't delete: {e}"), true),
            }
        }
    }

    fn neighbour(&self, id: &str) -> Option<String> {
        let shown = self.shown();
        let i = shown.iter().position(|c| c.id == id)?;
        shown.get(i + 1).or_else(|| i.checked_sub(1).and_then(|j| shown.get(j))).map(|c| c.id.clone())
    }

    pub fn toggle_favorite(&mut self, id: &str) {
        if let Some(mut c) = self.contact(id).cloned() {
            c.favorite = !c.favorite;
            self.save_contact(c);
        }
    }

    /// Every label in use, with how many contacts carry it.
    pub fn label_counts(&self) -> Vec<(String, usize)> {
        let mut map: Vec<(String, usize)> = Vec::new();
        for c in &self.contacts {
            for l in &c.labels {
                match map.iter_mut().find(|(n, _)| n.eq_ignore_ascii_case(l)) {
                    Some(e) => e.1 += 1,
                    None => map.push((l.clone(), 1)),
                }
            }
        }
        map.sort_by_key(|(n, _)| n.to_lowercase());
        map
    }

    pub fn matches_filter(&self, c: &Contact, f: &Filter) -> bool {
        match f {
            Filter::All => true,
            Filter::Favorites => c.favorite,
            Filter::Themed => !c.theme.is_default(),
            Filter::Recent => chrono::DateTime::parse_from_rfc3339(&c.updated).map(|d| chrono::Utc::now().signed_duration_since(d).num_days() < 7).unwrap_or(false),
            Filter::Unlabeled => c.labels.is_empty(),
            Filter::Label(l) => c.labels.iter().any(|x| x.eq_ignore_ascii_case(l)),
            Filter::Group(id) => self.group(id).is_some_and(|g| g.members.iter().any(|m| match m {
                GroupMember::Contact { id } => id == &c.id,
                GroupMember::Number { number } => c.phones.iter().any(|p| contactcraft_store::phone_key(number) == contactcraft_store::phone_key(&p.value) && contactcraft_store::phone_key(number).is_some()),
            })),
        }
    }

    /// The contacts the list shows: the filter, then the search.
    pub fn shown(&self) -> Vec<&Contact> {
        let q = self.search.trim().to_lowercase();
        self.contacts.iter().filter(|c| self.matches_filter(c, &self.filter)).filter(|c| q.is_empty() || c.search_text().contains(&q)).collect()
    }

    pub fn editing(&self) -> bool {
        matches!(self.pane, Pane::Edit(_) | Pane::Theme(_))
    }

    /// Open the editor on a new contact.
    pub fn new_contact(&mut self) {
        self.active_group = None;
        let mut c = Contact::new();
        c.phones.push(contactcraft_store::Labeled::new("mobile", ""));
        c.emails.push(contactcraft_store::Email { label: "home".into(), ..Default::default() });
        if let Filter::Label(l) = &self.filter {
            c.labels.push(l.clone());
        }
        if self.filter == Filter::Favorites {
            c.favorite = true;
        }
        self.backstage = None;
        self.set_pane(Pane::Edit(Box::new(c)));
    }

    /// Switch the right-hand pane, asking first when that would lose unsaved changes.
    pub fn set_pane(&mut self, p: Pane) {
        if self.unsaved() {
            self.confirm_discard = Some(p);
        } else {
            self.pane = p;
        }
    }

    pub fn unsaved(&self) -> bool {
        match &self.pane {
            Pane::Card => false,
            Pane::Edit(c) | Pane::Theme(c) => {
                if c.id.is_empty() {
                    !c.is_blank() || !c.theme.is_default()
                } else {
                    self.contact(&c.id).is_none_or(|orig| orig != c.as_ref())
                }
            }
        }
    }

    pub fn select(&mut self, id: &str) {
        if self.selected.as_deref() == Some(id) && !self.editing() {
            return;
        }
        if self.unsaved() {
            self.confirm_discard = Some(Pane::Card);
            return;
        }
        self.selected = Some(id.to_string());
        self.active_group = None;
        self.pane = Pane::Card;
    }

    // --- pictures ----------------------------------------------------------------------------

    /// A picture file as a texture (cached; `blur` softens it for backgrounds).
    pub fn texture(&mut self, ctx: &egui::Context, path: &Path, blur: bool, repeat: bool) -> Option<egui::TextureId> {
        let key = format!("{}|{blur}|{repeat}", path.display());
        if let Some(t) = self.textures.get(&key) {
            return t.as_ref().map(|t| t.id());
        }
        let img = decode_image(path, if blur { 900 } else { 1600 }).ok().map(|i| if blur { i.blur(8.0) } else { i });
        let handle = img.map(|i| {
            let rgba = i.to_rgba8();
            let ci = ColorImage::from_rgba_unmultiplied([rgba.width() as usize, rgba.height() as usize], rgba.as_raw());
            ctx.load_texture(key.clone(), ci, if repeat { egui::TextureOptions::LINEAR_REPEAT } else { egui::TextureOptions::LINEAR })
        });
        let id = handle.as_ref().map(|h| h.id());
        self.textures.insert(key, handle);
        id
    }

    /// The pixel size of a cached texture.
    pub fn texture_size(&self, ctx: &egui::Context, id: egui::TextureId) -> Option<egui::Vec2> {
        ctx.tex_manager().read().meta(id).map(|m| egui::vec2(m.size[0] as f32, m.size[1] as f32))
    }

    pub fn photo_texture(&mut self, ctx: &egui::Context, c: &Contact) -> Option<egui::TextureId> {
        let path = self.store.as_ref()?.photo_path(c)?;
        self.texture(ctx, &path, false, false)
    }

    /// Drop cached pictures (after a picture changed).
    pub fn forget_textures(&mut self) {
        self.textures.clear();
    }

    // --- fonts -------------------------------------------------------------------------------

    /// The egui family for a contact's theme font, once it's loaded (None means "use the app's
    /// font for now": it's loaded for the next frame).
    pub fn theme_family(&mut self, c: &Contact) -> Option<egui::FontFamily> {
        if c.theme.font_file.is_empty() {
            return None;
        }
        let key = format!("cf:{}", c.theme.font_file.to_lowercase());
        if self.fonts_ready.contains(&key) {
            return Some(egui::FontFamily::Name(key.into()));
        }
        if !self.fonts_bad.contains(&key) {
            if let Some(path) = self.store.as_ref().and_then(|s| s.font_path(c)) {
                self.fonts_wanted.insert(format!("{key}\u{1}{}", path.display()));
            } else {
                self.fonts_bad.insert(key);
            }
        }
        None
    }

    fn load_wanted_fonts(&mut self, ctx: &egui::Context) {
        if self.fonts_wanted.is_empty() {
            return;
        }
        let mut changed = false;
        for w in std::mem::take(&mut self.fonts_wanted) {
            let Some((key, path)) = w.split_once('\u{1}') else { continue };
            match std::fs::read(path) {
                Ok(bytes) if crate::theme::looks_like_font(&bytes) => {
                    // The theme's font first, then the app's own fonts for anything it lacks
                    // (emoji, symbols, other alphabets).
                    self.font_defs.font_data.insert(key.to_string(), std::sync::Arc::new(egui::FontData::from_owned(bytes)));
                    let mut chain = vec![key.to_string()];
                    chain.extend(self.font_defs.families.get(&egui::FontFamily::Proportional).cloned().unwrap_or_default());
                    self.font_defs.families.insert(egui::FontFamily::Name(key.into()), chain);
                    self.fonts_ready.insert(key.to_string());
                    changed = true;
                }
                _ => {
                    self.fonts_bad.insert(key.to_string());
                }
            }
        }
        if changed {
            ctx.set_fonts(self.font_defs.clone());
        }
        ctx.request_repaint();
    }

    pub fn font_is_missing(&self, c: &Contact) -> bool {
        !c.theme.font_file.is_empty() && self.fonts_bad.contains(&format!("cf:{}", c.theme.font_file.to_lowercase()))
    }

    // --- loop --------------------------------------------------------------------------------

    fn keyboard(&mut self, ctx: &egui::Context) {
        let (new, find, esc, del, up, down, enter, save) = ctx.input(|i| {
            (
                i.modifiers.command && i.key_pressed(egui::Key::N),
                i.modifiers.command && (i.key_pressed(egui::Key::F) || i.key_pressed(egui::Key::E)),
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::Delete),
                i.key_pressed(egui::Key::ArrowUp),
                i.key_pressed(egui::Key::ArrowDown),
                i.key_pressed(egui::Key::Enter),
                i.modifiers.command && i.key_pressed(egui::Key::S),
            )
        });
        let typing = ctx.memory(|m| m.focused().is_some());
        if let Some(id) = self.confirm_delete.clone() {
            if enter {
                ctx.input_mut(|i| { i.consume_key(egui::Modifiers::NONE, egui::Key::Enter); });
                self.confirm_delete = None;
                self.delete_contact(&id);
            } else if esc {
                self.confirm_delete = None;
            }
            return;
        }
        if let Some(id) = self.confirm_group_delete.clone() {
            if enter {
                ctx.input_mut(|i| { i.consume_key(egui::Modifiers::NONE, egui::Key::Enter); });
                self.confirm_group_delete = None;
                self.delete_group(&id);
            } else if esc { self.confirm_group_delete = None; }
            return;
        }
        if self.dialog.is_some() {
            if self.dialog.as_ref().is_some_and(|d| d.group.is_some()) {
                if save { self.save_group_dialog(ctx); }
                if esc { self.finish_dialog(ctx, "cancelled", None, None); }
                return;
            }
            if save {
                self.save_pane();
            }
            if esc && self.confirm_discard.is_none() {
                self.set_pane(Pane::Card);
            }
            return;
        }
        if new {
            self.new_contact();
        }
        if find {
            self.backstage = None;
            self.focus_search = true;
        }
        if save {
            self.save_pane();
        }
        if esc {
            if self.backstage.is_some() {
                self.backstage = None;
            } else if self.editing() {
                self.set_pane(Pane::Card);
            }
        }
        if typing || self.editing() || self.backstage.is_some() {
            return;
        }
        if del {
            if let Some(id) = &self.selected {
                self.confirm_delete = Some(id.clone());
            }
        }
        if enter {
            if let Some(c) = self.selected_contact() {
                self.pane = Pane::Edit(Box::new(c));
            }
        }
        if up || down {
            let ids: Vec<String> = self.shown().iter().map(|c| c.id.clone()).collect();
            if ids.is_empty() {
                return;
            }
            let i = self.selected.as_ref().and_then(|s| ids.iter().position(|x| x == s));
            let j = match (i, down) {
                (None, _) => 0,
                (Some(i), true) => (i + 1).min(ids.len() - 1),
                (Some(i), false) => i.saturating_sub(1),
            };
            self.selected = Some(ids[j].clone());
        }
    }

    /// Save what the editor or theme editor holds.
    pub fn save_pane(&mut self) {
        let c = match &self.pane {
            Pane::Edit(c) | Pane::Theme(c) => c.as_ref().clone(),
            Pane::Card => return,
        };
        if c.is_blank() {
            self.notify("Give the contact a name, a company, an email address or a phone number first.", true);
            return;
        }
        let new = c.id.is_empty() || self.contact(&c.id).is_none();
        let name = c.display_name();
        if self.save_contact(c) {
            if let Some(d) = self.dialog.as_mut() {
                d.last_saved = self.selected.clone();
            }
            self.pane = Pane::Card;
            self.notify(if new { format!("Added {name}.") } else { format!("Saved {name}.") }, false);
        }
    }

    fn dropped_files(&mut self, ctx: &egui::Context) {
        let files: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().map(|f| f.path().to_path_buf()).filter(|p| !p.as_os_str().is_empty()).collect());
        for f in files {
            let ext = f.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
            match ext.as_str() {
                "vcf" | "vcard" | "csv" | "json" => self.read_import_file(&f),
                "png" | "jpg" | "jpeg" | "gif" | "webp" => self.drop_picture(&f),
                _ => self.notify("Drop a contacts file (.vcf or .csv) or a picture.", true),
            }
        }
    }

    /// A picture dropped on the window becomes the selected (or edited) contact's photo.
    fn drop_picture(&mut self, path: &Path) {
        let Ok(bytes) = std::fs::read(path) else { return };
        let Some(store) = &self.store else { return };
        match &mut self.pane {
            Pane::Edit(c) => {
                if let Err(e) = store.set_photo(c, &bytes) {
                    self.notify(e, true);
                }
            }
            Pane::Theme(c) => {
                if let Err(e) = store.set_background(c, &bytes) {
                    self.notify(e, true);
                }
            }
            Pane::Card => {
                if let Some(mut c) = self.selected_contact() {
                    match store.set_photo(&mut c, &bytes) {
                        Ok(()) => {
                            self.save_contact(c);
                            self.notify("Picture added.", false);
                        }
                        Err(e) => self.notify(e, true),
                    }
                }
            }
        }
        self.forget_textures();
    }

    fn window_title(&mut self, ctx: &egui::Context) {
        let mut title = "ContactCraft".to_string();
        if let Some(d) = &self.dialog {
            title = crate::dialog::title(&d.request).to_string();
            if let Pane::Edit(c) | Pane::Theme(c) = &self.pane {
                if !c.id.is_empty() && !c.display_name().is_empty() {
                    title = format!("Edit {} - ContactCraft", c.display_name());
                }
            }
        } else if let Some(c) = self.selected_contact() {
            title = format!("{} - ContactCraft", c.display_name());
        }
        if self.demo {
            title.push_str(" (demo)");
        }
        if title != self.last_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.last_title = title;
        }
    }
}

pub fn open_path(path: &Path) {
    let _ = std::process::Command::new("explorer").arg(path).spawn();
}

/// Open a web address or mailto: link in the default program.
pub fn open_url(url: &str) {
    let _ = std::process::Command::new("rundll32").args(["url.dll,FileProtocolHandler", url]).spawn();
}

pub fn decode_image(path: &Path, max: u32) -> Result<image::DynamicImage, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let img = image::load_from_memory(&bytes).map_err(|e| e.to_string())?;
    Ok(if img.width() > max || img.height() > max { img.thumbnail(max, max) } else { img })
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.t = Tokens::for_dark(ui.visuals().dark_mode);
        self.poll_store();
        self.update_events();
        self.keyboard(&ctx);
        self.dropped_files(&ctx);
        if self.notice.as_ref().is_some_and(|n| n.until < Instant::now()) {
            self.notice = None;
        }
        self.window_title(&ctx);
        self.draw(ui);
        self.load_wanted_fonts(&ctx);
        // Notice another app's changes within a couple of seconds.
        ctx.request_repaint_after(Duration::from_millis(1500));
    }

    fn on_exit(&mut self) {
        // The dialog window closed with its X: that's a cancel.
        if let Some(d) = self.dialog.as_ref().filter(|d| !d.finished) {
            match d.error.clone() {
                Some(e) => self.write_result("error", None, Some(e)),
                None => self.write_result("cancelled", None, None),
            }
        }
        self.save_settings();
        if self.demo {
            if let Some(s) = self.store.take() {
                let dir = s.dir().to_path_buf();
                drop(s);
                let _ = std::fs::remove_dir_all(dir);
            }
        }
    }
}
