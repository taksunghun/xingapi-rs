// SPDX-License-Identifier: MIT

use clap::{App, Arg};
use std::fs::OpenOptions;

fn main() {
    let matches = App::new("res2json")
        .arg(Arg::with_name("input").short("i").takes_value(true))
        .arg(Arg::with_name("output").short("o").required(true).takes_value(true))
        .arg(Arg::with_name("pretty").short("p"))
        .get_matches();

    let input = matches.value_of("input");
    let output = matches.value_of("output").unwrap();
    let pretty = matches.is_present("pretty");

    let tr_layouts = if let Some(path) = input {
        xingapi_res::load_from_path(path)
    } else {
        xingapi_res::load()
    }
    .unwrap();

    println!("loaded: {}", tr_layouts.len());

    let file = OpenOptions::new().write(true).create_new(true).open(output).unwrap();
    if pretty {
        serde_json::to_writer_pretty(&file, &tr_layouts).unwrap();
    } else {
        serde_json::to_writer(&file, &tr_layouts).unwrap();
    }

    println!("json dumped: {}", output);
}
