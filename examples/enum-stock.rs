// SPDX-License-Identifier: MIT

// 주식 시장의 종목을 조회하는 예제입니다.

use clap::{App, Arg};
use xingapi::{
    data::{Block, Data, DataType},
    hashmap,
    response::Message,
    XingApi,
};

fn main() {
    let matches = App::new("enum-stock")
        .arg(Arg::with_name("id").short("i").required(true).takes_value(true))
        .arg(Arg::with_name("pw").short("p").required(true).takes_value(true))
        .get_matches();

    let id = matches.value_of("id").unwrap();
    let pw = matches.value_of("pw").unwrap();

    let xingapi = XingApi::new().unwrap();

    xingapi.connect("demo.ebestsec.co.kr", 20001, None, None).unwrap();
    println!("server connected");

    let login = xingapi.login(id, pw, "", false).unwrap();
    println!("login: {:?}", login);
    assert!(login.is_ok());

    let res = xingapi
        .request(
            &Data {
                code: "t8430".into(),
                data_type: DataType::Input,
                blocks: hashmap! {
                    "t8430InBlock" => Block::Block(hashmap! {
                        "gubun" => "0",
                    }),
                },
            },
            None,
            None,
        )
        .unwrap();

    let mut stocks: Vec<&str> = res.data().unwrap().blocks["t8430OutBlock"]
        .as_array()
        .unwrap()
        .iter()
        .map(|block| block["shcode"].as_str())
        .collect();

    stocks.sort_unstable();
    println!("{:?}", stocks);

    xingapi.disconnect();
    println!("server disconnected");

    assert!(!xingapi.is_connected());
}
