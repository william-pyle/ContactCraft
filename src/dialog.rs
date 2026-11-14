//! Dialog mode: other apps start ContactCraft to create or edit one contact.
//!
//! ```text
//! ContactCraft.exe --new-contact [--name "Anna Smith"] [--given Anna] [--family Smith]
//!                  [--phone "+1 555 010 2030"] [--email anna@example.com] [--company Northwind]
//!                  [--label Friends] [--notes "..."]
//! ContactCraft.exe --edit-contact <id>
//! ContactCraft.exe --edit-phone <number>      (edits the contact with that number, or starts a
//! ContactCraft.exe --edit-email <address>      new one with it filled in)
//! ... [--result-file <path>] [--theme-tab]
//! ```
//!
//! A small window that stays on top shows only the editor. Save stores the contact and closes;
//! Cancel or closing the window closes without saving. The outcome is given three ways:
//! the exit code (0 saved, 1 cancelled, 2 not found or failed), one line on standard output
//! (`saved <id>`, `cancelled`, `error <message>`), and, with `--result-file`, a JSON file
//! `{"status": "saved"|"cancelled"|"error", "id": "...", "displayName": "...", "message": "..."}`.

use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI32, Ordering};

use contactcraft_store::{Contact, Email, Labeled};
use egui::{Frame, Margin, Ui};

use crate::app::{App, Pane};
use crate::theme;
use crate::widgets;

/// The process's exit code, set when the dialog finishes.
pub static EXIT_CODE: AtomicI32 = AtomicI32::new(1);

#[derive(Clone, Debug, Default)]
pub struct Prefill {
    pub given: String,
    pub family: String,
    pub phone: String,
    pub email: String,
    pub company: String,
    pub label: String,
    pub notes: String,
}

#[derive(Clone, Debug)]
pub enum Target {
    New(Prefill),
    Edit(String),
    ByPhone(String, Prefill),
    ByEmail(String, Prefill),
}

#[derive(Clone, Debug)]
pub struct DialogRequest {
    pub target: Target,
    pub result_file: Option<PathBuf>,
    pub theme_tab: bool,
}

pub struct DialogState {
    pub request: DialogRequest,
    pub error: Option<String>,
    pub finished: bool,
    pub last_saved: Option<String>,
    /// The person read the error and clicked Close.
    pub acknowledged: bool,
}

fn value(args: &[String], flag: &str) -> Option<String> {
    let i = args.iter().position(|a| a == flag)?;
    args.get(i + 1).filter(|v| !v.starts_with("--")).cloned()
}

/// Read the dialog options, if ContactCraft was started for one contact.
pub fn parse(args: &[String]) -> Option<DialogRequest> {
    let mut p = Prefill {
        given: value(args, "--given").unwrap_or_default(),
        family: value(args, "--family").unwrap_or_default(),
        phone: value(args, "--phone").unwrap_or_default(),
        email: value(args, "--email").unwrap_or_default(),
        company: value(args, "--company").unwrap_or_default(),
        label: value(args, "--label").unwrap_or_default(),
        notes: value(args, "--notes").unwrap_or_default(),
    };
    if let Some(name) = value(args, "--name") {
        let name = name.trim().to_string();
        match name.rsplit_once(' ') {
            Some((g, f)) if p.given.is_empty() && p.family.is_empty() => {
                p.given = g.trim().to_string();
                p.family = f.trim().to_string();
            }
            None if p.given.is_empty() => p.given = name,
            _ => {}
        }
    }
    let target = if let Some(id) = value(args, "--edit-contact") {
        Target::Edit(id)
    } else if let Some(n) = value(args, "--edit-phone") {
        Target::ByPhone(n, p)
    } else if let Some(e) = value(args, "--edit-email") {
        Target::ByEmail(e, p)
    } else if args.iter().any(|a| a == "--new-contact") {
        Target::New(p)
    } else {
        return None;
    };
    Some(DialogRequest { target, result_file: value(args, "--result-file").map(PathBuf::from), theme_tab: args.iter().any(|a| a == "--theme-tab") })
}

/// The window title for a request.
pub fn title(r: &DialogRequest) -> &'static str {
    match r.target {
        Target::New(_) => "New contact - ContactCraft",
        _ => "Edit contact - ContactCraft",
    }
}

fn new_from(p: &Prefill) -> Contact {
    let mut c = Contact::new();
    c.name.given = p.given.trim().to_string();
    c.name.family = p.family.trim().to_string();
    c.phones.push(Labeled::new("mobile", p.phone.trim()));
    c.emails.push(Email { label: "home".into(), address: p.email.trim().to_string(), display_name: String::new() });
    if !p.company.trim().is_empty() {
        c.organizations.push(contactcraft_store::Organization { label: "work".into(), company: p.company.trim().to_string(), current: true, ..Default::default() });
    }
    if !p.label.trim().is_empty() {
        c.labels.push(p.label.trim().to_string());
    }
    c.notes = p.notes.clone();
    c
}

impl App {
    /// Set the editor up for the request.
    pub fn start_dialog(&mut self, request: DialogRequest) {
        let mut error = None;
        let contact = match &request.target {
            Target::New(p) => Some(new_from(p)),
            Target::Edit(id) => {
                let found = self.contact(id).cloned();
                if found.is_none() {
                    error = Some(format!("There's no contact with the id {id}."));
                }
                found
            }
            Target::ByPhone(n, p) => Some(self.store.as_ref().and_then(|s| s.find_by_phone(n).ok().flatten()).unwrap_or_else(|| new_from(&Prefill { phone: n.clone(), ..p.clone() }))),
            Target::ByEmail(e, p) => Some(self.store.as_ref().and_then(|s| s.find_by_email(e).ok().flatten()).unwrap_or_else(|| new_from(&Prefill { email: e.clone(), ..p.clone() }))),
        };
        if self.store.is_none() {
            error = Some(self.store_error.clone().unwrap_or_else(|| "The contacts couldn't be opened.".into()));
        }
        if let Some(c) = contact {
            self.selected = (!c.id.is_empty()).then(|| c.id.clone());
            self.pane = if request.theme_tab { Pane::Theme(Box::new(c)) } else { Pane::Edit(Box::new(c)) };
        }
        self.dialog = Some(DialogState { request, error, finished: false, last_saved: None, acknowledged: false });
    }

    /// The whole window in dialog mode: the editor (or theme editor) and nothing else.
    pub fn draw_dialog(&mut self, ui: &mut Ui) {
        let t = self.t;
        let error = self.dialog.as_ref().and_then(|d| d.error.clone());
        egui::CentralPanel::no_frame().frame(Frame::new().fill(t.canvas).inner_margin(Margin::same(10))).show(ui, |ui| {
            if let Some(e) = &error {
                ui.add_space(30.0);
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("ContactCraft couldn't open that contact").font(theme::semibold(18.0)).color(t.text));
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(e).font(theme::regular(13.0)).color(t.text_dim));
                    ui.add_space(14.0);
                    if widgets::primary_button(ui, &t, "Close", true).clicked() {
                        if let Some(d) = self.dialog.as_mut() {
                            d.acknowledged = true;
                        }
                    }
                });
                return;
            }
            match self.pane.clone() {
                Pane::Edit(_) => self.contact_editor(ui),
                Pane::Theme(_) => self.theme_editor(ui),
                Pane::Card => {}
            }
        });
        // Back on the card means the editing is over: saved or cancelled.
        let waiting_on_error = error.is_some() && !self.dialog.as_ref().is_some_and(|d| d.acknowledged);
        if self.pane == Pane::Card && self.confirm_discard.is_none() && !waiting_on_error {
            let saved = self.dialog.as_ref().and_then(|d| d.last_saved.clone());
            let ctx = ui.ctx().clone();
            match (saved, error) {
                (_, Some(e)) => self.finish_dialog(&ctx, "error", None, Some(e)),
                (Some(id), None) => self.finish_dialog(&ctx, "saved", Some(id), None),
                (None, None) => self.finish_dialog(&ctx, "cancelled", None, None),
            }
        }
    }

    /// Tell the caller how it went, then close the window.
    pub fn finish_dialog(&mut self, ctx: &egui::Context, status: &str, id: Option<String>, message: Option<String>) {
        let Some(d) = self.dialog.as_mut() else { return };
        if d.finished {
            return;
        }
        d.finished = true;
        self.write_result(status, id, message);
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }

    /// The exit code, the line on standard output and the result file.
    pub fn write_result(&mut self, status: &str, id: Option<String>, message: Option<String>) {
        let Some(d) = self.dialog.as_mut() else { return };
        d.finished = true;
        let result_file = d.request.result_file.clone();
        let name = id.as_deref().and_then(|i| self.contacts.iter().find(|c| c.id == i)).map(|c| c.display_name()).unwrap_or_default();
        let code = match status {
            "saved" => 0,
            "cancelled" => 1,
            _ => 2,
        };
        EXIT_CODE.store(code, Ordering::SeqCst);
        let line = match (status, &id, &message) {
            ("saved", Some(i), _) => format!("saved {i}"),
            ("error", _, Some(m)) => format!("error {m}"),
            _ => status.to_string(),
        };
        let _ = writeln!(std::io::stdout(), "{line}");
        let _ = std::io::stdout().flush();
        if let Some(path) = result_file {
            let json = serde_json::json!({"status": status, "id": id.unwrap_or_default(), "displayName": name, "message": message.unwrap_or_default()});
            let _ = std::fs::write(path, serde_json::to_string_pretty(&json).unwrap_or_default());
        }
    }

    pub fn in_dialog(&self) -> bool {
        self.dialog.is_some()
    }
}
