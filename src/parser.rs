use nom::types::CompleteStr;
use crate::Exec;

/// to combine nom parsing functions, they have to have
/// compatible return types, so they all return `Expr`.
#[derive(Debug,PartialEq)]
enum Expr {
    UInt(u8),
    Str(String),
    ArrOfStr(Vec<String>),
    ArrOfKeyVal(Vec<(String,String)>)
}

named!(string<CompleteStr, &str>,
    map!(
        delimited!(char!('"'), is_not!("\""), char!('"')),
        |cs| cs.0
    )
);
named!(string_expr<CompleteStr, Expr>,
    map!(string, |s| Expr::Str(s.to_string()))
);

#[cfg(test)]
const EMPTY: CompleteStr = CompleteStr("");

#[test]
fn test_string() {
    assert_eq!(
        string(CompleteStr("\"test\"")),
        Ok((EMPTY, "test"))
    );
    assert_eq!(
        string(CompleteStr("\"te\"st\"")),
        Ok((CompleteStr("st\""), "te"))
    );
    assert!(string(CompleteStr("\"\"")).is_err());
    assert!(string(CompleteStr("\"")).is_err());
}

named!(arr_of_str<CompleteStr, Vec<&str>>,
    delimited!(
        char!('['),
        separated_list!(
            tag!(", "),
            string
        ),
        char!(']')
    )
);
named!(arr_of_str_expr<CompleteStr, Expr>,
    map!(
        arr_of_str,
        |v| Expr::ArrOfStr(v
            .iter()
            .map(|s| String::from(*s))
            .collect::<Vec<String>>()
        )
    )
);

#[test]
fn test_arr_of_str() {
    assert_eq!(
        arr_of_str(CompleteStr("[]")),
        Ok((EMPTY, vec![]))
    );
    assert_eq!(
        arr_of_str(CompleteStr("[\"test\"]")),
        Ok((EMPTY, vec!["test"]))
    );
    assert_eq!(
        arr_of_str(CompleteStr("[\"test\", \"best\"]")),
        Ok((EMPTY, vec!["test", "best"]))
    );
}

fn from_dec(input: CompleteStr) -> Result<Expr, std::num::ParseIntError> {
    let val = u8::from_str_radix(&input, 10)?;
    Ok(Expr::UInt(val))
}

fn is_digit(c: char) -> bool {
  c.is_digit(10)
}

named!(retcode<CompleteStr, Expr>,
  map_res!(dbg_dmp!(take_while!(is_digit)), from_dec)
);

#[test]
fn test_retcode() {
    assert_eq!(retcode(CompleteStr("0")), Ok((EMPTY, Expr::UInt(0u8))));
}

named!(env_var<CompleteStr,(&str,&str)>,
    do_parse!(
                tag_s!("\"")                   >>
        key:    take_until_and_consume!("=")  >>
        value:  take_until_and_consume!("\"") >>
        ((&key, &value))
    )
);

#[test]
fn test_env_var() {
    assert_eq!(
        env_var(CompleteStr("\"key=value\"")),
        Ok((EMPTY, ("key", "value")))
    );
    assert_eq!(
        env_var(CompleteStr("\"key=value=value\"")),
        Ok((EMPTY, ("key", "value=value")))
    );
}

named!(arr_of_env_var<CompleteStr, Vec<(&str, &str)>>,
    delimited!(
        char!('['),
        separated_list!(
            tag!(", "),
            env_var
        ),
        char!(']')
    )
);

#[test]
fn test_arr_of_env_var() {
    assert_eq!(
        arr_of_env_var(CompleteStr("[]")),
        Ok((EMPTY, vec![]))
    );
    assert_eq!(
        arr_of_env_var(CompleteStr("[\"key=value\"]")),
        Ok((EMPTY, vec![("key", "value")]))
    );
}


named!(arr_of_env_var_expr<CompleteStr, Expr>,
    map!(
        arr_of_env_var,
        |v| Expr::ArrOfKeyVal(v
            .iter()
            .map(|s| (String::from(s.0), String::from(s.1)))
            .collect::<Vec<(String, String)>>()
        )
    )
);

named!(execve<CompleteStr, Exec>,
    do_parse!(
                tag_s!("execve(") >>
        path:   string_expr       >>
                tag_s!(", ") >>
        args:   arr_of_str_expr >>
                tag_s!(", ") >>
        env :   arr_of_env_var_expr >>
                tag_s!(") = ") >>
        retc:   retcode >>
        (
            if let (Expr::Str(path), Expr::ArrOfStr(args), Expr::ArrOfKeyVal(env), Expr::UInt(r)) =
                (path, args, env, retc) {
                Exec { path, args, env, retcode: r }
            } else { panic!() }
        )
    )
);

#[test]
fn test_execve() {
    assert_eq!(
        execve(CompleteStr("execve(\"/bin/ls\", [\"-la\"], []) = 0")),
        Ok((EMPTY, Exec {
            path:       "/bin/ls".to_string(),
            args:       vec!["-la".to_string()],
            env:        vec![],
            retcode:    0
        }))
    );
}

named!(footer<CompleteStr, Expr>,
    delimited!(tag_s!("+++ exited with "), retcode, tag_s!(" +++"))
);

#[test]
fn test_footer() {
    assert!(footer(CompleteStr("+++ exited with 0 +++")).is_ok());
    assert!(footer(CompleteStr("+++ exited with 255 +++")).is_ok());
    assert!(footer(CompleteStr("+++ exited with 1000 +++")).is_err());
}

named!(line<CompleteStr, Option<Exec>>,
    alt!(
        map!(footer, |_| None) |
        map!(execve, |e| Some(e))
    )
);

pub fn parseln(input: &str) -> Result<Option<Exec>, String> {
    let res = line(CompleteStr(input))
        .map_err(|_| format!("failed to parse:\n {}", input))?;

    if let Some(exec) = res.1 {
//        println!("{:?}", exec);
        return Ok(Some(exec));
    }

    Ok(None)
}
