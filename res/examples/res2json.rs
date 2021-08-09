// SPDX-License-Identifier: MIT

use clap::{App, Arg};

use std::fs::{self, OpenOptions};
use std::{error::Error, path::Path};

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new("res2json")
        .arg(Arg::with_name("input").short("i").takes_value(true))
        .arg(Arg::with_name("output").short("o").required(true).takes_value(true))
        .arg(Arg::with_name("pretty").short("p"))
        .get_matches();

    let input = matches.value_of("input").map(|i| Path::new(i));
    let output = matches.value_of("output").map(|i| Path::new(i)).unwrap();
    let pretty = matches.is_present("pretty");

    let tr_layouts = if let Some(path) = input {
        xingapi_res::load_from_path(path)?
    } else {
        xingapi_res::load()?
    };

    println!("loaded: {}", tr_layouts.len());

    let file = OpenOptions::new().write(true).create_new(true).open(output)?;
    if pretty {
        serde_json::to_writer_pretty(&file, &tr_layouts)?;
    } else {
        serde_json::to_writer(&file, &tr_layouts)?;
    }

    println!("json encoded: \"{}\"", fs::canonicalize(output)?.display());

    Ok(())
}
