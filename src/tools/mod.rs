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
            // C compiler patterns used in intercept-build
            static ref ICC: Regex = Regex::new(r"^i?cc$").unwrap();
            static ref GCC: Regex = Regex::new(r"^([^-]*-)*[mg]cc(-?\d+(\.\d+){0,2})?$").unwrap();
            static ref XLC: Regex = Regex::new(r"^g?xlc$").unwrap();
            static ref CLANG: Regex = Regex::new(r"^([^-]*-)*clang(-\d+(\.\d+){0,2})?$").unwrap();

            // C++ compiler patterns used in intercept-build
            static ref CPP: Regex = Regex::new(r"^(c\+\+|cxx|CC)$").unwrap();
            static ref GPP: Regex = Regex::new(r"^([^-]*-)*[mg]\+\+(-\d+(\.\d+){0,2})?$").unwrap();
            static ref CLANGPP: Regex = Regex::new(r"^([^-]*-)*clang\+\+(-\d+(\.\d+){0,2})?$").unwrap();
            static ref ICPC: Regex = Regex::new(r"^icpc$").unwrap();
            static ref XLCPP: Regex = Regex::new(r"^g?xl(C|c\+\+)$").unwrap();

            // Linker and wrapper patterns used in intercept-build
            static ref LD: Regex = Regex::new(r"^ld(\.(bfd|gold))?$").unwrap();
            static ref CC_WRAPPER: Regex = Regex::new(r"^(distcc|ccache)$").unwrap();
            static ref CC_MPI_WRAPPER: Regex = Regex::new(r"^mpi(cc|cxx|CC|c\+\+)$").unwrap();
        }
        let path = Path::new(&e.path);
        let file = path.file_name().unwrap().to_str().unwrap();

        if GCC.is_match(file) || CLANG.is_match(file) || ICC.is_match(file) || XLC.is_match(file) {
            let action = CompilerAction::from(&e.args);
            return ToolKind::CCompiler(action);
        } else if CPP.is_match(file)
            || GPP.is_match(file)
            || CLANGPP.is_match(file)
            || ICPC.is_match(file)
            || XLCPP.is_match(file)
        {
            let action = CompilerAction::from(&e.args);
            return ToolKind::CXXCompiler(action);
        } else if LD.is_match(file) {
            return ToolKind::Linker;
        } else if file == "ar" {
            return ToolKind::Archiver;
        } else if CC_WRAPPER.is_match(file) || CC_MPI_WRAPPER.is_match(file) {
            return ToolKind::CompilerWrapper;
        }

        ToolKind::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Exec {
        pub fn mock(path: &str, args: &[&str]) -> Self {
            let path = path.to_owned();
            let env = vec![];
            let args = args.iter().map(|s| s.to_string()).collect::<Vec<String>>();
            let retcode = 0;
            Exec {
                path,
                args,
                env,
                retcode,
            }
        }
    }

    #[test]
    fn test_toolkind_from() {
        let cc_paths = &[
            "/usr/bin/cc",
            "/usr/bin/icc",
            "/usr/bin/gcc",
            "/usr/bin/clang",
        ];
        for cc in cc_paths {
            assert_eq!(
                ToolKind::from(&Exec::mock(cc, &["-c"])),
                ToolKind::CCompiler(CompilerAction::Compile)
            );
        }

        let cxx_paths = &[
            "/usr/bin/c++",
            "/usr/bin/g++",
            "/usr/bin/clang++",
            "/usr/bin/xlc++",
        ];
        for cxx in cxx_paths {
            assert_eq!(
                ToolKind::from(&Exec::mock(cxx, &["-c"])),
                ToolKind::CXXCompiler(CompilerAction::Compile)
            );
        }
    }
}
