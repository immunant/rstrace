use crate::Exec;
use serde_json::Result;
use std::fs::File;
use std::io::Write;

#[derive(Deserialize, Serialize, Debug)]
struct CompileCmd {
    /// The working directory of the compilation. All paths specified in the command
    /// or file fields must be either absolute or relative to this directory.
    directory: String,
    /// The main translation unit source processed by this compilation step. This is
    /// used by tools as the key into the compilation database. There can be multiple
    /// command objects for the same file, for example if the same source file is compiled
    /// with different configurations.
    file: String,
    /// The compile command executed. After JSON unescaping, this must be a valid command
    /// to rerun the exact compilation step for the translation unit in the environment
    /// the build system uses. Parameters use shell quoting and shell escaping of quotes,
    /// with ‘"’ and ‘\’ being the only special characters. Shell expansion is not supported.
    command: Option<String>,
    /// The compile command executed as list of strings. Either arguments or command is required.
    #[serde(default)]
    arguments: Vec<String>,
    /// The name of the output created by this compilation step. This field is optional. It can
    /// be used to distinguish different processing modes of the same input file.
    output: Option<String>,
}

impl CompileCmd {
    fn from(e: Exec) -> Self {
        let path = &e.env.iter().find(|(k, _v)| k == "PWD").unwrap().1;
        let mut arguments = e.args;
        arguments[0] = "cc".to_owned(); // TODO: is this correct for C++? I think not.

        CompileCmd {
            directory: path.to_string(),
            file: "(not implemented)".to_owned(),
            command: None,
            arguments,
            output: None, // TODO: should use this field
        }
    }
}

pub fn write_compile_commands(v: Vec<Exec>) -> Result<()> {
    let mut cmds = vec![];
    for e in v {
        cmds.push(CompileCmd::from(e));
    }

    // Serialize it to a JSON string.
    let json = serde_json::to_string_pretty(&cmds)?;

    // Print, write to a file, or send to an HTTP server.
    //    println!("{:#?}", json);

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
