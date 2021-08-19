// SPDX-License-Identifier: MPL-2.0

use crate::XingApi;
use crate::{response::RealResponse, LoadError};

use std::fmt::{self, Display};
use std::{sync::Arc, time::Duration};

#[cfg(target_os = "windows")]
use crate::os::windows as imp;

/// 실시간 TR를 수신하는 리시버입니다.
///
/// `connect()`, `disconnect()`, `login()`과 같은 연결 및 로그인 함수를 호출하면 기존에 등록된
/// TR은 모두 사라지게 됩니다.
///
/// 실시간 TR을 등록하면 수신받은 TR은 내부적으로 큐에 저장되며 이를 처리하지 않을 경우 메모리
/// 누수로 이어집니다. 따라서 `Real::recv()`를 호출하여 수신받은 TR을 반드시 처리해야 합니다.
#[cfg(any(windows, doc))]
#[cfg_attr(doc_cfg, doc(cfg(windows)))]
pub struct Real(#[cfg(windows)] imp::Real, Arc<XingApi>);

#[cfg(any(windows, doc))]
impl Real {
    /// 실시간 TR을 수신하는 객체를 생성합니다.
    pub fn new(xingapi: Arc<XingApi>) -> Result<Self, LoadError> {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        Ok(Self(imp::Real::new(&xingapi.0)?, xingapi))
    }

    /// 실시간 TR을 지정된 종목 코드로 등록합니다.
    ///
    /// `data`는 종목 코드 목록이며 종목 코드는 ASCII 문자로만 구성되어야 합니다.
    pub fn subscribe<T: AsRef<str>>(
        &self,
        tr_code: &str,
        tickers: &[T],
    ) -> Result<(), SubscribeError> {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.subscribe(tr_code, tickers)
    }

    /// 실시간 TR을 지정된 종목 코드로 등록 해제합니다.
    ///
    /// `data`는 종목 코드 목록이며 종목 코드는 ASCII 문자로만 구성되어야 합니다.
    pub fn unsubscribe<T: AsRef<str>>(
        &self,
        tr_code: &str,
        tickers: &[T],
    ) -> Result<(), UnsubscribeError> {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.unsubscribe(tr_code, tickers)
    }

    /// 실시간 TR을 모두 등록 해제합니다.
    pub fn unsubscribe_all(&self) -> Result<(), UnsubscribeError> {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.unsubscribe_all()
    }

    /// 서버로부터 수신받은 실시간 TR을 큐에서 가져옵니다.
    pub fn try_recv(&self) -> Result<RealResponse, TryRecvError> {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.try_recv()
    }

    /// 서버로부터 수신받은 실시간 TR을 큐에서 가져올 때까지 기다립니다.
    pub fn recv(&self) -> Result<RealResponse, RecvError> {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.recv()
    }

    /// 지정된 시간 동안 서버로부터 수신받은 실시간 TR을 큐에서 가져올 때까지 기다립니다.
    pub fn recv_timeout(&self, timeout: Duration) -> Result<RealResponse, RecvTimeoutError> {
        #[cfg(not(windows))]
        unimplemented!();

        #[cfg(windows)]
        self.0.recv_timeout(timeout)
    }
}

/// 실시간 TR에 대한 등록 요청이 실패하면 발생하는 에러입니다.
#[derive(Debug)]
pub struct SubscribeError;

impl Display for SubscribeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "unable to subscribe TR".fmt(f)
    }
}

impl std::error::Error for SubscribeError {}

/// 실시간 TR에 대한 등록 해제 요청이 실패하면 발생하는 에러입니다.
#[derive(Debug)]
pub struct UnsubscribeError;

impl Display for UnsubscribeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        "unable to unsubscribe TR".fmt(f)
    }
}

impl std::error::Error for UnsubscribeError {}

#[derive(Debug)]
pub enum TryRecvError {
    Empty,
    Disconnected,
}

impl std::fmt::Display for TryRecvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::Empty => "receiving on an empty channel".fmt(f),
            Self::Disconnected => "receiving on an empty and disconnected channel".fmt(f),
        }
    }
}

impl std::error::Error for TryRecvError {}

impl From<RecvError> for TryRecvError {
    fn from(_: RecvError) -> Self {
        Self::Disconnected
    }
}

#[derive(Debug)]
pub struct RecvError;

impl std::fmt::Display for RecvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        "receiving on an empty and disconnected channel".fmt(f)
    }
}

impl std::error::Error for RecvError {}

#[derive(Debug)]
pub enum RecvTimeoutError {
    Timeout,
    Disconnected,
}

impl std::fmt::Display for RecvTimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            RecvTimeoutError::Timeout => "timed out waiting on receive operation".fmt(f),
            RecvTimeoutError::Disconnected => "channel is empty and disconnected".fmt(f),
        }
    }
}

impl std::error::Error for RecvTimeoutError {}

impl From<RecvError> for RecvTimeoutError {
    fn from(_: RecvError) -> Self {
        Self::Disconnected
    }
}
