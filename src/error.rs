// SPDX-License-Identifier: MPL-2.0

//! 일반적인 에러 모듈입니다.

pub use crate::data::error::{DecodeError, EncodeError};

#[cfg(windows)]
pub use crate::os::windows::error::Win32Error;

type ErrorBox = Box<dyn std::error::Error + Send + Sync + 'static>;

/// XingAPI의 에러 종류에 대한 열거형 객체입니다.
///
/// 자주 발생되는 에러를 좀 더 쉽게 처리할 수 있습니다.
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
    /// 유효하지 않은 계좌
    InvalidAccount,
    /// 유효하지 않은 인수 및 입력
    InvalidInput,
    /// 유효하지 않은 데이터
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

/// XingAPI를 사용하면서 발생할 수 있는 모든 에러에 대한 열거형 객체입니다.
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
    /// RES 불러오기 에러
    Res(xingapi_res::LoadError),
    /// DLL 불러오기 에러
    Entry(EntryError),
    /// 기타 에러
    Other(ErrorBox),
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
            _ => ErrorKind::Other,
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

impl From<xingapi_res::LoadError> for Error {
    fn from(err: xingapi_res::LoadError) -> Self {
        Self::Res(err)
    }
}

impl From<EntryError> for Error {
    fn from(err: EntryError) -> Self {
        Self::Entry(err)
    }
}

impl From<ErrorBox> for Error {
    fn from(err: ErrorBox) -> Self {
        Self::Other(err)
    }
}

#[cfg(target_os = "windows")]
impl From<Win32Error> for Error {
    fn from(err: Win32Error) -> Self {
        Self::Other(Box::new(err))
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
            Self::Res(err) => {
                write!(f, "res load error: ")?;
                err.fmt(f)
            }
            Self::Entry(err) => {
                write!(f, "entry error: ")?;
                err.fmt(f)
            }
            Self::Other(err) => {
                write!(f, "other error: ")?;
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
            Self::Res(err) => Some(err),
            Self::Entry(err) => Some(err),
            Self::Other(err) => Some(err.as_ref()),
            Self::TimedOut => None,
        }
    }
}

/// DLL 로드 과정에서 발생한 에러에 대한 객체입니다.
#[derive(Debug)]
pub enum EntryError {
    Library {
        /// DLL 경로
        path: String,
        /// 에러
        error: libloading::Error,
    },
    Symbol {
        /// DLL 심볼
        symbol: String,
        /// DLL 경로
        path: String,
        /// 에러
        error: libloading::Error,
    },
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
        }
    }
}

impl std::error::Error for EntryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Library { error, .. } => Some(error),
            Self::Symbol { error, .. } => Some(error),
        }
    }
}