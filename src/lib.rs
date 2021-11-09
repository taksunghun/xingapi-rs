// SPDX-License-Identifier: MPL-2.0

//! eBEST 증권의 XingAPI를 쉽고 안전하게 사용할 수 있는 래퍼 라이브러리입니다.
//!
//! 현재는 윈도우용 XingAPI만 지원하고 있습니다.
//!
//! # 요구 사항
//! - 시스템에 다음의 구성 요소가 설치되어 있어야 합니다.
//!   - 윈도우용 XingAPI SDK
//!   - VS2010 MFC 런타임
//! - 요청하고자 하는 TR의 'RES 파일'이 필요합니다.
//!
//! # 초기화 과정
//! - 먼저 XingAPI를 사용하기 위해 DLL을 불러옵니다.
//!
//!   ```rust
//!   xingapi::loader::load().unwrap();
//!   ```
//!
//! - 그리고 TR 요청에 필요한 TR 레이아웃도 불러옵니다.
//!
//!   ```rust
//!   let layout_tbl = xingapi::layout::load().unwrap();
//!   ```
//!
//! - 그 후에 서버 연결 및 로그인을 처리하시면 됩니다.
//!
//!   ```rust
//!   xingapi::connect(addr, port, Duration::from_secs(30)).unwrap();
//!
//!   let res = xingapi::login(id, pw, cert_pw, false).unwrap();
//!   if !res.is_ok() {
//!       panic!("login failed: {:?}", res);
//!   }
//!   ```

#![cfg_attr(doc_cfg, feature(doc_cfg))]

pub mod data;
pub mod layout;

#[cfg(windows)]
mod os;

#[cfg(windows)]
pub use os::windows::*;
