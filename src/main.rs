use std::process::{exit, Command};
use std::path::{Path, PathBuf};

#[macro_use]
extern crate nom;
#[macro_use(crate_version, crate_authors)]
extern crate clap;
use clap::{App, Arg, AppSettings, ArgMatches};

extern crate tempfile;
use tempfile::tempdir;
use std::fs::File;
use std::io::{BufReader, BufRead};


mod parser;

fn locate_strace() -> Result<String, &'static str> {
    // get path to strace
    let mut which_strace = Command::new("which");
    let mut which_output = which_strace
        .arg("strace")
        .output()
        .expect("strace is not in path");
    assert!(which_output.status.success());

    // drop the trailing newline
    assert_eq!(Some('\n' as u8), which_output.stdout.pop());
    let strace_path = String::from_utf8_lossy(&which_output.stdout);

    // check that strace -V produces sane output
    let strace_ver_output = Command::new(strace_path.to_string())
        .arg("-V")
        .output()
        .expect("could not get strace version");
    assert!(strace_ver_output.status.success());
    let strace_ver = String::from_utf8_lossy(
        &strace_ver_output.stdout[0..17]
    );
    assert_eq!("strace -- version", strace_ver);

    Ok(strace_path.to_string())
}

fn process_output_file(file: PathBuf) -> Result<(), String> {

    let f = File::open(file).unwrap();
    let buf = BufReader::new(&f);
    for line in buf.lines() {
        let line = line.unwrap();
        parser::parseln(&line);
    }
    Ok(())
}

fn run_strace(args: ArgMatches, output_file: &str) -> Result<(), String> {
    let strace_args = vec![
        "-o", output_file,     // output to `$output_file.$pid`
        "-ff",                 // follow forks
        "-e", "trace=execve",  // only trace execve calls
        "-s", "8192",          // set max string length
        "-v"                   // request unabridged output
    ];
    // get the build command
    let cmd: Vec<&str> = args.values_of("cmd").unwrap().collect();
    let strace_path = locate_strace()?;

    let mut strace_child = Command::new(strace_path)
        .args(strace_args)
        .args(cmd)
        .spawn()
        .expect("failed to run strace");

    let output = strace_child
        .wait()
        .expect("strace didn't exit cleanly");

    let tmp_dir = Path::new(output_file).parent().unwrap();
    let _res = tmp_dir
        .read_dir()
        .unwrap()
        .map(|rde| {
            if let Ok(entry) = rde {
                if let Ok(file_type) = entry.file_type()  {
                    assert!(file_type.is_file());
                    return process_output_file(entry.path())
                }
            }
            // shouldn't happen since we strace to a pristine tempdir
            panic!("unexpected non-file entry in {}",
                   tmp_dir.to_str().unwrap());
        }).collect::<Vec<_>>();


    Ok(())
}

fn run_app() -> Result<(), String> {

    if !cfg!(unix) {
        return Err("cctrace only runs on Unix hosts".to_string())
    }

    let matches = App::new("cctrace")
        .version(crate_version!())
        .author(crate_authors!(", "))
        .about("traces C/C++ compiler and linker invocations")
        .setting(AppSettings::TrailingVarArg)
        .arg(Arg::from_usage("<cmd>... 'build command'"))
        .get_matches();

    {
        // Create a directory inside of `std::env::temp_dir()`
        let tmp_dir = tempdir().map_err(|e| format!("{}", e))?;
        let strace_outfile = tmp_dir
            .path()
            .join("cctrace.out");
        let strace_outfile = strace_outfile
            .to_str()
            .expect("failed to construct temporary output filename");

        run_strace(matches, strace_outfile)?;

        // `tmp_dir` goes out of scope, the directory will be deleted here.
        tmp_dir.close().map_err(|e| format!("{}", e))?;
    }

    Ok(())
}

fn main() {
    exit(match run_app() {
       Ok(_) => 0,
       Err(err) => {
           eprintln!("error: {:?}", err);
           1
       }
    });
}


