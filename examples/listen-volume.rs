// SPDX-License-Identifier: MIT

// 실시간 TR을 요청하는 예제입니다.

use clap::Clap;
use lazy_static::lazy_static;
use xingapi::{
    data::{Block, Data, DataType},
    hashmap,
    response::Message,
    Real, XingApi,
};

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

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
            .find(|block| block["shcode"] == code)
            .is_some()
    }
}

#[derive(Clap)]
struct Opts {
    #[clap(short)]
    id: String,
    #[clap(short)]
    pw: String,
    #[clap(short)]
    code: String,
}

fn main() {
    lazy_static! {
        static ref QUIT: AtomicBool = AtomicBool::new(false);
    }

    ctrlc::set_handler(|| {
        QUIT.store(true, Ordering::Relaxed);
    })
    .unwrap();

    let opts = Opts::parse();
    let xingapi = XingApi::new().unwrap();

    xingapi.connect("demo.ebestsec.co.kr", 20001, None, None).unwrap();
    println!("server connected");

    let login = xingapi.login(&opts.id, &opts.pw, "", false).unwrap();
    if login.is_ok() {
        println!("login succeed: {}, {}", login.code(), login.message());
    } else {
        eprintln!("login failed: {}, {}", login.code(), login.message());
        return;
    }

    // 종목 코드가 어느 시장에 상장되어 있는지 검색합니다.
    let (tr_code, market) = {
        if Market::Kospi.is_listed(&xingapi, &opts.code) {
            ("S3_", "KOSPI")
        } else if Market::Kosdaq.is_listed(&xingapi, &opts.code) {
            ("K3_", "KOSDAQ")
        } else {
            eprintln!("unknown ticker: {}", opts.code);
            return;
        }
    };

    let real = Arc::new(Real::new(xingapi.clone()).unwrap());

    real.subscribe(tr_code, vec![opts.code.clone()]).unwrap();
    println!("registered: tr_code: {}, market: {}, ticker: {}", tr_code, market, opts.code);

    while !QUIT.load(Ordering::Relaxed) {
        if let Some(res) = real.recv_timeout(Duration::from_millis(10)) {
            let data = res.data().unwrap();
            let cvolume = data.blocks["OutBlock"]["cvolume"].parse::<u32>().unwrap();
            println!("real response: {}", cvolume);
        }
    }

    println!("ctrl-c interrupt");

    xingapi.disconnect();
    println!("server disconnected");

    assert_eq!(xingapi.is_connected(), false);
}
