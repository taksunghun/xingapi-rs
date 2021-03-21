// SPDX-License-Identifier: MPL-2.0

//! 운영체제별 구현입니다.

#[cfg(target_os = "windows")]
pub mod windows;
