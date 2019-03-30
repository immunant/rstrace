use std::process::{exit, Command};

extern crate clap;
use clap::{App};

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

fn run_app() -> Result<(), &'static str> {

    if !cfg!(unix) {
        return Err("cctrace only runs on Unix hosts")
    }

    let strace_path = locate_strace()?;

    // Application logic here
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


