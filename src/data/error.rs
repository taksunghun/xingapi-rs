// SPDX-License-Identifier: MPL-2.0

//! [`Data`](crate::data::Data)를 인코딩하거나 디코딩할 때 발생하는 에러에 대한 모듈입니다.
//!
//! 이러한 에러들은 RES 파일이 오래되어 서버와 달라 발생할 수도 있습니다.

/// 응답 데이터를 디코딩할 때 발생하는 에러입니다.
#[derive(Clone, Debug)]
pub enum DecodeError {
    /// TR 코드를 불러오지 않았거나 알 수 없습니다.
    UnknownTrCode,
    /// block이 누락되었습니다.
    UnknownBlockName { block_name: String },
    /// 데이터 길이가 맞지 않습니다.
    MismatchBufferLength,
    /// occurs(배열) 길이를 디코딩할 수 없습니다.
    DecodeOccursLength,
    /// CP949 데이터를 UTF-8로 디코딩할 수 없습니다.
    DecodeString,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownTrCode => write!(f, "unknown tr code"),
            Self::UnknownBlockName { block_name } => {
                write!(f, "unknown block name: {}", block_name)
            }
            Self::MismatchBufferLength => write!(f, "mismatch buffer length"),

            Self::DecodeOccursLength => write!(f, "could not decode occurs length"),
            Self::DecodeString => write!(f, "could not decode string"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// 데이터를 인코딩할 때 발생하는 에러입니다.
#[derive(Clone, Debug)]
pub enum EncodeError {
    /// TR 코드를 불러오지 않았거나 알 수 없습니다.
    UnknownTrCode,
    /// block이 누락되었습니다.
    MissingBlock { block_name: String },
    /// block 최대 개수를 초과했습니다.
    ///
    /// 개수는 5자리 이하의 자연수여야 합니다.
    ExceedMaxBlockCount { block_name: String },
    /// field가 누락되었습니다.
    MissingField { block_name: String, field_name: String },
    /// field의 최대 길이를 초과했습니다.
    ExceedFieldLength { block_name: String, field_name: String },
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownTrCode => write!(f, "unknown tr code"),
            Self::MissingBlock { block_name } => write!(f, "missing block: {}", block_name),
            Self::MissingField { block_name, field_name } => {
                write!(f, "missing field: {} in {}", field_name, block_name)
            }
            Self::ExceedMaxBlockCount { block_name } => {
                write!(f, "max block count exceeded: {}", block_name)
            }
            Self::ExceedFieldLength { block_name, field_name } => {
                write!(f, "field length exceeded: {} in {}", field_name, block_name)
            }
        }
    }
}

impl std::error::Error for EncodeError {}
