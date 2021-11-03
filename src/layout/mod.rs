// SPDX-License-Identifier: MPL-2.0

//! 데이터에 대한 레이아웃을 파싱하는 모듈
//!
//! 레이아웃은 EUC-KR로 인코딩된 'RES 파일'에서 가져올 수 있습니다.

pub mod error;

mod read;
mod tests;

use self::error::{Error, LoadError};
use self::read::{Read, StrRead};

use std::{collections::HashMap, convert::AsRef, path::Path, str::FromStr};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// XingAPI SDK의 기본 설치 경로에서 TR 레이아웃을 모두 불러옵니다.
#[cfg(any(doc, windows))]
#[cfg_attr(doc_cfg, doc(cfg(windows)))]
pub fn load() -> Result<HashMap<String, TrLayout>, LoadError> {
    load_dir("C:\\eBEST\\xingAPI\\Res")
}

/// 지정된 디렉터리에서 TR 레이아웃을 모두 불러옵니다.
///
/// 하위 디렉터리는 탐색하지 않습니다.
pub fn load_dir<P: AsRef<Path>>(path: P) -> Result<HashMap<String, TrLayout>, LoadError> {
    use encoding_rs::EUC_KR;
    use std::{fs, sync::mpsc};
    use threadpool::ThreadPool;

    let pool = ThreadPool::new(16);
    let (tx, rx) = mpsc::channel();

    for ent in fs::read_dir(&path)? {
        let path = ent?.path();
        if !path.is_file() || path.extension() != Some("res".as_ref()) {
            continue;
        }

        let tx = tx.clone();

        pool.execute(move || {
            let parse_layout = || -> Result<TrLayout, LoadError> {
                let raw_data = fs::read(&path)?;

                let (data, _, had_errors) = EUC_KR.decode(&raw_data);
                if had_errors {
                    return Err(LoadError::Encoding(path));
                }

                data.parse().map_err(|err| LoadError::Parse(path, err))
            };

            tx.send(parse_layout()).unwrap();
        });
    }

    drop(tx);

    let mut layout_tbl = HashMap::new();

    while let Ok(result) = rx.recv() {
        let layout = result?;

        if let Some(other) = layout_tbl.get(&layout.code) {
            if layout != *other {
                return Err(LoadError::Confilict(layout.code));
            }
        } else {
            layout_tbl.insert(layout.code.clone(), layout);
        }
    }

    pool.join();

    assert_eq!(pool.queued_count(), 0);
    assert_eq!(pool.panic_count(), 0);

    Ok(layout_tbl)
}

fn next_sym<'a, R: Read<'a>>(reader: &R) -> Result<&'a str, Error> {
    reader
        .next_sym()
        .ok_or_else(|| Error::unexpected_eof(reader))
}

fn peek_sym<'a, R: Read<'a>>(reader: &R) -> Result<&'a str, Error> {
    reader
        .peek_sym()
        .ok_or_else(|| Error::unexpected_eof(reader))
}

fn skip_delimiter<'a, R: Read<'a>>(reader: &R) -> Result<(), Error> {
    match next_sym(reader)? {
        "," => Ok(()),
        ";" => Err(Error::unexpected_data(reader)),
        _ => Err(Error::unexpected_syntax(reader)),
    }
}

/// 헤더 타입
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum HeaderType {
    /// A 타입
    ///
    /// 블록 모드만을 사용하는 타입입니다.
    A,
    /// B 타입
    B,
    /// C 타입
    C,
    /// D 타입
    D,
}

impl FromStr for HeaderType {
    type Err = ();
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "A" => Ok(Self::A),
            "B" => Ok(Self::B),
            "C" => Ok(Self::C),
            "D" => Ok(Self::D),
            _ => Err(()),
        }
    }
}

/// TR 타입 (조회 및 실시간)
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum TrType {
    /// 조회 TR
    #[cfg_attr(feature = "serde", serde(rename = "func"))]
    Func,
    /// 실시간 TR
    #[cfg_attr(feature = "serde", serde(rename = "feed"))]
    Feed,
}

impl FromStr for TrType {
    type Err = ();
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            ".Func" => Ok(Self::Func),
            ".Feed" => Ok(Self::Feed),
            _ => Err(()),
        }
    }
}

/// TR 레이아웃
///
/// [`FromStr`](FromStr)이 구현되어 있어 문자열로부터 파싱할 수 있습니다.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TrLayout {
    /// TR 타입
    pub tr_type: TrType,
    /// TR 설명
    pub desc: String,
    /// TR 코드
    pub code: String,
    /// attribute byte 존재 여부
    ///
    /// 각 필드의 끝에 attribute byte가 존재할 수 있습니다.
    pub attr_byte: bool,
    /// 블록 모드 여부
    pub block_mode: bool,
    /// 헤더 타입
    pub header_type: Option<HeaderType>,
    /// 요청 블록 목록
    pub in_blocks: Vec<BlockLayout>,
    /// 응답 블록 목록
    pub out_blocks: Vec<BlockLayout>,
}

impl TrLayout {
    fn from_reader<'a, R: Read<'a>>(reader: &R) -> Result<Self, Error> {
        if next_sym(reader)? != "BEGIN_FUNCTION_MAP" {
            return Err(Error::unexpected_syntax(reader));
        }

        let tr_type =
            TrType::from_str(next_sym(reader)?).map_err(|_| Error::unexpected_data(reader))?;
        skip_delimiter(reader)?;

        let desc = next_sym(reader)?.to_owned();
        skip_delimiter(reader)?;

        let code = next_sym(reader)?.to_owned();

        if tr_type == TrType::Feed && code.len() != 3 {
            return Err(Error::unexpected_data(reader));
        }

        let mut attr_byte = false;
        let mut block_mode = false;
        let mut header_type = None;

        loop {
            match next_sym(reader)? {
                "," => {}
                ";" => break,
                _ => return Err(Error::unexpected_syntax(reader)),
            }

            let param = next_sym(reader)?;

            if let Some((key, val)) = param.split_once('=') {
                if key.chars().any(|c| !c.is_ascii_alphabetic()) || val.contains('=') {
                    return Err(Error::unexpected_data(reader));
                }

                match key {
                    "headtype" => {
                        header_type = Some(
                            HeaderType::from_str(val)
                                .map_err(|_| Error::unexpected_data(reader))?,
                        );
                    }
                    "key" | "group" | "tuxcode" | "svr" | "SERVICE" | "CREATOR" | "CREDATE" => {}
                    _ => {
                        return Err(Error::unexpected_data(reader));
                    }
                }
            } else {
                match param {
                    "attr" => {
                        attr_byte = true;
                    }
                    "block" => {
                        block_mode = true;
                    }
                    "ENCRYPT" | "SIGNATURE" => {}
                    _ => {
                        return Err(Error::unexpected_data(reader));
                    }
                }
            }
        }

        if next_sym(reader)? != "BEGIN_DATA_MAP" {
            return Err(Error::unexpected_syntax(reader));
        }

        let mut in_blocks = Vec::new();
        let mut out_blocks = Vec::new();

        loop {
            if peek_sym(reader)? == "END_DATA_MAP" {
                next_sym(reader)?;
                break;
            }

            let block = BlockLayout::from_reader(reader, attr_byte)?;

            match block.block_type {
                BlockType::Input => {
                    in_blocks.push(block);
                }
                BlockType::Output => {
                    out_blocks.push(block);
                }
            }
        }

        Ok(TrLayout {
            tr_type,
            desc,
            code,
            attr_byte,
            block_mode,
            header_type,
            in_blocks,
            out_blocks,
        })
    }
}

impl FromStr for TrLayout {
    type Err = Error;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::from_reader(&StrRead::new(text))
    }
}

/// 블록 타입 (요청 및 응답)
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum BlockType {
    /// 요청 타입
    #[cfg_attr(feature = "serde", serde(rename = "input"))]
    Input,
    /// 응답 타입
    #[cfg_attr(feature = "serde", serde(rename = "output"))]
    Output,
}

impl FromStr for BlockType {
    type Err = ();
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "input" => Ok(Self::Input),
            "output" => Ok(Self::Output),
            _ => Err(()),
        }
    }
}

/// 블록 레이아웃
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BlockLayout {
    /// 블록 이름
    pub name: String,
    /// 블록 설명
    pub desc: String,
    /// 블록 타입
    pub block_type: BlockType,
    /// 배열 여부
    pub occurs: bool,
    /// 블록 하나의 길이
    ///
    /// 각 필드의 끝에 attribute byte가 존재하는 경우 모두 포함하여 계산합니다.
    pub len: usize,
    /// 필드 목록
    pub fields: Vec<FieldLayout>,
}

impl BlockLayout {
    fn from_reader<'a, R: Read<'a>>(reader: &R, attr_byte: bool) -> Result<Self, Error> {
        let name = next_sym(reader)?.to_owned();

        let (prefix, suffix) = name
            .rsplit_once("InBlock")
            .or_else(|| name.rsplit_once("OutBlock"))
            .ok_or_else(|| Error::unexpected_data(reader))?;

        if prefix.chars().any(|c| !c.is_ascii_alphanumeric())
            || suffix.chars().any(|c| !c.is_ascii_digit())
        {
            return Err(Error::unexpected_data(reader));
        }

        skip_delimiter(reader)?;

        let desc = next_sym(reader)?.to_owned();
        skip_delimiter(reader)?;

        let block_type =
            BlockType::from_str(next_sym(reader)?).map_err(|_| Error::unexpected_data(reader))?;

        let mut occurs = false;

        loop {
            match next_sym(reader)? {
                "," => {}
                ";" => break,
                _ => return Err(Error::unexpected_syntax(reader)),
            }

            match next_sym(reader)? {
                "occurs" => {
                    occurs = true;
                }
                _ => {
                    return Err(Error::unexpected_data(reader));
                }
            }
        }

        if next_sym(reader)? != "begin" {
            return Err(Error::unexpected_syntax(reader));
        }

        let mut fields = Vec::new();

        loop {
            if peek_sym(reader)? == "end" {
                reader.next_sym().unwrap();
                break;
            }

            fields.push(FieldLayout::from_reader(reader)?);
        }

        let len = fields
            .iter()
            .map(|f| f.len + if attr_byte { 1 } else { 0 })
            .sum();

        Ok(BlockLayout {
            name,
            desc,
            block_type,
            occurs,
            len,
            fields,
        })
    }
}

impl AsRef<BlockLayout> for BlockLayout {
    fn as_ref(&self) -> &BlockLayout {
        self
    }
}

/// 필드 타입
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FieldType {
    /// 문자열
    #[cfg_attr(feature = "serde", serde(rename = "char"))]
    Char,
    /// 날짜
    #[cfg_attr(feature = "serde", serde(rename = "date"))]
    Date,
    /// 정수
    #[cfg_attr(feature = "serde", serde(rename = "int"))]
    Int,
    /// 32비트 실수
    #[cfg_attr(feature = "serde", serde(rename = "float"))]
    Float,
    /// 64비트 실수
    #[cfg_attr(feature = "serde", serde(rename = "double"))]
    Double,
}

impl FromStr for FieldType {
    type Err = ();
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "char" => Ok(Self::Char),
            "date" => Ok(Self::Date),
            "long" | "int" => Ok(Self::Int),
            "float" => Ok(Self::Float),
            "double" => Ok(Self::Double),
            _ => Err(()),
        }
    }
}

/// 필드 레이아웃
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FieldLayout {
    /// 필드 설명
    pub desc: String,
    /// 필드의 첫 번째 이름
    pub name_old: String,
    /// 필드의 두 번째 이름
    pub name: String,
    /// 필드 타입
    pub field_type: FieldType,
    /// 필드 길이
    ///
    /// 필드의 끝에 attribute byte가 존재하더라도 제외하고 계산합니다.
    pub len: usize,
    /// 소수점 자릿수
    pub point: Option<usize>,
}

impl FieldLayout {
    fn from_reader<'a, R: Read<'a>>(reader: &R) -> Result<Self, Error> {
        let desc = next_sym(reader)?.to_owned();
        skip_delimiter(reader)?;

        let name_old = next_sym(reader)?.to_owned();
        skip_delimiter(reader)?;

        let name = next_sym(reader)?.to_owned();
        skip_delimiter(reader)?;

        let field_type =
            FieldType::from_str(next_sym(reader)?).map_err(|_| Error::unexpected_data(reader))?;
        skip_delimiter(reader)?;

        let raw_len = next_sym(reader)?;

        let parse_num = |text: &str| -> Result<usize, Error> {
            text.parse::<usize>()
                .map_err(|_| Error::unexpected_data(reader))
        };

        let (len, point) = if let Some((len, point)) = raw_len.split_once(".") {
            (parse_num(len)?, Some(parse_num(point)?))
        } else {
            (parse_num(raw_len)?, None)
        };

        // 필드가 세미콜론으로 끝나지 않는 경우도 있습니다.
        if peek_sym(reader)? == ";" {
            reader.next_sym().unwrap();
        }

        Ok(FieldLayout {
            desc,
            name_old,
            name,
            field_type,
            len,
            point,
        })
    }
}

impl AsRef<FieldLayout> for FieldLayout {
    fn as_ref(&self) -> &FieldLayout {
        self
    }
}
