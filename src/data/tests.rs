// SPDX-License-Identifier: MPL-2.0

#![cfg(all(test, windows))]

use super::{decode_block, decode_block_array, decode_non_block, encode, Block, Data, DataType};
use crate::hashmap;
use crate::layout::{self, HeaderType, TrLayout};

use hex_literal::hex;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref LAYOUT_TBL: HashMap<String, TrLayout> =
        layout::load_dir("C:\\eBEST\\xingAPI\\Res").unwrap();
}

// date="2021-01-11"
static T0424_BASE64_DATA: &str = "
    MDAwMDAwMDAwNTAwMDA3OTA3IDAwMDAwMDAwMDAwMDAwMDAwMCAwMDAwMDAwMDAwMDE5NTc0MDAg
    NTAwMDAwMDAwICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgMDAwMDAwMDAwMDAxOTY1
    NjAwsDAwMDAwMDAwMDAwMDAwODIwMLAwMDAwMjAwNTkzMCAgICAgICAgICAgICAgICAgIDIwICAg
    ICAgICAgICAgICAgICAyMCAgICAgICAgICAgICAgICAgOTA3MDAgICAgICAgICAgICAgIDE4MTQw
    MDAgICAgICAgICAgICAwICAgICAgICAgICAgICAgICAgICAgICAgICAgMjAgICAgICAgICAgICAg
    ICAgIDkwNzAwICAgICAgICAgICAgICAwICAgICAgICAgICAgICAgICAgMCAgICAgICAgICAgICAg
    ICAgIDAgICAgICAgICAgICAgICAgICAwICAgICAgICAgICAgICAgICAgMCAgICAgICAgICAgICAg
    ICAgIDAgICAgICAgICAgICAgICAgICA0ODk2NiAgICAgICAgICAgICAgILvvvLrA/MDaICAgICAg
    ICAgICAgICAgMyA5MjY3ICAgICAgIDAwMDkxMDAwsDAwMDAwMDAwMDAwMTgyMDAwMLAgICAgICAg
    ICAgICAgIDYwMDCwMDAwMDAwMDAzM7AwMDAwMDAwNTQ1IDAwMDAwMDQ1NTAgMDAwMDAwMDAwMCAw
    NzgwMjAgICAgICAgICAgICAgICAgICAyMCAgICAgICAgICAgICAgICAgMjAgICAgICAgICAgICAg
    ICAgIDcxNzAgICAgICAgICAgICAgICAxNDM0MDAgICAgICAgICAgICAgMCAgICAgICAgICAgICAg
    ICAgICAgICAgICAgIDIwICAgICAgICAgICAgICAgICA3MTcwICAgICAgICAgICAgICAgMCAgICAg
    ICAgICAgICAgICAgIDAgICAgICAgICAgICAgICAgICAwICAgICAgICAgICAgICAgICAgMCAgICAg
    ICAgICAgICAgICAgIDAgICAgICAgICAgICAgICAgICAwICAgICAgICAgICAgICAgICAgNDg5MzQg
    ICAgICAgICAgICAgICDAzLqjvbrGrsX1wNrB9bHHICAgICAgIDIgNzMzICAgICAgICAwMDAwNzI4
    MLAwMDAwMDAwMDAwMDAxNDU2MDCwICAgICAgICAgICAgICAyMjAwsDAwMDAwMDAxNTOwMDAwMDAw
    MDA0MiAwMDAwMDAwMzY0IDAwMDAwMDAwMDAg
";

#[test]
fn test_decode_t0424() {
    let t0424_data =
        base64::decode(T0424_BASE64_DATA.replace(|c| matches!(c, ' ' | '\n'), "")).unwrap();

    let tr_layout = LAYOUT_TBL.get("t0424").unwrap();
    assert!(tr_layout.attr_byte);
    assert!(!tr_layout.block_mode);
    assert_eq!(tr_layout.header_type, Some(HeaderType::D));

    let data = decode_non_block(tr_layout, DataType::Output, &t0424_data).unwrap();

    assert_eq!(data.tr_code, "t0424");
    assert_eq!(data.data_type, DataType::Output);
    assert_eq!(data.blocks.len(), 2);

    assert_eq!(
        data.blocks["t0424OutBlock"],
        Block::Block(hashmap! {
            "sunamt" => "000000000500007907",
            "dtsunik" => "000000000000000000",
            "mamt" => "000000000001957400",
            "sunamt1" => "500000000",
            "cts_expcode" => "",
            "tappamt" => "000000000001965600",
            "tdtsunik" => "000000000000008200",
        })
    );

    assert_eq!(
        data.blocks["t0424OutBlock1"],
        Block::Array(vec![
            hashmap! {
                "expcode" => "005930",
                "jangb" => "",
                "janqty" => "20",
                "mdposqt" => "20",
                "pamt" => "90700",
                "mamt" => "1814000",
                "sinamt" => "0",
                "lastdt" => "",
                "msat" => "20",
                "mpms" => "90700",
                "mdat" => "0",
                "mpmd" => "0",
                "jsat" => "0",
                "jpms" => "0",
                "jdat" => "0",
                "jpmd" => "0",
                "sysprocseq" => "48966",
                "loandt" => "",
                "hname" => "????????????",
                "marketgb" => "",
                "jonggb" => "3",
                "janrt" => "9267",
                "price" => "00091000",
                "appamt" => "000000000001820000",
                "dtsunik" => "6000",
                "sunikrt" => "0000000033",
                "fee" => "0000000545",
                "tax" => "0000004550",
                "sininter" => "0000000000",
            },
            hashmap! {
                "expcode" => "078020",
                "jangb" => "",
                "janqty" => "20",
                "mdposqt" => "20",
                "pamt" => "7170",
                "mamt" => "143400",
                "sinamt" => "0",
                "lastdt" => "",
                "msat" => "20",
                "mpms" => "7170",
                "mdat" => "0",
                "mpmd" => "0",
                "jsat" => "0",
                "jpms" => "0",
                "jdat" => "0",
                "jpmd" => "0",
                "sysprocseq" => "48934",
                "loandt" => "",
                "hname" => "????????????????????????",
                "marketgb" => "",
                "jonggb" => "2",
                "janrt" => "733",
                "price" => "00007280",
                "appamt" => "000000000000145600",
                "dtsunik" => "2200",
                "sunikrt" => "0000000153",
                "fee" => "0000000042",
                "tax" => "0000000364",
                "sininter" => "0000000000",
            }
        ])
    )
}

// date="2020-10-21", shcode="078020"
static T1101_BASE64_DATA: &str = "
    wMy6o726xq7F9cDawfWxxyAgICCwMDAwMDYwMDCwMrAwMDAwMDA5MLAwMDEuNTKwMDAwMDAwMTA5
    MjE0IDAwMDA1OTEwsDAwMDA2MDAwsDAwMDA1OTkwsDAwMDAwMDAwNDIwNyAwMDAwMDAwMDI2MjAg
    MDAwMDAwMDAwMDAwIDAwMDAwMDAwMDAwMCAwMDAwNjAxMLAwMDAwNTk4MLAwMDAwMDAwMDEyODgg
    MDAwMDAwMDAwNTc3IDAwMDAwMDAwMDAwMCAwMDAwMDAwMDAwMDAgMDAwMDYwMjCwMDAwMDU5NzCw
    MDAwMDAwMDAxMjY2IDAwMDAwMDAyMjI0OCAwMDAwMDAwMDAwMDAgMDAwMDAwMDAwMDAwIDAwMDA2
    MDMwsDAwMDA1OTYwsDAwMDAwMDAwMDcxMiAwMDAwMDAwMDEwOTggMDAwMDAwMDAwMDAwIDAwMDAw
    MDAwMDAwMCAwMDAwNjA0MLAwMDAwNTk1MLAwMDAwMDAwMDA1ODkgMDAwMDAwMDAxNjg4IDAwMDAw
    MDAwMDAwMCAwMDAwMDAwMDAwMDAgMDAwMDYwNTCwMDAwMDU5NDCwMDAwMDAwMDAwNTYzIDAwMDAw
    MDAwMTIyMSAwMDAwMDAwMDAwMDAgMDAwMDAwMDAwMDAwIDAwMDA2MDYwsDAwMDA1OTMwsDAwMDAw
    MDAwMDM4MSAwMDAwMDAwMDI1MDEgMDAwMDAwMDAwMDAwIDAwMDAwMDAwMDAwMCAwMDAwNjA3MLAw
    MDAwNTkyMLAwMDAwMDAwMDA0MTIgMDAwMDAwMDAxMTY3IDAwMDAwMDAwMDAwMCAwMDAwMDAwMDAw
    MDAgMDAwMDYwODCwMDAwMDU5MTDAMDAwMDAwMDAwMjI1IDAwMDAwMDAwMDcwMSAwMDAwMDAwMDAw
    MDAgMDAwMDAwMDAwMDAwIDAwMDA2MDkwsDAwMDA1OTAwoDAwMDAwMDAwMDY0MCAwMDAwMDAwMDA3
    NzcgMDAwMDAwMDAwMDAwIDAwMDAwMDAwMDAwMCAwMDAwMDAwMTAyODOgMDAwMDAwMDM0NTk4sC0w
    MDAwMDAxMTU4NaAtMDAwMDAwMzU5MzGgMTYwMDAxMTIgMDAwMDAwMDDAMDAwMDAwMDAwMDAwIDPA
    MDAwMDAwMDDAMDAwLjAwwDAwMDAwMDAwMDAwMLAwMDAwMDAwMDAwOTigMiAwNzgwMjAgMDAwMDc2
    ODCwMDAwMDQxNDCgMDAwMDU5MTCwMDAwMDYwMDCwMDAwMDU4NzCw
";

#[test]
fn test_decode_t1101() {
    let t1101_data =
        base64::decode(T1101_BASE64_DATA.replace(|c| matches!(c, ' ' | '\n'), "")).unwrap();

    let tr_layout = LAYOUT_TBL.get("t1101").unwrap();
    assert!(tr_layout.attr_byte);
    assert!(tr_layout.block_mode);
    assert_eq!(tr_layout.header_type, Some(HeaderType::A));

    let block = decode_block(
        tr_layout,
        tr_layout
            .out_blocks
            .iter()
            .find(|b| b.name == "t1101OutBlock")
            .unwrap(),
        &t1101_data,
    )
    .unwrap();

    assert_eq!(
        block,
        Block::Block(hashmap! {
            "hname" => "????????????????????????",
            "price" => "00006000",
            "sign" => "2",
            "change" => "00000090",
            "diff" => "001.52",
            "volume" => "000000109214",
            "jnilclose" => "00005910",
            "offerho1" => "00006000",
            "bidho1" => "00005990",
            "offerrem1" => "000000004207",
            "bidrem1" => "000000002620",
            "preoffercha1" => "000000000000",
            "prebidcha1" => "000000000000",
            "offerho2" => "00006010",
            "bidho2" => "00005980",
            "offerrem2" => "000000001288",
            "bidrem2" => "000000000577",
            "preoffercha2" => "000000000000",
            "prebidcha2" => "000000000000",
            "offerho3" => "00006020",
            "bidho3" => "00005970",
            "offerrem3" => "000000001266",
            "bidrem3" => "000000022248",
            "preoffercha3" => "000000000000",
            "prebidcha3" => "000000000000",
            "offerho4" => "00006030",
            "bidho4" => "00005960",
            "offerrem4" => "000000000712",
            "bidrem4" => "000000001098",
            "preoffercha4" => "000000000000",
            "prebidcha4" => "000000000000",
            "offerho5" => "00006040",
            "bidho5" => "00005950",
            "offerrem5" => "000000000589",
            "bidrem5" => "000000001688",
            "preoffercha5" => "000000000000",
            "prebidcha5" => "000000000000",
            "offerho6" => "00006050",
            "bidho6" => "00005940",
            "offerrem6" => "000000000563",
            "bidrem6" => "000000001221",
            "preoffercha6" => "000000000000",
            "prebidcha6" => "000000000000",
            "offerho7" => "00006060",
            "bidho7" => "00005930",
            "offerrem7" => "000000000381",
            "bidrem7" => "000000002501",
            "preoffercha7" => "000000000000",
            "prebidcha7" => "000000000000",
            "offerho8" => "00006070",
            "bidho8" => "00005920",
            "offerrem8" => "000000000412",
            "bidrem8" => "000000001167",
            "preoffercha8" => "000000000000",
            "prebidcha8" => "000000000000",
            "offerho9" => "00006080",
            "bidho9" => "00005910",
            "offerrem9" => "000000000225",
            "bidrem9" => "000000000701",
            "preoffercha9" => "000000000000",
            "prebidcha10" => "000000000000",
            "offerho10" => "00006090",
            "bidho10" => "00005900",
            "offerrem10" => "000000000640",
            "bidrem10" => "000000000777",
            "preoffercha10" => "000000000000",
            "prebidcha9" => "000000000000",
            "offer" => "000000010283",
            "bid" => "000000034598",
            "preoffercha" => "-00000011585",
            "prebidcha" => "-00000035931",
            "hotime" => "16000112",
            "yeprice" => "00000000",
            "yevolume" => "000000000000",
            "yesign" => "3",
            "yechange" => "00000000",
            "yediff" => "000.00",
            "tmoffer" => "000000000000",
            "tmbid" => "000000000098",
            "ho_status" => "2",
            "shcode" => "078020",
            "uplmtprice" => "00007680",
            "dnlmtprice" => "00004140",
            "open" => "00005910",
            "high" => "00006000",
            "low" => "00005870",
        })
    );
}

#[test]
fn test_decode_t1104() {
    // date="2021-01-12", code="078020", gubn="1", dat="1", dat2="1"
    const T1104_DATA: &[u8] = &hex!("00 31 30 30 30 30 37 32 36 30");

    let tr_layout = LAYOUT_TBL.get("t1104").unwrap();
    assert!(!tr_layout.attr_byte);
    assert!(tr_layout.block_mode);
    assert_eq!(tr_layout.header_type, Some(HeaderType::A));

    let block = decode_block_array(
        tr_layout,
        tr_layout
            .out_blocks
            .iter()
            .find(|b| b.name == "t1104OutBlock1")
            .unwrap(),
        T1104_DATA,
    )
    .unwrap();

    assert_eq!(
        block,
        Block::Array(vec![hashmap! {
            "indx" => "",
            "gubn" => "1",
            "vals" => "00007260",
        }])
    );
}

// date="2020-10-21", shcode="078020"
static T1764_BASE64_DATA: &str = "
    MAAgICAwMDAAv9yxubDoyLi/+LvnwPzDvAAgICAgMQAgICAwNDIgQ1PB9bHHICAgICAgIAAAAAAA
    AAAgMgAgICAwMTcgS0LB9bHHICAgICAgIAAAAAAAAAAgMwAgICAwMTIgTkjF9cDawfWxxyAgIAAA
    AAAAAAAgNAAgICAwMjUgU0vB9bHHICAgICAgIAAAAAAAAAAgNQAgICAwNDMgVUJTICAgICAgICAg
    IAAAAAAAAAAgNgAgICAwNjMgZUJFU1QgwfWxxyAgIAAAAAAAAAAgNwAgICAwNDUgsPG15bi4ICAg
    ICAgIAAAAAAAAAAgOAAgICAwMDQgtOu9xcH1sccgICAgIAAAAAAAAAAgOQAgICAwMTAguN64rsP3
    ICAgICAgIAAAAAAAAAAgMTAAICAwNDQguN64sbiwxKEgICAgIAAAAAAAAAAgMTEAICAwMDUgucy3
    obTrv+wgICAgIAAAAAAAAAAgMTIAICAwMzAgu++8usH1sccgICAgIAAAAAAAAAAgMTMAICAwMDYg
    vcW/tcH1sccgICAgIAAAAAAAAAAgMTQAICAwMDIgvcXH0cX1wNogICAgIAAAAAAAAAAgMTUAICAw
    MDggwK/B+MH1sccgICAgIAAAAAAAAAAgMTYAICAwNTAgxbC/8sH1sccgICAgIAAAAAAAAAAgMTcA
    ICAwNDYgx8/AzMH1sccgICAgIAAAAAAAAAAgMTgAICAwMDMgx9GxucH1sccgICAgIAAAAAAAAAAg
";

#[test]
fn test_decode_t1764() {
    let t1764_data =
        base64::decode(T1764_BASE64_DATA.replace(|c| matches!(c, ' ' | '\n'), "")).unwrap();

    let tr_layout = LAYOUT_TBL.get("t1764").unwrap();
    assert!(tr_layout.attr_byte);
    assert!(tr_layout.block_mode);
    assert_eq!(tr_layout.header_type, Some(HeaderType::A));

    let block = decode_block_array(
        tr_layout,
        tr_layout
            .out_blocks
            .iter()
            .find(|b| b.name == "t1764OutBlock")
            .unwrap(),
        &t1764_data,
    )
    .unwrap();

    assert_eq!(
        block,
        Block::Array(vec![
            hashmap! { "rank" => "0", "tradno" => "000", "tradname" => "????????????????????????" },
            hashmap! { "rank" => "1", "tradno" => "042", "tradname" => "CS??????" },
            hashmap! { "rank" => "2", "tradno" => "017", "tradname" => "KB??????" },
            hashmap! { "rank" => "3", "tradno" => "012", "tradname" => "NH????????????" },
            hashmap! { "rank" => "4", "tradno" => "025", "tradname" => "SK??????" },
            hashmap! { "rank" => "5", "tradno" => "043", "tradname" => "UBS" },
            hashmap! { "rank" => "6", "tradno" => "063", "tradname" => "eBEST ??????" },
            hashmap! { "rank" => "7",  "tradno" => "045", "tradname" => "?????????" },
            hashmap! { "rank" => "8", "tradno" => "004", "tradname" => "????????????" },
            hashmap! { "rank" => "9", "tradno" => "010", "tradname" => "?????????" },
            hashmap! { "rank" => "10", "tradno" => "044", "tradname" => "????????????" },
            hashmap! { "rank" => "11", "tradno" => "005", "tradname" => "????????????" },
            hashmap! { "rank" => "12", "tradno" => "030", "tradname" => "????????????" },
            hashmap! { "rank" => "13", "tradno" => "006", "tradname" => "????????????" },
            hashmap! { "rank" => "14", "tradno" => "002", "tradname" => "????????????" },
            hashmap! { "rank" => "15", "tradno" => "008", "tradname" => "????????????" },
            hashmap! { "rank" => "16", "tradno" => "050", "tradname" => "????????????" },
            hashmap! { "rank" => "17", "tradno" => "046", "tradname" => "????????????" },
            hashmap! { "rank" => "18", "tradno" => "003", "tradname" => "????????????" },
        ])
    );
}

#[test]
fn test_encode_t1104() {
    let data = Data {
        tr_code: "t1104".into(),
        data_type: DataType::Input,
        blocks: hashmap! {
            "t1104InBlock" => Block::Block(hashmap! {
                "code" => "096530",
                "nrec" => "1",
            }),
            "t1104InBlock1" => Block::Array(vec![hashmap! {
                "indx" => "0",
                "gubn" => "1",
                "dat1" => "1",
                "dat2" => "1",
            }]),
        },
    };

    assert_eq!(
        encode(&data, &LAYOUT_TBL.get(&data.tr_code).unwrap()).unwrap(),
        hex!("30 39 36 35 33 30 31 00 30 31 31 31 00 00 00 00 00 00 00")
    );
}
