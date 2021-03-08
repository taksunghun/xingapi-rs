// SPDX-License-Identifier: MPL-2.0

//! 요청 및 응답 데이터 모듈입니다.

pub mod error;
mod tests;

use self::error::{DecodeError, EncodeError};
use crate::euckr;

use encoding_rs::EUC_KR;
use std::{borrow::Cow, collections::HashMap};
use xingapi_res::{BlockLayout, TrLayout};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// HashMap을 초기화하는 매크로입니다.
///
/// 들어오는 모든 값은 [Into](Into) 트레이트를 통해 묵시적으로 변환됩니다.
///
/// ## 예제
/// ```rust
/// let block : HashMap<String, String> = hashmap! {
///     "shcode" => "096530",
///     "gubun" => "0",
/// };
/// ```
#[macro_export]
macro_rules! hashmap {
    ($($key:expr => $val:expr),*$(,)?) => {{
        use std::collections::HashMap;

        #[allow(unused_mut)]
        let mut map = HashMap::new();
        $(map.insert($key.into(), $val.into());)*
        map
    }};
}

/// 데이터가 요청인지 응답인지에 대한 여부입니다.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum DataType {
    /// 요청
    #[cfg_attr(feature = "serde", serde(rename = "input"))]
    Input,
    /// 응답
    #[cfg_attr(feature = "serde", serde(rename = "output"))]
    Output,
}

/// non-occurs(단일) block에 대한 HashMap입니다.
pub type Block = HashMap<String, String>;

/// occurs(배열) block에 대한 HashMap 배열입니다.
pub type ArrayBlock = Vec<Block>;

/// 서버와 주고받는 데이터를 나타내는 객체입니다.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Data {
    /// TR 코드
    pub code: String,
    /// 데이터가 요청인지 응답인지에 대한 여부
    pub data_type: DataType,
    /// non-occurs(단일) block에 대한 HashMap
    pub blocks: HashMap<String, Block>,
    /// occurs(배열) block에 대한 HashMap
    pub arr_blocks: HashMap<String, ArrayBlock>,
}

pub(crate) fn decode(
    tr_layouts: &HashMap<String, TrLayout>,
    tr_code: &str,
    block_data: HashMap<String, Vec<u8>>,
    non_block_data: Option<Vec<u8>>,
) -> Result<Data, DecodeError> {
    let tr_layout = tr_layouts.get(tr_code).ok_or_else(|| DecodeError::UnknownTrCode)?;

    if let Some(raw_data) = non_block_data {
        Ok(decode_non_block(tr_layout, &raw_data)?)
    } else {
        let mut data = Data {
            code: tr_layout.code.to_owned(),
            data_type: DataType::Output,
            blocks: HashMap::new(),
            arr_blocks: HashMap::new(),
        };

        for (block_name, raw_data) in block_data {
            let block_layout =
                tr_layout.out_blocks.iter().find(|b| b.name == block_name).ok_or_else(|| {
                    DecodeError::UnknownBlockName { block_name: block_name.to_string() }
                })?;

            if block_layout.occurs {
                data.arr_blocks
                    .insert(block_name, decode_array_block(tr_layout, block_layout, &raw_data)?);
            } else {
                data.blocks.insert(block_name, decode_block(tr_layout, block_layout, &raw_data)?);
            }
        }

        Ok(data)
    }
}

// block mode인 non-occurs(단일) output 데이터를 디코딩합니다.
pub(crate) fn decode_block(
    tr_layout: &TrLayout,
    block_layout: &BlockLayout,
    raw_data: &[u8],
) -> Result<Block, DecodeError> {
    assert!(tr_layout.block && !block_layout.occurs);

    if raw_data.len() != block_layout.len {
        return Err(DecodeError::MismatchBufferLength);
    }

    let mut fields = Block::with_capacity(block_layout.fields.len());
    let mut offset = 0;

    for field_layout in &block_layout.fields {
        fields.insert(
            field_layout.name.to_owned(),
            decode_str(&raw_data[offset..offset + field_layout.len])?.into(),
        );
        offset += field_layout.len + if tr_layout.attr { 1 } else { 0 };
    }

    Ok(fields)
}

// block mode인 occurs(배열) output 데이터를 디코딩합니다.
pub(crate) fn decode_array_block(
    tr_layout: &TrLayout,
    block_layout: &BlockLayout,
    raw_data: &[u8],
) -> Result<ArrayBlock, DecodeError> {
    assert!(tr_layout.block && block_layout.occurs);

    if raw_data.len() % block_layout.len != 0 {
        return Err(DecodeError::MismatchBufferLength);
    }

    let block_count = raw_data.len() / block_layout.len;

    let mut blocks = ArrayBlock::with_capacity(block_count);
    let mut offset = 0;

    for _ in 0..block_count {
        let mut fields = Block::with_capacity(block_layout.fields.len());
        for field_layout in &block_layout.fields {
            fields.insert(
                field_layout.name.to_owned(),
                decode_str(&raw_data[offset..offset + field_layout.len])?.into(),
            );
            offset += field_layout.len + if tr_layout.attr { 1 } else { 0 };
        }

        blocks.push(fields);
    }

    Ok(blocks)
}

// non-block mode인 output 데이터를 디코딩합니다.
pub(crate) fn decode_non_block(tr_layout: &TrLayout, raw_data: &[u8]) -> Result<Data, DecodeError> {
    assert!(!tr_layout.block);

    let mut data = Data {
        code: tr_layout.code.to_owned(),
        data_type: DataType::Output,
        blocks: HashMap::new(),
        arr_blocks: HashMap::new(),
    };

    let mut offset = 0;

    for block_layout in tr_layout.out_blocks.iter() {
        if block_layout.occurs {
            if offset + 5 > raw_data.len() {
                return Err(DecodeError::MismatchBufferLength);
            }

            let block_count =
                str::parse::<u32>(euckr::decode(&raw_data[offset..offset + 5]).as_ref())
                    .map_err(|_| DecodeError::DecodeOccursLength)? as usize;
            offset += 5;

            if offset + block_layout.len * block_count > raw_data.len() {
                return Err(DecodeError::MismatchBufferLength);
            }

            let mut blocks = ArrayBlock::with_capacity(block_count);

            for _ in 0..block_count {
                let mut fields = Block::with_capacity(block_layout.fields.len());
                for field_layout in &block_layout.fields {
                    fields.insert(
                        field_layout.name.to_owned(),
                        decode_str(&raw_data[offset..offset + field_layout.len])?.into(),
                    );
                    offset += field_layout.len + if tr_layout.attr { 1 } else { 0 };
                }

                blocks.push(fields);
            }

            data.arr_blocks.insert(block_layout.name.to_owned(), blocks);
        } else {
            if offset + block_layout.len > raw_data.len() {
                return Err(DecodeError::MismatchBufferLength);
            }

            let mut fields = Block::with_capacity(block_layout.fields.len());
            for field_layout in &block_layout.fields {
                fields.insert(
                    field_layout.name.to_owned(),
                    decode_str(&raw_data[offset..offset + field_layout.len])?.into(),
                );
                offset += field_layout.len + if tr_layout.attr { 1 } else { 0 };
            }

            data.blocks.insert(block_layout.name.to_owned(), fields);
        }
    }

    Ok(data)
}

fn decode_str(data: &[u8]) -> Result<Cow<str>, DecodeError> {
    let mut len = data.len();
    for i in 0..data.len() {
        if data[i] <= 0x20 {
            len = i;
            break;
        }
    }

    let (result, _, had_errors) = EUC_KR.decode(&data[0..len]);

    if had_errors {
        Err(DecodeError::DecodeString)
    } else {
        Ok(result)
    }
}

// non-block mode로 데이터를 인코딩합니다.
// t1104를 포함해서 InBlock은 항상 non-block으로 처리해야 합니다.
pub(crate) fn encode(
    tr_layouts: &HashMap<String, TrLayout>,
    data: &Data,
) -> Result<Vec<u8>, EncodeError> {
    let res = tr_layouts.get(&data.code).ok_or_else(|| EncodeError::UnknownTrCode)?;

    let block_layouts = match data.data_type {
        DataType::Input => &res.in_blocks,
        DataType::Output => &res.out_blocks,
    };

    let mut raw_data: Vec<u8> = Vec::new();

    for block_layout in block_layouts {
        if block_layout.occurs {
            let arr_block = data.arr_blocks.get(&block_layout.name).ok_or_else(|| {
                EncodeError::MissingBlock { block_name: block_layout.name.to_owned() }
            })?;

            if !res.block {
                // 블럭의 최대 개수는 십진수로 5자리
                if arr_block.len() >= 100000 {
                    return Err(EncodeError::ExceedMaxBlockCount {
                        block_name: block_layout.name.to_owned(),
                    });
                }

                let block_count = format!("{:0>5}", arr_block.len());
                raw_data.extend(EUC_KR.encode(block_count.as_str()).0.iter());
            }

            for block in arr_block.iter() {
                encode_block(res, block_layout, block, &mut raw_data)?;
            }
        } else {
            let block = data.blocks.get(&block_layout.name).ok_or_else(|| {
                EncodeError::MissingBlock { block_name: block_layout.name.to_owned() }
            })?;

            encode_block(res, block_layout, &block, &mut raw_data)?
        }
    }

    Ok(raw_data)
}

fn encode_block(
    tr_layout: &TrLayout,
    block_layout: &BlockLayout,
    fields: &HashMap<String, String>,
    raw_data: &mut Vec<u8>,
) -> Result<(), EncodeError> {
    for field_layout in &block_layout.fields {
        let field = fields
            .get(&field_layout.name)
            .or_else(|| fields.get(&field_layout.name_old))
            .ok_or_else(|| EncodeError::MissingField {
            block_name: block_layout.name.to_owned(),
            field_name: field_layout.name.to_owned(),
        })?;

        let mut field_encoded = EUC_KR.encode(field).0.to_vec();
        if field_encoded.len() > field_layout.len {
            return Err(EncodeError::ExceedFieldLength {
                block_name: block_layout.name.to_owned(),
                field_name: field_layout.name.to_owned(),
            });
        }

        if tr_layout.attr {
            field_encoded.resize(field_layout.len + 1, b'\0');
        } else {
            field_encoded.resize(field_layout.len, b'\0');
        }
        raw_data.extend(field_encoded.iter());
    }

    Ok(())
}