use crate::utils::get_window_exe;
use anyhow::{bail, Result};
use once_cell::sync::OnceCell;
use std::collections::HashSet;
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::{
        Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
        WindowsAndMessaging::{
            EVENT_SYSTEM_FOREGROUND, PostMessageW, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
        },
    },
};

pub static mut IS_FOREGROUND_IN_BLACKLIST: bool = false;

static BLACKLIST: OnceCell<HashSet<String>> = OnceCell::new();
static mut APP_HWND: Option<HWND> = None;

#[derive(Debug)]
pub struct ForegroundWatcher {
    hook: HWINEVENTHOOK,
}

impl ForegroundWatcher {
    pub fn init(blacklist: &HashSet<String>, hwnd: Option<HWND>) -> Result<Self> {
        unsafe { APP_HWND = hwnd; }
        if blacklist.is_empty() {
            return Ok(Self {
                hook: HWINEVENTHOOK::default(),
            });
        }
        if blacklist.is_empty() {
            return Ok(Self {
                hook: HWINEVENTHOOK::default(),
            });
        }

        let _ = BLACKLIST.set(blacklist.iter().map(|v| v.to_lowercase()).collect());

        let hook = unsafe {
            SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                None,
                Some(win_event_proc),
                0,
                0,
                WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
            )
        };
        if hook.is_invalid() {
            bail!("Failed to watch foreground");
        }

        info!("foreground watcher start");

        Ok(Self { hook })
    }
}

impl Drop for ForegroundWatcher {
    fn drop(&mut self) {
        debug!("foreground watcher destroyed");
        if !self.hook.is_invalid() {
            unsafe {
                let _ = UnhookWinEvent(self.hook);
            }
        }
    }
}

unsafe extern "system" fn win_event_proc(
    _h_win_event_hook: HWINEVENTHOOK,
    _event: u32,
    hwnd: HWND,
    _id_object: i32,
    _id_child: i32,
    _dw_event_thread: u32,
    _dwms_event_time: u32,
) {
    let exe = match get_window_exe(hwnd) {
        Some(v) => v.to_lowercase(),
        None => return,
    };
    let is_in_blacklist = BLACKLIST.get().unwrap().contains(&exe);
    IS_FOREGROUND_IN_BLACKLIST = is_in_blacklist;
    if !is_in_blacklist {
        if let Some(app_hwnd) = unsafe { APP_HWND } {
            unsafe {
                let _ = PostMessageW(app_hwnd, 6030, WPARAM(0), LPARAM(hwnd.0 as _));
            }
        }
    }
    debug!("foreground {exe} {is_in_blacklist}");
}
