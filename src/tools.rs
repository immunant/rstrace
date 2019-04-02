use regex::Regex;
use std::path::Path;

#[derive(Debug, PartialEq)]
pub enum ToolKind {
    CCompiler,
    CXXCompiler,
    CompilerWrapper,
    CCompilerAsLinker,
    CXXCompilerAsLinker,
    Linker,
    Archiver,
    Unknown,
}

lazy_static! {
    static ref ICC: Regex = Regex::new(r"^i?cc$").unwrap();
    static ref GCC: Regex = Regex::new(r"^([^-]*-)*[mg]cc(-?\d+(\.\d+){0,2})?$").unwrap();
    static ref XLC: Regex = Regex::new(r"^g?xlc$").unwrap();
    static ref CLANG: Regex = Regex::new(r"^([^-]*-)*clang(-\d+(\.\d+){0,2})?$").unwrap();
}

impl ToolKind {
    pub fn from(path: &str) -> Self {
        let path = Path::new(path);
        let file = path.file_name().unwrap().to_str().unwrap();

        if GCC.is_match(file) || CLANG.is_match(file) || ICC.is_match(file) || XLC.is_match(file) {
            return ToolKind::CCompiler;
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
