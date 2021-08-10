// SPDX-License-Identifier: MPL-2.0

use super::raw::{XM_DISCONNECT, XM_LOGIN, XM_LOGOUT};
use super::{caller::Caller, window::Window};
use crate::error::{Error, Win32Error};
use crate::response::LoginResponse;

use encoding_rs::EUC_KR;
use lazy_static::lazy_static;

use std::ffi::CStr;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::mpsc::{self, SyncSender};
use std::sync::{Arc, RwLock};

use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::HWND;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::winuser::{
    DefWindowProcA, GetWindowLongPtrA, RegisterClassExA, SetWindowLongPtrA, GWLP_USERDATA,
    WM_DESTROY, WNDCLASSEXA,
};

lazy_static! {
    static ref SESSION_WNDCLASS: Vec<i8> = {
        let class_name: Vec<i8> = b"xingapi_session\0".iter().map(|&c| c as i8).collect();

        unsafe {
            RegisterClassExA(&WNDCLASSEXA {
                cbSize: std::mem::size_of::<WNDCLASSEXA>() as UINT,
                lpfnWndProc: Some(SessionWindow::wndproc),
                cbWndExtra: std::mem::size_of::<usize>() as _,
                hInstance: GetModuleHandleA(std::ptr::null()),
                lpszClassName: class_name.as_ptr(),
                ..std::mem::zeroed()
            });
        }

        class_name
    };
}

struct WindowData {
    tx_res: Option<SyncSender<Option<LoginResponse>>>,
}

impl WindowData {
    fn empty() -> Self {
        WindowData { tx_res: None }
    }

    fn new(window: &Window) -> AtomicPtr<RwLock<Self>> {
        let data = RwLock::new(WindowData::empty());
        let mut ptr = AtomicPtr::new(Box::into_raw(Box::new(data)));
        unsafe {
            SetWindowLongPtrA(**window as _, GWLP_USERDATA, *ptr.get_mut() as _);
        }

        ptr
    }
}

pub struct SessionWindow {
    caller: Arc<Caller>,
    window: Window,
    window_data: AtomicPtr<RwLock<WindowData>>,
}

impl SessionWindow {
    pub(crate) fn new(caller: Arc<Caller>) -> Result<Self, Win32Error> {
        let window = Window::new(caller.clone(), &SESSION_WNDCLASS)?;
        let window_data = WindowData::new(&window);

        Ok(Self { caller, window, window_data })
    }

    pub fn connect(
        &self,
        addr: &str,
        port: u16,
        timeout: Option<i32>,
        packet_len_limit: Option<i32>,
    ) -> Result<(), Error> {
        self.caller.sync_handle().connect(*self.window, addr, port, timeout, packet_len_limit)
    }

    pub fn login(
        &self,
        id: &str,
        pw: &str,
        cert_pw: &str,
        cert_err_dialog: bool,
    ) -> Result<LoginResponse, Error> {
        let handle = self.caller.sync_handle();
        let window_data = unsafe { &mut *self.window_data.load(Ordering::Relaxed) };

        let (tx_res, rx_res) = mpsc::sync_channel(1);
        *window_data.write().unwrap() = WindowData { tx_res: Some(tx_res) };

        handle.login(*self.window, id, pw, cert_pw, cert_err_dialog)?;
        let result = rx_res.recv().unwrap();

        *window_data.write().unwrap() = WindowData::empty();

        if let Some(res) = result {
            Ok(res)
        } else {
            Err(Error::TimedOut)
        }
    }

    extern "system" fn wndproc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        debug_assert!(Caller::is_caller_thread());

        match msg {
            WM_DESTROY => unsafe {
                let window_data = GetWindowLongPtrA(hwnd, GWLP_USERDATA) as *mut RwLock<WindowData>;
                assert_ne!(window_data, std::ptr::null_mut());
                drop(Box::from_raw(window_data));

                0
            },
            XM_DISCONNECT | XM_LOGIN | XM_LOGOUT => {
                let window_data = unsafe {
                    let ptr = GetWindowLongPtrA(hwnd, GWLP_USERDATA) as *const RwLock<WindowData>;
                    assert_ne!(ptr, std::ptr::null());
                    (*ptr).read().unwrap()
                };

                match msg {
                    XM_DISCONNECT => {
                        if let Some(tx) = &window_data.tx_res {
                            let _ = tx.try_send(None);
                        }
                    }
                    XM_LOGIN => {
                        if let Some(tx) = &window_data.tx_res {
                            let code = EUC_KR
                                .decode(unsafe { CStr::from_ptr(wparam as _) }.to_bytes())
                                .0
                                .trim_end()
                                .to_owned();
                            let message = EUC_KR
                                .decode(unsafe { CStr::from_ptr(lparam as _) }.to_bytes())
                                .0
                                .trim_end()
                                .to_owned();

                            let _ = tx.try_send(Some(LoginResponse { code, message }));
                        }
                    }
                    XM_LOGOUT => {}
                    _ => unreachable!(),
                }

                0
            }
            _ => unsafe { DefWindowProcA(hwnd, msg, wparam, lparam) },
        }
    }
}

impl UnwindSafe for SessionWindow {}
impl RefUnwindSafe for SessionWindow {}
