use crate::Exec;
use regex::Regex;
use std::path::Path;

pub mod cc;

#[derive(Debug, PartialEq)]
pub enum CompilerAction {
    Compile,
    EmitAsm,
    Link,
    Other,
}

impl CompilerAction {
    // args from compiler invocation
    pub fn from(args: &Vec<String>) -> Self {
        lazy_static! {
            static ref LINKING_ARG: Regex = Regex::new(r"^-(l|L|Wl,).+").unwrap();
        }

        for a in args {
            if LINKING_ARG.is_match(a) {
                return CompilerAction::Link;
            } else if a == "-S" {
                return CompilerAction::EmitAsm;
            } else if a == "-c" {
                return CompilerAction::Compile;
            }
        }
        CompilerAction::Other
    }
}

#[derive(Debug, PartialEq)]
pub enum ToolKind {
    CCompiler(CompilerAction),
    CXXCompiler(CompilerAction),
    CompilerWrapper,
    Linker,
    Archiver,
    Unknown,
}

impl ToolKind {
    pub fn from(e: &Exec) -> Self {
        lazy_static! {
            static ref ICC: Regex = Regex::new(r"^i?cc$").unwrap();
            static ref GCC: Regex = Regex::new(r"^([^-]*-)*[mg]cc(-?\d+(\.\d+){0,2})?$").unwrap();
            static ref XLC: Regex = Regex::new(r"^g?xlc$").unwrap();
            static ref CLANG: Regex = Regex::new(r"^([^-]*-)*clang(-\d+(\.\d+){0,2})?$").unwrap();
        }
        let path = Path::new(&e.path);
        let file = path.file_name().unwrap().to_str().unwrap();

        if GCC.is_match(file) || CLANG.is_match(file) || ICC.is_match(file) || XLC.is_match(file) {
            let action = CompilerAction::from(&e.args);
            let is_c_plus_plus = file.ends_with("++"); // TODO: likely too crude
            if is_c_plus_plus {
                return ToolKind::CXXCompiler(action);
            } else {
                return ToolKind::CCompiler(action);
            }
        }

        ToolKind::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toolkind_from() {
        assert_eq!(ToolKind::from("/usr/bin/cc"), ToolKind::CCompiler);
        assert_eq!(ToolKind::from("/usr/bin/icc"), ToolKind::CCompiler);
        assert_eq!(ToolKind::from("/usr/bin/gcc"), ToolKind::CCompiler);
        assert_eq!(ToolKind::from("/usr/bin/clang"), ToolKind::CCompiler);
    }
}
