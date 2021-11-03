// SPDX-License-Identifier: MPL-2.0

//! 레이아웃 관련 에러 모듈

use super::read::{Position, Read};
use std::path::PathBuf;

/// 레이아웃 파싱이 실패하여 발생하는 에러
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    line: usize,
    column: usize,
}

impl Error {
    /// 에러 종류를 반환합니다.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// 에러가 발생한 행 위치를 반환합니다.
    pub fn line(&self) -> usize {
        self.line
    }

    /// 에러가 발생한 열 위치를 반환합니다.
    ///
    /// 탭 문자의 너비는 4이고 위치에 따라 들여쓰기로 추가되는 열 수가
    /// 결정됩니다.
    pub fn column(&self) -> usize {
        self.column
    }

    pub(crate) fn unexpected_syntax<'a, R: Read<'a>>(reader: &R) -> Error {
        Error::new(reader.position(), ErrorKind::Syntax)
    }

    pub(crate) fn unexpected_data<'a, R: Read<'a>>(reader: &R) -> Error {
        Error::new(reader.position(), ErrorKind::Data)
    }

    pub(crate) fn unexpected_eof<'a, R: Read<'a>>(reader: &R) -> Error {
        Error::new(reader.position(), ErrorKind::Eof)
    }

    pub(crate) fn new(pos: Position, kind: ErrorKind) -> Self {
        Self {
            line: pos.line(),
            column: pos.column(),
            kind,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} at line {} column {}",
            self.kind, self.line, self.column
        )
    }
}

impl std::error::Error for Error {}

/// 레이아웃 파싱에 실패하여 발생하는 에러의 종류
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ErrorKind {
    /// 구문 에러
    ///
    /// 특정한 식별자가 예상되었지만 다른 문자열이 발견된 경우입니다.
    ///
    /// 구문 내 식별자는 다음이 포함됩니다.
    /// - `BEGIN_FUNCTION_MAP` 및 `END_FUNCTION_MAP`
    /// - `BEGIN_DATA_MAP` 및 `END_DATA_MAP`
    /// - `begin` 및 `end`
    /// - `,` (콤마) 및 `;` (세미콜론)
    Syntax,

    /// 데이터 에러
    ///
    /// 콤마나 세미콜론으로 구분되는 데이터에 유효하지 않은 값이 발견되었거나
    /// 데이터 개수가 잘못된 경우입니다.
    Data,

    /// 예상치 못한 파일 끝 에러
    Eof,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Syntax => "unexpected syntax".fmt(f),
            Self::Data => "unexpected data".fmt(f),
            Self::Eof => "unexpected eof".fmt(f),
        }
    }
}

/// TR 레이아웃을 디렉터리에서 불러오는데 실패하여 발생하는 에러
#[derive(Debug)]
pub enum LoadError {
    /// 입출력 에러
    Io(std::io::Error),
    /// EUC-KR 디코딩 에러
    Encoding(PathBuf),
    /// TR 레이아웃 파싱 에러
    Parse(PathBuf, Error),
    /// TR 코드 중복 에러
    ///
    /// 코드는 같지만 서로 다른 두 레이아웃이 존재하는 경우 발생합니다.
    Confilict(String),
}

impl From<std::io::Error> for LoadError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => err.fmt(f),
            Self::Encoding(path) => {
                write!(f, "unable to decode file from euc-kr")?;
                write!(f, "; path: {}", path.display())
            }
            Self::Parse(path, err) => {
                write!(f, "unable to parse file")?;
                write!(f, "; path: {}, error: {}", path.display(), err)
            }
            Self::Confilict(layout) => {
                write!(f, "conflicts between files; name: {}", layout)
            }
        }
    }
}

impl std::error::Error for LoadError {}
