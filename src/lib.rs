// SPDX-License-Identifier: MPL-2.0

//! A safe, easy and optimized abstraction for XingAPI.
//!
//! 안전성과 간편성, 최적화를 동시에 추구하는 XingAPI 추상화 구현 라이브러리입니다.
//!
//! # 요구 사항
//! - 이베스트투자증권의 윈도우용 XingAPI 최신 버전
//! - RES 파일 (TR 레이아웃)
//! - VS2010 재배포 가능 패키지 (런타임)
//!
//! 아직은 윈도우 32비트 버전의 XingAPI만 지원합니다.

#![cfg_attr(doc_cfg, feature(doc_cfg))]

pub mod data;
pub mod error;
pub mod response;

mod euckr;
mod os;

use self::data::Data;
use self::error::{Error, LoadError};
use self::response::{LoginResponse, QueryResponse, RealResponse};

use std::path::{Path, PathBuf};
use std::{sync::Arc, time::Duration};

use xingapi_res::TrLayout;

#[cfg(target_os = "windows")]
use os::windows as imp;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// 이베스트투자증권 계좌를 저장하는 객체입니다.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Account {
    /// 계좌번호
    pub code: String,
    /// 계좌명
    pub name: String,
    /// 계좌 상세명
    pub detail_name: String,
    /// 계좌 별명
    pub nickname: String,
}

/// 지정된 설정으로 `XingApi` 객체를 생성하기 위한 빌더입니다.
#[cfg(any(windows, doc))]
#[cfg_attr(doc_cfg, doc(cfg(windows)))]
pub struct XingApiBuilder {
    path: Option<PathBuf>,
    layouts: Vec<TrLayout>,
}

#[cfg(any(windows, doc))]
impl XingApiBuilder {
    /// `XingApi` 빌더를 생성합니다.
    pub fn new() -> Self {
        Self { path: None, layouts: Vec::new() }
    }

    /// XingAPI 공유 라이브러리의 경로를 지정합니다.
    pub fn path<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.path = Some(path.as_ref().to_owned());
        self
    }

    /// XingAPI에서 사용될 TR 레이아웃을 지정합니다.
    ///
    /// TR 레이아웃이 지정되지 않은 경우 기본 경로에서 불러오기를 시도합니다.
    pub fn layouts<I>(mut self, layouts: I) -> Self
    where
        I: IntoIterator<Item = TrLayout>,
    {
        self.layouts.extend(layouts);
        self
    }

    /// `XingApi` 객체를 생성합니다.
    pub fn build(self) -> Result<Arc<XingApi>, LoadError> {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        Ok(Arc::new(XingApi(imp::XingApi::new(
            self.path.as_deref(),
            if self.layouts.is_empty() {
                xingapi_res::load()?
            } else {
                self.layouts.into_iter().map(|i| (i.code.to_owned(), i)).collect()
            },
        )?)))
    }
}

#[cfg(any(windows, doc))]
impl Default for XingApiBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// XingAPI를 비동기 함수로 추상화한 객체입니다.
///
/// `connect()`, `login()`과 같은 연결 및 로그인 함수를 호출할 경우, 다른 함수의 호출이 완료될
/// 때까지 대기하고, 동시에 호출되는 다른 함수의 호출을 일시적으로 대기시킵니다.
///
/// **이 객체는 소멸자가 반드시 호출되어야 합니다.** 소멸자 호출 없이 프로그램이 종료될 경우,
/// 비정상적으로 종료될 수 있습니다. Rust에서는 메인 스레드가 종료될 경우 서브 스레드가 자원 해제
/// 없이 곧바로 종료된다는 것에 유의해야 합니다.
#[cfg(any(windows, doc))]
#[cfg_attr(doc_cfg, doc(cfg(windows)))]
pub struct XingApi(#[cfg(windows)] Arc<imp::XingApi>);

#[cfg(any(windows, doc))]
impl XingApi {
    /// 기본적인 설정으로 객체를 초기화합니다
    pub fn new() -> Result<Arc<Self>, LoadError> {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        XingApiBuilder::new().build()
    }

    /// 서버 연결을 시도합니다.
    pub fn connect(
        &self,
        addr: &str,
        port: u16,
        timeout: Option<i32>,
        packet_len_limit: Option<i32>,
    ) -> Result<(), Error> {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.connect(addr, port, timeout, packet_len_limit)
    }

    /// 서버 연결 여부를 반환합니다
    pub fn is_connected(&self) -> bool {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.is_connected()
    }

    /// 서버 연결을 중단합니다.
    pub fn disconnect(&self) {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.disconnect()
    }

    /// 서버에 로그인 요청을 보내고 응답을 기다립니다.
    pub fn login(
        &self,
        id: &str,
        pw: &str,
        cert_pw: &str,
        cert_err_dialog: bool,
    ) -> Result<LoginResponse, Error> {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.login(id, pw, cert_pw, cert_err_dialog)
    }

    /// 서버에 TR를 요청하고 응답을 기다립니다.
    pub fn request(
        &self,
        data: &Data,
        continue_key: Option<&str>,
        timeout: Option<i32>,
    ) -> Result<QueryResponse, Error> {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.request(data, continue_key, timeout)
    }

    /// 서버에 로그인 되어 있는 경우 계좌 목록을 반환합니다.
    pub fn accounts(&self) -> Vec<Account> {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.accounts()
    }

    /// 서버에 접속되어 있는 경우 클라이언트 IP를 반환합니다.
    pub fn client_ip(&self) -> String {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.client_ip()
    }

    /// 서버에 접속되어 있는 경우 접속한 서버 이름을 반환합니다.
    pub fn server_name(&self) -> String {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.server_name()
    }

    /// XingAPI 공유 라이브러리의 디렉터리 경로를 반환합니다.
    pub fn path(&self) -> String {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.path()
    }

    /// 연결된 서버의 초당 TR 전송 제한 횟수를 반환합니다.
    pub fn limit_per_one_sec(&self, tr_code: &str) -> i32 {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.limit_per_one_sec(tr_code)
    }

    /// 연결된 서버의 1회당 TR 전송 제한 시간을 초 단위로 반환합니다.
    pub fn limit_sec_per_once(&self, tr_code: &str) -> i32 {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.limit_sec_per_once(tr_code)
    }

    /// 연결된 서버에 10분내 요청한 TR의 총 횟수를 반환합니다.
    pub fn count_in_ten_min(&self, tr_code: &str) -> i32 {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.count_in_ten_min(tr_code)
    }

    /// 연결된 서버의 10분당 TR 전송 제한 횟수를 반환합니다.
    pub fn limit_per_ten_min(&self, tr_code: &str) -> i32 {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.limit_per_ten_min(tr_code)
    }
}

/// 실시간 TR를 수신하는 리시버입니다.
///
/// `connect()`, `disconnect()`, `login()`과 같은 연결 및 로그인 함수를 호출하면 기존에 등록된
/// TR은 모두 사라지게 됩니다.
///
/// 실시간 TR을 등록하면 수신받은 TR은 내부적으로 큐에 저장되며 이를 처리하지 않을 경우 메모리
/// 누수로 이어집니다. 따라서 `Real::recv()`를 호출하여 수신받은 TR을 반드시 처리해야 합니다.
#[cfg(any(windows, doc))]
#[cfg_attr(doc_cfg, doc(cfg(windows)))]
pub struct Real(#[cfg(windows)] imp::Real, Arc<XingApi>);

#[cfg(any(windows, doc))]
impl Real {
    /// 실시간 TR을 수신하는 객체를 생성합니다.
    pub fn new(xingapi: Arc<XingApi>) -> Result<Self, LoadError> {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        Ok(Self(imp::Real::new(xingapi.0.clone())?, xingapi))
    }

    /// 실시간 TR을 지정된 종목 코드로 등록합니다.
    ///
    /// `data`는 종목 코드 목록이며 종목 코드는 ASCII 문자로만 구성되어야 합니다.
    pub fn subscribe(&self, tr_code: &str, data: Vec<String>) -> Result<(), ()> {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.subscribe(tr_code, data)
    }

    /// 실시간 TR을 지정된 종목 코드로 등록 해제합니다.
    ///
    /// `data`는 종목 코드 목록이며 종목 코드는 ASCII 문자로만 구성되어야 합니다.
    pub fn unsubscribe(&self, tr_code: &str, data: Vec<String>) -> Result<(), ()> {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.unsubscribe(tr_code, data)
    }

    /// 실시간 TR을 모두 등록 해제합니다.
    pub fn unsubscribe_all(&self) -> Result<(), ()> {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.unsubscribe_all()
    }

    /// 서버로부터 수신받은 실시간 TR을 큐에서 가져올 때까지 기다립니다.
    pub fn recv(&self) -> RealResponse {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.recv()
    }

    /// 지정된 시간 동안 서버로부터 수신받은 실시간 TR을 큐에서 가져올 때까지 기다립니다.
    ///
    /// 시간 초과가 발생하는 경우 `None`을 반환합니다.
    pub fn recv_timeout(&self, timeout: Duration) -> Option<RealResponse> {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.recv_timeout(timeout)
    }

    /// 서버로부터 수신받은 실시간 TR을 큐에서 가져오기를 시도합니다.
    ///
    /// 큐가 비어 있는 경우 `None`을 반환합니다.
    pub fn try_recv(&self) -> Option<RealResponse> {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.try_recv()
    }
}
