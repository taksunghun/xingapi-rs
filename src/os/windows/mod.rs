// SPDX-License-Identifier: MPL-2.0

//! 윈도우 운영체제 구현입니다.

#[cfg(windows)]
mod caller;
#[cfg(windows)]
mod entry;
#[cfg(windows)]
mod raw;
#[cfg(windows)]
mod window;

#[cfg(windows)]
mod query;
#[cfg(windows)]
mod real;
#[cfg(windows)]
mod session;

macro_rules! windows_only_impl {
    ($($code:stmt)*) => {
        #[cfg(not(windows))]
        unimplemented!();

        #[allow(redundant_semicolons)]
        #[cfg(windows)]
        {
            $($code)*
        }
    };
}

pub mod error;

#[cfg(windows)]
mod bindings {
    pub use winapi::{
        ctypes::{c_int, c_void},
        shared::{
            minwindef::{BOOL, DWORD, FALSE, LPARAM, LRESULT, TRUE, UINT, WPARAM},
            windef::HWND,
        },
        um::{
            errhandlingapi::GetLastError,
            libloaderapi::GetModuleHandleA,
            winuser::{
                CreateWindowExA, DefWindowProcA, DestroyWindow, DispatchMessageA,
                GetWindowLongPtrA, PeekMessageA, RegisterClassExA, SetWindowLongPtrA,
                TranslateMessage, GWLP_USERDATA, HWND_MESSAGE, MSG, PM_REMOVE, WM_DESTROY, WM_USER,
                WNDCLASSEXA,
            },
        },
    };
}

#[cfg(not(windows))]
mod bindings {
    pub enum DWORD {}
}

#[cfg(windows)]
use self::{caller::Caller, query::QueryWindow, real::RealWindow, session::SessionWindow};
use crate::{
    data::Data,
    error::Error,
    response::{LoginResponse, QueryResponse, RealResponse},
    Account,
};

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use xingapi_res::TrLayout;

/// 지정된 설정으로 XingAPI를 불러오기 위한 builder입니다.
pub struct XingApiBuilder {
    path: Option<PathBuf>,
    tr_layouts: Option<HashMap<String, TrLayout>>,
}

impl XingApiBuilder {
    /// builder를 생성합니다.
    pub fn new() -> Self {
        Self { path: None, tr_layouts: None }
    }

    /// XingAPI DLL의 경로를 지정합니다.
    pub fn path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.path = Some(path.as_ref().to_owned());
        self
    }

    /// XingAPI에서 사용될 RES 데이터를 지정합니다.
    pub fn layouts<I>(mut self, layouts: I) -> Self
    where
        I: IntoIterator<Item = TrLayout>,
    {
        if self.tr_layouts.is_none() {
            self.tr_layouts = Some(HashMap::new());
        }

        let tr_layouts = self.tr_layouts.as_mut().unwrap();
        tr_layouts.extend(layouts.into_iter().map(|tr| (tr.code.to_owned(), tr)));
        self
    }

    pub async fn build(self) -> Result<Arc<XingApi>, Error> {
        windows_only_impl! {
            let caller = Arc::new(Caller::new(self.path.as_deref())?);

            let tr_layouts =
                Arc::new(if let Some(val) = self.tr_layouts { val } else { xingapi_res::load()? });

            Ok(Arc::new(XingApi {
                session_window: SessionWindow::new(caller.clone()).await?,
                query_window: QueryWindow::new(caller.clone(), tr_layouts.clone()).await?,
                caller,
                tr_layouts,
            }))
        }
    }
}

/// XingAPI를 비동기 함수로 추상화한 객체입니다.
///
/// `connect()`, `disconnect()`, `login()`과 같은 연결 및 로그인 함수를 호출할 경우, 다른 함수의
/// 호출이 완료될 때까지 대기하고, 동시에 호출되는 다른 함수의 호출을 일시적으로 대기시킵니다.
pub struct XingApi {
    #[cfg(windows)]
    caller: Arc<Caller>,
    tr_layouts: Arc<HashMap<String, TrLayout>>,
    #[cfg(windows)]
    session_window: SessionWindow,
    #[cfg(windows)]
    query_window: QueryWindow,
}

impl XingApi {
    /// 기본적인 설정으로 XingAPI를 불러옵니다.
    pub async fn new() -> Result<Arc<Self>, Error> {
        windows_only_impl! {
            let caller = Arc::new(Caller::new(None)?);
            let layout_map = Arc::new(xingapi_res::load()?);

            Ok(Arc::new(Self {
                session_window: SessionWindow::new(caller.clone()).await?,
                query_window: QueryWindow::new(caller.clone(), layout_map.clone()).await?,
                caller,
                tr_layouts: layout_map,
            }))
        }
    }

    /// 해당하는 주소로 서버에 연결합니다.

    pub async fn connect(
        &self,
        addr: &str,
        port: u16,
        timeout: Option<i32>,
        max_packet_size: Option<i32>,
    ) -> Result<(), Error> {
        windows_only_impl! {
            self.session_window.connect(addr, port, timeout, max_packet_size).await
        }
    }

    /// 서버에 연결되어 있는지에 대한 여부를 반환합니다
    pub async fn is_connected(&self) -> bool {
        windows_only_impl! {
            self.caller.handle().read().await.is_connected().await
        }
    }

    /// 서버와의 연결을 중단합니다.
    pub async fn disconnect(&self) {
        windows_only_impl! {
            self.caller.handle().write().await.disconnect().await
        }
    }

    /// 서버에 로그인 요청을 보내고 응답을 기다립니다.
    pub async fn login(
        &self,
        id: &str,
        pw: &str,
        cert_pw: &str,
        cert_err_dialog: bool,
    ) -> Result<LoginResponse, Error> {
        windows_only_impl! {
            self.session_window.login(id, pw, cert_pw, cert_err_dialog).await
        }
    }

    /// 서버에 TR를 요청하고 응답을 기다립니다.
    pub async fn request(
        &self,
        data: &Data,
        continue_key: Option<&str>,
        timeout: Option<i32>,
    ) -> Result<QueryResponse, Error> {
        windows_only_impl! {
            self.query_window.request(data, continue_key, timeout).await
        }
    }

    /// 서버에 로그인 되어 있는 경우 계좌 목록을 반환합니다.
    pub async fn accounts(&self) -> Vec<Account> {
        windows_only_impl! {
            let handle = self.caller.handle().read().await;
            let codes = handle.get_account_list().await;

            let mut accounts = Vec::with_capacity(codes.len());
            for code in codes {
                accounts.push(Account {
                    name: handle.get_account_name(&code).await,
                    detail_name: handle.get_account_detail_name(&code).await,
                    nickname: handle.get_account_nickname(&code).await,
                    code,
                });
            }

            accounts
        }
    }

    /// 서버에 접속되어 있는 경우 클라이언트 IP를 반환합니다.
    pub async fn client_ip(&self) -> String {
        windows_only_impl! {
            self.caller.handle().read().await.get_client_ip().await
        }
    }

    /// 서버에 접속되어 있는 경우 접속한 서버 이름을 반환합니다.
    pub async fn server_name(&self) -> String {
        windows_only_impl! {
            self.caller.handle().read().await.get_server_name().await
        }
    }

    /// XingAPI의 디렉터리 경로를 반환합니다.
    pub async fn path(&self) -> String {
        windows_only_impl! {
            self.caller.handle().read().await.get_api_path().await
        }
    }

    /// 연결된 서버의 초당 TR 전송 제한 횟수를 반환합니다.
    pub async fn limit_per_one_sec(&self, tr_code: &str) -> i32 {
        windows_only_impl! {
            self.caller.handle().read().await.get_tr_count_per_sec(tr_code).await
        }
    }

    /// 연결된 서버의 1회당 TR 전송 제한 시간을 초 단위로 반환합니다.
    pub async fn limit_sec_per_once(&self, tr_code: &str) -> i32 {
        windows_only_impl! {
            self.caller.handle().read().await.get_tr_count_base_sec(tr_code).await
        }
    }

    /// 연결된 서버에 10분내 요청한 TR의 총 횟수를 반환합니다.
    pub async fn count_in_ten_min(&self, tr_code: &str) -> i32 {
        windows_only_impl! {
            self.caller.handle().read().await.get_tr_count_request(tr_code).await
        }
    }

    /// 연결된 서버의 10분당 TR 전송 제한 횟수를 반환합니다.
    pub async fn limit_per_ten_min(&self, tr_code: &str) -> i32 {
        windows_only_impl! {
            self.caller.handle().read().await.get_tr_count_limit(tr_code).await
        }
    }
}

/// 실시간 TR를 수신하는 리시버입니다.
///
/// `connect()`, `disconnect()`, `login()`과 같은 연결 및 로그인 함수를 호출하면 기존에 등록된 TR은
/// 모두 사라지게 됩니다.
///
/// 실시간 TR을 등록하면 수신받은 TR은 내부적으로 큐에 저장되며 `recv()`를 호출하여 반드시 처리해야
/// 합니다. 그렇지 않으면 메모리 누수가 발생할 것입니다.
pub struct Real {
    #[cfg(windows)]
    window: RealWindow,
}

impl Real {
    /// 실시간 TR을 수신하는 객체를 생성합니다.
    pub async fn new(xingapi: Arc<XingApi>) -> Result<Self, Error> {
        windows_only_impl! {
            Ok(Self { window: RealWindow::new(xingapi).await? })
        }
    }

    /// 실시간 TR을 지정된 종목 코드로 등록합니다.
    ///
    /// `data`는 종목 코드 목록이며 종목 코드는 ASCII 문자로만 구성되어야 합니다.
    pub async fn register(&self, tr_code: &str, data: Vec<String>) -> Result<(), Error> {
        windows_only_impl! {
            self.window.register(tr_code, data).await
        }
    }

    /// 실시간 TR을 지정된 종목 코드로 등록 해제합니다.
    ///
    /// `data`는 종목 코드 목록이며 종목 코드는 ASCII 문자로만 구성되어야 합니다.
    pub async fn unregister(&self, tr_code: &str, data: Vec<String>) -> Result<(), Error> {
        windows_only_impl! {
            self.window.unregister(tr_code, data).await
        }
    }

    /// 실시간 TR을 모두 등록 해제합니다.
    pub async fn unregister_all(&self) -> Result<(), Error> {
        windows_only_impl! {
            self.window.unregister_all().await
        }
    }

    /// 서버로부터 수신받은 실시간 TR을 큐에서 가져올 때까지 기다립니다.
    pub async fn recv(&self) -> RealResponse {
        windows_only_impl! {
            self.window.recv().await
        }
    }

    /// 서버로부터 수신받은 실시간 TR이 있는 경우 실시간 TR을 반환하고,
    /// 그렇지 않은 경우 `None`을 반환합니다.
    pub fn try_recv(&self) -> Option<RealResponse> {
        windows_only_impl! {
            self.window.try_recv()
        }
    }
}
