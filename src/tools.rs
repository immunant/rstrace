

pub enum ToolKind {
    Compiler,
    CompilerAsLinker,
    Linker,
    Archiver,
    Unknown,
}

impl ToolKind {
    pub fn from(path: &str) -> Self {
        match path {
            "/usr/bin/gcc" => ToolKind::Compiler,
            _ => ToolKind::Unknown
        }
    }
}