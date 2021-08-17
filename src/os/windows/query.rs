// SPDX-License-Identifier: MPL-2.0

use super::raw::{MSG_PACKET, RECV_PACKET, XM_RECEIVE_DATA, XM_TIMEOUT};
use super::{executor::Executor, window::Window};
use crate::data::{self, Data, RawData};
use crate::error::{Error, Win32Error};
use crate::{euckr, response::QueryResponse};

use array_init::array_init;
use lazy_static::lazy_static;
use xingapi_res::TrLayout;

use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::mpsc::{self, SyncSender};
use std::sync::{Arc, Mutex, Weak};
use std::{collections::HashMap, iter::FromIterator, ops::DerefMut};

use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::HWND;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::winuser::{
    DefWindowProcA, GetWindowLongPtrA, RegisterClassExA, SetWindowLongPtrA, GWLP_USERDATA,
    WM_DESTROY, WNDCLASSEXA,
};

lazy_static! {
    static ref QUERY_WNDCLASS: Vec<i8> = {
        let class_name: Vec<i8> = b"xingapi_query\0".iter().map(|&c| c as i8).collect();

        unsafe {
            RegisterClassExA(&WNDCLASSEXA {
                cbSize: std::mem::size_of::<WNDCLASSEXA>() as _,
                lpfnWndProc: Some(QueryWindow::wndproc),
                cbWndExtra: std::mem::size_of::<usize>() as _,
                hInstance: GetModuleHandleA(std::ptr::null()),
                lpszClassName: class_name.as_ptr(),
                ..std::mem::zeroed()
            });
        }

        class_name
    };
}

type TxResponse = SyncSender<Option<IncompleteResponse>>;

struct IncompleteResponse {
    code: String,
    message: String,
    elapsed_time: i32,
    continue_key: Option<String>,
    data: Option<RawData>,
}

impl IncompleteResponse {
    fn empty() -> Self {
        Self {
            code: String::new(),
            message: String::new(),
            elapsed_time: 0,
            continue_key: None,
            data: None,
        }
    }
}

struct WindowData {
    executor: Weak<Executor>,
    tr_layouts: Weak<HashMap<String, TrLayout>>,
    res_tbl: [Option<IncompleteResponse>; 256],
    tx_res_tbl: [Mutex<Option<TxResponse>>; 256],
}

impl WindowData {
    fn new(
        window: &Window,
        executor: &Arc<Executor>,
        tr_layouts: &Arc<HashMap<String, TrLayout>>,
    ) -> AtomicPtr<Self> {
        let mut data = AtomicPtr::new(Box::into_raw(Box::new(WindowData {
            executor: Arc::downgrade(executor),
            tr_layouts: Arc::downgrade(tr_layouts),
            res_tbl: array_init(|_| None),
            tx_res_tbl: array_init(|_| Mutex::new(None)),
        })));

        unsafe {
            SetWindowLongPtrA(**window as _, GWLP_USERDATA, *data.get_mut() as _);
        }

        data
    }
}

pub struct QueryWindow {
    executor: Arc<Executor>,
    tr_layouts: Arc<HashMap<String, TrLayout>>,
    window: Window,
    window_data: AtomicPtr<WindowData>,
}

impl QueryWindow {
    pub fn new(
        executor: Arc<Executor>,
        tr_layouts: Arc<HashMap<String, TrLayout>>,
    ) -> Result<Self, Win32Error> {
        let window = Window::new(executor.clone(), &QUERY_WNDCLASS)?;
        let window_data = WindowData::new(&window, &executor, &tr_layouts);

        Ok(Self { executor, tr_layouts, window, window_data })
    }

    pub fn request(
        &self,
        data: &Data,
        continue_key: Option<&str>,
        timeout: Option<i32>,
    ) -> Result<QueryResponse, Error> {
        let tr_code = &data.code;
        let req_id = self.executor.handle().request(
            *self.window,
            tr_code,
            data::encode(&self.tr_layouts, data)?,
            continue_key,
            timeout,
        )?;

        let (tx_res, rx_res) = mpsc::sync_channel(1);

        {
            let window_data = unsafe { &mut *self.window_data.load(Ordering::Relaxed) };

            let mut tx_res_ref = window_data.tx_res_tbl[req_id as usize].lock().unwrap();
            assert!(tx_res_ref.is_none());
            *tx_res_ref.deref_mut() = Some(tx_res);
        }

        if let Ok(Some(res)) = rx_res.recv() {
            Ok(QueryResponse::new(
                &res.code,
                &res.message,
                res.elapsed_time,
                res.continue_key,
                res.data.map(|d| data::decode(&self.tr_layouts, tr_code, d)),
            ))
        } else {
            Err(Error::TimedOut)
        }
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
            XM_RECEIVE_DATA => {
                let window_data = {
                    let ptr = GetWindowLongPtrA(hwnd, GWLP_USERDATA) as *mut WindowData;
                    assert_ne!(ptr, std::ptr::null_mut());
                    &mut *ptr
                };

                let executor = &*window_data.executor.as_ptr();
                let layout_tbl = &*window_data.tr_layouts.as_ptr();

                let req_id = match wparam {
                    1 => { &*(lparam as *const RECV_PACKET) }.req_id,
                    2 | 3 => { &*(lparam as *const MSG_PACKET) }.req_id,
                    4 => lparam as _,
                    _ => unreachable!(),
                } as usize;

                if window_data.res_tbl[req_id].is_none() {
                    window_data.res_tbl[req_id] = Some(IncompleteResponse::empty());
                }

                // RECV_PACKET보다 MSG_PACKET이 먼저 수신되는 경우도 있습니다.
                match wparam {
                    1 => {
                        let recv_packet = &*(lparam as *const RECV_PACKET);
                        let tr_code = euckr::decode(&recv_packet.tr_code);
                        let continue_key = euckr::decode(&recv_packet.continue_key);

                        let res = window_data.res_tbl[req_id].as_mut().unwrap();

                        if res.elapsed_time < recv_packet.elapsed_time {
                            res.elapsed_time = recv_packet.elapsed_time;
                        }

                        if !continue_key.is_empty() && res.continue_key.is_none() {
                            res.continue_key = Some(continue_key.to_string());
                        }

                        let raw_data =
                            std::slice::from_raw_parts(recv_packet.data, recv_packet.data_len as _)
                                .to_owned();

                        if let Some(tr_layout) = layout_tbl.get(tr_code.as_ref()) {
                            if tr_layout.block {
                                let block_name = euckr::decode(&recv_packet.block_name).to_string();

                                if let Some(RawData::Block(blocks)) = &mut res.data {
                                    blocks.insert(block_name, raw_data);
                                } else {
                                    res.data = Some(RawData::Block(HashMap::from_iter([(
                                        block_name, raw_data,
                                    )])));
                                }
                            } else {
                                res.data = Some(RawData::NonBlock(raw_data));
                            }
                        }
                    }
                    2 => {
                        let msg_packet = &*(lparam as *const MSG_PACKET);

                        let res = window_data.res_tbl[req_id].as_mut().unwrap();
                        res.code = euckr::decode(&msg_packet.msg_code).to_string();
                        res.message = euckr::decode(std::slice::from_raw_parts(
                            msg_packet.msg_data,
                            msg_packet.msg_data_len as usize,
                        ))
                        .to_string();

                        executor.entry().release_message_data(lparam);
                    }
                    3 => {
                        executor.entry().release_message_data(lparam);
                    }
                    4 => {
                        let res = window_data.res_tbl[req_id].take().unwrap();

                        let mut tx_res = window_data.tx_res_tbl[req_id].lock().unwrap();
                        let _ = tx_res.as_ref().unwrap().send(Some(res));
                        *tx_res = None;

                        executor.entry().release_request_data(req_id as _);
                    }
                    _ => unreachable!(),
                }

                0
            }
            XM_TIMEOUT => {
                let window_data = {
                    let ptr = GetWindowLongPtrA(hwnd, GWLP_USERDATA) as *mut WindowData;
                    assert_ne!(ptr, std::ptr::null_mut());
                    &mut *ptr
                };

                let req_id = lparam as usize;

                window_data.res_tbl[req_id] = None;

                let mut tx_res = window_data.tx_res_tbl[req_id].lock().unwrap();
                let _ = tx_res.as_ref().unwrap().send(None);
                *tx_res = None;

                0
            }
            _ => DefWindowProcA(hwnd, msg, wparam, lparam),
        }
    }
}
