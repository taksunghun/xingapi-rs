// SPDX-License-Identifier: MPL-2.0

//! 윈도우 운영체제 구현에서 발생하는 에러 모듈입니다.

use super::bindings::DWORD;

#[cfg(windows)]
use super::{bindings::GetLastError, win32::format_message};

/// Win32 API 호출 과정에서 발생한 오류 객체입니다.
pub struct Win32Error {
    code: DWORD,
}

impl Win32Error {
    /// Win32 API 에러 코드를 반환합니다.
    pub fn code(&self) -> DWORD {
        self.code
    }

    #[cold]
    pub(crate) fn from_last_error() -> Self {
        windows_only_impl! {
            unsafe {
                Self { code: GetLastError() }
            }
        }
    }
}

impl std::fmt::Debug for Win32Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        windows_only_impl! {
            f.debug_struct("SystemError")
                .field("code", &self.code)
                .field("message", &format_message(self.code))
                .finish()
        }
    }
}

impl std::fmt::Display for Win32Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        windows_only_impl! {
            write!(f, "{} {:#010x}", format_message(self.code).trim_end(), self.code)
        }
    }
}

impl std::error::Error for Win32Error {}
