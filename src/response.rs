// SPDX-License-Identifier: MPL-2.0

//! 서버 응답 모듈입니다.
//!
//! 응답의 코드나 메시지가 필요하다면 코드에 아래와 같이 [Message](Message) 트레이트를 추가해야
//! 합니다.
//! ```rust
//! use xingapi::response::Message;
//! ```

use crate::{data::Data, error::DecodeError};

use std::time::Duration;

/// 응답 메시지에 대한 트레이트입니다.
pub trait Message: std::fmt::Display {
    /// 4자리 이상의 응답 코드를 반환합니다. 응답 메시지가 없는 경우 빈 문자열을 반환합니다.
    ///
    /// 0000-0999: 정상, 1000-7999: 업무 오류, 8000-9999: 시스템 오류
    fn code(&self) -> &str;

    /// 응답 메시지를 반환합니다. 응답 메시지가 없는 경우 빈 문자열을 반환합니다.
    fn message(&self) -> &str;

    /// 정상 처리 여부를 반환합니다.
    ///
    /// t1764 TR과 같이 정상 처리시에 응답 메시지가 발생하지 않는 경우도 있습니다.
    fn is_ok(&self) -> bool {
        if let Ok(code) = self.code().parse::<u32>() {
            code < 1000
        } else {
            if self.code().is_empty() && self.message().is_empty() {
                true
            } else {
                false
            }
        }
    }

    /// 처리 실패 여부를 반환합니다.
    fn is_err(&self) -> bool {
        !self.is_ok()
    }
}

/// 로그인 요청에 대한 서버의 응답입니다.
#[derive(Clone, Debug)]
pub struct LoginResponse {
    code: String,
    message: String,
}

impl LoginResponse {
    pub(crate) fn new(code: &str, message: &str) -> Self {
        Self { code: code.trim_end().to_owned(), message: message.trim_end().to_owned() }
    }
}

impl Message for LoginResponse {
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

/// TR 요청에 대한 서버 응답입니다.
#[derive(Clone, Debug)]
pub struct QueryResponse {
    code: String,
    message: String,
    elapsed: Duration,
    continue_key: Option<String>,
    data: Option<Result<Data, DecodeError>>,
}

impl QueryResponse {
    /// 서버 요청 후 응답까지 소요된 시간을 밀리초 정확도로 반환합니다.
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// 연속 조회 키에 대한 `Option`을 반환합니다. 연속 조회 키는 TR당 하나입니다.
    pub fn continue_key(&self) -> Option<&str> {
        self.continue_key.as_deref()
    }

    /// 수신한 데이터에 대한 디코딩 결과를 반환합니다.
    ///
    /// Response가 에러인 경우 패닉이 발생합니다.
    pub fn data<'a>(&'a self) -> Result<&'a Data, &'a DecodeError> {
        self.data
            .as_ref()
            .expect("this response has no data. check if the response is an error.")
            .as_ref()
    }

    pub(crate) fn new(
        code: &str,
        message: &str,
        elapsed: i32,
        continue_key: Option<String>,
        data: Option<Result<Data, DecodeError>>,
    ) -> Self {
        Self {
            code: code.trim_end().to_owned(),
            message: message.trim_end().to_owned(),
            elapsed: Duration::from_millis(elapsed as _),
            continue_key,
            data,
        }
    }
}

impl Message for QueryResponse {
    fn code(&self) -> &str {
        &self.code
    }
    fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for QueryResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

/// 실시간 TR에 대한 서버의 응답입니다.
#[derive(Clone, Debug)]
pub struct RealResponse {
    key: String,
    reg_key: String,
    data: Result<Data, DecodeError>,
}

impl RealResponse {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn reg_key(&self) -> &str {
        &self.reg_key
    }

    /// 수신한 데이터에 대한 디코딩 결과를 반환합니다.
    pub fn data<'a>(&'a self) -> Result<&'a Data, &'a DecodeError> {
        self.data.as_ref()
    }

    pub(crate) fn new(key: String, reg_key: String, data: Result<Data, DecodeError>) -> Self {
        Self { key, reg_key, data }
    }
}

impl std::fmt::Display for RealResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}
