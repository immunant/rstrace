use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;

use regex::Regex;
use serde_json::Result;

use crate::tools::{CompilerAction, ToolKind};
use crate::Exec;
use std::path::Path;

include!("ccmd.rs");

impl CompileCmd {
    fn from(e: Exec, t: ToolKind) -> Self {
        let path = &e.env
            .iter()
            .find(|(k, _v)| k == "PWD")
            .unwrap().1;
        let (mut arguments, file) = filter_args(e.args);
        arguments[0] = match t {
            ToolKind::CCompiler(_) => "cc".to_owned(),
            ToolKind::CXXCompiler(_) => "c++".to_owned(),
            _ => panic!(),
        };
        arguments.insert(1, "-c".to_owned());

        // TODO: we need something like python's normpath which is not merged yet
        // https://github.com/rust-lang/rust/issues/59117
//        file_path = Path.new(file.unwrap());
//        path_path = Path.new(path);
//        // we need
//        let file = if file_path.is_absolute() {
//            file.unwrap();
//        } else {
//
//        }

        CompileCmd {
            directory: path.to_string(),
            file: file.unwrap(),
            command: None,
            arguments,
            output: None, // TODO: should use this field
        }
    }
}

#[allow(unused_must_use)]
fn filter_args(args: Vec<String>) -> (Vec<String>, Option<String>) {
    lazy_static! {
        static ref IGNORED_FLAGS: HashMap<&'static str, u8> = {
            let mut s = HashMap::new();
            // ignored because we will set it explicitly
            // for compatibility with intercept build.
            s.insert("-c", 0);
            // preprocessor macros
            s.insert("-MD", 0);
            s.insert("-MMD", 0);
            s.insert("-MG", 0);
            s.insert("-MP", 0);
            s.insert("-MF", 1);
            s.insert("-MT", 1);
            s.insert("-MQ", 1);
            // linker options
            s.insert("-static", 0);
            s.insert("-shared", 0);
            s.insert("-s", 0);
            s.insert("-rdynamic", 0);
            s.insert("-l", 1);
            s.insert("-L", 1);
            s.insert("-u", 1);
            s.insert("-z", 1);
            s.insert("-T", 1);
            s.insert("-Xlinker", 1);
            s
        };
        static ref FILE: Regex = Regex::new(r"^[^-].+").unwrap();
    }
    let mut args = args.iter();
    let mut file = None;
    let mut filtered: Vec<String> = vec![];
    while let Some(arg) = args.next() {
        let value = IGNORED_FLAGS.get::<str>(&arg.to_string());
        if let Some(&n) = value {
            (&mut args).skip(n as usize);
        } else if arg == "-D" || arg == "-I" {
            (&mut args).skip(1);
        } else {
            filtered.push(arg.to_string());
            if FILE.is_match(arg) {
                file = Some(arg.to_string());
            }
        }
    }
    (filtered, file)
}

pub fn filter_execs(e: Exec) -> Option<(Exec, ToolKind)> {
    lazy_static! {
        static ref NOT_COMPILING: HashSet<&'static str> = {
            let mut s = HashSet::new();
            s.insert("-E");
            s.insert("-cc1");
            s.insert("-cc1as");
            s.insert("-M");
            s.insert("-MM");
            s.insert("-###");
            s
        };
    }

    let tk = ToolKind::from(&e);
    match tk {
        ToolKind::CCompiler(ref a) | ToolKind::CXXCompiler(ref a)
            if a == &CompilerAction::Compile =>
        {
            Some((e, tk))
        }
        _ => None,
    }
}

pub fn write_compile_commands(v: Vec<(Exec, ToolKind)>) -> Result<()> {
    let mut cmds = vec![];
    for (e, t) in v {
        cmds.push(CompileCmd::from(e, t));
    }

    // Serialize it to a JSON string.
    let json = serde_json::to_string_pretty(&cmds)?;

    let mut file = match File::create("compile_commands.json") {
        Ok(file) => file,
        Err(e) => panic!("Unable to open file for writing: {}", e),
    };
    match file.write_all(json.as_bytes()) {
        Ok(()) => (),
        Err(e) => panic!("Unable to write compile_commands.json: {}", e),
    };

    Ok(())
}
