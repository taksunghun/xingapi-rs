// SPDX-License-Identifier: MIT

// TR을 요청하는 예제입니다.

use clap::Clap;
use xingapi::{
    data::{Data, DataType},
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

#[async_std::main]
async fn main() {
    let opts = Opts::parse();
    let xingapi = XingApi::new().await.unwrap();

    xingapi.connect("demo.ebestsec.co.kr", 20001, None, None).await.unwrap();
    println!("server connected");

    let login = xingapi.login(&opts.id, &opts.pw, "", false).await.unwrap();
    println!("login: {:?}", login);
    assert!(login.is_ok());

    let data = Data {
        code: "t8425".into(),
        data_type: DataType::Input,
        blocks: hashmap! {
            "t8425InBlock" => hashmap! {
                "dummy" => "",
            },
        },
        arr_blocks: hashmap! {},
    };

    let res = xingapi.request(&data, None, None).await.unwrap();

    for block in &res.data().unwrap().arr_blocks["t8425OutBlock"] {
        println!("tmcode: {:0>4}, tmname: {}", block["tmcode"], block["tmname"]);
    }

    xingapi.disconnect().await;
    println!("server disconnected");

    assert_eq!(xingapi.is_connected().await, false);
}
