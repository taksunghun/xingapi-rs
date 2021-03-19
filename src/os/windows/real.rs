// SPDX-License-Identifier: MPL-2.0

use super::{
    caller::Caller,
    error::Win32Error,
    raw::{RECV_REAL_PACKET, XM_RECEIVE_REAL_DATA},
    win32::Window,
    XingApi,
};
use crate::{
    data::{self, error::DecodeError},
    error::Error,
    euckr,
    response::RealResponse,
};

use lazy_static::lazy_static;
use std::sync::{atomic::AtomicPtr, Arc};

use winapi::{
    shared::{
        minwindef::{LPARAM, LRESULT, UINT, WPARAM},
        windef::HWND,
    },
    um::{
        libloaderapi::GetModuleHandleA,
        winuser::{
            DefWindowProcA, GetWindowLongPtrA, SetWindowLongPtrA, GWLP_USERDATA, WM_DESTROY,
        },
    },
};

lazy_static! {
    static ref REAL_WNDCLASS: Vec<i8> = {
        use winapi::um::winuser::{RegisterClassExA, WNDCLASSEXA};

        let class_name: Vec<i8> =
            b"xingapi::real::REAL_WNDCLASS\0".iter().map(|&c| c as i8).collect();

        unsafe {
            RegisterClassExA(&WNDCLASSEXA {
                cbSize: std::mem::size_of::<WNDCLASSEXA>() as UINT,
                lpfnWndProc: Some(RealWindow::wndproc),
                cbWndExtra: std::mem::size_of::<usize>() as _,
                hInstance: GetModuleHandleA(std::ptr::null()),
                lpszClassName: class_name.as_ptr(),
                ..std::mem::zeroed()
            });
        }

        class_name
    };
}

struct IncompleteResponse {
    tr_code: String,
    key: String,
    reg_key: String,
    data: Vec<u8>,
}

struct WindowData {
    tx_res: async_channel::Sender<IncompleteResponse>,
}

impl WindowData {
    fn new(window: &Window, tx_res: async_channel::Sender<IncompleteResponse>) -> AtomicPtr<Self> {
        let mut data = AtomicPtr::new(Box::into_raw(Box::new(WindowData { tx_res })));
        unsafe {
            SetWindowLongPtrA(window.handle(), GWLP_USERDATA, *data.get_mut() as _);
        }

        data
    }
}

pub struct RealWindow {
    xingapi: Arc<XingApi>,
    window: Arc<Window>,
    _window_data: AtomicPtr<WindowData>,
    rx_res: async_channel::Receiver<IncompleteResponse>,
}

impl RealWindow {
    pub(crate) async fn new(xingapi: Arc<XingApi>) -> Result<Self, Win32Error> {
        let window = Window::new(xingapi.caller.clone(), &REAL_WNDCLASS).await?;

        let (tx_res, rx_res) = async_channel::unbounded();
        let _window_data = WindowData::new(&window, tx_res);

        Ok(Self { xingapi, window, _window_data, rx_res })
    }

    pub async fn register(&self, tr_code: &str, data: Vec<String>) -> Result<(), Error> {
        data.iter().for_each(|ticker| assert!(ticker.is_ascii()));

        let handle = self.xingapi.caller.handle().read().await;
        handle.advise_real_data(self.window.clone(), tr_code, data).await
    }

    pub async fn unregister(&self, tr_code: &str, data: Vec<String>) -> Result<(), Error> {
        data.iter().for_each(|ticker| assert!(ticker.is_ascii()));

        let handle = self.xingapi.caller.handle().read().await;
        handle.unadvise_real_data(self.window.clone(), tr_code, data).await
    }

    pub async fn unregister_all(&self) -> Result<(), Error> {
        self.xingapi.caller.unadvise_window(self.window.clone()).await
    }

    pub async fn recv(&self) -> RealResponse {
        let res = self.rx_res.recv().await.unwrap();
        RealResponse::new(
            res.key,
            res.reg_key,
            if let Some(layout) = self.xingapi.tr_layouts.get(&res.tr_code) {
                data::decode_non_block(layout, &res.data)
            } else {
                Err(DecodeError::UnknownTrCode)
            },
        )
    }

    pub fn try_recv(&self) -> Option<RealResponse> {
        if let Ok(res) = self.rx_res.try_recv() {
            Some(RealResponse::new(
                res.key,
                res.reg_key,
                if let Some(layout) = self.xingapi.tr_layouts.get(&res.tr_code) {
                    data::decode_non_block(layout, &res.data)
                } else {
                    Err(DecodeError::UnknownTrCode)
                },
            ))
        } else {
            None
        }
    }

    unsafe extern "system" fn wndproc(
        hwnd: HWND,
        msg: UINT,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        debug_assert!(Caller::is_caller_thread());

        match msg {
            WM_DESTROY => {
                let window_data = GetWindowLongPtrA(hwnd, GWLP_USERDATA) as *mut WindowData;
                assert_ne!(window_data, std::ptr::null_mut());
                drop(Box::from_raw(window_data));

                0
            }
            XM_RECEIVE_REAL_DATA => {
                let window_data = {
                    let ptr = GetWindowLongPtrA(hwnd, GWLP_USERDATA) as *mut WindowData;
                    assert_ne!(ptr, std::ptr::null_mut());
                    &mut *ptr
                };

                let packet = &*(lparam as *const RECV_REAL_PACKET);
                let _ = window_data.tx_res.try_send(IncompleteResponse {
                    tr_code: euckr::decode(&packet.tr_code).to_string(),
                    key: euckr::decode(&packet.key).to_string(),
                    reg_key: euckr::decode(&packet.reg_key).to_string(),
                    data: std::slice::from_raw_parts(packet.data, packet.data_len as _).into(),
                });

                0
            }
            _ => DefWindowProcA(hwnd, msg, wparam, lparam),
        }
    }
}

impl Drop for RealWindow {
    fn drop(&mut self) {
        self.xingapi.caller.unadvise_window(self.window.clone());
    }
}
