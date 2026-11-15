//! Bringing contacts in from files (Google Contacts' CSV, Outlook's CSV, vCard from anywhere,
//! ContactCraft's own JSON) and writing them out again.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use contactcraft_store::{Contact, ContactGroup, GroupMember, csv, vcard};

use crate::app::{App, ImportPreview};
use crate::theme;
use crate::widgets;

/// What a contacts file holds, read by its contents (the extension is only a hint).
pub struct ImportData {
    pub contacts: Vec<(Contact, Option<Vec<u8>>)>,
    pub groups: Vec<ContactGroup>,
}

pub fn read_contacts_file(path: &Path) -> Result<ImportData, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let text = csv::decode_text(&bytes);
    let head = text.trim_start();
    if head.to_uppercase().starts_with("BEGIN:VCARD") {
        return Ok(ImportData { contacts: vcard::import(&text), groups: Vec::new() });
    }
    if head.starts_with('[') || head.starts_with('{') {
        let v: serde_json::Value = serde_json::from_str(head).map_err(|e| format!("That JSON file couldn't be read: {e}"))?;
        let list = v.get("contacts").cloned().unwrap_or_else(|| v.clone());
        let items: Vec<Contact> = serde_json::from_value(list).map_err(|e| format!("That file isn't a ContactCraft export: {e}"))?;
        let groups: Vec<ContactGroup> = v.get("groups").cloned().map(serde_json::from_value).transpose().map_err(|e| format!("The groups in that file couldn't be read: {e}"))?.unwrap_or_default();
        return Ok(ImportData { contacts: items
            .into_iter()
            .map(|mut c| {
                c.photo.clear();
                c.google = None;
                (c, None)
            })
            .collect(), groups });
    }
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    if ext == "csv" || head.contains(',') {
        let items = csv::import(&text);
        if items.is_empty() {
            return Err("No contacts were found in that file. Export from Google Contacts as \"Google CSV\" or \"vCard\", or from Outlook as a CSV file.".into());
        }
        return Ok(ImportData { contacts: items.into_iter().map(|(c, _photo_url)| (c, None)).collect(), groups: Vec::new() });
    }
    Err("That doesn't look like a contacts file (vCard .vcf or CSV).".into())
}

impl App {
    pub fn pick_import_file(&mut self) {
        if self.store.is_none() {
            return;
        }
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Import contacts")
            .add_filter("Contacts files", &["vcf", "vcard", "csv", "json"])
            .add_filter("vCard (Google, Apple, Outlook)", &["vcf", "vcard"])
            .add_filter("CSV (Google CSV, Outlook CSV)", &["csv"])
            .add_filter("ContactCraft export", &["json"])
            .pick_file()
        {
            self.read_import_file(&path);
        }
    }

    pub fn read_import_file(&mut self, path: &Path) {
        match read_contacts_file(path) {
            Ok(data) if data.contacts.is_empty() && data.groups.is_empty() => self.notify("No contacts or groups were found in that file.", true),
            Ok(data) => {
                self.backstage = None;
                self.import = Some(ImportPreview { file: path.to_path_buf(), items: data.contacts, groups: data.groups, skip_duplicates: true, label: String::new() });
            }
            Err(e) => self.notify(e, true),
        }
    }

    /// Whether a contact like this one (same email, phone, or name) is already here.
    fn is_duplicate(&self, c: &Contact, emails: &HashSet<String>, phones: &HashSet<String>, names: &HashSet<String>) -> bool {
        c.email_keys().iter().any(|e| emails.contains(e)) || c.phone_keys().iter().any(|p| phones.contains(p)) || (!c.display_name().is_empty() && names.contains(&c.display_name().to_lowercase()))
    }

    fn run_import(&mut self) {
        let Some(preview) = self.import.take() else { return };
        let Some(store) = &self.store else { return };
        let mut emails: HashSet<String> = self.contacts.iter().flat_map(|c| c.email_keys()).collect();
        let mut phones: HashSet<String> = self.contacts.iter().flat_map(|c| c.phone_keys()).collect();
        let mut names: HashSet<String> = self.contacts.iter().map(|c| c.display_name().to_lowercase()).collect();
        let (mut added, mut skipped, mut failed) = (0, 0, 0);
        let mut id_map: HashMap<String, String> = HashMap::new();
        let label = preview.label.trim().to_string();
        for (mut c, photo) in preview.items {
            let old_id = std::mem::take(&mut c.id);
            if preview.skip_duplicates && self.is_duplicate(&c, &emails, &phones, &names) {
                if !old_id.is_empty() {
                    let matches = |x: &Contact| {
                        c.email_keys().iter().any(|e| x.email_keys().contains(e)) ||
                        c.phone_keys().iter().any(|p| x.phone_keys().contains(p)) ||
                        (!c.display_name().is_empty() && x.display_name().eq_ignore_ascii_case(&c.display_name()))
                    };
                    let existing_id = self.contacts.iter().find(|x| matches(x)).map(|x| x.id.clone())
                        .or_else(|| store.list().ok()?.into_iter().find(matches).map(|x| x.id));
                    if let Some(id) = existing_id { id_map.insert(old_id, id); }
                }
                skipped += 1;
                continue;
            }
            if !label.is_empty() && !c.labels.iter().any(|l| l.eq_ignore_ascii_case(&label)) {
                c.labels.push(label.clone());
            }
            if let Some(bytes) = photo {
                let bytes = crate::editor::shrink_picture(&bytes).unwrap_or(bytes);
                let _ = store.set_photo(&mut c, &bytes);
            }
            match store.save(&mut c) {
                Ok(()) => {
                    if !old_id.is_empty() { id_map.insert(old_id, c.id.clone()); }
                    added += 1;
                    emails.extend(c.email_keys());
                    phones.extend(c.phone_keys());
                    names.insert(c.display_name().to_lowercase());
                }
                Err(_) => failed += 1,
            }
        }
        let mut groups_added = 0;
        for mut group in preview.groups {
            group.id.clear();
            group.members.retain_mut(|member| match member {
                GroupMember::Contact { id } => {
                    if let Some(new_id) = id_map.get(id) { *id = new_id.clone(); true }
                    else { store.get(id).ok().flatten().is_some() }
                }
                GroupMember::Number { .. } => true,
            });
            match store.save_group(&mut group) {
                Ok(()) => groups_added += 1,
                Err(_) => failed += 1,
            }
        }
        self.reload();
        let mut msg = format!("Imported {added} contact{}.", if added == 1 { "" } else { "s" });
        if groups_added > 0 { msg.push_str(&format!(" Imported {groups_added} group{}.", if groups_added == 1 { "" } else { "s" })); }
        if skipped > 0 {
            msg.push_str(&format!(" Skipped {skipped} you already had."));
        }
        if failed > 0 {
            msg.push_str(&format!(" {failed} couldn't be saved."));
        }
        if !label.is_empty() {
            self.filter = crate::app::Filter::Label(label);
        }
        self.notify(msg, failed > 0);
    }

    pub fn import_dialog(&mut self, ctx: &egui::Context) {
        let t = self.t;
        let Some(preview) = self.import.as_mut() else { return };
        let mut go = false;
        let mut close = false;
        let file = preview.file.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let total = preview.items.len();
        let group_total = preview.groups.len();
        let with_photo = preview.items.iter().filter(|(_, p)| p.is_some()).count();
        let sample: Vec<String> = preview.items.iter().take(6).map(|(c, _)| c.display_name()).collect();
        let modal = egui::Modal::new(egui::Id::unique("import")).frame(crate::chrome::dialog_frame(&t)).show(ctx, |ui| {
            ui.set_width(480.0);
            ui.label(egui::RichText::new(format!("Import {total} contact{} from {file}?", if total == 1 { "" } else { "s" })).font(theme::semibold(17.0)).color(t.text));
            if group_total > 0 { ui.label(format!("Also includes {group_total} group{}.", if group_total == 1 { "" } else { "s" })); }
            ui.add_space(4.0);
            let mut s = sample.join(", ");
            if total > sample.len() {
                s.push_str(&format!(" and {} more", total - sample.len()));
            }
            ui.add(egui::Label::new(egui::RichText::new(s).font(theme::regular(12.5)).color(t.text_dim)).wrap());
            if with_photo > 0 {
                ui.label(egui::RichText::new(format!("{with_photo} come with a picture.")).font(theme::regular(12.5)).color(t.text_dim));
            }
            ui.add_space(10.0);
            ui.checkbox(&mut preview.skip_duplicates, "Skip contacts I already have (same email, phone or name)");
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Give them a label").font(theme::regular(13.0)).color(t.text));
                ui.add(egui::TextEdit::singleline(&mut preview.label).hint_text("Optional, e.g. From Google").desired_width(220.0).margin(egui::Margin::symmetric(8, 5)));
            });
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if widgets::primary_button(ui, &t, "Import", true).clicked() {
                    go = true;
                }
                if widgets::secondary_button(ui, &t, "Cancel").clicked() {
                    close = true;
                }
            });
        });
        if go {
            self.run_import();
        } else if close || modal.should_close() {
            self.import = None;
        }
    }

    /// Write contacts to a file the person picks. `kind`: vcf, csv or json.
    pub fn export(&mut self, kind: &str, only_shown: bool) {
        let list: Vec<Contact> = if only_shown { self.shown().into_iter().cloned().collect() } else { self.contacts.clone() };
        if list.is_empty() && (kind != "json" || only_shown || self.groups.is_empty()) {
            self.notify("There are no contacts to export.", true);
            return;
        }
        let (filter, ext, default) = match kind {
            "csv" => ("Google CSV", "csv", "contacts-google.csv"),
            "json" => ("ContactCraft (everything)", "json", "contacts-contactcraft.json"),
            _ => ("vCard", "vcf", "contacts.vcf"),
        };
        let Some(path) = rfd::FileDialog::new().set_title("Export contacts").add_filter(filter, &[ext]).set_file_name(default).save_file() else { return };
        let text = match kind {
            "csv" => csv::export_google(&list),
            "json" => serde_json::to_string_pretty(&serde_json::json!({"format": "contactcraft", "version": 2, "contacts": list, "groups": if only_shown { Vec::new() } else { self.groups.clone() }})).unwrap_or_default(),
            _ => {
                let store = self.store.as_ref();
                vcard::export(&list, |c| store.and_then(|s| s.photo_path(c)).and_then(|p| std::fs::read(p).ok()))
            }
        };
        match std::fs::write(&path, text) {
            Ok(()) => self.notify(format!("Exported {} contacts{} to {}.", list.len(), if kind == "json" && !only_shown { format!(" and {} groups", self.groups.len()) } else { String::new() }, path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()), false),
            Err(e) => self.notify(format!("Couldn't write the file: {e}"), true),
        }
    }
}

#[allow(dead_code)]
pub fn exe_dir() -> PathBuf {
    std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::read_contacts_file;
    use contactcraft_store::GroupMember;

    #[test]
    fn reads_groups_from_contactcraft_json() {
        let path = std::env::temp_dir().join(format!("contactcraft-import-groups-{}.json", std::process::id()));
        std::fs::write(&path, r#"{"format":"contactcraft","version":2,"contacts":[],"groups":[{"id":"g1","name":"Team","members":[{"kind":"number","number":"555-0101"}]}]}"#).unwrap();
        let data = read_contacts_file(&path).unwrap();
        assert!(data.contacts.is_empty());
        assert_eq!(data.groups.len(), 1);
        assert_eq!(data.groups[0].members, vec![GroupMember::Number { number: "555-0101".into() }]);
        let _ = std::fs::remove_file(path);
    }
}
