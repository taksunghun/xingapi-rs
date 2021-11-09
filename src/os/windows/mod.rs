// SPDX-License-Identifier: MPL-2.0

mod entry;
mod event;
mod executor;
mod raw;
mod session;

pub use self::event::RealEvent;

use crate::data::{Data, DecodeError, EncodeError};
use crate::layout::TrLayout;

use std::{path::PathBuf, time::Duration};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// DLL 로더 모듈
///
/// XingAPI 함수를 호출하기 전에 DLL을 불러오기 위해 사용합니다.
///
/// XingAPI 구버전의 경우 DLL을 불러온 후 언로드하지 않으면 버그로 인해
/// 프로그램이 정상적으로 종료되지 않을 수도 있습니다.
pub mod loader {
    use super::{executor, session, LoadError};

    use std::path::{Path, PathBuf};

    /// XingAPI SDK의 기본 설치 경로에서 DLL을 불러옵니다.
    ///
    /// DLL을 이미 불러온 경우 아무런 동작을 하지 않습니다.
    ///
    /// 만일 기본 설치 경로에서 불러오지 못한 경우 윈도우 운영체제가 DLL 파일을
    /// 검색하도록 합니다. 실행 파일과 같은 디렉터리인 경우 불러올 수 있지만,
    /// 보안상의 이유로 아무 위치에서나 불러오지는 못합니다.
    pub fn load() -> Result<(), LoadError> {
        executor::load(None)?;
        if let Err(err) = session::load() {
            executor::unload();
            return Err(err.into());
        }

        Ok(())
    }

    /// 특정 위치로 XingAPI DLL을 불러옵니다.
    ///
    /// DLL을 이미 불러온 경우 아무런 동작을 하지 않습니다.
    pub fn load_with_path<P: AsRef<Path>>(path: &P) -> Result<(), LoadError> {
        executor::load(Some(path.as_ref().to_owned()))?;
        if let Err(err) = session::load() {
            executor::unload();
            return Err(err.into());
        }

        Ok(())
    }

    /// 불러온 XingAPI DLL이 존재하는 경우 언로드합니다.
    pub fn unload() {
        session::unload();
        executor::unload();
    }

    /// XingAPI DLL이 불러와졌는지 여부를 반환합니다.
    ///
    /// 이 값이 반환된 후 처리하기 전에 즉시 상태가 바뀔 수도 있기 때문에
    /// 주의해서 사용해야 합니다.
    pub fn is_loaded() -> bool {
        executor::is_loaded() && session::is_loaded()
    }

    /// 불러온 XingAPI DLL이 존재하는 경우 DLL 경로를 반환합니다.
    ///
    /// 상대 경로를 반환할 수도 있습니다.
    pub fn loaded_path() -> Option<PathBuf> {
        executor::loaded_path()
    }
}

/// 서버에 연결합니다.
pub fn connect(addr: &str, port: u16, timeout: Duration) -> Result<(), Error> {
    session::global().connect(addr, port, timeout)
}

/// 서버 연결 여부를 반환합니다.
pub fn is_connected() -> bool {
    executor::global().handle().is_connected()
}

/// 서버와의 연결을 종료합니다.
pub fn disconnect() {
    session::global().disconnect()
}

/// 서버에 로그인 요청을 합니다.
///
/// 모의투자 서버에 접속한 경우 공동인증서 비밀번호는 무시됩니다.
pub fn login(
    id: &str,
    pw: &str,
    cert_pw: &str,
    cert_err_dialog: bool,
) -> Result<LoginResponse, Error> {
    session::global().login(id, pw, cert_pw, cert_err_dialog)
}

/// 서버에 조회 TR 요청을 합니다.
pub fn request(
    data: &Data,
    tr_layout: &TrLayout,
    next_key: Option<&str>,
    timeout: Duration,
) -> Result<QueryResponse, Error> {
    session::global().request(data, tr_layout, next_key, timeout)
}

/// 계좌 목록을 반환합니다.
pub fn accounts() -> Vec<Account> {
    executor::global().handle().accounts()
}

/// 통신 매체를 반환합니다.
pub fn comm_media() -> Option<String> {
    executor::global().handle().get_comm_media()
}

/// 당사 매체를 반환합니다.
pub fn etk_media() -> Option<String> {
    executor::global().handle().get_etk_media()
}

/// 서버 이름을 반환합니다.
pub fn server_name() -> Option<String> {
    executor::global().handle().get_server_name()
}

/// 선물 관련 요청 가능 여부를 반환합니다.
pub fn is_future_allowed() -> bool {
    executor::global().handle().get_use_over_future()
}

/// FX 관련 요청 가능 여부를 반환합니다.
pub fn is_fx_allowed() -> bool {
    executor::global().handle().get_use_fx()
}

/// TR의 초당 요청 제한 횟수를 반환합니다.
pub fn tr_limit_per_sec(tr_code: &str) -> Option<i32> {
    executor::global().handle().get_tr_count_per_sec(tr_code)
}

/// TR의 요청당 대기 초를 반환합니다.
pub fn tr_limit_wait_sec(tr_code: &str) -> Option<i32> {
    executor::global().handle().get_tr_count_base_sec(tr_code)
}

/// TR의 10분 내 요청한 횟수를 반환합니다.
pub fn tr_count_in_ten_min(tr_code: &str) -> Option<i32> {
    executor::global().handle().get_tr_count_request(tr_code)
}

/// TR의 10분 내 제한 횟수를 반환합니다.
pub fn tr_limit_per_ten_min(tr_code: &str) -> Option<i32> {
    executor::global().handle().get_tr_count_limit(tr_code)
}

/// 이베스트투자증권 계좌 정보
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Account {
    /// 계좌번호
    pub code: String,
    /// 계좌명
    pub name: String,
    /// 계좌 상세명
    pub detailed_name: String,
    /// 계좌 별명
    pub nickname: String,
}

/// XingAPI 함수가 실패하여 발생하는 에러
#[derive(Debug)]
pub enum Error {
    /// XingAPI 에러
    XingApi {
        /// 음수로 표현되는 에러 코드
        code: i32,
        /// 에러 메시지
        message: String,
    },
    /// 인코딩 에러
    Encode(EncodeError),
    /// 디코딩 에러
    Decode(DecodeError),
    /// 시간 초과
    TimedOut,
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
                write!(f, "xingapi error; code: {}, message: {}", code, message)
            }
            Self::Encode(err) => err.fmt(f),
            Self::Decode(err) => err.fmt(f),
            Self::TimedOut => "request timed out".fmt(f),
        }
    }
}

impl std::error::Error for Error {}

/// XingAPI를 불러오는데 실패하여 발생하는 에러
#[derive(Debug)]
pub enum LoadError {
    /// DLL 에러
    Dll(DllError),
    /// I/O 에러
    Io(std::io::Error),
}

impl From<DllError> for LoadError {
    fn from(err: DllError) -> Self {
        Self::Dll(err)
    }
}

impl From<std::io::Error> for LoadError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dll(err) => {
                write!(f, "dll error: {}", err)
            }
            Self::Io(err) => {
                write!(f, "io error: {}", err)
            }
        }
    }
}

impl std::error::Error for LoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dll(err) => Some(err),
            Self::Io(err) => Some(err),
        }
    }
}

/// DLL을 불러오는데 실패하여 발생하는 에러
#[derive(Debug)]
pub enum DllError {
    /// 라이브러리 에러
    Library {
        /// DLL 경로
        path: PathBuf,
        /// 에러 내용
        error: libloading::Error,
    },
    /// 심볼 에러
    Symbol {
        /// 심볼 이름
        symbol: String,
        /// DLL 경로
        path: PathBuf,
        /// 에러 내용
        error: libloading::Error,
    },
    /// DLL이 현재 프로세스에서 이미 사용 중임
    LibraryInUse,
}

impl std::fmt::Display for DllError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Library { path, error } => {
                write!(f, "could not load a library; ")?;
                write!(f, "path: {}, error: {}", path.display(), error)
            }
            Self::Symbol {
                symbol,
                path,
                error,
            } => {
                write!(f, "could not load a symbol: {}; ", symbol)?;
                write!(f, "path: {}, error: {}", path.display(), error)
            }
            Self::LibraryInUse => {
                write!(f, "a library is already in use in current process")
            }
        }
    }
}

impl std::error::Error for DllError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Library { error, .. } | Self::Symbol { error, .. } => Some(error),
            Self::LibraryInUse => None,
        }
    }
}

/// 응답에 대한 트레이트
///
/// 서버에서 발생하는 응답의 공통 부분인 코드와 메시지를 트레이트로 묶어서
/// 제공합니다.
pub trait Response {
    /// 4자리 이상의 응답 코드를 반환합니다. 응답 메시지가 없는 경우 빈 문자열을
    /// 반환합니다.
    ///
    /// | 코드        | 내용        |
    /// | ----------- | ----------- |
    /// | 0000 - 0999 | 정상        |
    /// | 1000 - 7999 | 업무 오류   |
    /// | 8000 - 9999 | 시스템 오류 |
    fn code(&self) -> &str;

    /// 응답 메시지를 반환합니다. 응답 메시지가 없는 경우 빈 문자열을
    /// 반환합니다.
    fn message(&self) -> &str;

    /// 정상 처리 여부를 반환합니다.
    ///
    /// 제공되는 구현은 응답 코드가 `0 <= x < 1000`이거나 응답 메시지와 코드가
    /// 비어 있으면 참으로 간주합니다.
    ///
    /// t1764 TR과 같이 정상 처리시에 응답 메시지와 코드가 발생하지 않는 경우도
    /// 고려하였습니다.
    fn is_ok(&self) -> bool {
        if let Ok(code) = self.code().parse::<i32>() {
            (0..1000).contains(&code)
        } else {
            self.code().is_empty() && self.message().is_empty()
        }
    }

    /// 처리 실패 여부를 반환합니다.
    ///
    /// 제공되는 구현은 `is_ok()`의 논리 부정 값을 반환합니다.
    fn is_err(&self) -> bool {
        !self.is_ok()
    }
}

/// 로그인 요청에 대한 서버 응답
#[derive(Clone, Debug)]
pub struct LoginResponse {
    code: String,
    message: String,
}

impl Response for LoginResponse {
    fn code(&self) -> &str {
        &self.code
    }
    fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for LoginResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

/// 조회 TR에 대한 서버 응답
#[derive(Clone, Debug)]
pub struct QueryResponse {
    code: String,
    message: String,
    elapsed: Duration,
    next_key: Option<String>,
    data: Option<Result<Data, DecodeError>>,
}

impl QueryResponse {
    /// 서버 요청 후 응답까지 소요된 시간을 밀리초 정확도로 반환합니다.
    ///
    /// XingAPI의 수신 이벤트에서 반환한 값을 사용합니다.
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// 연속 조회 키가 존재하는 경우 연속 조회 키를 반환합니다.
    ///
    /// 연속 조회 키는 TR당 하나입니다.
    pub fn next_key(&self) -> Option<&str> {
        self.next_key.as_deref()
    }

    /// 수신한 데이터에 대한 디코딩 결과를 반환합니다.
    ///
    /// [`Response::is_ok()`][Response::is_ok]가 거짓인 경우 패닉이 발생합니다.
    pub fn data(&self) -> Result<&Data, DecodeError> {
        self.data
            .as_ref()
            .expect("this response has no data")
            .as_ref()
            .map_err(|err| err.clone())
    }
}

impl Response for QueryResponse {
    fn code(&self) -> &str {
        &self.code
    }
    fn message(&self) -> &str {
        &self.message
    }
}

/// 실시간 TR에 대한 서버의 응답
#[derive(Clone, Debug)]
pub struct RealResponse {
    key: String,
    data: Result<Data, DecodeError>,
}

impl RealResponse {
    /// 실시간 TR을 등록하는데 사용한 키를 반환합니다.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// 수신한 데이터에 대한 디코딩 결과를 반환합니다.
    pub fn data(&self) -> Result<&Data, DecodeError> {
        self.data.as_ref().map_err(|err| err.clone())
    }
}

trait Byte: Sized {}
impl Byte for u8 {}
impl Byte for i8 {}

fn decode_euckr<T: Byte>(data: &[T]) -> String {
    let data = unsafe { std::slice::from_raw_parts(data.as_ptr().cast(), data.len()) };

    let len = data
        .iter()
        .enumerate()
        .find(|&(_, &ch)| ch == b'\0')
        .map_or_else(|| data.len(), |(i, _)| i);

    encoding_rs::EUC_KR
        .decode_without_bom_handling(&data[..len])
        .0
        .trim_matches(|c| (c as u32) < 0x20 || c == ' ')
        .to_owned()
}
