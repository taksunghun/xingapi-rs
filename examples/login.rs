// SPDX-License-Identifier: MIT

// 서버 연결 및 로그인 하는 예제입니다.

use clap::{App, Arg};
use xingapi::{response::Message, XingApi};

fn main() {
    let matches = App::new("login")
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

    println!("accounts:");
    xingapi.accounts().iter().for_each(|acc| println!("  {:?}", acc));

    println!("client_ip: {:?}", xingapi.client_ip());
    println!("server_name: {:?}", xingapi.server_name(),);
    println!("api_path: {:?}", xingapi.path());

    xingapi.disconnect();
    println!("server disconnected");

    assert!(!xingapi.is_connected());
}
