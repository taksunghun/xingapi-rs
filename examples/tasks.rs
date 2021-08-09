// SPDX-License-Identifier: MIT

// 시간당 요청 제한 횟수에 맞춰 여러 가지 TR을 동시에 요청하는 예제입니다.

use clap::{App, Arg};
use std::time::Duration;

use xingapi::data::{Block, Data, DataType};
use xingapi::{error::ErrorKind, hashmap, response::Message, XingApi};

fn main() {
    let matches = App::new("tasks")
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

    let t1101_one_sec_limit = xingapi.limit_per_one_sec("t1101") as u64;
    let t1101_ten_min_limit = xingapi.limit_per_ten_min("t1101") as u64;
    println!("t1101 one_sec_limit: {}", t1101_one_sec_limit);
    println!("t1101 ten_min_limit: {}", t1101_ten_min_limit);

    let t1764_one_sec_limit = xingapi.limit_per_one_sec("t1764") as u64;
    let t1764_ten_min_limit = xingapi.limit_per_ten_min("t1764") as u64;
    println!("t1764 one_sec_limit: {}", t1764_one_sec_limit);
    println!("t1764 ten_min_limit: {}", t1764_ten_min_limit);

    let xingapi_clone = xingapi.clone();

    let t1101_loop = std::thread::spawn(move || {
        let xingapi = xingapi_clone;

        let data = Data {
            code: "t1101".into(),
            data_type: DataType::Input,
            blocks: hashmap! {
                "t1101InBlock" => Block::Block(hashmap! {
                    "shcode" => "078020",
                }),
            },
        };

        for i in 0..20 * t1101_one_sec_limit {
            let res = loop {
                let result = xingapi.request(&data, None, None);
                match &result {
                    Ok(_) => {}
                    Err(err) => {
                        if err.kind() == ErrorKind::LimitReached {
                            println!("t1101: limit reached");
                            std::thread::sleep(Duration::from_millis(5));
                            continue;
                        }
                    }
                }

                break result;
            }
            .unwrap();

            assert!(res.is_ok());

            let elapsed = res.elapsed();
            println!("t1101: index: {}, elapsed: {} ms", i, elapsed.as_millis());

            let wait_duration = Duration::from_secs_f32(1.0 / t1101_one_sec_limit as f32);
            if wait_duration > elapsed {
                std::thread::sleep(wait_duration - elapsed);
            }
        }
    });

    let xingapi_clone = xingapi.clone();

    let t1764_loop = std::thread::spawn(move || {
        let xingapi = xingapi_clone;

        let data = Data {
            code: "t1764".into(),
            data_type: DataType::Input,
            blocks: hashmap! {
                "t1764InBlock" => Block::Block(hashmap! {
                    "shcode" => "096530",
                    "gubun1" => "0",
                }),
            },
        };

        for i in 0..20 * t1764_one_sec_limit + 1 {
            let res = loop {
                let result = xingapi.request(&data, None, None);
                match &result {
                    Ok(_) => {}
                    Err(err) => {
                        if err.kind() == ErrorKind::LimitReached {
                            println!("t1764: limit reached");
                            std::thread::sleep(Duration::from_millis(5));
                            continue;
                        }
                    }
                }

                break result;
            }
            .unwrap();

            assert!(res.is_ok());
            println!("t1764: index: {}, elapsed: {} ms", i, res.elapsed().as_millis());

            let wait_duration = Duration::from_secs_f32(1.0 / t1764_one_sec_limit as f32);
            std::thread::sleep(wait_duration);
        }
    });

    t1101_loop.join().unwrap();
    t1764_loop.join().unwrap();

    xingapi.disconnect();
    println!("server disconnected");

    assert!(!xingapi.is_connected());
}
