// SPDX-License-Identifier: MPL-2.0

use crate::data::{self, DataType, DecodeError};
use crate::layout::TrLayout;

use super::executor::{self, Executor, Window};
use super::raw::{RECV_REAL_PACKET, XM_RECEIVE_REAL_DATA};
use super::{decode_euckr, RealResponse};

use crossbeam_channel::{Receiver, Sender};
use lazy_static::lazy_static;
use std::sync::{atomic::AtomicPtr, RwLock};
use std::{collections::HashMap, ffi::CString, time::Duration};

use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::HWND;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::winuser::{
    DefWindowProcA, GetWindowLongPtrA, RegisterClassExA, SetWindowLongPtrA, GWLP_USERDATA,
    WM_DESTROY, WNDCLASSEXA,
};

lazy_static! {
    static ref REAL_EVENT_WNDCLASS: CString = {
        let class_name = CString::new("rust_xingapi_event_real").unwrap();

        unsafe {
            RegisterClassExA(&WNDCLASSEXA {
                cbSize: std::mem::size_of::<WNDCLASSEXA>() as _,
                lpfnWndProc: Some(RealEvent::window_proc),
                cbWndExtra: std::mem::size_of::<usize>() as _,
                hInstance: GetModuleHandleA(std::ptr::null()),
                lpszClassName: class_name.as_ptr(),
                ..std::mem::zeroed()
            });
        }

        class_name
    };
}

struct IncompleteRealResponse {
    tr_code: String,
    key: String,
    data: Vec<u8>,
}

impl IncompleteRealResponse {
    fn decode(self, layout_tbl: &HashMap<String, TrLayout>) -> RealResponse {
        RealResponse {
            key: self.key,
            data: (|| -> Result<_, DecodeError> {
                data::decode_non_block(
                    layout_tbl
                        .get(&self.tr_code)
                        .ok_or_else(|| DecodeError::UnknownLayout(self.tr_code.clone()))?,
                    DataType::Output,
                    &self.data,
                )
            })(),
        }
    }
}

struct RealEventWindowData {
    tx_res: Sender<IncompleteRealResponse>,
}

/// 실시간 TR을 등록하고 수신하는 객체
///
/// 실시간 TR을 등록한 이후에는 수신한 응답을 `try_recv()`나 `recv_timeout()`
/// 함수를 호출하여 지속적으로 큐에서 가져와야 합니다. 그렇지 않을 경우 메모리
/// 누수로 이어질 수 있습니다.
pub struct RealEvent {
    window: Window,
    _window_data: AtomicPtr<RealEventWindowData>,
    layout_tbl: RwLock<HashMap<String, TrLayout>>,
    rx_res: Receiver<IncompleteRealResponse>,
}

impl RealEvent {
    /// 객체를 생성합니다.
    pub fn new() -> Result<Self, std::io::Error> {
        let window = Window::new(REAL_EVENT_WNDCLASS.clone())?;

        let layout_tbl = RwLock::new(HashMap::new());
        let (tx_res, rx_res) = crossbeam_channel::unbounded();

        let mut _window_data =
            AtomicPtr::new(Box::into_raw(Box::new(RealEventWindowData { tx_res })));

        unsafe {
            SetWindowLongPtrA(*window as _, GWLP_USERDATA, *_window_data.get_mut() as _);
        }

        Ok(Self {
            window,
            _window_data,
            layout_tbl,
            rx_res,
        })
    }

    /// 응답을 디코딩하기 위한 레이아웃을 추가합니다.
    pub fn insert_layout(&self, tr_layout: TrLayout) {
        self.layout_tbl
            .write()
            .unwrap()
            .insert(tr_layout.code.clone(), tr_layout);
    }

    /// 응답을 디코딩하기 위한 레이아웃을 삭제합니다.
    pub fn remove_layout(&self, tr_code: &str) {
        self.layout_tbl.write().unwrap().remove(tr_code);
    }

    /// 실시간 TR을 지정된 키들로 등록합니다.
    pub fn subscribe<T: AsRef<str>>(&self, tr_code: &str, keys: &[T]) {
        executor::global().handle().advise_real_data(
            *self.window,
            tr_code,
            keys.iter().map(|k| k.as_ref().to_owned()).collect(),
        );
    }

    /// 실시간 TR을 지정된 키들로 등록 해제합니다.
    pub fn unsubscribe<T: AsRef<str>>(&self, tr_code: &str, keys: &[T]) {
        executor::global().handle().unadvise_real_data(
            *self.window,
            tr_code,
            keys.iter().map(|k| k.as_ref().to_owned()).collect(),
        );
    }

    /// 실시간 TR을 모두 등록 해제합니다.
    pub fn unsubscribe_all(&self) {
        executor::global().unadvise_window(*self.window);
    }

    /// 수신한 응답이 큐에 있는 경우 가져옵니다.
    pub fn try_recv(&self) -> Option<RealResponse> {
        if let Ok(res) = self.rx_res.try_recv() {
            Some(res.decode(&self.layout_tbl.read().unwrap()))
        } else {
            None
        }
    }

    /// 수신한 응답을 큐에서 가져올 때까지 지정된 시간 동안 기다립니다.
    pub fn recv_timeout(&self, timeout: Duration) -> Option<RealResponse> {
        if let Ok(res) = self.rx_res.recv_timeout(timeout) {
            Some(res.decode(&self.layout_tbl.read().unwrap()))
        } else {
            None
        }
    }

    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        msg: UINT,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        debug_assert!(Executor::is_executor_thread());

        match msg {
            WM_DESTROY => {
                let ptr = GetWindowLongPtrA(hwnd, GWLP_USERDATA) as *mut RealEventWindowData;
                assert!(!ptr.is_null());
                drop(Box::from_raw(ptr));

                0
            }
            XM_RECEIVE_REAL_DATA => {
                let window_data = {
                    let ptr = GetWindowLongPtrA(hwnd, GWLP_USERDATA) as *mut RealEventWindowData;
                    assert!(!ptr.is_null());
                    &mut *ptr
                };

                let packet = &*(lparam as *const RECV_REAL_PACKET);

                assert!(!packet.data.is_null());
                assert!(packet.data_len >= 0);

                let _ = window_data.tx_res.send(IncompleteRealResponse {
                    tr_code: decode_euckr(&packet.tr_code),
                    key: decode_euckr(&packet.key),
                    data: std::slice::from_raw_parts(
                        packet.data,
                        packet.data_len.try_into().unwrap(),
                    )
                    .to_owned(),
                });

                0
            }
            _ => DefWindowProcA(hwnd, msg, wparam, lparam),
        }
    }
}

impl Drop for RealEvent {
    fn drop(&mut self) {
        if let Some(executor) = &*executor::GLOBAL_EXECUTOR.read().unwrap() {
            executor.unadvise_window(*self.window);
        }
    }
}
