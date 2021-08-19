// SPDX-License-Identifier: MPL-2.0

//! 일반적인 에러 모듈입니다.

pub use crate::data::error::{DecodeError, EncodeError};

use std::path::PathBuf;

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
