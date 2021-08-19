// SPDX-License-Identifier: MIT

// 실시간 TR을 요청하는 예제입니다.

use clap::{App, Arg};
use lazy_static::lazy_static;

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use xingapi::data::{Block, Data, DataType};
use xingapi::{hashmap, real::Real, response::Response, XingApi};

enum Market {
    Kospi,
    Kosdaq,
}

impl Market {
    fn is_listed(&self, xingapi: &XingApi, code: &str) -> bool {
        let res = xingapi
            .request(
                &Data {
                    code: "t8430".into(),
                    data_type: DataType::Input,
                    blocks: hashmap! {
                        "t8430InBlock" => Block::Block(hashmap! {
                            "gubun" => match self {
                                Self::Kospi => "1",
                                Self::Kosdaq => "2",
                            },
                        }),
                    },
                },
                None,
                None,
            )
            .unwrap();

        res.data().unwrap().blocks["t8430OutBlock"]
            .as_array()
            .unwrap()
            .iter()
            .any(|block| block["shcode"] == code)
    }
}

fn main() {
    lazy_static! {
        static ref QUIT: AtomicBool = AtomicBool::new(false);
    }

    ctrlc::set_handler(|| {
        QUIT.store(true, Ordering::Relaxed);
    })
    .unwrap();

    let matches = App::new("listen-volume")
        .arg(Arg::with_name("id").short("i").required(true).takes_value(true))
        .arg(Arg::with_name("pw").short("p").required(true).takes_value(true))
        .arg(Arg::with_name("code").short("c").required(true).takes_value(true))
        .get_matches();

    let id = matches.value_of("id").unwrap();
    let pw = matches.value_of("pw").unwrap();
    let code = matches.value_of("code").unwrap();

    let xingapi = XingApi::new().unwrap();

    xingapi.connect("demo.ebestsec.co.kr", 20001, None, None).unwrap();
    println!("server connected");

    let login = xingapi.login(id, pw, "", false).unwrap();
    if login.is_ok() {
        println!("login succeed: {}, {}", login.code(), login.message());
    } else {
        eprintln!("login failed: {}, {}", login.code(), login.message());
        return;
    }

    // 종목 코드가 어느 시장에 상장되어 있는지 검색합니다.
    let (tr_code, market) = {
        if Market::Kospi.is_listed(&xingapi, code) {
            ("S3_", "KOSPI")
        } else if Market::Kosdaq.is_listed(&xingapi, code) {
            ("K3_", "KOSDAQ")
        } else {
            eprintln!("unknown ticker: {}", code);
            return;
        }
    };

    let real = Real::new(xingapi.clone()).unwrap();

    real.subscribe(tr_code, &[code]).unwrap();
    println!("registered: tr_code: {}, market: {}, ticker: {}", tr_code, market, code);

    while !QUIT.load(Ordering::Relaxed) {
        if let Ok(res) = real.recv_timeout(Duration::from_millis(10)) {
            let data = res.data().unwrap();
            let cvolume = data.blocks["OutBlock"]["cvolume"].parse::<u32>().unwrap();
            println!("real response: {}", cvolume);
        }
    }

    println!("ctrl-c interrupt");

    xingapi.disconnect();
    println!("server disconnected");

    assert!(!xingapi.is_connected());
}
