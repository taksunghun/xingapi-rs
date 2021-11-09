// SPDX-License-Identifier: MPL-2.0

use crate::data::{self, Data, RawData};
use crate::layout::TrLayout;

use super::executor::{self, Executor, Window};
use super::raw::{MSG_PACKET, RECV_PACKET};
use super::raw::{XM_DISCONNECT, XM_LOGIN, XM_LOGOUT, XM_RECEIVE_DATA, XM_TIMEOUT};
use super::{decode_euckr, Error, LoginResponse, QueryResponse};

use array_init::array_init;
use lazy_static::lazy_static;

use std::ffi::{CStr, CString};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError, SyncSender};
use std::sync::{Mutex, RwLock, RwLockReadGuard};
use std::{cmp::Ord, collections::HashMap, time::Duration};

use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::HWND;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::winuser::{
    DefWindowProcA, GetWindowLongPtrA, RegisterClassExA, SetWindowLongPtrA, GWLP_USERDATA,
    WM_DESTROY, WNDCLASSEXA,
};

lazy_static! {
    static ref GLOBAL_SESSION: RwLock<Option<Session>> = RwLock::new(None);
}

pub(crate) struct GlobalSession {
    guard: RwLockReadGuard<'static, Option<Session>>,
}

impl std::ops::Deref for GlobalSession {
    type Target = Session;
    fn deref(&self) -> &Self::Target {
        self.guard.as_ref().unwrap()
    }
}

pub(crate) fn global() -> GlobalSession {
    let guard = GLOBAL_SESSION.read().unwrap();
    assert!(guard.is_some(), "global session is not loaded");

    GlobalSession { guard }
}

pub(crate) fn load() -> Result<(), std::io::Error> {
    let mut session = GLOBAL_SESSION.write().unwrap();
    if session.is_none() {
        *session = Some(Session::new()?);
    }

    Ok(())
}

pub(crate) fn unload() {
    *GLOBAL_SESSION.write().unwrap() = None;
}

pub(crate) fn is_loaded() -> bool {
    GLOBAL_SESSION.read().unwrap().is_some()
}

lazy_static! {
    static ref SESSION_WNDCLASS: CString = {
        let class_name = CString::new("rust_xingapi_session").unwrap();

        unsafe {
            RegisterClassExA(&WNDCLASSEXA {
                cbSize: std::mem::size_of::<WNDCLASSEXA>() as _,
                lpfnWndProc: Some(Session::window_proc),
                cbWndExtra: std::mem::size_of::<usize>() as _,
                hInstance: GetModuleHandleA(std::ptr::null()),
                lpszClassName: class_name.as_ptr(),
                ..std::mem::zeroed()
            });
        }

        class_name
    };
}

struct IncompleteQueryResponse {
    code: String,
    message: String,
    elapsed_time: Duration,
    next_key: Option<String>,
    data: Option<RawData>,
}

impl IncompleteQueryResponse {
    const fn empty() -> Self {
        Self {
            code: String::new(),
            message: String::new(),
            elapsed_time: Duration::ZERO,
            next_key: None,
            data: None,
        }
    }
}

struct QueryState {
    tr_layout: TrLayout,
    tx_res: SyncSender<IncompleteQueryResponse>,
    res: Option<IncompleteQueryResponse>,
}

struct SessionWindowData {
    tx_login_res: Mutex<Option<SyncSender<LoginResponse>>>,
    state_tbl: [Mutex<Option<QueryState>>; 256],
}

pub(crate) struct Session {
    window: Window,
    window_data: AtomicPtr<SessionWindowData>,
}

impl Session {
    fn window_data(&self) -> &SessionWindowData {
        unsafe { &mut *self.window_data.load(Ordering::Relaxed) }
    }

    pub fn new() -> Result<Self, std::io::Error> {
        let window = Window::new(SESSION_WNDCLASS.clone())?;

        let mut window_data = AtomicPtr::new(Box::into_raw(Box::new(SessionWindowData {
            tx_login_res: Mutex::new(None),
            state_tbl: array_init(|_| Mutex::new(None)),
        })));

        unsafe {
            SetWindowLongPtrA(*window as _, GWLP_USERDATA, *window_data.get_mut() as _);
        }

        Ok(Self {
            window,
            window_data,
        })
    }

    pub fn connect(&self, addr: &str, port: u16, timeout: Duration) -> Result<(), Error> {
        let executor = executor::global();
        let mut handle = executor.lock_handle();

        handle.connect(*self.window, addr, port, timeout)
    }

    pub fn disconnect(&self) {
        executor::global().lock_handle().disconnect()
    }

    pub fn login(
        &self,
        id: &str,
        pw: &str,
        cert_pw: &str,
        cert_err_dialog: bool,
    ) -> Result<LoginResponse, Error> {
        let executor = executor::global();
        let mut handle = executor.lock_handle();

        let window_data = self.window_data();
        let (tx_res, rx_res) = mpsc::sync_channel(1);

        *window_data.tx_login_res.lock().unwrap() = Some(tx_res);

        if let Err(err) = handle.login(*self.window, id, pw, cert_pw, cert_err_dialog) {
            *window_data.tx_login_res.lock().unwrap() = None;
            return Err(err);
        }

        let result = rx_res.recv();

        *window_data.tx_login_res.lock().unwrap() = None;

        match result {
            Ok(res) => Ok(res),
            Err(_) => Err(Error::TimedOut),
        }
    }

    pub fn request(
        &self,
        data: &Data,
        tr_layout: &TrLayout,
        next_key: Option<&str>,
        timeout: Duration,
    ) -> Result<QueryResponse, Error> {
        let executor = executor::global();
        let handle = executor.handle();

        let tr_code = &data.tr_code;
        let enc_data = data::encode(data, tr_layout)?;

        let req_id: usize = handle
            .request(*self.window, tr_code, enc_data, next_key, timeout)?
            .try_into()
            .unwrap();

        let (tx_res, rx_res) = mpsc::sync_channel(1);

        {
            let mut state = self.window_data().state_tbl[req_id].lock().unwrap();
            assert!(state.is_none());

            *state = Some(QueryState {
                tr_layout: tr_layout.clone(),
                tx_res,
                res: None,
            });
        }

        match rx_res.recv_timeout(timeout + Duration::from_millis(100)) {
            Ok(res) => Ok(QueryResponse {
                code: res.code,
                message: res.message,
                elapsed: res.elapsed_time,
                next_key: res.next_key,
                data: res.data.map(|d| data::decode(tr_layout, d)),
            }),
            Err(RecvTimeoutError::Timeout) => {
                *self.window_data().state_tbl[req_id].lock().unwrap() = None;

                Err(Error::TimedOut)
            }
            Err(_) => Err(Error::TimedOut),
        }
    }

    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        msg: UINT,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        debug_assert!(Executor::is_executor_thread());

        let load_window_data = || -> &SessionWindowData {
            let ptr = GetWindowLongPtrA(hwnd, GWLP_USERDATA) as *const _;
            assert_ne!(ptr, std::ptr::null());
            &*ptr
        };

        match msg {
            WM_DESTROY => {
                let ptr = GetWindowLongPtrA(hwnd, GWLP_USERDATA) as *mut SessionWindowData;
                assert_ne!(ptr, std::ptr::null_mut());
                drop(Box::from_raw(ptr));

                0
            }
            XM_DISCONNECT | XM_LOGOUT => {
                *load_window_data().tx_login_res.lock().unwrap() = None;

                0
            }
            XM_LOGIN => {
                if let Some(tx) = load_window_data().tx_login_res.lock().unwrap().take() {
                    let _ = tx.try_send(LoginResponse {
                        code: decode_euckr(CStr::from_ptr(wparam as _).to_bytes()),
                        message: decode_euckr(CStr::from_ptr(lparam as _).to_bytes()),
                    });
                }

                0
            }
            XM_RECEIVE_DATA => {
                let req_id: usize = match wparam {
                    1 => { &*(lparam as *const RECV_PACKET) }.req_id,
                    2 | 3 => { &*(lparam as *const MSG_PACKET) }.req_id,
                    4 => lparam.try_into().unwrap(),
                    _ => unreachable!(),
                }
                .try_into()
                .unwrap();

                // RECV_PACKET보다 MSG_PACKET이 먼저 수신될 수도 있습니다.
                match wparam {
                    1 => {
                        let recv_packet = &*(lparam as *const RECV_PACKET);

                        let mut state_guard = load_window_data().state_tbl[req_id].lock().unwrap();
                        let state = state_guard.as_mut().unwrap();
                        let res = state.res.get_or_insert(IncompleteQueryResponse::empty());

                        res.elapsed_time = Ord::max(
                            res.elapsed_time,
                            Duration::from_millis(recv_packet.elapsed_time.try_into().unwrap()),
                        );

                        match decode_euckr(&recv_packet.next_key) {
                            key if key.is_empty() => {}
                            key => res.next_key = Some(key),
                        }

                        assert!(!recv_packet.data.is_null());

                        let raw_data = std::slice::from_raw_parts(
                            recv_packet.data,
                            recv_packet.data_len.try_into().unwrap(),
                        )
                        .to_owned();

                        // 블록 모드 여부는 레이아웃에서 확인해야 정확합니다.
                        if state.tr_layout.block_mode {
                            if let RawData::Block(block_tbl) = res
                                .data
                                .get_or_insert_with(|| RawData::Block(HashMap::new()))
                            {
                                block_tbl.insert(decode_euckr(&recv_packet.block_name), raw_data);
                            } else {
                                unreachable!();
                            }
                        } else {
                            res.data = Some(RawData::NonBlock(raw_data));
                        }
                    }
                    2 => {
                        let msg_packet = &*(lparam as *const MSG_PACKET);

                        let mut state_guard = load_window_data().state_tbl[req_id].lock().unwrap();
                        let state = state_guard.as_mut().unwrap();
                        let res = state.res.get_or_insert(IncompleteQueryResponse::empty());

                        res.code = decode_euckr(&msg_packet.msg_code);
                        res.message = decode_euckr(std::slice::from_raw_parts(
                            msg_packet.msg_data,
                            msg_packet.msg_data_len.try_into().unwrap(),
                        ));

                        executor::global().entry().release_message_data(lparam);
                    }
                    3 => {
                        executor::global().entry().release_message_data(lparam);
                    }
                    4 => {
                        let mut state_guard = load_window_data().state_tbl[req_id].lock().unwrap();
                        let state = state_guard.as_mut().unwrap();

                        let _ = state.tx_res.send(state.res.take().unwrap());
                        *state_guard = None;

                        executor::global().entry().release_request_data(req_id as _);
                    }
                    _ => unreachable!(),
                }

                0
            }
            XM_TIMEOUT => {
                let req_id: usize = lparam.try_into().unwrap();
                *load_window_data().state_tbl[req_id].lock().unwrap() = None;

                0
            }
            _ => DefWindowProcA(hwnd, msg, wparam, lparam),
        }
    }
}
