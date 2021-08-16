// SPDX-License-Identifier: MPL-2.0

use super::raw::{RECV_REAL_PACKET, XM_RECEIVE_REAL_DATA};
use super::{executor::Executor, window::Window};
use crate::data::{self, error::DecodeError};
use crate::error::{RecvError, RecvTimeoutError, TryRecvError, Win32Error};
use crate::{euckr, response::RealResponse};

use crossbeam_channel::{Receiver, Sender};
use lazy_static::lazy_static;
use xingapi_res::TrLayout;

use std::sync::{atomic::AtomicPtr, Arc};
use std::{collections::HashMap, time::Duration};

use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::HWND;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::winuser::{
    DefWindowProcA, GetWindowLongPtrA, RegisterClassExA, SetWindowLongPtrA, GWLP_USERDATA,
    WM_DESTROY, WNDCLASSEXA,
};

lazy_static! {
    static ref REAL_WNDCLASS: Vec<i8> = {
        let class_name: Vec<i8> = b"xingapi_real\0".iter().map(|&c| c as i8).collect();

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

impl IncompleteResponse {
    fn into_real_res(self, tr_layouts: &HashMap<String, TrLayout>) -> RealResponse {
        RealResponse {
            key: self.key,
            reg_key: self.reg_key,
            data: if let Some(layout) = tr_layouts.get(&self.tr_code) {
                data::decode_non_block(layout, &self.data)
            } else {
                Err(DecodeError::UnknownLayout)
            },
        }
    }
}

struct WindowData {
    tx_res: Sender<IncompleteResponse>,
}

impl WindowData {
    fn new(window: &Window, tx_res: Sender<IncompleteResponse>) -> AtomicPtr<Self> {
        let mut data = AtomicPtr::new(Box::into_raw(Box::new(WindowData { tx_res })));
        unsafe {
            SetWindowLongPtrA(**window as _, GWLP_USERDATA, *data.get_mut() as _);
        }

        data
    }
}

pub struct RealWindow {
    executor: Arc<Executor>,
    tr_layouts: Arc<HashMap<String, TrLayout>>,
    window: Window,
    _window_data: AtomicPtr<WindowData>,
    rx_res: Receiver<IncompleteResponse>,
}

impl RealWindow {
    pub fn new(
        executor: Arc<Executor>,
        tr_layouts: Arc<HashMap<String, TrLayout>>,
    ) -> Result<Self, Win32Error> {
        let window = Window::new(executor.clone(), &REAL_WNDCLASS)?;

        let (tx_res, rx_res) = crossbeam_channel::unbounded();
        let _window_data = WindowData::new(&window, tx_res);

        Ok(Self { executor, tr_layouts, window, _window_data, rx_res })
    }

    pub fn subscribe<T: AsRef<str>>(&self, tr_code: &str, tickers: &[T]) -> Result<(), ()> {
        self.executor.handle().advise_real_data(
            *self.window,
            tr_code,
            tickers.iter().map(|t| t.as_ref().into()).collect(),
        )
    }

    pub fn unsubscribe<T: AsRef<str>>(&self, tr_code: &str, tickers: &[T]) -> Result<(), ()> {
        self.executor.handle().unadvise_real_data(
            *self.window,
            tr_code,
            tickers.iter().map(|t| t.as_ref().into()).collect(),
        )
    }

    pub fn unsubscribe_all(&self) -> Result<(), ()> {
        self.executor.unadvise_window(*self.window)
    }

    pub fn try_recv(&self) -> Result<RealResponse, TryRecvError> {
        Ok(self.rx_res.try_recv()?.into_real_res(&self.tr_layouts))
    }

    pub fn recv(&self) -> Result<RealResponse, RecvError> {
        Ok(self.rx_res.recv()?.into_real_res(&self.tr_layouts))
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Result<RealResponse, RecvTimeoutError> {
        Ok(self.rx_res.recv_timeout(timeout)?.into_real_res(&self.tr_layouts))
    }

    unsafe extern "system" fn wndproc(
        hwnd: HWND,
        msg: UINT,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        debug_assert!(Executor::is_executor_thread());

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
                let _ = window_data.tx_res.send(IncompleteResponse {
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
        let _ = self.executor.unadvise_window(*self.window);
    }
}

impl From<crossbeam_channel::TryRecvError> for TryRecvError {
    fn from(err: crossbeam_channel::TryRecvError) -> Self {
        match err {
            crossbeam_channel::TryRecvError::Empty => Self::Empty,
            crossbeam_channel::TryRecvError::Disconnected => Self::Disconnected,
        }
    }
}

impl From<crossbeam_channel::RecvError> for RecvError {
    fn from(_: crossbeam_channel::RecvError) -> Self {
        Self
    }
}

impl From<crossbeam_channel::RecvTimeoutError> for RecvTimeoutError {
    fn from(err: crossbeam_channel::RecvTimeoutError) -> Self {
        match err {
            crossbeam_channel::RecvTimeoutError::Timeout => Self::Timeout,
            crossbeam_channel::RecvTimeoutError::Disconnected => Self::Disconnected,
        }
    }
}
