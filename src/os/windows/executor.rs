// SPDX-License-Identifier: MPL-2.0

// XingAPI 및 Win32 API 함수 호출을 별도의 스레드에서 비동기적으로 대신 수행하고
// 윈도우 메시지 루프를 반복하는 스레드에 대한 모듈입니다.

use super::entry::Entry;
use crate::{EntryError, Error, Win32Error};

use std::panic::{RefUnwindSafe, UnwindSafe};
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::thread::{self, JoinHandle};
use std::{mem::MaybeUninit, path::Path, pin::Pin};

use winapi::shared::minwindef::TRUE;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::winuser::{
    CreateWindowExA, DestroyWindow, DispatchMessageA, PeekMessageA, TranslateMessage, HWND_MESSAGE,
    MSG, PM_REMOVE,
};

macro_rules! define_fn {
    ($($func_camel_case:ident($($arg:ty$(,)?)*) -> $ret:ty)*) => {
        #[allow(dead_code)]
        enum Func {
            $($func_camel_case {
                args: ($($arg,)*),
                tx_ret: crossbeam_channel::Sender<$ret>,
            },)*
        }
    };
}

define_fn! {
    // Win32 API
    CreateWindow(Vec<i8>) -> Result<usize, Win32Error>
    DestroyWindow(usize) -> Result<(), Win32Error>

    // XingAPI
    Connect(usize, String, u16, Option<i32>, Option<i32>) -> Result<(), Error>
    IsConnected() -> bool
    Disconnect() -> ()
    Login(usize, String, String, String, bool) -> Result<(), Error>
    Request(usize, String, Vec<u8>, Option<String>, Option<i32>) -> Result<i32, Error>
    AdviseRealData(usize, String, String) -> bool
    UnadviseRealData(usize, String, String) -> bool
    UnadviseWindow(usize) -> bool
    GetAccountList() -> Vec<String>
    GetAccountName(String) -> String
    GetAccountDetailName(String) -> String
    GetAccountNickname(String) -> String
    GetClientIp() -> String
    GetServerName() -> String
    GetApiPath() -> String
    GetTrCountPerSec(String) -> i32
    GetTrCountBaseSec(String) -> i32
    GetTrCountRequest(String) -> i32
    GetTrCountLimit(String) -> i32
}

macro_rules! call {
    ($self:ident, $func_camel_case:ident($($arg:expr$(,)?)*)) => {{
        let (tx_ret, rx_ret) = crossbeam_channel::bounded(1);

        $self.tx_func.send(Func::$func_camel_case {
            args: ($($arg.into(),)*),
            tx_ret,
        }).unwrap();

        rx_ret.recv().unwrap()
    }};
}

// Executor의 핸들로, 함수 호출을 요청할 수 있는 핸들입니다.
//
// XingAPI 서버 연결 및 로그인 이벤트의 동기적 처리를 위해 RwLock에 묶여 있습니다.
pub struct ExecutorHandle {
    tx_func: crossbeam_channel::Sender<Func>,
}

impl ExecutorHandle {
    pub fn connect(
        &self,
        hwnd: usize,
        addr: &str,
        port: u16,
        timeout: Option<i32>,
        packet_len_limit: Option<i32>,
    ) -> Result<(), Error> {
        call!(self, Connect(hwnd, addr, port, timeout, packet_len_limit))
    }

    pub fn is_connected(&self) -> bool {
        call!(self, IsConnected())
    }
    pub fn disconnect(&self) {
        call!(self, Disconnect())
    }

    pub fn login(
        &self,
        hwnd: usize,
        id: &str,
        pw: &str,
        cert_pw: &str,
        cert_err_dialog: bool,
    ) -> Result<(), Error> {
        call!(self, Login(hwnd, id, pw, cert_pw, cert_err_dialog))
    }

    pub fn request(
        &self,
        hwnd: usize,
        tr_code: &str,
        data: Vec<u8>,
        continue_key: Option<&str>,
        timeout: Option<i32>,
    ) -> Result<i32, Error> {
        let continue_key = continue_key.map(|s| s.to_owned());
        call!(self, Request(hwnd, tr_code, data, continue_key, timeout))
    }

    pub fn advise_real_data(&self, hwnd: usize, tr_code: &str, data: &str) -> bool {
        call!(self, AdviseRealData(hwnd, tr_code, data))
    }

    pub fn unadvise_real_data(&self, hwnd: usize, tr_code: &str, data: &str) -> bool {
        call!(self, UnadviseRealData(hwnd, tr_code, data))
    }

    pub fn get_account_list(&self) -> Vec<String> {
        call!(self, GetAccountList())
    }
    pub fn get_account_name(&self, account: &str) -> String {
        call!(self, GetAccountName(account))
    }
    pub fn get_account_detail_name(&self, account: &str) -> String {
        call!(self, GetAccountDetailName(account))
    }
    pub fn get_account_nickname(&self, account: &str) -> String {
        call!(self, GetAccountNickname(account))
    }
    pub fn get_client_ip(&self) -> String {
        call!(self, GetClientIp())
    }
    pub fn get_server_name(&self) -> String {
        call!(self, GetServerName())
    }
    pub fn get_api_path(&self) -> String {
        call!(self, GetApiPath())
    }
    pub fn get_tr_count_per_sec(&self, tr_code: &str) -> i32 {
        call!(self, GetTrCountPerSec(tr_code))
    }
    pub fn get_tr_count_base_sec(&self, tr_code: &str) -> i32 {
        call!(self, GetTrCountBaseSec(tr_code))
    }
    pub fn get_tr_count_request(&self, tr_code: &str) -> i32 {
        call!(self, GetTrCountRequest(tr_code))
    }
    pub fn get_tr_count_limit(&self, tr_code: &str) -> i32 {
        call!(self, GetTrCountLimit(tr_code))
    }
}

// 함수 호출 및 윈도우 메시지 루프를 대신 수행하는 스레드 객체입니다.
pub struct Executor {
    join_handle: Option<JoinHandle<()>>,
    tx_func: crossbeam_channel::Sender<Func>,
    tx_quit: crossbeam_channel::Sender<()>,
    handle: RwLock<ExecutorHandle>,
    entry: AtomicPtr<Entry>,
}

impl Executor {
    pub fn create_window(&self, class_name: &[i8]) -> Result<usize, Win32Error> {
        call!(self, CreateWindow(class_name.to_owned()))
    }

    pub fn destroy_window(&self, hwnd: usize) -> Result<(), Win32Error> {
        call!(self, DestroyWindow(hwnd))
    }

    pub fn unadvise_window(&self, hwnd: usize) -> bool {
        call!(self, UnadviseWindow(hwnd))
    }

    pub fn is_executor_thread() -> bool {
        std::thread::current().name() == Some("xingapi_executor_thread")
    }

    pub fn handle(&self) -> RwLockReadGuard<ExecutorHandle> {
        self.handle.read().unwrap()
    }

    pub fn lock_handle(&self) -> RwLockWriteGuard<ExecutorHandle> {
        self.handle.write().unwrap()
    }

    pub fn entry(&self) -> &Entry {
        debug_assert!(Executor::is_executor_thread());
        unsafe { &*self.entry.load(Ordering::Relaxed) }
    }

    pub fn new(path: Option<&Path>) -> Result<Self, EntryError> {
        let path = path.map(|s| s.to_owned());

        let (tx_result, rx_result) = crossbeam_channel::bounded(1);
        let (tx_func, rx_func) = crossbeam_channel::unbounded();
        let (tx_quit, rx_quit) = crossbeam_channel::bounded(1);

        let thread_main = move || {
            let load_entry = || -> Result<Pin<Box<Entry>>, EntryError> {
                Ok(Pin::new(Box::new(if let Some(path) = path.as_deref() {
                    Entry::new_with_path(path)?
                } else {
                    Entry::new()?
                })))
            };

            let entry = match load_entry() {
                Ok(mut entry) => {
                    tx_result.send(Ok(AtomicPtr::new(entry.as_mut().get_mut()))).unwrap();
                    entry
                }
                Err(err) => {
                    tx_result.send(Err(err)).unwrap();
                    return;
                }
            };

            let mut quit = false;
            loop {
                unsafe {
                    #[allow(clippy::uninit_assumed_init)]
                    let mut msg = MaybeUninit::<MSG>::uninit().assume_init();
                    while PeekMessageA(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) == TRUE {
                        TranslateMessage(&msg);
                        DispatchMessageA(&msg);
                    }
                }

                if let Ok(func) = rx_func.try_recv() {
                    Self::call_func(&entry, func);
                } else if quit {
                    break;
                }

                if let Ok(()) = rx_quit.try_recv() {
                    quit = true;
                }
            }
        };

        let join_handle = Some(
            thread::Builder::new()
                .name("xingapi_executor_thread".into())
                .spawn(thread_main)
                .unwrap(),
        );

        let entry = rx_result.recv().unwrap()?;

        Ok(Self {
            join_handle,
            tx_func: tx_func.clone(),
            tx_quit,
            handle: RwLock::new(ExecutorHandle { tx_func }),
            entry,
        })
    }

    fn call_func(entry: &Entry, func: Func) {
        #[allow(unused_must_use)]
        #[allow(clippy::unit_arg)]
        match func {
            // Win32 API
            Func::CreateWindow { args: (class_name,), tx_ret } => {
                #[rustfmt::skip]
                let hwnd = unsafe {
                    CreateWindowExA(
                        0,
                        class_name.as_ptr(),
                        std::ptr::null_mut(),
                        0, 0, 0, 0, 0,
                        HWND_MESSAGE,
                        std::ptr::null_mut(),
                        GetModuleHandleA(std::ptr::null()),
                        std::ptr::null_mut(),
                    )
                };

                let _ = tx_ret.try_send(if !hwnd.is_null() {
                    Ok(hwnd as _)
                } else {
                    Err(Win32Error::from_last_error())
                });
            }
            Func::DestroyWindow { args: (hwnd,), tx_ret } => {
                let _ = tx_ret.try_send(if unsafe { DestroyWindow(hwnd as _) } == TRUE {
                    Ok(())
                } else {
                    Err(Win32Error::from_last_error())
                });
            }

            // XingAPI
            Func::Connect { args: (hwnd, addr, port, timeout, packet_len_limit), tx_ret } => {
                tx_ret.try_send(entry.connect(hwnd, &addr, port, timeout, packet_len_limit));
            }
            Func::IsConnected { args: (), tx_ret } => {
                tx_ret.try_send(entry.is_connected());
            }
            Func::Disconnect { args: (), tx_ret } => {
                tx_ret.try_send(entry.disconnect());
            }
            Func::Login { args: (hwnd, id, pw, cert_pw, cert_err_dialog), tx_ret } => {
                tx_ret.try_send(entry.login(hwnd, &id, &pw, &cert_pw, cert_err_dialog));
            }
            Func::Request { args: (hwnd, tr_code, data, continue_key, timeout), tx_ret } => {
                let ret = entry.request(hwnd, &tr_code, &data, continue_key.as_deref(), timeout);
                tx_ret.try_send(ret);
            }
            Func::AdviseRealData { args: (hwnd, tr_code, data), tx_ret } => {
                tx_ret.try_send(entry.advise_real_data(hwnd, &tr_code, &data));
            }
            Func::UnadviseRealData { args: (hwnd, tr_code, data), tx_ret } => {
                tx_ret.try_send(entry.unadvise_real_data(hwnd, &tr_code, &data));
            }
            Func::UnadviseWindow { args: (hwnd,), tx_ret } => {
                tx_ret.try_send(entry.unadvise_window(hwnd));
            }
            Func::GetAccountList { args: (), tx_ret } => {
                tx_ret.try_send(entry.get_account_list());
            }
            Func::GetAccountName { args: (account,), tx_ret } => {
                tx_ret.try_send(entry.get_account_name(&account));
            }
            Func::GetAccountDetailName { args: (account,), tx_ret } => {
                tx_ret.try_send(entry.get_account_detail_name(&account));
            }
            Func::GetAccountNickname { args: (account,), tx_ret } => {
                tx_ret.try_send(entry.get_account_nickname(&account));
            }
            Func::GetClientIp { args: (), tx_ret } => {
                tx_ret.try_send(entry.get_client_ip());
            }
            Func::GetServerName { args: (), tx_ret } => {
                tx_ret.try_send(entry.get_server_name());
            }
            Func::GetApiPath { args: (), tx_ret } => {
                tx_ret.try_send(entry.get_api_path());
            }
            Func::GetTrCountPerSec { args: (tr_code,), tx_ret } => {
                tx_ret.try_send(entry.get_tr_count_per_sec(&tr_code));
            }
            Func::GetTrCountBaseSec { args: (tr_code,), tx_ret } => {
                tx_ret.try_send(entry.get_tr_count_base_sec(&tr_code));
            }
            Func::GetTrCountRequest { args: (tr_code,), tx_ret } => {
                tx_ret.try_send(entry.get_tr_count_request(&tr_code));
            }
            Func::GetTrCountLimit { args: (tr_code,), tx_ret } => {
                tx_ret.try_send(entry.get_tr_count_limit(&tr_code));
            }
        }
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
        self.tx_quit.try_send(()).unwrap();
        self.join_handle.take().unwrap().join().unwrap();
    }
}

impl UnwindSafe for Executor {}
impl RefUnwindSafe for Executor {}

#[cfg(test)]
mod tests {
    use super::Executor;
    use std::sync::Arc;

    #[test]
    fn test_load_executor() {
        let executor = Arc::new(Executor::new(None).unwrap());
        println!("api_path: {:?}", executor.handle().get_api_path());
        assert!(!executor.handle().is_connected());
    }
}
