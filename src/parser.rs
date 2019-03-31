use nom::types::CompleteStr;

#[derive(Debug,PartialEq)]
pub struct Exec {
    pub path: String,
}

#[derive(Debug,PartialEq)]
pub enum Expr {
    UInt(u8),
    Str(String),
    ArrOfStr(Vec<String>)
}

const empty: CompleteStr = CompleteStr("");

named!(string<CompleteStr, &str>,
    map!(
        delimited!(char!('"'), is_not!("\""), char!('"')),
        |cs| cs.0
    )
);
named!(string_expr<CompleteStr, Expr>,
    map!(string, |s| Expr::Str(s.to_string()))
);

#[test]
fn test_string() {
    assert_eq!(
        string(CompleteStr("\"test\"")),
        Ok((empty, "test"))
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
        Ok((empty, vec![]))
    );
    assert_eq!(
        arr_of_str(CompleteStr("[\"test\"]")),
        Ok((empty, vec!["test"]))
    );
    assert_eq!(
        arr_of_str(CompleteStr("[\"test\", \"best\"]")),
        Ok((empty, vec!["test", "best"]))
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
    assert_eq!(retcode(CompleteStr("0")), Ok((empty, Expr::UInt(0u8))));
}


named!(execve<CompleteStr, Exec>,
    do_parse!(
                tag_s!("execve(") >>
        path:   string_expr       >>
                tag_s!(", ") >>
        args:   arr_of_str_expr >>
                tag_s!(", ") >>
        env :   arr_of_str_expr >>
                tag_s!(") = ") >>
        retc:   retcode >>
        (
            if let Expr::Str(path) = path {
                Exec { path }
            } else { panic!() }
        )
    )
);

#[test]
fn test_execve() {
    assert_eq!(
        execve(CompleteStr("execve(\"/bin/ls\", [\"la\"], []) = 0")),
        Ok((empty, Exec { path: "/bin/ls".to_string() }))
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

pub fn parseln(input: &str) {
    println!("{:?}", input);
    let res = line(CompleteStr(input));
    println!("{:?}", res);
}
