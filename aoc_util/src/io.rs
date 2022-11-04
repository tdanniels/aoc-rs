use crate::errors::{failure, AocResult};

use std::env;
use std::path::Path;

pub fn get_cli_arg() -> AocResult<String> {
    let mut args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        return failure(format!("Bad CLI args: {:?}", args));
    }
    Ok(args.pop().unwrap())
}

pub fn get_input_file(codefile: &str) -> AocResult<String> {
    get_data_file(codefile, "input")
}

pub fn get_test_file(codefile: &str) -> AocResult<String> {
    get_data_file(codefile, "test")
}

fn get_data_file(codefile: &str, kind: &str) -> AocResult<String> {
    let stem = Path::new(codefile)
        .file_stem()
        .ok_or(format!("No stem for {codefile}?"))?;
    let datafile = "data/".to_string()
        + stem
            .to_str()
            .ok_or(format!("OsStr {stem:?} -> str failed?"))?
        + "_"
        + kind
        + ".txt";
    Ok(datafile)
}
