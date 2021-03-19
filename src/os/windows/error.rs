// SPDX-License-Identifier: MPL-2.0

//! 윈도우 운영체제 구현에서 발생하는 에러 모듈입니다.

use super::bindings::DWORD;

#[cfg(windows)]
use super::bindings::GetLastError;

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

#[cfg(windows)]
pub fn format_message(code: DWORD) -> String {
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
            &mut buf as *mut *mut u16 as _,
            0,
            std::ptr::null_mut(),
        );
        assert_ne!(buf_len, 0);

        let message =
            String::from_utf16(std::slice::from_raw_parts(buf, buf_len as usize)).unwrap();
        LocalFree(buf as *mut _);

        message
    }
}

#[cfg(all(windows, test))]
mod tests {
    use super::format_message;

    #[test]
    fn test_format_message() {
        println!("{:?}", format_message(0));
    }
}
