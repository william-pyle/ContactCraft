//! Native hit testing over the existing egui toolbar. Winit keeps the sizing,
//! system-menu and snap styles and calculates the maximized monitor work area.
//! We supply only the missing undecorated hit tests; the OS owns move/resize.
#[cfg(windows)]
mod native {
    #[cfg(debug_assertions)]
    #[path = "smoke.rs"]
    mod smoke;
    use egui::{Pos2, Rect};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use std::cell::{Cell, RefCell};
    use windows_sys::Win32::{
        Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
        Graphics::Gdi::ScreenToClient,
        UI::{
            HiDpi::{GetDpiForWindow, GetSystemMetricsForDpi},
            Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
            WindowsAndMessaging::*,
        },
    };

    const SUBCLASS: usize = 0x4343;
    const SHOW_MENU: u32 = WM_APP + 0x434;

    #[derive(Default)]
    struct Layout {
        toolbar: Option<Rect>,
        controls: Vec<Rect>,
        pixels_per_point: f32,
    }

    struct State {
        hwnd: Cell<HWND>,
        layout: RefCell<Layout>,
        #[cfg(debug_assertions)]
        smoke: RefCell<Option<smoke::Smoke>>,
    }

    pub struct WindowChrome(Box<State>);

    impl WindowChrome {
        pub fn install(
            cc: &eframe::CreationContext<'_>,
        ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
            let RawWindowHandle::Win32(handle) = cc.window_handle()?.as_raw() else {
                return Err("ContactCraft requires a Windows window".into());
            };
            let state = Box::new(State {
                hwnd: Cell::new(handle.hwnd.get() as HWND),
                layout: RefCell::new(Layout::default()),
                #[cfg(debug_assertions)]
                smoke: RefCell::new(smoke::Smoke::from_fixture()),
            });
            // The allocation stays stable until Drop. Both egui and this subclass
            // run on the event-loop thread. NCDESTROY invalidates the handle first.
            if unsafe {
                SetWindowSubclass(
                    state.hwnd.get(),
                    Some(window_proc),
                    SUBCLASS,
                    (&*state as *const State) as usize,
                )
            } == 0
            {
                return Err(std::io::Error::last_os_error().into());
            }
            Ok(Self(state))
        }

        pub fn set_toolbar(&self, toolbar: Rect, controls: Vec<Rect>, pixels_per_point: f32) {
            *self.0.layout.borrow_mut() = Layout {
                toolbar: Some(toolbar),
                controls,
                pixels_per_point,
            };
        }

        #[cfg(debug_assertions)]
        pub fn smoke_frame(&self, ctx: &egui::Context) {
            if let Some(smoke) = self.0.smoke.borrow_mut().as_mut() {
                smoke.frame(self.0.hwnd.get(), ctx, &self.0.layout.borrow());
            }
        }

        pub fn system_menu(&self) -> Result<(), String> {
            // Never enter a modal Win32 menu loop while egui holds frame locks.
            if unsafe { PostMessageW(self.0.hwnd.get(), SHOW_MENU, 0, 0) } == 0 {
                return Err(std::io::Error::last_os_error().to_string());
            }
            Ok(())
        }
    }

    impl Drop for WindowChrome {
        fn drop(&mut self) {
            let hwnd = self.0.hwnd.get();
            if !hwnd.is_null() {
                unsafe {
                    RemoveWindowSubclass(hwnd, Some(window_proc), SUBCLASS);
                }
            }
        }
    }

    fn hit_test(
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        border: i32,
        maximized: bool,
        layout: &Layout,
    ) -> u32 {
        if !maximized {
            let left = x < border;
            let right = x >= width - border;
            let top = y < border;
            let bottom = y >= height - border;
            match (left, right, top, bottom) {
                (true, _, true, _) => return HTTOPLEFT,
                (_, true, true, _) => return HTTOPRIGHT,
                (true, _, _, true) => return HTBOTTOMLEFT,
                (_, true, _, true) => return HTBOTTOMRIGHT,
                (true, _, _, _) => return HTLEFT,
                (_, true, _, _) => return HTRIGHT,
                (_, _, true, _) => return HTTOP,
                (_, _, _, true) => return HTBOTTOM,
                _ => {}
            }
        }
        let p = Pos2::new(x as f32, y as f32) / layout.pixels_per_point.max(1.0e-6);
        if layout.toolbar.is_some_and(|r| r.contains(p))
            && !layout.controls.iter().any(|r| r.contains(p))
        {
            HTCAPTION
        } else {
            HTCLIENT
        }
    }

    unsafe fn show_menu(hwnd: HWND) {
        unsafe {
            let menu = GetSystemMenu(hwnd, 0);
            if menu.is_null() {
                return;
            }
            let maximized = IsZoomed(hwnd) != 0;
            for (command, enabled) in [
                (SC_RESTORE, maximized),
                (SC_MAXIMIZE, !maximized),
                (SC_SIZE, !maximized),
                (SC_MOVE, !maximized),
            ] {
                EnableMenuItem(
                    menu,
                    command,
                    MF_BYCOMMAND | if enabled { MF_ENABLED } else { MF_GRAYED },
                );
            }
            let dpi = GetDpiForWindow(hwnd) as f32 / 96.0;
            let mut anchor = POINT {
                x: (8.0 * dpi) as i32,
                y: (48.0 * dpi) as i32,
            };
            windows_sys::Win32::Graphics::Gdi::ClientToScreen(hwnd, &mut anchor);
            let command = TrackPopupMenu(
                menu,
                TPM_RETURNCMD | TPM_RIGHTBUTTON,
                anchor.x,
                anchor.y,
                0,
                hwnd,
                std::ptr::null(),
            );
            if command != 0 {
                PostMessageW(hwnd, WM_SYSCOMMAND, command as usize, 0);
            }
        }
    }

    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
        _: usize,
        data: usize,
    ) -> LRESULT {
        unsafe {
            let state = &*(data as *const State);
            match msg {
                WM_NCHITTEST => {
                    // Signed extraction is required for monitors left/above primary.
                    let mut point = POINT {
                        x: lparam as i16 as i32,
                        y: (lparam >> 16) as i16 as i32,
                    };
                    let mut rect: RECT = std::mem::zeroed();
                    if ScreenToClient(hwnd, &mut point) != 0 && GetClientRect(hwnd, &mut rect) != 0
                    {
                        let dpi = GetDpiForWindow(hwnd);
                        let border = GetSystemMetricsForDpi(SM_CXSIZEFRAME, dpi)
                            + GetSystemMetricsForDpi(SM_CXPADDEDBORDER, dpi);
                        return hit_test(
                            point.x,
                            point.y,
                            rect.right,
                            rect.bottom,
                            border.max(1),
                            IsZoomed(hwnd) != 0,
                            &state.layout.borrow(),
                        ) as LRESULT;
                    }
                }
                SHOW_MENU => {
                    show_menu(hwnd);
                    return 0;
                }
                WM_NCRBUTTONUP if wparam == HTCAPTION as usize => {
                    show_menu(hwnd);
                    return 0;
                }
                WM_SYSCOMMAND if wparam as u32 & 0xfff0 == SC_KEYMENU && lparam == 32 => {
                    show_menu(hwnd);
                    return 0;
                }
                WM_NCDESTROY => {
                    state.hwnd.set(std::ptr::null_mut());
                    RemoveWindowSubclass(hwnd, Some(window_proc), SUBCLASS);
                }
                _ => {}
            }
            DefSubclassProc(hwnd, msg, wparam, lparam)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn all_edges_corners_and_maximized_client() {
            let layout = Layout::default();
            for (x, y, expected) in [
                (0, 0, HTTOPLEFT),
                (999, 0, HTTOPRIGHT),
                (0, 699, HTBOTTOMLEFT),
                (999, 699, HTBOTTOMRIGHT),
                (0, 300, HTLEFT),
                (999, 300, HTRIGHT),
                (500, 0, HTTOP),
                (500, 699, HTBOTTOM),
            ] {
                assert_eq!(hit_test(x, y, 1000, 700, 8, false, &layout), expected);
                assert_eq!(hit_test(x, y, 1000, 700, 8, true, &layout), HTCLIENT);
            }
        }
        #[test]
        fn toolbar_excludes_controls_at_each_scale() {
            for scale in [1.0, 1.25, 1.5, 2.0, 2.5] {
                let layout = Layout {
                    toolbar: Some(Rect::from_min_max(Pos2::ZERO, Pos2::new(1000.0, 48.0))),
                    controls: vec![Rect::from_min_max(
                        Pos2::new(300.0, 7.0),
                        Pos2::new(600.0, 41.0),
                    )],
                    pixels_per_point: scale,
                };
                let hit = |x: f32, y: f32| {
                    hit_test(
                        (x * scale) as i32,
                        (y * scale) as i32,
                        (1000.0 * scale) as i32,
                        (700.0 * scale) as i32,
                        (8.0 * scale) as i32,
                        false,
                        &layout,
                    )
                };
                assert_eq!(hit(200.0, 24.0), HTCAPTION);
                assert_eq!(hit(400.0, 24.0), HTCLIENT);
                assert_eq!(hit(200.0, 60.0), HTCLIENT);
                assert_eq!(hit(200.0, 2.0), HTTOP);
            }
        }
    }
}

#[cfg(windows)]
pub use native::WindowChrome;

#[cfg(not(windows))]
pub struct WindowChrome;
#[cfg(not(windows))]
impl WindowChrome {
    pub fn install(
        _: &eframe::CreationContext<'_>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self)
    }
    pub fn set_toolbar(&self, _: egui::Rect, _: Vec<egui::Rect>, _: f32) {}
    pub fn system_menu(&self) -> Result<(), String> {
        Err("The system menu requires Windows".into())
    }
}
