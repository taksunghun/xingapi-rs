// SPDX-License-Identifier: MIT

// TR을 요청하는 예제입니다.

use clap::Clap;
use xingapi::{
    data::{Block, Data, DataType},
    hashmap,
    response::Message,
    XingApi,
};

#[derive(Clap)]
struct Opts {
    #[clap(short)]
    id: String,
    #[clap(short)]
    pw: String,
}

fn main() {
    let opts = Opts::parse();
    let xingapi = XingApi::new().unwrap();

    xingapi.connect("demo.ebestsec.co.kr", 20001, None, None).unwrap();
    println!("server connected");

    let login = xingapi.login(&opts.id, &opts.pw, "", false).unwrap();
    println!("login: {:?}", login);
    assert!(login.is_ok());

    let data = Data {
        code: "t8425".into(),
        data_type: DataType::Input,
        blocks: hashmap! {
            "t8425InBlock" => Block::Block(hashmap! {
                "dummy" => "",
            }),
        },
    };

    let res = xingapi.request(&data, None, None).unwrap();

    for block in res.data().unwrap().blocks["t8425OutBlock"].as_array().unwrap() {
        println!("tmcode: {:0>4}, tmname: {}", block["tmcode"], block["tmname"]);
    }

    xingapi.disconnect();
    println!("server disconnected");

    assert_eq!(xingapi.is_connected(), false);
}
