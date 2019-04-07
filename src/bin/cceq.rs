use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::exit;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;
use serde_json::Result;

#[macro_use(crate_version, crate_authors)]
extern crate clap;
use clap::{App, AppSettings, Arg, ArgMatches};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
struct CompareError {
    details: String,
}

impl CompareError {
    fn new(msg: &str) -> CompareError {
        CompareError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for CompareError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for CompareError {
    fn description(&self) -> &str {
        &self.details
    }
}

include!("../tools/ccmd.rs");

fn read_json(filename: &Path) -> Result<Vec<CompileCmd>> {
    let contents = read_to_string(filename).expect("Something went wrong reading the file");

    serde_json::from_str(&contents)
}

//fn write_json(output_path: &Path, output: String) -> Option<PathBuf> {
//    let mut file = match File::create(&output_path) {
//        Ok(file) => file,
//        Err(e) => panic!("Unable to open file for writing: {}", e),
//    };
//    match file.write_all(output.as_bytes()) {
//        Ok(()) => (),
//        Err(e) => panic!("Unable to write translation to file: {}", e),
//    };
//
//    Some(PathBuf::from(output_path))
//}

fn compare_cmds(
    mut ref_cmds: Vec<CompileCmd>,
    mut tst_cmds: Vec<CompileCmd>
) -> std::result::Result<(), CompareError> {
    unimplemented!();
//    Ok(())
}

fn run_app() -> std::result::Result<(), CompareError> {
    let matches = App::new("cceq")
        .version(crate_version!())
        .author(crate_authors!(", "))
        .about("compares two compile_commands.json files")
        .arg(
            Arg::with_name("REFINPUT")
                .help("reference compile commands file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("TSTINPUT")
                .help("test  compile commands file")
                .required(true)
                .index(2),
        )
        .get_matches();

    // unwraps will succeed because the arguments are required
    let ref_json = Path::new(matches.value_of("REFINPUT").unwrap());
    let tst_json = Path::new(matches.value_of("TSTINPUT").unwrap());

    //    println!("files {:?} & {:?}", ref_json, tst_json);

    let ref_cmds = read_json(&ref_json).map_err(|e| CompareError::new(e.description()))?;
    let tst_cmds = read_json(&tst_json).map_err(|e| CompareError::new(e.description()))?;

    if ref_cmds.len() != tst_cmds.len() {
        return Err(CompareError::new(
            "reference and test inputs differ in number of commands.",
        ));
    }

    compare_cmds(ref_cmds, tst_cmds)?;

    Ok(())
}

fn main() {
    exit(match run_app() {
        Ok(_) => 0,
        Err(err) => {
            eprintln!("error: {}", err);
            1
        }
    });
}
