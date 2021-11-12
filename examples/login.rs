// SPDX-License-Identifier: MIT

#![cfg(windows)]

use clap::{App, Arg};
use std::time::Duration;
use xingapi::Response;

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

    xingapi::loader::load().unwrap();
    println!("xingapi loaded");

    print_connection_info();

    xingapi::connect(addr, 20001, Duration::from_secs(10)).unwrap();
    println!("server connected");

    let res = xingapi::login(id, pw, cert_pw, false).unwrap();
    if res.is_ok() {
        println!("login succeed");
    } else {
        panic!("login failed: {:?}", res);
    }

    print_connection_info();

    xingapi::disconnect();
    println!("server disconnected");

    xingapi::loader::unload();
    println!("xingapi unloaded");
}

fn print_connection_info() {
    println!("connection info:");

    let accounts = xingapi::accounts();
    if !accounts.is_empty() {
        println!("├── accounts:");

        let mut iter = accounts.iter().peekable();
        while let Some(acc) = iter.next() {
            if iter.peek().is_some() {
                println!("│   ├── {:?}", acc);
            } else {
                println!("│   └── {:?}", acc);
            }
        }
    }

    println!("├── comm_media: {:?}", xingapi::comm_media());
    println!("├── etk_media: {:?}", xingapi::etk_media());
    println!("├── server_name: {:?}", xingapi::server_name());
    println!("├── is_future_allowed: {:?}", xingapi::is_future_allowed());
    println!("└── is_fx_allowed: {:?}", xingapi::is_fx_allowed());
}
