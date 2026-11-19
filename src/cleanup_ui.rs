//! Clean Up: tidy names, phone numbers and addresses, remove repeated emails and numbers, and
//! find duplicate contacts. Exact duplicates are removed quietly; the others are shown one group
//! at a time in the merge screen, where the person keeps one (choosing which entry wins each
//! field they disagree on) or keeps them all. A copy of every contact (and picture) is saved
//! first, so a clean-up can be undone.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use contactsolutions_store::Contact;
use contactsolutions_store::cleanup::{self, Options};
use egui::{Rect, ScrollArea, Sense, Stroke, StrokeKind, Ui, pos2, vec2};

use crate::app::App;
use crate::theme;
use crate::widgets;

pub struct CleanupDialog {
    pub opts: Options,
    pub find_duplicates: bool,
    pub only_shown: bool,
}

impl Default for CleanupDialog {
    fn default() -> Self {
        CleanupDialog { opts: Options::default(), find_duplicates: true, only_shown: false }
    }
}

/// Duplicate groups waiting for the person, one at a time.
pub struct MergeSession {
    pub groups: Vec<(Vec<String>, bool, bool)>,
    pub index: usize,
    pub winner: usize,
    pub picks: HashMap<String, usize>,
    prepared: Option<usize>,
    pub merged: usize,
    pub kept: usize,
}

const NOT_DUPES: &str = "contactsolutions.notDuplicates";

enum MergeAction {
    Merge,
    KeepAll,
    Skip,
    Stop,
}

impl App {
    pub fn open_cleanup(&mut self) {
        if self.store.is_some() && !self.contacts.is_empty() {
            self.backstage = None;
            self.cleanup_dialog = Some(CleanupDialog::default());
        }
    }

    fn not_duplicates(&self) -> HashSet<(String, String)> {
        self.store
            .as_ref()
            .and_then(|s| s.meta(NOT_DUPES))
            .and_then(|v| serde_json::from_str::<Vec<(String, String)>>(&v).ok())
            .unwrap_or_default()
            .into_iter()
            .collect()
    }

    fn remember_not_duplicates(&self, ids: &[String]) {
        let Some(store) = &self.store else { return };
        let mut set = self.not_duplicates();
        for (i, a) in ids.iter().enumerate() {
            for b in &ids[i + 1..] {
                set.insert((a.clone(), b.clone()));
            }
        }
        let mut list: Vec<(String, String)> = set.into_iter().collect();
        list.sort();
        let _ = store.set_meta(NOT_DUPES, Some(&serde_json::to_string(&list).unwrap_or_default()));
    }

    fn backups_dir(&self) -> Option<PathBuf> {
        Some(self.store.as_ref()?.dir().join("backups"))
    }

    /// Save every contact and picture before changing anything.
    fn backup(&self) -> Result<PathBuf, String> {
        let store = self.store.as_ref().ok_or("The contact store isn't open.")?;
        let dir = self.backups_dir().unwrap().join(format!("before-cleanup-{}", chrono::Local::now().format("%Y%m%d-%H%M%S")));
        std::fs::create_dir_all(dir.join("photos")).map_err(|e| e.to_string())?;
        let json = serde_json::to_string_pretty(&serde_json::json!({"format": "contactsolutions", "version": 1, "contacts": self.contacts})).map_err(|e| e.to_string())?;
        std::fs::write(dir.join("contacts.json"), json).map_err(|e| e.to_string())?;
        for c in &self.contacts {
            if let Some(p) = store.photo_path(c) {
                let _ = std::fs::copy(&p, dir.join("photos").join(&c.photo));
            }
        }
        Ok(dir)
    }

    /// The newest clean-up backup, if any.
    pub fn last_backup(&self) -> Option<PathBuf> {
        let dir = self.backups_dir()?;
        let mut v: Vec<PathBuf> = std::fs::read_dir(dir).ok()?.flatten().map(|e| e.path()).filter(|p| p.join("contacts.json").is_file()).collect();
        v.sort();
        v.pop()
    }

    /// Put every contact back as it was before the last clean-up (contacts added since stay).
    pub fn undo_cleanup(&mut self, dir: &Path) {
        let Some(store) = &self.store else { return };
        let Ok(text) = std::fs::read_to_string(dir.join("contacts.json")) else {
            self.notify("That backup couldn't be read.", true);
            return;
        };
        let list: Vec<Contact> = serde_json::from_str::<serde_json::Value>(&text).ok().and_then(|v| serde_json::from_value(v.get("contacts").cloned()?).ok()).unwrap_or_default();
        let mut n = 0;
        for mut c in list {
            if !c.photo.is_empty() {
                let dest = store.photos_dir().join(&c.photo);
                if !dest.is_file() {
                    let _ = std::fs::copy(dir.join("photos").join(&c.photo), &dest);
                }
            }
            if store.save(&mut c).is_ok() {
                n += 1;
            }
        }
        let done = dir.with_file_name(format!("{}-undone", dir.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()));
        let _ = std::fs::rename(dir.join("contacts.json"), dir.join("contacts.restored.json"));
        let _ = std::fs::rename(dir, done);
        self.forget_textures();
        self.reload();
        self.notify(format!("Put {n} contacts back as they were before the clean-up."), false);
    }

    pub fn run_cleanup(&mut self, dlg: CleanupDialog) {
        if self.store.is_none() {
            return;
        }
        if let Err(e) = self.backup() {
            self.notify(format!("Nothing was changed: the backup couldn't be saved ({e})."), true);
            return;
        }
        let scope: Vec<Contact> = if dlg.only_shown { self.shown().into_iter().cloned().collect() } else { self.contacts.clone() };
        let scope_ids: HashSet<String> = scope.iter().map(|c| c.id.clone()).collect();
        let (mut names, mut phones, mut addrs, mut emails_gone, mut phones_gone, mut touched) = (0, 0, 0, 0, 0, 0);
        {
            let store = self.store.as_ref().unwrap();
            for c in &scope {
                let mut n = c.clone();
                let ch = cleanup::clean(&mut n, &dlg.opts);
                if ch.any() && n != *c && store.save(&mut n).is_ok() {
                    touched += 1;
                    names += usize::from(ch.names);
                    phones += ch.phones;
                    addrs += ch.addresses;
                    emails_gone += ch.emails_removed;
                    phones_gone += ch.phones_removed;
                }
            }
        }
        self.reload();
        let mut exact = 0;
        let mut review: Vec<(Vec<String>, bool, bool)> = Vec::new();
        if dlg.find_duplicates {
            let list: Vec<Contact> = self.contacts.iter().filter(|c| scope_ids.contains(&c.id)).cloned().collect();
            let groups = cleanup::duplicate_groups(&list, &self.not_duplicates());
            let store = self.store.as_ref().unwrap();
            for g in groups {
                let cs: Vec<Contact> = g.members.iter().map(|&i| list[i].clone()).collect();
                if cleanup::identical(&cs) {
                    // The same entry more than once: keep the oldest, remove the rest.
                    let keep = cs.iter().enumerate().min_by_key(|(_, c)| c.created.clone()).map(|(i, _)| i).unwrap_or(0);
                    for (i, c) in cs.iter().enumerate() {
                        if i != keep && store.delete(&c.id).is_ok() {
                            exact += 1;
                        }
                    }
                } else {
                    review.push((cs.iter().map(|c| c.id.clone()).collect(), g.same_name, g.same_number));
                }
            }
            self.reload();
        }
        let mut parts = Vec::new();
        if names > 0 {
            parts.push(format!("tidied {names} name{}", plural(names)));
        }
        if phones > 0 {
            parts.push(format!("reformatted {phones} phone number{}", plural(phones)));
        }
        if addrs > 0 {
            parts.push(format!("reformatted {addrs} address{}", if addrs == 1 { "" } else { "es" }));
        }
        if emails_gone > 0 {
            parts.push(format!("removed {emails_gone} repeated email{}", plural(emails_gone)));
        }
        if phones_gone > 0 {
            parts.push(format!("removed {phones_gone} repeated number{}", plural(phones_gone)));
        }
        if exact > 0 {
            parts.push(format!("removed {exact} exact duplicate contact{}", plural(exact)));
        }
        let mut msg = if parts.is_empty() { "Everything was already tidy.".to_string() } else { format!("Clean-up {} ({touched} contacts changed).", parts.join(", ")) };
        if !review.is_empty() {
            msg.push_str(&format!(" {} possible duplicate{} to look at.", review.len(), plural(review.len())));
            self.merge = Some(MergeSession { groups: review, index: 0, winner: 0, picks: HashMap::new(), prepared: None, merged: 0, kept: 0 });
        }
        self.notify(msg, false);
    }

    pub fn cleanup_dialog(&mut self, ctx: &egui::Context) {
        let t = self.t;
        let shown_n = self.shown().len();
        let total = self.contacts.len();
        let backup = self.last_backup();
        let Some(dlg) = self.cleanup_dialog.as_mut() else { return };
        let mut go = false;
        let mut close = false;
        let mut undo = false;
        let modal = egui::Modal::new(egui::Id::unique("cleanup")).frame(crate::chrome::dialog_frame(&t)).show(ctx, |ui| {
            ui.set_width(520.0);
            ui.label(egui::RichText::new("Clean up contacts").font(theme::semibold(18.0)).color(t.text));
            ui.label(egui::RichText::new("Choose what to tidy. A copy of every contact is saved first, so you can undo it.").font(theme::regular(12.5)).color(t.text_dim));
            ui.add_space(8.0);
            ui.spacing_mut().item_spacing.y = 6.0;
            let o = &mut dlg.opts;
            check(ui, &t, &mut o.proper_case, "Names in proper case", "JOHN o'brien → John O'Brien. Names already written in mixed case (DeShawn) are kept.");
            check(ui, &t, &mut o.strip_symbols, "Remove special characters from names", "Symbols, numbers and emoji go; letters, accents, hyphens and apostrophes stay.");
            check(ui, &t, &mut o.phones, "Phone numbers in one standard format", "5550102030 → (555) 010-2030, with +1 or other country codes kept.");
            check(ui, &t, &mut o.addresses, "Addresses in one standard format", "123 MAIN STREET, illinois → 123 Main St, IL; ZIP+4 and country names tidied.");
            check(ui, &t, &mut o.dedupe_emails, "Remove repeated email addresses on a contact", "");
            check(ui, &t, &mut o.dedupe_phones, "Remove repeated phone numbers on a contact", "The same number written two ways counts as one.");
            check(ui, &t, &mut dlg.find_duplicates, "Find duplicate contacts", "Same name or same number. Exact copies are removed; the rest are shown so you can merge them.");
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.radio_value(&mut dlg.only_shown, false, format!("All contacts ({total})"));
                ui.radio_value(&mut dlg.only_shown, true, format!("Only the contacts shown ({shown_n})"));
            });
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                if widgets::primary_button(ui, &t, "Clean up", true).clicked() {
                    go = true;
                }
                if widgets::secondary_button(ui, &t, "Cancel").clicked() {
                    close = true;
                }
                if backup.is_some() && widgets::secondary_button(ui, &t, "Undo the last clean-up").on_hover_text("Puts every contact back as it was before the last clean-up; contacts added since stay.").clicked() {
                    undo = true;
                }
            });
        });
        if go {
            if let Some(d) = self.cleanup_dialog.take() {
                self.run_cleanup(d);
            }
        } else if undo {
            self.cleanup_dialog = None;
            if let Some(b) = backup {
                self.undo_cleanup(&b);
            }
        } else if close || modal.should_close() {
            self.cleanup_dialog = None;
        }
    }

    pub fn merge_dialog(&mut self, ctx: &egui::Context) {
        let t = self.t;
        let Some(m) = self.merge.as_mut() else { return };
        // Skip groups that no longer have two contacts (deleted or merged meanwhile).
        let contacts = &self.contacts;
        let current = loop {
            if m.index >= m.groups.len() {
                break None;
            }
            let cs: Vec<Contact> = m.groups[m.index].0.iter().filter_map(|id| contacts.iter().find(|c| &c.id == id).cloned()).collect();
            if cs.len() >= 2 {
                break Some(cs);
            }
            m.index += 1;
        };
        let Some(cs) = current else {
            let (merged, kept) = (m.merged, m.kept);
            self.merge = None;
            if merged + kept > 0 {
                self.notify(format!("Duplicates done: merged {merged}, kept {kept} as different people."), false);
            }
            return;
        };
        if m.prepared != Some(m.index) {
            // The most complete entry wins by default (the oldest when they're even).
            m.winner = (0..cs.len()).max_by_key(|&i| (cleanup::completeness(&cs[i]), std::cmp::Reverse(cs[i].created.clone()))).unwrap_or(0);
            m.picks.clear();
            m.prepared = Some(m.index);
        }
        let conflicts = cleanup::conflicts(&cs);
        let (_, same_name, same_number) = m.groups[m.index].clone();
        let (index, total) = (m.index, m.groups.len());
        let photos: Vec<Option<egui::TextureId>> = {
            let cs2 = cs.clone();
            cs2.iter().map(|c| self.photo_texture(ctx, c)).collect()
        };
        let Some(m) = self.merge.as_mut() else { return };
        let mut action = None;
        let screen = ctx.content_rect();
        let width = (screen.width() - 80.0).clamp(560.0, 1040.0);
        egui::Modal::new(egui::Id::unique("merge")).frame(crate::chrome::dialog_frame(&t)).show(ctx, |ui| {
            ui.set_width(width);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("Possible duplicate {} of {total}", index + 1)).font(theme::semibold(18.0)).color(t.text));
                ui.add_space(8.0);
                let why = match (same_name, same_number) {
                    (true, true) => "Same name and phone number",
                    (true, false) => "Same name",
                    _ => "Same phone number",
                };
                ui.label(egui::RichText::new(why).font(theme::regular(12.5)).color(t.text_dim));
            });
            ui.label(egui::RichText::new("Keep one (the entries are merged into it) or keep them all if they're different people.").font(theme::regular(12.5)).color(t.text_dim));
            ui.add_space(10.0);

            // The entries side by side.
            let n = cs.len();
            let card_w = ((width - 10.0 * (n as f32 - 1.0)) / n as f32).max(220.0);
            ScrollArea::horizontal().id_salt("merge_cards").max_height(250.0).show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    for (i, c) in cs.iter().enumerate() {
                        let selected = m.winner == i;
                        let (r, resp) = ui.allocate_exact_size(vec2(card_w, 232.0), Sense::click());
                        if resp.clicked() {
                            m.winner = i;
                        }
                        let p = ui.painter();
                        p.rect_filled(r, 8.0, if selected { t.selection } else { t.surface });
                        p.rect_stroke(r, 8.0, Stroke::new(if selected { 2.0 } else { 1.0 }, if selected { t.accent } else { t.border }), StrokeKind::Inside);
                        widgets::contact_avatar(p, c, pos2(r.min.x + 30.0, r.min.y + 32.0), 18.0, photos[i]);
                        let tx = r.min.x + 58.0;
                        let tw = r.max.x - tx - 10.0;
                        widgets::clipped(p, pos2(tx, r.min.y + 24.0), tw, &c.display_name(), theme::semibold(14.0), t.text);
                        let role = [c.job_title(), c.company()].iter().filter(|s| !s.is_empty()).copied().collect::<Vec<_>>().join(", ");
                        widgets::clipped(p, pos2(tx, r.min.y + 42.0), tw, &role, theme::regular(11.5), t.text_dim);
                        let mut y = r.min.y + 70.0;
                        let mut line = |text: String| {
                            if y < r.max.y - 40.0 {
                                widgets::clipped(p, pos2(r.min.x + 14.0, y), r.width() - 28.0, &text, theme::regular(12.0), t.text);
                                y += 18.0;
                            }
                        };
                        for ph in c.phones.iter().take(3) {
                            line(format!("{}  {}", ph.label, ph.value));
                        }
                        for e in c.emails.iter().take(3) {
                            line(e.address.clone());
                        }
                        if let Some(a) = c.addresses.first() {
                            line(a.lines().join(", "));
                        }
                        if !c.birthday.is_empty() {
                            line(format!("Birthday {}", c.birthday));
                        }
                        let added = chrono::DateTime::parse_from_rfc3339(&c.created).map(|d| d.with_timezone(&chrono::Local).format("%b %e, %Y").to_string()).unwrap_or_default();
                        widgets::clipped(p, pos2(r.min.x + 14.0, r.max.y - 52.0), r.width() - 28.0, &format!("Added {added}  ·  {} details", cleanup::completeness(c)), theme::regular(11.0), t.text_dim);
                        // "Keep this one" radio.
                        let rr = Rect::from_min_size(pos2(r.min.x + 12.0, r.max.y - 34.0), vec2(r.width() - 24.0, 24.0));
                        p.circle_stroke(pos2(rr.min.x + 9.0, rr.center().y), 7.0, Stroke::new(1.5, if selected { t.accent } else { t.border_strong }));
                        if selected {
                            p.circle_filled(pos2(rr.min.x + 9.0, rr.center().y), 4.0, t.accent);
                        }
                        p.text(pos2(rr.min.x + 24.0, rr.center().y), egui::Align2::LEFT_CENTER, if selected { "Keep this one (main entry)" } else { "Keep this one" }, if selected { theme::semibold(12.5) } else { theme::regular(12.5) }, t.text);
                    }
                });
            });

            ui.add_space(10.0);
            ui.label(egui::RichText::new("Where they differ").font(theme::semibold(14.0)).color(t.text));
            if conflicts.is_empty() {
                ui.label(egui::RichText::new("Nothing conflicts. Merging combines what each entry has.").font(theme::regular(12.5)).color(t.text_dim));
            } else {
                ui.label(egui::RichText::new("Choose which entry's value wins for each:").font(theme::regular(12.5)).color(t.text_dim));
            }
            ScrollArea::vertical().id_salt("merge_conflicts").max_height((screen.height() - 520.0).max(140.0)).show(ui, |ui| {
                egui::Grid::new("merge_grid").num_columns(n + 1).spacing(vec2(12.0, 6.0)).striped(true).show(ui, |ui| {
                    for cf in &conflicts {
                        ui.label(egui::RichText::new(&cf.label).font(theme::regular(12.5)).color(t.text_dim));
                        let default = if !cf.values[m.winner].is_empty() { m.winner } else { cf.values.iter().position(|v| !v.is_empty()).unwrap_or(m.winner) };
                        let chosen = m.picks.get(&cf.path).copied().unwrap_or(default);
                        for (i, v) in cf.values.iter().enumerate() {
                            if v.is_empty() {
                                ui.label(egui::RichText::new("—").color(t.text_disabled));
                                continue;
                            }
                            let shown = if cf.path == "photo" || cf.path == "theme.backgroundImage" { "This entry's picture".to_string() } else { v.chars().take(70).collect::<String>() };
                            if ui.radio(chosen == i, shown).on_hover_text(v).clicked() {
                                m.picks.insert(cf.path.clone(), i);
                            }
                        }
                        ui.end_row();
                    }
                });
            });
            ui.add_space(6.0);
            ui.label(egui::RichText::new("Phone numbers, emails, addresses, labels and every other list are combined from all entries, without repeats.").font(theme::regular(12.0)).color(t.text_dim));
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;
                if widgets::primary_button(ui, &t, "Merge into one", true).clicked() {
                    action = Some(MergeAction::Merge);
                }
                if widgets::secondary_button(ui, &t, "Keep all (different people)").clicked() {
                    action = Some(MergeAction::KeepAll);
                }
                if widgets::secondary_button(ui, &t, "Skip for now").clicked() {
                    action = Some(MergeAction::Skip);
                }
                if widgets::secondary_button(ui, &t, "Stop").clicked() {
                    action = Some(MergeAction::Stop);
                }
            });
        });
        let winner = m.winner;
        let picks = m.picks.clone();
        match action {
            Some(MergeAction::Merge) => {
                if let Some(id) = self.apply_merge(&cs, winner, &picks) {
                    self.selected = Some(id);
                }
                if let Some(m) = self.merge.as_mut() {
                    m.merged += 1;
                    m.index += 1;
                }
            }
            Some(MergeAction::KeepAll) => {
                let ids: Vec<String> = cs.iter().map(|c| c.id.clone()).collect();
                self.remember_not_duplicates(&ids);
                if let Some(m) = self.merge.as_mut() {
                    m.kept += 1;
                    m.index += 1;
                }
            }
            Some(MergeAction::Skip) => {
                if let Some(m) = self.merge.as_mut() {
                    m.index += 1;
                }
            }
            Some(MergeAction::Stop) => {
                if let Some(m) = self.merge.take() {
                    self.notify(format!("Merged {}, kept {} as different people. Clean Up finds the rest again later.", m.merged, m.kept), false);
                }
            }
            None => {}
        }
    }

    /// Merge the entries into the winner and delete the others. Returns the merged contact's id.
    fn apply_merge(&mut self, cs: &[Contact], winner: usize, picks: &HashMap<String, usize>) -> Option<String> {
        let store = self.store.as_ref()?;
        let merged = cleanup::merge(cs, winner, picks);
        let mut c = merged.contact;
        let mut photo_error = None;
        if let Some(i) = merged.photo_from.filter(|&i| i != winner) {
            if let Some(bytes) = store.photo_path(&cs[i]).and_then(|p| std::fs::read(p).ok()) {
                c.photo = cs[winner].photo.clone();
                if let Err(e) = store.set_photo(&mut c, &bytes) {
                    photo_error = Some(e);
                }
            }
        }
        if let Err(e) = store.save(&mut c) {
            self.notify(format!("Couldn't merge: {e}"), true);
            return None;
        }
        for (i, other) in cs.iter().enumerate() {
            if i != winner {
                let _ = store.delete(&other.id);
            }
        }
        if let Some(e) = photo_error {
            self.notify(format!("Merged, but the picture couldn't be carried over: {e}"), true);
        }
        self.forget_textures();
        self.reload();
        Some(c.id)
    }
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

fn check(ui: &mut Ui, t: &theme::Tokens, value: &mut bool, label: &str, hint: &str) {
    ui.horizontal_top(|ui| {
        ui.checkbox(value, "");
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            ui.label(egui::RichText::new(label).font(theme::regular(13.5)).color(t.text));
            if !hint.is_empty() {
                ui.add(egui::Label::new(egui::RichText::new(hint).font(theme::regular(11.5)).color(t.text_dim)).wrap());
            }
        });
    });
}
