// SPDX-License-Identifier: MPL-2.0

use super::{caller::Caller, error::Win32Error};

use std::sync::Arc;
use winapi::{ctypes::c_void, shared::minwindef::DWORD, shared::windef::HWND};

#[derive(Clone)]
pub struct Window {
    caller: Arc<Caller>,
    hwnd: usize,
}

impl Window {
    pub async fn new(caller: Arc<Caller>, class_name: &[i8]) -> Result<Arc<Self>, Win32Error> {
        // 힙 영역 메모리 할당을 먼저 합니다.
        let mut window = Arc::new(Window { caller: caller.clone(), hwnd: 0 });
        let hwnd = caller.create_window(class_name).await?;

        // 윈도우 생성 이후 패닉이 발생하면 윈도우가 소멸되지 않기 때문입니다.
        if hwnd != 0 {
            Arc::get_mut(&mut window).unwrap().hwnd = hwnd;
            Ok(window)
        } else {
            Err(Win32Error::from_last_error())
        }
    }

    pub fn handle(&self) -> HWND {
        self.hwnd as HWND
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        self.caller.destroy_window(self.hwnd);
    }
}

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
        LocalFree(buf as *mut c_void);

        message
    }
}
