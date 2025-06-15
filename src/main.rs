mod nsp;

use core::str;
use std::{io, path::PathBuf, process::exit, str::FromStr};

use rouille::Response;

use crate::nsp::{Cnmt, Nsp, ParsingError};

fn calm_exit(e: &str) -> ! {
    println!("{e}");
    exit(-1)
}

const MAX_CLIENTS: usize = 1;

fn main() {
    let mut dir = std::env::args().skip(1);
    let Some(first) = dir.by_ref().next() else {
        calm_exit("no dirs/files specified")
    };

    let mut nsp_list = vec![];

    for d in std::iter::once(first).chain(dir) {
        match std::fs::read_dir(&d) {
            Ok(dir_list) => nsp_list.extend(
                dir_list
                    .map(|f| {
                        f.map_err(ParsingError::from).and_then(|f| {
                            Nsp::from_file(f.path()).inspect_err(|e| println!("{e:?}"))
                        })
                    })
                    .flatten(),
            ),
            Err(e) if e.kind() == io::ErrorKind::NotADirectory => {
                let Ok(d_f) = PathBuf::from_str(&d);
                if let Ok(f) = Nsp::from_file(d_f).inspect_err(|e| println!("{e:?}")) {
                    nsp_list.push(f);
                }
            }
            Err(e) => {
                println!("{d} err: {e:?}");
                continue;
            }
        };
    }

    let mut response_str = "[".to_string();
    for Nsp {
        name,
        file_size,
        cnmt: Cnmt {
            title_id, version, ..
        },
        ..
    } in &nsp_list
    {
        response_str.push_str(&format!(
            r#"{{"id": "{title_id}", "name": "{name}", "size": {file_size}, "version": {version}}},"#
        ));
    }
    response_str.replace_range(response_str.len() - 1.., "]");

    println!("Serving {nsp_list:?}\n{response_str}");

    rouille::start_server("0.0.0.0:9000", move |req| {
        println!("{}", req.url());
        match req.url().as_str() {
            "/api/search" => (),
            _ => (),
        };

        println!(
            "{}",
            r#"
        [{"id": "0000000000000000", "name": "hbl [0000000000000000].nsp", "size": 41896, "version": 0}, {"id": "01008E20047DC000", "name": "Snipperclips Plus - Cut It Out, Together [01008E20047DC000][JP][v0].nsp", "size": 1262965600, "version": 0}]
        "#
        );
        println!("{response_str}");
        Response::text(
            r#"
        [{"id": "0000000000000000", "name": "hbl [0000000000000000].nsp", "size": 41896, "version": 0}, {"id": "01008E20047DC000", "name": "Snipperclips Plus - Cut It Out, Together [01008E20047DC000][JP][v0].nsp", "size": 1262965600, "version": 0}]
        "#,
        )
    })
}
