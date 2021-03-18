// SPDX-License-Identifier: MPL-2.0

//! XingAPI library that supports async/await syntax and multithreading.
//!
//! This document was written almost in Korean because it is based on a service provided by Korea
//! Securities for users who can read Korean.
//!
//! 이 라이브러리는 다음의 주안점에 기초하여 제작되었습니다.
//!
//! - 비동기 함수 제공을 통한 직관적인 비동기 처리
//! - 데이터 및 I/O에 대한 고성능 처리
//! - 간편하고 쉬운 사용
//!
//! 라이브러리의 구현에 Rust 언어를 사용하게 된 결정적인 이유는 강력한 동시성 제어와 비동기 함수
//! 지원 때문입니다. C++도 C++20 버전부터 비동기 함수를 지원하기 시작했지만, 이를 지원하는
//! 라이브러리가 전무합니다.
//!
//! # Requirements
//! - 이베스트투자증권에서 회원들에게 제공하는 윈도우용 XingAPI 최신 버전
//! - TR에 필요한 RES (TR 레이아웃) 파일. DevCenter 프로그램에서 전부 다운로드 받을 수 있습니다.
//! - 비동기 함수를 실행하기 위한 실행자. 이를 위한 라이브러리로는 [async_std][async-std-docs],
//!   [futures][futures-docs], [tokio][tokio-docs] 등이 있습니다.
//!
//! XingAPI에는 리눅스 버전도 있지만 아직은 윈도우 32비트 버전의 XingAPI만 지원합니다.
//!
//! [async-std-docs]: https://docs.rs/async-std/
//! [futures-docs]: https://docs.rs/futures/
//! [tokio-docs]: https://docs.rs/tokio/

#![cfg_attr(feature = "doc_cfg", feature(doc_cfg))]

pub mod data;
pub mod error;
pub mod response;
pub mod os;

mod euckr;

#[cfg(target_os = "windows")]
pub use os::windows::{Real, XingApi, XingApiBuilder};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// 이베스트투자증권 계좌 정보입니다.
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
