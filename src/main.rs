//! ContactSolutions: your contacts on Windows, shared with every Craft app, in the MailCraft style.

#![windows_subsystem = "windows"]

mod app;
mod backstage;
mod card;
mod chrome;
mod cleanup_ui;
mod demo;
mod dialog;
mod groups;
mod editor;
mod icons;
mod io;
mod list;
mod theme;
mod theme_editor;
mod update;
mod widgets;
mod window_chrome;

/// The blue Craft tile and the person silhouette used by the installer and website.
fn app_icon() -> egui::IconData {
    let img = image::load_from_memory(include_bytes!("../assets/contactsolutions-256.png"))
        .expect("ContactSolutions icon PNG").to_rgba8();
    egui::IconData { rgba: img.as_raw().clone(), width: img.width(), height: img.height() }
}

fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    // Started by another app to create or edit one contact or group.
    if let Some(req) = dialog::parse(&args) {
        let group_dialog = matches!(req.target, dialog::Target::NewGroup { .. } | dialog::Target::EditGroup(_));
        let size = if group_dialog { [780.0, 760.0] } else { [900.0, 880.0] };
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title(dialog::title(&req))
                .with_decorations(false)
                .with_app_id("ContactSolutions.Dialog")
                .with_inner_size(size)
                .with_min_inner_size([620.0, 480.0])
                .with_always_on_top()
                .with_drag_and_drop(true)
                .with_icon(app_icon()),
            centered: true,
            ..Default::default()
        };
        let result = eframe::run_native("ContactSolutions", options, Box::new(move |cc| {
            let mut app = app::App::new(cc, window_chrome::WindowChrome::install(cc)?);
            app.start_dialog(req);
            Ok(Box::new(app))
        }));
        if let Err(e) = result {
            eprintln!("error {e}");
            std::process::exit(2);
        }
        std::process::exit(dialog::EXIT_CODE.load(std::sync::atomic::Ordering::SeqCst));
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ContactSolutions")
            .with_decorations(false)
            .with_app_id("ContactSolutions")
            .with_inner_size([1440.0, 920.0])
            .with_min_inner_size([960.0, 600.0])
            .with_drag_and_drop(true)
            .with_icon(app_icon()),
        ..Default::default()
    };
    eframe::run_native("ContactSolutions", options, Box::new(|cc| Ok(Box::new(app::App::new(cc, window_chrome::WindowChrome::install(cc)?)))))
}
