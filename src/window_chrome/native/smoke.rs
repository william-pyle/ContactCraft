//! Opt-in native fixture, excluded from release builds. This checks viewport
//! commands and rendered pixels, not real mouse/keyboard input.
use super::*;
use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
use windows_sys::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow,
};

pub(super) struct Smoke {
    dir: PathBuf,
    step: u8,
    next: Instant,
    initial: [i32; 4],
    checks: Vec<(&'static str, bool)>,
    minimized: Arc<AtomicBool>,
}

impl Smoke {
    pub fn from_fixture() -> Option<Self> {
        if !std::env::args().any(|a| a == "--chrome-smoke-test") {
            return None;
        }
        assert!(
            std::env::args().any(|a| a == "--demo"),
            "Native checks require demo data"
        );
        let dir = PathBuf::from(
            std::env::var_os("CONTACTCRAFT_CHROME_CHECK_DIR").expect("Isolated output directory"),
        );
        let profile = PathBuf::from(std::env::var_os("APPDATA").expect("Isolated APPDATA"));
        assert!(profile.starts_with(&dir), "Do not use the real app profile");
        std::fs::create_dir_all(&dir).unwrap();
        Some(Self {
            dir,
            step: 0,
            next: Instant::now() + Duration::from_millis(700),
            initial: [0; 4],
            checks: vec![],
            minimized: Arc::new(AtomicBool::new(false)),
        })
    }

    fn capture(&self, ctx: &egui::Context, name: &str) {
        let path = self.dir.join(format!("{name}.png"));
        ctx.request_screenshot(move |pixels| {
            let bytes: Vec<u8> = pixels.pixels.iter().flat_map(|p| p.to_array()).collect();
            image::save_buffer(
                path,
                &bytes,
                pixels.width() as u32,
                pixels.height() as u32,
                image::ColorType::Rgba8,
            )
            .unwrap();
        });
    }

    pub fn frame(&mut self, hwnd: HWND, ctx: &egui::Context, layout: &Layout) {
        ctx.request_repaint_after(Duration::from_millis(50));
        if Instant::now() < self.next {
            return;
        }
        self.next = Instant::now() + Duration::from_millis(500);
        unsafe {
            let bounds = || {
                let mut r: RECT = std::mem::zeroed();
                GetWindowRect(hwnd, &mut r);
                [r.left, r.top, r.right, r.bottom]
            };
            match self.step {
                0 => {
                    self.initial = bounds();
                    self.checks.push((
                        "native_system_menu_exists",
                        !GetSystemMenu(hwnd, 0).is_null(),
                    ));
                    let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
                    self.checks.push((
                        "native_resize_snap_keyboard_styles",
                        style & (WS_THICKFRAME | WS_SYSMENU | WS_MAXIMIZEBOX | WS_MINIMIZEBOX)
                            == (WS_THICKFRAME | WS_SYSMENU | WS_MAXIMIZEBOX | WS_MINIMIZEBOX),
                    ));
                    let mut client: RECT = std::mem::zeroed();
                    GetClientRect(hwnd, &mut client);
                    let mut origin = POINT { x: 0, y: 0 };
                    windows_sys::Win32::Graphics::Gdi::ClientToScreen(hwnd, &mut origin);
                    // Winit keeps a one-pixel seam for the undecorated DWM shadow.
                    self.checks.push((
                        "no_native_caption_insets",
                        (origin.y - self.initial[1]).abs() <= 1
                            && client.bottom == self.initial[3] - self.initial[1],
                    ));
                    let hit = |x: i32, y: i32| {
                        let mut p = POINT { x, y };
                        windows_sys::Win32::Graphics::Gdi::ClientToScreen(hwnd, &mut p);
                        SendMessageW(
                            hwnd,
                            WM_NCHITTEST,
                            0,
                            ((p.x as u16 as u32) | ((p.y as u16 as u32) << 16)) as isize,
                        ) as u32
                    };
                    let w = client.right - 1;
                    let h = client.bottom - 1;
                    self.checks.push((
                        "eight_native_resize_hits",
                        [
                            (0, 0, HTTOPLEFT),
                            (w, 0, HTTOPRIGHT),
                            (0, h, HTBOTTOMLEFT),
                            (w, h, HTBOTTOMRIGHT),
                            (0, h / 2, HTLEFT),
                            (w, h / 2, HTRIGHT),
                            (w / 2, 0, HTTOP),
                            (w / 2, h, HTBOTTOM),
                        ]
                        .iter()
                        .all(|&(x, y, expected)| hit(x, y) == expected),
                    ));
                    self.checks.push((
                        "toolbar_controls_are_client_hits",
                        layout.controls.iter().all(|r| {
                            let p = r.center() * layout.pixels_per_point;
                            hit(p.x as i32, p.y as i32) == HTCLIENT
                        }),
                    ));
                    self.checks.push((
                        "empty_toolbar_is_native_caption",
                        hit(
                            (200.0 * layout.pixels_per_point) as i32,
                            (24.0 * layout.pixels_per_point) as i32,
                        ) == HTCAPTION,
                    ));
                    self.capture(ctx, "normal");
                }
                1 => ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true)),
                2 => {
                    let mut monitor: MONITORINFO = std::mem::zeroed();
                    monitor.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
                    GetMonitorInfoW(
                        MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST),
                        &mut monitor,
                    );
                    let mut client: RECT = std::mem::zeroed();
                    GetClientRect(hwnd, &mut client);
                    let mut origin = POINT { x: 0, y: 0 };
                    windows_sys::Win32::Graphics::Gdi::ClientToScreen(hwnd, &mut origin);
                    let work = monitor.rcWork;
                    self.checks
                        .push(("maximized_native_state", IsZoomed(hwnd) != 0));
                    self.checks.push((
                        "maximized_client_equals_monitor_work_area",
                        [
                            origin.x,
                            origin.y,
                            origin.x + client.right,
                            origin.y + client.bottom,
                        ] == [work.left, work.top, work.right, work.bottom],
                    ));
                    self.capture(ctx, "maximized");
                }
                3 => ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false)),
                4 => {
                    self.checks.push((
                        "restore_recovers_original_bounds",
                        bounds() == self.initial && IsZoomed(hwnd) == 0,
                    ));
                    ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                        960.0, 600.0,
                    )));
                    ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(egui::pos2(
                        60.0, 60.0,
                    )));
                }
                5 => {
                    let b = bounds();
                    let dpi = GetDpiForWindow(hwnd) as f32 / 96.0;
                    self.checks.push((
                        "native_move_and_resize_commands",
                        (b[0] as f32 - 60.0 * dpi).abs() <= 1.0
                            && (b[2] - b[0]) as f32 == 960.0 * dpi,
                    ));
                    self.capture(ctx, "narrow");
                }
                6 => ctx.set_zoom_factor(2.0),
                7 => self.capture(ctx, "zoom-200"),
                8 => {
                    ctx.set_zoom_factor(1.0);
                    ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    let handle = hwnd as usize;
                    let minimized = self.minimized.clone();
                    // A minimized window may stop repainting. Restore our own
                    // fixture after observing its native state, without input.
                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(400));
                        minimized.store(IsIconic(handle as HWND) != 0, Ordering::SeqCst);
                        PostMessageW(handle as HWND, WM_SYSCOMMAND, SC_RESTORE as usize, 0);
                    });
                }
                9 => {
                    self.checks.push((
                        "minimize_native_state",
                        self.minimized.load(Ordering::SeqCst),
                    ));
                    self.checks
                        .push(("restore_after_minimize", IsIconic(hwnd) == 0));
                    let result = serde_json::json!({
                        "passed": self.checks.iter().all(|(_, pass)| *pass),
                        "checks": self.checks, "dpi": GetDpiForWindow(hwnd),
                        "limits": "Viewport commands and native hit tests only; no pointer, keyboard, modal menu interaction or mixed-monitor DPI transition verification."
                    });
                    std::fs::write(
                        self.dir.join("native-results.json"),
                        serde_json::to_vec_pretty(&result).unwrap(),
                    )
                    .unwrap();
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
                _ => {}
            }
        }
        self.step += 1;
    }
}
