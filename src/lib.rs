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
pub mod real;
pub mod response;

mod euckr;
mod os;

use self::data::{Data, DecodeError, EncodeError};
use self::response::{LoginResponse, QueryResponse};

use std::path::{Path, PathBuf};
use std::sync::Arc;

use xingapi_res::TrLayout;

#[cfg(windows)]
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
pub struct XingApi(#[cfg(windows)] imp::XingApi);

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

/// XingAPI의 오류 종류에 대한 열거형 객체입니다.
///
/// 자주 발생되는 오류를 좀 더 쉽게 처리할 수 있습니다.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ErrorKind {
    /// 연결 오류
    Connection,
    /// 암복호화 오류
    Encryption,
    /// (시세전용이 아닌) 로그인이 필요함
    LoginRequired,
    /// (공동인증) 로그인 실패
    LoginFailed,
    /// 계좌가 유효하지 않음
    InvalidAccount,
    /// 인수가 올바르지 않음
    InvalidInput,
    /// 수신 데이터가 올바르지 않음
    InvalidData,
    /// 시간 초과
    TimedOut,
    /// 요청 또는 등록 제한 초과
    LimitReached,
    /// 기타 오류
    Other,
}

impl ErrorKind {
    pub fn from_code(code: i32) -> Self {
        match code {
            -1 | -2 | -14 | -16 => Self::Connection,
            -15 | -17 => Self::Encryption,
            -7 | -8 => Self::LoginRequired,
            -18 | -19 => Self::LoginFailed,
            -9 | -12 | -24 | -25 => Self::InvalidAccount,
            -3 | -10 | -22 | -23 | -28 => Self::InvalidInput,
            -11 => Self::InvalidData,
            -4 => Self::TimedOut,
            -21 | -27 => Self::LimitReached,
            _ => Self::Other,
        }
    }
}

/// 여러 오류에 대한 열거형 객체입니다.
#[derive(Debug)]
pub enum Error {
    /// XingAPI 오류
    XingApi {
        /// 음수로 표현되는 에러 코드
        code: i32,
        /// 에러 메시지
        message: String,
    },
    /// 인코딩 오류
    Encode(EncodeError),
    /// 디코딩 오류
    Decode(DecodeError),
    /// 시간 초과
    TimedOut,
}

impl Error {
    /// 에러 메시지 종류를 반환합니다.
    ///
    /// 자주 발생되는 에러를 좀 더 쉽게 처리할 수 있습니다.
    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::XingApi { code, .. } => ErrorKind::from_code(*code),
            Self::Encode(_) => ErrorKind::InvalidInput,
            Self::Decode(_) => ErrorKind::InvalidData,
            Self::TimedOut => ErrorKind::TimedOut,
        }
    }
}

impl From<EncodeError> for Error {
    fn from(err: EncodeError) -> Self {
        Self::Encode(err)
    }
}

impl From<DecodeError> for Error {
    fn from(err: DecodeError) -> Self {
        Self::Decode(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::XingApi { code, message } => {
                write!(f, "xingapi error: {} ({})", message, code)
            }
            Self::Encode(err) => {
                write!(f, "encode error: ")?;
                err.fmt(f)
            }
            Self::Decode(err) => {
                write!(f, "decode error: ")?;
                err.fmt(f)
            }
            Self::TimedOut => f.write_str("request timed out"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::XingApi { .. } => None,
            Self::Encode(err) => Some(err),
            Self::Decode(err) => Some(err),
            Self::TimedOut => None,
        }
    }
}

#[derive(Debug)]
pub enum LoadError {
    /// TR 레이아웃 파싱 오류
    Layout(xingapi_res::LoadError),
    /// DLL 불러오기 오류
    Entry(EntryError),
    /// 기타 오류
    #[cfg(any(windows, doc))]
    #[cfg_attr(doc_cfg, doc(cfg(windows)))]
    Win32(Win32Error),
}

impl From<xingapi_res::LoadError> for LoadError {
    fn from(err: xingapi_res::LoadError) -> Self {
        Self::Layout(err)
    }
}

impl From<EntryError> for LoadError {
    fn from(err: EntryError) -> Self {
        Self::Entry(err)
    }
}

#[cfg(windows)]
impl From<Win32Error> for LoadError {
    fn from(err: Win32Error) -> Self {
        Self::Win32(err)
    }
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Layout(err) => {
                write!(f, "layout error: {}", err)
            }
            Self::Entry(err) => {
                write!(f, "entry error: {}", err)
            }
            #[cfg(windows)]
            Self::Win32(err) => {
                write!(f, "win32 error: {}", err)
            }
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Layout(err) => Some(err),
            Self::Entry(err) => Some(err),
            #[cfg(windows)]
            Self::Win32(err) => Some(err),
        }
    }
}

/// DLL 불러오기 오류에 대한 객체입니다.
#[derive(Debug)]
pub enum EntryError {
    /// 라이브러리 불러오기 오류
    Library {
        /// DLL 경로
        path: PathBuf,
        /// 에러
        error: libloading::Error,
    },
    /// 기호 불러오기 오류
    Symbol {
        /// 기호명
        symbol: String,
        /// DLL 경로
        path: PathBuf,
        /// 에러
        error: libloading::Error,
    },
    /// 해당 라이브러리가 현재 프로세스에서 이미 사용 중임.
    #[cfg(any(windows, doc))]
    #[cfg_attr(doc_cfg, doc(cfg(windows)))]
    LibraryInUse,
}

impl std::fmt::Display for EntryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Library { path, error } => {
                write!(f, "could not load a library; ")?;
                write!(f, "path: {:?}, error: {}", path, error)
            }
            Self::Symbol { path, symbol, error } => {
                write!(f, "could not load a symbol: {:?}; ", symbol)?;
                write!(f, "path: {:?}, , error: {}", path, error)
            }
            #[cfg(any(windows, doc))]
            Self::LibraryInUse => {
                write!(f, "a library is already in use in current process")
            }
        }
    }
}

impl std::error::Error for EntryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Library { error, .. } => Some(error),
            Self::Symbol { error, .. } => Some(error),
            #[cfg(any(windows, doc))]
            Self::LibraryInUse => None,
        }
    }
}

/// Win32 API 호출 과정에서 발생하는 오류에 대한 객체입니다.
#[cfg(any(windows, doc))]
#[cfg_attr(doc_cfg, doc(cfg(windows)))]
pub struct Win32Error {
    code: u32,
}

#[cfg(any(windows, doc))]
impl Win32Error {
    /// Win32 에러 코드를 반환합니다.
    pub fn code(&self) -> u32 {
        self.code
    }

    #[cold]
    pub(crate) fn from_last_error() -> Self {
        #[cfg(windows)]
        {
            use winapi::um::errhandlingapi::GetLastError;
            unsafe { Self { code: GetLastError() } }
        }
    }
}

#[cfg(any(windows, doc))]
impl std::fmt::Debug for Win32Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        f.debug_struct("SystemError")
            .field("code", &self.code)
            .field("message", &format_message(self.code))
            .finish()
    }
}

#[cfg(any(windows, doc))]
impl std::fmt::Display for Win32Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        write!(f, "{} {:#010x}", format_message(self.code).trim_end(), self.code)
    }
}

#[cfg(any(windows, doc))]
impl std::error::Error for Win32Error {}

#[cfg(windows)]
fn format_message(code: u32) -> String {
    use winapi::um::winbase::{
        FormatMessageW, LocalFree, FORMAT_MESSAGE_ALLOCATE_BUFFER, FORMAT_MESSAGE_FROM_SYSTEM,
        FORMAT_MESSAGE_MAX_WIDTH_MASK,
    };

    unsafe {
        let message: Vec<u16> = "%0\0".encode_utf16().into_iter().collect();

        let mut buf: *mut u16 = std::ptr::null_mut();
        let buf_len = FormatMessageW(
            FORMAT_MESSAGE_MAX_WIDTH_MASK
                | FORMAT_MESSAGE_ALLOCATE_BUFFER
                | FORMAT_MESSAGE_FROM_SYSTEM,
            message.as_ptr() as *const _,
            code,
            0,
            &mut buf as *mut *mut _ as _,
            0,
            std::ptr::null_mut(),
        );
        assert_ne!(buf_len, 0);

        let message = String::from_utf16(std::slice::from_raw_parts(buf, buf_len as _)).unwrap();
        LocalFree(buf as *mut _);

        message
    }
}

#[cfg(test)]
mod tests {
    #[cfg(windows)]
    #[test]
    fn test_format_message() {
        use super::format_message;

        println!("{:?}", format_message(0));
    }
}
