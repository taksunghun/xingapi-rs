// SPDX-License-Identifier: MPL-2.0

use super::entry::Entry;
use super::{Account, Error, LoadError};

use lazy_static::lazy_static;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::{mpsc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{ffi::CString, ops::Deref, path::PathBuf, pin::Pin, thread::JoinHandle, time::Duration};

use winapi::shared::minwindef::{LPARAM, LRESULT, TRUE, UINT, WPARAM};
use winapi::shared::windef::HWND;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::winuser::{
    CreateWindowExA, DefWindowProcA, DispatchMessageA, GetMessageA, GetWindowLongPtrA,
    PostMessageA, PostQuitMessage, RegisterClassExA, SendMessageA, SetWindowLongPtrA,
    TranslateMessage, GWLP_USERDATA, HWND_MESSAGE, WM_DESTROY, WM_USER, WNDCLASSEXA,
};

lazy_static! {
    pub(crate) static ref GLOBAL_EXECUTOR: RwLock<Option<Executor>> = RwLock::new(None);
}

pub(crate) struct GlobalExecutor {
    guard: RwLockReadGuard<'static, Option<Executor>>,
}

impl Deref for GlobalExecutor {
    type Target = Executor;
    fn deref(&self) -> &Self::Target {
        self.guard.as_ref().unwrap()
    }
}

pub(crate) fn global() -> GlobalExecutor {
    let guard = GLOBAL_EXECUTOR.read().unwrap();
    assert!(guard.is_some(), "global executor is not loaded");

    GlobalExecutor { guard }
}

pub(crate) fn load(path: Option<PathBuf>) -> Result<(), LoadError> {
    let mut executor = GLOBAL_EXECUTOR.write().unwrap();
    if executor.is_none() {
        *executor = Some(Executor::new(path)?);
    }

    Ok(())
}

pub(crate) fn unload() {
    *GLOBAL_EXECUTOR.write().unwrap() = None;
}

pub(crate) fn is_loaded() -> bool {
    GLOBAL_EXECUTOR.read().unwrap().is_some()
}

pub(crate) fn loaded_path() -> Option<PathBuf> {
    Some(self::global().guard.as_ref()?.path())
}

// 호출 요청 객체를 정의하는 매크로입니다.
macro_rules! define_req {
    ($($func:ident($($arg:ty),*) -> $ret:ty)*) => {
        #[allow(dead_code)]
        enum CallReq {
            $($func {
                args: ($($arg,)*),
                tx_ret: mpsc::SyncSender<$ret>,
            },)*
        }
    };
}

define_req! {
    DllPath() -> PathBuf
    CreateWindow(CString) -> Result<usize, std::io::Error>

    Connect(usize, String, u16, Duration) -> Result<(), Error>
    IsConnected() -> bool
    Disconnect() -> ()
    Login(usize, String, String, String, bool) -> Result<(), Error>

    Request(usize, String, Vec<u8>, Option<String>, Duration) -> Result<i32, Error>

    AdviseRealData(usize, String, Vec<String>) -> ()
    UnadviseRealData(usize, String, Vec<String>) -> ()
    UnadviseWindow(usize) -> bool

    Accounts() -> Vec<Account>
    GetCommMedia() -> Option<String>
    GetEtkMedia() -> Option<String>
    GetServerName() -> Option<String>
    GetUseOverFuture() -> bool
    GetUseFx() -> bool

    GetTrCountPerSec(String) -> Option<i32>
    GetTrCountBaseSec(String) -> Option<i32>
    GetTrCountRequest(String) -> Option<i32>
    GetTrCountLimit(String) -> Option<i32>
}

// 호출 요청을 보내는 매크로입니다.
macro_rules! req {
    ($self:ident, $func_camel_case:ident($($arg:expr),*)) => {{
        let (tx_ret, rx_ret) = std::sync::mpsc::sync_channel(1);

        let req = Box::into_raw(Box::new(CallReq::$func_camel_case {
            args: ($($arg.into(),)*),
            tx_ret,
        }));

        unsafe {
            if PostMessageA($self.hwnd as _, WM_USER, 20210922, req as _) != TRUE {
                drop(Box::from_raw(req));
                panic!("unable to send a call request");
            }
        }

        rx_ret.recv().unwrap()
    }};
}

pub(crate) struct ExecutorHandle {
    hwnd: usize,
}

impl ExecutorHandle {
    pub fn connect(
        &mut self,
        hwnd: usize,
        addr: &str,
        port: u16,
        timeout: Duration,
    ) -> Result<(), Error> {
        req!(self, Connect(hwnd, addr, port, timeout))
    }

    pub fn is_connected(&self) -> bool {
        req!(self, IsConnected())
    }

    pub fn disconnect(&mut self) {
        req!(self, Disconnect())
    }

    pub fn login(
        &mut self,
        hwnd: usize,
        id: &str,
        pw: &str,
        cert_pw: &str,
        cert_err_dialog: bool,
    ) -> Result<(), Error> {
        req!(self, Login(hwnd, id, pw, cert_pw, cert_err_dialog))
    }

    pub fn request(
        &self,
        hwnd: usize,
        tr_code: &str,
        data: Vec<u8>,
        next_key: Option<&str>,
        timeout: Duration,
    ) -> Result<i32, Error> {
        let next_key = next_key.map(|k| k.to_owned());
        req!(self, Request(hwnd, tr_code, data, next_key, timeout))
    }

    pub fn advise_real_data(&self, hwnd: usize, tr_code: &str, keys: Vec<String>) {
        req!(self, AdviseRealData(hwnd, tr_code, keys))
    }

    pub fn unadvise_real_data(&self, hwnd: usize, tr_code: &str, keys: Vec<String>) {
        req!(self, UnadviseRealData(hwnd, tr_code, keys))
    }

    pub fn accounts(&self) -> Vec<Account> {
        req!(self, Accounts())
    }
    pub fn get_comm_media(&self) -> Option<String> {
        req!(self, GetCommMedia())
    }
    pub fn get_etk_media(&self) -> Option<String> {
        req!(self, GetEtkMedia())
    }
    pub fn get_server_name(&self) -> Option<String> {
        req!(self, GetServerName())
    }
    pub fn get_use_over_future(&self) -> bool {
        req!(self, GetUseOverFuture())
    }
    pub fn get_use_fx(&self) -> bool {
        req!(self, GetUseFx())
    }

    pub fn get_tr_count_per_sec(&self, tr_code: &str) -> Option<i32> {
        req!(self, GetTrCountPerSec(tr_code))
    }
    pub fn get_tr_count_base_sec(&self, tr_code: &str) -> Option<i32> {
        req!(self, GetTrCountBaseSec(tr_code))
    }
    pub fn get_tr_count_request(&self, tr_code: &str) -> Option<i32> {
        req!(self, GetTrCountRequest(tr_code))
    }
    pub fn get_tr_count_limit(&self, tr_code: &str) -> Option<i32> {
        req!(self, GetTrCountLimit(tr_code))
    }
}

lazy_static! {
    static ref EXECUTOR_WNDCLASS: CString = {
        let class_name = CString::new("rust_xingapi_executor").unwrap();

        unsafe {
            RegisterClassExA(&WNDCLASSEXA {
                cbSize: std::mem::size_of::<WNDCLASSEXA>() as _,
                lpfnWndProc: Some(Executor::window_proc),
                cbWndExtra: std::mem::size_of::<usize>() as _,
                hInstance: GetModuleHandleA(std::ptr::null()),
                lpszClassName: class_name.as_ptr(),
                ..std::mem::zeroed()
            });
        }

        class_name
    };
}

struct ExecutorWindowData {
    entry: Pin<Box<Entry>>,
}

pub(crate) struct Executor {
    thread: Option<JoinHandle<()>>,
    hwnd: usize,
    window_data: AtomicPtr<ExecutorWindowData>,
    handle: RwLock<ExecutorHandle>,
}

impl Executor {
    pub fn is_executor_thread() -> bool {
        std::thread::current().name() == Some("rust_xingapi_executor")
    }

    #[allow(clippy::needless_lifetimes)]
    pub fn entry<'a>(&'a self) -> &'a Entry {
        debug_assert!(Self::is_executor_thread());
        unsafe { &(*self.window_data.load(Ordering::Relaxed)).entry }
    }

    pub fn handle(&self) -> RwLockReadGuard<ExecutorHandle> {
        self.handle.read().unwrap()
    }

    pub fn lock_handle(&self) -> RwLockWriteGuard<ExecutorHandle> {
        self.handle.write().unwrap()
    }

    pub fn path(&self) -> PathBuf {
        req!(self, DllPath())
    }

    pub fn create_window(&self, class_name: CString) -> Result<usize, std::io::Error> {
        req!(self, CreateWindow(class_name))
    }

    pub fn unadvise_window(&self, hwnd: usize) -> bool {
        req!(self, UnadviseWindow(hwnd))
    }

    pub fn new(path: Option<PathBuf>) -> Result<Self, LoadError> {
        let (tx_result, rx_result) = mpsc::sync_channel(1);

        let thread_main = move || {
            let load = || -> Result<_, LoadError> {
                let entry = Pin::new(Box::new(if let Some(path) = path.as_deref() {
                    Entry::new_with_path(path)?
                } else {
                    Entry::new()?
                }));

                let window_data = Box::new(ExecutorWindowData { entry });

                #[rustfmt::skip]
                let hwnd = unsafe {
                    CreateWindowExA(
                        0,
                        EXECUTOR_WNDCLASS.as_ptr(),
                        std::ptr::null_mut(),
                        0, 0, 0, 0, 0,
                        HWND_MESSAGE,
                        std::ptr::null_mut(),
                        GetModuleHandleA(std::ptr::null()),
                        std::ptr::null_mut(),
                    )
                };

                if hwnd.is_null() {
                    return Err(std::io::Error::last_os_error().into());
                }

                let window_data = AtomicPtr::new(Box::into_raw(window_data));

                unsafe {
                    SetWindowLongPtrA(
                        hwnd,
                        GWLP_USERDATA,
                        window_data.load(Ordering::Relaxed) as _,
                    );
                }

                Ok((hwnd as _, window_data))
            };

            match load() {
                Ok(ret) => {
                    tx_result.send(Ok(ret)).unwrap();
                }
                Err(err) => {
                    tx_result.send(Err(err)).unwrap();
                    return;
                }
            };

            unsafe {
                let mut msg = std::mem::zeroed();

                while GetMessageA(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                    TranslateMessage(&msg);
                    DispatchMessageA(&msg);
                }
            }
        };

        let thread = Some(
            std::thread::Builder::new()
                .name("rust_xingapi_executor".into())
                .spawn(thread_main)
                .unwrap(),
        );

        let (hwnd, window_data) = rx_result.recv().unwrap()?;
        let handle = RwLock::new(ExecutorHandle { hwnd });

        Ok(Self {
            thread,
            hwnd,
            window_data,
            handle,
        })
    }

    unsafe extern "system" fn window_proc(
        hwnd: HWND,
        msg: UINT,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_DESTROY => {
                let window_data = GetWindowLongPtrA(hwnd, GWLP_USERDATA) as *mut ExecutorWindowData;
                assert_ne!(window_data, std::ptr::null_mut());
                drop(Box::from_raw(window_data));

                PostQuitMessage(0);

                0
            }
            // 호출 요청을 수신한 경우
            WM_USER => {
                let window_data = {
                    let ptr = GetWindowLongPtrA(hwnd, GWLP_USERDATA) as *const ExecutorWindowData;
                    assert_ne!(ptr, std::ptr::null());
                    &*ptr
                };

                assert_eq!(wparam, 20210922);
                assert_ne!(lparam, 0);

                let req = Box::from_raw(lparam as *mut CallReq);
                Self::on_request(&window_data.entry, *req);

                0
            }
            _ => DefWindowProcA(hwnd, msg, wparam, lparam),
        }
    }

    fn on_request(entry: &Entry, req: CallReq) {
        macro_rules! match_req {
            ($($func:ident($($arg:ident),*) => $code:expr$(,)?)*) => {
                match req {
                    $(
                        CallReq::$func { args: ($($arg,)*), tx_ret } => {
                            let _ = tx_ret.try_send($code);
                        }
                    )*
                }
            };
        }

        match_req! {
            DllPath() => entry.path().to_owned(),

            CreateWindow(class_name) => {
                #[rustfmt::skip]
                let hwnd = unsafe {
                    CreateWindowExA(
                        0,
                        class_name.as_ptr(),
                        std::ptr::null(),
                        0, 0, 0, 0, 0,
                        HWND_MESSAGE,
                        std::ptr::null_mut(),
                        GetModuleHandleA(std::ptr::null()),
                        std::ptr::null_mut(),
                    )
                };

                if !hwnd.is_null() {
                    Ok(hwnd as _)
                } else {
                    Err(std::io::Error::last_os_error())
                }
            }

            Connect(hwnd, addr, port, timeout) => {
                entry.connect(hwnd, &addr, port, timeout)
            }
            IsConnected() => entry.is_connected(),
            Disconnect() => entry.disconnect(),
            Login(hwnd, id, pw, cert_pw, cert_err_dialog) => {
                entry.login(hwnd, &id, &pw, &cert_pw, cert_err_dialog)
            }
            Request(hwnd, tr_code, data, next_key, timeout) => {
                entry.request(hwnd, &tr_code, &data, next_key.as_deref(), timeout)
            }
            AdviseRealData(hwnd, tr_code, keys) => {
                entry.advise_real_data(hwnd, &tr_code, &keys)
            }
            UnadviseRealData(hwnd, tr_code, keys) => {
                entry.unadvise_real_data(hwnd, &tr_code, &keys)
            }
            UnadviseWindow(hwnd) => {
                entry.unadvise_window(hwnd)
            }
            Accounts() => entry.accounts(),
            GetCommMedia() => entry.get_comm_media(),
            GetEtkMedia() => entry.get_etk_media(),
            GetServerName() => entry.get_server_name(),
            GetUseOverFuture() => entry.get_use_over_future(),
            GetUseFx() => entry.get_use_fx(),
            GetTrCountPerSec(tr_code) => {
                entry.get_tr_count_per_sec(&tr_code)
            }
            GetTrCountBaseSec(tr_code) => {
                entry.get_tr_count_base_sec(&tr_code)
            }
            GetTrCountRequest(tr_code) => {
                entry.get_tr_count_request(&tr_code)
            }
            GetTrCountLimit(tr_code) => {
                entry.get_tr_count_limit(&tr_code)
            }
        }
    }
}

impl Drop for Executor {
    fn drop(&mut self) {
        unsafe {
            if PostMessageA(self.hwnd as _, WM_DESTROY, 0, 0) != TRUE {
                SendMessageA(self.hwnd as _, WM_DESTROY, 0, 0);
            }
        }

        let _ = self.thread.take().unwrap().join();
    }
}

pub(crate) struct Window {
    hwnd: usize,
}

impl Window {
    pub fn new(class_name: CString) -> Result<Self, std::io::Error> {
        let hwnd = self::global().create_window(class_name)?;

        Ok(Self { hwnd })
    }
}

impl Deref for Window {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.hwnd
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            if PostMessageA(self.hwnd as _, WM_DESTROY, 0, 0) != TRUE {
                SendMessageA(self.hwnd as _, WM_DESTROY, 0, 0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{DllError, LoadError};
    use super::Executor;

    #[test]
    fn test_load_executor() {
        let executor = Executor::new(None).unwrap();
        assert!(!executor.handle().is_connected());
        assert!(matches!(
            Executor::new(None),
            Err(LoadError::Dll(DllError::LibraryInUse))
        ));
    }
}
