//! ContactCraft: your contacts on Windows, shared with every Craft app, in the MailCraft style.

#![windows_subsystem = "windows"]

mod app;
mod backstage;
mod card;
mod chrome;
mod cleanup_ui;
mod demo;
mod editor;
mod icons;
mod io;
mod list;
mod theme;
mod theme_editor;
mod update;
mod widgets;

/// The window icon: the family's blue tile with a white contact card, drawn here.
fn app_icon() -> egui::IconData {
    const N: usize = 64;
    let mut rgba = vec![0u8; N * N * 4];
    let blue = [0x0F, 0x68, 0xF0];
    let inside_round = |x: f32, y: f32, x0: f32, y0: f32, x1: f32, y1: f32, r: f32| -> bool {
        let cx = x.clamp(x0 + r, x1 - r);
        let cy = y.clamp(y0 + r, y1 - r);
        (x - cx).powi(2) + (y - cy).powi(2) <= r * r
    };
    for py in 0..N {
        for px in 0..N {
            let (x, y) = (px as f32 + 0.5, py as f32 + 0.5);
            let i = (py * N + px) * 4;
            if !inside_round(x, y, 2.0, 2.0, 62.0, 62.0, 14.0) {
                continue;
            }
            let mut c = blue;
            if inside_round(x, y, 11.0, 16.0, 53.0, 48.0, 6.0) {
                c = [255, 255, 255];
                let head = (x - 22.5).powi(2) + (y - 26.5).powi(2) <= 30.0;
                let body = y >= 33.0 && (x - 22.5).powi(2) + (y - 42.0).powi(2) <= 72.0 && y <= 47.0;
                let lines = (34.0..=48.0).contains(&x) && ((24.0..=27.5).contains(&y) || ((31.0..=34.5).contains(&y) && x <= 44.0) || ((38.0..=41.5).contains(&y) && x <= 46.0));
                if head || body || lines {
                    c = blue;
                }
            }
            rgba[i] = c[0];
            rgba[i + 1] = c[1];
            rgba[i + 2] = c[2];
            rgba[i + 3] = 255;
        }
    }
    egui::IconData { rgba, width: N as u32, height: N as u32 }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("ContactCraft")
            .with_app_id("ContactCraft")
            .with_inner_size([1440.0, 920.0])
            .with_min_inner_size([960.0, 600.0])
            .with_drag_and_drop(true)
            .with_icon(app_icon()),
        ..Default::default()
    };
    eframe::run_native("ContactCraft", options, Box::new(|cc| Ok(Box::new(app::App::new(cc)))))
}
