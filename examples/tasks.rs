// SPDX-License-Identifier: MIT

#![cfg(windows)]

use clap::{App, Arg};
use std::time::Duration;

use xingapi::data::{Block, Data, DataType};
use xingapi::{hashmap, Error, Response};

fn main() {
    let matches = App::new("login")
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
        .get_matches();

    let addr = matches.value_of("addr").unwrap();
    let id = matches.value_of("id").unwrap();
    let pw = matches.value_of("pw").unwrap();
    let cert_pw = matches.value_of("cert-pw").unwrap_or("");

    let layout_tbl = xingapi::layout::load().unwrap();

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

    let t1101_limit_per_sec = xingapi::tr_limit_per_sec("t1101").unwrap();
    let t1101_limit_per_ten_min = xingapi::tr_limit_per_ten_min("t1101");
    println!("t1101 limit_per_sec: {}", t1101_limit_per_sec);
    println!("t1101 limit_per_ten_min: {:?}", t1101_limit_per_ten_min);

    let t1764_limit_per_sec = xingapi::tr_limit_per_sec("t1764").unwrap();
    let t1764_limit_per_ten_min = xingapi::tr_limit_per_ten_min("t1764");
    println!("t1764 limit_per_sec: {}", t1764_limit_per_sec);
    println!("t1764 limit_per_ten_min: {:?}", t1764_limit_per_ten_min);

    let t1101_layout = layout_tbl.get("t1101").unwrap().to_owned();

    let t1101_loop = std::thread::spawn(move || {
        let req_data = Data {
            tr_code: "t1101".into(),
            data_type: DataType::Input,
            blocks: hashmap! {
                "t1101InBlock" => Block::Block(hashmap! {
                    "shcode" => "078020",
                }),
            },
        };

        for i in 0..20 * t1101_limit_per_sec {
            let res = loop {
                match xingapi::request(&req_data, &t1101_layout, None, Duration::from_secs(30)) {
                    Err(Error::XingApi { code: -21, .. }) => {
                        println!("t1101: limit reached");
                        std::thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    result => break result,
                }
            }
            .unwrap();

            assert!(res.is_ok());

            let elapsed = res.elapsed();
            println!("t1101: index: {}, elapsed: {} ms", i, elapsed.as_millis());

            let wait_duration = Duration::from_secs_f32(1.0 / t1101_limit_per_sec as f32);
            if wait_duration > elapsed {
                std::thread::sleep(wait_duration - elapsed);
            }
        }
    });

    let t1764_layout = layout_tbl.get("t1764").unwrap().to_owned();

    let t1764_loop = std::thread::spawn(move || {
        let req_data = Data {
            tr_code: "t1764".into(),
            data_type: DataType::Input,
            blocks: hashmap! {
                "t1764InBlock" => Block::Block(hashmap! {
                    "shcode" => "096530",
                    "gubun1" => "0",
                }),
            },
        };

        for i in 0..=20 * t1764_limit_per_sec {
            let res = loop {
                match xingapi::request(&req_data, &t1764_layout, None, Duration::from_secs(30)) {
                    Err(Error::XingApi { code: -21, .. }) => {
                        println!("t1764: limit reached");
                        std::thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    result => break result,
                }
            }
            .unwrap();

            assert!(res.is_ok());

            let elapsed = res.elapsed();
            println!("t1764: index: {}, elapsed: {} ms", i, elapsed.as_millis());

            let wait_duration = Duration::from_secs_f32(1.0 / t1764_limit_per_sec as f32);
            if wait_duration > elapsed {
                std::thread::sleep(wait_duration - elapsed);
            }
        }
    });

    t1101_loop.join().unwrap();
    t1764_loop.join().unwrap();

    xingapi::disconnect();
    println!("server disconnected");

    xingapi::loader::unload();
    println!("xingapi unloaded");
}
