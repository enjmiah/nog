use crate::config::rule::Rule;
use crate::util;
use crate::{display::Display, CONFIG};
use gwl_ex_style::GwlExStyle;
use gwl_style::GwlStyle;
use log::error;
use winapi::shared::windef::HWND;
use winapi::shared::windef::RECT;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::winuser::AdjustWindowRectEx;
use winapi::um::winuser::GetForegroundWindow;
use winapi::um::winuser::GetParent;
use winapi::um::winuser::GetWindowLongA;
use winapi::um::winuser::GetWindowRect;
use winapi::um::winuser::SendMessageA;
use winapi::um::winuser::SetForegroundWindow;
use winapi::um::winuser::SetWindowLongA;
use winapi::um::winuser::SetWindowPos;
use winapi::um::winuser::ShowWindow;
use winapi::um::winuser::GWL_EXSTYLE;
use winapi::um::winuser::GWL_STYLE;
use winapi::um::winuser::HWND_NOTOPMOST;
use winapi::um::winuser::HWND_TOP;
use winapi::um::winuser::HWND_TOPMOST;
use winapi::um::winuser::SM_CXFRAME;
use winapi::um::winuser::SM_CYCAPTION;
use winapi::um::winuser::SM_CYFRAME;
use winapi::um::winuser::SWP_NOMOVE;
use winapi::um::winuser::SWP_NOSIZE;
use winapi::um::winuser::SW_HIDE;
use winapi::um::winuser::SW_SHOW;
use winapi::um::{
    processthreadsapi::OpenProcess,
    psapi::GetModuleFileNameExA,
    winnt::{PROCESS_QUERY_INFORMATION, PROCESS_VM_READ},
    winuser::{
        GetClientRect, GetSystemMetricsForDpi, GetWindowThreadProcessId, SC_MAXIMIZE, SC_MINIMIZE,
        SC_RESTORE, WM_CLOSE, WM_PAINT, WM_SYSCOMMAND,
    },
};

pub mod gwl_ex_style;
pub mod gwl_style;

#[derive(Clone)]
pub struct Window {
    pub id: i32,
    pub title: String,
    pub maximized: bool,
    pub rule: Option<Rule>,
    pub style: GwlStyle,
    pub exstyle: GwlExStyle,
    pub original_style: GwlStyle,
    pub original_rect: RECT,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            id: 0,
            title: String::from(""),
            maximized: false,
            rule: None,
            style: GwlStyle::default(),
            exstyle: GwlExStyle::default(),
            original_style: GwlStyle::default(),
            original_rect: RECT::default(),
        }
    }
}

impl Window {
    pub fn new(hwnd: i32) -> Self {
        Self {
            id: hwnd,
            ..Self::default()
        }
    }
    pub fn reset_style(&mut self) {
        self.style = self.original_style;
    }
    pub fn reset(&mut self) {
        self.reset_style();
        self.update_style();
        self.reset_pos();

        if self.maximized {
            self.maximize();
        }
    }
    pub fn reset_pos(&self) {
        unsafe {
            SetWindowPos(
                self.id as HWND,
                std::ptr::null_mut(),
                self.original_rect.left,
                self.original_rect.top,
                self.original_rect.right - self.original_rect.left,
                self.original_rect.bottom - self.original_rect.top,
                0,
            );
        }
    }
    pub fn get_client_rect(&self) -> RECT {
        let mut rect: RECT = RECT::default();
        unsafe {
            GetClientRect(self.id as HWND, &mut rect);
        }
        rect
    }
    pub fn get_foreground_window() -> Result<HWND, util::WinApiResultError> {
        unsafe { util::winapi_ptr_to_result(GetForegroundWindow()) }
    }
    pub fn get_parent_window(&self) -> Result<HWND, util::WinApiResultError> {
        unsafe { util::winapi_ptr_to_result(GetParent(self.id as HWND)) }
    }
    pub fn get_style(&self) -> Result<GwlStyle, util::WinApiResultError> {
        unsafe {
            let bits = util::winapi_nullable_to_result(GetWindowLongA(self.id as HWND, GWL_STYLE))?;
            Ok(GwlStyle::from_bits_unchecked(bits as u32 as i32))
        }
    }
    pub fn get_ex_style(&self) -> Result<GwlExStyle, util::WinApiResultError> {
        unsafe {
            let bits =
                util::winapi_nullable_to_result(GetWindowLongA(self.id as HWND, GWL_EXSTYLE))?;
            Ok(GwlExStyle::from_bits_unchecked(bits as u32 as i32))
        }
    }
    pub fn get_rect(&self) -> Result<RECT, util::WinApiResultError> {
        unsafe {
            let mut temp = RECT::default();
            util::winapi_nullable_to_result(GetWindowRect(self.id as HWND, &mut temp))?;
            Ok(temp)
        }
    }
    pub fn show(&self) {
        unsafe {
            ShowWindow(self.id as HWND, SW_SHOW);
        }
    }
    pub fn hide(&self) {
        unsafe {
            ShowWindow(self.id as HWND, SW_HIDE);
        }
    }
    pub fn calculate_window_rect(
        &self,
        display: &Display,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> RECT {
        let rule = self.rule.clone().unwrap_or_default();
        let (display_app_bar, remove_title_bar, bar_height, use_border) = {
            let config = CONFIG.lock().unwrap();

            (
                config.display_app_bar,
                config.remove_title_bar,
                config.bar.height,
                config.use_border,
            )
        };

        let mut left = x;
        let mut right = x + width;
        let mut top = y;
        let mut bottom = y + height;

        unsafe {
            let border_width = GetSystemMetricsForDpi(SM_CXFRAME, display.dpi);
            let border_height = GetSystemMetricsForDpi(SM_CYFRAME, display.dpi);

            if rule.chromium || rule.firefox || !remove_title_bar {
                let caption_height = GetSystemMetricsForDpi(SM_CYCAPTION, display.dpi);
                top += caption_height;
            } else {
                top -= border_height * 2;

                if use_border {
                    left += 1;
                    right -= 1;
                    top += 1;
                    bottom -= 1;
                }
            }

            if display_app_bar {
                top += bar_height;
                bottom += bar_height;
            }

            if rule.firefox || rule.chromium || (!remove_title_bar && rule.has_custom_titlebar) {
                if rule.firefox {
                    left -= (border_width as f32 * 1.5) as i32;
                    right += (border_width as f32 * 1.5) as i32;
                    bottom += (border_height as f32 * 1.5) as i32;
                } else if rule.chromium {
                    top -= border_height / 2;
                    left -= border_width * 2;
                    right += border_width * 2;
                    bottom += border_height * 2;
                }
                left += border_width * 2;
                right -= border_width * 2;
                top += border_height * 2;
                bottom -= border_height * 2;
            } else {
                top += border_height * 2;
            }
        }

        let mut rect = RECT {
            left,
            right,
            top,
            bottom,
        };

        //println!("before {}", rect_to_string(rect));

        unsafe {
            AdjustWindowRectEx(
                &mut rect,
                self.style.bits() as u32,
                0,
                self.exstyle.bits() as u32,
            );
        }

        // println!("after {}", rect_to_string(rect));

        rect
    }
    pub fn to_foreground(&self, topmost: bool) -> Result<(), util::WinApiResultError> {
        unsafe {
            util::winapi_nullable_to_result(SetWindowPos(
                self.id as HWND,
                if topmost { HWND_TOPMOST } else { HWND_TOP },
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE,
            ))?;
        }

        Ok(())
    }
    pub fn remove_topmost(&self) -> Result<(), util::WinApiResultError> {
        unsafe {
            util::winapi_nullable_to_result(SetWindowPos(
                self.id as HWND,
                HWND_NOTOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE,
            ))?;
        }

        Ok(())
    }
    /**
     * This also brings the window to the foreground
     */
    pub fn focus(&self) -> Result<(), util::WinApiResultError> {
        unsafe {
            SetForegroundWindow(self.id as HWND);
        }

        Ok(())
    }
    pub fn get_process_name(&self) -> String {
        self.get_process_path()
            .split('\\')
            .last()
            .unwrap()
            .to_string()
    }
    pub fn get_process_path(&self) -> String {
        let mut buffer = [0; 0x200];

        unsafe {
            let mut process_id = 0;
            GetWindowThreadProcessId(self.id as HWND, &mut process_id);
            let process_handle =
                OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, process_id);

            if process_handle as i32 == 0 {
                error!("winapi: {}", GetLastError());
            }
            if GetModuleFileNameExA(
                process_handle,
                std::ptr::null_mut(),
                buffer.as_mut_ptr(),
                buffer.len() as u32,
            ) == 0
            {
                error!("winapi: {}", GetLastError());
            };
        }

        util::bytes_to_string(&buffer)
    }
    pub fn close(&self) {
        unsafe {
            SendMessageA(self.id as HWND, WM_CLOSE, 0, 0);
        }
    }
    pub fn update_style(&self) {
        unsafe {
            SetWindowLongA(self.id as HWND, GWL_STYLE, self.style.bits());
        }
    }
    pub fn update_exstyle(&self) {
        unsafe {
            SetWindowLongA(self.id as HWND, GWL_EXSTYLE, self.exstyle.bits());
        }
    }
    pub fn remove_title_bar(&mut self) {
        let rule = self.rule.clone().unwrap_or_default();
        if !rule.chromium && !rule.firefox {
            self.style.remove(GwlStyle::CAPTION);
            self.style.remove(GwlStyle::THICKFRAME);
        }
        if CONFIG.lock().unwrap().use_border {
            self.style.insert(GwlStyle::BORDER);
        }
    }

    pub fn maximize(&self) {
        unsafe {
            SendMessageA(self.id as HWND, WM_SYSCOMMAND, SC_MAXIMIZE, 0);
        }
    }

    pub fn minimize(&self) {
        unsafe {
            SendMessageA(self.id as HWND, WM_SYSCOMMAND, SC_MINIMIZE, 0);
        }
    }

    pub fn redraw(&self) {
        unsafe {
            SendMessageA(self.id as HWND, WM_PAINT, 0, 0);
        }
    }

    pub fn restore(&self) {
        unsafe {
            SendMessageA(self.id as HWND, WM_SYSCOMMAND, SC_RESTORE, 0);
        }
    }
}
