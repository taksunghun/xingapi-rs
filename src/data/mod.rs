// SPDX-License-Identifier: MPL-2.0

//! 데이터를 인코딩 및 디코딩하기 위한 모듈

#![allow(dead_code)]

mod tests;

use crate::layout::{BlockLayout, TrLayout};

use encoding_rs::EUC_KR;
use std::{collections::HashMap, ops::Index};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// HashMap을 초기화하는 매크로
///
/// 매크로의 모든 인자는 [`Into`][Into]를 통해 묵시적으로 변환됩니다.
///
/// ## 예제
/// ```rust
/// use std::collections::HashMap;
///
/// let block: HashMap<String, String> = hashmap! {
///     "shcode" => "096530",
///     "gubun" => "0",
/// };
/// ```
#[macro_export]
macro_rules! hashmap {
    ($($key:expr => $val:expr),*$(,)?) => {{
        use std::collections::HashMap;
        use std::iter::FromIterator;

        HashMap::from_iter([
            $(($key.into(), $val.into()),)*
        ])
    }};
}

/// 서버와 주고받는 데이터를 나타내는 객체
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Data {
    /// TR 코드
    pub tr_code: String,
    /// 데이터 종류
    pub data_type: DataType,
    /// 블록 테이블
    pub blocks: HashMap<String, Block>,
}

/// 데이터 종류 (요청 및 응답)
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DataType {
    /// 요청 데이터
    #[cfg_attr(feature = "serde", serde(rename = "input"))]
    Input,
    /// 응답 데이터
    #[cfg_attr(feature = "serde", serde(rename = "output"))]
    Output,
}

/// 블록을 나타내는 객체
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(untagged))]
pub enum Block {
    /// 단일 블록
    Block(HashMap<String, String>),
    /// 배열 블록
    Array(Vec<HashMap<String, String>>),
}

impl Block {
    /// 단일 블록 여부를 반환합니다.
    pub fn is_block(&self) -> bool {
        matches!(self, Self::Block(_))
    }

    /// 배열 블록 여부를 반환합니다.
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }

    /// 단일 블록에 대한 참조자를 반환힙니다.
    pub fn as_block(&self) -> Option<&HashMap<String, String>> {
        match self {
            Self::Block(block) => Some(block),
            Self::Array(_) => None,
        }
    }

    /// 단일 블록에 대한 가변 참조자를 반환힙니다.
    pub fn as_block_mut(&mut self) -> Option<&mut HashMap<String, String>> {
        match self {
            Self::Block(block) => Some(block),
            Self::Array(_) => None,
        }
    }

    /// 배열 블록에 대한 참조자를 반환합니다.
    pub fn as_array(&self) -> Option<&Vec<HashMap<String, String>>> {
        match self {
            Self::Array(array) => Some(array),
            Self::Block(_) => None,
        }
    }

    /// 배열 블록에 대한 가변 참조자를 반환합니다.
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<HashMap<String, String>>> {
        match self {
            Self::Array(array) => Some(array),
            Self::Block(_) => None,
        }
    }
}

impl Index<&str> for Block {
    type Output = str;
    fn index(&self, index: &str) -> &Self::Output {
        &self
            .as_block()
            .expect("expected a block but found an array")[index]
    }
}

impl Index<usize> for Block {
    type Output = HashMap<String, String>;
    fn index(&self, index: usize) -> &Self::Output {
        &self
            .as_array()
            .expect("expected an array but found a block")[index]
    }
}

/// 데이터를 디코딩에 실패하여 발생하는 에러
#[derive(Clone, Debug)]
pub enum DecodeError {
    /// 레이아웃이 없습니다.
    UnknownLayout(String),
    /// 레이아웃에 존재하지 않는 블록이 있습니다.
    UnknownBlock(String),
    /// 데이터 크기가 일치하지 않습니다.
    MismatchDataLength,
    /// 데이터에 명시된 배열 크기가 유효하지 않습니다.
    InvalidArrayLength,
    /// EUC-KR 문자열에 잘못된 형식의 문자가 존재합니다.
    MalformedString,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnknownLayout(name) => {
                write!(f, "unknown layout: {}", name)
            }
            Self::UnknownBlock(name) => {
                write!(f, "unknown block: {}", name)
            }
            Self::MismatchDataLength => "mismatch data length".fmt(f),
            Self::InvalidArrayLength => "invalid array length".fmt(f),
            Self::MalformedString => "malformed euc-kr string".fmt(f),
        }
    }
}

impl std::error::Error for DecodeError {}

/// 데이터를 인코딩에 실패하여 발생하는 에러
#[derive(Clone, Debug)]
pub enum EncodeError {
    /// 레이아웃의 TR 코드가 일치하지 않습니다.
    MismatchLayout,
    /// 블록이 누락되었습니다.
    MissingBlock { block: String },
    /// 블록 타입이 일치하지 않습니다.
    MismatchBlockType { block: String },
    /// 블록 배열이 최대 크기에 도달했습니다.
    ExceedArrayLength { block: String },
    /// 필드가 누락되었습니다.
    MissingField { block: String, field: String },
    /// 필드가 최대 크기에 도달했습니다.
    ExceedFieldLength { block: String, field: String },
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MismatchLayout => "mismatch layout".fmt(f),
            Self::MissingBlock { block } => {
                write!(f, "missing {} block", block)
            }
            Self::MismatchBlockType { block } => {
                write!(f, "mismatch type of {} block", block)
            }
            Self::ExceedArrayLength { block } => {
                write!(f, "reached max length of {} block array", block)
            }
            Self::MissingField { block, field } => {
                write!(f, "missing {} field in {} block", field, block)
            }
            Self::ExceedFieldLength { block, field } => {
                write!(
                    f,
                    "reached max length of {} field in {} block",
                    field, block
                )
            }
        }
    }
}

impl std::error::Error for EncodeError {}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum RawData {
    Block(HashMap<String, Vec<u8>>),
    NonBlock(Vec<u8>),
}

// 응답 데이터를 디코딩합니다.
pub(crate) fn decode(tr_layout: &TrLayout, raw_data: RawData) -> Result<Data, DecodeError> {
    match raw_data {
        RawData::Block(raw_block_tbl) => {
            assert!(tr_layout.block_mode);

            let mut blocks = HashMap::new();

            for (block_name, raw_block) in raw_block_tbl {
                let block_layout = tr_layout
                    .out_blocks
                    .iter()
                    .find(|b| b.name == block_name)
                    .ok_or_else(|| DecodeError::UnknownBlock(block_name.clone()))?;

                blocks.insert(
                    block_name,
                    if block_layout.occurs {
                        decode_block_array(tr_layout, block_layout, &raw_block)?
                    } else {
                        decode_block(tr_layout, block_layout, &raw_block)?
                    },
                );
            }

            Ok(Data {
                tr_code: tr_layout.code.clone(),
                data_type: DataType::Output,
                blocks,
            })
        }
        RawData::NonBlock(raw_data) => {
            assert!(!tr_layout.block_mode);

            decode_non_block(tr_layout, DataType::Output, &raw_data)
        }
    }
}

// block mode인 응답 데이터의 단일 블록을 디코딩합니다.
fn decode_block(
    tr_layout: &TrLayout,
    block_layout: &BlockLayout,
    raw_block: &[u8],
) -> Result<Block, DecodeError> {
    assert!(tr_layout.block_mode && !block_layout.occurs);

    if raw_block.len() != block_layout.len {
        return Err(DecodeError::MismatchDataLength);
    }

    let mut fields = HashMap::with_capacity(block_layout.fields.len());
    let mut offset = 0;

    for field_layout in &block_layout.fields {
        fields.insert(
            field_layout.name.clone(),
            decode_str(&raw_block[offset..offset + field_layout.len])?,
        );
        offset += field_layout.len + if tr_layout.attr_byte { 1 } else { 0 };
    }

    Ok(Block::Block(fields))
}

// block mode인 응답 데이터의 배열 블록을 디코딩합니다.
fn decode_block_array(
    tr_layout: &TrLayout,
    block_layout: &BlockLayout,
    raw_block: &[u8],
) -> Result<Block, DecodeError> {
    assert!(tr_layout.block_mode && block_layout.occurs);

    if raw_block.len() % block_layout.len != 0 {
        return Err(DecodeError::MismatchDataLength);
    }

    let blocks_len = raw_block.len() / block_layout.len;

    let mut blocks = Vec::with_capacity(blocks_len);
    let mut offset = 0;

    for _ in 0..blocks_len {
        let mut fields = HashMap::with_capacity(block_layout.fields.len());
        for field_layout in &block_layout.fields {
            fields.insert(
                field_layout.name.clone(),
                decode_str(&raw_block[offset..offset + field_layout.len])?,
            );
            offset += field_layout.len + if tr_layout.attr_byte { 1 } else { 0 };
        }

        blocks.push(fields);
    }

    Ok(Block::Array(blocks))
}

// non-block mode인 데이터를 디코딩합니다.
pub(crate) fn decode_non_block(
    tr_layout: &TrLayout,
    data_type: DataType,
    raw_data: &[u8],
) -> Result<Data, DecodeError> {
    assert!(!tr_layout.block_mode);

    let mut blocks = HashMap::new();
    let mut offset = 0;

    for block_layout in &tr_layout.out_blocks {
        let block = if block_layout.occurs {
            if offset + 5 > raw_data.len() {
                return Err(DecodeError::MismatchDataLength);
            }

            let blocks_len: usize = str::parse(
                &EUC_KR
                    .decode_without_bom_handling_and_without_replacement(
                        &raw_data[offset..offset + 5],
                    )
                    .ok_or(DecodeError::InvalidArrayLength)?,
            )
            .map_err(|_| DecodeError::InvalidArrayLength)?;

            offset += 5;

            if offset + block_layout.len * blocks_len > raw_data.len() {
                return Err(DecodeError::MismatchDataLength);
            }

            let mut blocks = Vec::with_capacity(blocks_len);

            for _ in 0..blocks_len {
                let mut fields = HashMap::with_capacity(block_layout.fields.len());
                for field_layout in &block_layout.fields {
                    fields.insert(
                        field_layout.name.clone(),
                        decode_str(&raw_data[offset..offset + field_layout.len])?,
                    );

                    offset += field_layout.len + if tr_layout.attr_byte { 1 } else { 0 };
                }

                blocks.push(fields);
            }

            Block::Array(blocks)
        } else {
            if offset + block_layout.len > raw_data.len() {
                return Err(DecodeError::MismatchDataLength);
            }

            let mut fields = HashMap::with_capacity(block_layout.fields.len());
            for field_layout in &block_layout.fields {
                fields.insert(
                    field_layout.name.clone(),
                    decode_str(&raw_data[offset..offset + field_layout.len])?,
                );
                offset += field_layout.len + if tr_layout.attr_byte { 1 } else { 0 };
            }

            Block::Block(fields)
        };

        blocks.insert(block_layout.name.clone(), block);
    }

    Ok(Data {
        tr_code: tr_layout.code.clone(),
        data_type,
        blocks,
    })
}

fn decode_str(data: &[u8]) -> Result<String, DecodeError> {
    EUC_KR
        .decode_without_bom_handling_and_without_replacement(data)
        .map(|s| s.trim_matches(|c| (c as u32) < 0x20 || c == ' ').to_owned())
        .ok_or(DecodeError::MalformedString)
}

// non-block mode로 데이터를 인코딩합니다.
pub(crate) fn encode(data: &Data, tr_layout: &TrLayout) -> Result<Vec<u8>, EncodeError> {
    if data.tr_code != tr_layout.code {
        return Err(EncodeError::MismatchLayout);
    }

    let block_layouts = match data.data_type {
        DataType::Input => &tr_layout.in_blocks,
        DataType::Output => &tr_layout.out_blocks,
    };

    let mut enc_data: Vec<u8> = Vec::new();

    for block_layout in block_layouts {
        let missing_block = || -> EncodeError {
            EncodeError::MissingBlock {
                block: block_layout.name.clone(),
            }
        };
        let mismatch_block_type = || -> EncodeError {
            EncodeError::MismatchBlockType {
                block: block_layout.name.clone(),
            }
        };

        if block_layout.occurs {
            let arr_block = data
                .blocks
                .get(&block_layout.name)
                .ok_or_else(missing_block)?
                .as_array()
                .ok_or_else(mismatch_block_type)?;

            if !tr_layout.block_mode {
                // 블럭의 최대 개수는 십진수로 5자리
                if arr_block.len() >= 100000 {
                    return Err(EncodeError::ExceedArrayLength {
                        block: block_layout.name.clone(),
                    });
                }

                enc_data.extend(format!("{:0>5}", arr_block.len()).as_bytes());
            }

            for block in arr_block.iter() {
                encode_block(tr_layout, block_layout, block, &mut enc_data)?;
            }
        } else {
            let block = data
                .blocks
                .get(&block_layout.name)
                .ok_or_else(missing_block)?
                .as_block()
                .ok_or_else(mismatch_block_type)?;

            encode_block(tr_layout, block_layout, block, &mut enc_data)?;
        }
    }

    Ok(enc_data)
}

fn encode_block(
    tr_layout: &TrLayout,
    block_layout: &BlockLayout,
    block: &HashMap<String, String>,
    enc_data: &mut Vec<u8>,
) -> Result<(), EncodeError> {
    for field_layout in &block_layout.fields {
        let field = block
            .get(&field_layout.name)
            .or_else(|| block.get(&field_layout.name_old))
            .ok_or_else(|| EncodeError::MissingField {
                block: block_layout.name.clone(),
                field: field_layout.name.clone(),
            })?;

        let mut enc_field = EUC_KR.encode(field).0.to_vec();

        if enc_field.len() > field_layout.len {
            return Err(EncodeError::ExceedFieldLength {
                block: block_layout.name.clone(),
                field: field_layout.name.clone(),
            });
        }

        if tr_layout.attr_byte {
            enc_field.resize(field_layout.len + 1, b'\0');
        } else {
            enc_field.resize(field_layout.len, b'\0');
        }

        enc_data.extend(enc_field.iter());
    }

    Ok(())
}
