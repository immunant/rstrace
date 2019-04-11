use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use regex::Regex;
use serde_json::Result;

use crate::tools::{CompilerAction, ToolKind};
use crate::Exec;

include!("ccmd.rs");

impl CompileCmd {
    fn try_from(e: Exec, t: ToolKind) -> Option<Self> {
        let path = &e.env
            .iter()
            .find(|(k, _v)| k == "PWD")
            .unwrap().1;
        let (mut arguments, file) =
            filter_args(e.args);
        if file.is_none() {
            return None;
        }

        arguments[0] = match t {
            ToolKind::CCompiler(_) => "cc".to_owned(),
            ToolKind::CXXCompiler(_) => "c++".to_owned(),
            _ => panic!(),
        };
        arguments.insert(1, "-c".to_owned());

        Some(CompileCmd {
            directory: path.to_string(),
            file: file.unwrap(),
            command: None,
            arguments,
            output: None, // TODO: should use this field
        })
    }
}

fn is_source(file: &str) -> bool {
    lazy_static! {
        static ref SRC_EXT: HashSet<&'static str> = {
            let mut s = HashSet::new();
            s.insert("c");
            s.insert("i");
            s.insert("ii");
            s.insert("m");
            s.insert("mm");
            s.insert("mii");
            s.insert("C");
            s.insert("cc");
            s.insert("CC");
            s.insert("cp");
            s.insert("cpp");
            s.insert("cxx");
            s.insert("c++");
            s.insert("C++");
            s.insert("t++");
            s.insert("txx");
            // don't trace assembly to match intercept-build
            // s.insert("s");
            // s.insert("S");
            // s.insert("sx");
            // s.insert("asm");
            s
        };
    }

    match Path::new(file).extension() {
        Some(ext) => SRC_EXT.get(ext.to_str().unwrap()).is_some(),
        None => false
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
            filtered.push(arg.to_string());
            filtered.push(args.next().unwrap().to_string());
        } else {
            if FILE.is_match(arg) && is_source(arg) {
                // chop off leading ./ to match output of intercept-build
                let f = arg.to_string();
                let f = if f.starts_with("./") { f[2..].to_owned() } else { f };
                filtered.push(f.clone());
                file = Some(f);
            } else {
                filtered.push(arg.to_string());
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
        let cmd = CompileCmd::try_from(e, t);
        if cmd.is_some() {
            cmds.push(cmd.unwrap());
        }
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
