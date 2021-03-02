// SPDX-License-Identifier: MPL-2.0

use crate::os::windows::win32::format_message;
use winapi::{shared::minwindef::DWORD, um::errhandlingapi::GetLastError};

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
        unsafe { Self { code: GetLastError() } }
    }
}

impl std::fmt::Debug for Win32Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemError")
            .field("code", &self.code)
            .field("message", &format_message(self.code))
            .finish()
    }
}

impl std::fmt::Display for Win32Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({:#010x})", format_message(self.code), self.code)
    }
}

impl std::error::Error for Win32Error {}

#[cfg(test)]
mod tests {
    use super::Win32Error;

    #[test]
    fn display_system_error() {
        let error = Win32Error { code: 0 };

        println!("{}", error);
    }
}
