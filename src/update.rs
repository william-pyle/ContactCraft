//! New versions: on start (and from Help › Check for Updates) ContactSolutions asks GitHub for the
//! latest release of william-pyle/ContactSolutions. When it's newer than this copy, it offers to
//! download the installer and run it (the installer replaces this version and can start the new
//! one when it finishes). Nothing about the person or their contacts is sent: it's one anonymous
//! request for the public release list.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::Duration;

use egui::{Margin, ScrollArea, Vec2};

use crate::app::App;
use crate::theme;
use crate::widgets;

pub const REPO: &str = "william-pyle/ContactSolutions";
pub const RELEASES_PAGE: &str = "https://github.com/william-pyle/ContactSolutions/releases";
const INSTALLER_SUFFIX: &str = "-windows-x64.msi";

#[derive(Clone, Debug)]
pub struct Release {
    pub version: String,
    pub notes: String,
    pub page: String,
    pub installer_url: Option<String>,
    pub installer_name: String,
    pub size: u64,
}

pub enum UpdateState {
    Idle,
    Checking,
    Available(Release),
    Downloading { release: Release, got: u64, total: u64 },
    Ready { release: Release, file: PathBuf },
    Failed(String),
}

enum Msg {
    Latest(Option<Release>, bool),
    CheckFailed(String, bool),
    Progress(u64, u64),
    Downloaded(PathBuf),
    DownloadFailed(String),
}

pub struct Updater {
    pub state: UpdateState,
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
    /// The person closed the prompt: don't show it again this session.
    pub dismissed: bool,
}

impl Default for Updater {
    fn default() -> Self {
        let (tx, rx) = channel();
        Updater { state: UpdateState::Idle, tx, rx, dismissed: false }
    }
}

/// "v1.2.3" / "1.2.3-beta" → (1, 2, 3).
pub fn parse_version(s: &str) -> Option<(u64, u64, u64)> {
    let core = s.trim().trim_start_matches(['v', 'V']).split(['-', '+']).next()?;
    let mut it = core.split('.').map(|p| p.parse::<u64>());
    let a = it.next()?.ok()?;
    let b = it.next().unwrap_or(Ok(0)).ok()?;
    let c = it.next().unwrap_or(Ok(0)).ok()?;
    Some((a, b, c))
}

pub fn is_newer(candidate: &str, current: &str) -> bool {
    match (parse_version(candidate), parse_version(current)) {
        (Some(a), Some(b)) => a > b,
        _ => false,
    }
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .user_agent(concat!("ContactSolutions/", env!("CARGO_PKG_VERSION"), " (update check)"))
        .build()
        .into()
}

fn fetch_latest() -> Result<Option<Release>, String> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let mut resp = match agent().get(&url).header("Accept", "application/vnd.github+json").call() {
        Ok(r) => r,
        // No release published yet.
        Err(ureq::Error::StatusCode(404)) => return Ok(None),
        Err(e) => return Err(e.to_string()),
    };
    let v: serde_json::Value = resp.body_mut().read_json().map_err(|e| e.to_string())?;
    if v.get("draft").and_then(|d| d.as_bool()) == Some(true) || v.get("prerelease").and_then(|d| d.as_bool()) == Some(true) {
        return Ok(None);
    }
    let tag = v.get("tag_name").and_then(|t| t.as_str()).unwrap_or_default().to_string();
    let Some(_) = parse_version(&tag) else { return Ok(None) };
    let installer = v.get("assets").and_then(|a| a.as_array()).and_then(|a| a.iter().find(|x| x.get("name").and_then(|n| n.as_str()).is_some_and(|n| n.ends_with(INSTALLER_SUFFIX))));
    Ok(Some(Release {
        version: tag.trim_start_matches(['v', 'V']).to_string(),
        notes: v.get("body").and_then(|b| b.as_str()).unwrap_or_default().to_string(),
        page: v.get("html_url").and_then(|u| u.as_str()).unwrap_or(RELEASES_PAGE).to_string(),
        installer_url: installer.and_then(|a| a.get("browser_download_url")).and_then(|u| u.as_str()).map(String::from),
        installer_name: installer.and_then(|a| a.get("name")).and_then(|n| n.as_str()).unwrap_or_default().to_string(),
        size: installer.and_then(|a| a.get("size")).and_then(|s| s.as_u64()).unwrap_or(0),
    }))
}

fn download(url: &str, name: &str, tx: &Sender<Msg>, ctx: &egui::Context) -> Result<PathBuf, String> {
    let resp = agent().get(url).call().map_err(|e| e.to_string())?;
    let total = resp.headers().get("content-length").and_then(|v| v.to_str().ok()).and_then(|v| v.parse().ok()).unwrap_or(0u64);
    let dir = std::env::temp_dir().join("ContactSolutions-update");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let safe: String = name.chars().filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | '_')).collect();
    let path = dir.join(if safe.is_empty() { "ContactSolutions-windows-x64.msi".to_string() } else { safe });
    let part = path.with_extension("part");
    let mut file = std::fs::File::create(&part).map_err(|e| e.to_string())?;
    let mut reader = resp.into_body().into_reader();
    let mut buf = vec![0u8; 64 * 1024];
    let mut got = 0u64;
    loop {
        let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        got += n as u64;
        let _ = tx.send(Msg::Progress(got, total));
        ctx.request_repaint();
    }
    file.flush().map_err(|e| e.to_string())?;
    drop(file);
    if total > 0 && got != total {
        return Err("The download stopped before it finished.".into());
    }
    // An MSI file starts with the compound-file signature.
    let mut head = [0u8; 8];
    std::fs::File::open(&part).and_then(|mut f| f.read_exact(&mut head)).map_err(|e| e.to_string())?;
    if head != [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1] {
        return Err("What came down isn't an installer.".into());
    }
    let _ = std::fs::remove_file(&path);
    std::fs::rename(&part, &path).map_err(|e| e.to_string())?;
    Ok(path)
}

impl App {
    /// Ask GitHub for the latest release (in the background).
    pub fn check_for_updates(&mut self, ctx: &egui::Context, manual: bool) {
        if matches!(self.updater.state, UpdateState::Checking | UpdateState::Downloading { .. }) {
            return;
        }
        self.updater.state = UpdateState::Checking;
        self.updater.dismissed = false;
        let tx = self.updater.tx.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let msg = match fetch_latest() {
                Ok(r) => Msg::Latest(r, manual),
                Err(e) => Msg::CheckFailed(e, manual),
            };
            let _ = tx.send(msg);
            ctx.request_repaint();
        });
    }

    fn start_download(&mut self, ctx: &egui::Context, release: Release) {
        let Some(url) = release.installer_url.clone() else {
            crate::app::open_url(&release.page);
            return;
        };
        let tx = self.updater.tx.clone();
        let ctx = ctx.clone();
        let name = release.installer_name.clone();
        self.updater.state = UpdateState::Downloading { got: 0, total: release.size, release };
        std::thread::spawn(move || {
            let msg = match download(&url, &name, &tx, &ctx) {
                Ok(p) => Msg::Downloaded(p),
                Err(e) => Msg::DownloadFailed(e),
            };
            let _ = tx.send(msg);
            ctx.request_repaint();
        });
    }

    /// Answers from the background work.
    pub fn update_events(&mut self) {
        while let Ok(m) = self.updater.rx.try_recv() {
            match m {
                Msg::Latest(Some(r), manual) if is_newer(&r.version, env!("CARGO_PKG_VERSION")) => {
                    if !manual && self.settings.skip_version == r.version {
                        self.updater.state = UpdateState::Idle;
                    } else {
                        self.updater.state = UpdateState::Available(r);
                    }
                }
                Msg::Latest(_, manual) => {
                    self.updater.state = UpdateState::Idle;
                    if manual {
                        self.notify(format!("You have the latest version of ContactSolutions ({}).", env!("CARGO_PKG_VERSION")), false);
                    }
                }
                Msg::CheckFailed(e, manual) => {
                    self.updater.state = UpdateState::Idle;
                    if manual {
                        self.notify(format!("Couldn't check for a new version: {e}"), true);
                    }
                }
                Msg::Progress(got, total) => {
                    if let UpdateState::Downloading { got: g, total: t, .. } = &mut self.updater.state {
                        *g = got;
                        if total > 0 {
                            *t = total;
                        }
                    }
                }
                Msg::Downloaded(file) => {
                    if let UpdateState::Downloading { release, .. } = &self.updater.state {
                        self.updater.state = UpdateState::Ready { release: release.clone(), file };
                    }
                }
                Msg::DownloadFailed(e) => self.updater.state = UpdateState::Failed(e),
            }
        }
    }

    /// The "a new version is available" prompt.
    pub fn update_dialog(&mut self, ctx: &egui::Context) {
        if self.updater.dismissed {
            return;
        }
        let t = self.t;
        let current = env!("CARGO_PKG_VERSION");
        enum Act {
            Download(Release),
            Install(PathBuf),
            Later,
            Skip(String),
            Page(String),
        }
        let mut act = None;
        let (title, release) = match &self.updater.state {
            UpdateState::Available(r) => (format!("ContactSolutions {} is available", r.version), r.clone()),
            UpdateState::Downloading { release, .. } => (format!("Downloading ContactSolutions {}…", release.version), release.clone()),
            UpdateState::Ready { release, .. } => (format!("ContactSolutions {} is ready to install", release.version), release.clone()),
            UpdateState::Failed(_) => ("The new version couldn't be downloaded".to_string(), Release { version: String::new(), notes: String::new(), page: RELEASES_PAGE.into(), installer_url: None, installer_name: String::new(), size: 0 }),
            _ => return,
        };
        let modal = egui::Modal::new(egui::Id::unique("update")).frame(crate::chrome::dialog_frame(&t)).show(ctx, |ui| {
            ui.set_width(520.0);
            ui.horizontal(|ui| {
                let (r, _) = ui.allocate_exact_size(Vec2::splat(30.0), egui::Sense::hover());
                ui.painter().rect_filled(r, 7.0, t.accent);
                crate::icons::paint(ui.painter(), r.shrink(5.0), crate::icons::Icon::Export, egui::Color32::WHITE, egui::Color32::WHITE);
                ui.label(egui::RichText::new(&title).font(theme::semibold(18.0)).color(t.text));
            });
            ui.add_space(4.0);
            match &self.updater.state {
                UpdateState::Available(r) => {
                    ui.label(egui::RichText::new(format!("You have version {current}. Your contacts stay exactly as they are when you update.")).font(theme::regular(12.5)).color(t.text_dim));
                    if !r.notes.trim().is_empty() {
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new("What's new").font(theme::semibold(13.5)).color(t.text));
                        egui::Frame::new().fill(t.panel).corner_radius(6).inner_margin(Margin::same(10)).show(ui, |ui| {
                            ScrollArea::vertical().max_height(220.0).show(ui, |ui| {
                                ui.set_width(480.0);
                                let notes: String = r.notes.chars().take(4000).collect();
                                ui.add(egui::Label::new(egui::RichText::new(notes.replace("**", "").replace("### ", "").replace("## ", "")).font(theme::regular(12.5)).color(t.text)).wrap());
                            });
                        });
                    }
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        let label = if r.installer_url.is_some() { "Download and install" } else { "Open the download page" };
                        if widgets::primary_button(ui, &t, label, true).clicked() {
                            act = Some(Act::Download(r.clone()));
                        }
                        if widgets::secondary_button(ui, &t, "Later").clicked() {
                            act = Some(Act::Later);
                        }
                        if widgets::secondary_button(ui, &t, "Skip this version").clicked() {
                            act = Some(Act::Skip(r.version.clone()));
                        }
                    });
                }
                UpdateState::Downloading { got, total, .. } => {
                    let frac = if *total > 0 { *got as f32 / *total as f32 } else { 0.0 };
                    ui.add_space(6.0);
                    ui.add(egui::ProgressBar::new(frac).desired_width(480.0).text(format!("{:.1} of {:.1} MB", *got as f64 / 1e6, (*total).max(*got) as f64 / 1e6)));
                    ui.add_space(8.0);
                    if widgets::secondary_button(ui, &t, "Hide").clicked() {
                        act = Some(Act::Later);
                    }
                }
                UpdateState::Ready { file, .. } => {
                    ui.label(egui::RichText::new("ContactSolutions closes, and the installer replaces it with the new version. Your contacts aren't touched. Windows may ask for permission.").font(theme::regular(12.5)).color(t.text_dim));
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        if widgets::primary_button(ui, &t, "Install now", true).clicked() {
                            act = Some(Act::Install(file.clone()));
                        }
                        if widgets::secondary_button(ui, &t, "Later").clicked() {
                            act = Some(Act::Later);
                        }
                    });
                }
                UpdateState::Failed(e) => {
                    ui.add(egui::Label::new(egui::RichText::new(e).font(theme::regular(12.5)).color(t.red)).wrap());
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        if widgets::primary_button(ui, &t, "Open the download page", true).clicked() {
                            act = Some(Act::Page(release.page.clone()));
                        }
                        if widgets::secondary_button(ui, &t, "Close").clicked() {
                            act = Some(Act::Later);
                        }
                    });
                }
                _ => {}
            }
        });
        if modal.should_close() && act.is_none() {
            act = Some(Act::Later);
        }
        match act {
            Some(Act::Download(r)) => self.start_download(ctx, r),
            Some(Act::Install(file)) => {
                match std::process::Command::new("msiexec").arg("/i").arg(&file).spawn() {
                    Ok(_) => {
                        self.save_settings();
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    Err(e) => self.updater.state = UpdateState::Failed(format!("The installer couldn't be started: {e}")),
                }
            }
            Some(Act::Later) => {
                self.updater.dismissed = true;
                if matches!(self.updater.state, UpdateState::Failed(_)) {
                    self.updater.state = UpdateState::Idle;
                }
            }
            Some(Act::Skip(v)) => {
                self.settings.skip_version = v;
                self.save_settings();
                self.updater.state = UpdateState::Idle;
            }
            Some(Act::Page(url)) => {
                crate::app::open_url(&url);
                self.updater.state = UpdateState::Idle;
            }
            None => {}
        }
    }

    /// A small "new version" chip for the status bar while the prompt is hidden.
    pub fn update_badge(&self) -> Option<String> {
        if !self.updater.dismissed {
            return None;
        }
        match &self.updater.state {
            UpdateState::Available(r) => Some(format!("ContactSolutions {} is available", r.version)),
            UpdateState::Downloading { got, total, .. } if *total > 0 => Some(format!("Downloading the new version… {}%", got * 100 / total)),
            UpdateState::Ready { release, .. } => Some(format!("ContactSolutions {} is ready to install", release.version)),
            _ => None,
        }
    }

    pub fn show_update_prompt(&mut self) {
        self.updater.dismissed = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions() {
        assert!(is_newer("v1.0.1", "1.0.0"));
        assert!(is_newer("1.1", "1.0.9"));
        assert!(is_newer("v2.0.0-beta", "1.9.9"));
        assert!(!is_newer("v1.0.0", "1.0.0"));
        assert!(!is_newer("v0.9.0", "1.0.0"));
        assert!(!is_newer("nightly", "1.0.0"));
    }
}
