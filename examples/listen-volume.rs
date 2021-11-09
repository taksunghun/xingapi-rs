// SPDX-License-Identifier: MIT

#![cfg(windows)]

use clap::{App, Arg};
use lazy_static::lazy_static;

use std::sync::atomic::{AtomicBool, Ordering};
use std::{collections::HashMap, sync::RwLock, time::Duration};

use xingapi::data::{Block, Data, DataType};
use xingapi::layout::TrLayout;
use xingapi::{hashmap, QueryResponse, RealEvent, Response};

lazy_static! {
    static ref LAYOUT_TBL: RwLock<HashMap<String, TrLayout>> = RwLock::new(HashMap::new());
}

enum Market {
    Kospi,
    Kosdaq,
}

impl Market {
    fn request_t8430(&self) -> Result<QueryResponse, xingapi::Error> {
        xingapi::request(
            &Data {
                tr_code: "t8430".into(),
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
            LAYOUT_TBL.read().unwrap().get("t8430").unwrap(),
            None,
            Duration::from_secs(10),
        )
    }

    #[allow(dead_code)]
    pub fn symbols(&self) -> Vec<String> {
        let res = self.request_t8430().unwrap();
        let data = res.data().unwrap();

        data.blocks["t8430OutBlock"]
            .as_array()
            .unwrap()
            .iter()
            .map(|block| block["shcode"].to_owned())
            .collect()
    }

    pub fn is_listed(&self, ticker: &str) -> bool {
        let res = self.request_t8430().unwrap();
        let data = res.data().unwrap();

        data.blocks["t8430OutBlock"]
            .as_array()
            .unwrap()
            .iter()
            .any(|block| block["shcode"] == ticker)
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
        .arg(
            Arg::with_name("addr")
                .long("addr")
                .takes_value(true)
                .default_value("demo.ebestsec.co.kr"),
        )
        .arg(
            Arg::with_name("id")
                .long("id")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name("pw")
                .long("pw")
                .required(true)
                .takes_value(true),
        )
        .arg(Arg::with_name("cert-pw").long("cert-pw").takes_value(true))
        .arg(Arg::with_name("stock").long("stock").takes_value(true))
        .get_matches();

    let addr = matches.value_of("addr").unwrap();
    let id = matches.value_of("id").unwrap();
    let pw = matches.value_of("pw").unwrap();
    let cert_pw = matches.value_of("cert-pw").unwrap_or("");
    let ticker_symbol = matches.value_of("stock").unwrap_or("005930");

    *LAYOUT_TBL.write().unwrap() = xingapi::layout::load().unwrap();

    assert!(
        LAYOUT_TBL.read().unwrap().contains_key("t8430"),
        "t8430 layout is missing"
    );

    xingapi::loader::load().unwrap();
    println!("xingapi loaded");

    xingapi::connect(addr, 20001, Duration::from_secs(10)).unwrap();
    println!("server connected");

    let res = xingapi::login(id, pw, cert_pw, false).unwrap();
    if res.is_ok() {
        println!("login succeed");
    } else {
        panic!("login failed: {:?}", res);
    }

    let (tr_code, market) = {
        if Market::Kospi.is_listed(ticker_symbol) {
            ("S3_", "KOSPI")
        } else if Market::Kosdaq.is_listed(ticker_symbol) {
            ("K3_", "KOSDAQ")
        } else {
            eprintln!("unknown ticker: {}", ticker_symbol);
            return;
        }
    };

    let real = RealEvent::new().unwrap();

    real.insert_layout(LAYOUT_TBL.read().unwrap().get(tr_code).unwrap().to_owned());
    real.subscribe(tr_code, &[ticker_symbol]);

    println!(
        "registered: tr_code: {}, market: {}, ticker: {}",
        tr_code, market, ticker_symbol
    );

    while !QUIT.load(Ordering::Relaxed) {
        if let Some(res) = real.recv_timeout(Duration::from_millis(10)) {
            let data = res.data().unwrap();
            let cvolume = data.blocks["OutBlock"]["cvolume"].parse::<u32>().unwrap();
            println!("real response: {}", cvolume);
        }
    }

    println!("ctrl-c interrupt");

    real.unsubscribe(tr_code, &[ticker_symbol]);

    xingapi::disconnect();
    println!("server disconnected");

    xingapi::loader::unload();
    println!("xingapi unloaded")
}
